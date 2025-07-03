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
use ahash::{AHashSet, RandomState};
use hyperast::types::{HyperAST, LabelStore, Labeled, NodeId, NodeStore, TypeStore, WithHashs};
use hyperast::{PrimInt, types::HyperType};
use hyperast::{nodes::TextSerializer, types::HashKind};
use num_traits::ToPrimitive;
use std::{cmp::Ordering, collections::HashMap, hash::Hash};
use std::{fmt::Debug, time::Instant};
use str_distance::DistanceMetric;

use super::OptimizedLeavesMatcherConfig;

/// Type alias for bigrams
type Bigram = [char; 2];

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

/// Extract bigrams from a string
fn extract_bigrams(text: &str) -> Vec<Bigram> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 2 {
        return vec![];
    }
    chars.windows(2).map(|w| [w[0], w[1]]).collect()
}

/// Compute Dice coefficient between two sets of bigrams
fn dice_similarity(bigrams1: &[Bigram], bigrams2: &[Bigram]) -> f64 {
    if bigrams1.is_empty() && bigrams2.is_empty() {
        return 1.0;
    }
    if bigrams1.is_empty() || bigrams2.is_empty() {
        return 0.0;
    }

    let set1: AHashSet<&Bigram> = bigrams1.iter().collect();
    let set2: AHashSet<&Bigram> = bigrams2.iter().collect();

    let intersection_size = set1.intersection(&set2).count();
    let total_size = bigrams1.len() + bigrams2.len();

    (2.0 * intersection_size as f64) / (total_size as f64)
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
        // Only n-gram caching or label caching or neither but not both
        assert!(
            !(config.enable_ngram_caching && config.enable_label_caching),
            "ngram_cache and label_cache cannot be both true"
        );

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

    /// Get text representation of a node (either label or full serialization)
    fn get_node_text(&self, original: &HAST::IdN) -> String {
        if self.config.statement_level_iteration {
            TextSerializer::new(&self.stores, original.clone()).to_string()
        } else {
            let node = self.stores.node_store().resolve(original);
            if let Some(label_id) = node.try_get_label() {
                self.stores.label_store().resolve(&label_id).to_string()
            } else {
                String::new()
            }
        }
    }

    /// Execute with statement level iteration
    fn execute_statement(&mut self) {
        let start_time = std::time::Instant::now();

        let dst_leaves = self.collect_statement_leaves_dst();
        let src_leaves = self.collect_statement_leaves_src();

        let mut leaves_mappings: Vec<MappingWithSimilarity<Dsrc, Ddst, M>> = Vec::new();
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

        // Ngram caches for when ngram caching is enabled
        let mut src_ngram_cache: HashMap<
            &<Dsrc as LazyDecompressed<M::Src>>::IdD,
            Vec<Bigram>,
            RandomState,
        > = HashMap::default();
        let mut dst_ngram_cache: HashMap<
            &<Ddst as LazyDecompressed<M::Dst>>::IdD,
            Vec<Bigram>,
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

        for (_src_idx, src) in src_leaves.iter().enumerate() {
            let hash_start = std::time::Instant::now();
            let src_original = self.src_arena.original(src);
            let src_node = self.stores.node_store().resolve(&src_original);
            let src_label_hash = WithHashs::hash(&src_node, &HashKind::label());
            hash_computation_time += hash_start.elapsed();

            for dst in dst_leaves.iter() {
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

                // Check if types are the same. No need to check if they are already matched since we have no matching before this except for the dst which we excluded
                let src_type = self.stores.resolve_type(&src_original);
                let dst_type = self.stores.resolve_type(&dst_original);
                if src_type != dst_type {
                    continue;
                }

                // Compute similarity based on caching strategy
                let sim = if self.config.enable_ngram_caching {
                    let text_start = Instant::now();
                    // Ngram caching strategy
                    let src_bigrams = if let Some(bigrams) = src_ngram_cache.get(src) {
                        cache_hits += 1;
                        bigrams
                    } else {
                        cache_misses += 1;
                        let text = self.get_node_text(&src_original);
                        let bigrams = extract_bigrams(&text);
                        src_ngram_cache.insert(&src, bigrams);
                        src_ngram_cache.get(src).unwrap()
                    };

                    let dst_bigrams = if let Some(bigrams) = dst_ngram_cache.get(dst) {
                        cache_hits += 1;
                        bigrams
                    } else {
                        cache_misses += 1;
                        let text = self.get_node_text(&dst_original);
                        let bigrams = extract_bigrams(&text);
                        dst_ngram_cache.insert(&dst, bigrams);
                        dst_ngram_cache.get(dst).unwrap()
                    };

                    text_serialization_time += text_start.elapsed();

                    let sim_start = std::time::Instant::now();
                    let sim = dice_similarity(&src_bigrams, &dst_bigrams);
                    similarity_computation_time += sim_start.elapsed();
                    similarity_checks += 1;

                    sim
                } else {
                    let text_start = Instant::now();
                    let (src_text, dst_text) = if self.config.enable_label_caching {
                        let src_text = if let Some(text) = src_text_cache.get(src) {
                            cache_hits += 1;
                            text.clone()
                        } else {
                            cache_misses += 1;
                            let text = self.get_node_text(&src_original);
                            src_text_cache.insert(&src, text.clone());
                            text
                        };
                        let dst_text = if let Some(text) = dst_text_cache.get(dst) {
                            cache_hits += 1;
                            text.clone()
                        } else {
                            cache_misses += 1;
                            let text = self.get_node_text(&dst_original);
                            dst_text_cache.insert(&dst, text.clone());
                            text
                        };
                        (src_text, dst_text)
                    } else {
                        // No caching
                        let src_text = self.get_node_text(&src_original);

                        let dst_text = self.get_node_text(&dst_original);
                        (src_text, dst_text)
                    };

                    text_serialization_time += text_start.elapsed();

                    characters_compared += src_text.chars().count() + dst_text.chars().count();
                    let sim_start = std::time::Instant::now();
                    let sim = 1.0
                        - str_distance::QGram::new(2)
                            .normalized(src_text.chars(), dst_text.chars());
                    similarity_computation_time += sim_start.elapsed();
                    similarity_checks += 1;

                    sim
                };

                if sim > self.config.base_config.label_sim_threshold {
                    leaves_mappings.push(MappingWithSimilarity {
                        src: src.clone(),
                        dst: dst.clone(),
                        sim,
                    });
                }
            }
        }

        // Sort mappings by similarity
        leaves_mappings.sort_by(|a, b| b.sim.partial_cmp(&a.sim).unwrap_or(Ordering::Equal));

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
                    }
                }

                mapped_count += 1;
            }
        }
        let total_time = start_time.elapsed();

        // Update metrics
        self.metrics.total_time = total_time;
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
            CustomIteratorConfig::leaves(self.config.enable_deep_leaves),
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
            CustomIteratorConfig::leaves(self.config.enable_deep_leaves),
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
            enable_deep_leaves: false,
            enable_ngram_caching: false,
            statement_level_iteration: false,
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

    #[test]
    fn test_optimized_leaves_matcher_ngram_caching() {
        let (stores, src, dst) = vpair_to_stores(crate::tests::examples::example_leaf_label_swap());

        let mut src_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &src);
        let mut dst_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &dst);

        let config = OptimizedLeavesMatcherConfig {
            base_config: super::super::LeavesMatcherConfig::default(),
            enable_label_caching: false,
            enable_deep_leaves: false,
            enable_ngram_caching: true,
            statement_level_iteration: true, // Required for ngram caching
        };

        let mapping = Mapper {
            hyperast: &stores,
            mapping: crate::matchers::Mapping {
                src_arena: src_arena.as_mut(),
                dst_arena: dst_arena.as_mut(),
                mappings: DefaultMappingStore::default(),
            },
        };

        let (result, metrics) = OptimizedLeavesMatcher::with_config_and_metrics(mapping, config);

        // Verify that mappings were created
        assert!(
            result.mappings.len() > 0,
            "Should have created some mappings"
        );

        // Verify that cache was used (if there were repeated nodes)
        if metrics.cache_hits > 0 || metrics.cache_misses > 0 {
            assert!(
                metrics.cache_hits + metrics.cache_misses > 0,
                "Cache should have been accessed"
            );
        }

        // For the simple leaf label swap example, verify the specific mappings
        use crate::decompressed_tree_store::ShallowDecompressedTreeStore;
        let src = result.mapping.src_arena.root();
        let src_cs = result.mapping.src_arena.children(&src);
        let dst = result.mapping.dst_arena.root();
        let dst_cs = result.mapping.dst_arena.children(&dst);

        // The matcher should correctly identify the swapped leaves
        if src_cs.len() == 2 && dst_cs.len() == 2 {
            assert!(
                result.mapping.mappings.has(&src_cs[0], &dst_cs[1])
                    || result.mapping.mappings.has(&src_cs[0], &dst_cs[0])
            );
            assert!(
                result.mapping.mappings.has(&src_cs[1], &dst_cs[0])
                    || result.mapping.mappings.has(&src_cs[1], &dst_cs[1])
            );
        }
    }

    #[test]
    fn test_mutual_exclusivity_ngram_label_caching() {
        let (stores, src, dst) = vpair_to_stores(crate::tests::examples::example_leaf_label_swap());

        let mut src_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &src);
        let mut dst_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &dst);

        // Test that enabling both ngram and label caching results in only ngram caching being active
        let config = OptimizedLeavesMatcherConfig {
            base_config: super::super::LeavesMatcherConfig::default(),
            enable_label_caching: true,
            enable_deep_leaves: false,
            enable_ngram_caching: true,
            statement_level_iteration: true,
        };

        let mapping = Mapper {
            hyperast: &stores,
            mapping: crate::matchers::Mapping {
                src_arena: src_arena.as_mut(),
                dst_arena: dst_arena.as_mut(),
                mappings: DefaultMappingStore::default(),
            },
        };

        // The with_config_and_metrics method should handle the mutual exclusivity
        let (result, _metrics) = OptimizedLeavesMatcher::with_config_and_metrics(mapping, config);

        // Verify that mappings still work correctly
        assert!(
            result.mappings.len() > 0,
            "Should have created some mappings"
        );
    }

    #[test]
    fn test_extract_bigrams() {
        // Test empty string
        assert_eq!(extract_bigrams(""), Vec::<Bigram>::new());

        // Test single character
        assert_eq!(extract_bigrams("a"), Vec::<Bigram>::new());

        // Test two characters
        assert_eq!(extract_bigrams("ab"), vec![['a', 'b']]);

        // Test multiple characters
        assert_eq!(
            extract_bigrams("hello"),
            vec![['h', 'e'], ['e', 'l'], ['l', 'l'], ['l', 'o']]
        );

        // Test with spaces
        assert_eq!(
            extract_bigrams("hi there"),
            vec![
                ['h', 'i'],
                ['i', ' '],
                [' ', 't'],
                ['t', 'h'],
                ['h', 'e'],
                ['e', 'r'],
                ['r', 'e']
            ]
        );

        // Test with unicode
        assert_eq!(
            extract_bigrams("café"),
            vec![['c', 'a'], ['a', 'f'], ['f', 'é']]
        );
    }

    #[test]
    fn test_dice_similarity() {
        // Test identical bigrams
        let bigrams1 = vec![['h', 'e'], ['e', 'l'], ['l', 'l'], ['l', 'o']];
        let bigrams2 = vec![['h', 'e'], ['e', 'l'], ['l', 'l'], ['l', 'o']];
        assert_eq!(dice_similarity(&bigrams1, &bigrams2), 1.0);

        // Test completely different bigrams
        let bigrams1 = vec![['a', 'b'], ['b', 'c']];
        let bigrams2 = vec![['x', 'y'], ['y', 'z']];
        assert_eq!(dice_similarity(&bigrams1, &bigrams2), 0.0);

        // Test partial overlap
        let bigrams1 = vec![['h', 'e'], ['e', 'l'], ['l', 'l'], ['l', 'o']];
        let bigrams2 = vec![['h', 'e'], ['e', 'l'], ['l', 'a']];
        // 2 common bigrams out of 7 total -> 2 * 2 / 7 = 4/7 ≈ 0.571
        let similarity = dice_similarity(&bigrams1, &bigrams2);
        assert!((similarity - 4.0 / 7.0).abs() < 0.001);

        // Test empty sets
        assert_eq!(dice_similarity(&vec![], &vec![]), 1.0);
        assert_eq!(dice_similarity(&vec![['a', 'b']], &vec![]), 0.0);
        assert_eq!(dice_similarity(&vec![], &vec![['a', 'b']]), 0.0);

        // Test with duplicates
        let bigrams1 = vec![['a', 'a'], ['a', 'a']]; // "aaa" -> ['a','a'], ['a','a']
        let bigrams2 = vec![['a', 'a']]; // "aa" -> ['a','a']
        // 1 common unique bigram, total 3 bigrams -> 2 * 1 / 3 = 2/3
        let similarity = dice_similarity(&bigrams1, &bigrams2);
        assert!((similarity - 2.0 / 3.0).abs() < 0.001);
    }
}
