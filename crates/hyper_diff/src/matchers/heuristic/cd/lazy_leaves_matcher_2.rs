use crate::{
    decompressed_tree_store::{
        ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, LazyDecompressed,
        LazyDecompressedTreeStore, LazyPOBorrowSlice, POBorrowSlice, PostOrder, PostOrderIterable,
        Shallow, ShallowDecompressedTreeStore,
    },
    matchers::mapping_store::MonoMappingStore,
};
use hyperast::PrimInt;
use hyperast::types::{
    DecompressedFrom, HyperAST, LabelStore, Labeled, NodeId, NodeStore, WithHashs,
};
use std::fmt::Debug;
use std::{
    cmp::{Ordering, Reverse},
    collections::BinaryHeap,
};
use str_distance::DistanceMetric;

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

pub struct LazyLeavesMatcher<Dsrc, Ddst, HAST, M> {
    pub(super) stores: HAST,
    pub src_arena: Dsrc,
    pub dst_arena: Ddst,
    pub mappings: M,
    pub label_sim_threshold: f64,
}

impl<
    Dsrc: LazyDecompressed<M::Src>,
    Ddst: LazyDecompressed<M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore,
> LazyLeavesMatcher<Dsrc, Ddst, HAST, M>
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
    M::Src: PrimInt,
    M::Dst: PrimInt,
    HAST::Label: Eq,
    HAST::IdN: Debug,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
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

        // Get all leaves and index them by node type
        let mut src_leaves_by_type: std::collections::HashMap<_, Vec<M::Src>> =
            std::collections::HashMap::new();
        let mut dst_leaves_by_type: std::collections::HashMap<_, Vec<M::Dst>> =
            std::collections::HashMap::new();

        // Collect src leaves and index by type
        for src_leaf in src_leaves {
            let src = self.src_arena.decompress_to(&src_leaf);
            let original_src = self.src_arena.original(&src);
            let src_type = self.stores.resolve_type(&original_src);

            src_leaves_by_type
                .entry(src_type)
                .or_insert_with(Vec::new)
                .push(src_leaf);
        }

        // Collect dst leaves and index by type
        for dst_leaf in dst_leaves {
            let dst = self.dst_arena.decompress_to(&dst_leaf);
            let original_dst = self.dst_arena.original(&dst);
            let dst_type = self.stores.resolve_type(&original_dst);

            dst_leaves_by_type
                .entry(dst_type)
                .or_insert_with(Vec::new)
                .push(dst_leaf);
        }

        // Use binary heap to keep track of best mappings
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
                            let sim = self.compute_label_similarity(&src, &dst);
                            if sim > self.label_sim_threshold {
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

    fn compute_label_similarity(&self, src_tree: &Dsrc::IdD, dst_tree: &Ddst::IdD) -> f64 {
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
    use hyperast::nodes::SyntaxSerializer;
    use hyperast::test_utils::simple_tree::DisplayTree;
    use hyperast::types::WithChildren;

    #[test]
    fn test_leaves_matcher() {
        let (stores, src, dst) = vpair_to_stores(crate::tests::examples::example_leaf_label_swap());

        println!(
            "Src Tree:\n{}",
            DisplayTree::new(&stores.label_store, &stores.node_store, src)
        );

        println!(
            "Dst Tree:\n{}",
            DisplayTree::new(&stores.label_store, &stores.node_store, dst)
        );
        println!("Src Tree:\n{}", SyntaxSerializer::new(&stores, src));
        println!("Dst Tree:\n{}", SyntaxSerializer::new(&stores, dst));

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

        let result = LazyLeavesMatcher::match_it(mapping);

        assert_eq!(2, result.mappings.len());
        println!("Mappings: {:?}", result.mappings);
        assert!(HyperAST::resolve(&stores, &dst).child(&0).is_some());
        assert!(HyperAST::resolve(&stores, &dst).child(&1).is_some());
        assert!(HyperAST::resolve(&stores, &src).child(&0).is_some());
        assert!(HyperAST::resolve(&stores, &src).child(&1).is_some());

        println!(
            "Src Children: {:?}",
            HyperAST::resolve(&stores, &src).children()
        );
        println!(
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
