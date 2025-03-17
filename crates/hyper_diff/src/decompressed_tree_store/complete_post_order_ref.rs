use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
    ops::Deref,
};

use bitvec::slice::BitSlice;
use num_traits::{cast, ToPrimitive, Zero};

use hyperast::{
    position::Position,
    types::{
        self, HyperAST, HyperType, NodeId, NodeStore, Stored, Tree, WithChildren, WithSerialization,
    },
};
use hyperast::{test_utils::tree::WithStats, PrimInt};

use super::{
    lazy_post_order::LazyPostOrder,
    pre_order_wrapper::{DisplaySimplePreOrderMapper, SimplePreOrderMapper},
    CIdx, DecompressedParentsLending, DecompressedTreeStore, DecompressedWithParent,
    DecompressedWithSiblings, FullyDecompressedTreeStore, Iter, IterKr, PostOrdKeyRoots, PostOrder,
    PostOrderIterable, PostOrderKeyRoots, ShallowDecompressedTreeStore,
};
use crate::matchers::Decompressible;

use logging_timer::time;

/// made for TODO
/// - post order
/// - key roots
/// - parents
pub struct CompletePostOrder<'a, IdN, IdD> {
    pub(super) lazy: &'a LazyPostOrder<IdN, IdD>,
    /// LR_keyroots(T) = {k | there exists no k < k' such that l(k) = l(k’)}.
    pub(super) kr: bitvec::boxed::BitBox,
}

impl<IdN, IdD> LazyPostOrder<IdN, IdD> {
    pub fn as_slice(&self) -> LazyPOSlice<'_, IdN, IdD> {
        LazyPOSlice {
            id_parent: &self.id_parent,
            id_compressed: &self.id_compressed[..],
            llds: &self.llds,
        }
    }
}

impl<'a, IdN, IdD> Deref for CompletePostOrder<'a, IdN, IdD> {
    type Target = LazyPostOrder<IdN, IdD>;

    fn deref(&self) -> &Self::Target {
        &self.lazy
    }
}

impl<'a, HAST: HyperAST + Copy, IdD> Decompressible<HAST, CompletePostOrder<'a, HAST::IdN, IdD>> {
    fn as_lazy(&self) -> Decompressible<HAST, &'a LazyPostOrder<HAST::IdN, IdD>> {
        let hyperast = self.hyperast;
        let decomp = &self.lazy;
        Decompressible { hyperast, decomp }
    }
}

impl<'a, IdN, IdD> CompletePostOrder<'a, IdN, IdD> {
    pub fn as_slice(&self) -> CompletePOSlice<'_, IdN, IdD, &'_ BitSlice> {
        CompletePOSlice {
            simple: self.lazy.as_slice(),
            kr: &self.kr,
        }
    }
}

impl<'a, IdN, IdD: PrimInt> CompletePostOrder<'a, IdN, IdD> {
    pub fn iter(&self) -> impl Iterator<Item = &IdN> {
        self.lazy.iter()
    }
}

impl<'a, IdN, IdD: PrimInt> From<&'a LazyPostOrder<IdN, IdD>> for CompletePostOrder<'a, IdN, IdD> {
    #[time("warn")]
    fn from(lazy: &'a LazyPostOrder<IdN, IdD>) -> Self {
        let kr = lazy._compute_kr_bitset();
        Self { lazy, kr }
    }
}

impl<'a, IdN, IdD: PrimInt> From<&'a mut LazyPostOrder<IdN, IdD>> for CompletePostOrder<'a, IdN, IdD> {
    fn from(lazy: &'a mut LazyPostOrder<IdN, IdD>) -> Self {
        let kr = lazy._compute_kr_bitset();
        Self { lazy, kr }
    }
}

impl<'a, IdN, IdD: PrimInt + Debug> Debug for CompletePostOrder<'a, IdN, IdD>
where
    IdN: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompletePostOrder")
            .field("simple", &self.lazy)
            .field("kr", &self.kr)
            .finish()
    }
}

pub struct DisplayCompletePostOrder<'store: 'a, 'a, IdD: PrimInt, HAST, D>
where
    HAST: HyperAST,
    // D: ShallowDecompressedTreeStore<HAST::IdN, IdD>,
{
    inner: &'a D,
    stores: &'store HAST,
    _phantom: PhantomData<&'a IdD>,
}
impl<'store: 'a, 'a, IdD: PrimInt, HAST, D> DisplayCompletePostOrder<'store, 'a, IdD, HAST, D>
where
    HAST: HyperAST,
    // D: ShallowDecompressedTreeStore<HAST::IdN, IdD>,
{
    pub fn new(stores: &'store HAST, inner: &'a D) -> Self {
        Self {
            inner,
            stores,
            _phantom: PhantomData,
        }
    }
}
impl<'store: 'a, 'a, IdD: PrimInt, HAST, D> Display
    for DisplayCompletePostOrder<'store, 'a, IdD, HAST, D>
where
    HAST: HyperAST + Copy,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithSerialization,
    // // T::TreeId: Clone + Debug,
    // // T::Type: Copy + Debug + Send + Sync,
    D: DecompressedTreeStore<HAST, IdD>
        + PostOrder<HAST, IdD>
        + FullyDecompressedTreeStore<HAST, IdD>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
        // let m = SimplePreOrderMapper::from(self.inner);
        // std::fmt::Display::fmt(
        //     &DisplaySimplePreOrderMapper {
        //         inner: m,
        //         stores: self.stores,
        //     },
        //     f,
        // )
    }
}

impl<'store: 'a, 'a, IdD: PrimInt, HAST, D> Debug
    for DisplayCompletePostOrder<'store, 'a, IdD, HAST, D>
where
    HAST: HyperAST + Copy,
    // // for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithSerialization,
    // // T::TreeId: Clone + Debug,
    // // S: NodeStore<T::TreeId, R<'store> = T>,
    // // T: Tree,
    // // T::Type: Copy + Debug + Send + Sync,
    // // LS: LabelStore<str, I = T::Label>,
    D: DecompressedTreeStore<HAST, IdD>
        + PostOrder<HAST, IdD>
        + FullyDecompressedTreeStore<HAST, IdD>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
        // let m = SimplePreOrderMapper::from(self.inner);
        // DisplaySimplePreOrderMapper {
        //     inner: m,
        //     stores: self.stores,
        // }
        // .fmt(f)
    }
}

impl<'a, 'b, HAST: HyperAST + Copy, IdD: PrimInt> DecompressedParentsLending<'a, IdD>
    for Decompressible<HAST, CompletePostOrder<'b, HAST::IdN, IdD>>
{
    type PIt = <Decompressible<HAST, &'a mut LazyPostOrder<HAST::IdN, IdD>> as DecompressedParentsLending<'a, IdD>>::PIt;
}

impl<'b, 'd, HAST: HyperAST + Copy, IdD: PrimInt> DecompressedWithParent<HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<'b, HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    // for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren + WithStats,
{
    fn parent(&self, id: &IdD) -> Option<IdD> {
        self.as_lazy().parent(id)
    }

    fn has_parent(&self, id: &IdD) -> bool {
        self.as_lazy().has_parent(id)
    }

    fn position_in_parent<Idx: PrimInt>(&self, c: &IdD) -> Option<Idx> {
        self.as_lazy().position_in_parent(c)
    }

    fn parents(&self, id: IdD) -> <Self as DecompressedParentsLending<'_, IdD>>::PIt {
        super::lazy_post_order::IterParents {
            id,
            id_parent: &self.id_parent,
        }
    }

    fn lca(&self, a: &IdD, b: &IdD) -> IdD {
        self.as_lazy().lca(a, b)
    }

    fn path<Idx: PrimInt>(&self, parent: &IdD, descendant: &IdD) -> Vec<Idx> {
        self.as_lazy().path(parent, descendant)
    }
}

// impl<'a, 'd, T: WithChildren, IdD: PrimInt> DecompressedWithSiblings<'d, T, IdD>
//     for CompletePostOrder<'a, HAST::IdN, IdD>
// where
//     T::TreeId: Debug + NodeId<IdN = T::TreeId>,
// {
//     fn lsib(&self, x: &IdD) -> Option<IdD> {
//         DecompressedWithSiblings::lsib(&self.simple, x)
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

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> PostOrder<HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<'a, HAST::IdN, IdD>>
where
    // for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren + WithStats,
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn lld(&self, i: &IdD) -> IdD {
        self.as_lazy().lld(i)
    }

    fn tree(&self, id: &IdD) -> HAST::IdN {
        self.as_lazy().tree(id)
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> PostOrderIterable<HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<'a, HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    // for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren + WithStats,
{
    type It = Iter<IdD>;
    fn iter_df_post<const ROOT: bool>(&self) -> Iter<IdD> {
        self.as_lazy().iter_df_post::<ROOT>()
    }
}

impl<'a, 'b, HAST: HyperAST + Copy, IdD: PrimInt> PostOrdKeyRoots<'b, HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<'a, HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    // for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren + WithStats,
{
    type Iter = IterKr<'b, IdD>;
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> PostOrderKeyRoots<HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<'a, HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    // for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren + WithStats,
{
    fn iter_kr(&self) -> <Self as PostOrdKeyRoots<'_, HAST, IdD>>::Iter {
        IterKr(self.kr.iter_ones(), PhantomData)
    }
}

// impl<'b, 'a, HAST: HyperAST + Copy, IdD: PrimInt> super::DecompressedSubtree<T>
//     for CompletePostOrder<'b, T, IdD>
// where
//     T::TreeId: Clone + NodeId<IdN = T::TreeId>,
//     <T as WithChildren>::ChildIdx: PrimInt,
// {
//     type Out = Self;

//     fn decompress<S>(store: &'a S, root: &<T as types::Stored>::TreeId) -> Self
//     where
//         S: NodeStore<<T as types::Stored>::TreeId, R<'a> = T>,
//     {
//         SimplePostOrder::decompress(store, root).into()
//     }
// }

// impl<'a, 'b, HAST: HyperAST + Copy, IdD: PrimInt> types::NLending<'b, T::TreeId>
//     for CompletePostOrder<'a, HAST::IdN, IdD>
// where
//     // T: for<'t> types::NLending<'t, T::TreeId>,
// {
//     type N = <T as types::NLending<'a, T::TreeId>>::N;
// }

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> ShallowDecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<'a, HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn len(&self) -> usize {
        self.as_lazy().len()
    }

    fn original(&self, id: &IdD) -> HAST::IdN {
        self.as_lazy().original(id)
    }

    fn root(&self) -> IdD {
        cast(self.len() - 1).unwrap()
    }

    fn child(&self, x: &IdD, p: &[impl PrimInt]) -> IdD {
        self.as_lazy().child(x, p)
    }

    fn children(&self, x: &IdD) -> Vec<IdD> {
        self.as_lazy().children(x)
    }
}

impl<'d, HAST: HyperAST + Copy, IdD: PrimInt> DecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<'d, HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    // for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren + WithStats,
{
    fn descendants(&self, x: &IdD) -> Vec<IdD> {
        self.as_lazy().descendants(x)
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.as_lazy().first_descendant(i)
    }

    fn descendants_count(&self, x: &IdD) -> usize {
        self.as_lazy().descendants_count(x)
    }

    fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
        self.as_lazy().is_descendant(desc, of)
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> FullyDecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<'a, HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
}

// impl<'a, T: WithChildren, IdD: PrimInt> ContiguousDescendants<'a, T, IdD>
//     for CompletePostOrder<'a, HAST::IdN, IdD>
// where
//     T::TreeId: Debug + NodeId<IdN = T::TreeId>,
// {
//     fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdD> {
//         self.first_descendant(x)..*x
//     }

//     type Slice<'b>=CompletePOSlice<'b,T,IdD,&'b BitSlice>
//     where
//         Self: 'b;

//     fn slice(&self, x: &IdD) -> <Self as DecendantsLending<'_>>::Slice {
//         let range = self.slice_range(x);
//         CompletePOSlice {
//             simple: self.lazy.slice(x),
//             kr: &self.kr[range],
//         }
//     }
// }

// impl<'a, T: WithChildren, IdD: PrimInt> POBorrowSlice<'a, T, IdD> for CompletePostOrder<'a, HAST::IdN, IdD>
// where
//     T::TreeId: Debug + NodeId<IdN = T::TreeId>,
// {
//     type SlicePo<'b>=Self::Slice<'b>
//     where
//         Self: 'b;

//     fn slice_po(&self, x: &IdD) -> <Self as DecendantsLending<'_>>::Slice {
//         self.slice(x)
//     }
// }

pub struct RecCachedPositionProcessor<'a, IdN, IdD: Hash + Eq> {
    pub(crate) ds: &'a CompletePostOrder<'a, IdN, IdD>,
    root: IdN,
    cache: HashMap<IdD, Position>,
}

impl<'a, IdN, IdD: PrimInt + Hash + Eq> From<(&'a CompletePostOrder<'a, IdN, IdD>, IdN)>
    for RecCachedPositionProcessor<'a, IdN, IdD>
{
    fn from((ds, root): (&'a CompletePostOrder<'a, IdN, IdD>, IdN)) -> Self {
        Self {
            ds,
            root,
            cache: Default::default(),
        }
    }
}

// impl<'a, T: Tree, IdD: PrimInt + Hash + Eq> RecCachedPositionProcessor<'a, T, IdD> {
//     pub fn position<'b, HAST>(&mut self, stores: &'b HAST, c: &IdD) -> &Position
//     where
//         HAST: HyperAST<'b, IdN = T::TreeId, T = T, Label = T::Label>, //NodeStore<T::TreeId, R<'b> = T>,
//         T::TreeId: Clone + Debug + NodeId<IdN = T::TreeId>,
//         // LS: LabelStore<str>,
//         T: WithSerialization,
//         // T: Tree<Label = LS::I> + WithSerialization,
//     {
//         if self.cache.contains_key(&c) {
//             return self.cache.get(&c).unwrap();
//         } else if let Some(p) = self.ds.parent(c) {
//             let p_r = stores.node_store().resolve(&self.ds.original(&p));
//             let p_t = stores.type_store().resolve_type(&p_r);
//             if p_t.is_directory() {
//                 let ori = self.ds.original(&c);
//                 if self.root == ori {
//                     let r = stores.node_store().resolve(&ori);
//                     return self.cache.entry(*c).or_insert(Position::new(
//                         stores.label_store().resolve(r.get_label_unchecked()).into(),
//                         0,
//                         r.try_bytes_len().unwrap_or(0),
//                     ));
//                 }
//                 let mut pos = self
//                     .cache
//                     .get(&p)
//                     .cloned()
//                     .unwrap_or_else(|| self.position(stores, &p).clone());
//                 let r = stores.node_store().resolve(&ori);
//                 pos.inc_path(stores.label_store().resolve(r.get_label_unchecked()));
//                 pos.set_len(r.try_bytes_len().unwrap_or(0));
//                 return self.cache.entry(*c).or_insert(pos);
//             }

//             if let Some(lsib) = self.ds.lsib(c) {
//                 assert_ne!(lsib.to_usize(), c.to_usize());
//                 let mut pos = self
//                     .cache
//                     .get(&lsib)
//                     .cloned()
//                     .unwrap_or_else(|| self.position(stores, &lsib).clone());
//                 pos.inc_offset(pos.range().end - pos.range().start);
//                 let r = stores.node_store().resolve(&self.ds.original(&c));
//                 pos.set_len(r.try_bytes_len().unwrap());
//                 self.cache.entry(*c).or_insert(pos)
//             } else {
//                 assert!(
//                     self.ds.position_in_parent(c).unwrap().is_zero(),
//                     "{:?}",
//                     self.ds.position_in_parent(c).unwrap().to_usize()
//                 );
//                 let ori = self.ds.original(&c);
//                 if self.root == ori {
//                     let r = stores.node_store().resolve(&ori);
//                     return self.cache.entry(*c).or_insert(Position::new(
//                         "".into(),
//                         0,
//                         r.try_bytes_len().unwrap(),
//                     ));
//                 }
//                 let mut pos = self
//                     .cache
//                     .get(&p)
//                     .cloned()
//                     .unwrap_or_else(|| self.position(stores, &p).clone());
//                 let r = stores.node_store().resolve(&ori);
//                 pos.set_len(
//                     r.try_bytes_len()
//                         .unwrap_or_else(|| panic!("{:?}", stores.type_store().resolve_type(&r))),
//                 );
//                 self.cache.entry(*c).or_insert(pos)
//             }
//         } else {
//             let ori = self.ds.original(&c);
//             assert_eq!(self.root, ori);
//             let r = stores.node_store().resolve(&ori);
//             let t = stores.type_store().resolve_type(&r);
//             let pos = if t.is_directory() || t.is_file() {
//                 let file = stores.label_store().resolve(r.get_label_unchecked()).into();
//                 let offset = 0;
//                 let len = r.try_bytes_len().unwrap_or(0);
//                 Position::new(file, offset, len)
//             } else {
//                 let file = "".into();
//                 let offset = 0;
//                 let len = r.try_bytes_len().unwrap_or(0);
//                 Position::new(file, offset, len)
//             };
//             self.cache.entry(*c).or_insert(pos)
//         }
//     }
// }
pub struct RecCachedProcessor<'a, IdN, D, IdD: Hash + Eq, U, F, G> {
    pub(crate) ds: &'a D,
    root: IdN,
    cache: HashMap<IdD, U>,
    with_p: F,
    with_lsib: G,
}

impl<'a, IdN, D, IdD: PrimInt + Hash + Eq, U, F, G> From<(&'a D, IdN, F, G)>
    for RecCachedProcessor<'a, IdN, D, IdD, U, F, G>
{
    fn from((ds, root, with_p, with_lsib): (&'a D, IdN, F, G)) -> Self {
        Self {
            ds,
            root,
            cache: Default::default(),
            with_p,
            with_lsib,
        }
    }
}

impl<'a, IdN, D, IdD: PrimInt + Hash + Eq, U: Clone + Default, F, G>
    RecCachedProcessor<'a, IdN, D, IdD, U, F, G>
where
    F: Fn(U, IdN) -> U,
    G: Fn(U, IdN) -> U,
    IdN: NodeId<IdN = IdN>,
    IdN: Debug,
{
    pub fn position<HAST: HyperAST<IdN = IdN> + Copy>(&mut self, store: HAST, c: &IdD) -> &U
    where
        D: DecompressedTreeStore<HAST, IdD>
            + DecompressedWithSiblings<HAST, IdD>
            + PostOrder<HAST, IdD>,
        // T::TreeId: Clone + Debug,
        // T: Tree + WithSerialization,
    {
        if self.cache.contains_key(&c) {
            return self.cache.get(&c).unwrap();
        } else if let Some(p) = self.ds.parent(c) {
            let id = self.ds.original(&p);
            let p_r = store.node_store().resolve(&id);
            let p_t = store.resolve_type(&id);
            if p_t.is_directory() {
                let ori = self.ds.original(&c);
                if self.root == ori {
                    // let r = store.resolve(&ori);
                    return self
                        .cache
                        .entry(*c)
                        .or_insert((self.with_p)(Default::default(), ori));
                    // Position::new(
                    //     lstore.resolve(&r.get_label()).into(),
                    //     0,
                    //     r.try_bytes_len().unwrap_or(0),
                    // )
                }
                let pos = self.position(store, &p).clone();
                // let r = store.resolve(&ori);
                // pos.inc_path(lstore.resolve(&r.get_label()));
                // pos.set_len(r.try_bytes_len().unwrap_or(0));
                // return self.cache.entry(*c).or_insert(pos);
                return self.cache.entry(*c).or_insert((self.with_p)(pos, ori));
            }

            if let Some(lsib) = self.ds.lsib(c) {
                assert_ne!(lsib.to_usize(), c.to_usize());
                let pos = self.position(store, &lsib).clone();
                // pos.inc_offset(pos.range().end - pos.range().start);
                // let r = store.resolve(&self.ds.original(&c));
                // pos.set_len(r.try_bytes_len().unwrap());
                // self.cache.entry(*c).or_insert(pos)
                self.cache
                    .entry(*c)
                    .or_insert((self.with_lsib)(pos, self.ds.original(&c)))
            } else {
                assert!(
                    self.ds.position_in_parent::<usize>(c).unwrap().is_zero(),
                    "{:?}",
                    self.ds.position_in_parent::<usize>(c).unwrap().to_usize()
                );
                let ori = self.ds.original(&c);
                if self.root == ori {
                    // let r = store.resolve(&ori);
                    return self
                        .cache
                        .entry(*c)
                        .or_insert((self.with_p)(Default::default(), ori));
                    // Position::new(
                    //     "".into(),
                    //     0,
                    //     r.try_bytes_len().unwrap(),
                    // )
                }
                let pos = self.position(store, &p).clone();
                // let r = store.resolve(&ori);
                // pos.set_len(
                //     r.try_bytes_len()
                //         .unwrap_or_else(|| panic!("{:?}", r.get_type())),
                // );
                // self.cache.entry(*c).or_insert(pos)
                self.cache.entry(*c).or_insert((self.with_p)(pos, ori))
            }
        } else {
            let ori = self.ds.original(&c);
            assert_eq!(self.root, ori);
            // let r = store.resolve(&ori);
            // let t = r.get_type();
            // let pos = if t.is_directory() || t.is_file() {
            //     let file = lstore.resolve(&r.get_label()).into();
            //     let offset = 0;
            //     let len = r.try_bytes_len().unwrap_or(0);
            //     Position::new(file, offset, len)
            // } else {
            //     let file = "".into();
            //     let offset = 0;
            //     let len = r.try_bytes_len().unwrap_or(0);
            //     Position::new(file, offset, len)
            // };
            // self.cache.entry(*c).or_insert(pos)
            self.cache
                .entry(*c)
                .or_insert((self.with_p)(Default::default(), ori))
        }
    }
    pub fn position2<HAST: HyperAST<IdN = IdN> + Copy>(&mut self, c: &IdD) -> &U
    where
        D: DecompressedWithParent<HAST, IdD>
            + DecompressedWithSiblings<HAST, IdD>
            + ShallowDecompressedTreeStore<HAST, IdD>,
    {
        if self.cache.contains_key(&c) {
            return self.cache.get(&c).unwrap();
        } else if let Some(p) = self.ds.parent(c) {
            if let Some(lsib) = self.ds.lsib(c) {
                assert_ne!(lsib.to_usize(), c.to_usize());
                let pos = self.position2(&lsib).clone();
                self.cache
                    .entry(*c)
                    .or_insert((self.with_lsib)(pos, self.ds.original(&c)))
            } else {
                assert!(
                    self.ds.position_in_parent::<usize>(c).unwrap().is_zero(),
                    "{:?}",
                    self.ds.position_in_parent::<usize>(c).unwrap().to_usize()
                );
                let ori = self.ds.original(&c);
                if self.root == ori {
                    // let r = store.resolve(&ori);
                    return self
                        .cache
                        .entry(*c)
                        .or_insert((self.with_p)(Default::default(), ori));
                }
                let pos = self.position2(&p).clone();
                self.cache.entry(*c).or_insert((self.with_p)(pos, ori))
            }
        } else {
            let ori = self.ds.original(&c);
            assert_eq!(self.root, ori);
            self.cache
                .entry(*c)
                .or_insert((self.with_p)(Default::default(), ori))
        }
    }
}

pub struct CompletePOSlice<'a, IdN, IdD, Kr: Borrow<BitSlice>> {
    pub(super) simple: LazyPOSlice<'a, IdN, IdD>,
    pub(super) kr: Kr,
}

pub struct LazyPOSlice<'a, IdN, IdD> {
    /// Ids of subtrees in HyperAST
    pub(super) id_compressed: &'a [IdN],
    /// leftmost leaf descendant of nodes
    ///
    /// it is so powerful even the basic layout should keep it
    pub(crate) llds: &'a [IdD],
    // pub(super) basic: super::basic_post_order::BasicPOSlice<'a, T, IdD>,
    #[allow(unused)] // WIP
    pub(super) id_parent: &'a [IdD],
}

impl<'a, IdN, IdD, Kr: Borrow<BitSlice>> Deref for CompletePOSlice<'a, IdN, IdD, Kr> {
    type Target = LazyPOSlice<'a, IdN, IdD>;

    fn deref(&self) -> &Self::Target {
        &self.simple
    }
}

// impl<'a, T: WithChildren, IdD: PrimInt, Kr: Borrow<BitSlice>>
//     ShallowDecompressedTreeStore<T, IdD> for CompletePOSlice<'a, T, IdD, Kr>
// where
//     T::TreeId: Debug + NodeId<IdN = T::TreeId>,
// {
//     fn len(&self) -> usize {
//         self.simple.len()
//     }

//     fn original(&self, id: &IdD) -> HAST::IdN {
//         self.simple.original(id)
//     }

//     // fn leaf_count(&self) -> IdD {
//     //     cast(self.kr.len()).unwrap()
//     // }

//     fn root(&self) -> IdD {
//         cast(self.len() - 1).unwrap()
//     }

//     fn child<'b, S>(&self, store: &'b S, x: &IdD, p: &[impl PrimInt]) -> IdD
//     where
//         S: NodeStore<T::TreeId, R<'b> = T>,
//     {
//         self.simple.child(store, x, p)
//     }

//     fn children<'b, S>(&self, store: &S, x: &IdD) -> Vec<IdD>
//     where
//         S: NodeStore<T::TreeId, R<'b> = T>,
//     {
//         self.simple.children(store, x)
//     }
// }

// impl<'a, T: WithChildren, IdD: PrimInt, Kr: Borrow<BitSlice>> DecompressedTreeStore<T, IdD>
//     for CompletePOSlice<'a, T, IdD, Kr>
// where
//     T::TreeId: Debug + NodeId<IdN = T::TreeId>,
// {
//     fn descendants<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
//     where
//         S: for<'b> NodeStore<T::TreeId, R<'b> = T>,
//     {
//         self.simple.descendants(store, x)
//     }

//     fn first_descendant(&self, i: &IdD) -> IdD {
//         self.simple.first_descendant(i)
//     }

//     fn descendants_count<S>(&self, store: &S, x: &IdD) -> usize
//     where
//         S: for<'b> NodeStore<T::TreeId, R<'b> = T>,
//     {
//         self.simple.descendants_count(store, x)
//     }

//     fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
//         self.simple.is_descendant(desc, of)
//     }
// }

// impl<'a, T: WithChildren, IdD: PrimInt, Kr: Borrow<BitSlice>> PostOrder<'a, T, IdD>
//     for CompletePOSlice<'a, T, IdD, Kr>
// where
//     T::TreeId: Debug + NodeId<IdN = T::TreeId>,
// {
//     fn lld(&self, i: &IdD) -> IdD {
//         self.simple.lld(i)
//     }

//     fn tree(&self, id: &IdD) -> HAST::IdN {
//         self.simple.tree(id)
//     }
// }

// impl<'a, T: WithChildren, IdD: PrimInt, Kr: Borrow<BitSlice>> PostOrderKeyRoots<'a, T, IdD>
//     for CompletePOSlice<'a, T, IdD, Kr>
// where
//     T::TreeId: Debug + NodeId<IdN = T::TreeId>,
// {
//     // fn kr(&self, x: IdD) -> IdD {
//     //     self.kr[x.to_usize().unwrap()]
//     // }
//     type Iter<'b> = IterKr<'b,IdD>
//     where
//         Self: 'b;

//     fn iter_kr(&self) -> Self::Iter<'_> {
//         IterKr(self.kr.borrow().iter_ones(), PhantomData)
//     }
// }
