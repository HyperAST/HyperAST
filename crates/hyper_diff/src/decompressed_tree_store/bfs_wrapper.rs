use std::{borrow::Borrow, fmt::Debug, marker::PhantomData};

use num_traits::{cast, zero};

use crate::decompressed_tree_store::{
    BreadthFirstIterable, CIdx, DecompressedParentsLending, DecompressedTreeStore,
    DecompressedWithParent, PostOrder, ShallowDecompressedTreeStore,
};
use hyperast::types::{self, NodeStore, Stored, WithChildren};
use hyperast::PrimInt;

use super::BreadthFirstIt;

/// Wrap or just map a decommpressed tree in breadth-first eg. post-order,
pub struct SimpleBfsMapper<
    'a,
    T: Stored,
    IdD,
    DTS, //: DecompressedTreeStore<T, IdD>,
    D: Borrow<DTS> = DTS,
> {
    map: Vec<IdD>,
    // fc: Vec<IdD>,
    rev: Vec<IdD>,
    pub back: D,
    phantom: PhantomData<&'a (T, DTS)>,
}

// TODO deref to back

impl<'a, T: Stored, IdD: Debug, DTS: Debug, D: Borrow<DTS>> Debug
    for SimpleBfsMapper<'a, T, IdD, DTS, D>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SD")
            .field("map", &self.map)
            .field("rev", &self.rev)
            .field("back", &self.back.borrow())
            .field("phantom", &self.phantom)
            .finish()
    }
}

impl<'a, T: Stored, IdD: PrimInt, DTS: PostOrder<T, IdD>, D: Borrow<DTS>>
    SimpleBfsMapper<'a, T, IdD, DTS, D>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
{
    pub fn from_node_store<S>(store: &'a S, back: D) -> Self
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
            + NodeStore<T::TreeId>,
    {
        let x: &DTS = back.borrow();
        let mut map = Vec::with_capacity(x.len());
        let mut rev = vec![num_traits::zero(); x.len()];
        let mut i = 0;
        rev[x.root().to_usize().unwrap()] = cast(i).unwrap();
        map.push(x.root());

        while map.len() < x.len() {
            let curr = &map[i];
            let cs = x.children4(store, curr);
            rev[(*curr).to_usize().unwrap()] = cast(i).unwrap();
            map.extend(cs);
            i += 1;
        }

        map.shrink_to_fit();
        Self {
            map,
            // fc,
            rev,
            back,
            phantom: PhantomData,
        }
    }
}

impl<'a, HAST: types::HyperAST, IdD: PrimInt, DTS: PostOrder<HAST::TM, IdD>, D: Borrow<DTS>> From<(&'a HAST, D)>
    for SimpleBfsMapper<'a, HAST::TM, IdD, DTS, D>
{
    fn from((store, back): (&'a HAST, D)) -> Self {
        let x: &DTS = back.borrow();
        let mut map = Vec::with_capacity(x.len());
        let mut rev = vec![num_traits::zero(); x.len()];
        let mut i = 0;
        rev[x.root().to_usize().unwrap()] = cast(i).unwrap();
        map.push(x.root());

        while map.len() < x.len() {
            let curr = &map[i];
            let cs = x.children4(store, curr);
            rev[(*curr).to_usize().unwrap()] = cast(i).unwrap();
            map.extend(cs);
            i += 1;
        }

        map.shrink_to_fit();
        Self {
            map,
            // fc,
            rev,
            back,
            phantom: PhantomData,
        }
    }
}

// impl<'a, T: WithChildren, IdD, DTS: DecompressedTreeStore<'a, T, IdD>, D: Borrow<DTS>>
//     Initializable<'a, T> for SimpleBfsMapper<'a, T, IdD, DTS, D>
// {
//     fn make<S>(_store: &'a S, _root: &T::TreeId) -> Self
//     where
//         S: NodeStore<T::TreeId, R<'a> = T>,
//     {
//         panic!()
//     }
// }

impl<'a, 'b, T: Stored, IdD, DTS: DecompressedTreeStore<T, IdD>, D: Borrow<DTS>>
    types::NLending<'b, T::TreeId> for SimpleBfsMapper<'a, T, IdD, DTS, D>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
{
    type N = <T as types::NLending<'b, T::TreeId>>::N;
}

impl<'a, T: Stored, IdD, DTS: DecompressedTreeStore<T, IdD>, D: Borrow<DTS>>
    ShallowDecompressedTreeStore<T, IdD> for SimpleBfsMapper<'a, T, IdD, DTS, D>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
{
    fn len(&self) -> usize {
        self.map.len()
    }

    fn original(&self, id: &IdD) -> T::TreeId {
        self.back.borrow().original(id)
    }

    fn root(&self) -> IdD {
        self.back.borrow().root()
    }

    fn child<S>(&self, store: &S, x: &IdD, p: &[impl PrimInt]) -> IdD
    where
        S: NodeStore<T::TreeId, NMarker = T>,
    {
        let b: &DTS = self.back.borrow();
        b.child(store, x, p)
    }

    fn child4<S>(&self, store: &S, x: &IdD, p: &[impl PrimInt]) -> IdD
where
        // S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>,
    {
        let b: &DTS = self.back.borrow();
        b.child4(store, x, p)
    }

    fn children<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = <T as types::NLending<'b, T::TreeId>>::N>
            + NodeStore<T::TreeId>,
    {
        let b: &DTS = self.back.borrow();
        b.children(store, x)
    }
    fn children4<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
where
        // S: hyperast::types::inner_ref::NodeStore<T::TreeId, Ref = T>,
    {
        let b: &DTS = self.back.borrow();
        b.children4(store, x)
    }
}

impl<'a, T: Stored, IdD, DTS: DecompressedTreeStore<T, IdD>, D: Borrow<DTS>>
    DecompressedTreeStore<T, IdD> for SimpleBfsMapper<'a, T, IdD, DTS, D>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
{
    fn descendants<S>(&self, store: &S, x: &IdD) -> Vec<IdD>
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = types::LendN<'b, T, T::TreeId>>
            + NodeStore<T::TreeId>,
    {
        self.back.borrow().descendants(store, x)
    }

    fn descendants_count<S>(&self, store: &S, x: &IdD) -> usize
    where
        S: for<'b> types::NLending<'b, T::TreeId, N = types::LendN<'b, T, T::TreeId>>
            + NodeStore<T::TreeId>,
    {
        self.back.borrow().descendants_count(store, x)
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.back.borrow().first_descendant(i)
    }

    fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
        self.back.borrow().is_descendant(desc, of)
    }
}

impl<
        'a,
        'd,
        T: Stored,
        IdD: PrimInt,
        DTS: DecompressedTreeStore<T, IdD> + DecompressedWithParent<T, IdD>,
        D: Borrow<DTS>,
    > DecompressedParentsLending<'a, IdD> for SimpleBfsMapper<'d, T, IdD, DTS, D>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
{
    type PIt = <DTS as DecompressedParentsLending<'a, IdD>>::PIt;
}

impl<
        'd,
        T: Stored,
        IdD: PrimInt,
        DTS: DecompressedTreeStore<T, IdD> + DecompressedWithParent<T, IdD>,
        D: Borrow<DTS>,
    > DecompressedWithParent<T, IdD> for SimpleBfsMapper<'d, T, IdD, DTS, D>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
{
    fn has_parent(&self, id: &IdD) -> bool {
        self.back.borrow().has_parent(id)
    }

    fn parent(&self, id: &IdD) -> Option<IdD> {
        self.back.borrow().parent(id)
    }

    fn position_in_parent<Idx: PrimInt>(&self, c: &IdD) -> Option<Idx> {
        self.back.borrow().position_in_parent(c)
    }

    fn parents(&self, id: IdD) -> <Self as DecompressedParentsLending<'_, IdD>>::PIt {
        self.back.borrow().parents(id)
    }

    fn path<Idx: PrimInt>(&self, parent: &IdD, descendant: &IdD) -> Vec<Idx> {
        self.back.borrow().path(parent, descendant)
    }

    fn lca(&self, a: &IdD, b: &IdD) -> IdD {
        self.back.borrow().lca(a, b)
    }
}

impl<'d, T: Stored, IdD: 'static + Clone, DTS: DecompressedTreeStore<T, IdD>, D: Borrow<DTS>>
    BreadthFirstIt<T, IdD> for SimpleBfsMapper<'d, T, IdD, DTS, D>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
{
    type It<'b> = Iter<'b, IdD>;
}

impl<'d, T: Stored, IdD: 'static + Clone, DTS: DecompressedTreeStore<T, IdD>, D: Borrow<DTS>>
    BreadthFirstIterable<T, IdD> for SimpleBfsMapper<'d, T, IdD, DTS, D>
where
    T: for<'t> types::NLending<'t, T::TreeId>,
    for<'t> <T as types::NLending<'t, T::TreeId>>::N: WithChildren,
{
    fn iter_bf(&self) -> Iter<'_, IdD> {
        Iter {
            curr: zero(),
            len: self.map.len(),
            map: &self.map,
        }
    }
}

pub struct Iter<'a, IdD> {
    curr: usize,
    len: usize,
    map: &'a [IdD],
}

impl<'a, IdD: Clone> Iterator for Iter<'a, IdD> {
    type Item = IdD;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr == self.len {
            None
        } else {
            let r = self.curr;
            self.curr = r + 1;
            Some(self.map[r].clone())
        }
    }
}
