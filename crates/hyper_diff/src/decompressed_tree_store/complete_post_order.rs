use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
    ops::Deref,
};

use bitvec::slice::BitSlice;
use num_traits::cast;

use super::{
    ContiguousDescendants, DecendantsLending, DecompressedParentsLending, DecompressedTreeStore,
    DecompressedWithParent, DecompressedWithSiblings, FullyDecompressedTreeStore, Iter, IterKr,
    POBorrowSlice, POSliceLending, PostOrdKeyRoots, PostOrder, PostOrderIterable,
    PostOrderKeyRoots, ShallowDecompressedTreeStore,
    simple_post_order::{SimplePOSlice, SimplePostOrder},
};
use crate::matchers::Decompressible;
use hyperast::PrimInt;
use hyperast::types::{self, HyperAST};

/// Decompressed tree with a post-order layout
/// provides:
/// - origines (through [`SimplePostOrder`])
/// - llds (through [`SimplePostOrder`])
/// - parents (through [`SimplePostOrder`])
/// - key roots
#[derive(Clone)]
pub struct CompletePostOrder<IdN, IdD> {
    pub(super) simple: SimplePostOrder<IdN, IdD>,
    /// LR_keyroots(T) = {k | there exists no k < k' such that l(k) = l(k’)}.
    pub(super) kr: bitvec::boxed::BitBox,
}

impl<IdN, IdD> Deref for CompletePostOrder<IdN, IdD> {
    type Target = SimplePostOrder<IdN, IdD>;

    fn deref(&self) -> &Self::Target {
        &self.simple
    }
}

impl<HAST: HyperAST + Copy, IdD> CompletePostOrder<HAST, IdD> {
    pub fn as_slice(&self) -> CompletePOSlice<'_, HAST, IdD, &'_ BitSlice> {
        CompletePOSlice {
            simple: self.simple.as_slice(),
            kr: &self.kr,
        }
    }
}

impl<HAST: HyperAST + Copy, IdD> Decompressible<HAST, CompletePostOrder<HAST::IdN, IdD>> {
    fn as_simple(&self) -> Decompressible<HAST, &SimplePostOrder<HAST::IdN, IdD>> {
        let hyperast = self.hyperast;
        let decomp = &self.simple;
        Decompressible { hyperast, decomp }
    }
}

impl<IdN, IdD: PrimInt> CompletePostOrder<IdN, IdD> {
    pub fn iter(&self) -> impl Iterator<Item = &IdN> {
        self.simple.iter()
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt>
    From<Decompressible<HAST, SimplePostOrder<HAST::IdN, IdD>>>
    for Decompressible<HAST, CompletePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    // #[time("warn")]
    fn from(simple: Decompressible<HAST, SimplePostOrder<HAST::IdN, IdD>>) -> Self {
        let hyperast = simple.hyperast;
        let kr = Decompressible {
            hyperast,
            decomp: &simple.basic,
        }
        .as_slice()
        .compute_kr_bitset();
        let simple = simple.decomp;
        Decompressible {
            hyperast,
            decomp: CompletePostOrder { simple, kr },
        }
    }
}

impl<IdN: Debug, IdD: PrimInt + Debug> Debug for CompletePostOrder<IdN, IdD> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompletePostOrder")
            .field("simple", &self.simple)
            .field("kr", &self.kr)
            .finish()
    }
}

pub struct DisplayCompletePostOrder<'a, IdD: PrimInt, HAST, D>
where
    HAST: HyperAST,
{
    inner: &'a D,
    stores: HAST,
    _phantom: PhantomData<&'a IdD>,
}

impl<'a, IdD: PrimInt, HAST, D> DisplayCompletePostOrder<'a, IdD, HAST, D>
where
    HAST: HyperAST,
{
    pub fn new(stores: HAST, inner: &'a D) -> Self {
        Self {
            inner,
            stores,
            _phantom: PhantomData,
        }
    }
}

impl<'a, IdD: PrimInt, HAST, D> Display for DisplayCompletePostOrder<'a, IdD, HAST, D>
where
    HAST: HyperAST + Copy,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithSerialization,
    for<'t> D: DecompressedTreeStore<HAST, IdD>
        + PostOrder<HAST, IdD>
        + FullyDecompressedTreeStore<HAST, IdD>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let m = super::pre_order_wrapper::SimplePreOrderMapper::from(self.inner);
        let m = super::pre_order_wrapper::DisplaySimplePreOrderMapper {
            inner: &m,
            stores: &self.stores,
        };
        std::fmt::Display::fmt(&m, f)
    }
}

impl<'a, IdD: PrimInt, HAST, D> Debug for DisplayCompletePostOrder<'a, IdD, HAST, D>
where
    HAST: HyperAST + Copy,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithSerialization,
    D: DecompressedTreeStore<HAST, IdD>
        + PostOrder<HAST, IdD>
        + FullyDecompressedTreeStore<HAST, IdD>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let m = super::pre_order_wrapper::SimplePreOrderMapper::from(self.inner);
        Debug::fmt(
            &super::pre_order_wrapper::DisplaySimplePreOrderMapper {
                inner: &m,
                stores: &self.stores,
            },
            f,
        )
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> DecompressedParentsLending<'a, IdD>
    for Decompressible<HAST, CompletePostOrder<HAST::IdN, IdD>>
{
    type PIt =
        <Decompressible<HAST, SimplePostOrder<HAST::IdN, IdD>> as DecompressedParentsLending<
            'a,
            IdD,
        >>::PIt;
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> DecompressedWithParent<HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn parent(&self, id: &IdD) -> Option<IdD> {
        self.as_simple().parent(id)
    }

    fn has_parent(&self, id: &IdD) -> bool {
        self.as_simple().has_parent(id)
    }

    fn position_in_parent<Idx: PrimInt>(&self, c: &IdD) -> Option<Idx> {
        self.as_simple().position_in_parent(c)
    }

    fn parents(&self, id: IdD) -> <Self as DecompressedParentsLending<'_, IdD>>::PIt {
        // self.as_simple().parents(id)
        super::simple_post_order::IterParents {
            id,
            id_parent: &self.simple.id_parent,
        }
    }

    fn lca(&self, a: &IdD, b: &IdD) -> IdD {
        self.as_simple().lca(a, b)
    }

    fn path<Idx: PrimInt>(&self, parent: &IdD, descendant: &IdD) -> Vec<Idx> {
        self.as_simple().path(parent, descendant)
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> DecompressedWithSiblings<HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn lsib(&self, x: &IdD) -> Option<IdD> {
        DecompressedWithSiblings::lsib(&self.as_simple(), x)
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

impl<HAST: HyperAST + Copy, IdD: PrimInt> PostOrder<HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn lld(&self, i: &IdD) -> IdD {
        self.as_simple().lld(i)
    }

    fn tree(&self, id: &IdD) -> HAST::IdN {
        self.as_simple().tree(id)
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> PostOrderIterable<HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type It = Iter<IdD>;
    fn iter_df_post<const ROOT: bool>(&self) -> Iter<IdD> {
        self.as_simple().as_basic().iter_df_post::<ROOT>()
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> PostOrdKeyRoots<'a, HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type Iter = IterKr<'a, IdD>;
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> PostOrderKeyRoots<HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn iter_kr(&self) -> <Self as PostOrdKeyRoots<'_, HAST, IdD>>::Iter {
        IterKr(self.kr.iter_ones(), PhantomData)
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt + Debug> types::DecompressedFrom<HAST>
    for CompletePostOrder<HAST::IdN, IdD>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type Out = Self;

    // #[time("warn")]
    fn decompress(hyperast: HAST, root: &HAST::IdN) -> Self {
        let decomp = <SimplePostOrder<HAST::IdN, IdD> as types::DecompressedFrom<HAST>>::decompress(
            hyperast, root,
        );
        let r: Decompressible<_, Self> = Decompressible { hyperast, decomp }.into();
        r.decomp
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> super::DecompressedSubtree<HAST::IdN>
    for Decompressible<HAST, CompletePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type Out = Self;

    fn decompress(self, root: &HAST::IdN) -> Self
where {
        let hyperast = self.hyperast;
        let basic = self.decomp.simple.basic;
        let id_parent = self.decomp.simple.id_parent;
        let basic = Decompressible {
            hyperast,
            decomp: basic,
        };
        let basic = basic.decompress(root);
        let basic = basic.decomp;
        let decomp = SimplePostOrder { basic, id_parent };
        // let simple = SimplePostOrder::make(store, root);
        Decompressible { hyperast, decomp }.into()
        // SimplePostOrder::decompress(self.hyperast, root).into()
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> ShallowDecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn len(&self) -> usize {
        self.as_simple().len()
    }

    fn original(&self, id: &IdD) -> HAST::IdN {
        self.as_simple().original(id)
    }

    fn root(&self) -> IdD {
        cast(self.len() - 1).unwrap()
    }

    fn child(&self, x: &IdD, p: &[impl PrimInt]) -> IdD {
        self.as_simple().child(x, p)
    }

    fn children(&self, x: &IdD) -> Vec<IdD> {
        self.as_simple().children(x)
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> DecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn descendants(&self, x: &IdD) -> Vec<IdD> {
        self.as_simple().descendants(x)
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.as_simple().first_descendant(i)
    }

    fn descendants_count(&self, x: &IdD) -> usize {
        self.as_simple().descendants_count(x)
    }

    fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
        self.as_simple().is_descendant(desc, of)
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> FullyDecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> DecendantsLending<'a>
    for Decompressible<HAST, CompletePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type Slice = CompletePOSlice<'a, HAST::IdN, IdD, &'a BitSlice>;
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt> POSliceLending<'a, HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type SlicePo = Decompressible<HAST, <Self as DecendantsLending<'a>>::Slice>;
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> ContiguousDescendants<HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn descendants_range(&self, x: &IdD) -> std::ops::Range<IdD> {
        self.first_descendant(x)..*x
    }

    fn slice(&self, x: &IdD) -> <Self as DecendantsLending<'_>>::Slice {
        let range = self.as_simple().as_basic().slice_range(x);
        let hyperast = self.hyperast;
        let decomp = &self.simple;
        let simple = Decompressible { hyperast, decomp };
        let decomp = CompletePOSlice {
            simple: simple._slice(x),
            kr: &self.kr[range],
        };
        decomp
    }
}

impl<HAST: HyperAST + Copy, IdD: PrimInt> POBorrowSlice<HAST, IdD>
    for Decompressible<HAST, CompletePostOrder<HAST::IdN, IdD>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn slice_po(&self, x: &IdD) -> <Self as POSliceLending<'_, HAST, IdD>>::SlicePo {
        let hyperast = self.hyperast;
        let decomp = self.slice(x);
        Decompressible { hyperast, decomp }
    }
}

// pub struct RecCachedPositionProcessor<'a, HAST: HyperASTShared + Copy, IdD: Hash + Eq> {
//     pub(crate) ds: Decompressible<HAST, &'a CompletePostOrder<HAST::IdN, IdD>>,
//     root: HAST::IdN,
//     cache: HashMap<IdD, Position>,
// }

// impl<'a, HAST: HyperAST + Copy, IdD: PrimInt + Hash + Eq>
//     From<(
//         Decompressible<HAST, &'a CompletePostOrder<HAST::IdN, IdD>>,
//         HAST::IdN,
//     )> for RecCachedPositionProcessor<'a, HAST, IdD>
// {
//     fn from(
//         (ds, root): (
//             Decompressible<HAST, &'a CompletePostOrder<HAST::IdN, IdD>>,
//             HAST::IdN,
//         ),
//     ) -> Self {
//         Self {
//             ds,
//             root,
//             cache: Default::default(),
//         }
//     }
// }

// // impl<'a, T: HyperAST + Copy, IdD: PrimInt + Hash + Eq> RecCachedPositionProcessor<'a, T, IdD> {
// //     pub fn position<'b, HAST>(&mut self, stores: &'b HAST, c: &IdD) -> &Position
// //     where
// //         HAST: for<'t> HyperAST<IdN = T::IdN, Label = T::Label>,
// //         // , T<'t> = T
// //         HAST::IdN: Clone + Debug + NodeId<IdN = HAST::IdN>,
// //         // T: WithSerialization,
// //     {
// //         if self.cache.contains_key(&c) {
// //             return self.cache.get(&c).unwrap();
// //         } else if let Some(p) = self.ds.parent(c) {
// //             let id = self.ds.original(&p);
// //             let p_r = stores.node_store().resolve(&id);
// //             let p_t = stores.resolve_type(&id);
// //             if p_t.is_directory() {
// //                 let ori = self.ds.original(&c);
// //                 if self.root == ori {
// //                     let r = stores.node_store().resolve(&ori);
// //                     return self.cache.entry(*c).or_insert(Position::new(
// //                         stores.label_store().resolve(r.get_label_unchecked()).into(),
// //                         0,
// //                         r.try_bytes_len().unwrap_or(0),
// //                     ));
// //                 }
// //                 let mut pos = self
// //                     .cache
// //                     .get(&p)
// //                     .cloned()
// //                     .unwrap_or_else(|| self.position(stores, &p).clone());
// //                 let r = stores.node_store().resolve(&ori);
// //                 pos.inc_path(stores.label_store().resolve(r.get_label_unchecked()));
// //                 pos.set_len(r.try_bytes_len().unwrap_or(0));
// //                 return self.cache.entry(*c).or_insert(pos);
// //             }

// //             if let Some(lsib) = super::DecompressedWithSiblings::lsib(&self.ds, c) {
// //                 assert_ne!(lsib.to_usize(), c.to_usize());
// //                 let mut pos = self
// //                     .cache
// //                     .get(&lsib)
// //                     .cloned()
// //                     .unwrap_or_else(|| self.position(stores, &lsib).clone());
// //                 pos.inc_offset(pos.range().end - pos.range().start);
// //                 let r = stores.node_store().resolve(&self.ds.original(&c));
// //                 pos.set_len(r.try_bytes_len().unwrap());
// //                 self.cache.entry(*c).or_insert(pos)
// //             } else {
// //                 assert!(
// //                     self.ds.position_in_parent::<usize>(c).unwrap().is_zero(),
// //                     "{:?}",
// //                     self.ds.position_in_parent::<usize>(c).unwrap().to_usize()
// //                 );
// //                 let ori = self.ds.original(&c);
// //                 if self.root == ori {
// //                     let r = stores.node_store().resolve(&ori);
// //                     return self.cache.entry(*c).or_insert(Position::new(
// //                         "".into(),
// //                         0,
// //                         r.try_bytes_len().unwrap(),
// //                     ));
// //                 }
// //                 let mut pos = self
// //                     .cache
// //                     .get(&p)
// //                     .cloned()
// //                     .unwrap_or_else(|| self.position(stores, &p).clone());
// //                 let r = stores.node_store().resolve(&ori);
// //                 pos.set_len(
// //                     r.try_bytes_len()
// //                         .unwrap_or_else(|| panic!("{:?}", stores.resolve_type(&ori))),
// //                 );
// //                 self.cache.entry(*c).or_insert(pos)
// //             }
// //         } else {
// //             let ori = self.ds.original(&c);
// //             assert_eq!(self.root, ori);
// //             let r = stores.node_store().resolve(&ori);
// //             let t = stores.resolve_type(&ori);
// //             let pos = if t.is_directory() || t.is_file() {
// //                 let file = stores.label_store().resolve(r.get_label_unchecked()).into();
// //                 let offset = 0;
// //                 let len = r.try_bytes_len().unwrap_or(0);
// //                 Position::new(file, offset, len)
// //             } else {
// //                 let file = "".into();
// //                 let offset = 0;
// //                 let len = r.try_bytes_len().unwrap_or(0);
// //                 Position::new(file, offset, len)
// //             };
// //             self.cache.entry(*c).or_insert(pos)
// //         }
// //     }
// // }

#[allow(unused)]
pub struct RecCachedProcessor<'a, IdN, D, IdD: Hash + Eq, U, F, G> {
    pub(crate) ds: &'a D,
    root: IdN,
    cache: HashMap<IdD, U>,
    with_p: F,
    with_lsib: G,
}

#[allow(unused)]
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

#[allow(unused)]
impl<'a, IdN, D, IdD: PrimInt + Hash + Eq, U: Clone + Default, F, G>
    RecCachedProcessor<'a, IdN, D, IdD, U, F, G>
where
    F: Fn(U, IdN) -> U,
    G: Fn(U, IdN) -> U,
{
    pub fn position<'b, S>(&mut self, store: S, c: &IdD) -> &U
    where
        D: DecompressedTreeStore<S, IdD> + DecompressedWithSiblings<S, IdD> + PostOrder<S, IdD>,
        S: for<'t> HyperAST<IdN = IdN> + Copy,
        IdN: Clone + Debug,
        // T: Tree + WithSerialization,
    {
        todo!()
        // if self.cache.contains_key(&c) {
        //     return self.cache.get(&c).unwrap();
        // } else if let Some(p) = self.ds.parent(c) {
        //     let id = self.ds.original(&p);
        //     let p_r = store.node_store().resolve(&id);
        //     let p_t = store.resolve_type(&id);
        //     if p_t.is_directory() {
        //         let ori = self.ds.original(&c);
        //         if self.root == ori {
        //             // let r = store.resolve(&ori);
        //             return self
        //                 .cache
        //                 .entry(*c)
        //                 .or_insert((self.with_p)(Default::default(), ori));
        //             // Position::new(
        //             //     lstore.resolve(&r.get_label()).into(),
        //             //     0,
        //             //     r.try_bytes_len().unwrap_or(0),
        //             // )
        //         }
        //         let pos = self.position(store, &p).clone();
        //         // let r = store.resolve(&ori);
        //         // pos.inc_path(lstore.resolve(&r.get_label()));
        //         // pos.set_len(r.try_bytes_len().unwrap_or(0));
        //         // return self.cache.entry(*c).or_insert(pos);
        //         return self.cache.entry(*c).or_insert((self.with_p)(pos, ori));
        //     }

        //     if let Some(lsib) = self.ds.lsib(c) {
        //         assert_ne!(lsib.to_usize(), c.to_usize());
        //         let pos = self.position(store, &lsib).clone();
        //         // pos.inc_offset(pos.range().end - pos.range().start);
        //         // let r = store.resolve(&self.ds.original(&c));
        //         // pos.set_len(r.try_bytes_len().unwrap());
        //         // self.cache.entry(*c).or_insert(pos)
        //         self.cache
        //             .entry(*c)
        //             .or_insert((self.with_lsib)(pos, self.ds.original(&c)))
        //     } else {
        //         assert!(
        //             self.ds.position_in_parent::<usize>(c).unwrap().is_zero(),
        //             "{:?}",
        //             self.ds.position_in_parent::<usize>(c).unwrap().to_usize()
        //         );
        //         let ori = self.ds.original(&c);
        //         if self.root == ori {
        //             // let r = store.resolve(&ori);
        //             return self
        //                 .cache
        //                 .entry(*c)
        //                 .or_insert((self.with_p)(Default::default(), ori));
        //             // Position::new(
        //             //     "".into(),
        //             //     0,
        //             //     r.try_bytes_len().unwrap(),
        //             // )
        //         }
        //         let pos = self.position(store, &p).clone();
        //         // let r = store.resolve(&ori);
        //         // pos.set_len(
        //         //     r.try_bytes_len()
        //         //         .unwrap_or_else(|| panic!("{:?}", r.get_type())),
        //         // );
        //         // self.cache.entry(*c).or_insert(pos)
        //         self.cache.entry(*c).or_insert((self.with_p)(pos, ori))
        //     }
        // } else {
        //     let ori = self.ds.original(&c);
        //     assert_eq!(self.root, ori);
        //     // let r = store.resolve(&ori);
        //     // let t = r.get_type();
        //     // let pos = if t.is_directory() || t.is_file() {
        //     //     let file = lstore.resolve(&r.get_label()).into();
        //     //     let offset = 0;
        //     //     let len = r.try_bytes_len().unwrap_or(0);
        //     //     Position::new(file, offset, len)
        //     // } else {
        //     //     let file = "".into();
        //     //     let offset = 0;
        //     //     let len = r.try_bytes_len().unwrap_or(0);
        //     //     Position::new(file, offset, len)
        //     // };
        //     // self.cache.entry(*c).or_insert(pos)
        //     self.cache
        //         .entry(*c)
        //         .or_insert((self.with_p)(Default::default(), ori))
        // }
    }
    pub fn position2(&mut self, c: &IdD) -> &U
where
        // T::TreeId: Clone + Debug,
        // T: Stored,
    {
        todo!()
        // if self.cache.contains_key(&c) {
        //     return self.cache.get(&c).unwrap();
        // } else if let Some(p) = self.ds.parent(c) {
        //     if let Some(lsib) = self.ds.lsib(c) {
        //         assert_ne!(lsib.to_usize(), c.to_usize());
        //         let pos = self.position2(&lsib).clone();
        //         self.cache
        //             .entry(*c)
        //             .or_insert((self.with_lsib)(pos, self.ds.original(&c)))
        //     } else {
        //         assert!(
        //             self.ds.position_in_parent(c).unwrap().is_zero(),
        //             "{:?}",
        //             self.ds.position_in_parent(c).unwrap().to_usize()
        //         );
        //         let ori = self.ds.original(&c);
        //         if self.root == ori {
        //             // let r = store.resolve(&ori);
        //             return self
        //                 .cache
        //                 .entry(*c)
        //                 .or_insert((self.with_p)(Default::default(), ori));
        //         }
        //         let pos = self.position2(&p).clone();
        //         self.cache.entry(*c).or_insert((self.with_p)(pos, ori))
        //     }
        // } else {
        //     let ori = self.ds.original(&c);
        //     assert_eq!(self.root, ori);
        //     self.cache
        //         .entry(*c)
        //         .or_insert((self.with_p)(Default::default(), ori))
        // }
    }
}

pub struct CompletePOSlice<'a, IdN, IdD, Kr: Borrow<BitSlice>> {
    pub(super) simple: SimplePOSlice<'a, IdN, IdD>,
    pub(super) kr: Kr,
}

impl<'a, IdN, IdD, Kr: Borrow<BitSlice>> Deref for CompletePOSlice<'a, IdN, IdD, Kr> {
    type Target = SimplePOSlice<'a, IdN, IdD>;

    fn deref(&self) -> &Self::Target {
        &self.simple
    }
}

impl<'a, HAST: HyperAST + Copy, IdD, Kr: Borrow<BitSlice>>
    Decompressible<HAST, CompletePOSlice<'a, HAST::IdN, IdD, Kr>>
{
    pub(crate) fn as_simple(&self) -> Decompressible<HAST, SimplePOSlice<'a, HAST::IdN, IdD>> {
        let hyperast = self.hyperast;
        let decomp = self.decomp.simple;
        Decompressible { hyperast, decomp }
    }
}

impl<'a, HAST: HyperAST + Copy, IdD, Kr: Borrow<BitSlice>>
    Decompressible<HAST, CompletePOSlice<'a, HAST::IdN, IdD, Kr>>
{
    pub(crate) fn as_basic(
        &self,
    ) -> Decompressible<HAST, super::basic_post_order::BasicPOSlice<'a, HAST::IdN, IdD>> {
        self.as_simple().as_basic()
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt, Kr: Borrow<BitSlice>>
    ShallowDecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, CompletePOSlice<'a, HAST::IdN, IdD, Kr>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn len(&self) -> usize {
        self.as_basic().len()
    }

    fn original(&self, id: &IdD) -> HAST::IdN {
        self.as_basic().original(id)
    }

    fn root(&self) -> IdD {
        cast(self.len() - 1).unwrap()
    }

    fn child(&self, x: &IdD, p: &[impl PrimInt]) -> IdD {
        self.as_basic().child(x, p)
    }

    fn children(&self, x: &IdD) -> Vec<IdD> {
        self.as_basic().children(x)
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt, Kr: Borrow<BitSlice>> DecompressedTreeStore<HAST, IdD>
    for Decompressible<HAST, CompletePOSlice<'a, HAST::IdN, IdD, Kr>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn descendants(&self, x: &IdD) -> Vec<IdD> {
        self.as_basic().descendants(x)
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.as_basic().first_descendant(i)
    }

    fn descendants_count(&self, x: &IdD) -> usize {
        self.as_basic().descendants_count(x)
    }

    fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
        self.as_basic().is_descendant(desc, of)
    }
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt, Kr: Borrow<BitSlice>> PostOrder<HAST, IdD>
    for Decompressible<HAST, CompletePOSlice<'a, HAST::IdN, IdD, Kr>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn lld(&self, i: &IdD) -> IdD {
        self.as_basic().lld(i)
    }

    fn tree(&self, id: &IdD) -> HAST::IdN {
        self.as_basic().tree(id)
    }
}

impl<'a, 'b, HAST: HyperAST + Copy, IdD: PrimInt, Kr: Borrow<BitSlice>>
    PostOrdKeyRoots<'b, HAST, IdD> for Decompressible<HAST, CompletePOSlice<'a, HAST::IdN, IdD, Kr>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    type Iter = IterKr<'b, IdD>;
}

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt, Kr: Borrow<BitSlice>> PostOrderKeyRoots<HAST, IdD>
    for Decompressible<HAST, CompletePOSlice<'a, HAST::IdN, IdD, Kr>>
where
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
{
    fn iter_kr(&self) -> <Self as PostOrdKeyRoots<'_, HAST, IdD>>::Iter {
        IterKr(self.kr.borrow().iter_ones(), PhantomData)
    }
}
