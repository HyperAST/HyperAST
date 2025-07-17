#![allow(unexpected_cfgs)]
use super::{Similarity, TextSimilarity, is_leaf, is_leaf_file, is_leaf_stmt, is_leaf_sub_file};
use crate::decompressed_tree_store::{
    DecompressedTreeStore, PostOrder, PostOrderIterable, Shallow, ShallowDecompressedTreeStore,
};
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::{Mapper, Mapping};
use hyperast::store::nodes::compo;
use hyperast::types::{HyperAST, NodeId, NodeStore as _, WithMetaData};
use hyperast::{PrimInt, types};
use std::fmt::Debug;

pub struct LeavesMatcher<
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
    Dsrc: DecompressedTreeStore<HAST, M::Src>,
    Ddst: DecompressedTreeStore<HAST, M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore,
    S: Similarity<HAST = HAST, IdN = HAST::IdN>,
    const SIM_THRESHOLD_NUM: u64,
    const SIM_THRESHOLD_DEN: u64, // DEFAULT_LABEL_SIM_THRESHOLD = 0.5
> LeavesMatcher<Dsrc, Ddst, HAST, M, S, SIM_THRESHOLD_NUM, SIM_THRESHOLD_DEN>
where
    M::Src: PrimInt,
    M::Dst: PrimInt,
    Dsrc: PostOrderIterable<HAST, M::Src> + PostOrder<HAST, M::Src>,
    Ddst: PostOrderIterable<HAST, M::Dst> + PostOrder<HAST, M::Dst>,
    HAST::Label: Eq + Clone,
    HAST::IdN: Debug + Copy,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    pub fn match_it(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
    where
        for<'t> <HAST as types::AstLending<'t>>::RT: WithMetaData<compo::StmtCount>,
        for<'t> <HAST as types::AstLending<'t>>::RT: WithMetaData<compo::MemberImportCount>,
        for<'t> <HAST as types::AstLending<'t>>::RT: hyperast::types::WithHashs,
        M::Src: Shallow<M::Src>,
        M::Dst: Shallow<M::Dst>,
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

    pub fn match_files(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
    where
        for<'t> <HAST as types::AstLending<'t>>::RT: types::WithHashs,
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
        matcher.internal
    }

    pub fn match_stmt(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
    where
        for<'t> <HAST as types::AstLending<'t>>::RT: WithMetaData<compo::StmtCount>,
        for<'t> <HAST as types::AstLending<'t>>::RT: types::WithHashs,
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
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
    where
        M::Src: Shallow<M::Src>,
        M::Dst: Shallow<M::Dst>,
        for<'t> <HAST as types::AstLending<'t>>::RT: types::WithHashs,
    {
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

    pub fn execute(
        internal: &mut Mapper<HAST, Dsrc, Ddst, M>,
        is_leaf_src: fn(HAST, &Dsrc, M::Src) -> bool,
        is_leaf_dst: fn(HAST, &Ddst, M::Dst) -> bool,
    ) where
        for<'t> <HAST as types::AstLending<'t>>::RT: types::WithHashs,
    {
        let hyperast = internal.hyperast;
        let mut leaves_mappings = vec![];

        let mut src_iter = LeafIter::new(hyperast, &internal.mapping.src_arena, is_leaf_src);
        while let Some(src) = src_iter.next_mappable(|src| internal.mapping.mappings.is_src(&src)) {
            let mut dst_iter = LeafIter::new(hyperast, &internal.mapping.dst_arena, is_leaf_dst);
            while let Some(dst) =
                dst_iter.next_mappable(|dst| internal.mapping.mappings.is_dst(&dst))
            {
                let mappings = &mut internal.mapping.mappings;
                let osrc = src_iter.arena.original(&src);
                let odst = dst_iter.arena.original(&dst);
                if osrc == odst {
                    Self::link(mappings, src_iter.arena, dst_iter.arena, src, dst);
                } else if !mappings.is_src(&src) && !mappings.is_dst(&dst) {
                    let tsrc = hyperast.resolve_type(&osrc);
                    let tdst = hyperast.resolve_type(&odst);
                    if tsrc == tdst {
                        let p = Self::ori_pair(&internal.mapping, src, dst);

                        if types::WithHashs::hash(
                            &hyperast.node_store().resolve(&p[0]),
                            &types::HashKind::structural(),
                        ) != types::WithHashs::hash(
                            &hyperast.node_store().resolve(&p[1]),
                            &types::HashKind::structural(),
                        ) {
                            continue; // cannot easily link descendants
                            // NOTE having the same number of descendants might be a sufficient condition
                            // but it might produce weird mappings if not cautious
                        }
                        let sim = S::norm(&hyperast, &p);
                        // dbg!(&p, sim);
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

    fn link(mappings: &mut M, src_arena: &Dsrc, dst_arena: &Ddst, src: M::Src, dst: M::Dst) {
        if mappings.link_if_both_unmapped(src, dst) {
            let src = src_arena.descendants(&src);
            let dst = dst_arena.descendants(&dst);
            assert_eq!(src.len(), dst.len());
            for (src, dst) in src.iter().zip(dst.iter()) {
                mappings.link(*src, *dst)
            }
        }
    }

    fn ori_pair(mapping: &Mapping<Dsrc, Ddst, M>, src: M::Src, dst: M::Dst) -> [HAST::IdN; 2] {
        let src = mapping.src_arena.original(&src);
        let dst = mapping.dst_arena.original(&dst);
        [src, dst]
    }
}

pub(super) struct LeafIter<'a, HAST, D, IdD> {
    stores: HAST,
    pub(super) arena: &'a D,
    to_traverse: Vec<IdD>,
    idd: IdD,
    down: bool,
    is_leaf: fn(HAST, &D, IdD) -> bool,
}

impl<'a, HAST, D, IdD> LeafIter<'a, HAST, D, IdD>
where
    HAST: HyperAST + Copy,
    D: ShallowDecompressedTreeStore<HAST, IdD>,
    IdD: Copy,
{
    pub fn new(stores: HAST, arena: &'a D, is_leaf: fn(HAST, &D, IdD) -> bool) -> Self {
        Self {
            stores,
            idd: arena.root(),
            arena,
            to_traverse: Vec::new(),
            down: true,
            is_leaf,
        }
    }
}

impl<HAST, D, IdD> Iterator for LeafIter<'_, HAST, D, IdD>
where
    HAST: HyperAST + Copy,
    D: DecompressedTreeStore<HAST, IdD>,
    IdD: Copy,
{
    type Item = IdD;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_mappable(|_| false)
    }
}

impl<HAST, D, IdD> LeafIter<'_, HAST, D, IdD>
where
    HAST: HyperAST + Copy,
    D: DecompressedTreeStore<HAST, IdD>,
    IdD: Copy,
{
    pub fn next_mappable(&mut self, skip: impl Fn(IdD) -> bool) -> Option<IdD> {
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
                let mut cs = self.arena.children(&self.idd);
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
                self.down = true;
                self.idd = sib;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::LabelSimilarity;
    use super::*;
    use crate::decompressed_tree_store::CompletePostOrder;
    use crate::matchers::Decompressible;
    use crate::matchers::mapping_store::MappingStore;
    use crate::tests::examples::example_change_distiller;
    use hyperast::test_utils::simple_tree::vpair_to_stores;
    use hyperast::types::{DecompressedFrom, HyperASTShared};

    #[allow(type_alias_bounds)]
    type DS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;

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
                mappings: crate::matchers::mapping_store::VecStore::default(),
                src_arena,
                dst_arena,
            },
        };
        //  MappingStore mappings = new ChangeDistillerLeavesMatcher().match(src, dst);
        let mapping = LeavesMatcher::<_, _, _, _>::match_all(mapping);
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
        // NOTE cannot use TextSimilarity here because `SimpleTree` does not have a textual source code representation
        let mapping = LeavesMatcher::<_, _, _, _, LabelSimilarity<_>>::match_stmt(mapping);
        let mapping = LeavesMatcher::<_, _, _, _, LabelSimilarity<_>>::match_all(mapping);
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
