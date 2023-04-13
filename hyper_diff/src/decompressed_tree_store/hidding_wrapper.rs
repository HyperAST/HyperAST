use std::{
    borrow::{Borrow, BorrowMut},
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, Index},
    slice,
};

use bitvec::slice::BitSlice;
use num_traits::{cast, zero, PrimInt, ToPrimitive, Zero};

use crate::{
    decompressed_tree_store::{
        BreadthFirstIterable, DecompressedTreeStore, DecompressedWithParent, PostOrder,
        ShallowDecompressedTreeStore,
    },
    matchers::mapping_store::{MappingStore, MonoMappingStore},
};
use hyper_ast::types::{self, NodeStore, Stored, WithChildren, WithStats};

use super::{
    basic_post_order::BasicPOSlice,
    lazy_post_order::{self, LazyPostOrder},
    simple_post_order::SimplePOSlice,
    ContiguousDescendants, IterKr, LazyDecompressedTreeStore, LazyPOBorrowSlice, PostOrderIterable,
    PostOrderKeyRoots, Shallow,
};

/// Wrap or just map a decommpressed tree in breadth-first eg. post-order,
pub struct SimpleHiddingMapper<
    'a,
    T: WithChildren,
    IdD,
    DTS,
    M: Borrow<Vec<IdD>>,
    R: Borrow<BTreeMap<IdD, IdD>>,
    D: BorrowMut<DTS> = DTS,
> {
    map: M,
    rev: R,
    pub back: D,
    phantom: PhantomData<&'a (T, DTS, IdD)>,
}

// TODO deref to back
impl<
        'a,
        T: WithChildren,
        IdD: Debug,
        DTS: DecompressedTreeStore<'a, T, IdD> + Debug,
        M: BorrowMut<Vec<IdD>>,
        R: Borrow<BTreeMap<IdD, IdD>>,
        D: BorrowMut<DTS>,
    > Debug for SimpleHiddingMapper<'a, T, IdD, DTS, M, R, D>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SD")
            .field("map", self.map.borrow())
            .field("back", &self.back.borrow())
            .field("phantom", &self.phantom)
            .finish()
    }
}

// impl<'a, T: 'a + WithChildren, IdD: PrimInt, D: BorrowMut<LazyPostOrder<T, IdD>>>
//     SimpleHiddingMapper<'a, T, IdD, LazyPostOrder<T, IdD>, D>
// where
//     <T as types::Stored>::TreeId: Clone + std::fmt::Debug,
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

pub fn hiding_map<
    'a,
    T: 'a + WithChildren,
    IdD: PrimInt,
    D: BorrowMut<LazyPostOrder<T, IdD>>,
    M: Index<usize>,
>(
    back: &D,
    side: &M,
) -> (Vec<IdD>, BTreeMap<IdD, IdD>)
where
    M::Output: PrimInt,
    <T as types::Stored>::TreeId: Clone + std::fmt::Debug,
{
    let x: &LazyPostOrder<T, IdD> = back.borrow();
    let mut map = Vec::with_capacity(x.len());
    let mut i = x.root();
    // map.push(i);
    // i = i - num_traits::one();

    loop {
        map.push(i);
        if !side[i.to_usize().unwrap()].is_zero() {
            i = x.lld(&i);
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
    <M as MappingStore>::Src: num_traits::PrimInt,
    <M as MappingStore>::Dst: num_traits::PrimInt,
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
    <M as MappingStore>::Src: num_traits::PrimInt,
    <M as MappingStore>::Dst: num_traits::PrimInt,
{
    fn get_src_unchecked(&self, dst: &Self::Dst) -> Self::Src {
        let src = self
            .back
            .get_src_unchecked(&self.map_dst[self.map_dst.len() - 1 - dst.to_usize().unwrap()]);
        *self.rev_src.borrow().get(&src).unwrap()
    }

    fn get_dst_unchecked(&self, src: &Self::Src) -> Self::Dst {
        let dst = self
            .back
            .get_dst_unchecked(&self.map_src[self.map_src.len() - 1 - src.to_usize().unwrap()]);
        *self.rev_dst.borrow().get(&dst).unwrap()
    }

    fn get_src(&self, dst: &Self::Dst) -> Option<Self::Src> {
        let src = self
            .back
            .get_src(&self.map_dst[self.map_dst.len() - 1 - dst.to_usize().unwrap()])?;
        self.rev_src.borrow().get(&src).copied()
    }

    fn get_dst(&self, src: &Self::Src) -> Option<Self::Dst> {
        let dst = self
            .back
            .get_dst(&self.map_src[self.map_src.len() - 1 - src.to_usize().unwrap()])?;
        self.rev_dst.borrow().get(&dst).copied()
    }

    fn link_if_both_unmapped(&mut self, src: Self::Src, dst: Self::Dst) -> bool {
        self.back.link_if_both_unmapped(
            self.map_src[self.map_src.len() - 1 - src.to_usize().unwrap()],
            self.map_dst[self.map_dst.len() - 1 - dst.to_usize().unwrap()],
        )
    }

    type Iter<'a> = MonoIter<'a,Self::Src,Self::Dst>
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
    T: 'a + WithChildren,
    Src: BorrowMut<LazyPostOrder<T, M::Src>>,
    Dst: BorrowMut<LazyPostOrder<T, M::Dst>>,
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
    SimpleHiddingMapper<
        'a,
        T,
        M::Src,
        LazyPostOrder<T, M::Src>,
        &'map Vec<M::Src>,
        &'map BTreeMap<M::Src, M::Src>,
        Src,
    >,
    SimpleHiddingMapper<
        'a,
        T,
        M::Dst,
        LazyPostOrder<T, M::Dst>,
        &'map Vec<M::Dst>,
        &'map BTreeMap<M::Dst, M::Dst>,
        Dst,
    >,
    MonoMappingWrap<'map, 'back, M::Src, M::Dst, M>,
)
where
    M::Src: PrimInt,
    M::Dst: PrimInt,
    <T as types::Stored>::TreeId: Clone + std::fmt::Debug,
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

// impl<'a, T: 'a + WithChildren, IdD: PrimInt, DTS: PostOrder<'a, T, IdD>, D: BorrowMut<DTS>>
//     SimpleHiddingMapper<'a, T, IdD, DTS, D>
// {

//     pub fn from_subtree_mapping<S, M: Index<usize>>(store: &'a S, back: D, side: &M) -> Self
//     where
//         S: NodeStore<T::TreeId, R<'a> = T>,
//         M::Output: PrimInt
//     {
//         let x: &DTS = back.borrow();
//         let mut map = Vec::with_capacity(x.len());
//         let mut i = x.root();

//         while num_traits::zero() < i {
//             map.push(cs);
//             i += 1;
//         }

//         map.shrink_to_fit();
//         Self {
//             map,
//             // fc,
//             back,
//             phantom: PhantomData,
//         }
//     }
// }

// impl<'a, T: WithChildren, IdD, DTS: DecompressedTreeStore<'a, T, IdD>, D: BorrowMut<DTS>>
//     Initializable<'a, T> for SimpleHiddingMapper<'a, T, IdD, DTS, D>
// {
//     fn make<S>(_store: &'a S, _root: &T::TreeId) -> Self
//     where
//         S: NodeStore<T::TreeId, R<'a> = T>,
//     {
//         panic!()
//     }
// }

impl<
        'a,
        T: WithChildren,
        IdD: PrimInt,
        DTS: DecompressedTreeStore<'a, T, IdD>,
        M: Borrow<Vec<IdD>>,
        R: Borrow<BTreeMap<IdD, IdD>>,
        D: BorrowMut<DTS>,
    > ShallowDecompressedTreeStore<'a, T, IdD> for SimpleHiddingMapper<'a, T, IdD, DTS, M, R, D>
{
    fn len(&self) -> usize {
        self.map.borrow().len()
    }

    fn original(&self, id: &IdD) -> T::TreeId {
        self.back
            .borrow()
            .original(&self.map.borrow()[self.len() - id.to_usize().unwrap() - 1])
    }

    fn root(&self) -> IdD {
        num_traits::cast(self.len() - 1).unwrap()
    }

    fn child<'b, S>(&self, store: &'b S, x: &IdD, p: &[T::ChildIdx]) -> IdD
    where
        S: 'b + NodeStore<T::TreeId, R<'b> = T>,
    {
        let b: &DTS = self.back.borrow();
        let c = b.child(
            store,
            &self.map.borrow()[self.len() - x.to_usize().unwrap() - 1],
            p,
        );
        *self.rev.borrow().get(&c).unwrap()
    }

    fn children<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: 'b + NodeStore<T::TreeId, R<'b> = T>,
    {
        let b: &DTS = self.back.borrow();
        let cs = b.children(
            store,
            &self.map.borrow()[self.len() - x.to_usize().unwrap() - 1],
        );
        cs.into_iter()
            .map(|x| *self.rev.borrow().get(&x).unwrap())
            .collect()
    }
}

impl<
        'a,
        T: WithChildren,                             // + WithStats,
        IdD: PrimInt+Debug,                                // + Shallow<IdD> + Debug,
        DTS: DecompressedTreeStore<'a, T, IdD, IdD> + PostOrder<'a,T,IdD>, // + LazyDecompressedTreeStore<'a, T, IdD>,
        M: Borrow<Vec<IdD>>,
        R: Borrow<BTreeMap<IdD, IdD>>,
        D: BorrowMut<DTS>,
    > DecompressedTreeStore<'a, T, IdD, IdD> for SimpleHiddingMapper<'a, T, IdD, DTS, M, R, D>
{
    fn descendants<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: 'b + NodeStore<T::TreeId, R<'b> = T>,
    {
        let cs = self.back.borrow().descendants(
            store,
            &self.map.borrow()[self.len() - x.to_usize().unwrap() - 1],
        );
        cs.into_iter()
            .filter_map(|x| self.rev.borrow().get(&x).copied())
            .collect()
    }

    fn descendants_count<'b, S>(&self, store: &'b S, x: &IdD) -> usize
    where
        S: 'b + NodeStore<T::TreeId, R<'b> = T>,
        // S: 'b + NodeStore<IdC>,
        // S::R<'b>: WithChildren<TreeId = IdC>,
    {
        self.descendants(store, x).len()
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        let conv = self.map.borrow()[self.map.borrow().len() - 1 - i.to_usize().unwrap()]; //self.back.borrow_mut().lld(aaa);
        let lld = self.back.borrow().lld(&conv);
        dbg!(lld, conv, i);
        let mut y = *i;
        loop {
            if y.is_zero() {
                break;
            }
            // dbg!(y, lld);
            let Some(conv) = self.map.borrow().get(self.map.borrow().len() - 1 - y.to_usize().unwrap()) else {
                break
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
        'd,
        T: WithChildren,
        IdD: PrimInt,
        DTS: DecompressedTreeStore<'d, T, IdD> + DecompressedWithParent<'d, T, IdD>,
        M: Borrow<Vec<IdD>>,
        R: Borrow<BTreeMap<IdD, IdD>>,
        D: BorrowMut<DTS>,
    > DecompressedWithParent<'d, T, IdD> for SimpleHiddingMapper<'d, T, IdD, DTS, M, R, D>
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

    fn position_in_parent(&self, c: &IdD) -> Option<T::ChildIdx> {
        self.back
            .borrow()
            .position_in_parent(&self.map.borrow()[self.len() - c.to_usize().unwrap() - 1])
    }

    type PIt<'a>=DTS::PIt<'a> where D: 'a, Self:'a;

    fn parents(&self, id: IdD) -> Self::PIt<'_> {
        // self.back.borrow().parents(id)
        todo!()
    }

    fn path(&self, parent: &IdD, descendant: &IdD) -> Vec<T::ChildIdx> {
        // self.back.borrow().path(parent, descendant)
        todo!()
    }

    fn lca(&self, a: &IdD, b: &IdD) -> IdD {
        // self.back.borrow().lca(a, b)
        todo!()
    }
}
impl<
        'a,
        T,
        IdD: PrimInt + Debug,
        DTS: DecompressedTreeStore<'a, T, IdD> + DecompressedWithParent<'a, T, IdD>,
        M: Borrow<Vec<IdD>>,
        R: Borrow<BTreeMap<IdD, IdD>>,
        D: BorrowMut<DTS>,
    > super::DecompressedSubtree<'a, T> for SimpleHiddingMapper<'a, T, IdD, DTS, M, R, D>
where
    T: WithChildren + WithStats,
    T::TreeId: Clone + Debug,
    <T as WithChildren>::ChildIdx: PrimInt,
{
    fn decompress<S>(store: &'a S, id: &<T as Stored>::TreeId) -> Self::Out
    where
        S: NodeStore<<T as Stored>::TreeId, R<'a> = T>,
    {
        todo!()
    }

    type Out = LazyPostOrder<T, IdD>;
}

impl<
        'a,
        T: WithChildren,
        IdD: PrimInt+Debug,
        DTS: PostOrder<'a, T, IdD>,
        M: Borrow<Vec<IdD>>,
        R: Borrow<BTreeMap<IdD, IdD>>,
        D: BorrowMut<DTS>,
    > PostOrder<'a, T, IdD> for SimpleHiddingMapper<'a, T, IdD, DTS, M, R, D>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn lld(&self, i: &IdD) -> IdD {
        todo!()
        // // TODO not sure
        // let c = self
        //     .back
        //     .borrow()
        //     .lld(&self.map.borrow()[self.len() - i.to_usize().unwrap() - 1]);
        // *self.rev.borrow().get(&c).unwrap()
    }

    fn tree(&self, id: &IdD) -> T::TreeId {
        self.back
            .borrow()
            .tree(&self.map.borrow()[self.len() - id.to_usize().unwrap() - 1])
    }
}
impl<
        'd,
        T: WithChildren + 'd,
        IdD: PrimInt+Debug,
        DTS: DecompressedTreeStore<'d, T, IdD> + DecompressedWithParent<'d, T, IdD> + PostOrder<'d,T,IdD>,
        M: Borrow<Vec<IdD>>,
        R: Borrow<BTreeMap<IdD, IdD>>,
        D: BorrowMut<DTS>,
    > PostOrderIterable<'d, T, IdD> for SimpleHiddingMapper<'d, T, IdD, DTS, M, R, D>
where
    T::TreeId: Clone + Debug,
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
        T: 'd + WithChildren,
        IdD: PrimInt + Debug,
        DTS: DecompressedTreeStore<'d, T, IdD>
            + DecompressedWithParent<'d, T, IdD>
            + PostOrder<'d, T, IdD>,
        M: Borrow<Vec<IdD>>,
        R: Borrow<BTreeMap<IdD, IdD>>,
        D: BorrowMut<DTS>,
    > ContiguousDescendants<'d, T, IdD, IdD> for SimpleHiddingMapper<'d, T, IdD, DTS, M, R, D>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdD> {
        let conv = self.map.borrow()[self.map.borrow().len() - 1 - x.to_usize().unwrap()]; //self.back.borrow_mut().lld(aaa);
        let lld = self.back.borrow().lld(&conv);
        let mut y = *x;
        // dbg!(self.map.borrow());
        loop {
            if y.is_zero() {
                break;
            }
            // dbg!(y, lld);
            let Some(conv) = self.map.borrow().get(self.map.borrow().len() - 1 - y.to_usize().unwrap()) else {
                break
            };
            if lld < *conv {
                y = y - num_traits::one();
            } else {
                break;
            }
        }
        y..*x
    }

    type Slice<'b> = SimplePOSlice<'b,T,IdD> where Self: 'b;

    fn slice(&self, x: &IdD) -> Self::Slice<'_> {
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
//     > DecompressedWithParent<'d, T, IdD> for SimpleHiddingMapper<'d, T, IdD, DTS, M, D>
// where
//     T::TreeId: Clone + Eq + Debug,
// {
//     fn has_parent(&self, id: &IdD) -> bool {
//         <LazyPostOrder<T, IdD>>::has_parent(&self, id)
//     }

//     fn parent(&self, id: &IdD) -> Option<IdD> {
//         <LazyPostOrder<T, IdD>>::parent(&self, id)
//     }

//     type PIt<'a> = IterParents<'a, IdD> where IdD: 'a, T::TreeId:'a, T: 'a, Self: 'a;

//     fn parents(&self, id: IdD) -> Self::PIt<'_> {
//         <LazyPostOrder<T, IdD>>::parents(&self, id)
//     }

//     fn position_in_parent(&self, c: &IdD) -> Option<<T as WithChildren>::ChildIdx> {
//         <LazyPostOrder<T, IdD>>::position_in_parent(&self, c)
//     }

//     fn path(&self, parent: &IdD, descendant: &IdD) -> Vec<<T as WithChildren>::ChildIdx> {
//         <LazyPostOrder<T, IdD>>::path(&self, parent, descendant)
//     }

//     fn lca(&self, a: &IdD, b: &IdD) -> IdD {
//         <LazyPostOrder<T, IdD>>::lca(&self, a, b)
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

// impl<'a, T: WithChildren, IdD: PrimInt> ShallowDecompressedTreeStore<'a, T, IdD>
//     for SimpleHiddingMapper<'d, T, IdD, DTS, M, D>
// where
//     T::TreeId: Clone + Eq + Debug,
// {
//     fn len(&self) -> usize {
//         <LazyPostOrder<T, IdD>>::len(&self)
//     }

//     fn original(&self, id: &IdD) -> <T>::TreeId {
//         <LazyPostOrder<T, IdD>>::original(&self, id)
//     }

//     fn root(&self) -> IdD {
//         <LazyPostOrder<T, IdD>>::root(&self)
//     }

//     fn child<'b, S>(&self, store: &'b S, x: &IdD, p: &[<T as WithChildren>::ChildIdx]) -> IdD
//     where
//         //'a: 'b,
//         S: 'b + NodeStore<<T>::TreeId, R<'b> = T>,
//     {
//         <LazyPostOrder<T, IdD>>::child(&self, store, x, p)
//     }

//     fn children<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
//     where
//         // 'a: 'b,
//         S: NodeStore<<T>::TreeId, R<'b> = T>,
//     {
//         <LazyPostOrder<T, IdD>>::children(&self, store, x)
//     }
// }

impl<
        'd,
        T: WithChildren + WithStats,
        IdS: PrimInt + Shallow<IdS> + Debug,
        DTS: LazyDecompressedTreeStore<'d, T, IdS>, // + DecompressedTreeStore<'d, T, IdD>,
        M: Borrow<Vec<<DTS as LazyDecompressedTreeStore<'d, T, IdS>>::IdD>>,
        R: Borrow<
            BTreeMap<
                <DTS as LazyDecompressedTreeStore<'d, T, IdS>>::IdD,
                <DTS as LazyDecompressedTreeStore<'d, T, IdS>>::IdD,
            >,
        >,
        D: BorrowMut<DTS>,
    > LazyDecompressedTreeStore<'d, T, IdS>
    for SimpleHiddingMapper<
        'd,
        T,
        <DTS as LazyDecompressedTreeStore<'d, T, IdS>>::IdD,
        DTS,
        M,
        R,
        D,
    >
where
    T::TreeId: Clone + Eq + Debug,
    Self: DecompressedTreeStore<'d, T, <DTS as LazyDecompressedTreeStore<'d, T, IdS>>::IdD, IdS>,
    <DTS as LazyDecompressedTreeStore<'d, T, IdS>>::IdD: PrimInt + Shallow<IdS>,
{
    type IdD = <DTS as LazyDecompressedTreeStore<'d, T, IdS>>::IdD;

    fn starter(&self) -> Self::IdD {
        num_traits::cast(self.len() - 1).unwrap()
    }

    fn decompress_children<'b, S>(&mut self, store: &'b S, x: &Self::IdD) -> Vec<Self::IdD>
    where
        S: NodeStore<<T>::TreeId, R<'b> = T>,
    {
        let len = self.len();
        let b: &mut DTS = self.back.borrow_mut();
        let cs = b.decompress_children(store, &self.map.borrow()[len - x.to_usize().unwrap() - 1]);
        cs.into_iter()
            .map(|x| *self.rev.borrow().get(&x).unwrap())
            .collect()
    }

    fn decompress_to<'b, S>(&mut self, store: &'b S, x: &IdS) -> Self::IdD
    where
        S: NodeStore<<T>::TreeId, R<'b> = T>,
    {
        let len = self.len();
        let b: &mut DTS = self.back.borrow_mut();
        let c = b.decompress_to(
            store,
            self.map.borrow()[len - x.to_usize().unwrap() - 1].shallow(),
        );
        *self.rev.borrow().get(&c).unwrap()
    }
}

// impl<'d, T: WithChildren, IdD: PrimInt> DecompressedTreeStore<'d, T, IdD>
//     for SimpleHiddingMapper<'d, T, IdD, DTS, M, D>
// where
//     T::TreeId: Clone + Eq + Debug,
// {
//     fn descendants<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
//     where
//         S: NodeStore<<T>::TreeId, R<'b> = T>,
//     {
//         <LazyPostOrder<T, IdD>>::descendants(&self, store, x)
//     }

//     fn descendants_count<'b, S>(&self, store: &'b S, x: &IdD) -> usize
//     where
//         S: NodeStore<<T>::TreeId, R<'b> = T>,
//     {
//         <LazyPostOrder<T, IdD>>::descendants_count(&self, store, x)
//     }

//     fn first_descendant(&self, i: &IdD) -> IdD {
//         <LazyPostOrder<T, IdD>>::first_descendant(&self, i)
//     }

//     fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
//         <LazyPostOrder<T, IdD>>::is_descendant(&self, desc, of)
//     }
// }

impl<
        'a,
        T: WithChildren + WithStats,
        IdD: PrimInt + Shallow<IdD> + Debug,
        M: Borrow<Vec<IdD>>,
        R: Borrow<BTreeMap<IdD, IdD>>,
        D: BorrowMut<LazyPostOrder<T, IdD>>,
    > SimpleHiddingMapper<'a, T, IdD, LazyPostOrder<T, IdD>, M, R, D>
where
    <T as Stored>::TreeId: Clone,
    <T as Stored>::TreeId: Debug,
{
    fn decompress_visible_descendants<'b, S>(&mut self, store: &'b S, x: &IdD)
    where
        S: NodeStore<T::TreeId, R<'b> = T>,
    {
        let mut q: Vec<IdD> =
            vec![self.map.borrow()[self.map.borrow().len() - 1 - x.to_usize().unwrap()]];
        while let Some(x) = q.pop() {
            if !self.rev.borrow().contains_key(&x) {
                continue;
            }
            if self.back.borrow().descendants_count(store, &x) == 0 {
                continue;
            }
            if !self.rev.borrow().contains_key(&(x - num_traits::one())) {
                continue;
            }
            assert!(self.back.borrow().id_parent[x.to_usize().unwrap()] != zero());
            q.extend(self.back.borrow_mut().decompress_children(store, &x));
        }
    }
}

impl<
        'd,
        T: 'd + WithChildren,
        IdD: PrimInt,
        // DTS: DecompressedTreeStore<'d, T, IdD> + DecompressedWithParent<'d, T, IdD>,
        M: Borrow<Vec<IdD>>,
        R: Borrow<BTreeMap<IdD, IdD>>,
        D: BorrowMut<LazyPostOrder<T, IdD>>,
    > LazyPOBorrowSlice<'d, T, IdD, IdD>
    for SimpleHiddingMapper<'d, T, IdD, LazyPostOrder<T, IdD>, M, R, D>
where
    T: WithStats,
    T::TreeId: Clone + Eq + Debug,
    IdD: Shallow<IdD> + Debug,
{
    type SlicePo<'b> = CompleteWHPO<'b,T,IdD, bitvec::boxed::BitBox>
    where
        Self: 'b;

    fn slice_po<'b, S>(&mut self, store: &'b S, x: &IdD) -> Self::SlicePo<'_>
    where
        S: NodeStore<<T>::TreeId, R<'b> = T>,
    {
        self.decompress_visible_descendants(store, x);
        // let aaa = &self.map.borrow()[self.map.borrow().len() - 1 - x.to_usize().unwrap()];
        // dbg!(&aaa);
        // let mut aaa = *aaa; //self.back.borrow_mut().lld(aaa);
        // dbg!(&aaa);
        // loop {
        //     dbg!(&aaa);
        //     let cs = self.back.borrow_mut().decompress_children(store, &aaa);
        //     let c = *cs.get(0).unwrap();
        //     // assert_ne!(c, aaa);
        //     if !self.rev.borrow().contains_key(&c) {
        //         dbg!(c);
        //         break;
        //     }
        //     aaa = c;
        // }
        // let map_lld = self.rev.borrow().get(&aaa).unwrap();
        let map_lld = self.first_descendant(x);
        // dbg!(x, map_lld, &self.map.borrow(), self.rev.borrow());
        // TODO extract actual values
        let len = x.to_usize().unwrap() - map_lld.to_usize().unwrap() + 1;
        // - id_compressed ez
        let mut id_compressed: Vec<T::TreeId> = Vec::with_capacity(len);
        // - id_parent: direct resolve ?
        let mut id_parent: Vec<IdD> = Vec::with_capacity(len);
        // - kr: adapt the algo
        let mut kr = bitvec::bitbox!(0;len);
        // - llds: should be easy when extracting
        let mut llds: Vec<IdD> = vec![*x; len];
        let mut curr = map_lld.clone();
        dbg!();
        while curr <= *x {
            let conv = self.map.borrow()[self.map.borrow().len()-1-curr.to_usize().unwrap()];
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
        dbg!();
        let mut visited = bitvec::bitbox!(0; len);
        for i in (1..len).rev() {
            if !visited[llds[i].to_usize().unwrap()] {
                kr.set(i, true);
                // kr.push(cast(i + 1).unwrap());
                visited.set(llds[i].to_usize().unwrap(), true);
            }
        }
        dbg!(id_compressed.len(),id_parent.len(), llds.len(), self.map.borrow().len());
        CompleteWHPO {
            map: self.map.borrow(),
            id_compressed,
            llds,
            id_parent,
            kr,
        }
    }
}

pub struct CompleteWHPO<'a, T: Stored, IdD, Kr: Borrow<BitSlice>> {
    pub(crate) map: &'a [IdD],
    pub(crate) id_compressed: Vec<T::TreeId>,
    pub(crate) llds: Vec<IdD>,
    pub(crate) id_parent: Vec<IdD>,
    pub(super) kr: Kr,
}

// impl<'a, T: Stored, IdD, Kr: Borrow<BitSlice>> Deref for CompletePOSlice<'a, T, IdD, Kr> {
//     type Target = SimplePOSlice<'a, T, IdD>;

//     fn deref(&self) -> &Self::Target {
//         &self.simple
//     }
// }

impl<'a, T: WithChildren, IdD: PrimInt, Kr: Borrow<BitSlice>>
    ShallowDecompressedTreeStore<'a, T, IdD> for CompleteWHPO<'a, T, IdD, Kr>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn len(&self) -> usize {
        self.id_compressed.len()
    }

    fn original(&self, id: &IdD) -> T::TreeId {
        self.id_compressed[id.to_usize().unwrap()].clone()
    }

    // fn leaf_count(&self) -> IdD {
    //     cast(self.kr.len()).unwrap()
    // }

    fn root(&self) -> IdD {
        cast(self.len() - 1).unwrap()
    }

    fn child<'b, S>(&self, store: &'b S, x: &IdD, p: &[T::ChildIdx]) -> IdD
    where
        S: NodeStore<T::TreeId, R<'b> = T>,
    {
        todo!()
        // self.simple.child(store, x, p)
    }

    fn children<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: NodeStore<T::TreeId, R<'b> = T>,
    {
        todo!()
        // self.simple.children(store, x)
    }
}

impl<'a, T: WithChildren, IdD: PrimInt, Kr: Borrow<BitSlice>> DecompressedTreeStore<'a, T, IdD>
    for CompleteWHPO<'a, T, IdD, Kr>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn descendants<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: 'b + NodeStore<T::TreeId, R<'b> = T>,
    {
        todo!()
        // self.simple.descendants(store, x)
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        todo!()
        // self.simple.first_descendant(i)
    }

    fn descendants_count<'b, S>(&self, store: &'b S, x: &IdD) -> usize
    where
        S: 'b + NodeStore<T::TreeId, R<'b> = T>,
    {
        todo!()
        // self.simple.descendants_count(store, x)
    }

    fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
        todo!()
        // self.simple.is_descendant(desc, of)
    }
}

impl<'a, T: WithChildren, IdD: PrimInt, Kr: Borrow<BitSlice>> PostOrder<'a, T, IdD>
    for CompleteWHPO<'a, T, IdD, Kr>
where
    T::TreeId: Clone + Eq + Debug,
{
    fn lld(&self, i: &IdD) -> IdD {
        self.llds[i.to_usize().unwrap()]
        // self.simple.lld(i)
    }

    fn tree(&self, id: &IdD) -> T::TreeId {
        self.id_compressed[id.to_usize().unwrap()].clone()
        // self.simple.tree(id)
    }
}

impl<'a, T: WithChildren + 'a, IdD: PrimInt, Kr: Borrow<BitSlice>> PostOrderKeyRoots<'a, T, IdD>
    for CompleteWHPO<'a, T, IdD, Kr>
where
    T::TreeId: Clone + Eq + Debug,
{
    // fn kr(&self, x: IdD) -> IdD {
    //     self.kr[x.to_usize().unwrap()]
    // }
    type Iter<'b> = IterKr<'b,IdD>
    where
        Self: 'b;

    fn iter_kr(&self) -> Self::Iter<'_> {
        IterKr(self.kr.borrow().iter_ones(), PhantomData)
    }
}
