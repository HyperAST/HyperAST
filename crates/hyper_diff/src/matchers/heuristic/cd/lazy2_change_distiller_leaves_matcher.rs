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

    fn is_leaf<IdD>(_: IdD) -> bool {
        false // reach down to real leaves
    }

    pub fn execute(internal: &mut Mapper<HAST, Dsrc, Ddst, M>) {
        let hyperast = internal.hyperast;
        let mapping = &mut internal.mapping;
        let mappings = &mut mapping.mappings;
        let mut leaves_mappings = vec![];

        let mut src = mapping.src_arena.starter();
        // go to the left most "leaf" (i.e., a statement or real leaf)
        let mut src_to_traverse = vec![]; // easier like that
        loop {
            if Self::is_leaf(src) {
                break;
            }
            let mut cs = mapping.src_arena.decompress_children(&src);
            if cs.is_empty() {
                break;
            }
            cs.reverse();
            src = cs.pop().unwrap(); // cs is non empty
            src_to_traverse.extend(cs);
        }

        // for src in src_leaves
        loop {
            let mut dst = mapping.dst_arena.starter();
            // go to the left most "leaf" (i.e., a statement or real leaf)
            let mut dst_to_traverse = vec![]; // easier like that
            loop {
                if Self::is_leaf(&dst) {
                    break;
                }
                let mut cs = mapping.dst_arena.decompress_children(&dst);
                if cs.is_empty() {
                    break;
                }
                cs.reverse();
                dst = cs.pop().unwrap(); // cs is non empty
                dst_to_traverse.extend(cs);
            }

            // for &dst in &dst_leaves {
            loop {
                if !mappings.is_src(&src.shallow()) && !mappings.is_dst(&dst.shallow()) {
                    let osrc = mapping.src_arena.original(&src);
                    let tsrc = hyperast.resolve_type(&osrc);
                    let odst = mapping.dst_arena.original(&dst);
                    let tdst = hyperast.resolve_type(&odst);
                    if osrc == odst {
                        if mappings.link_if_both_unmapped(*src.shallow(), *dst.shallow()) {
                            let count = mapping.src_arena.descendants_count(&src);
                            let mut src = mapping.src_arena.descendants_range(&src).start;
                            let mut dst = mapping.dst_arena.descendants_range(&dst).start;
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
                let Some(sib) = dst_to_traverse.pop() else {
                    break;
                };
                dst = sib;
                loop {
                    let mut cs = mapping.dst_arena.decompress_children(&dst);
                    if cs.is_empty() {
                        break;
                    }
                    cs.reverse();
                    dst = cs.pop().unwrap(); // cs is non empty
                    dst_to_traverse.extend(cs);
                    if Self::is_leaf(&dst) {
                        break;
                    }
                }
            }
            let Some(sib) = src_to_traverse.pop() else {
                break;
            };
            src = sib;
            loop {
                if Self::is_leaf(src) {
                    break;
                }
                let mut cs = mapping.src_arena.decompress_children(&src);
                if cs.is_empty() {
                    break;
                }
                cs.reverse();
                src = cs.pop().unwrap(); // cs is non empty
                src_to_traverse.extend(cs);
            }
        }

        leaves_mappings.sort_unstable_by(|a, b| a.2.total_cmp(&b.2));

        for best_mapping in leaves_mappings {
            mappings.link(*best_mapping.0.shallow(), *best_mapping.1.shallow());
            // mappings.link_if_both_unmapped(*best_mapping.0.shallow(), *best_mapping.1.shallow());
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
