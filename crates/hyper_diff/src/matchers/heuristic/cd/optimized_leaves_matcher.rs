use crate::{
    decompressed_tree_store::{
        ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, LazyDecompressed,
        LazyDecompressedTreeStore, LazyPOBorrowSlice, PostOrder, PostOrderIterable, Shallow,
        ShallowDecompressedTreeStore,
    },
    matchers::{
        heuristic::cd::{
            LeavesMatcherMetrics,
            iterator::{CustomIteratorConfig, CustomPostOrderIterator},
        },
        mapping_store::MonoMappingStore,
    },
};
use ahash::RandomState;
use hyperast::types::{HyperAST, LabelStore, Labeled, NodeId, NodeStore, TypeStore, WithHashs};
use hyperast::{PrimInt, types::HyperType};
use hyperast::{nodes::TextSerializer, types::HashKind};
use num_traits::ToPrimitive;
use std::fmt::Debug;
use std::{cmp::Ordering, collections::HashMap, hash::Hash};
use str_distance::DistanceMetric;

use super::OptimizedLeavesMatcherConfig;

/// A mapping candidate with similarity score for priority queue ordering
struct MappingWithSimilarity<
    Dsrc: LazyDecompressed<M::Src>,
    Ddst: LazyDecompressed<M::Dst>,
    M: MonoMappingStore,
> {
    src: Dsrc::IdD,
    dst: Ddst::IdD,
    sim: f64,
}

impl<Dsrc: LazyDecompressed<M::Src>, Ddst: LazyDecompressed<M::Dst>, M: MonoMappingStore> PartialEq
    for MappingWithSimilarity<Dsrc, Ddst, M>
{
    fn eq(&self, other: &Self) -> bool {
        self.sim == other.sim
    }
}

impl<Dsrc: LazyDecompressed<M::Src>, Ddst: LazyDecompressed<M::Dst>, M: MonoMappingStore> Eq
    for MappingWithSimilarity<Dsrc, Ddst, M>
{
}

impl<Dsrc: LazyDecompressed<M::Src>, Ddst: LazyDecompressed<M::Dst>, M: MonoMappingStore> PartialOrd
    for MappingWithSimilarity<Dsrc, Ddst, M>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.sim.partial_cmp(&other.sim)
    }
}

impl<Dsrc: LazyDecompressed<M::Src>, Ddst: LazyDecompressed<M::Dst>, M: MonoMappingStore> Ord
    for MappingWithSimilarity<Dsrc, Ddst, M>
{
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
    pub metrics: LeavesMatcherMetrics,
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
    Dsrc::IdD: std::fmt::Debug,
    Ddst::IdD: std::fmt::Debug,
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
        Self::with_config_and_metrics(mapping, config).0
    }

    /// Create matcher with custom configuration and return metrics
    pub fn with_config_and_metrics(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
        config: OptimizedLeavesMatcherConfig,
    ) -> (
        crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
        LeavesMatcherMetrics,
    ) {
        let mut matcher = Self {
            stores: mapping.hyperast,
            src_arena: mapping.mapping.src_arena,
            dst_arena: mapping.mapping.dst_arena,
            mappings: mapping.mapping.mappings,
            config,
            metrics: LeavesMatcherMetrics::default(),
        };

        matcher
            .mappings
            .topit(matcher.src_arena.len(), matcher.dst_arena.len());

        let start_time = std::time::Instant::now();
        matcher.execute();
        matcher.metrics.total_time = start_time.elapsed();

        let metrics = matcher.metrics.clone();
        let mapper = crate::matchers::Mapper {
            hyperast: mapping.hyperast,
            mapping: crate::matchers::Mapping {
                src_arena: matcher.src_arena,
                dst_arena: matcher.dst_arena,
                mappings: matcher.mappings,
            },
        };

        (mapper, metrics)
    }

    /// Create matcher with default optimized configuration
    pub fn match_it(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M> {
        Self::with_config(mapping, OptimizedLeavesMatcherConfig::default())
    }

    /// Execute the leaves matching with configured optimizations
    fn execute(&mut self) {
        self.execute_statement();
    }

    /// Execute with statement level iteration
    fn execute_statement(&mut self) {
        let start_time = std::time::Instant::now();

        let collect_start = std::time::Instant::now();
        let dst_leaves = self.collect_statement_leaves_dst();
        let src_leaves = self.collect_statement_leaves_src();
        log::trace!("src_leaves count: {:?}", src_leaves.len());
        log::trace!("node count: {:?}", self.src_arena.len());

        let collect_time = collect_start.elapsed();
        log::trace!(
            "✓ Statement leaf collection: {:?} (src: {}, dst: {})",
            collect_time,
            src_leaves.len(),
            dst_leaves.len()
        );

        let mut leaves_mappings: Vec<MappingWithSimilarity<Dsrc, Ddst, M>> = Vec::new();
        let total_comparisons = src_leaves.len() * dst_leaves.len();
        log::trace!("✓ Total comparisons needed: {}", total_comparisons);

        let comparison_start = std::time::Instant::now();
        log::trace!("✓ Using label caching optimization");
        let cache_build_start = std::time::Instant::now();
        let mut src_text_cache: HashMap<
            &<Dsrc as LazyDecompressed<M::Src>>::IdD,
            String,
            RandomState,
        > = HashMap::default();
        let mut dst_text_cache: HashMap<
            &<Ddst as LazyDecompressed<M::Dst>>::IdD,
            String,
            RandomState,
        > = HashMap::default();

        let mut ignore_dst = bitvec::bitbox![0;self.dst_arena.len()];

        // Performance counters
        let mut hash_computation_time = std::time::Duration::ZERO;
        let mut text_serialization_time = std::time::Duration::ZERO;
        let mut similarity_computation_time = std::time::Duration::ZERO;
        let mut characters_compared = 0;
        let mut cache_hits = 0;
        let mut cache_misses = 0;
        let mut exact_matches = 0;
        let mut similarity_checks = 0;
        let mut skipped_dst = 0;
        let total_comparisons = src_leaves.len() * dst_leaves.len();

        for (src_idx, src) in src_leaves.iter().enumerate() {
            if src_idx % 100 == 0 && src_idx > 0 {
                log::trace!(
                    "  → Processing src leaf {}/{} ({:.1}%)",
                    src_idx,
                    src_leaves.len(),
                    (src_idx as f64 / src_leaves.len() as f64) * 100.0
                );
            }

            let hash_start = std::time::Instant::now();
            let src_original = self.src_arena.original(src);
            let src_node = self.stores.node_store().resolve(&src_original);
            let src_label_hash = WithHashs::hash(&src_node, &HashKind::label());
            hash_computation_time += hash_start.elapsed();

            for (dst_idx, dst) in dst_leaves.iter().enumerate() {
                let dst_bit_idx = dst.shallow().to_usize().unwrap();
                if ignore_dst[dst_bit_idx] {
                    skipped_dst += 1;
                    continue;
                }
                let hash_start = std::time::Instant::now();
                let dst_original = self.dst_arena.original(&dst);
                let dst_node = self.stores.node_store().resolve(&dst_original);
                let dst_label_hash = WithHashs::hash(&dst_node, &HashKind::label());
                hash_computation_time += hash_start.elapsed();

                if src_label_hash == dst_label_hash {
                    exact_matches += 1;
                    leaves_mappings.push(MappingWithSimilarity {
                        src: src.clone(),
                        dst: dst.clone(),
                        sim: f64::MAX,
                    });

                    ignore_dst.set(dst_bit_idx, true);
                    break;
                }

                // Skip comparison if the nodes are not the same type
                if !self.config.statement_level_iteration {
                    let src_type = self.stores.resolve_type(&src_original);
                    let dst_type = self.stores.resolve_type(&dst_original);
                    if src_type != dst_type {
                        continue;
                    }
                }

                // get src and dst text
                let text_start = std::time::Instant::now();

                let (src_text, dst_text) = {
                    if !self.config.statement_level_iteration {
                        // only get the label of a node and not its serialized representation

                        let src_node = self.stores.node_store().resolve(&src_original);
                        let src_label = if let Some(src_label_id) = src_node.try_get_label() {
                            Some(self.stores.label_store().resolve(&src_label_id).to_string())
                        } else {
                            // let text =
                            //     TextSerializer::new(&self.stores, src_original.clone()).to_string();
                            // log::trace!("src_label_id is None:\n{}", text);
                            None
                        }
                        .unwrap_or_default();

                        let dst_node = self.stores.node_store().resolve(&dst_original);
                        let dst_label = if let Some(dst_label_id) = dst_node.try_get_label() {
                            Some(self.stores.label_store().resolve(&dst_label_id).to_string())
                        } else {
                            // log::trace!("dst_label_id is None");
                            None
                        }
                        .unwrap_or_default();

                        (src_label, dst_label)
                    } else {
                        if self.config.enable_label_caching {
                            let src_text = if let Some(text) = src_text_cache.get(src) {
                                cache_hits += 1;
                                text.clone()
                            } else {
                                cache_misses += 1;

                                let text = TextSerializer::new(&self.stores, src_original.clone())
                                    .to_string();
                                src_text_cache.insert(&src, text.clone());
                                text.clone()
                            };
                            let dst_text = if let Some(text) = dst_text_cache.get(dst) {
                                cache_hits += 1;
                                text.clone()
                            } else {
                                cache_misses += 1;
                                let text = TextSerializer::new(&self.stores, dst_original.clone())
                                    .to_string();
                                dst_text_cache.insert(&dst, text.clone());
                                text.clone()
                            };
                            (src_text, dst_text)
                        } else {
                            let original_src = self.src_arena.original(&src);
                            let src_text =
                                TextSerializer::new(&self.stores, original_src).to_string();
                            let original_dst = self.dst_arena.original(&dst);
                            let dst_text =
                                TextSerializer::new(&self.stores, original_dst).to_string();
                            (src_text, dst_text)
                        }
                    }
                };

                text_serialization_time += text_start.elapsed();

                // no need to check for equal types since all nodes are statements
                characters_compared += src_text.chars().count() + dst_text.chars().count();
                let sim_start = std::time::Instant::now();
                let sim = 1.0
                    - str_distance::QGram::new(3).normalized(src_text.chars(), dst_text.chars());
                similarity_computation_time += sim_start.elapsed();
                similarity_checks += 1;

                if sim > self.config.base_config.label_sim_threshold {
                    leaves_mappings.push(MappingWithSimilarity {
                        src: src.clone(),
                        dst: dst.clone(),
                        sim,
                    });
                }
            }
        }
        let cache_build_time = cache_build_start.elapsed();
        log::trace!(
            "✓ Cached text serialization & comparison: {:?}",
            cache_build_time
        );
        log::trace!("  → Hash computation time: {:?}", hash_computation_time);
        log::trace!("  → Text serialization time: {:?}", text_serialization_time);
        log::trace!(
            "  → Similarity computation time: {:?}",
            similarity_computation_time
        );
        log::trace!(
            "  → Cache hits: {}, Cache misses: {}",
            cache_hits,
            cache_misses
        );
        log::trace!("  → Characters compared: {}", characters_compared);

        log::trace!("  → Exact matches (hash): {}", exact_matches);
        log::trace!("  → Similarity checks performed: {}", similarity_checks);
        log::trace!("  → Skipped dst nodes: {}", skipped_dst);
        log::trace!(
            "  → Avg time per similarity check: {:?}",
            if similarity_checks > 0 {
                similarity_computation_time / similarity_checks
            } else {
                std::time::Duration::ZERO
            }
        );
        log::trace!(
            "  → Avg time per text serialization: {:?}",
            if cache_misses > 0 {
                text_serialization_time / cache_misses
            } else {
                std::time::Duration::ZERO
            }
        );

        let comparison_time = comparison_start.elapsed();
        log::trace!(
            "✓ All comparisons: {:?} ({} candidates found)",
            comparison_time,
            leaves_mappings.len()
        );

        let sort_start = std::time::Instant::now();
        // Sort mappings by similarity
        leaves_mappings.sort_by(|a, b| b.sim.partial_cmp(&a.sim).unwrap_or(Ordering::Equal));
        let sort_time = sort_start.elapsed();
        log::trace!("✓ Sorting candidates: {:?}", sort_time);

        let mapping_start = std::time::Instant::now();
        let mut mapped_count = 0;
        // Process mappings in order
        for mapping in leaves_mappings {
            if self
                .mappings
                .link_if_both_unmapped(mapping.src.shallow().clone(), mapping.dst.shallow().clone())
            {
                // Only try to match descendants if statement level iteration is enabled,
                // because if it is disabled, we only have actual leaves, which do not have descendants
                if self.config.statement_level_iteration {
                    let src = self.src_arena.descendants(&mapping.src);
                    let dst = self.dst_arena.descendants(&mapping.dst);

                    if src.len() == dst.len() {
                        src.iter()
                            .zip(dst.iter())
                            .for_each(|(src, dst)| self.mappings.link(*src, *dst));
                    } else {
                        log::trace!("Skipping mapping due to different number of descendants");
                    }
                }

                mapped_count += 1;
            }
        }
        let mapping_time = mapping_start.elapsed();
        log::trace!(
            "✓ Creating mappings: {:?} ({} mappings created)",
            mapping_time,
            mapped_count
        );

        let total_time = start_time.elapsed();
        log::trace!("Statement level matcher complete: {:?} \n", total_time);

        // Update metrics
        self.metrics.total_comparisons += total_comparisons;
        self.metrics.successful_matches += mapped_count;
        self.metrics.hash_computation_time += hash_computation_time;
        self.metrics.text_serialization_time += text_serialization_time;
        self.metrics.similarity_time += similarity_computation_time;
        self.metrics.characters_compared += characters_compared;
        self.metrics.cache_hits += cache_hits;
        self.metrics.cache_misses += cache_misses as usize;
        self.metrics.exact_matches += exact_matches;
        self.metrics.similarity_checks += similarity_checks as usize;
        self.metrics.skipped_dst += skipped_dst;
    }

    fn collect_statement_leaves_src(&mut self) -> Vec<<Dsrc as LazyDecompressed<M::Src>>::IdD> {
        let src_root = self.src_arena.starter();

        let iter = CustomPostOrderIterator::new(
            &mut self.src_arena,
            self.stores,
            src_root,
            CustomIteratorConfig {
                yield_leaves: true,
                yield_inner: false,
            },
            |arena: &mut Dsrc,
             stores: HAST,
             node: &<Dsrc as LazyDecompressed<M::Src>>::IdD|
             -> bool {
                if arena.decompress_children(node).is_empty() {
                    return true;
                }
                // If we are in statement level iteration, we should regard statement leaves as logical leaves
                if self.config.statement_level_iteration {
                    let original = arena.original(node);
                    let node_type = stores.resolve_type(&original);
                    return node_type.is_statement();
                }
                return false;
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
                yield_leaves: true,
                yield_inner: false,
            },
            |arena: &mut Ddst, stores: HAST, node: &<Ddst as LazyDecompressed<M::Dst>>::IdD| {
                if arena.decompress_children(node).is_empty() {
                    return true;
                }
                // If we are in statement level iteration, we should regard statement leaves as logical leaves
                if self.config.statement_level_iteration {
                    let original = arena.original(node);
                    let node_type = stores.resolve_type(&original);
                    return node_type.is_statement();
                }
                return false;
            },
        );
        let nodes: Vec<_> = iter.collect();
        nodes
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
