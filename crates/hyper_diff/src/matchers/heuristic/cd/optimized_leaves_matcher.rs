use crate::{
    decompressed_tree_store::{
        ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, LazyDecompressed,
        LazyDecompressedTreeStore, LazyPOBorrowSlice, PostOrder, PostOrderIterable, Shallow,
        ShallowDecompressedTreeStore,
    },
    matchers::{
        heuristic::cd::iterator::{CustomIteratorConfig, CustomPostOrderIterator},
        mapping_store::MonoMappingStore,
    },
};
use ahash::RandomState;
use hyperast::nodes::TextSerializer;
use hyperast::types::{HyperAST, LabelStore, Labeled, NodeId, NodeStore, TypeStore, WithHashs};
use hyperast::{PrimInt, types::HyperType};
use std::fmt::Debug;
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
    hash::Hash,
};
use str_distance::DistanceMetric;

use super::OptimizedLeavesMatcherConfig;

/// A mapping candidate with similarity score for priority queue ordering
struct MappingWithSimilarity<M: MonoMappingStore> {
    src: M::Src,
    dst: M::Dst,
    sim: f64,
}

impl<M: MonoMappingStore> PartialEq for MappingWithSimilarity<M> {
    fn eq(&self, other: &Self) -> bool {
        self.sim == other.sim
    }
}

impl<M: MonoMappingStore> Eq for MappingWithSimilarity<M> {}

impl<M: MonoMappingStore> PartialOrd for MappingWithSimilarity<M> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.sim.partial_cmp(&other.sim)
    }
}

impl<M: MonoMappingStore> Ord for MappingWithSimilarity<M> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sim.partial_cmp(&other.sim).unwrap_or(Ordering::Equal)
    }
}

pub struct OptimizedLeavesMatcher<Dsrc, Ddst, HAST, M> {
    pub(super) stores: HAST,
    pub src_arena: Dsrc,
    pub dst_arena: Ddst,
    pub mappings: M,
    pub config: OptimizedLeavesMatcherConfig,
}

impl<
    Dsrc: LazyDecompressed<M::Src>,
    Ddst: LazyDecompressed<M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore,
> OptimizedLeavesMatcher<Dsrc, Ddst, HAST, M>
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
    M::Src: PrimInt,
    M::Dst: PrimInt,
    HAST::Label: Eq,
    HAST::IdN: Debug,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    <HAST::TS as TypeStore>::Ty: Hash + Eq + Clone + HyperType,
    Dsrc: DecompressedTreeStore<HAST, Dsrc::IdD, M::Src>
        + DecompressedWithParent<HAST, Dsrc::IdD>
        + PostOrder<HAST, Dsrc::IdD, M::Src>
        + PostOrderIterable<HAST, Dsrc::IdD, M::Src>
        + ContiguousDescendants<HAST, Dsrc::IdD, M::Src>
        + LazyPOBorrowSlice<HAST, Dsrc::IdD, M::Src>
        + ShallowDecompressedTreeStore<HAST, Dsrc::IdD, M::Src>
        + LazyDecompressedTreeStore<HAST, M::Src>,
    Ddst: DecompressedTreeStore<HAST, Ddst::IdD, M::Dst>
        + DecompressedWithParent<HAST, Ddst::IdD>
        + PostOrder<HAST, Ddst::IdD, M::Dst>
        + PostOrderIterable<HAST, Ddst::IdD, M::Dst>
        + ContiguousDescendants<HAST, Ddst::IdD, M::Dst>
        + LazyPOBorrowSlice<HAST, Ddst::IdD, M::Dst>
        + ShallowDecompressedTreeStore<HAST, Ddst::IdD, M::Dst>
        + LazyDecompressedTreeStore<HAST, M::Dst>,
    Dsrc::IdD: Clone + Hash + Eq,
    Ddst::IdD: Clone + Hash + Eq,
{
    /// Create matcher with custom configuration
    pub fn with_config(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
        config: OptimizedLeavesMatcherConfig,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M> {
        let mut matcher = Self {
            stores: mapping.hyperast,
            src_arena: mapping.mapping.src_arena,
            dst_arena: mapping.mapping.dst_arena,
            mappings: mapping.mapping.mappings,
            config,
        };

        matcher
            .mappings
            .topit(matcher.src_arena.len(), matcher.dst_arena.len());
        matcher.execute();

        crate::matchers::Mapper {
            hyperast: mapping.hyperast,
            mapping: crate::matchers::Mapping {
                src_arena: matcher.src_arena,
                dst_arena: matcher.dst_arena,
                mappings: matcher.mappings,
            },
        }
    }

    /// Create matcher with default optimized configuration
    pub fn match_it(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M> {
        Self::with_config(mapping, OptimizedLeavesMatcherConfig::default())
    }

    /// Execute the leaves matching with configured optimizations
    fn execute(&mut self) {
        if self.config.statement_level_iteration {
            self.execute_statement();
        } else if self.config.enable_type_grouping {
            self.execute_with_type_grouping();
        } else {
            self.execute_naive();
        }
    }

    /// Execute with type grouping optimization - only compare leaves of same type
    fn execute_with_type_grouping(&mut self) {
        // Pre-compute and cache label info (always when using type grouping for best performance)
        let mut label_cache: HashMap<(HAST::IdN, HAST::IdN), f64, RandomState> =
            HashMap::with_hasher(RandomState::new());

        // Create QGram object once if reuse is enabled
        let qgram = str_distance::QGram::new(3);

        println!("=== Testing Custom Iterator ===");

        // Collect leaves
        let dst_leaves: Vec<M::Dst> = self
            .dst_arena
            .iter_df_post::<true>()
            .filter(|t| {
                let id = self.dst_arena.decompress_to(&t);
                self.dst_arena.children(&id).is_empty()
            })
            .collect();

        let src_leaves: Vec<M::Src> = self
            .src_arena
            .iter_df_post::<true>()
            .filter(|t| {
                let id = self.src_arena.decompress_to(&t);
                self.src_arena.children(&id).is_empty()
            })
            .collect();

        // Group leaves by type and build label cache in single pass for optimal performance
        let mut src_leaves_by_type: HashMap<<HAST::TS as TypeStore>::Ty, Vec<M::Src>, RandomState> =
            HashMap::default();
        let mut dst_leaves_by_type: HashMap<<HAST::TS as TypeStore>::Ty, Vec<M::Dst>, RandomState> =
            HashMap::default();
        let mut src_label_cache: HashMap<M::Src, Option<(HAST::IdN, String)>, RandomState> =
            HashMap::default();
        let mut dst_label_cache: HashMap<M::Dst, Option<(HAST::IdN, String)>, RandomState> =
            HashMap::default();

        // Process source leaves: group by type and cache labels in single pass
        for src_leaf in &src_leaves {
            let src = self.src_arena.decompress_to(src_leaf);
            let original_src = self.src_arena.original(&src);
            let src_type = self.stores.resolve_type(&original_src);

            // Always cache labels when using type grouping for optimal performance
            let src_node = self.stores.node_store().resolve(&original_src);
            let src_label_entry = if let Some(src_label_id) = src_node.try_get_label() {
                let src_label = self.stores.label_store().resolve(&src_label_id).to_string();
                Some((original_src.clone(), src_label))
            } else {
                None
            };
            src_label_cache.insert(*src_leaf, src_label_entry);

            src_leaves_by_type
                .entry(src_type)
                .or_insert_with(Vec::new)
                .push(*src_leaf);
        }

        // Process destination leaves: group by type and cache labels in single pass
        for dst_leaf in &dst_leaves {
            let dst = self.dst_arena.decompress_to(dst_leaf);
            let original_dst = self.dst_arena.original(&dst);
            let dst_type = self.stores.resolve_type(&original_dst);

            // Always cache labels when using type grouping for optimal performance
            let dst_node = self.stores.node_store().resolve(&original_dst);
            let dst_label_entry = if let Some(dst_label_id) = dst_node.try_get_label() {
                let dst_label = self.stores.label_store().resolve(&dst_label_id).to_string();
                Some((original_dst.clone(), dst_label))
            } else {
                None
            };
            dst_label_cache.insert(*dst_leaf, dst_label_entry);

            dst_leaves_by_type
                .entry(dst_type)
                .or_insert_with(Vec::new)
                .push(*dst_leaf);
        }

        // Use appropriate collection type for mappings
        if self.config.use_binary_heap {
            let mut best_mappings = BinaryHeap::new();

            // Only compare leaves of the same type
            for (node_type, src_leaves) in src_leaves_by_type.iter() {
                if let Some(dst_leaves) = dst_leaves_by_type.get(node_type) {
                    for &src_leaf in src_leaves {
                        let src = self.src_arena.decompress_to(&src_leaf);

                        for &dst_leaf in dst_leaves {
                            let dst = self.dst_arena.decompress_to(&dst_leaf);

                            if !self.mappings.is_src(src.shallow())
                                && !self.mappings.is_dst(dst.shallow())
                            {
                                let sim = self.compute_cached_label_similarity(
                                    &src_leaf,
                                    &dst_leaf,
                                    &src,
                                    &dst,
                                    &mut label_cache,
                                    &src_label_cache,
                                    &dst_label_cache,
                                    &qgram,
                                );

                                if sim > self.config.base_config.label_sim_threshold {
                                    best_mappings.push(MappingWithSimilarity::<M> {
                                        src: src_leaf,
                                        dst: dst_leaf,
                                        sim,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            // Process mappings in order
            while let Some(mapping) = best_mappings.pop() {
                self.mappings
                    .link_if_both_unmapped(mapping.src, mapping.dst);
            }
        } else {
            let mut leaves_mappings: Vec<MappingWithSimilarity<M>> = Vec::new();

            // Only compare leaves of the same type
            for (node_type, src_leaves) in src_leaves_by_type.iter() {
                if let Some(dst_leaves) = dst_leaves_by_type.get(node_type) {
                    for &src_leaf in src_leaves {
                        let src = self.src_arena.decompress_to(&src_leaf);

                        for &dst_leaf in dst_leaves {
                            let dst = self.dst_arena.decompress_to(&dst_leaf);

                            if !self.mappings.is_src(src.shallow())
                                && !self.mappings.is_dst(dst.shallow())
                            {
                                let sim = self.compute_cached_label_similarity(
                                    &src_leaf,
                                    &dst_leaf,
                                    &src,
                                    &dst,
                                    &mut label_cache,
                                    &src_label_cache,
                                    &dst_label_cache,
                                    &qgram,
                                );

                                if sim > self.config.base_config.label_sim_threshold {
                                    leaves_mappings.push(MappingWithSimilarity {
                                        src: src_leaf,
                                        dst: dst_leaf,
                                        sim,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            // Sort mappings by similarity
            leaves_mappings.sort_by(|a, b| b.sim.partial_cmp(&a.sim).unwrap_or(Ordering::Equal));

            // Process mappings in order
            for mapping in leaves_mappings {
                self.mappings
                    .link_if_both_unmapped(mapping.src, mapping.dst);
            }
        }
    }

    /// Execute with statement level iteration
    fn execute_statement(&mut self) {
        let dst_leaves = self.collect_statement_leaves_dst();
        let src_leaves = self.collect_statement_leaves_src();

        let mut leaves_mappings: Vec<MappingWithSimilarity<M>> = Vec::new();

        if self.config.enable_label_caching {
            let mut src_text_cache = HashMap::new();
            let mut dst_text_cache = HashMap::new();

            for src in &src_leaves {
                let src_text = if let Some(text) = src_text_cache.get(&src) {
                    text
                } else {
                    let original_src = self.src_arena.original(&src);

                    let text = TextSerializer::new(&self.stores, original_src).to_string();
                    src_text_cache.insert(src, text);
                    src_text_cache.get(&src).unwrap()
                };
                for dst in &dst_leaves {
                    let dst_text = if let Some(text) = dst_text_cache.get(&dst) {
                        text
                    } else {
                        let original_dst = self.dst_arena.original(&dst);

                        let text = TextSerializer::new(&self.stores, original_dst).to_string();
                        dst_text_cache.insert(dst, text);
                        dst_text_cache.get(dst).unwrap()
                    };
                    // no need to check for equal types since all nodes are statements
                    let sim = 1.0
                        - str_distance::QGram::new(3)
                            .normalized(src_text.chars(), dst_text.chars());
                    if sim > self.config.base_config.label_sim_threshold {
                        leaves_mappings.push(MappingWithSimilarity {
                            src: src.shallow().clone(),
                            dst: dst.shallow().clone(),
                            sim,
                        });
                    }
                }
            }
        } else {
            for src in &src_leaves {
                let original_src = self.src_arena.original(&src);

                let src_text = TextSerializer::new(&self.stores, original_src).to_string();

                for dst in &dst_leaves {
                    let original_dst = self.dst_arena.original(&dst);

                    let dst_text = TextSerializer::new(&self.stores, original_dst).to_string();

                    // no need to check for equal types since all nodes are statements
                    let sim = 1.0
                        - str_distance::QGram::new(3)
                            .normalized(src_text.chars(), dst_text.chars());
                    if sim > self.config.base_config.label_sim_threshold {
                        leaves_mappings.push(MappingWithSimilarity {
                            src: src.shallow().clone(),
                            dst: dst.shallow().clone(),
                            sim,
                        });
                    }
                }
            }
        }

        // Sort mappings by similarity
        leaves_mappings.sort_by(|a, b| b.sim.partial_cmp(&a.sim).unwrap_or(Ordering::Equal));

        // Process mappings in order
        for mapping in leaves_mappings {
            self.mappings
                .link_if_both_unmapped(mapping.src, mapping.dst);
        }
    }

    /// Execute naive approach without type grouping - compare all leaves
    fn execute_naive(&mut self) {
        let dst_leaves = self.collect_statement_leaves_dst();
        let src_leaves = self.collect_statement_leaves_src();

        let mut leaves_mappings: Vec<MappingWithSimilarity<M>> = Vec::new();

        for src in &src_leaves {
            for dst in &dst_leaves {
                // no need to check for equal types since all are only statements
                let sim = self.compute_label_similarity_simple(&src, &dst);
                if sim > self.config.base_config.label_sim_threshold {
                    leaves_mappings.push(MappingWithSimilarity {
                        src: src.shallow().clone(),
                        dst: dst.shallow().clone(),
                        sim,
                    });
                }
            }
        }

        // Sort mappings by similarity
        leaves_mappings.sort_by(|a, b| b.sim.partial_cmp(&a.sim).unwrap_or(Ordering::Equal));

        // Process mappings in order
        for mapping in leaves_mappings {
            self.mappings
                .link_if_both_unmapped(mapping.src, mapping.dst);
        }
    }

    /// Collect all destination leaf nodes
    fn collect_leaves_dst(&mut self) -> Vec<M::Dst> {
        self.dst_arena
            .iter_df_post::<true>()
            .filter(|t| {
                let id = self.dst_arena.decompress_to(&t);
                self.dst_arena.children(&id).is_empty()
            })
            .collect()
    }

    /// Collect all source leaf nodes
    fn collect_leaves_src(&mut self) -> Vec<M::Src> {
        self.src_arena
            .iter_df_post::<true>()
            .filter(|t| {
                let id = self.src_arena.decompress_to(&t);
                self.src_arena.children(&id).is_empty()
            })
            .collect()
    }

    fn node_is_statement(
        arena: &mut Dsrc,
        stores: HAST,
        node: &<Dsrc as LazyDecompressed<M::Src>>::IdD,
    ) -> bool {
        let original = arena.original(node);
        let node_type = stores.resolve_type(&original);
        node_type.is_statement()
    }

    fn collect_statement_leaves_src(&mut self) -> Vec<<Dsrc as LazyDecompressed<M::Src>>::IdD> {
        let src_root = self.src_arena.starter();

        let iter = CustomPostOrderIterator::new(
            &mut self.src_arena,
            self.stores,
            src_root,
            CustomIteratorConfig {
                yield_leaves: false,
                yield_inner: true,
            },
            |arena: &mut Dsrc,
             stores: HAST,
             node: &<Dsrc as LazyDecompressed<M::Src>>::IdD|
             -> bool {
                let original = arena.original(node);
                let node_type = stores.resolve_type(&original);
                node_type.is_statement()
            },
        );
        let nodes: Vec<_> = iter.collect();
        nodes
    }

    fn collect_statement_leaves_dst(&mut self) -> Vec<<Ddst as LazyDecompressed<M::Dst>>::IdD> {
        let dst_root = self.dst_arena.starter();

        let iter = CustomPostOrderIterator::new(
            &mut self.dst_arena,
            self.stores,
            dst_root,
            CustomIteratorConfig {
                yield_leaves: false,
                yield_inner: true,
            },
            |arena: &mut Ddst, stores: HAST, node: &<Ddst as LazyDecompressed<M::Dst>>::IdD| {
                let original = arena.original(node);
                let node_type = stores.resolve_type(&original);
                node_type.is_statement()
            },
        );
        let nodes: Vec<_> = iter.collect();
        nodes
    }

    /// Check if mapping between two nodes is allowed (same type, both unmapped)
    fn is_mapping_allowed(&self, src_tree: &Dsrc::IdD, dst_tree: &Ddst::IdD) -> bool {
        if self.mappings.is_src(src_tree.shallow()) || self.mappings.is_dst(dst_tree.shallow()) {
            return false;
        }

        let original_src = self.src_arena.original(src_tree);
        let original_dst = self.dst_arena.original(dst_tree);

        let src_type = self.stores.resolve_type(&original_src);
        let dst_type = self.stores.resolve_type(&original_dst);

        src_type == dst_type
    }

    /// Compute label similarity without caching (fallback method)
    fn compute_label_similarity_simple(&self, src_tree: &Dsrc::IdD, dst_tree: &Ddst::IdD) -> f64 {
        let original_src = self.src_arena.original(src_tree);
        let original_dst = self.dst_arena.original(dst_tree);

        let src_node = self.stores.node_store().resolve(&original_src);
        let dst_node = self.stores.node_store().resolve(&original_dst);

        let src_label_id = src_node.try_get_label();
        let dst_label_id = dst_node.try_get_label();

        match (src_label_id, dst_label_id) {
            (Some(src_label_id), Some(dst_label_id)) => {
                let src_label = self.stores.label_store().resolve(&src_label_id);
                let dst_label = self.stores.label_store().resolve(&dst_label_id);
                let dist =
                    str_distance::QGram::new(3).normalized(src_label.chars(), dst_label.chars());
                1.0 - dist
            }
            _ => 0.0,
        }
    }

    /// Exact implementation of lazy_2 cached label similarity computation
    fn compute_cached_label_similarity(
        &self,
        src_leaf: &M::Src,
        dst_leaf: &M::Dst,
        src_tree: &Dsrc::IdD,
        dst_tree: &Ddst::IdD,
        label_cache: &mut HashMap<(HAST::IdN, HAST::IdN), f64, RandomState>,
        src_label_cache: &HashMap<M::Src, Option<(HAST::IdN, String)>, RandomState>,
        dst_label_cache: &HashMap<M::Dst, Option<(HAST::IdN, String)>, RandomState>,
        qgram: &str_distance::QGram,
    ) -> f64 {
        // Get the original node IDs
        let original_src = self.src_arena.original(src_tree);
        let original_dst = self.dst_arena.original(dst_tree);

        // Check if similarity is already cached
        if let Some(sim) = label_cache.get(&(original_src.clone(), original_dst.clone())) {
            return *sim;
        }

        // Get cached label data
        let src_label_data = src_label_cache.get(src_leaf);
        let dst_label_data = dst_label_cache.get(dst_leaf);

        let similarity = match (src_label_data, dst_label_data) {
            (Some(Some((_, src_label))), Some(Some((_, dst_label)))) => {
                // Use the pre-computed QGram object
                let dist = qgram.normalized(src_label.chars(), dst_label.chars());
                1.0 - dist
            }
            _ => 0.0,
        };

        // Cache the result
        label_cache.insert((original_src, original_dst), similarity);

        similarity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompressed_tree_store::lazy_post_order::LazyPostOrder;
    use crate::matchers::Decompressible;
    use crate::matchers::mapping_store::MappingStore;
    use crate::matchers::{Mapper, mapping_store::DefaultMappingStore};
    use crate::tree::simple_tree::vpair_to_stores;
    use hyperast::types::DecompressedFrom;

    #[test]
    fn test_optimized_leaves_matcher_all_optimizations() {
        let (stores, src, dst) = vpair_to_stores(crate::tests::examples::example_leaf_label_swap());

        let mut src_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &src);
        let mut dst_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &dst);

        let mapping = Mapper {
            hyperast: &stores,
            mapping: crate::matchers::Mapping {
                src_arena: src_arena.as_mut(),
                dst_arena: dst_arena.as_mut(),
                mappings: DefaultMappingStore::default(),
            },
        };

        let result = OptimizedLeavesMatcher::match_it(mapping);

        assert_eq!(2, result.mappings.len());

        use crate::decompressed_tree_store::ShallowDecompressedTreeStore;
        let src = result.mapping.src_arena.root();
        let src_cs = result.mapping.src_arena.children(&src);
        let dst = result.mapping.dst_arena.root();
        let dst_cs = result.mapping.dst_arena.children(&dst);

        assert!(result.mapping.mappings.has(&src_cs[0], &dst_cs[1]));
        assert!(result.mapping.mappings.has(&src_cs[1], &dst_cs[0]));
    }

    #[test]
    fn test_optimized_leaves_matcher_no_optimizations() {
        let (stores, src, dst) = vpair_to_stores(crate::tests::examples::example_leaf_label_swap());

        let mut src_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &src);
        let mut dst_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &dst);

        let config = OptimizedLeavesMatcherConfig {
            base_config: super::super::LeavesMatcherConfig::default(),
            enable_label_caching: false,
            enable_type_grouping: false,
            statement_level_iteration: false,
            use_binary_heap: false,
            reuse_qgram_object: false,
        };

        let mapping = Mapper {
            hyperast: &stores,
            mapping: crate::matchers::Mapping {
                src_arena: src_arena.as_mut(),
                dst_arena: dst_arena.as_mut(),
                mappings: DefaultMappingStore::default(),
            },
        };

        let result = OptimizedLeavesMatcher::with_config(mapping, config);

        assert_eq!(2, result.mappings.len());

        use crate::decompressed_tree_store::ShallowDecompressedTreeStore;
        let src = result.mapping.src_arena.root();
        let src_cs = result.mapping.src_arena.children(&src);
        let dst = result.mapping.dst_arena.root();
        let dst_cs = result.mapping.dst_arena.children(&dst);

        assert!(result.mapping.mappings.has(&src_cs[0], &dst_cs[1]));
        assert!(result.mapping.mappings.has(&src_cs[1], &dst_cs[0]));
    }
}
