use std::{borrow::Borrow, fmt::Debug, marker::PhantomData};

use num_traits::{cast, zero};

use crate::decompressed_tree_store::{
    BreadthFirstIterable, DecompressedParentsLending, DecompressedTreeStore,
    DecompressedWithParent, PostOrder, ShallowDecompressedTreeStore,
};
use hyperast::types::HyperAST;
use hyperast::PrimInt;

use super::BreadthFirstIt;

/// Wrap or just map a decompressed tree in breadth-first eg. post-order,
pub struct SimpleBfsMapper<'a, IdD, DTS, D: Borrow<DTS> = DTS> {
    map: Vec<IdD>,
    rev: Vec<IdD>,
    pub back: D,
    phantom: PhantomData<&'a DTS>,
}

impl<'a, IdD: Debug, DTS: Debug, D: Borrow<DTS>> Debug for SimpleBfsMapper<'a, IdD, DTS, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SD")
            .field("map", &self.map)
            .field("rev", &self.rev)
            .field("back", &self.back.borrow())
            .field("phantom", &self.phantom)
            .finish()
    }
}

impl<'a, IdD: PrimInt, DTS, D: Borrow<DTS>> SimpleBfsMapper<'a, IdD, DTS, D> {
    pub fn with_store<HAST>(store: HAST, back: D) -> Self
    where
        HAST: HyperAST + Copy,
        DTS: PostOrder<HAST, IdD>,
    {
        let x: &DTS = back.borrow();
        let mut map = Vec::with_capacity(x.len());
        let mut rev = vec![num_traits::zero(); x.len()];
        let mut i = 0;
        rev[x.root().to_usize().unwrap()] = cast(i).unwrap();
        map.push(x.root());

        while map.len() < x.len() {
            let curr = &map[i];
            let cs = x.children(curr);
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

impl<'a, HAST: HyperAST + Copy, IdD: PrimInt, DTS: PostOrder<HAST, IdD>, D: Borrow<DTS>>
    From<(&'a HAST, D)> for SimpleBfsMapper<'a, IdD, DTS, D>
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
            let cs = x.children(curr);
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

impl<'a, HAST: HyperAST + Copy, IdD, DTS: DecompressedTreeStore<HAST, IdD>, D: Borrow<DTS>>
    ShallowDecompressedTreeStore<HAST, IdD> for SimpleBfsMapper<'a, IdD, DTS, D>
{
    fn len(&self) -> usize {
        self.map.len()
    }

    fn original(&self, id: &IdD) -> HAST::IdN {
        self.back.borrow().original(id)
    }

    fn root(&self) -> IdD {
        self.back.borrow().root()
    }

    fn child(&self, x: &IdD, p: &[impl PrimInt]) -> IdD {
        let b: &DTS = self.back.borrow();
        b.child(x, p)
    }

    fn children(&self, x: &IdD) -> Vec<IdD> {
        let b: &DTS = self.back.borrow();
        b.children(x)
    }
}

impl<'a, HAST: HyperAST + Copy, IdD, DTS: DecompressedTreeStore<HAST, IdD>, D: Borrow<DTS>>
    DecompressedTreeStore<HAST, IdD> for SimpleBfsMapper<'a, IdD, DTS, D>
{
    fn descendants(&self, x: &IdD) -> Vec<IdD>
where {
        self.back.borrow().descendants(x)
    }

    fn descendants_count(&self, x: &IdD) -> usize {
        self.back.borrow().descendants_count(x)
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        self.back.borrow().first_descendant(i)
    }

    fn is_descendant(&self, desc: &IdD, of: &IdD) -> bool {
        self.back.borrow().is_descendant(desc, of)
    }
}

impl<'a, 'd, IdD: PrimInt, DTS: DecompressedParentsLending<'a, IdD>, D: Borrow<DTS>>
    DecompressedParentsLending<'a, IdD> for SimpleBfsMapper<'d, IdD, DTS, D>
{
    type PIt = <DTS as DecompressedParentsLending<'a, IdD>>::PIt;
}

impl<
        'd,
        HAST: HyperAST + Copy,
        IdD: PrimInt,
        DTS: DecompressedTreeStore<HAST, IdD> + DecompressedWithParent<HAST, IdD>,
        D: Borrow<DTS>,
    > DecompressedWithParent<HAST, IdD> for SimpleBfsMapper<'d, IdD, DTS, D>
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

impl<
        'd,
        HAST: HyperAST + Copy,
        IdD: 'static + Clone,
        DTS: DecompressedTreeStore<HAST, IdD>,
        D: Borrow<DTS>,
    > BreadthFirstIt<HAST, IdD> for SimpleBfsMapper<'d, IdD, DTS, D>
{
    type It<'b> = Iter<'b, IdD>;
}

impl<
        'd,
        HAST: HyperAST + Copy,
        IdD: 'static + Clone,
        DTS: DecompressedTreeStore<HAST, IdD>,
        D: Borrow<DTS>,
    > BreadthFirstIterable<HAST, IdD> for SimpleBfsMapper<'d, IdD, DTS, D>
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
