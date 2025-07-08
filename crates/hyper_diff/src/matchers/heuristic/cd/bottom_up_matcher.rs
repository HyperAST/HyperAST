use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, PostOrder,
    PostOrderIterable,
};
use crate::matchers::Mapper;
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::similarity_metrics;
use hyperast::PrimInt;
use hyperast::store::nodes::compo;
use hyperast::types::{DecompressedFrom, HyperAST, NodeId, WithHashs, WithMetaData};
use num_traits::ToPrimitive;
use std::fmt::Debug;

use super::leaf_count;

pub struct BottomUpMatcher<
    Dsrc,
    Ddst,
    HAST,
    M: MonoMappingStore,
    const SIZE_THRESHOLD: usize = 4,
    const SIM_THRESHOLD_NUM: u64 = 6,
    const SIM_THRESHOLD_DEN: u64 = 10,
    const SIM_THRESHOLD2_NUM: u64 = 4,
    const SIM_THRESHOLD2_DEN: u64 = 10,
> {
    internal: Mapper<HAST, Dsrc, Ddst, M>,
}

impl<
    'a,
    Dsrc: DecompressedWithParent<HAST, M::Src>
        + PostOrder<HAST, M::Src>
        + PostOrderIterable<HAST, M::Src>
        + DecompressedFrom<HAST, Out = Dsrc>
        + ContiguousDescendants<HAST, M::Src>,
    Ddst: DecompressedWithParent<HAST, M::Dst>
        + PostOrder<HAST, M::Dst>
        + PostOrderIterable<HAST, M::Dst>
        + DecompressedFrom<HAST, Out = Ddst>
        + ContiguousDescendants<HAST, M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore + Default,
    const SIZE_THRESHOLD: usize,   // = 1000,
    const SIM_THRESHOLD_NUM: u64,  // = 6,
    const SIM_THRESHOLD_DEN: u64,  // = 10,
    const SIM_THRESHOLD2_NUM: u64, // = 4,
    const SIM_THRESHOLD2_DEN: u64, // = 10,
>
    BottomUpMatcher<
        Dsrc,
        Ddst,
        HAST,
        M,
        SIZE_THRESHOLD,
        SIM_THRESHOLD_NUM,
        SIM_THRESHOLD_DEN,
        SIM_THRESHOLD2_NUM,
        SIM_THRESHOLD2_DEN,
    >
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
    M::Src: PrimInt,
    M::Dst: PrimInt,
    HAST::Label: Eq,
    HAST::IdN: Debug,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    pub fn match_it(
        mut matcher: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
    where
        for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithMetaData<compo::StmtCount>,
        for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
            WithMetaData<compo::MemberImportCount>,
    {
        matcher.mapping.mappings.topit(
            matcher.mapping.src_arena.len(),
            matcher.mapping.dst_arena.len(),
        );
        let mut matcher = Self { internal: matcher };
        Self::execute(&mut matcher, leaf_count);
        matcher.internal
    }

    // simply goes over all the idd in post order, goes multiple time over dst
    pub fn execute0<'b>(&mut self, leaf_count: fn(HAST, HAST::IdN) -> usize) {
        let hyperast = self.internal.hyperast;
        let src_arena = &self.internal.mapping.src_arena;
        let dst_arena = &self.internal.mapping.dst_arena;
        for src in src_arena.iter_df_post::<true>() {
            if self.internal.mappings.is_src(&src) {
                continue;
            }
            let osrc = self.internal.mapping.src_arena.original(&src);
            let leaves = leaf_count(hyperast, osrc);

            for dst in dst_arena.iter_df_post::<false>() {
                if self.internal.mappings.is_dst(&dst) {
                    continue;
                }
                let mappings = &mut self.internal.mapping.mappings;
                let dst_arena = &self.internal.mapping.dst_arena;
                if Self::inner(hyperast, mappings, src_arena, dst_arena, src, dst, leaves) {
                    break;
                }
            }
        }
    }

    /// uses a hybrid traversal:
    ///     src is traversed fully through iter_df_post only skipping nodes already matched
    ///     dst is traversed using PostIter, thus nodes are still yielded like before,
    ///     but internally we traverse in pre-order to skip nodes already matched faster.
    /// execute0 and execute1 might work better at different scales, it needs further investigations.
    pub fn execute<'b>(&mut self, leaf_count: fn(HAST, HAST::IdN) -> usize) {
        let hyperast = self.internal.hyperast;
        let src_arena = &self.internal.mapping.src_arena;
        for src in src_arena.iter_df_post::<true>() {
            if self.internal.mappings.is_src(&src) {
                continue;
            }
            let osrc = self.internal.mapping.src_arena.original(&src);
            let leaves = leaf_count(hyperast, osrc);

            let mut dst_iter = PostIter::new(hyperast, &self.internal.mapping.dst_arena);
            while let Some(dst) = dst_iter.next_mappable(|dst|
                // we assume the whole subtree is already mapped
                self.internal.mapping.mappings.is_dst(&dst))
            {
                if self.internal.mappings.is_dst(&dst) {
                    continue;
                }
                let mappings = &mut self.internal.mapping.mappings;
                let dst_arena = &self.internal.mapping.dst_arena;
                if Self::inner(hyperast, mappings, src_arena, dst_arena, src, dst, leaves) {
                    break;
                }
            }
        }
    }

    // uses a more clever traversal to go over the minimum src and dst nodes
    pub fn execute1<'b>(&mut self, leaf_count: fn(HAST, HAST::IdN) -> usize) {
        let hyperast = self.internal.hyperast;
        let src_arena = &self.internal.mapping.src_arena;
        let mut src_iter = PostIter::new(hyperast, src_arena);
        while let Some(src) = src_iter.next_mappable(|src|
            // we assume the whole subtree is already mapped
            self.internal.mapping.mappings.is_src(&src))
        {
            let osrc = self.internal.mapping.src_arena.original(&src);
            let leaves = leaf_count(hyperast, osrc);

            let mut dst_iter = PostIter::new(hyperast, &self.internal.mapping.dst_arena);
            while let Some(dst) = dst_iter.next_mappable(|dst|
                // we assume the whole subtree is already mapped
                self.internal.mapping.mappings.is_dst(&dst))
            {
                let mappings = &mut self.internal.mapping.mappings;
                let dst_arena = &self.internal.mapping.dst_arena;
                if Self::inner(hyperast, mappings, src_arena, dst_arena, src, dst, leaves) {
                    break;
                }
            }
        }
    }

    fn inner(
        hyperast: HAST,
        mappings: &mut M,
        src_arena: &Dsrc,
        dst_arena: &Ddst,
        src: M::Src,
        dst: M::Dst,
        number_of_leaves: usize,
    ) -> bool
    where
        HAST: HyperAST + Copy,
    {
        let osrc = src_arena.original(&src);
        let tsrc = hyperast.resolve_type(&osrc);
        let odst = dst_arena.original(&dst);
        let tdst = hyperast.resolve_type(&odst);
        if tsrc == tdst {
            if !(src_arena.lld(&src) == src || dst_arena.lld(&dst) == dst) {
                let sim = similarity_metrics::SimilarityMeasure::range(
                    &src_arena.descendants_range(&src),
                    &dst_arena.descendants_range(&dst),
                    &*mappings,
                )
                .chawathe();
                let cond1 = number_of_leaves > SIZE_THRESHOLD
                    && sim >= SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64;
                let cond2 = number_of_leaves <= SIZE_THRESHOLD
                    && sim >= SIM_THRESHOLD2_NUM as f64 / SIM_THRESHOLD2_DEN as f64;
                if cond1 || cond2 {
                    mappings.link(src, dst);
                    return true;
                }
            }
        }
        false
    }
}

pub(super) struct PostIter<'a, HAST, D, IdD> {
    #[allow(unused)]
    stores: HAST,
    pub(super) arena: &'a D,
    to_traverse: Vec<IdD>,
    sibs: Vec<u16>,
    idd: IdD,
    down: bool,
}

impl<'a, HAST, D, IdD> PostIter<'a, HAST, D, IdD>
where
    HAST: HyperAST + Copy,
    D: DecompressedTreeStore<HAST, IdD>,
    IdD: Copy,
{
    pub fn new(stores: HAST, arena: &'a D) -> Self {
        Self {
            stores,
            idd: arena.root(),
            arena,
            to_traverse: Vec::new(),
            sibs: Vec::new(),
            down: true,
        }
    }
}

impl<HAST, D, IdD> Iterator for PostIter<'_, HAST, D, IdD>
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

impl<HAST, D, IdD> PostIter<'_, HAST, D, IdD>
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
                let mut cs = self.arena.children(&self.idd);
                cs.reverse();
                let Some(idd) = cs.pop() else {
                    self.down = false;
                    return Some(self.idd);
                };
                self.to_traverse.push(self.idd);
                self.sibs.push(cs.len().to_u16().unwrap());
                self.idd = idd;
                self.to_traverse.extend(cs);
            } else {
                let Some(sib) = self.to_traverse.pop() else {
                    return None;
                };
                let sibs = self.sibs.last_mut().unwrap();
                if sibs == &0 {
                    self.sibs.pop();
                    return Some(sib);
                }
                *sibs -= 1;
                self.down = true;
                self.idd = sib;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::leaves_matcher::LeavesMatcher;
    use crate::decompressed_tree_store::CompletePostOrder;
    use crate::matchers::Decompressible;
    use crate::matchers::mapping_store::MappingStore;
    use hyperast::test_utils::simple_tree::vpair_to_stores;
    use hyperast::types::{DecompressedFrom, HyperASTShared};

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
        let mapping = LeavesMatcher::<_, _, _, _>::match_stmt(mapping);
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

        let mapping = BottomUpMatcher::<_, _, _, _, 1>::match_it(mapping);
        dbg!(&mapping.mapping.mappings);
        assert!(mapping.mapping.mappings.has(&src, &dst));
    }
}
