use crate::decompressed_tree_store::*;
use crate::matchers::Mapper;
use crate::matchers::mapping_store::{MonoMappingStore, MultiMappingStore};
use crate::matchers::optimal::zs::str_distance_patched::QGram;
use hyperast::types::{self, HyperAST, NodeId, NodeStore};
use hyperast::{PrimInt, types::LabelStore};
use num_traits::ToPrimitive;
use std::fmt::Debug;

pub struct ChangeDistillerLeavesMatcher<
    Dsrc,
    Ddst,
    HAST,
    M,
    const SIM_THRESHOLD_NUM: u64 = 1,
    const SIM_THRESHOLD_DEN: u64 = 2,
> {
    internal: Mapper<HAST, Dsrc, Ddst, M>,
}

impl<
    Dsrc: DecompressedTreeStore<HAST, M::Src> + DecompressedWithParent<HAST, M::Src>,
    Ddst: DecompressedTreeStore<HAST, M::Dst> + DecompressedWithParent<HAST, M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore,
    const SIM_THRESHOLD_NUM: u64,
    const SIM_THRESHOLD_DEN: u64, // DEFAULT_LABEL_SIM_THRESHOLD = 0.5
> ChangeDistillerLeavesMatcher<Dsrc, Ddst, HAST, M, SIM_THRESHOLD_NUM, SIM_THRESHOLD_DEN>
where
    M::Src: PrimInt,
    M::Dst: PrimInt,
    Dsrc: PostOrderIterable<HAST, M::Src> + PostOrder<HAST, M::Src>,
    Ddst: PostOrderIterable<HAST, M::Dst> + PostOrder<HAST, M::Dst>,
    HAST::Label: Eq,
    HAST::IdN: Debug,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    pub fn match_it<MM>(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
    where
        MM: MultiMappingStore<Src = M::Src, Dst = M::Dst> + Default,
    {
        let mut matcher = Self { internal: mapping };
        matcher.internal.mapping.mappings.topit(
            matcher.internal.mapping.src_arena.len(),
            matcher.internal.mapping.dst_arena.len(),
        );
        Self::execute::<MM>(&mut matcher.internal);
        matcher.internal
    }

    pub fn execute<MM>(internal: &mut Mapper<HAST, Dsrc, Ddst, M>)
    where
        MM: MultiMappingStore<Src = M::Src, Dst = M::Dst> + Default,
    {
        let mapping = &internal.mapping;
        let mappings = &mapping.mappings;
        let mut leaves_mappings = vec![];
        let dst_leaves: Vec<_> = Self::iter_leaves(&internal.dst_arena).collect();

        for src in Self::iter_leaves(&internal.src_arena) {
            for &dst in &dst_leaves {
                if !mappings.is_src(&src) && !mappings.is_dst(&dst) {
                    let tsrc = mapping.src_arena.original(&src);
                    let tsrc = internal.hyperast.resolve_type(&tsrc);
                    let tdst = mapping.dst_arena.original(&dst);
                    let tdst = internal.hyperast.resolve_type(&tdst);
                    if tsrc == tdst {
                        let sim = Self::sim(internal, src, dst);
                        dbg!(src, dst, sim);
                        if sim > SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64 {
                            leaves_mappings.push((src, dst));
                        }
                    }
                }
            }
        }

        let mut src_ignored = bitvec::bitbox![0;internal.src_arena.len()];
        let mut dst_ignored = bitvec::bitbox![0;internal.dst_arena.len()];
        dbg!(&leaves_mappings);
        leaves_mappings.sort_by(|a, b| {
            let a = Self::sim_cmp(internal, &(a.0, a.1));
            let b = Self::sim_cmp(internal, &(b.0, b.1));
            dbg!(a, b);
            b.cmp(&a)
        });
        while leaves_mappings.len() > 0 {
            let best_mapping = leaves_mappings.remove(0);
            let src_i = best_mapping.0.to_usize().unwrap();
            let dst_i = best_mapping.1.to_usize().unwrap();
            if !(src_ignored[src_i] || dst_ignored[dst_i]) {
                internal.mappings.link(best_mapping.0, best_mapping.1);
                src_ignored.set(src_i, true);
                dst_ignored.set(dst_i, true);
            }
        }
    }

    fn is_leaf<D, IdD>(_: &D, _: &IdD) -> bool {
        false // reach down to real leaves
    }

    pub fn iter_leaves<D, IdD: Eq>(arena: &D) -> impl Iterator<Item = IdD>
    where
        D: PostOrderIterable<HAST, IdD> + PostOrder<HAST, IdD>,
    {
        arena.iter_df_post::<true>().filter(|x|
            // is leaf. kind of an optimisation, it was easier like this anyway.
            arena.lld(x) == *x
        //
            || Self::is_leaf(arena, x))
    }

    fn sim(internal: &Mapper<HAST, Dsrc, Ddst, M>, src: M::Src, dst: M::Dst) -> f64 {
        let (src_l, dst_l) = Self::label_pair(internal, src, dst);
        use str_distance::DistanceMetric;
        let src_l = src_l.as_bytes().into_iter();
        let dst_l = dst_l.as_bytes().into_iter();
        1.0_f64 - QGram::new(3).normalized(src_l, dst_l)
    }

    fn sim_cmp(internal: &Mapper<HAST, Dsrc, Ddst, M>, (src, dst): &(M::Src, M::Dst)) -> usize {
        let (src_l, dst_l) = Self::label_pair(internal, *src, *dst);
        use str_distance::DistanceMetric;
        let src_l = src_l.as_bytes().into_iter();
        let dst_l = dst_l.as_bytes().into_iter();
        QGram::new(3).distance(src_l, dst_l)
    }

    fn label_pair(
        internal: &Mapper<HAST, Dsrc, Ddst, M>,
        src: M::Src,
        dst: M::Dst,
    ) -> (&str, &str) {
        use types::Labeled;
        let src = internal.mapping.src_arena.original(&src);
        let src = internal.hyperast.node_store().resolve(&src);
        let src_l = src.try_get_label().unwrap();
        let src_l = internal.hyperast.label_store().resolve(src_l);
        let dst = internal.mapping.dst_arena.original(&dst);
        let dst = internal.hyperast.node_store().resolve(&dst);
        let dst_l = dst.try_get_label().unwrap();
        let dst_l = internal.hyperast.label_store().resolve(&dst_l);
        (src_l, dst_l)
    }
}

#[cfg(test)]
mod tests {
    use hyperast::test_utils::simple_tree::{SimpleTree, vpair_to_stores};
    use types::{DecompressedFrom, HyperASTShared};

    use crate::{
        decompressed_tree_store::CompletePostOrder,
        matchers::{
            Decompressible,
            mapping_store::{MappingStore, MultiVecStore},
        },
    };

    use super::*;

    #[test]
    fn test_leaf_matcher() {
        use crate::tests::tree;
        let (stores, src, dst) = vpair_to_stores((
            tree!(0, "a"; [
                tree!(0, "b"),
                tree!(0, "c"),
            ]),
            tree!(0, "a"; [
                tree!(0, "c"),
                tree!(0, "b"),
            ]),
        ));

        #[allow(type_alias_bounds)]
        type DS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;

        let src_arena = <DS<_> as DecompressedFrom<_>>::decompress(&stores, &src);
        let dst_arena = <DS<_> as DecompressedFrom<_>>::decompress(&stores, &dst);
        let mapping = Mapper {
            hyperast: &stores,
            mapping: crate::matchers::Mapping {
                src_arena,
                dst_arena,
                mappings: crate::matchers::mapping_store::VecStore::default(),
            },
        };
        //  MappingStore mappings = new ChangeDistillerLeavesMatcher().match(src, dst);
        let mapping =
            ChangeDistillerLeavesMatcher::<_, _, _, _>::match_it::<MultiVecStore<_>>(mapping);
        // assertEquals(2, mappings.size());
        assert_eq!(2, mapping.mapping.mappings.len());
        use crate::decompressed_tree_store::ShallowDecompressedTreeStore;
        let src = mapping.mapping.src_arena.root();
        let src_cs = mapping.mapping.src_arena.children(&src);
        let dst = mapping.mapping.dst_arena.root();
        let dst_cs = mapping.mapping.dst_arena.children(&dst);
        dbg!(&mapping.mapping.mappings);
        dbg!(&src_cs);
        dbg!(&dst_cs);
        // assertTrue(mappings.has(src.getChild(0), dst.getChild(1)));
        assert!(mapping.mapping.mappings.has(&src_cs[0], &dst_cs[1]));
        // assertTrue(mappings.has(src.getChild(1), dst.getChild(0)));
        assert!(mapping.mapping.mappings.has(&src_cs[1], &dst_cs[0]));
    }
}
