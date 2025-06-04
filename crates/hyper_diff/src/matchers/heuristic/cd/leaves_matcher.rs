use crate::{
    decompressed_tree_store::{
        ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, POBorrowSlice,
        PostOrder, PostOrderIterable,
    },
    matchers::{
        heuristic::cd::iterator::{CustomIteratorConfig, DecompressedCustomPostOrderIterator},
        mapping_store::{MappingStore, MonoMappingStore},
    },
};
use hyperast::types::{
    DecompressedFrom, HyperAST, LabelStore, Labeled, NodeId, NodeStore, WithHashs,
};
use hyperast::{PrimInt, types::HyperType};
use std::cmp::Ordering;
use std::fmt::Debug;
use str_distance::DistanceMetric;

use super::LeavesMatcherConfig;

struct MappingWithSimilarity<M: MonoMappingStore> {
    src: M::Src,
    dst: M::Dst,
    sim: f64,
}

pub struct LeavesMatcher<Dsrc, Ddst, HAST, M> {
    pub(super) stores: HAST,
    pub src_arena: Dsrc,
    pub dst_arena: Ddst,
    pub mappings: M,
    pub config: LeavesMatcherConfig,
}

impl<
    Dsrc: DecompressedTreeStore<HAST, M::Src>
        + DecompressedWithParent<HAST, M::Src>
        + PostOrder<HAST, M::Src>
        + PostOrderIterable<HAST, M::Src>
        + DecompressedFrom<HAST, Out = Dsrc>
        + ContiguousDescendants<HAST, M::Src>
        + POBorrowSlice<HAST, M::Src>,
    Ddst: DecompressedTreeStore<HAST, M::Dst>
        + DecompressedWithParent<HAST, M::Dst>
        + PostOrder<HAST, M::Dst>
        + PostOrderIterable<HAST, M::Dst>
        + DecompressedFrom<HAST, Out = Ddst>
        + ContiguousDescendants<HAST, M::Dst>
        + POBorrowSlice<HAST, M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore,
> LeavesMatcher<Dsrc, Ddst, HAST, M>
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
    M::Src: PrimInt,
    M::Dst: PrimInt,
    HAST::Label: Eq,
    HAST::IdN: Debug,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    pub fn with_config(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
        config: LeavesMatcherConfig,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M> {
        let mut matcher = Self {
            stores: mapping.hyperast,
            src_arena: mapping.mapping.src_arena,
            dst_arena: mapping.mapping.dst_arena,
            mappings: mapping.mapping.mappings,
            config,
        };
        // Rest of the code remains the same
        matcher
            .mappings
            .topit(matcher.src_arena.len(), matcher.dst_arena.len());
        matcher.execute();
        // Return the mapper
        crate::matchers::Mapper {
            hyperast: mapping.hyperast,
            mapping: crate::matchers::Mapping {
                src_arena: matcher.src_arena,
                dst_arena: matcher.dst_arena,
                mappings: matcher.mappings,
            },
        }
    }

    pub fn match_it(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M> {
        Self::with_config(mapping, LeavesMatcherConfig::default())
    }

    fn execute(&mut self) {
        let start_time = std::time::Instant::now();

        let collect_start = std::time::Instant::now();
        let dst_leaves = self.get_dst_nodes();
        let src_leaves = self.get_src_nodes();
        let collect_time = collect_start.elapsed();
        log::trace!(
            "✓ Leaf collection: {:?} (src: {}, dst: {})",
            collect_time,
            src_leaves.len(),
            dst_leaves.len()
        );

        let mut leaves_mappings: Vec<MappingWithSimilarity<M>> = Vec::new();
        let total_comparisons = src_leaves.len() * dst_leaves.len();
        log::trace!("✓ Total comparisons needed: {}", total_comparisons);

        let comparison_start = std::time::Instant::now();
        let mut comparison_count = 0;
        let mut allowed_count = 0;
        for &src_leaf in &src_leaves {
            for &dst_leaf in &dst_leaves {
                comparison_count += 1;
                if self.is_mapping_allowed(&src_leaf, &dst_leaf) {
                    allowed_count += 1;
                    let sim = self.compute_label_similarity(&src_leaf, &dst_leaf);
                    if sim > self.config.label_sim_threshold {
                        leaves_mappings.push(MappingWithSimilarity {
                            src: src_leaf,
                            dst: dst_leaf,
                            sim,
                        });
                    }
                }
            }
        }
        let comparison_time = comparison_start.elapsed();
        log::trace!(
            "✓ Similarity calculations: {:?} ({} total, {} allowed, {} candidates)",
            comparison_time,
            comparison_count,
            allowed_count,
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
                .link_if_both_unmapped(mapping.src, mapping.dst)
            {
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
    }

    fn get_src_nodes(&self) -> Vec<<M as MappingStore>::Src> {
        let iter = DecompressedCustomPostOrderIterator::new(
            &self.src_arena,
            self.stores,
            self.src_arena.root(),
            CustomIteratorConfig {
                yield_leaves: false,
                yield_inner: true,
            },
            |arena: &Dsrc, stores: HAST, node: &<M as MappingStore>::Src| -> bool {
                if arena.children(node).is_empty() {
                    return true;
                }
                let original = arena.original(node);
                let node_type = stores.resolve_type(&original);
                return node_type.is_statement();
            },
        );
        iter.collect::<Vec<_>>()
    }

    fn get_dst_nodes(&self) -> Vec<<M as MappingStore>::Dst> {
        let iter = DecompressedCustomPostOrderIterator::new(
            &self.dst_arena,
            self.stores,
            self.dst_arena.root(),
            CustomIteratorConfig {
                yield_leaves: false,
                yield_inner: true,
            },
            |arena: &Ddst, stores: HAST, node: &<M as MappingStore>::Dst| -> bool {
                if arena.children(node).is_empty() {
                    return true;
                }
                let original = arena.original(node);
                let node_type = stores.resolve_type(&original);
                return node_type.is_statement();
            },
        );
        iter.collect::<Vec<_>>()
    }

    fn is_mapping_allowed(&self, src_tree: &M::Src, dst_tree: &M::Dst) -> bool {
        let src_linked = self.mappings.get_src(dst_tree).is_some();
        let dst_linked = self.mappings.get_dst(src_tree).is_some();

        if src_linked || dst_linked {
            return false;
        }

        let original_src = self.src_arena.original(src_tree);
        let original_dst = self.dst_arena.original(dst_tree);

        let src_type = self.stores.resolve_type(&original_src);
        let dst_type = self.stores.resolve_type(&original_dst);

        src_type == dst_type
    }

    fn compute_label_similarity(&self, src_tree: &M::Src, dst_tree: &M::Dst) -> f64 {
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
    use hyperast::nodes::SyntaxSerializer;
    use hyperast::test_utils::simple_tree::DisplayTree;
    use hyperast::types::WithChildren;

    use super::*;
    use crate::decompressed_tree_store::{CompletePostOrder, ShallowDecompressedTreeStore};
    use crate::matchers::Decompressible;
    use crate::matchers::mapping_store::MappingStore;
    use crate::matchers::{Mapper, mapping_store::DefaultMappingStore};
    use crate::tree::simple_tree::vpair_to_stores;

    #[test]
    #[ignore]
    fn test_leaves_matcher_manual() {
        let (stores, src, dst) = vpair_to_stores(crate::tests::examples::example_gt_java_code());

        let mapping = Mapper {
            hyperast: &stores,
            mapping: crate::matchers::Mapping {
                src_arena: Decompressible::<_, CompletePostOrder<_, u16>>::decompress(
                    &stores, &src,
                ),
                dst_arena: Decompressible::<_, CompletePostOrder<_, u16>>::decompress(
                    &stores, &dst,
                ),
                mappings: DefaultMappingStore::default(),
            },
        };

        let result = LeavesMatcher::match_it(mapping);

        let src_fmt = |src: u16| result.src_arena.original(&src).to_string();
        let dst_fmt = |dst: u16| result.dst_arena.original(&dst).to_string();
        let display_vec = result.mapping.mappings.display(&src_fmt, &dst_fmt);
        log::trace!("Mappings:\n{}", display_vec);

        assert!(result.mapping.mappings.src_to_dst.len() > 0);
    }

    #[test]
    fn test_leaves_matcher() {
        let (stores, src, dst) = vpair_to_stores(crate::tests::examples::example_leaf_label_swap());

        log::trace!(
            "Src Tree:\n{}",
            DisplayTree::new(&stores.label_store, &stores.node_store, src)
        );

        log::trace!(
            "Dst Tree:\n{}",
            DisplayTree::new(&stores.label_store, &stores.node_store, dst)
        );
        log::trace!("Src Tree:\n{}", SyntaxSerializer::new(&stores, src));
        log::trace!("Dst Tree:\n{}", SyntaxSerializer::new(&stores, dst));

        let mapping = Mapper {
            hyperast: &stores,
            mapping: crate::matchers::Mapping {
                src_arena: Decompressible::<_, CompletePostOrder<_, u16>>::decompress(
                    &stores, &src,
                ),
                dst_arena: Decompressible::<_, CompletePostOrder<_, u16>>::decompress(
                    &stores, &dst,
                ),
                mappings: DefaultMappingStore::default(),
            },
        };

        let result = LeavesMatcher::match_it(mapping);

        assert_eq!(2, result.mappings.len());
        log::trace!("Mappings: {:?}", result.mappings);
        assert!(HyperAST::resolve(&stores, &dst).child(&0).is_some());
        assert!(HyperAST::resolve(&stores, &dst).child(&1).is_some());
        assert!(HyperAST::resolve(&stores, &src).child(&0).is_some());
        assert!(HyperAST::resolve(&stores, &src).child(&1).is_some());

        log::trace!(
            "Src Children: {:?}",
            HyperAST::resolve(&stores, &src).children()
        );
        log::trace!(
            "Dst Children: {:?}",
            HyperAST::resolve(&stores, &dst).children()
        );

        use crate::decompressed_tree_store::ShallowDecompressedTreeStore;
        let src = result.mapping.src_arena.root();
        let src_cs = result.mapping.src_arena.children(&src);
        let dst = result.mapping.dst_arena.root();
        let dst_cs = result.mapping.dst_arena.children(&dst);

        assert!(result.mapping.mappings.has(&src_cs[0], &dst_cs[1]));
        assert!(result.mapping.mappings.has(&src_cs[1], &dst_cs[0]));
    }
}
