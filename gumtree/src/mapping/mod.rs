use std::marker::PhantomData;

use num_traits::PrimInt;

use crate::tree::tree_path::CompressedTreePath;

pub mod remapping;
pub mod visualize;
pub mod compress;




pub struct ArenaMStore<M: Mree> {
    v: Vec<M>,
}
impl<M: Mree<Id = usize>> ArenaMStore<M>
where
    for<'a> &'a M: Mree<Id = usize, Idx = M::Idx>,
{
    fn insert(&mut self, x: M) -> usize {
        let id = self.v.len();
        self.v.push(x);
        id
    }
}

impl<M: Mree<Id = usize>> MS for ArenaMStore<M>
where
    for<'a> &'a M: Mree<Id = usize, Idx = M::Idx>,
{
    type Id = M::Id;

    type Idx = M::Idx;

    type R<'a> = &'a M
    where
        Self: 'a;

    fn resolve<'a>(&'a self, id: Self::Id) -> Self::R<'a> {
        &self.v[id]
    }
}

pub trait Mree {
    type Id;
    type Idx;

    fn definitely_mapped(
        &self,
        i: Self::Idx,
    ) -> Option<(Option<Self::Id>, CompressedTreePath<Self::Idx>)>;
    fn maybe_mapped(&self, i: Self::Idx) -> Vec<(Self::Id, CompressedTreePath<Self::Idx>)>;
}

impl<Id: Clone, Idx: PrimInt + Clone> Mree for SimpleCompressedMapping<Id, Idx> {
    type Id = Id;

    type Idx = Idx;

    fn definitely_mapped(
        &self,
        i: Self::Idx,
    ) -> Option<(Option<Self::Id>, CompressedTreePath<Self::Idx>)> {
        // self.dm[i.to_usize().unwrap()].clone()
        None
    }

    fn maybe_mapped(&self, i: Self::Idx) -> Vec<(Self::Id, CompressedTreePath<Self::Idx>)> {
        self.mm[i.to_usize().unwrap()].clone()
    }
}

impl<Id: Clone, Idx: PrimInt + Clone> Mree for &SimpleCompressedMapping<Id, Idx> {
    type Id = Id;

    type Idx = Idx;

    fn definitely_mapped(
        &self,
        i: Self::Idx,
    ) -> Option<(Option<Self::Id>, CompressedTreePath<Self::Idx>)> {
        // self.dm[i.to_usize().unwrap()].clone()
        None
    }

    fn maybe_mapped(&self, i: Self::Idx) -> Vec<(Self::Id, CompressedTreePath<Self::Idx>)> {
        self.mm[i.to_usize().unwrap()].clone()
    }
}

#[derive(Default,Debug)]
pub struct SimpleCompressedMapping<Id, Idx: PrimInt> {
    is_mapped: bool,
    // dm: Vec<Option<(Option<Id>, CompressedTreePath<Idx>)>>,
    mm: Vec<Vec<(Id, CompressedTreePath<Idx>)>>,
}


struct BoxedCompressedMapping<Id, Idx> {
    nodes: Box<[Option<Id>]>,
    paths: Box<[Idx]>,
}
struct SingleCompressedMapping<Id, Idx, const N: usize> {
    nodes: Option<Id>,
    paths: [u8; N],
    phantom: PhantomData<*const Idx>,
}
struct ShiftedCompressedMapping<Id, Idx> {
    nodes: Box<[Option<Id>]>,
    offsets: Box<[u8]>,
    paths: Box<[u8]>,
    phantom: PhantomData<*const Idx>,
}

trait MS {
    type Id;
    type Idx;
    type R<'a>: Mree<Id = Self::Id, Idx = Self::Idx>
    where
        Self: 'a;

    fn resolve(&self, id: Self::Id) -> Self::R<'_>;
}