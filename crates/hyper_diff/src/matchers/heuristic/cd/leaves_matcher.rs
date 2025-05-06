use crate::{
    decompressed_tree_store::{
        ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, POBorrowSlice,
        PostOrder, PostOrderIterable,
    },
    matchers::mapping_store::MonoMappingStore,
};
use hyperast::types::{DecompressedFrom, HyperAST, NodeId, NodeStore, WithHashs};
use hyperast::{PrimInt, types::Labeled};
use std::cmp::Ordering;
use std::fmt::Debug;

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
    pub label_sim_threshold: f64,
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
    pub fn match_it(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M> {
        let mut matcher = Self {
            stores: mapping.hyperast,
            src_arena: mapping.mapping.src_arena,
            dst_arena: mapping.mapping.dst_arena,
            mappings: mapping.mapping.mappings,
            label_sim_threshold: 0.5, // Default threshold
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

    fn execute(&mut self) {
        let mut leaves_mappings: Vec<MappingWithSimilarity<M>> = Vec::new();
        let dst_leaves: Vec<M::Dst> = self
            .dst_arena
            .iter_df_post::<true>()
            .filter(|t| self.dst_arena.children(t).is_empty())
            .collect();

        // Collect potential mappings
        for src_leaf in self
            .src_arena
            .iter_df_post::<true>()
            .filter(|t| self.src_arena.children(t).is_empty())
        {
            for &dst_leaf in &dst_leaves {
                if self.is_mapping_allowed(&src_leaf, &dst_leaf) {
                    let sim = self.compute_label_similarity(&src_leaf, &dst_leaf);
                    if sim > self.label_sim_threshold {
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

        let src_label = src_node.try_get_label();
        let dst_label = dst_node.try_get_label();

        match (src_label, dst_label) {
            (Some(src_label), Some(dst_label)) => {
                if src_label == dst_label {
                    1.0
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompressed_tree_store::CompletePostOrder;
    use crate::matchers::{Decompressible, Mapper, mapping_store::DefaultMappingStore};
    use crate::tests::examples::example_simple;
    use crate::tree::simple_tree::vpair_to_stores;

    #[test]
    fn test_leaves_matcher() {
        let (stores, src, dst) = vpair_to_stores(example_simple());

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

        assert!(result.mapping.mappings.src_to_dst.len() > 0);
    }
}
