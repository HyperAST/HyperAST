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
        .dice()
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         decompressed_tree_store::CompletePostOrder,
//         matchers::{Decompressible, mapping_store::DefaultMappingStore},
//         tests::examples::example_simple,
//         tree::simple_tree::vpair_to_stores,
//     };

//     fn init() {
//         let _ = env_logger::builder()
//             .is_test(true)
//             .filter_level(log::LevelFilter::Debug)
//             .try_init();
//     }

//     #[test]
//     fn test_single_node_match() {
//         init();
//         let (stores, src, dst) = vpair_to_stores(example_simple());

//         let mapping = Mapper {
//             hyperast: &stores,
//             mapping: crate::matchers::Mapping {
//                 src_arena: Decompressible::<_, CompletePostOrder<_, u16>>::decompress(
//                     &stores, &src,
//                 ),
//                 dst_arena: Decompressible::<_, CompletePostOrder<_, u16>>::decompress(
//                     &stores, &dst,
//                 ),
//                 mappings: DefaultMappingStore::default(),
//             },
//         };

//         let result = LazyBottomUpMatcher::<_, _, _, _>::match_it(mapping);

//         let mapped_root = result
//             .mapping
//             .mappings
//             .get_dst(&result.mapping.src_arena.root());
//         assert!(mapped_root.is_some());
//         assert_eq!(mapped_root.unwrap(), result.mapping.dst_arena.root());
//     }
// }
