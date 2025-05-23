use crate::{
    decompressed_tree_store::{
        ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, LazyDecompressed,
        LazyDecompressedTreeStore, LazyPOBorrowSlice, PostOrder, PostOrderIterable, Shallow,
        ShallowDecompressedTreeStore,
    },
    matchers::mapping_store::MonoMappingStore,
};
use hyperast::PrimInt;
use hyperast::types::{HyperAST, LabelStore, Labeled, NodeId, NodeStore, TypeStore, WithHashs};
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

/// Optimized leaves matcher with configurable optimizations
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
    <HAST::TS as TypeStore>::Ty: Hash + Eq + Clone,
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
        if self.config.enable_type_grouping {
            self.execute_with_type_grouping();
        } else {
            self.execute_naive();
        }
    }

    /// Execute with type grouping optimization - only compare leaves of same type
    fn execute_with_type_grouping(&mut self) {
        let dst_leaves = self.collect_leaves_dst();
        let src_leaves = self.collect_leaves_src();

        // Group leaves by type when optimization is enabled
        let mut src_leaves_by_type: HashMap<<HAST::TS as TypeStore>::Ty, Vec<M::Src>> =
            HashMap::new();
        let mut dst_leaves_by_type: HashMap<<HAST::TS as TypeStore>::Ty, Vec<M::Dst>> =
            HashMap::new();

        // Pre-compute label cache if enabled
        let mut src_label_cache: HashMap<M::Src, Option<(HAST::IdN, String)>> = HashMap::new();
        let mut dst_label_cache: HashMap<M::Dst, Option<(HAST::IdN, String)>> = HashMap::new();

        if self.config.enable_label_caching {
            self.build_label_caches(
                &src_leaves,
                &dst_leaves,
                &mut src_label_cache,
                &mut dst_label_cache,
            );
        }

        // Group source leaves by type
        for src_leaf in &src_leaves {
            let src = self.src_arena.decompress_to(src_leaf);
            let original_src = self.src_arena.original(&src);
            let src_type = self.stores.resolve_type(&original_src);
            src_leaves_by_type
                .entry(src_type)
                .or_default()
                .push(*src_leaf);
        }

        // Group destination leaves by type
        for dst_leaf in &dst_leaves {
            let dst = self.dst_arena.decompress_to(dst_leaf);
            let original_dst = self.dst_arena.original(&dst);
            let dst_type = self.stores.resolve_type(&original_dst);
            dst_leaves_by_type
                .entry(dst_type)
                .or_default()
                .push(*dst_leaf);
        }

        // Create QGram object once if reuse is enabled
        let qgram = if self.config.reuse_qgram_object {
            Some(str_distance::QGram::new(3))
        } else {
            None
        };

        // Process mappings using appropriate collection type
        if self.config.use_binary_heap {
            self.process_with_binary_heap(
                &src_leaves_by_type,
                &dst_leaves_by_type,
                &src_label_cache,
                &dst_label_cache,
                qgram.as_ref(),
            );
        } else {
            self.process_with_vector(
                &src_leaves_by_type,
                &dst_leaves_by_type,
                &src_label_cache,
                &dst_label_cache,
                qgram.as_ref(),
            );
        }
    }

    /// Execute naive approach without type grouping - compare all leaves
    fn execute_naive(&mut self) {
        let dst_leaves = self.collect_leaves_dst();
        let src_leaves = self.collect_leaves_src();

        let mut leaves_mappings: Vec<MappingWithSimilarity<M>> = Vec::new();

        for &src_leaf in &src_leaves {
            let src = self.src_arena.decompress_to(&src_leaf);

            for &dst_leaf in &dst_leaves {
                let dst = self.dst_arena.decompress_to(&dst_leaf);

                if self.is_mapping_allowed(&src, &dst) {
                    let sim = self.compute_label_similarity_simple(&src, &dst);
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

    /// Build label caches for source and destination leaves
    fn build_label_caches(
        &mut self,
        src_leaves: &[M::Src],
        dst_leaves: &[M::Dst],
        src_label_cache: &mut HashMap<M::Src, Option<(HAST::IdN, String)>>,
        dst_label_cache: &mut HashMap<M::Dst, Option<(HAST::IdN, String)>>,
    ) {
        // Cache source labels
        for &src_leaf in src_leaves {
            let src = self.src_arena.decompress_to(&src_leaf);
            let original_src = self.src_arena.original(&src);
            let src_node = self.stores.node_store().resolve(&original_src);

            let src_label_entry = if let Some(src_label_id) = src_node.try_get_label() {
                let src_label = self.stores.label_store().resolve(&src_label_id).to_string();
                Some((original_src.clone(), src_label))
            } else {
                None
            };
            src_label_cache.insert(src_leaf, src_label_entry);
        }

        // Cache destination labels
        for &dst_leaf in dst_leaves {
            let dst = self.dst_arena.decompress_to(&dst_leaf);
            let original_dst = self.dst_arena.original(&dst);
            let dst_node = self.stores.node_store().resolve(&original_dst);

            let dst_label_entry = if let Some(dst_label_id) = dst_node.try_get_label() {
                let dst_label = self.stores.label_store().resolve(&dst_label_id).to_string();
                Some((original_dst.clone(), dst_label))
            } else {
                None
            };
            dst_label_cache.insert(dst_leaf, dst_label_entry);
        }
    }

    /// Process mappings using binary heap for optimal ordering
    fn process_with_binary_heap(
        &mut self,
        src_leaves_by_type: &HashMap<<HAST::TS as TypeStore>::Ty, Vec<M::Src>>,
        dst_leaves_by_type: &HashMap<<HAST::TS as TypeStore>::Ty, Vec<M::Dst>>,
        src_label_cache: &HashMap<M::Src, Option<(HAST::IdN, String)>>,
        dst_label_cache: &HashMap<M::Dst, Option<(HAST::IdN, String)>>,
        qgram: Option<&str_distance::QGram>,
    ) {
        let mut best_mappings = BinaryHeap::new();

        // Only compare leaves of the same type
        for (node_type, src_leaves) in src_leaves_by_type.iter() {
            if let Some(dst_leaves) = dst_leaves_by_type.get(node_type) {
                for &src_leaf in src_leaves {
                    let src = self.src_arena.decompress_to(&src_leaf);

                    for &dst_leaf in dst_leaves {
                        let dst = self.dst_arena.decompress_to(&dst_leaf);

                        // Since we're already comparing same types, just check if both are unmapped
                        if !self.mappings.is_src(src.shallow())
                            && !self.mappings.is_dst(dst.shallow())
                        {
                            let sim = if self.config.enable_label_caching {
                                self.compute_cached_label_similarity(
                                    &src_leaf,
                                    &dst_leaf,
                                    &src,
                                    &dst,
                                    src_label_cache,
                                    dst_label_cache,
                                    qgram,
                                )
                            } else {
                                self.compute_label_similarity_simple(&src, &dst)
                            };

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
    }

    /// Process mappings using vector with sorting
    fn process_with_vector(
        &mut self,
        src_leaves_by_type: &HashMap<<HAST::TS as TypeStore>::Ty, Vec<M::Src>>,
        dst_leaves_by_type: &HashMap<<HAST::TS as TypeStore>::Ty, Vec<M::Dst>>,
        src_label_cache: &HashMap<M::Src, Option<(HAST::IdN, String)>>,
        dst_label_cache: &HashMap<M::Dst, Option<(HAST::IdN, String)>>,
        qgram: Option<&str_distance::QGram>,
    ) {
        let mut leaves_mappings: Vec<MappingWithSimilarity<M>> = Vec::new();

        // Only compare leaves of the same type
        for (node_type, src_leaves) in src_leaves_by_type.iter() {
            if let Some(dst_leaves) = dst_leaves_by_type.get(node_type) {
                for &src_leaf in src_leaves {
                    let src = self.src_arena.decompress_to(&src_leaf);

                    for &dst_leaf in dst_leaves {
                        let dst = self.dst_arena.decompress_to(&dst_leaf);

                        // Since we're already comparing same types, just check if both are unmapped
                        if !self.mappings.is_src(src.shallow())
                            && !self.mappings.is_dst(dst.shallow())
                        {
                            let sim = if self.config.enable_label_caching {
                                self.compute_cached_label_similarity(
                                    &src_leaf,
                                    &dst_leaf,
                                    &src,
                                    &dst,
                                    src_label_cache,
                                    dst_label_cache,
                                    qgram,
                                )
                            } else {
                                self.compute_label_similarity_simple(&src, &dst)
                            };

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

    /// Compute label similarity using cached data
    fn compute_cached_label_similarity(
        &self,
        src_leaf: &M::Src,
        dst_leaf: &M::Dst,
        _src_tree: &Dsrc::IdD,
        _dst_tree: &Ddst::IdD,
        src_label_cache: &HashMap<M::Src, Option<(HAST::IdN, String)>>,
        dst_label_cache: &HashMap<M::Dst, Option<(HAST::IdN, String)>>,
        qgram: Option<&str_distance::QGram>,
    ) -> f64 {
        let src_label_data = src_label_cache.get(src_leaf);
        let dst_label_data = dst_label_cache.get(dst_leaf);

        match (src_label_data, dst_label_data) {
            (Some(Some((_, src_label))), Some(Some((_, dst_label)))) => {
                if let Some(qgram) = qgram {
                    // Use the pre-computed QGram object
                    let dist = qgram.normalized(src_label.chars(), dst_label.chars());
                    1.0 - dist
                } else {
                    // Create QGram object for this computation
                    let dist = str_distance::QGram::new(3)
                        .normalized(src_label.chars(), dst_label.chars());
                    1.0 - dist
                }
            }
            _ => 0.0,
        }
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
