//! Map tree pairs, allow to compress those mappings

use std::{fmt::Debug, marker::PhantomData};

use num_traits::{PrimInt, ToPrimitive};

use crate::tree::tree_path::{TreePath};

pub mod compress;
pub mod remapping;
pub mod visualize;

#[derive(Debug)]
pub struct ArenaMStore<M: Mree> {
    v: Vec<M>,
}

impl<IdM, P: TreePath> CmBuilder<IdM, P> for SimpleCompressedMapping<IdM, P>
where
    P::Item: PrimInt,
{
    fn mapped(&mut self) {
        self.is_mapped = true;
    }

    fn push(&mut self, i: P::Item, x: IdM, p: P) {
        let i = i.to_usize().unwrap();
        if self.mm.len() <= i {
            self.mm.resize_with(i + 1, || vec![]);
        }
        self.mm[i].push((x, p));
    }
}

impl<IdM: Clone + PrimInt, P: TreePath> CompressedMappingStore
    for ArenaMStore<SimpleCompressedMapping<IdM, P>>
where
    P::Item: PrimInt,
    P: Clone,
{
    type Id = IdM;

    type Idx = P::Item;
    type P = P;

    type R<'a> = &'a SimpleCompressedMapping<IdM, P>
    where
        Self: 'a;

    fn resolve<'a>(&'a self, id: Self::Id) -> Self::R<'a> {
        &self.v[id.to_usize().unwrap()]
    }

    type Builder = SimpleCompressedMapping<IdM, P>;

    fn insert(&mut self, x: Self::Builder) -> IdM {
        let id = self.v.len();
        self.v.push(x);
        num_traits::cast(id).unwrap()
    }
}

pub trait Mree {
    type Id;
    type Idx;
    type P: IntoIterator<Item = Self::Idx>;

    fn definitely_mapped(&self, i: Self::Idx) -> Option<(Option<Self::Id>, Self::P)>;
    fn maybe_mapped(&self, i: Self::Idx) -> Vec<(Self::Id, Self::P)>;
    fn is_mapped(&self) -> bool;
}

impl<Id: Clone, P: IntoIterator> Mree for SimpleCompressedMapping<Id, P>
where
    P::Item: PrimInt + Clone,
    P: Clone,
{
    type Id = Id;

    type Idx = P::Item;

    type P = P;

    fn definitely_mapped(&self, i: Self::Idx) -> Option<(Option<Self::Id>, Self::P)> {
        // self.dm[i.to_usize().unwrap()].clone()
        None
    }

    fn maybe_mapped(&self, i: Self::Idx) -> Vec<(Self::Id, Self::P)> {
        self.mm[i.to_usize().unwrap()].clone()
    }

    fn is_mapped(&self) -> bool {
        self.is_mapped
    }
}

impl<Id: Clone, P: IntoIterator> Mree for &SimpleCompressedMapping<Id, P>
where
    P::Item: PrimInt + Clone,
    P: Clone,
{
    type Id = Id;

    type Idx = P::Item;

    type P = P;

    fn definitely_mapped(&self, i: Self::Idx) -> Option<(Option<Self::Id>, Self::P)> {
        // self.dm[i.to_usize().unwrap()].clone()
        None
    }

    fn maybe_mapped(&self, i: Self::Idx) -> Vec<(Self::Id, Self::P)> {
        self.mm
            .get(i.to_usize().unwrap())
            .map_or(vec![], |x| x.clone())
    }

    fn is_mapped(&self) -> bool {
        self.is_mapped
    }
}

pub struct SimpleCompressedMapping<Id, P: IntoIterator> {
    is_mapped: bool,
    // dm: Vec<Option<(Option<Id>, CompressedTreePath<Idx>)>>,
    mm: Vec<Vec<(Id, P)>>,
}

impl<IdM, P: IntoIterator> Default for SimpleCompressedMapping<IdM, P> {
    fn default() -> Self {
        Self {
            is_mapped: Default::default(),
            mm: Default::default(),
        }
    }
}

impl<IdM: Debug, P: IntoIterator + Debug> Debug for SimpleCompressedMapping<IdM, P> {
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

pub trait CmBuilder<IdM, P: TreePath>: Default {
    fn mapped(&mut self);
    fn push(&mut self, i: P::Item, x: IdM, p: P);
}

pub trait CompressedMappingStore {
    type Id;
    type Idx;
    type P: TreePath<Item=Self::Idx>;
    type R<'a>: Mree<Id = Self::Id, Idx = Self::Idx, P = Self::P>
    where
        Self: 'a;
    type Builder: CmBuilder<Self::Id, Self::P>;

    fn resolve(&self, id: Self::Id) -> Self::R<'_>;
    fn insert(&mut self, x: Self::Builder) -> Self::Id;
}
