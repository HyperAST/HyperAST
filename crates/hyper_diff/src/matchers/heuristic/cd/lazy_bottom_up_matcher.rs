use crate::{
    decompressed_tree_store::{
        ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, LazyDecompressed,
        LazyDecompressedTreeStore, LazyPOBorrowSlice, PostOrder, PostOrderIterable, Shallow,
        ShallowDecompressedTreeStore,
    },
    matchers::{Mapper, Mapping, mapping_store::MonoMappingStore, similarity_metrics},
};
use hyperast::PrimInt;
use hyperast::types::{DecompressedFrom, HyperAST, NodeId, WithHashs};
use num_traits::cast;
use std::fmt::Debug;

const MAX_LEAVES: usize = 4;
const SIM_THRESHOLD_LARGE_TREES: f64 = 0.6;
const SIM_THRESHOLD_SMALL_TREES: f64 = 0.4;

pub struct LazyBottomUpMatcher<Dsrc, Ddst, HAST, M: MonoMappingStore> {
    pub hyperast: HAST,
    pub src_arena: Dsrc,
    pub dst_arena: Ddst,
    pub mappings: M,
}

impl<
    Dsrc: LazyDecompressed<M::Src>,
    Ddst: LazyDecompressed<M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore,
> LazyBottomUpMatcher<Dsrc, Ddst, HAST, M>
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
    M::Src: PrimInt,
    M::Dst: PrimInt,
    HAST::Label: Eq,
    HAST::IdN: Debug + NodeId<IdN = HAST::IdN>,
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
    pub fn match_it(mapping: Mapper<HAST, Dsrc, Ddst, M>) -> Mapper<HAST, Dsrc, Ddst, M> {
        let mut matcher = Self {
            hyperast: mapping.hyperast,
            mappings: mapping.mapping.mappings,
            src_arena: mapping.mapping.src_arena,
            dst_arena: mapping.mapping.dst_arena,
        };
        matcher
            .mappings
            .topit(matcher.src_arena.len(), matcher.dst_arena.len());
        matcher.execute();
        matcher.into()
    }

    fn execute(&mut self) {
        for s in self.src_arena.iter_df_post::<false>() {
            let src = self.src_arena.decompress_to(&s);
            let number_of_leaves = self.count_leaves(&src);

            for d in self.dst_arena.iter_df_post::<false>() {
                let dst = self.dst_arena.decompress_to(&d);
                if self.is_mapping_allowed(&src, &dst) {
                    let src_is_leaf = self.src_arena.children(&src).is_empty();
                    let dst_is_leaf = self.dst_arena.children(&dst).is_empty();

                    if !(src_is_leaf || dst_is_leaf) {
                        let similarity = self.compute_similarity(&src, &dst);
                        let threshold = if number_of_leaves > MAX_LEAVES {
                            SIM_THRESHOLD_LARGE_TREES
                        } else {
                            SIM_THRESHOLD_SMALL_TREES
                        };

                        if similarity >= threshold {
                            self.mappings.link(*src.shallow(), *dst.shallow());
                            break;
                        }
                    }
                }
            }
        }
    }

    fn count_leaves(&mut self, src: &Dsrc::IdD) -> usize {
        self.src_arena
            .descendants(src)
            .iter()
            .filter(|t| {
                let id = self.src_arena.decompress_to(&t);
                self.src_arena.children(&id).is_empty()
            })
            .count()
    }

    fn is_mapping_allowed(&self, src: &Dsrc::IdD, dst: &Ddst::IdD) -> bool {
        if self.mappings.is_src(src.shallow()) || self.mappings.is_dst(dst.shallow()) {
            return false;
        }

        let src_type = self.hyperast.resolve_type(&self.src_arena.original(src));
        let dst_type = self.hyperast.resolve_type(&self.dst_arena.original(dst));

        src_type == dst_type
    }

    fn compute_similarity(&self, src: &Dsrc::IdD, dst: &Ddst::IdD) -> f64 {
        // Using the optimized range-based similarity computation
        similarity_metrics::SimilarityMeasure::range(
            &self.src_arena.descendants_range(src),
            &self.dst_arena.descendants_range(dst),
            &self.mappings,
        )
        .chawathe()
    }
}

impl<HAST: HyperAST + Copy, Dsrc, Ddst, M: MonoMappingStore> Into<Mapper<HAST, Dsrc, Ddst, M>>
    for LazyBottomUpMatcher<Dsrc, Ddst, HAST, M>
{
    fn into(self) -> Mapper<HAST, Dsrc, Ddst, M> {
        Mapper {
            hyperast: self.hyperast,
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
    use crate::matchers::Decompressible;
    use crate::matchers::mapping_store::MappingStore;
    use crate::matchers::{Mapper, mapping_store::DefaultMappingStore};
    use crate::tests::simple_examples;
    use crate::tree::simple_tree::vpair_to_stores;
    use hyperast::nodes::SyntaxSerializer;
    use hyperast::test_utils::simple_tree::DisplayTree;

    #[test]
    fn test_bottom_up_matcher() {
        // Using an example where nodes are moved but maintain their structure
        let (stores, src, dst) = vpair_to_stores(simple_examples::example_move_action());

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

        let result = LazyBottomUpMatcher::match_it(mapping);

        // Verify mappings - we expect to have several mappings for this example
        // The root 'a', the 'b', 'c', 'd', 'e', and 'f' nodes should all be mapped
        assert!(
            result.mappings.len() >= 5,
            "Expected at least 5 mappings, got {}",
            result.mappings.len()
        );

        // Get the actual nodes from the arenas
        use crate::decompressed_tree_store::ShallowDecompressedTreeStore;
        let src_root = result.mapping.src_arena.root();
        let dst_root = result.mapping.dst_arena.root();

        // Root nodes should be mapped
        assert!(
            result
                .mapping
                .mappings
                .has(&src_root.shallow(), &dst_root.shallow()),
            "Root nodes should be mapped"
        );

        // Get children of root
        let src_children = result.mapping.src_arena.children(&src_root);
        let dst_children = result.mapping.dst_arena.children(&dst_root);

        // Both trees should have 2 children under root
        assert_eq!(src_children.len(), 2);
        assert_eq!(dst_children.len(), 2);

        // The 'e' and 'b' nodes should be mapped
        assert!(
            result
                .mapping
                .mappings
                .has(&src_children[0], &dst_children[0]),
            "The 'e' nodes should be mapped"
        );
        assert!(
            result
                .mapping
                .mappings
                .has(&src_children[1], &dst_children[1]),
            "The 'b' nodes should be mapped"
        );

        // Get children of 'b'
        let src_b_children = result.mapping.src_arena.children(&src_children[1]);
        let dst_b_children = result.mapping.dst_arena.children(&dst_children[1]);

        // Verify 'c' and 'd' are mapped correctly
        assert!(
            result
                .mapping
                .mappings
                .has(&src_b_children[0], &dst_b_children[0]),
            "The 'c' nodes should be mapped"
        );
        assert!(
            result
                .mapping
                .mappings
                .has(&src_b_children[1], &dst_b_children[2]),
            "The 'd' nodes should be mapped"
        );

        // Check for the moved 'f' node
        let src_e_children = result.mapping.src_arena.children(&src_children[0]);
        assert_eq!(
            src_e_children.len(),
            1,
            "Source 'e' node should have 1 child"
        );

        // The 'f' node should be mapped to the middle child of 'b' in the destination
        assert!(
            result
                .mapping
                .mappings
                .has(&src_e_children[0], &dst_b_children[1]),
            "The 'f' node should be mapped to the middle child of 'b'"
        );

        println!("Mappings: {:?}", result.mappings);
    }
}
