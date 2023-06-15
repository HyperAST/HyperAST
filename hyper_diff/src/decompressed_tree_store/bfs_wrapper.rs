use std::{borrow::Borrow, fmt::Debug, marker::PhantomData};

use num_traits::{cast, zero, PrimInt};

use crate::{
    decompressed_tree_store::{
        BreadthFirstIterable, DecompressedTreeStore, DecompressedWithParent,
        PostOrder, ShallowDecompressedTreeStore,
    },
};
use hyper_ast::types::{NodeStore, WithChildren};

/// Wrap or just map a decommpressed tree in breadth-first eg. post-order,
pub struct SimpleBfsMapper<
    'a,
    T: WithChildren,
    IdD,
    DTS: DecompressedTreeStore<'a, T, IdD>,
    D: Borrow<DTS> = DTS,
> {
    map: Vec<IdD>,
    // fc: Vec<IdD>,
    rev: Vec<IdD>,
    pub back: D,
    phantom: PhantomData<&'a (T, DTS)>,
}

// TODO deref to back

impl<
        'a,
        T: WithChildren,
        IdD: Debug,
        DTS: DecompressedTreeStore<'a, T, IdD> + Debug,
        D: Borrow<DTS>,
    > Debug for SimpleBfsMapper<'a, T, IdD, DTS, D>
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

impl<'a, T: 'a + WithChildren, IdD: PrimInt, DTS: PostOrder<'a, T, IdD>, D: Borrow<DTS>>
    SimpleBfsMapper<'a, T, IdD, DTS, D>
{
    pub fn from<S>(store: &'a S, back: D) -> Self
    where
        S: NodeStore<T::TreeId, R<'a> = T>,
    {
        let x: &DTS = back.borrow();
        let mut map = Vec::with_capacity(x.len());
        let mut rev = vec![num_traits::zero(); x.len()];
        let mut i = 0;
        rev[x.root().to_usize().unwrap()] = cast(i).unwrap();
        map.push(x.root());

        while map.len() < x.len() {
            let curr = &map[i];
            let cs = x.children(store, curr);
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

impl<'a, T: WithChildren, IdD, DTS: DecompressedTreeStore<'a, T, IdD>, D: Borrow<DTS>>
    ShallowDecompressedTreeStore<'a, T, IdD> for SimpleBfsMapper<'a, T, IdD, DTS, D>
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

    fn child<'b, S>(&self, store: &'b S, x: &IdD, p: &[T::ChildIdx]) -> IdD
    where
        S: 'b + NodeStore<T::TreeId, R<'b> = T>,
    {
        let b: &DTS = self.back.borrow();
        b.child(store, x, p)
    }

    fn children<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: 'b + NodeStore<T::TreeId, R<'b> = T>,
    {
        let b: &DTS = self.back.borrow();
        b.children(store, x)
    }
}

impl<'a, T: WithChildren, IdD, DTS: DecompressedTreeStore<'a, T, IdD>, D: Borrow<DTS>>
    DecompressedTreeStore<'a, T, IdD> for SimpleBfsMapper<'a, T, IdD, DTS, D>
{
    fn descendants<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    where
        S: 'b + NodeStore<T::TreeId, R<'b> = T>,
    {
        self.back.borrow().descendants(store, x)
    }

    fn descendants_count<'b, S>(&self, store: &'b S, x: &IdD) -> usize
    where
        S: 'b + NodeStore<T::TreeId, R<'b> = T>,
        // S: 'b + NodeStore<IdC>,
        // S::R<'b>: WithChildren<TreeId = IdC>,
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
        'd,
        T: WithChildren,
        IdD: PrimInt,
        DTS: DecompressedTreeStore<'d, T, IdD> + DecompressedWithParent<'d, T, IdD>,
        D: Borrow<DTS>,
    > DecompressedWithParent<'d, T, IdD> for SimpleBfsMapper<'d, T, IdD, DTS, D>
{
    fn has_parent(&self, id: &IdD) -> bool {
        self.back.borrow().has_parent(id)
    }

    fn parent(&self, id: &IdD) -> Option<IdD> {
        self.back.borrow().parent(id)
    }

    fn position_in_parent(&self, c: &IdD) -> Option<T::ChildIdx> {
        self.back.borrow().position_in_parent(c)
    }

    type PIt<'a>=DTS::PIt<'a> where D: 'a, Self:'a;

    fn parents(&self, id: IdD) -> Self::PIt<'_> {
        self.back.borrow().parents(id)
    }

    fn path(&self, parent: &IdD, descendant: &IdD) -> Vec<T::ChildIdx> {
        self.back.borrow().path(parent, descendant)
    }

    fn lca(&self, a: &IdD, b: &IdD) -> IdD {
        self.back.borrow().lca(a, b)
    }
}

impl<
        'd,
        T: WithChildren,
        IdD: 'static + Clone,
        DTS: DecompressedTreeStore<'d, T, IdD>,
        D: Borrow<DTS>,
    > BreadthFirstIterable<'d, T, IdD> for SimpleBfsMapper<'d, T, IdD, DTS, D>
{
    type It = Iter<'d, IdD>;

    fn iter_bf(&'_ self) -> Iter<'_, IdD> {
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
