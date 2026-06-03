//! This decompressed tree store is I thnk a pretty good utils to improve perfs of the bottom-up matcher.
//! But it will require some more love, as I initially did not finish to implement everithing.

use super::{
    ContiguousDescendants, DecendantsLending, IterKr, LazyDecompressed, LazyDecompressedTreeStore,
    LazyPOBorrowSlice, LazyPOSliceLending, PostOrdKeyRoots, PostOrderIterable, PostOrderKeyRoots,
    Shallow, lazy_post_order::LazyPostOrder,
};
use crate::{
    decompressed_tree_store::{
        DecompressedParentsLending, DecompressedTreeStore, DecompressedWithParent, PostOrder,
        ShallowDecompressedTreeStore,
    },
    matchers::{
        Decompressible,
        mapping_store::{MappingStore, MonoMappingStore},
    },
};
use bitvec::slice::BitSlice;
use hyperast::PrimInt;
use hyperast::types::{self, AstLending, HyperAST, WithChildren, WithStats};
use num_traits::{ToPrimitive, Zero, cast, zero};
use std::{
    borrow::{Borrow, BorrowMut},
    collections::BTreeMap,
    fmt::Debug,
    marker::PhantomData,
    ops::Index,
};

/// Wrap or just map a decommpressed tree in breadth-first eg. post-order,
pub struct SimpleHiddingMapper<
    'a,
    IdD,
    DTS,
    M: Borrow<Vec<IdD>>,
    R: Borrow<BTreeMap<IdD, IdD>>,
    D: BorrowMut<DTS> = DTS,
> {
    map: M,
    rev: R,
    pub back: D,
    phantom: PhantomData<&'a (DTS, IdD)>,
}

// TODO deref to back
impl<
    'a,
    IdD: Debug,
    DTS: Debug, // + DecompressedTreeStore<HAST, IdD>,
    M: BorrowMut<Vec<IdD>>,
    R: Borrow<BTreeMap<IdD, IdD>>,
    D: BorrowMut<DTS>,
> Debug for SimpleHiddingMapper<'a, IdD, DTS, M, R, D>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SD")
            .field("map", self.map.borrow())
            .field("back", &self.back.borrow())
            .field("phantom", &self.phantom)
            .finish()
    }
}

// impl<'a, T: 'a + WithChildren, IdD: PrimInt, D: BorrowMut<LazyPostOrder<HAST::IdN, IdD>>>
//     SimpleHiddingMapper<'a, IdD, LazyPostOrder<HAST::IdN, IdD>, D>
// where
//     T::TreeId: Clone + std::fmt::Debug,
// {
//     pub fn from_subtree_mapping<S, M: Index<usize>>(store: &'a S, back: D, side: &M) -> Self
//     where
//         S: NodeStore<T::TreeId, R<'a> = T>,
//         M::Output: PrimInt,
//     {
//         let map = hiding_map(&back, side);
//         Self {
//             map,
//             // fc,
//             back,
//             phantom: PhantomData,
//         }
//     }
// }

pub fn hiding_map<IdN, IdD: PrimInt, D: BorrowMut<LazyPostOrder<IdN, IdD>>, M: Index<usize>>(
    back: &D,
    side: &M,
) -> (Vec<IdD>, BTreeMap<IdD, IdD>)
where
    M::Output: PrimInt,
{
    let x: &LazyPostOrder<IdN, IdD> = back.borrow();
    let mut map = Vec::with_capacity(x._len());
    let mut i: IdD = cast(x._root()).unwrap();
    // map.push(i);
    // i = i - num_traits::one();

    loop {
        map.push(i);
        if !side[i.to_usize().unwrap()].is_zero() {
            i = cast(x._lld(i.to_usize().unwrap())).unwrap();
        }
        if i == num_traits::zero() {
            break;
        }
        i = i - num_traits::one();
    }

    map.shrink_to_fit();

    let rev = map
        .iter()
        .enumerate()
        .map(|(i, x)| (*x, cast(map.len() - i - 1).unwrap()))
        .collect();
    (map, rev)
}

pub struct MonoMappingWrap<'map, 'back, Src, Dst, M> {
    pub map_src: &'map Vec<Src>,
    pub rev_src: &'map BTreeMap<Src, Src>,
    pub map_dst: &'map Vec<Dst>,
    pub rev_dst: &'map BTreeMap<Dst, Dst>,
    back: &'back mut M,
}

impl<'map, 'back, M: MappingStore> MappingStore for MonoMappingWrap<'map, 'back, M::Src, M::Dst, M>
where
    <M as MappingStore>::Src: PrimInt,
    <M as MappingStore>::Dst: PrimInt,
{
    type Src = M::Src;
    type Dst = M::Dst;

    fn len(&self) -> usize {
        todo!()
        // self.src_to_dst.iter().filter(|x| **x != zero()).count()
    }

    fn capacity(&self) -> (usize, usize) {
        (self.map_src.len(), self.map_dst.len())
    }

    fn link(&mut self, src: Self::Src, dst: Self::Dst) {
        self.back.link(
            self.map_src[self.map_src.len() - 1 - src.to_usize().unwrap()],
            self.map_dst[self.map_dst.len() - 1 - dst.to_usize().unwrap()],
        )
    }

    fn cut(&mut self, src: Self::Src, dst: Self::Dst) {
        self.back.cut(
            self.map_src[self.map_src.len() - 1 - src.to_usize().unwrap()],
            self.map_dst[self.map_dst.len() - 1 - dst.to_usize().unwrap()],
        )
    }

    fn is_src(&self, src: &Self::Src) -> bool {
        self.back
            .is_src(&self.map_src[self.map_src.len() - 1 - src.to_usize().unwrap()])
    }

    fn is_dst(&self, dst: &Self::Dst) -> bool {
        self.back
            .is_dst(&self.map_dst[self.map_dst.len() - 1 - dst.to_usize().unwrap()])
    }

    fn topit(&mut self, left: usize, right: usize) {
        self.back.topit(left, right)
    }

    fn has(&self, src: &Self::Src, dst: &Self::Dst) -> bool {
        self.back.has(
            &self.map_src[self.map_src.len() - 1 - src.to_usize().unwrap()],
            &self.map_dst[self.map_dst.len() - 1 - dst.to_usize().unwrap()],
        )
    }
}

impl<'map, 'back, M: MonoMappingStore> MonoMappingStore
    for MonoMappingWrap<'map, 'back, M::Src, M::Dst, M>
where
    <M as MappingStore>::Src: PrimInt,
    <M as MappingStore>::Dst: PrimInt,
{
    fn get_src_unchecked(&self, dst: &Self::Dst) -> Self::Src {
        let src = self
            .back
            .get_src_unchecked(&self.map_dst[self.map_dst.len() - 1 - dst.to_usize().unwrap()]);
        *self.rev_src.get(&src).unwrap()
    }

    fn get_dst_unchecked(&self, src: &Self::Src) -> Self::Dst {
        let dst = self
            .back
            .get_dst_unchecked(&self.map_src[self.map_src.len() - 1 - src.to_usize().unwrap()]);
        *self.rev_dst.get(&dst).unwrap()
    }

    fn get_src(&self, dst: &Self::Dst) -> Option<Self::Src> {
        let src = self
            .back
            .get_src(&self.map_dst[self.map_dst.len() - 1 - dst.to_usize().unwrap()])?;
        self.rev_src.get(&src).copied()
    }

    fn get_dst(&self, src: &Self::Src) -> Option<Self::Dst> {
        let dst = self
            .back
            .get_dst(&self.map_src[self.map_src.len() - 1 - src.to_usize().unwrap()])?;
        self.rev_dst.get(&dst).copied()
    }

    fn link_if_both_unmapped(&mut self, src: Self::Src, dst: Self::Dst) -> bool {
        self.back.link_if_both_unmapped(
            self.map_src[self.map_src.len() - 1 - src.to_usize().unwrap()],
            self.map_dst[self.map_dst.len() - 1 - dst.to_usize().unwrap()],
        )
    }

    type Iter<'a>
        = MonoIter<'a, Self::Src, Self::Dst>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        todo!()
        // MonoIter {
        //     v: self.map_src.iter().enumerate(),
        //     // .filter(|x|*x.1 != zero()),
        //     // .map(|(src, dst)| (cast::<_, T>(src).unwrap(), *dst - one())),
        //     _phantom: std::marker::PhantomData,
        // }
    }

    fn number_of_common_descendants_ranges(
        &self,
        src: &std::ops::Range<Self::Src>,
        dst: &std::ops::Range<Self::Dst>,
    ) -> u32
    where
        Self::Src: PrimInt,
        Self::Dst: PrimInt,
        Self: Sized,
    {
        crate::matchers::similarity_metrics::number_of_common_descendants_ranges(
            src, dst, self.back,
        )

        // (src.start.to_usize().unwrap()..src.end.to_usize().unwrap())
        //         .into_iter()
        //         .filter(|t| src.is_src(&cast(*t).unwrap()))
        //         .filter(|t| dst.contains(&src.get_dst_unchecked(&cast(*t).unwrap())))
        //         .count()
        //         .try_into()
        //         .unwrap()
    }
}

pub struct MonoIter<'a, T: 'a + PrimInt, U: 'a> {
    v: std::iter::Enumerate<core::slice::Iter<'a, U>>,
    _phantom: std::marker::PhantomData<*const T>,
}

impl<'a, T: PrimInt, U: PrimInt> Iterator for MonoIter<'a, T, U> {
    type Item = (T, U);

    fn next(&mut self) -> Option<Self::Item> {
        let mut a = self.v.next();
        loop {
            let (i, x) = a?;
            if x.to_usize().unwrap() != 0 {
                return Some((cast::<_, T>(i).unwrap(), *x - num_traits::one()));
            } else {
                a = self.v.next();
            }
        }
    }
}

pub fn hide<
    'a,
    'map,
    'back,
    _Src,
    _Dst,
    Src: BorrowMut<_Src>,
    Dst: BorrowMut<_Dst>,
    M: MonoMappingStore,
>(
    src: Src,
    map_src: &'map Vec<M::Src>,
    rev_src: &'map BTreeMap<M::Src, M::Src>,
    dst: Dst,
    map_dst: &'map Vec<M::Dst>,
    rev_dst: &'map BTreeMap<M::Dst, M::Dst>,
    mappings: &'back mut M,
) -> (
    SimpleHiddingMapper<'a, M::Src, _Src, &'map Vec<M::Src>, &'map BTreeMap<M::Src, M::Src>, Src>,
    SimpleHiddingMapper<'a, M::Dst, _Dst, &'map Vec<M::Dst>, &'map BTreeMap<M::Dst, M::Dst>, Dst>,
    MonoMappingWrap<'map, 'back, M::Src, M::Dst, M>,
)
where
    M::Src: PrimInt,
    M::Dst: PrimInt,
{
    (
        SimpleHiddingMapper {
            rev: rev_src,
            map: map_src,
            back: src,
            phantom: PhantomData,
        },
        SimpleHiddingMapper {
            rev: rev_dst,
            map: map_dst,
            back: dst,
            phantom: PhantomData,
        },
        MonoMappingWrap {
            map_src,
            rev_src,
            map_dst,
            rev_dst,
            back: mappings,
        },
    )
}

// impl<
//         'd,
//         'a,
//         'b,
//         HAST: HyperAST + Copy,
//         IdD: PrimInt + Debug,
//         DTS, //: DecompressedTreeStore<HAST, IdD> + DecompressedWithParent<HAST, IdD> + PostOrder<HAST, IdD>,
//         M: Borrow<Vec<IdD>>,
//         R: Borrow<BTreeMap<IdD, IdD>>,
//         D: BorrowMut<DTS>,
//     > types::NLending<'b, T::TreeId> for SimpleHiddingMapper<'d, IdD, DTS, M, R, D>
// where
//     // T: for<'t> types::NLending<'t, T::TreeId>,
//     // DTS: for<'t> types::NLending<'t, T::TreeId>,
//     // for<'t> <DTS as types::NLending<'t, T::TreeId>>::N: hyperast::types::WithChildren,
// {
//     type N = <T as types::NLending<'b, T::TreeId>>::N;
// }

impl<
    'a,
    HAST: HyperAST + Copy,
    IdD: PrimInt + Debug,
    DTS: ShallowDecompressedTreeStore<HAST, IdD>, //: DecompressedTreeStore<HAST, IdD>,
    M: Borrow<Vec<IdD>>,
    R: Borrow<BTreeMap<IdD, IdD>>,
    D: BorrowMut<DTS>,
> ShallowDecompressedTreeStore<HAST, IdD> for SimpleHiddingMapper<'a, IdD, DTS, M, R, D>
{
    fn len(&self) -> usize {
        self.map.borrow().len()
    }

    fn original(&self, id: &IdD) -> HAST::IdN {
        self.back
            .borrow()
            .original(&self.map.borrow()[self.len() - id.to_usize().unwrap() - 1])
    }

    fn root(&self) -> IdD {
        num_traits::cast(self.len() - 1).unwrap()
    }

    fn child(&self, x: &IdD, p: &[impl PrimInt]) -> IdD {
        let b: &DTS = self.back.borrow();
        let c = b.child(
            &self.map.borrow()[self.len() - x.to_usize().unwrap() - 1],
            p,
        );
        *self.rev.borrow().get(&c).unwrap()
    }

    fn children(&self, x: &IdD) -> Vec<IdD> {
        let b: &DTS = self.back.borrow();
        let cs = b.children(&self.map.borrow()[self.len() - x.to_usize().unwrap() - 1]);
        cs.into_iter()
            .map(|x| *self.rev.borrow().get(&x).unwrap())
            .collect()
    }
}

impl<
    'a,
    HAST: HyperAST + Copy, // + WithStats,
    IdD: PrimInt + Debug,  // + Shallow<IdD> + Debug,
    DTS: DecompressedTreeStore<HAST, IdD> + DecompressedWithParent<HAST, IdD> + PostOrder<HAST, IdD>, //: DecompressedTreeStore<HAST, IdD, IdD> + PostOrder<HAST, IdD>, // + LazyDecompressedTreeStore<HAST, IdD>,
    M: Borrow<Vec<IdD>>,
    R: Borrow<BTreeMap<IdD, IdD>>,
    D: BorrowMut<DTS>,
> DecompressedTreeStore<HAST, IdD> for SimpleHiddingMapper<'a, IdD, DTS, M, R, D>
{
    fn descendants(&self, x: &IdD) -> Vec<IdD> {
        let cs = self
            .back
            .borrow()
            .descendants(&self.map.borrow()[self.len() - x.to_usize().unwrap() - 1]);
        cs.into_iter()
            .filter_map(|x| self.rev.borrow().get(&x).copied())
            .collect()
    }

    fn descendants_count(&self, x: &IdD) -> usize {
        self.descendants(x).len()
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        let conv = self.map.borrow()[self.map.borrow().len() - 1 - i.to_usize().unwrap()]; //self.back.borrow_mut().lld(aaa);
        let lld = self.back.borrow().lld(&conv);
        let mut y = *i;
        loop {
            if y.is_zero() {
                break;
            }
            // dbg!(y, lld);
            let Some(conv) = self
                .map
                .borrow()
                .get(self.map.borrow().len() - 1 - y.to_usize().unwrap())
            else {
                break;
            };
            // dbg!(conv, y);
            if lld < *conv {
                y = y - num_traits::one();
            } else {
                break;
            }
        }
        y
    }

    fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
        self.back.borrow().is_descendant(
            &self.map.borrow()[self.len() - desc.to_usize().unwrap() - 1],
            &self.map.borrow()[self.len() - of.to_usize().unwrap() - 1],
        )
    }
}

impl<
    'a,
    'd,
    // HAST: HyperAST + Copy,
    IdD: PrimInt,
    DTS: DecompressedParentsLending<'a, IdD>, //: DecompressedTreeStore<HAST, IdD> + DecompressedWithParent<HAST, IdD>,
    M: Borrow<Vec<IdD>>,
    R: Borrow<BTreeMap<IdD, IdD>>,
    D: BorrowMut<DTS>,
> DecompressedParentsLending<'a, IdD> for SimpleHiddingMapper<'d, IdD, DTS, M, R, D>
{
    type PIt = <DTS as DecompressedParentsLending<'a, IdD>>::PIt;
}

impl<
    'd,
    HAST: HyperAST + Copy,
    IdD: PrimInt + Debug,
    DTS: DecompressedTreeStore<HAST, IdD> + DecompressedWithParent<HAST, IdD>,
    M: Borrow<Vec<IdD>>,
    R: Borrow<BTreeMap<IdD, IdD>>,
    D: BorrowMut<DTS>,
> DecompressedWithParent<HAST, IdD> for SimpleHiddingMapper<'d, IdD, DTS, M, R, D>
{
    fn has_parent(&self, id: &IdD) -> bool {
        self.back
            .borrow()
            .has_parent(&self.map.borrow()[self.len() - id.to_usize().unwrap() - 1])
    }

    fn parent(&self, id: &IdD) -> Option<IdD> {
        let p = self
            .back
            .borrow()
            .parent(&self.map.borrow()[self.len() - id.to_usize().unwrap() - 1])?;
        self.rev.borrow().get(&p).copied()
    }

    fn position_in_parent<Idx: PrimInt>(&self, c: &IdD) -> Option<Idx> {
        self.back
            .borrow()
            .position_in_parent(&self.map.borrow()[self.len() - c.to_usize().unwrap() - 1])
    }

    fn parents(&self, _id: IdD) -> <Self as DecompressedParentsLending<'_, IdD>>::PIt {
        // self.back.borrow().parents(id)
        todo!()
    }

    fn path<Idx: PrimInt>(&self, _parent: &IdD, _descendant: &IdD) -> Vec<Idx> {
        // self.back.borrow().path(parent, descendant)
        todo!()
    }

    fn lca(&self, _a: &IdD, _b: &IdD) -> IdD {
        // self.back.borrow().lca(a, b)
        todo!()
    }
}
impl<
    'a,
    HAST: HyperAST + Copy,
    IdD: PrimInt + Debug,
    DTS, //DecompressedTreeStore<HAST, IdD> + DecompressedWithParent<HAST, IdD>,
    M: Borrow<Vec<IdD>>,
    R: Borrow<BTreeMap<IdD, IdD>>,
    D: BorrowMut<DTS>,
> super::DecompressedSubtree<HAST::IdN>
    for Decompressible<HAST, SimpleHiddingMapper<'a, IdD, DTS, M, R, D>>
where
    for<'t> <HAST as AstLending<'t>>::RT: WithStats,
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type Out = Decompressible<HAST, LazyPostOrder<HAST::IdN, IdD>>;

    fn decompress(self, _id: &HAST::IdN) -> Self::Out {
        todo!()
    }
}

impl<
    'a,
    HAST: HyperAST + Copy,
    IdD: PrimInt + Debug,
    DTS: DecompressedTreeStore<HAST, IdD> + DecompressedWithParent<HAST, IdD> + PostOrder<HAST, IdD>,
    M: Borrow<Vec<IdD>>,
    R: Borrow<BTreeMap<IdD, IdD>>,
    D: BorrowMut<DTS>,
> PostOrder<HAST, IdD> for SimpleHiddingMapper<'a, IdD, DTS, M, R, D>
where
    HAST::IdN: Debug,
{
    fn lld(&self, i: &IdD) -> IdD {
        // todo!()
        // // TODO not sure
        let c = self
            .back
            .borrow()
            .lld(&self.map.borrow()[self.len() - i.to_usize().unwrap() - 1]);
        *self.rev.borrow().get(&c).unwrap()
    }

    fn tree(&self, id: &IdD) -> HAST::IdN {
        self.back
            .borrow()
            .tree(&self.map.borrow()[self.len() - id.to_usize().unwrap() - 1])
    }
}
impl<
    'd,
    HAST: HyperAST + Copy,
    IdD: PrimInt + Debug,
    DTS: DecompressedTreeStore<HAST, IdD> + DecompressedWithParent<HAST, IdD> + PostOrder<HAST, IdD>,
    M: Borrow<Vec<IdD>>,
    R: Borrow<BTreeMap<IdD, IdD>>,
    D: BorrowMut<DTS>,
> PostOrderIterable<HAST, IdD> for SimpleHiddingMapper<'d, IdD, DTS, M, R, D>
{
    type It = super::Iter<IdD>;
    fn iter_df_post<const ROOT: bool>(&self) -> Self::It {
        let len = if ROOT {
            cast(self.len()).unwrap()
        } else {
            self.root()
        };
        super::Iter {
            current: zero(),
            len,
        }
    }
}
// pub struct Iter<IdD> {
//     map: M,
//     phantom: PhantomData<IdD>,
// }

// impl<IdD: PrimInt, M: Borrow<Vec<IdD>>> Iterator for Iter<IdD, M> {
//     type Item = IdD;

//     fn next(&mut self) -> Option<Self::Item> {

//     }
// }

impl<
    'd,
    'a,
    // HAST: HyperAST + Copy,
    IdD: PrimInt + Debug,
    DTS: for<'t> DecendantsLending<'t>, // : DecompressedTreeStore<HAST, IdD> + DecompressedWithParent<HAST, IdD> + PostOrder<HAST, IdD>,
    M: Borrow<Vec<IdD>>,
    R: Borrow<BTreeMap<IdD, IdD>>,
    D: BorrowMut<DTS>,
> DecendantsLending<'a> for SimpleHiddingMapper<'d, IdD, DTS, M, R, D>
{
    type Slice = <DTS as DecendantsLending<'a>>::Slice;
}

impl<
    'd,
    HAST: HyperAST + Copy,
    IdD: PrimInt + Debug,
    DTS: for<'t> DecendantsLending<'t>
        + DecompressedTreeStore<HAST, IdD>
        + DecompressedWithParent<HAST, IdD>
        + PostOrder<HAST, IdD>,
    M: Borrow<Vec<IdD>>,
    R: Borrow<BTreeMap<IdD, IdD>>,
    D: BorrowMut<DTS>,
> ContiguousDescendants<HAST, IdD> for SimpleHiddingMapper<'d, IdD, DTS, M, R, D>
{
    fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdD> {
        let conv = self.map.borrow()[self.map.borrow().len() - 1 - x.to_usize().unwrap()]; //self.back.borrow_mut().lld(aaa);
        let lld = self.back.borrow().lld(&conv);
        return lld..conv;
        // let mut y = *x;
        // // dbg!(self.map.borrow());
        // loop {
        //     if y.is_zero() {
        //         break;
        //     }
        //     // dbg!(y, lld);
        //     let Some(conv) = self
        //         .map
        //         .borrow()
        //         .get(self.map.borrow().len() - 1 - y.to_usize().unwrap())
        //     else {
        //         break;
        //     };
        //     if lld < *conv {
        //         y = y - num_traits::one();
        //     } else {
        //         break;
        //     }
        // }
        // y..*x
    }

    // type Slice<'b>
    //     = SimplePOSlice<'b, T, IdD>
    // where
    //     Self: 'b;

    fn slice(&self, _x: &IdD) -> <Self as DecendantsLending<'_>>::Slice {
        todo!()
    }
}

// impl<
//         'd,
//         T: WithChildren,
//         IdD: PrimInt,
//         DTS: DecompressedTreeStore<'d, T, IdD> + DecompressedWithParent<'d, T, IdD>,
//         M: Borrow<Vec<IdD>>,
// R: Borrow<BTreeMap<IdD,IdD>>,
//         D: BorrowMut<DTS>,
//     > DecompressedWithParent<'d, T, IdD> for SimpleHiddingMapper<'d, IdD, DTS, M, D>
// where
//     T::TreeId: Debug,
// {
//     fn has_parent(&self, id: &IdD) -> bool {
//         <LazyPostOrder<HAST::IdN, IdD>>::has_parent(&self, id)
//     }

//     fn parent(&self, id: &IdD) -> Option<IdD> {
//         <LazyPostOrder<HAST::IdN, IdD>>::parent(&self, id)
//     }

//     type PIt<'a> = IterParents<'a, IdD> where IdD: 'a, T::TreeId:'a, T: 'a, Self: 'a;

//     fn parents(&self, id: IdD) -> <Self as DecompressedParentsLending<'_, IdD>>::PIt {
//         <LazyPostOrder<HAST::IdN, IdD>>::parents(&self, id)
//     }

// fn position_in_parent<Idx: PrimInt>(&self, c: &IdD) -> Option<Idx> {
//         <LazyPostOrder<HAST::IdN, IdD>>::position_in_parent(&self, c)
//     }

// fn path<Idx: PrimInt>(&self, parent: &IdD, descendant: &IdD) -> Vec<Idx> {
//         <LazyPostOrder<HAST::IdN, IdD>>::path(&self, parent, descendant)
//     }

//     fn lca(&self, a: &IdD, b: &IdD) -> IdD {
//         <LazyPostOrder<HAST::IdN, IdD>>::lca(&self, a, b)
//     }
// }

pub struct IterParents<'a, IdD> {
    id: IdD,
    id_parent: &'a [IdD],
}

impl<'a, IdD: PrimInt> Iterator for IterParents<'a, IdD> {
    type Item = IdD;

    fn next(&mut self) -> Option<Self::Item> {
        if self.id == cast(self.id_parent.len() - 1).unwrap() {
            return None;
        }
        let r = self.id_parent[self.id.to_usize().unwrap()];
        self.id = r.clone();
        Some(r)
    }
}

// impl<'a, T: WithChildren, IdD: PrimInt> ShallowDecompressedTreeStore<HAST, IdD>
//     for SimpleHiddingMapper<'d, IdD, DTS, M, D>
// where
//     T::TreeId: Debug,
// {
//     fn len(&self) -> usize {
//         <LazyPostOrder<HAST::IdN, IdD>>::len(&self)
//     }

//     fn original(&self, id: &IdD) -> <T>::TreeId {
//         <LazyPostOrder<HAST::IdN, IdD>>::original(&self, id)
//     }

//     fn root(&self) -> IdD {
//         <LazyPostOrder<HAST::IdN, IdD>>::root(&self)
//     }

// fn child<'b, S>(&self, store: &S, x: &IdD, p: &[<T as WithChildren>::ChildIdx]) -> IdD
//     where
//         //'a: 'b,
//         S: 'b + NodeStore<T::TreeId, R<'b> = T>,
//     {
//         <LazyPostOrder<HAST::IdN, IdD>>::child(&self, store, x, p)
//     }

//     fn children<'b, S>(&self, store: &S, x: &IdD) -> Vec<IdD>
//     where
//         // 'a: 'b,
//         S: NodeStore<T::TreeId, R<'b> = T>,
//     {
//         <LazyPostOrder<HAST::IdN, IdD>>::children(&self, store, x)
//     }
// }

impl<
    'd,
    IdS: PrimInt + Shallow<IdS> + Debug,
    IdD: Shallow<IdS>,
    DTS,
    M: Borrow<Vec<IdD>>,
    R: Borrow<BTreeMap<IdD, IdD>>,
    D: BorrowMut<DTS>,
> LazyDecompressed<IdS> for SimpleHiddingMapper<'d, IdD, DTS, M, R, D>
{
    type IdD = IdD;
}

impl<
    'd,
    HAST: HyperAST + Copy,
    IdS: PrimInt + Shallow<IdS> + Debug,
    IdD: PrimInt + Shallow<IdS> + Debug,
    DTS: LazyDecompressed<IdS, IdD = IdD>
        + LazyDecompressedTreeStore<HAST, IdS>
        + DecompressedTreeStore<HAST, IdD>
        + DecompressedWithParent<HAST, IdD>
        + PostOrder<HAST, IdD>,
    M: Borrow<Vec<IdD>>,
    R: Borrow<BTreeMap<IdD, IdD>>,
    D: BorrowMut<DTS>,
> LazyDecompressedTreeStore<HAST, IdS> for SimpleHiddingMapper<'d, IdD, DTS, M, R, D>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    Self: DecompressedTreeStore<HAST, IdD, IdS>,
{
    fn starter(&self) -> Self::IdD {
        num_traits::cast(self.len() - 1).unwrap()
    }

    fn decompress_children(&mut self, x: &Self::IdD) -> Vec<Self::IdD> {
        let len = self.len();
        let b: &mut DTS = self.back.borrow_mut();
        let cs = b.decompress_children(&self.map.borrow()[len - x.to_usize().unwrap() - 1]);
        cs.into_iter()
            .map(|x| *self.rev.borrow().get(&x).unwrap())
            .collect()
    }

    fn decompress_to(&mut self, x: &IdS) -> Self::IdD {
        let len = self.len();
        let b: &mut DTS = self.back.borrow_mut();
        let c = b.decompress_to(self.map.borrow()[len - x.to_usize().unwrap() - 1].shallow());
        *self.rev.borrow().get(&c).unwrap()
    }
}

impl<
    'a,
    HAST: HyperAST + Copy,
    IdD: PrimInt + Shallow<IdD> + Debug,
    M: Borrow<Vec<IdD>>,
    R: Borrow<BTreeMap<IdD, IdD>>,
    D: BorrowMut<Decompressible<HAST, &'a mut LazyPostOrder<HAST::IdN, IdD>>>,
> SimpleHiddingMapper<'a, IdD, Decompressible<HAST, &'a mut LazyPostOrder<HAST::IdN, IdD>>, M, R, D>
where
    for<'t> <HAST as types::AstLending<'t>>::RT: WithChildren + WithStats,
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn decompress_visible_descendants(&mut self, x: &IdD) {
        let mut q: Vec<IdD> =
            vec![self.map.borrow()[self.map.borrow().len() - 1 - x.to_usize().unwrap()]];
        while let Some(x) = q.pop() {
            if !self.rev.borrow().contains_key(&x) {
                continue;
            }
            if self.back.borrow().descendants_count(&x) == 0 {
                continue;
            }
            if !self.rev.borrow().contains_key(&(x - num_traits::one())) {
                continue;
            }
            assert!(self.back.borrow().id_parent[x.to_usize().unwrap()] != zero());
            q.extend(self.back.borrow_mut().decompress_children(&x));
        }
    }
}

impl<
    'a,
    'd,
    HAST: HyperAST + Copy,
    IdD: PrimInt,
    M: Borrow<Vec<IdD>>,
    R: Borrow<BTreeMap<IdD, IdD>>,
    D: BorrowMut<Decompressible<HAST, &'d mut LazyPostOrder<HAST::IdN, IdD>>>,
> LazyPOSliceLending<'a, HAST, IdD>
    for SimpleHiddingMapper<
        'd,
        IdD,
        Decompressible<HAST, &'d mut LazyPostOrder<HAST::IdN, IdD>>,
        M,
        R,
        D,
    >
where
    for<'t> <HAST as types::AstLending<'t>>::RT: WithChildren + WithStats,
{
    type SlicePo = CompleteWHPO<'a, HAST::IdN, IdD, bitvec::boxed::BitBox>;
}

impl<
    'd,
    HAST: HyperAST + Copy,
    IdD: PrimInt,
    M: Borrow<Vec<IdD>>,
    R: Borrow<BTreeMap<IdD, IdD>>,
    D: BorrowMut<Decompressible<HAST, &'d mut LazyPostOrder<HAST::IdN, IdD>>>,
> LazyPOBorrowSlice<HAST, IdD>
    for SimpleHiddingMapper<
        'd,
        IdD,
        Decompressible<HAST, &'d mut LazyPostOrder<HAST::IdN, IdD>>,
        M,
        R,
        D,
    >
where
    IdD: Shallow<IdD> + Debug,
    for<'t> <HAST as types::AstLending<'t>>::RT: WithChildren + WithStats,
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn slice_po(&mut self, x: &IdD) -> <Self as LazyPOSliceLending<'_, HAST, IdD>>::SlicePo {
        self.decompress_visible_descendants(x);
        let map_lld = self.first_descendant(x);
        let len = x.to_usize().unwrap() - map_lld.to_usize().unwrap() + 1;
        // - id_compressed ez
        let mut id_compressed: Vec<HAST::IdN> = Vec::with_capacity(len);
        // - id_parent: direct resolve ?
        let mut id_parent: Vec<IdD> = Vec::with_capacity(len);
        // - kr: adapt the algo
        let mut kr = bitvec::bitbox!(0;len);
        // - llds: should be easy when extracting
        let mut llds: Vec<IdD> = vec![*x; len];
        let mut curr = map_lld.clone();
        while curr <= *x {
            let conv = self.map.borrow()[self.map.borrow().len() - 1 - curr.to_usize().unwrap()];
            // dbg!(conv);
            let parent = self
                .back
                .borrow_mut()
                .parent(&conv)
                .unwrap_or(self.back.borrow_mut().root());
            let parent = self.rev.borrow().get(&parent).unwrap();
            id_parent.push(*parent);
            if llds[id_compressed.len()].to_usize().unwrap() >= id_compressed.len() {
                llds[id_compressed.len()] = cast(id_compressed.len()).unwrap();
            }
            if llds.get(parent.to_usize().unwrap()) >= Some(parent) {
                llds[parent.to_usize().unwrap()] = llds[id_compressed.len()];
            }
            id_compressed.push(self.back.borrow_mut().original(&conv));
            curr = curr + num_traits::one();
        }
        let mut visited = bitvec::bitbox!(0; len);
        for i in (1..len).rev() {
            if !visited[llds[i].to_usize().unwrap()] {
                kr.set(i, true);
                // kr.push(cast(i + 1).unwrap());
                visited.set(llds[i].to_usize().unwrap(), true);
            }
        }
        CompleteWHPO {
            map: self.map.borrow(),
            id_compressed,
            llds,
            id_parent,
            kr,
        }
    }
}

pub struct CompleteWHPO<'a, IdN, IdD, Kr: Borrow<BitSlice>> {
    #[allow(unused)]
    // TODO continue implementing traits, but after so long I would need test to avoid writting garbage.
    pub(crate) map: &'a [IdD],
    pub(crate) id_compressed: Vec<IdN>,
    pub(crate) llds: Vec<IdD>,
    #[allow(unused)]
    // TODO continue implementing traits, but after so long I would need test to avoid writting garbage.
    pub(crate) id_parent: Vec<IdD>,
    pub(super) kr: Kr,
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt, Kr: Borrow<BitSlice>>
    ShallowDecompressedTreeStore<HAST, IdD> for CompleteWHPO<'a, HAST::IdN, IdD, Kr>
{
    fn len(&self) -> usize {
        self.id_compressed.len()
    }

    fn original(&self, id: &IdD) -> HAST::IdN {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }

    fn root(&self) -> IdD {
        cast(self.id_compressed.len() - 1).unwrap()
    }

    fn child(&self, _x: &IdD, _p: &[impl PrimInt]) -> IdD {
        todo!()
        // self.simple.child(store, x, p)
    }

    fn children(&self, _x: &IdD) -> Vec<IdD> {
        todo!()
        // self.simple.children(store, x)
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt, Kr: Borrow<BitSlice>> DecompressedTreeStore<HAST, IdD>
    for CompleteWHPO<'a, HAST::IdN, IdD, Kr>
{
    fn descendants(&self, _x: &IdD) -> Vec<IdD> {
        todo!()
        // self.simple.descendants(store, x)
    }

    fn first_descendant(&self, _i: &IdD) -> IdD {
        todo!()
        // self.simple.first_descendant(i)
    }

    fn descendants_count(&self, _x: &IdD) -> usize {
        todo!()
        // self.simple.descendants_count(store, x)
    }

    fn is_descendant(&self, _desc: &IdD, _of: &IdD) -> bool {
        todo!()
        // self.simple.is_descendant(desc, of)
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt, Kr: Borrow<BitSlice>> PostOrder<HAST, IdD>
    for CompleteWHPO<'a, HAST::IdN, IdD, Kr>
{
    fn lld(&self, i: &IdD) -> IdD {
        self.llds[i.to_usize().unwrap()]
    }

    fn tree(&self, id: &IdD) -> HAST::IdN {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }
}

impl<'a, 'b, HAST: HyperAST + Copy, IdD: PrimInt, Kr: Borrow<BitSlice>>
    PostOrdKeyRoots<'b, HAST, IdD> for CompleteWHPO<'a, HAST::IdN, IdD, Kr>
{
    type Iter = IterKr<'b, IdD>;
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt, Kr: Borrow<BitSlice>> PostOrderKeyRoots<HAST, IdD>
    for CompleteWHPO<'a, HAST::IdN, IdD, Kr>
{
    fn iter_kr(&self) -> <Self as PostOrdKeyRoots<'_, HAST, IdD>>::Iter {
        IterKr(self.kr.borrow().iter_ones(), PhantomData)
    }
}
