use std::{borrow::Borrow, fmt::Debug, marker::PhantomData};

use num_traits::PrimInt;

use crate::tree::tree_path::CompressedTreePath;

pub mod compress;
pub mod remapping;
pub mod visualize;

pub struct ArenaMStore<M: Mree> {
    v: Vec<M>,
}
// impl<M: Mree<Id = usize>> ArenaMStore<M>
// where
//     for<'a> &'a M: Mree<Id = usize, Idx = M::Idx>,
// {
//     fn insert(&mut self, x: M) -> usize {
//         let id = self.v.len();
//         self.v.push(x);
//         id
//     }
// }

impl<IdM, Idx: PrimInt> CmBuilder<IdM, Idx> for SimpleCompressedMapping<IdM, Idx> {
    fn mapped(&mut self) {
        self.is_mapped = true;
    }

    fn push(&mut self, i: Idx, x: IdM, p: CompressedTreePath<Idx>) {
        let i = i.to_usize().unwrap();
        if self.mm.len() <= i {
            self.mm.resize_with(i + 1, || vec![]);
        }
        self.mm[i].push((x, p));
    }
}

impl<IdM: Clone + PrimInt, Idx: PrimInt> CompressedMappingStore
    for ArenaMStore<SimpleCompressedMapping<IdM, Idx>>
{
    type Id = IdM;

    type Idx = Idx;

    type R<'a> = &'a SimpleCompressedMapping<IdM, Idx>
    where
        Self: 'a;

    fn resolve<'a>(&'a self, id: Self::Id) -> Self::R<'a> {
        &self.v[id.to_usize().unwrap()]
    }

    type Builder = SimpleCompressedMapping<IdM, Idx>;

    fn insert(&mut self, x: Self::Builder) -> IdM {
        let id = self.v.len();
        self.v.push(x);
        num_traits::cast(id).unwrap()
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
    fn is_mapped(&self) -> bool;
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

    fn is_mapped(&self) -> bool {
        self.is_mapped
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
        self.mm
            .get(i.to_usize().unwrap())
            .map_or(vec![], |x| x.clone())
    }

    fn is_mapped(&self) -> bool {
        self.is_mapped
    }
}

pub struct SimpleCompressedMapping<Id, Idx> {
    is_mapped: bool,
    // dm: Vec<Option<(Option<Id>, CompressedTreePath<Idx>)>>,
    mm: Vec<Vec<(Id, CompressedTreePath<Idx>)>>,
}

impl<IdM, Idx> Default for SimpleCompressedMapping<IdM, Idx> {
    fn default() -> Self {
        Self {
            is_mapped: Default::default(),
            mm: Default::default(),
        }
    }
}

impl<IdM: Debug, Idx: PrimInt> Debug for SimpleCompressedMapping<IdM, Idx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleCompressedMapping")
            .field("is_mapped", &self.is_mapped)
            .field("mm", &self.mm)
            .finish()
    }
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

pub trait CmBuilder<IdM, Idx>: Default {
    fn mapped(&mut self);
    fn push(&mut self, i: Idx, x: IdM, p: CompressedTreePath<Idx>);
}

pub trait CompressedMappingStore {
    type Id;
    type Idx;
    type R<'a>: Mree<Id = Self::Id, Idx = Self::Idx>
    where
        Self: 'a;
    type Builder: CmBuilder<Self::Id, Self::Idx>;

    fn resolve(&self, id: Self::Id) -> Self::R<'_>;
    fn insert(&mut self, x: Self::Builder) -> Self::Id;
}
