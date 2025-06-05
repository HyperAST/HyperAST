use crate::{
    decompressed_tree_store::{
        ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, LazyDecompressed,
        LazyDecompressedTreeStore, LazyPOBorrowSlice, PostOrder, PostOrderIterable, Shallow,
        ShallowDecompressedTreeStore,
    },
    matchers::{
        Mapper, Mapping, heuristic::cd::BottomUpMatcherMetrics, mapping_store::MonoMappingStore,
        similarity_metrics,
    },
};
use ahash::RandomState;
use hyperast::types::{HyperAST, HyperType, NodeId, WithHashs};
use hyperast::{PrimInt, types::TypeStore};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use super::{
    OptimizedBottomUpMatcherConfig,
    iterator::{CustomIteratorConfig, CustomPostOrderIterator},
};

/// Optimized bottom-up matcher with configurable optimizations
pub struct OptimizedBottomUpMatcher<Dsrc, Ddst, HAST, M: MonoMappingStore> {
    pub stores: HAST,
    pub src_arena: Dsrc,
    pub dst_arena: Ddst,
    pub mappings: M,
    pub config: OptimizedBottomUpMatcherConfig,
    pub metrics: BottomUpMatcherMetrics,
}

impl<
    Dsrc: LazyDecompressed<M::Src>,
    Ddst: LazyDecompressed<M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore,
> OptimizedBottomUpMatcher<Dsrc, Ddst, HAST, M>
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
    M::Src: PrimInt,
    M::Dst: PrimInt,
    HAST::Label: Eq,
    HAST::IdN: Debug + NodeId<IdN = HAST::IdN>,
    <HAST::TS as TypeStore>::Ty: Hash + Eq + Clone + HyperType,
    Dsrc::IdD: std::fmt::Debug + Clone,
    Ddst::IdD: std::fmt::Debug + Clone,
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
        mapping: Mapper<HAST, Dsrc, Ddst, M>,
        config: OptimizedBottomUpMatcherConfig,
    ) -> Mapper<HAST, Dsrc, Ddst, M> {
        let mut matcher = Self {
            stores: mapping.hyperast,
            mappings: mapping.mapping.mappings,
            src_arena: mapping.mapping.src_arena,
            dst_arena: mapping.mapping.dst_arena,
            config,
            metrics: BottomUpMatcherMetrics::default(),
        };
        matcher
            .mappings
            .topit(matcher.src_arena.len(), matcher.dst_arena.len());
        let start_time = std::time::Instant::now();
        matcher.execute();
        matcher.metrics.total_time = start_time.elapsed();
        matcher.into()
    }

    /// Create matcher with custom configuration and return metrics
    pub fn with_config_and_metrics(
        mapping: Mapper<HAST, Dsrc, Ddst, M>,
        config: OptimizedBottomUpMatcherConfig,
    ) -> (Mapper<HAST, Dsrc, Ddst, M>, BottomUpMatcherMetrics) {
        let mut matcher = Self {
            stores: mapping.hyperast,
            mappings: mapping.mapping.mappings,
            src_arena: mapping.mapping.src_arena,
            dst_arena: mapping.mapping.dst_arena,
            config,
            metrics: BottomUpMatcherMetrics::default(),
        };
        matcher
            .mappings
            .topit(matcher.src_arena.len(), matcher.dst_arena.len());

        let start = std::time::Instant::now();
        matcher.execute();
        matcher.metrics.total_time = start.elapsed();

        let metrics = matcher.metrics.clone();
        let mapper = matcher.into();
        (mapper, metrics)
    }

    /// Create matcher with default optimized configuration
    pub fn match_it(mapping: Mapper<HAST, Dsrc, Ddst, M>) -> Mapper<HAST, Dsrc, Ddst, M> {
        Self::with_config(mapping, OptimizedBottomUpMatcherConfig::default())
    }

    /// Execute the bottom-up matching with configured optimizations
    fn execute(&mut self) {
        if self.config.statement_level_iteration {
            self.execute_statement_level();
        } else if self.config.enable_type_grouping {
            self.execute_with_type_grouping();
        } else {
            self.execute_naive();
        }
    }

    /// Execute with type grouping and leaf count pre-computation optimizations
    fn execute_with_type_grouping(&mut self) {
        // Always pre-compute leaf counts when using type grouping for optimal performance
        let mut leaf_counts: HashMap<M::Src, usize> = HashMap::new();

        // Process nodes in post-order so children are processed before parents
        for s in self.src_arena.iter_df_post::<true>() {
            let src = self.src_arena.decompress_to(&s);
            let is_leaf = self.src_arena.children(&src).is_empty();

            let leaf_count = if is_leaf {
                1 // Leaf nodes have a count of 1
            } else {
                // Sum the leaf counts of all children
                self.src_arena
                    .children(&src)
                    .iter()
                    .map(|child| {
                        let child = self.src_arena.decompress_to(child);
                        leaf_counts.get(child.shallow()).copied().unwrap_or(0)
                    })
                    .sum()
            };

            leaf_counts.insert(*src.shallow(), leaf_count);
        }

        let mut src_nodes_by_type: HashMap<_, Vec<_>, RandomState> = HashMap::default();
        let mut dst_nodes_by_type: HashMap<_, Vec<_>, RandomState> = HashMap::default();

        // Index source nodes by their type (only unmapped non-leaf nodes)
        for s in self.src_arena.iter_df_post::<true>() {
            let src = self.src_arena.decompress_to(&s);
            let src_type = self.stores.resolve_type(&self.src_arena.original(&src));

            let is_leaf = self.src_arena.children(&src).is_empty();
            let is_mapped = self.mappings.is_src(src.shallow());

            if !is_mapped && !is_leaf {
                src_nodes_by_type.entry(src_type).or_default().push(src);
            }
        }

        // Index destination nodes by their type (only unmapped non-leaf nodes)
        for d in self.dst_arena.iter_df_post::<true>() {
            let dst = self.dst_arena.decompress_to(&d);
            let dst_type = self.stores.resolve_type(&self.dst_arena.original(&dst));

            let is_leaf = self.dst_arena.children(&dst).is_empty();
            let is_mapped = self.mappings.is_dst(dst.shallow());

            if !is_mapped && !is_leaf {
                dst_nodes_by_type.entry(dst_type).or_default().push(dst);
            }
        }

        // Process nodes by type, comparing only nodes with matching types
        let mut total_comparisons = 0;
        let mut successful_matches = 0;
        let mut similarity_time = std::time::Duration::ZERO;

        for (node_type, src_nodes) in src_nodes_by_type.iter() {
            if let Some(dst_nodes) = dst_nodes_by_type.get(node_type) {
                for src in src_nodes {
                    // Skip if the source node is already mapped
                    if self.mappings.is_src(src.shallow()) {
                        continue;
                    }

                    let number_of_leaves = *leaf_counts.get(src.shallow()).unwrap_or(&0);
                    let threshold = if number_of_leaves > self.config.base.max_leaves {
                        self.config.base.sim_threshold_large_trees
                    } else {
                        self.config.base.sim_threshold_small_trees
                    };

                    for dst in dst_nodes {
                        // Skip if the destination node is already mapped
                        if self.mappings.is_dst(dst.shallow()) {
                            continue;
                        }

                        total_comparisons += 1;

                        // Use range-based similarity computation for optimal performance
                        let sim_start = std::time::Instant::now();
                        let similarity = similarity_metrics::SimilarityMeasure::range(
                            &self.src_arena.descendants_range(src),
                            &self.dst_arena.descendants_range(dst),
                            &self.mappings,
                        )
                        .chawathe();
                        similarity_time += sim_start.elapsed();

                        if similarity >= threshold {
                            self.mappings.link(*src.shallow(), *dst.shallow());
                            successful_matches += 1;
                            break;
                        }
                    }
                }
            }
        }

        // Update metrics
        self.metrics.total_comparisons += total_comparisons;
        self.metrics.successful_matches += successful_matches;
        self.metrics.similarity_time += similarity_time;
    }

    /// Execute with up to statement level iteration
    fn execute_statement_level(&mut self) {
        let dst_nodes = self.collect_statement_inner_dst();
        let src_nodes = self.collect_statement_inner_src(false, true);

        let leaf_counts = self.get_leaf_counts(&src_nodes);

        let mut total_comparisons = 0;
        let mut successful_matches = 0;
        let mut similarity_time = std::time::Duration::ZERO;

        for src in &src_nodes {
            let threshold =
                if leaf_counts.get(src.shallow()).unwrap_or(&0) > &self.config.base.max_leaves {
                    self.config.base.sim_threshold_large_trees
                } else {
                    self.config.base.sim_threshold_small_trees
                };

            for dst in &dst_nodes {
                if self.is_mapping_allowed(&src, &dst) {
                    // no need to check if both are leaves since we only iterate over inner nodes
                    total_comparisons += 1;

                    let sim_start = std::time::Instant::now();
                    let similarity = self.compute_similarity(&src, &dst);
                    similarity_time += sim_start.elapsed();

                    if similarity >= threshold {
                        self.mappings.link(*src.shallow(), *dst.shallow());
                        successful_matches += 1;
                        break;
                    }
                }
            }
        }

        // Update metrics
        self.metrics.total_comparisons += total_comparisons;
        self.metrics.successful_matches += successful_matches;
        self.metrics.similarity_time += similarity_time;
    }

    fn get_leaf_counts(
        &mut self,
        nodes: &[<Dsrc as LazyDecompressed<M::Src>>::IdD],
    ) -> HashMap<M::Src, usize> {
        let src_leaves = self.collect_statement_inner_src(true, false);

        let mut leaf_counts: HashMap<M::Src, usize> = HashMap::new();

        for src in &src_leaves {
            leaf_counts.insert(*src.shallow(), 1);
        }

        // Process nodes in post-order so children are processed before parents
        for src in nodes {
            let leaf_count = self
                .src_arena
                .children(&src)
                .iter()
                .map(|child| leaf_counts.get(child).copied().unwrap_or(0))
                .sum();

            leaf_counts.insert(*src.shallow(), leaf_count);
        }

        leaf_counts
    }

    fn collect_statement_inner_src(
        &mut self,
        leaves: bool,
        inner: bool,
    ) -> Vec<<Dsrc as LazyDecompressed<M::Src>>::IdD> {
        let src_root = self.src_arena.starter();

        let iter = CustomPostOrderIterator::new(
            &mut self.src_arena,
            self.stores,
            src_root,
            CustomIteratorConfig::inner(self.config.enable_deep_leaves),
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

    fn collect_statement_inner_dst(&mut self) -> Vec<<Ddst as LazyDecompressed<M::Dst>>::IdD> {
        let dst_root = self.dst_arena.starter();

        let iter = CustomPostOrderIterator::new(
            &mut self.dst_arena,
            self.stores,
            dst_root,
            CustomIteratorConfig::inner(self.config.enable_deep_leaves),
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

    /// Execute naive approach without optimizations - compare all nodes
    fn execute_naive(&mut self) {
        let mut total_comparisons = 0;
        let mut successful_matches = 0;
        let mut similarity_time = std::time::Duration::ZERO;

        for s in self.src_arena.iter_df_post::<true>() {
            let src = self.src_arena.decompress_to(&s);
            let number_of_leaves = self
                .src_arena
                .descendants(&src)
                .iter()
                .filter(|t| {
                    let id = self.src_arena.decompress_to(&t);
                    self.src_arena.children(&id).is_empty()
                })
                .count();

            for d in self.dst_arena.iter_df_post::<true>() {
                let dst = self.dst_arena.decompress_to(&d);

                if self.is_mapping_allowed(&src, &dst) {
                    let src_is_leaf = self.src_arena.children(&src).is_empty();
                    let dst_is_leaf = self.dst_arena.children(&dst).is_empty();

                    if !(src_is_leaf || dst_is_leaf) {
                        total_comparisons += 1;

                        let sim_start = std::time::Instant::now();
                        let similarity = self.compute_similarity(&src, &dst);
                        similarity_time += sim_start.elapsed();

                        let threshold = if number_of_leaves > self.config.base.max_leaves {
                            self.config.base.sim_threshold_large_trees
                        } else {
                            self.config.base.sim_threshold_small_trees
                        };

                        if similarity >= threshold {
                            self.mappings.link(*src.shallow(), *dst.shallow());
                            successful_matches += 1;
                            break;
                        }
                    }
                }
            }
        }

        // Update metrics
        self.metrics.total_comparisons += total_comparisons;
        self.metrics.successful_matches += successful_matches;
        self.metrics.similarity_time += similarity_time;
    }

    /// Check if mapping between two nodes is allowed (same type, both unmapped)
    fn is_mapping_allowed(&self, src: &Dsrc::IdD, dst: &Ddst::IdD) -> bool {
        if self.mappings.is_src(src.shallow()) || self.mappings.is_dst(dst.shallow()) {
            return false;
        }

        let src_type = self.stores.resolve_type(&self.src_arena.original(src));
        let dst_type = self.stores.resolve_type(&self.dst_arena.original(dst));

        src_type == dst_type
    }

    /// Compute similarity between two nodes using configured method
    fn compute_similarity(&self, src: &Dsrc::IdD, dst: &Ddst::IdD) -> f64 {
        // Always use range-based similarity for lazy decompressed trees as it's more efficient
        similarity_metrics::SimilarityMeasure::range(
            &self.src_arena.descendants_range(src),
            &self.dst_arena.descendants_range(dst),
            &self.mappings,
        )
        .chawathe()
    }
}

impl<HAST: HyperAST + Copy, Dsrc, Ddst, M: MonoMappingStore> Into<Mapper<HAST, Dsrc, Ddst, M>>
    for OptimizedBottomUpMatcher<Dsrc, Ddst, HAST, M>
{
    fn into(self) -> Mapper<HAST, Dsrc, Ddst, M> {
        Mapper {
            hyperast: self.stores,
            mapping: Mapping {
                src_arena: self.src_arena,
                dst_arena: self.dst_arena,
                mappings: self.mappings,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompressed_tree_store::lazy_post_order::LazyPostOrder;
    use crate::decompressed_tree_store::{CompletePostOrder, ShallowDecompressedTreeStore};
    use crate::matchers::Decompressible;
    use crate::matchers::mapping_store::MappingStore;
    use crate::matchers::{Mapper, mapping_store::DefaultMappingStore};
    use crate::tests::tree;
    use crate::tree::simple_tree::vpair_to_stores;

    use hyperast::types::DecompressedFrom;

    #[test]
    fn test_optimized_bottom_up_matcher_all_optimizations() {
        // Setup simple source and destination trees with similar structure
        let src = tree!(
            0,"a"; [
                tree!(0, "e"; [
                    tree!(0, "f")]),
                tree!(0, "b"; [
                    tree!(0, "c"),
                    tree!(0, "d")]),
        ]);
        let dst = tree!(
            0,"a"; [
                tree!(0, "e"; [
                    tree!(0, "g")]),
                tree!(0, "b"; [
                    tree!(0, "c"),
                    tree!(0, "d")]),
        ]);

        let (stores, src, dst) = vpair_to_stores((src, dst));

        let mut src_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &src);
        let mut dst_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &dst);

        let src_arena_decomp =
            Decompressible::<_, CompletePostOrder<_, u16>>::decompress(&stores, &src);
        let dst_arena_decomp =
            Decompressible::<_, CompletePostOrder<_, u16>>::decompress(&stores, &dst);

        let mut mappings = DefaultMappingStore::default();
        mappings.topit(src_arena.len(), dst_arena.len());

        let src_root = src_arena_decomp.root();
        let dst_root = dst_arena_decomp.root();
        let src_node_c = src_arena_decomp.child(&src_root, &[1, 0]);
        let src_node_d = src_arena_decomp.child(&src_root, &[1, 1]);
        let dst_node_c = dst_arena_decomp.child(&dst_root, &[1, 0]);
        let dst_node_d = dst_arena_decomp.child(&dst_root, &[1, 1]);

        mappings.link(src_node_c, dst_node_c);
        mappings.link(src_node_d, dst_node_d);

        let mapping = Mapper {
            hyperast: &stores,
            mapping: crate::matchers::Mapping {
                src_arena: src_arena.as_mut(),
                dst_arena: dst_arena.as_mut(),
                mappings,
            },
        };

        let result = OptimizedBottomUpMatcher::match_it(mapping);

        assert_eq!(
            result.mappings.len(),
            4,
            "Expected exactly 4 mappings, got {}",
            result.mappings.len()
        );

        let src_root = result.mapping.src_arena.root();
        let dst_root = result.mapping.dst_arena.root();

        assert!(
            result.mapping.mappings.has(&src_root, &dst_root),
            "Root nodes should be mapped"
        );
    }

    #[test]
    fn test_optimized_bottom_up_matcher_no_optimizations() {
        let src = tree!(
            0,"a"; [
                tree!(0, "e"; [
                    tree!(0, "f")]),
                tree!(0, "b"; [
                    tree!(0, "c"),
                    tree!(0, "d")]),
        ]);
        let dst = tree!(
            0,"a"; [
                tree!(0, "e"; [
                    tree!(0, "g")]),
                tree!(0, "b"; [
                    tree!(0, "c"),
                    tree!(0, "d")]),
        ]);

        let (stores, src, dst) = vpair_to_stores((src, dst));

        let mut src_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &src);
        let mut dst_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &dst);

        let src_arena_decomp =
            Decompressible::<_, CompletePostOrder<_, u16>>::decompress(&stores, &src);
        let dst_arena_decomp =
            Decompressible::<_, CompletePostOrder<_, u16>>::decompress(&stores, &dst);

        let mut mappings = DefaultMappingStore::default();
        mappings.topit(src_arena.len(), dst_arena.len());

        let src_root = src_arena_decomp.root();
        let dst_root = dst_arena_decomp.root();
        let src_node_c = src_arena_decomp.child(&src_root, &[1, 0]);
        let src_node_d = src_arena_decomp.child(&src_root, &[1, 1]);
        let dst_node_c = dst_arena_decomp.child(&dst_root, &[1, 0]);
        let dst_node_d = dst_arena_decomp.child(&dst_root, &[1, 1]);

        mappings.link(src_node_c, dst_node_c);
        mappings.link(src_node_d, dst_node_d);

        let config = OptimizedBottomUpMatcherConfig {
            base: super::super::BottomUpMatcherConfig::default(),
            enable_type_grouping: false,
            enable_deep_leaves: false,
            statement_level_iteration: false,
            enable_leaf_count_precomputation: false,
        };

        let mapping = Mapper {
            hyperast: &stores,
            mapping: crate::matchers::Mapping {
                src_arena: src_arena.as_mut(),
                dst_arena: dst_arena.as_mut(),
                mappings,
            },
        };

        let result = OptimizedBottomUpMatcher::with_config(mapping, config);

        assert_eq!(
            result.mappings.len(),
            4,
            "Expected exactly 4 mappings, got {}",
            result.mappings.len()
        );

        let src_root = result.mapping.src_arena.root();
        let dst_root = result.mapping.dst_arena.root();

        assert!(
            result.mapping.mappings.has(&src_root, &dst_root),
            "Root nodes should be mapped"
        );
    }
}
