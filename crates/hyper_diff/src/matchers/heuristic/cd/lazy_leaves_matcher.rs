#![allow(unexpected_cfgs)]
use super::{Similarity, TextSimilarity, is_leaf, is_leaf_file, is_leaf_stmt, is_leaf_sub_file};
use crate::decompressed_tree_store::{
    ContiguousDescendants, LazyDecompressed, LazyDecompressedTreeStore, Shallow,
};
use crate::matchers::Mapper;
use crate::matchers::mapping_store::MonoMappingStore;
use hyperast::PrimInt;
use hyperast::store::nodes::compo;
use hyperast::types::{HashKind, HyperAST, NodeId, NodeStore, WithHashs, WithMetaData};
use num_traits::one;
use std::fmt::Debug;

pub struct LazyLeavesMatcher<
    Dsrc,
    Ddst,
    HAST,
    M,
    S = TextSimilarity<HAST>,
    const SIM_THRESHOLD_NUM: u64 = 1,
    const SIM_THRESHOLD_DEN: u64 = 2,
> {
    internal: Mapper<HAST, Dsrc, Ddst, M>,
    _phantom: std::marker::PhantomData<S>,
}

impl<
    Dsrc: LazyDecompressed<M::Src>,
    Ddst: LazyDecompressed<M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore + Debug,
    S: Similarity<HAST = HAST, IdN = HAST::IdN>,
    const SIM_THRESHOLD_NUM: u64,
    const SIM_THRESHOLD_DEN: u64, // DEFAULT_LABEL_SIM_THRESHOLD = 0.5
> LazyLeavesMatcher<Dsrc, Ddst, HAST, M, S, SIM_THRESHOLD_NUM, SIM_THRESHOLD_DEN>
where
    M::Src: PrimInt,
    M::Dst: PrimInt,
    Dsrc: ContiguousDescendants<HAST, Dsrc::IdD, M::Src> + LazyDecompressedTreeStore<HAST, M::Src>,
    Ddst: ContiguousDescendants<HAST, Ddst::IdD, M::Dst> + LazyDecompressedTreeStore<HAST, M::Dst>,
    Ddst::IdD: Eq + Debug + Copy + PrimInt,
    Dsrc::IdD: Eq + Debug + Copy + PrimInt,
    HAST::Label: Eq,
    HAST::IdN: Debug,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
{
    pub fn match_it(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
    where
        for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
            WithMetaData<compo::StmtCount> + WithMetaData<compo::MemberImportCount>,
    {
        let mut matcher = Self {
            internal: mapping,
            _phantom: std::marker::PhantomData,
        };
        matcher.internal.mapping.mappings.topit(
            matcher.internal.mapping.src_arena.len(),
            matcher.internal.mapping.dst_arena.len(),
        );
        Self::execute(&mut matcher.internal, is_leaf_file, is_leaf_file);
        Self::execute(&mut matcher.internal, is_leaf_sub_file, is_leaf_sub_file);
        Self::execute(&mut matcher.internal, is_leaf_stmt, is_leaf_stmt);
        Self::execute(&mut matcher.internal, is_leaf, is_leaf);
        matcher.internal
    }
    pub fn match_stmt(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
    where
        for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithMetaData<compo::StmtCount>,
    {
        let mut matcher = Self {
            internal: mapping,
            _phantom: std::marker::PhantomData,
        };
        matcher.internal.mapping.mappings.topit(
            matcher.internal.mapping.src_arena.len(),
            matcher.internal.mapping.dst_arena.len(),
        );
        Self::execute(&mut matcher.internal, is_leaf_stmt, is_leaf_stmt);
        matcher.internal
    }
    pub fn match_all(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M> {
        let mut matcher = Self {
            internal: mapping,
            _phantom: std::marker::PhantomData,
        };
        matcher.internal.mapping.mappings.topit(
            matcher.internal.mapping.src_arena.len(),
            matcher.internal.mapping.dst_arena.len(),
        );

        Self::execute(&mut matcher.internal, is_leaf, is_leaf);
        matcher.internal
    }
    pub fn match_files(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M> {
        let mut matcher = Self {
            internal: mapping,
            _phantom: std::marker::PhantomData,
        };
        matcher.internal.mapping.mappings.topit(
            matcher.internal.mapping.src_arena.len(),
            matcher.internal.mapping.dst_arena.len(),
        );
        Self::execute(&mut matcher.internal, is_leaf_file, is_leaf_file);
        matcher.internal
    }
    pub fn match_sub_files(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
    where
        for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
            WithMetaData<compo::MemberImportCount>,
    {
        let mut matcher = Self {
            internal: mapping,
            _phantom: std::marker::PhantomData,
        };
        matcher.internal.mapping.mappings.topit(
            matcher.internal.mapping.src_arena.len(),
            matcher.internal.mapping.dst_arena.len(),
        );
        Self::execute(&mut matcher.internal, is_leaf_sub_file, is_leaf_sub_file);
        matcher.internal
    }

    pub fn execute(
        internal: &mut Mapper<HAST, Dsrc, Ddst, M>,
        is_leaf_src: fn(HAST, &Dsrc, Dsrc::IdD) -> bool,
        is_leaf_dst: fn(HAST, &Ddst, Ddst::IdD) -> bool,
    ) {
        let hyperast = internal.hyperast;
        let mapping = &mut internal.mapping;
        let mappings = &mut mapping.mappings;
        let mut leaves_mappings = vec![];

        let mut src_iter = LeafIter::new(hyperast, &mut mapping.src_arena, is_leaf_src);
        while let Some(src) = src_iter.next_mappable(|src| mappings.is_src(&src.shallow())) {
            let mut dst_iter = LeafIter::new(hyperast, &mut mapping.dst_arena, is_leaf_dst);
            while let Some(dst) = dst_iter.next_mappable(|dst| mappings.is_dst(&dst.shallow())) {
                if !mappings.is_src(&src.shallow()) && !mappings.is_dst(&dst.shallow()) {
                    let osrc = src_iter.arena.original(&src);
                    let tsrc = hyperast.resolve_type(&osrc);
                    let odst = dst_iter.arena.original(&dst);
                    let tdst = hyperast.resolve_type(&odst);
                    if osrc == odst {
                        // VALIDITY delaying and sorting would not change the result as sim would be 1.0
                        // NOTE it also avoids going multiple times over the same mapping
                        Self::link(mappings, &src_iter.arena, &dst_iter.arena, src, dst);
                    } else if tsrc == tdst
                        && !mappings.is_src(src.shallow())
                        && !mappings.is_dst(dst.shallow())
                    {
                        if WithHashs::hash(
                            &hyperast.node_store().resolve(&osrc),
                            &HashKind::structural(),
                        ) != WithHashs::hash(
                            &hyperast.node_store().resolve(&odst),
                            &HashKind::structural(),
                        ) {
                            continue; // cannot easily link descendants
                            // NOTE having the same number of descendants might be a sufficient condition
                            // but it might produce weird mappings if not cautious
                        }
                        let p = [src_iter.arena.original(&src), dst_iter.arena.original(&dst)];
                        let sim = S::norm(&hyperast, &p);
                        if sim > SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64 {
                            #[cfg(not(feature = "no_precomp_sim"))]
                            leaves_mappings.push((src, dst, sim));
                            #[cfg(feature = "no_precomp_sim")]
                            leaves_mappings.push((src, dst, ()));
                        }
                    }
                }
            }
        }
        #[cfg(not(feature = "no_precomp_sim"))]
        leaves_mappings.sort_by(|a, b| a.2.total_cmp(&b.2));
        #[cfg(feature = "no_precomp_sim")]
        leaves_mappings.sort_by(|a, b| {
            let a = S::dist(&hyperast, &Self::ori_pair(internal, a.0, a.1));
            let b = S::dist(&hyperast, &Self::ori_pair(internal, b.0, b.1));
            b.cmp(&a)
        });
        for (src, dst, _) in leaves_mappings {
            Self::link(
                &mut internal.mapping.mappings,
                &internal.mapping.src_arena,
                &internal.mapping.dst_arena,
                src,
                dst,
            );
        }
    }

    fn link(mappings: &mut M, src_arena: &Dsrc, dst_arena: &Ddst, src: Dsrc::IdD, dst: Ddst::IdD) {
        if mappings.link_if_both_unmapped(src.to_shallow(), dst.to_shallow()) {
            let count = src_arena.descendants_count(&src);
            assert_eq!(count, dst_arena.descendants_count(&dst));
            let mut src = src_arena.descendants_range(&src).start;
            let mut dst = dst_arena.descendants_range(&dst).start;
            for _ in 0..count {
                mappings.link_if_both_unmapped(src, dst);
                src += one();
                dst += one();
            }
        }
    }
}

struct LeafIter<'a, HAST, D, IdS, IdD> {
    stores: HAST,
    arena: &'a mut D,
    to_traverse: Vec<IdD>,
    idd: IdD,
    down: bool,
    is_leaf: fn(HAST, &D, IdD) -> bool,
    _phantom: std::marker::PhantomData<IdS>,
}

impl<'a, HAST, D, IdS> LeafIter<'a, HAST, D, IdS, D::IdD>
where
    HAST: HyperAST + Copy,
    D: LazyDecompressedTreeStore<HAST, IdS>,
    D::IdD: Copy,
{
    fn new(stores: HAST, arena: &'a mut D, is_leaf: fn(HAST, &D, D::IdD) -> bool) -> Self {
        Self {
            stores,
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
        self.next_mappable(|_| false)
    }
}

impl<HAST, D, IdS> LeafIter<'_, HAST, D, IdS, D::IdD>
where
    HAST: HyperAST + Copy,
    D: LazyDecompressedTreeStore<HAST, IdS>,
    D::IdD: Copy,
{
    fn next_mappable(&mut self, skip: impl Fn(D::IdD) -> bool) -> Option<D::IdD> {
        loop {
            if self.down {
                if skip(self.idd) {
                    self.down = false;
                    continue;
                }
                if (self.is_leaf)(self.stores, &self.arena, self.idd) {
                    self.down = false;
                    return Some(self.idd);
                }
                let mut cs = self.arena.decompress_children(&self.idd);
                cs.reverse();
                let Some(idd) = cs.pop() else {
                    self.down = false;
                    // return Some(self.idd);
                    continue; // only stops on specified leafs
                };
                self.idd = idd;
                self.to_traverse.extend(cs);
            } else {
                let Some(sib) = self.to_traverse.pop() else {
                    return None;
                };
                self.idd = sib;
                self.down = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompressed_tree_store::lazy_post_order::LazyPostOrder;
    use crate::matchers::{Decompressible, mapping_store::MappingStore};
    use crate::tests::examples::example_change_distiller;
    use hyperast::test_utils::simple_tree::vpair_to_stores;
    use hyperast::types::{DecompressedFrom, HyperASTShared};

    #[allow(type_alias_bounds)]
    type DS<HAST: HyperASTShared> = Decompressible<HAST, LazyPostOrder<HAST::IdN, u32>>;

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
        let mapping = LazyLeavesMatcher::<_, _, _, _>::match_all(mapping);
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

    #[test]
    fn test_leaf_iterator() {
        let (stores, src, dst) = vpair_to_stores(example_change_distiller());
        println!(
            "{:?}",
            hyperast::test_utils::simple_tree::DisplayTree::new(
                &stores.label_store,
                &stores.node_store,
                src
            )
        );
        let mut src_arena = <DS<_> as DecompressedFrom<_>>::decompress(&stores, &src);
        let mut dst_arena = <DS<_> as DecompressedFrom<_>>::decompress(&stores, &dst);
        let mut src_arena = src_arena.as_mut();
        let mut dst_arena = dst_arena.as_mut();

        let src_iter = LeafIter::new(&stores, &mut src_arena, is_leaf);
        assert_eq!(4, src_iter.count());

        let dst_iter = LeafIter::new(&stores, &mut dst_arena, is_leaf);
        assert_eq!(4, dst_iter.count());
    }

    #[test]
    fn test_leaf_stmt_iterator() {
        let (stores, src, dst) = vpair_to_stores(example_change_distiller());
        println!(
            "{:?}",
            hyperast::test_utils::simple_tree::DisplayTree::new(
                &stores.label_store,
                &stores.node_store,
                src
            )
        );
        let mut src_arena = <DS<_> as DecompressedFrom<_>>::decompress(&stores, &src);
        let mut dst_arena = <DS<_> as DecompressedFrom<_>>::decompress(&stores, &dst);
        let mut src_arena = src_arena.as_mut();
        let mut dst_arena = dst_arena.as_mut();

        let src_iter = LeafIter::new(&stores, &mut src_arena, is_leaf_stmt);
        assert_eq!(2, src_iter.count());

        let dst_iter = LeafIter::new(&stores, &mut dst_arena, is_leaf_stmt);
        assert_eq!(2, dst_iter.count());
    }

    #[test]
    fn test_leaf_stmt_matcher() {
        let (stores, src, dst) = vpair_to_stores(example_change_distiller());
        println!(
            "{:?}",
            hyperast::test_utils::simple_tree::DisplayTree::new(
                &stores.label_store,
                &stores.node_store,
                src
            )
        );
        println!(
            "{:?}",
            hyperast::test_utils::simple_tree::DisplayTree::new(
                &stores.label_store,
                &stores.node_store,
                dst
            )
        );

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
        let mapping = LazyLeavesMatcher::<_, _, _, _>::match_stmt(mapping);
        let mapping = LazyLeavesMatcher::<_, _, _, _>::match_all(mapping);
        assert_eq!(5, mapping.mapping.mappings.len());
        use crate::decompressed_tree_store::ShallowDecompressedTreeStore;
        let src = mapping.mapping.src_arena.root();
        let src_cs = mapping.mapping.src_arena.children(&src);
        let dst = mapping.mapping.dst_arena.root();
        let dst_cs = mapping.mapping.dst_arena.children(&dst);
        dbg!(&mapping.mapping.mappings);
        dbg!(&src_cs);
        dbg!(&dst_cs);
        assert!(mapping.mapping.mappings.has(&0, &5));
        assert!(mapping.mapping.mappings.has(&3, &2));
        assert!(mapping.mapping.mappings.has(&4, &3));
    }
}
