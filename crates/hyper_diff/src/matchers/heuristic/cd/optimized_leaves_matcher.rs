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
use hyperast::types::{HyperAST, LabelStore, Labeled, NodeId, NodeStore, TypeStore, WithHashs};
use hyperast::{PrimInt, types::HyperType};
use hyperast::{nodes::TextSerializer, types::HashKind};
use num_traits::ToPrimitive;
use std::fmt::Debug;
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
    hash::Hash,
};
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
        println!("=== OPTIMIZED LEAVES MATCHER START ===");
        if self.config.statement_level_iteration {
            println!("=== STATEMENT LEVEL ITERATION START ===");
            self.execute_statement();
        } else if self.config.enable_type_grouping {
            println!("=== TYPE GROUPING MATCHER START ===");

            self.execute_with_type_grouping();
        } else {
            println!("=== NAIVE MATCHER START ===");
            self.execute_naive();
        }
    }

    /// Execute with type grouping optimization - only compare leaves of same type
    fn execute_with_type_grouping(&mut self) {
        let start_time = std::time::Instant::now();
        println!("=== TYPE GROUPING MATCHER START ===");

        // Pre-compute and cache label info (always when using type grouping for best performance)
        let mut label_cache: HashMap<(HAST::IdN, HAST::IdN), f64, RandomState> =
            HashMap::with_hasher(RandomState::new());

        // Create QGram object once if reuse is enabled
        let qgram = str_distance::QGram::new(3);

        // Collect leaves
        let collect_start = std::time::Instant::now();
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

        let collect_time = collect_start.elapsed();
        println!(
            "✓ Leaf collection: {:?} (src: {}, dst: {})",
            collect_time,
            src_leaves.len(),
            dst_leaves.len()
        );

        // Group leaves by type and build label cache in single pass for optimal performance
        let grouping_start = std::time::Instant::now();
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

        let grouping_time = grouping_start.elapsed();
        println!(
            "✓ Type grouping & caching: {:?} (src types: {}, dst types: {})",
            grouping_time,
            src_leaves_by_type.len(),
            dst_leaves_by_type.len()
        );

        // Calculate total comparisons that will be made vs naive approach
        let total_naive_comparisons = src_leaves.len() * dst_leaves.len();
        let total_grouped_comparisons: usize = src_leaves_by_type
            .iter()
            .map(|(node_type, src_leaves)| {
                dst_leaves_by_type
                    .get(node_type)
                    .map(|dst_leaves| src_leaves.len() * dst_leaves.len())
                    .unwrap_or(0)
            })
            .sum();

        println!(
            "✓ Comparison reduction: {} → {} ({:.1}x speedup)",
            total_naive_comparisons,
            total_grouped_comparisons,
            total_naive_comparisons as f64 / total_grouped_comparisons.max(1) as f64
        );

        // Use appropriate collection type for mappings
        let comparison_start = std::time::Instant::now();
        let mut comparison_count = 0;
        let mut similarity_calculations = 0;

        let mut leaves_mappings: Vec<MappingWithSimilarity<Dsrc, Ddst, M>> = Vec::new();

        // Only compare leaves of the same type
        for (node_type, src_leaves) in src_leaves_by_type.iter() {
            if let Some(dst_leaves) = dst_leaves_by_type.get(node_type) {
                for &src_leaf in src_leaves {
                    let src = self.src_arena.decompress_to(&src_leaf);

                    for &dst_leaf in dst_leaves {
                        let dst = self.dst_arena.decompress_to(&dst_leaf);
                        comparison_count += 1;

                        if !self.mappings.is_src(src.shallow())
                            && !self.mappings.is_dst(dst.shallow())
                        {
                            similarity_calculations += 1;
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
                                    src: src.clone(),
                                    dst: dst.clone(),
                                    sim,
                                });
                            }
                        }
                    }
                }
            }
        }

        let comparison_time = comparison_start.elapsed();
        println!(
            "✓ Vector comparisons: {:?} ({} total, {} similarities calculated, {} candidates)",
            comparison_time,
            comparison_count,
            similarity_calculations,
            leaves_mappings.len()
        );

        let sort_start = std::time::Instant::now();
        // Sort mappings by similarity
        leaves_mappings.sort_by(|a, b| b.sim.partial_cmp(&a.sim).unwrap_or(Ordering::Equal));
        let sort_time = sort_start.elapsed();
        println!("✓ Vector sorting: {:?}", sort_time);

        let mapping_start = std::time::Instant::now();
        let mut mapped_count = 0;
        // Process mappings in order
        for mapping in leaves_mappings {
            if self
                .mappings
                .link_if_both_unmapped(mapping.src.shallow().clone(), mapping.dst.shallow().clone())
            {
                mapped_count += 1;
            }
        }
        let mapping_time = mapping_start.elapsed();
        println!(
            "✓ Vector mapping: {:?} ({} mappings created)",
            mapping_time, mapped_count
        );

        let total_time = start_time.elapsed();
        println!("=== TYPE GROUPING MATCHER COMPLETE: {:?} ===\n", total_time);
    }

    /// Execute with statement level iteration
    fn execute_statement(&mut self) {
        let start_time = std::time::Instant::now();
        println!("=== STATEMENT LEVEL MATCHER START ===");

        let collect_start = std::time::Instant::now();
        let dst_leaves = self.collect_statement_leaves_dst();
        let src_leaves = self.collect_statement_leaves_src();
        let collect_time = collect_start.elapsed();
        println!(
            "✓ Statement leaf collection: {:?} (src: {}, dst: {})",
            collect_time,
            src_leaves.len(),
            dst_leaves.len()
        );

        let mut leaves_mappings: Vec<MappingWithSimilarity<Dsrc, Ddst, M>> = Vec::new();
        let total_comparisons = src_leaves.len() * dst_leaves.len();
        println!("✓ Total comparisons needed: {}", total_comparisons);

        let comparison_start = std::time::Instant::now();
        if self.config.enable_label_caching {
            println!("✓ Using label caching optimization");
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

            for src in &src_leaves {
                let src_original = self.src_arena.original(src);
                let src_node = self.stores.node_store().resolve(&src_original);
                let src_label_hash = WithHashs::hash(&src_node, &HashKind::label());

                for dst in &dst_leaves {
                    let dst_idx = dst.shallow().to_usize().unwrap();
                    if ignore_dst[dst_idx] {
                        continue;
                    }
                    let dst_original = self.dst_arena.original(&dst);
                    let dst_node = self.stores.node_store().resolve(&dst_original);
                    let dst_label_hash = WithHashs::hash(&dst_node, &HashKind::label());

                    if src_label_hash == dst_label_hash {
                        leaves_mappings.push(MappingWithSimilarity {
                            src: src.clone(),
                            dst: dst.clone(),
                            sim: f64::MAX,
                        });
                        // self.mappings
                        //     .link(src.shallow().clone(), dst.shallow().clone());

                        // let src = self.src_arena.descendants(src);
                        // let dst = self.dst_arena.descendants(dst);
                        // src.iter()
                        //     .zip(dst.iter())
                        //     .for_each(|(src, dst)| self.mappings.link(*src, *dst));

                        ignore_dst.set(dst_idx, true);
                        break;
                    }

                    // get src and dst text

                    let src_text = if let Some(text) = src_text_cache.get(src) {
                        text
                    } else {
                        let original_src = self.src_arena.original(&src);

                        let text = TextSerializer::new(&self.stores, original_src).to_string();
                        src_text_cache.insert(&src, text.clone());
                        src_text_cache.get(src).unwrap()
                    };
                    let dst_text = if let Some(text) = dst_text_cache.get(dst) {
                        text
                    } else {
                        let original_dst = self.dst_arena.original(&dst);

                        let text = TextSerializer::new(&self.stores, original_dst).to_string();
                        dst_text_cache.insert(&dst, text.clone());
                        dst_text_cache.get(dst).unwrap()
                    };

                    // no need to check for equal types since all nodes are statements
                    let sim = 1.0
                        - str_distance::QGram::new(3)
                            .normalized(src_text.chars(), dst_text.chars());
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
            println!(
                "✓ Cached text serialization & comparison: {:?}",
                cache_build_time
            );
        } else {
            println!("✓ Using direct text serialization (no caching)");
            for src in &src_leaves {
                let original_src = self.src_arena.original(&src);

                let src_text = TextSerializer::new(&self.stores, original_src).to_string();

                for dst in &dst_leaves {
                    let original_dst = self.dst_arena.original(&dst);

                    let dst_text = TextSerializer::new(&self.stores, original_dst).to_string();

                    // no need to check for equal types since all are only statements
                    let sim = 1.0
                        - str_distance::QGram::new(3)
                            .normalized(src_text.chars(), dst_text.chars());
                    if sim > self.config.base_config.label_sim_threshold {
                        leaves_mappings.push(MappingWithSimilarity {
                            src: src.clone(),
                            dst: dst.clone(),
                            sim,
                        });
                    }
                }
            }
        }

        let comparison_time = comparison_start.elapsed();
        println!(
            "✓ All comparisons: {:?} ({} candidates found)",
            comparison_time,
            leaves_mappings.len()
        );

        let sort_start = std::time::Instant::now();
        // Sort mappings by similarity
        leaves_mappings.sort_by(|a, b| b.sim.partial_cmp(&a.sim).unwrap_or(Ordering::Equal));
        let sort_time = sort_start.elapsed();
        println!("✓ Sorting candidates: {:?}", sort_time);

        let mapping_start = std::time::Instant::now();
        let mut mapped_count = 0;
        // Process mappings in order
        for mapping in leaves_mappings {
            if self
                .mappings
                .link_if_both_unmapped(mapping.src.shallow().clone(), mapping.dst.shallow().clone())
            {
                let src = self.src_arena.descendants(&mapping.src);
                let dst = self.dst_arena.descendants(&mapping.dst);

                src.iter()
                    .zip(dst.iter())
                    .for_each(|(src, dst)| self.mappings.link(*src, *dst));

                mapped_count += 1;
            }
        }
        let mapping_time = mapping_start.elapsed();
        println!(
            "✓ Creating mappings: {:?} ({} mappings created)",
            mapping_time, mapped_count
        );

        let total_time = start_time.elapsed();
        println!(
            "=== STATEMENT LEVEL MATCHER COMPLETE: {:?} ===\n",
            total_time
        );
    }

    /// Execute naive approach without type grouping - compare all leaves
    fn execute_naive(&mut self) {
        let start_time = std::time::Instant::now();
        println!("=== NAIVE MATCHER START ===");

        let collect_start = std::time::Instant::now();
        let dst_leaves = self.collect_statement_leaves_dst();
        let src_leaves = self.collect_statement_leaves_src();
        let collect_time = collect_start.elapsed();
        println!(
            "✓ Statement leaf collection: {:?} (src: {}, dst: {})",
            collect_time,
            src_leaves.len(),
            dst_leaves.len()
        );

        let total_comparisons = src_leaves.len() * dst_leaves.len();
        println!("✓ Total comparisons needed: {}", total_comparisons);

        let mut leaves_mappings: Vec<MappingWithSimilarity<Dsrc, Ddst, M>> = Vec::new();

        let comparison_start = std::time::Instant::now();
        let mut comparison_count = 0;
        for src in &src_leaves {
            for dst in &dst_leaves {
                comparison_count += 1;
                // no need to check for equal types since all are only statements
                let sim = self.compute_label_similarity_simple(&src, &dst);
                if sim > self.config.base_config.label_sim_threshold {
                    leaves_mappings.push(MappingWithSimilarity {
                        src: src.clone(),
                        dst: dst.clone(),
                        sim,
                    });
                }
            }
        }
        let comparison_time = comparison_start.elapsed();
        println!(
            "✓ Simple similarity calculations: {:?} ({} comparisons, {} candidates)",
            comparison_time,
            comparison_count,
            leaves_mappings.len()
        );

        let sort_start = std::time::Instant::now();
        // Sort mappings by similarity
        leaves_mappings.sort_by(|a, b| b.sim.partial_cmp(&a.sim).unwrap_or(Ordering::Equal));
        let sort_time = sort_start.elapsed();
        println!("✓ Sorting candidates: {:?}", sort_time);

        let mapping_start = std::time::Instant::now();
        let mut mapped_count = 0;
        // Process mappings in order
        for mapping in leaves_mappings {
            if self
                .mappings
                .link_if_both_unmapped(mapping.src.shallow().clone(), mapping.dst.shallow().clone())
            {
                mapped_count += 1;
            }
        }
        let mapping_time = mapping_start.elapsed();
        println!(
            "✓ Creating mappings: {:?} ({} mappings created)",
            mapping_time, mapped_count
        );

        let total_time = start_time.elapsed();
        println!("=== NAIVE MATCHER COMPLETE: {:?} ===\n", total_time);
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
                if arena.decompress_children(node).is_empty() {
                    return true;
                }

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
                if arena.decompress_children(node).is_empty() {
                    return true;
                }
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
