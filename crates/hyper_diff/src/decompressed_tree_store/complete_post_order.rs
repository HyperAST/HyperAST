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

use hyperast::PrimInt;
use hyperast::{
    position::Position,
    types::{
        self, HyperAST, HyperType, LabelStore, NodeId, NodeStore, Stored, Tree, WithChildren,
        WithSerialization,
    },
};

use super::{
    pre_order_wrapper::{DisplaySimplePreOrderMapper, SimplePreOrderMapper},
    simple_post_order::{SimplePOSlice, SimplePostOrder},
    CIdx, ContiguousDescendants, DecendantsLending, DecompressedParentsLending,
    DecompressedTreeStore, DecompressedWithParent, DecompressedWithSiblings,
    FullyDecompressedTreeStore, Iter, IterKr, POBorrowSlice, POSliceLending, PostOrdKeyRoots,
    PostOrder, PostOrderIterable, PostOrderKeyRoots, ShallowDecompressedTreeStore,
};

/// made for TODO
/// - post order
/// - key roots
/// - parents
pub struct CompletePostOrder<T: Stored, IdD> {
    pub(super) simple: SimplePostOrder<T, IdD>,
    /// LR_keyroots(T) = {k | there exists no k < k' such that l(k) = l(k’)}.
    pub(super) kr: bitvec::boxed::BitBox,
}

impl<T: Stored, IdD> Deref for CompletePostOrder<T, IdD> {
    type Target = SimplePostOrder<T, IdD>;

    fn deref(&self) -> &Self::Target {
        &self.simple
    }
}

impl<T: Stored, IdD> CompletePostOrder<T, IdD> {
    pub fn as_slice(&self) -> CompletePOSlice<'_, T, IdD, &'_ BitSlice> {
        CompletePOSlice {
            simple: self.simple.as_slice(),
            kr: &self.kr,
        }
    }
}

impl<T: Stored, IdD: PrimInt> CompletePostOrder<T, IdD> {
    pub fn iter(&self) -> impl Iterator<Item = &T::TreeId> {
        self.simple.iter()
    }
}

impl<T: Stored, IdD: PrimInt> From<SimplePostOrder<T, IdD>> for CompletePostOrder<T, IdD>
where
    T: for<'a> types::NLending<'a, T::TreeId>,
    for<'a> <T as types::NLending<'a, T::TreeId>>::N: WithChildren,
{
    // #[time("warn")]
    fn from(simple: SimplePostOrder<T, IdD>) -> Self {
        let kr = simple.compute_kr_bitset();
        Self { simple, kr }
    }
}

impl<T: Stored, IdD: PrimInt + Debug> Debug for CompletePostOrder<T, IdD>
where
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompletePostOrder")
            .field("simple", &self.simple)
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
    HAST: HyperAST,
    // // for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithSerialization,
    // // T::TreeId: Clone + Debug,
    // // T::Type: Copy + Debug + Send + Sync,
    // for<'t> D: DecompressedTreeStore<HAST::IdN, IdD>
    //     + PostOrder<HAST::TM, IdD>
    //     + FullyDecompressedTreeStore<HAST::IdN, IdD>,
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
    HAST: HyperAST,
    // // for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithSerialization,
    // // T::TreeId: Clone + Debug,
    // // S: NodeStore<T::TreeId, R<'store> = T>,
    // // T: Tree,
    // // T::Type: Copy + Debug + Send + Sync,
    // // LS: LabelStore<str, I = T::Label>,
    // for<'t> D: DecompressedTreeStore<HAST::IdN, IdD>
    //     + PostOrder<HAST::TM, IdD>
    //     + FullyDecompressedTreeStore<HAST::IdN, IdD>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
        // let m = SimplePreOrderMapper::from(self.inner);
        // DisplaySimplePreOrderMapper {
        //     inner: m.without_t(),
        //     stores: self.stores,
        // }
        // .fmt(f)
    }
}

// impl<'a, T: Stored, IdD: PrimInt> types::NLending<'a, T::TreeId> for CompletePostOrder<T, IdD>
// where
//     T: for<'t> types::NLending<'t, T::TreeId>,
// {
//     type N = <T as types::NLending<'a, T::TreeId>>::N;
// }

impl<'a, T: Stored, IdD: PrimInt> DecompressedParentsLending<'a, IdD>
    for CompletePostOrder<T, IdD>
{
    type PIt = <SimplePostOrder<T, IdD> as DecompressedParentsLending<'a, IdD>>::PIt;
}

impl<T: Stored, IdD: PrimInt> DecompressedWithParent<T, IdD> for CompletePostOrder<T, IdD>
where
    T: for<'a> types::NLending<'a, T::TreeId>,
    for<'a> <T as types::NLending<'a, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn parent(&self, id: &IdD) -> Option<IdD> {
        self.simple.parent(id)
    }

    fn has_parent(&self, id: &IdD) -> bool {
        self.simple.has_parent(id)
    }

    fn position_in_parent<Idx: PrimInt>(&self, c: &IdD) -> Option<Idx> {
        self.simple.position_in_parent(c)
    }

    fn parents(&self, id: IdD) -> <Self as DecompressedParentsLending<'_, IdD>>::PIt {
        self.simple.parents(id)
    }

    fn lca(&self, a: &IdD, b: &IdD) -> IdD {
        self.simple.lca(a, b)
    }

    fn path<Idx: PrimInt>(&self, parent: &IdD, descendant: &IdD) -> Vec<Idx> {
        self.simple.path(parent, descendant)
    }
}

impl<T: Stored, IdD: PrimInt> DecompressedWithSiblings<T, IdD> for CompletePostOrder<T, IdD>
where
    T: for<'a> types::NLending<'a, T::TreeId>,
    for<'a> <T as types::NLending<'a, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn lsib(&self, x: &IdD) -> Option<IdD> {
        DecompressedWithSiblings::lsib(&self.simple, x)
    }
}

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

impl<T: Stored, IdD: PrimInt> PostOrder<T, IdD> for CompletePostOrder<T, IdD>
where
    T: for<'a> types::NLending<'a, T::TreeId>,
    for<'a> <T as types::NLending<'a, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn lld(&self, i: &IdD) -> IdD {
        self.simple.lld(i)
    }

    fn tree(&self, id: &IdD) -> T::TreeId {
        self.simple.tree(id)
    }
}

impl<'a, T: Stored, IdD: PrimInt> PostOrderIterable<T, IdD> for CompletePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    type It = Iter<IdD>;
    fn iter_df_post<const ROOT: bool>(&self) -> Iter<IdD> {
        self.simple.iter_df_post::<ROOT>()
    }
}

impl<'a, T: Stored, IdD: PrimInt> PostOrdKeyRoots<'a, T, IdD> for CompletePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    type Iter = IterKr<'a, IdD>;
}

impl<T: Stored, IdD: PrimInt> PostOrderKeyRoots<T, IdD> for CompletePostOrder<T, IdD>
where
    T: for<'a> types::NLending<'a, T::TreeId>,
    for<'a> <T as types::NLending<'a, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn iter_kr(&self) -> <Self as PostOrdKeyRoots<'_, T, IdD>>::Iter {
        IterKr(self.kr.iter_ones(), PhantomData)
    }
}

impl<'a, T: Stored, IdD: PrimInt> super::DecompressedSubtree<T> for CompletePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Clone + NodeId<IdN = T::TreeId>,
    // <T as WithChildren>::ChildIdx: PrimInt,
{
    type Out = Self;

    fn decompress<S>(store: &S, root: &T::TreeId) -> Self
    where
        S: for<'t> types::NLending<'t, T::TreeId, N = <T as types::NLending<'t, T::TreeId>>::N>
            + types::NodeStore<T::TreeId>,
    {
        SimplePostOrder::decompress(store, root).into()
    }

    fn decompress2<HAST>(store: &HAST, root: &<T as Stored>::TreeId) -> Self::Out
    where
        T: for<'t> types::AstLending<'t>,
        HAST: HyperAST<IdN = <T as Stored>::TreeId, TM = T>,
    {
        SimplePostOrder::decompress2(store, root).into()
    }
}

impl<'a, T: Stored, IdD: PrimInt> ShallowDecompressedTreeStore<T, IdD> for CompletePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn len(&self) -> usize {
        self.simple.len()
    }

    fn original(&self, id: &IdD) -> T::TreeId {
        self.simple.original(id)
    }

    // fn leaf_count(&self) -> IdD {
    //     cast(self.kr.len()).unwrap()
    // }

    fn root(&self) -> IdD {
        cast(self.len() - 1).unwrap()
    }

    fn child<S>(&self, store: &S, x: &IdD, p: &[impl PrimInt]) -> IdD
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
            + NodeStore<T::TreeId>,
    {
        self.simple.child(store, x, p)
    }

    fn child4<S>(&self, store: &S, x: &IdD, p: &[impl PrimInt]) -> IdD
where
        // S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>,
    {
        self.simple.child4(store, x, p)
    }

    fn children<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
            + NodeStore<T::TreeId>,
    {
        self.simple.children(store, x)
    }
    fn children4<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
where
        // S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>,
    {
        self.simple.children4(store, x)
    }
}

impl<T: Stored, IdD: PrimInt> DecompressedTreeStore<T, IdD> for CompletePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn descendants<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = types::LendN<'b, T, T::TreeId>>
            + NodeStore<T::TreeId>,
    {
        self.simple.descendants(store, x)
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.simple.first_descendant(i)
    }

    fn descendants_count<S>(&self, store: &S, x: &IdD) -> usize
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = types::LendN<'b, T, T::TreeId>>
            + NodeStore<T::TreeId>,
    {
        self.simple.descendants_count(store, x)
    }

    fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
        self.simple.is_descendant(desc, of)
    }
}

impl<T: Stored, IdD: PrimInt> FullyDecompressedTreeStore<T, IdD> for CompletePostOrder<T, IdD>
where
    T: for<'a> types::NLending<'a, T::TreeId>,
    for<'a> <T as types::NLending<'a, T::TreeId>>::N: WithChildren,
    T::TreeId: Clone + Debug + NodeId<IdN = T::TreeId>,
{
}

impl<'a, T: Stored, IdD: PrimInt> DecendantsLending<'a> for CompletePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    type Slice = CompletePOSlice<'a, T, IdD, &'a BitSlice>;
}

impl<'a, T: Stored, IdD: PrimInt> POSliceLending<'a, T, IdD> for CompletePostOrder<T, IdD>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    type SlicePo = <Self as DecendantsLending<'a>>::Slice;
}

impl<T: Stored, IdD: PrimInt> ContiguousDescendants<T, IdD> for CompletePostOrder<T, IdD>
where
    T: for<'a> types::NLending<'a, T::TreeId>,
    for<'a> <T as types::NLending<'a, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdD> {
        self.first_descendant(x)..*x
    }

    // type Slice<'b>
    //     = CompletePOSlice<'b, T, IdD, &'b BitSlice>
    // where
    //     Self: 'b;

    fn slice(&self, x: &IdD) -> <Self as DecendantsLending<'_>>::Slice {
        let range = self.slice_range(x);
        CompletePOSlice {
            simple: self.simple.slice(x),
            kr: &self.kr[range],
        }
    }
}

impl<T: Stored, IdD: PrimInt> POBorrowSlice<T, IdD> for CompletePostOrder<T, IdD>
where
    T: for<'a> types::NLending<'a, T::TreeId>,
    for<'a> <T as types::NLending<'a, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    // type SlicePo<'b>
    //     = Self::Slice<'b>
    // where
    //     Self: 'b;

    fn slice_po(&self, x: &IdD) -> <Self as POSliceLending<'_, T, IdD>>::SlicePo {
        self.slice(x)
    }
}

pub struct RecCachedPositionProcessor<'a, T: Stored, IdD: Hash + Eq> {
    pub(crate) ds: &'a CompletePostOrder<T, IdD>,
    root: T::TreeId,
    cache: HashMap<IdD, Position>,
}

impl<'a, T: Stored, IdD: PrimInt + Hash + Eq> From<(&'a CompletePostOrder<T, IdD>, T::TreeId)>
    for RecCachedPositionProcessor<'a, T, IdD>
{
    fn from((ds, root): (&'a CompletePostOrder<T, IdD>, T::TreeId)) -> Self {
        Self {
            ds,
            root,
            cache: Default::default(),
        }
    }
}

#[cfg(feature = "cached_position_computing")]
impl<'a, T: Tree, IdD: PrimInt + Hash + Eq> RecCachedPositionProcessor<'a, T, IdD> {
    pub fn position<'b, HAST>(&mut self, stores: &'b HAST, c: &IdD) -> &Position
    where
        HAST: for<'t> HyperAST<IdN = T::TreeId, Label = T::Label>,
        // , T<'t> = T
        T::TreeId: Clone + Debug + NodeId<IdN = T::TreeId>,
        // T: WithSerialization,
    {
        if self.cache.contains_key(&c) {
            return self.cache.get(&c).unwrap();
        } else if let Some(p) = self.ds.parent(c) {
            let id = self.ds.original(&p);
            let p_r = stores.node_store().resolve(&id);
            let p_t = stores.resolve_type(&id);
            if p_t.is_directory() {
                let ori = self.ds.original(&c);
                if self.root == ori {
                    let r = stores.node_store().resolve(&ori);
                    return self.cache.entry(*c).or_insert(Position::new(
                        stores.label_store().resolve(r.get_label_unchecked()).into(),
                        0,
                        r.try_bytes_len().unwrap_or(0),
                    ));
                }
                let mut pos = self
                    .cache
                    .get(&p)
                    .cloned()
                    .unwrap_or_else(|| self.position(stores, &p).clone());
                let r = stores.node_store().resolve(&ori);
                pos.inc_path(stores.label_store().resolve(r.get_label_unchecked()));
                pos.set_len(r.try_bytes_len().unwrap_or(0));
                return self.cache.entry(*c).or_insert(pos);
            }

            if let Some(lsib) = super::DecompressedWithSiblings::lsib(&self.ds, c) {
                assert_ne!(lsib.to_usize(), c.to_usize());
                let mut pos = self
                    .cache
                    .get(&lsib)
                    .cloned()
                    .unwrap_or_else(|| self.position(stores, &lsib).clone());
                pos.inc_offset(pos.range().end - pos.range().start);
                let r = stores.node_store().resolve(&self.ds.original(&c));
                pos.set_len(r.try_bytes_len().unwrap());
                self.cache.entry(*c).or_insert(pos)
            } else {
                assert!(
                    self.ds.position_in_parent::<usize>(c).unwrap().is_zero(),
                    "{:?}",
                    self.ds.position_in_parent::<usize>(c).unwrap().to_usize()
                );
                let ori = self.ds.original(&c);
                if self.root == ori {
                    let r = stores.node_store().resolve(&ori);
                    return self.cache.entry(*c).or_insert(Position::new(
                        "".into(),
                        0,
                        r.try_bytes_len().unwrap(),
                    ));
                }
                let mut pos = self
                    .cache
                    .get(&p)
                    .cloned()
                    .unwrap_or_else(|| self.position(stores, &p).clone());
                let r = stores.node_store().resolve(&ori);
                pos.set_len(
                    r.try_bytes_len()
                        .unwrap_or_else(|| panic!("{:?}", stores.resolve_type(&ori))),
                );
                self.cache.entry(*c).or_insert(pos)
            }
        } else {
            let ori = self.ds.original(&c);
            assert_eq!(self.root, ori);
            let r = stores.node_store().resolve(&ori);
            let t = stores.resolve_type(&ori);
            let pos = if t.is_directory() || t.is_file() {
                let file = stores.label_store().resolve(r.get_label_unchecked()).into();
                let offset = 0;
                let len = r.try_bytes_len().unwrap_or(0);
                Position::new(file, offset, len)
            } else {
                let file = "".into();
                let offset = 0;
                let len = r.try_bytes_len().unwrap_or(0);
                Position::new(file, offset, len)
            };
            self.cache.entry(*c).or_insert(pos)
        }
    }
}
pub struct RecCachedProcessor<'a, T: Stored, D, IdD: Hash + Eq, U, F, G> {
    pub(crate) ds: &'a D,
    root: T::TreeId,
    cache: HashMap<IdD, U>,
    with_p: F,
    with_lsib: G,
}

impl<'a, T: Stored, D, IdD: PrimInt + Hash + Eq, U, F, G> From<(&'a D, T::TreeId, F, G)>
    for RecCachedProcessor<'a, T, D, IdD, U, F, G>
{
    fn from((ds, root, with_p, with_lsib): (&'a D, T::TreeId, F, G)) -> Self {
        Self {
            ds,
            root,
            cache: Default::default(),
            with_p,
            with_lsib,
        }
    }
}

#[cfg(feature = "cached_position_computing")]
impl<'a, T: Stored, D, IdD: PrimInt + Hash + Eq, U: Clone + Default, F, G>
    RecCachedProcessor<'a, T, D, IdD, U, F, G>
where
    // D: DecompressedTreeStore<T, IdD> + DecompressedWithSiblings<T, IdD> + PostOrder<T, IdD>,
    F: Fn(U, T::TreeId) -> U,
    G: Fn(U, T::TreeId) -> U,
{
    pub fn position<'b, S>(&mut self, store: &'b S, c: &IdD) -> &U
    where
        S: for<'t> HyperAST<IdN = T::TreeId>,
        // T<'t> = T,
        T::TreeId: Clone + Debug,
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
    pub fn position2(&mut self, c: &IdD) -> &U
    where
        T::TreeId: Clone + Debug,
        T: Stored,
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
                    self.ds.position_in_parent(c).unwrap().is_zero(),
                    "{:?}",
                    self.ds.position_in_parent(c).unwrap().to_usize()
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

pub struct CompletePOSlice<'a, T: Stored, IdD, Kr: Borrow<BitSlice>> {
    pub(super) simple: SimplePOSlice<'a, T, IdD>,
    pub(super) kr: Kr,
}

impl<'a, T: Stored, IdD, Kr: Borrow<BitSlice>> Deref for CompletePOSlice<'a, T, IdD, Kr> {
    type Target = SimplePOSlice<'a, T, IdD>;

    fn deref(&self) -> &Self::Target {
        &self.simple
    }
}

// impl<'a, 'b, T: Stored, IdD: PrimInt, Kr: Borrow<BitSlice>> types::NLending<'b, T::TreeId>
//     for CompletePOSlice<'a, T, IdD, Kr>
// where
//     T: for<'t> types::NLending<'t, T::TreeId>,
// {
//     type N = <T as types::NLending<'b, T::TreeId>>::N;
// }

impl<'a, T: Stored, IdD: PrimInt, Kr: Borrow<BitSlice>> ShallowDecompressedTreeStore<T, IdD>
    for CompletePOSlice<'a, T, IdD, Kr>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn len(&self) -> usize {
        self.simple.len()
    }

    fn original(&self, id: &IdD) -> T::TreeId {
        self.simple.original(id)
    }

    // fn leaf_count(&self) -> IdD {
    //     cast(self.kr.len()).unwrap()
    // }

    fn root(&self) -> IdD {
        cast(self.len() - 1).unwrap()
    }

    fn child<S>(&self, store: &S, x: &IdD, p: &[impl PrimInt]) -> IdD
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
            + NodeStore<T::TreeId>,
    {
        self.simple.child(store, x, p)
    }

    fn child4<S>(&self, store: &S, x: &IdD, p: &[impl PrimInt]) -> IdD
where
        // S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>,
    {
        self.simple.child4(store, x, p)
    }

    fn children<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
            + NodeStore<T::TreeId>,
    {
        self.simple.children(store, x)
    }

    fn children4<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
where
        // S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>,
    {
        self.simple.children4(store, x)
    }
}

impl<'a, T: Stored, IdD: PrimInt, Kr: Borrow<BitSlice>> DecompressedTreeStore<T, IdD>
    for CompletePOSlice<'a, T, IdD, Kr>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn descendants<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = types::LendN<'b, T, T::TreeId>>
            + NodeStore<T::TreeId>,
    {
        self.simple.descendants(store, x)
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.simple.first_descendant(i)
    }

    fn descendants_count<S>(&self, store: &S, x: &IdD) -> usize
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = types::LendN<'b, T, T::TreeId>>
            + NodeStore<T::TreeId>,
    {
        self.simple.descendants_count(store, x)
    }

    fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
        self.simple.is_descendant(desc, of)
    }
}

impl<'a, T: Stored, IdD: PrimInt, Kr: Borrow<BitSlice>> PostOrder<T, IdD>
    for CompletePOSlice<'a, T, IdD, Kr>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn lld(&self, i: &IdD) -> IdD {
        self.simple.lld(i)
    }

    fn tree(&self, id: &IdD) -> T::TreeId {
        self.simple.tree(id)
    }
}

impl<'a, 'b, T: Stored, IdD: PrimInt, Kr: Borrow<BitSlice>> PostOrdKeyRoots<'b, T, IdD>
    for CompletePOSlice<'a, T, IdD, Kr>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    // fn kr(&self, x: IdD) -> IdD {
    //     self.kr[x.to_usize().unwrap()]
    // }
    type Iter = IterKr<'b, IdD>;
}

impl<'a, T: Stored, IdD: PrimInt, Kr: Borrow<BitSlice>> PostOrderKeyRoots<T, IdD>
    for CompletePOSlice<'a, T, IdD, Kr>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
    T::TreeId: Debug + NodeId<IdN = T::TreeId>,
{
    fn iter_kr(&self) -> <Self as PostOrdKeyRoots<'_, T, IdD>>::Iter {
        IterKr(self.kr.borrow().iter_ones(), PhantomData)
    }
}

// impl<'a, IdD> super::Persistable for CompletePostOrder<HashedNodeRef<'a>, IdD> {
//     type Persisted = CompletePostOrder<
//         super::PersistedNode<<HashedNodeRef<'a> as types::Stored>::TreeId>,
//         IdD,
//     >;

//     fn persist(self) -> Self::Persisted {
//         CompletePostOrder {
//             simple: self.simple.persist(),
//             kr: self.kr,
//         }
//     }
//     unsafe fn unpersist(this: Self::Persisted) -> Self {
//         Self {
//             simple: <SimplePostOrder<hyperast::store::nodes::legion::HashedNodeRef<'a>,IdD> as super::Persistable>::unpersist(this.simple),
//             kr: this.kr,
//         }
//     }
// }
