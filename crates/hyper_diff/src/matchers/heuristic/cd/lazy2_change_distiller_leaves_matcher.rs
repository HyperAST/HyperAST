use crate::decompressed_tree_store::*;
use crate::matchers::Mapper;
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::optimal::zs::str_distance_patched::QGram;
use hyperast::types::{self, HyperAST, NodeId, NodeStore};
use hyperast::{PrimInt, types::LabelStore};
use num_traits::one;
use std::fmt::Debug;

pub struct LazyChangeDistillerLeavesMatcher<
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
    Dsrc: LazyDecompressed<M::Src>,
    Ddst: LazyDecompressed<M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore + Debug,
    const SIM_THRESHOLD_NUM: u64,
    const SIM_THRESHOLD_DEN: u64, // DEFAULT_LABEL_SIM_THRESHOLD = 0.5
> LazyChangeDistillerLeavesMatcher<Dsrc, Ddst, HAST, M, SIM_THRESHOLD_NUM, SIM_THRESHOLD_DEN>
where
    M::Src: PrimInt,
    M::Dst: PrimInt,
    Dsrc: DecompressedWithParent<HAST, Dsrc::IdD>
        + ContiguousDescendants<HAST, Dsrc::IdD, M::Src>
        + LazyDecompressedTreeStore<HAST, M::Src>,
    Ddst: DecompressedWithParent<HAST, Ddst::IdD>
        + ContiguousDescendants<HAST, Ddst::IdD, M::Dst>
        + LazyDecompressedTreeStore<HAST, M::Dst>,
    Ddst::IdD: Eq + Debug + Copy + PrimInt,
    Dsrc::IdD: Eq + Debug + Copy + PrimInt,
    HAST::Label: Eq,
    HAST::IdN: Debug,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    pub fn match_it(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M> {
        let mut matcher = Self { internal: mapping };
        matcher.internal.mapping.mappings.topit(
            matcher.internal.mapping.src_arena.len(),
            matcher.internal.mapping.dst_arena.len(),
        );
        Self::execute(&mut matcher.internal);
        matcher.internal
    }

    fn is_leaf<D, IdD>(_: &D, _: IdD) -> bool {
        false // reach down to real leaves
    }

    pub fn execute(internal: &mut Mapper<HAST, Dsrc, Ddst, M>) {
        let hyperast = internal.hyperast;
        let mapping = &mut internal.mapping;
        let mappings = &mut mapping.mappings;
        let mut leaves_mappings = vec![];

        let mut src_iter = LeafIter::new(&mut mapping.src_arena, Self::is_leaf);
        while let Some(src) = src_iter.next() {
            let mut dst_iter = LeafIter::new(&mut mapping.dst_arena, Self::is_leaf);
            while let Some(dst) = dst_iter.next() {
                if !mappings.is_src(&src.shallow()) && !mappings.is_dst(&dst.shallow()) {
                    let osrc = src_iter.arena.original(&src);
                    let tsrc = hyperast.resolve_type(&osrc);
                    let odst = dst_iter.arena.original(&dst);
                    let tdst = hyperast.resolve_type(&odst);
                    if osrc == odst {
                        if mappings.link_if_both_unmapped(*src.shallow(), *dst.shallow()) {
                            let count = src_iter.arena.descendants_count(&src);
                            let mut src = src_iter.arena.descendants_range(&src).start;
                            let mut dst = dst_iter.arena.descendants_range(&dst).start;
                            for _ in 0..count {
                                mappings.link_if_both_unmapped(src, dst);
                                src += one();
                                dst += one();
                            }
                        }
                    } else if tsrc == tdst {
                        // Self::sim(internal, src, dst);
                        let Some(src_l) = Self::lab(&hyperast, osrc) else {
                            break;
                        };
                        let Some(dst_l) = Self::lab(&hyperast, odst) else {
                            break;
                        };
                        dbg!(src_l, dst_l);
                        use str_distance::DistanceMetric;
                        let src_l = src_l.as_bytes().into_iter();
                        let dst_l = dst_l.as_bytes().into_iter();
                        let sim = 1.0 - QGram::new(3).normalized(src_l, dst_l);
                        dbg!(src, dst, sim);

                        if sim > SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64 {
                            leaves_mappings.push((src, dst, sim));
                        }
                    }
                }
            }
        }
        leaves_mappings.sort_by(|a, b| a.2.total_cmp(&b.2));

        for best_mapping in leaves_mappings {
            mappings.link_if_both_unmapped(*best_mapping.0.shallow(), *best_mapping.1.shallow());
        }
    }

    fn lab(hyperast: &HAST, i: HAST::IdN) -> Option<&str> {
        use types::Labeled;
        let n = hyperast.node_store().resolve(&i);
        let l = n.try_get_label()?;
        let l = hyperast.label_store().resolve(l);
        Some(l)
    }
}

struct LeafIter<'a, HAST, D, IdS, IdD> {
    arena: &'a mut D,
    to_traverse: Vec<IdD>,
    idd: IdD,
    down: bool,
    is_leaf: fn(&D, IdD) -> bool,
    _phantom: std::marker::PhantomData<(HAST, IdS)>,
}
impl<'a, HAST, D, IdS> LeafIter<'a, HAST, D, IdS, D::IdD>
where
    HAST: HyperAST + Copy,
    D: LazyDecompressedTreeStore<HAST, IdS>,
    D::IdD: Copy,
{
    fn new(arena: &'a mut D, is_leaf: fn(&D, D::IdD) -> bool) -> Self {
        Self {
            idd: arena.starter(),
            arena,
            to_traverse: Vec::new(),
            down: true,
            is_leaf,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<HAST, D, IdS> Iterator for LeafIter<'_, HAST, D, IdS, D::IdD>
where
    HAST: HyperAST + Copy,
    D: LazyDecompressedTreeStore<HAST, IdS>,
    D::IdD: Copy,
{
    type Item = D::IdD;

    fn next(&mut self) -> Option<Self::Item> {
        if self.down {
            loop {
                if (self.is_leaf)(&self.arena, self.idd) {
                    break;
                }
                let mut cs = self.arena.decompress_children(&self.idd);
                if cs.is_empty() {
                    break;
                }
                cs.reverse();
                self.idd = cs.pop().unwrap(); // cs is non empty
                self.to_traverse.extend(cs);
            }
            self.down = false;
            Some(self.idd)
        } else {
            let Some(sib) = self.to_traverse.pop() else {
                return None;
            };
            self.idd = sib;
            Some(self.idd)
        }
    }
}

#[cfg(test)]
mod tests {
    use hyperast::test_utils::simple_tree::{SimpleTree, vpair_to_stores};
    use lazy_post_order::LazyPostOrder;
    use types::{DecompressedFrom, HyperASTShared};

    use crate::matchers::{Decompressible, mapping_store::MappingStore};

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
        type DS<HAST: HyperASTShared> = Decompressible<HAST, LazyPostOrder<HAST::IdN, u32>>;

        let mut src_arena = <DS<_> as DecompressedFrom<_>>::decompress(&stores, &src);
        let mut dst_arena = <DS<_> as DecompressedFrom<_>>::decompress(&stores, &dst);
        let mapping = Mapper {
            hyperast: &stores,
            mapping: crate::matchers::Mapping {
                src_arena: src_arena.as_mut(),
                dst_arena: dst_arena.as_mut(),
                mappings: crate::matchers::mapping_store::VecStore::default(),
            },
        };
        let mapping = LazyChangeDistillerLeavesMatcher::<_, _, _, _>::match_it(mapping);
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
