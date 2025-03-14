use std::{
    fmt::Debug,
    hash::{Hash, Hasher}, ops::Deref,
};

use crate::{store::labels::DefaultLabelIdentifier, store::nodes::DefaultNodeIdentifier};
use crate::{
    types::{AnyType, HashKind, HyperType, NodeId},
    PrimInt,
};
use num::traits::WrappingAdd;

use crate::nodes::{CompressedNode, HashSize};

pub type HashedNode = HashedCompressedNode<
    SyntaxNodeHashs<HashSize>,
    DefaultNodeIdentifier,
    DefaultLabelIdentifier,
    AnyType,
>;

pub trait NodeHashs {
    type Hash: PrimInt;
    /// the Default value is the most discriminating one
    type Kind: Default + HashKind;
    fn hash(&self, kind: &Self::Kind) -> Self::Hash;
    fn acc(&mut self, other: &Self);
}
pub trait ComputableNodeHashs: NodeHashs {
    fn prepare<T: ?Sized + Hash>(t: &T) -> Self::Hash;
    fn compute(
        &self,
        kind: &Self::Kind,
        k: Self::Hash,
        l: Self::Hash,
        size: Self::Hash,
    ) -> Self::Hash;
}

#[derive(Default, Clone, Copy, Eq)]
pub struct SyntaxNodeHashs<T: PrimInt> {
    pub structt: T,
    pub label: T,
    pub syntax: T,
}

pub trait IndexingHashBuilder<H: NodeHashs> {
    fn new<K: ?Sized + Hash, L: ?Sized + Hash>(hashs: H, k: &K, l: &L, size: impl PrimInt) -> Self;
    fn most_discriminating(&self) -> H::Hash;
}
pub trait MetaDataHashsBuilder<H: NodeHashs>: IndexingHashBuilder<H> {
    fn build(self) -> H;
}

pub struct HashesBuilder<H: NodeHashs> {
    h0: H::Hash,
    k: H::Hash,
    l: H::Hash,
    size: H::Hash,
    hashs: H,
}

impl<H: ComputableNodeHashs> IndexingHashBuilder<H> for HashesBuilder<H> {
    /// use size ignoring spaces to stay consistent with other AST approaches, i.e. those not tracking spaces
    fn new<K: ?Sized + Hash, L: ?Sized + Hash>(hashs: H, k: &K, l: &L, size: impl PrimInt) -> Self {
        // TODO redo a better builder
        let k = H::prepare(k);
        let l = H::prepare(l);
        let size = H::prepare(&size.to_usize().unwrap());
        let h0 = hashs.compute(&Default::default(), k, l, size);
        Self {
            h0,
            k,
            l,
            size,
            hashs,
        }
    }

    fn most_discriminating(&self) -> <H as NodeHashs>::Hash {
        self.h0
    }
}

/// TODO use some compile time macro/lib to make it ieasy to extend.
/// Moreover, only Syntax variant is possibly mandatory anyway
#[derive(Clone, Copy)]
pub enum SyntaxNodeHashsKinds {
    Struct,
    Label,
    Syntax,
}

impl std::ops::Deref for SyntaxNodeHashsKinds {
    type Target = Self;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl<T: PrimInt> Debug for SyntaxNodeHashs<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "H: {:?}/{:?}/{:?}",
            &self.structt.to_usize().unwrap(),
            &self.label.to_usize().unwrap(),
            &self.syntax.to_usize().unwrap(),
        )
    }
}

impl Default for SyntaxNodeHashsKinds {
    fn default() -> Self {
        Self::Syntax
    }
}

impl<T: PrimInt> PartialEq for SyntaxNodeHashs<T> {
    fn eq(&self, other: &Self) -> bool {
        self.syntax == other.syntax && self.label == other.label && self.structt == other.structt
    }
}

impl HashKind for SyntaxNodeHashsKinds {
    fn structural() -> Self {
        SyntaxNodeHashsKinds::Struct
    }

    fn label() -> Self {
        SyntaxNodeHashsKinds::Label
    }
}

impl<T: PrimInt + WrappingAdd> NodeHashs for SyntaxNodeHashs<T> {
    type Kind = SyntaxNodeHashsKinds;
    type Hash = T;
    fn hash(&self, kind: &Self::Kind) -> T {
        match kind {
            SyntaxNodeHashsKinds::Struct => self.structt,
            SyntaxNodeHashsKinds::Label => self.label,
            SyntaxNodeHashsKinds::Syntax => self.syntax,
        }
    }

    fn acc(&mut self, other: &Self) {
        self.structt = self.structt.wrapping_add(&other.structt);
        self.label = self.label.wrapping_add(&other.label);
        self.syntax = self.syntax.wrapping_add(&other.syntax);
    }
}

impl ComputableNodeHashs for SyntaxNodeHashs<u32> {
    fn prepare<U: ?Sized + Hash>(x: &U) -> u32 {
        use crate::utils::{self, clamp_u64_to_u32};
        clamp_u64_to_u32(&utils::hash(x))
    }

    fn compute(
        &self,
        kind: &Self::Kind,
        k: Self::Hash,
        l: Self::Hash,
        size: Self::Hash,
    ) -> Self::Hash {
        inner_node_hash(k, l, size as u32, self.hash(kind))
    }
}

impl MetaDataHashsBuilder<SyntaxNodeHashs<u32>> for HashesBuilder<SyntaxNodeHashs<u32>> {
    fn build(self) -> SyntaxNodeHashs<u32> {
        type K = SyntaxNodeHashsKinds;
        SyntaxNodeHashs {
            structt: self.hashs.compute(&K::Struct, self.k, 0, self.size),
            label: self.hashs.compute(&K::Label, self.k, self.l, self.size),
            syntax: self.h0,
        }
    }
}

impl SyntaxNodeHashs<u32> {
    pub fn persist(self, dyn_builder: &mut impl crate::store::nodes::EntityBuilder) {
        dyn_builder.add(self);
    }
}

#[derive(Debug)]
pub struct HashedCompressedNode<U: NodeHashs, N, L, T> {
    pub(crate) hashs: U,
    pub(crate) node: CompressedNode<N, L, T>,
}

impl<H: Hash + PrimInt, U: NodeHashs<Hash = H>, N, L, T> crate::types::Node
    for HashedCompressedNode<U, N, L, T>
{
}

impl<H: Hash + PrimInt, U: NodeHashs<Hash = H>, N: NodeId, L, T> crate::types::Stored
    for HashedCompressedNode<U, N, L, T>
{
    type TreeId = N;
}

impl<U: NodeHashs + PartialEq, N: PartialEq, L: PartialEq, T: PartialEq> PartialEq
    for HashedCompressedNode<U, N, L, T>
{
    fn eq(&self, other: &Self) -> bool {
        self.hashs.eq(&other.hashs) && self.node.eq(&other.node)
    }
}

impl<U: NodeHashs + PartialEq, N: Eq, L: Eq, T: Eq> Eq for HashedCompressedNode<U, N, L, T> {}

impl<
        H: Hash + PrimInt,
        U: NodeHashs<Hash = H>,
        N,
        L,
        T: HyperType + Copy + Hash + Eq + Sync + Send,
    > crate::types::Typed for HashedCompressedNode<U, N, L, T>
{
    type Type = T;

    fn get_type(&self) -> T
    where
        T: Send + Sync,
    {
        self.node.get_type()
    }
}

impl<H: Hash + PrimInt, U: NodeHashs<Hash = H>, N, L: Eq, T> crate::types::Labeled
    for HashedCompressedNode<U, N, L, T>
{
    type Label = L;

    fn get_label_unchecked(&self) -> &L {
        self.node.get_label_unchecked()
    }

    fn try_get_label<'a>(&'a self) -> Option<&'a Self::Label> {
        self.node.try_get_label()
    }
}

impl<'a, H: Hash + PrimInt, U: NodeHashs<Hash = H>, N: NodeId<IdN = N> + Copy + Eq, L, T>
    crate::types::CLending<'a, u16, N::IdN> for HashedCompressedNode<U, N, L, T>
where
    N::IdN: Copy + Eq,
{
    type Children = crate::types::ChildrenSlice<'a, N::IdN>;
}

impl<H: Hash + PrimInt, U: NodeHashs<Hash = H>, N: NodeId<IdN = N> + Copy + Eq, L, T>
    crate::types::WithChildren for HashedCompressedNode<U, N, L, T>
where
    N::IdN: Copy + Eq,
{
    type ChildIdx = u16;
    // type Children<'a>
    //     = MySlice<N::IdN>
    // where
    //     Self: 'a;

    fn child_count(&self) -> u16 {
        self.node.child_count()
    }

    fn child(&self, idx: &Self::ChildIdx) -> Option<N::IdN> {
        self.node.child(idx)
    }

    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<N::IdN> {
        self.node.child_rev(idx)
    }

    // fn descendants_count(&self) -> Self::TreeId {
    //     self.node.descendants_count()
    // }

    // fn children_unchecked<'a>(&'a self) -> &'a [Self::TreeId] {
    //     self.node.children_unchecked()
    // }

    // fn get_children_cpy<'a>(&'a self) -> Vec<Self::TreeId> {
    //     self.node.get_children_cpy()
    // }

    fn children(&self) -> Option<crate::types::LendC<'_, Self, Self::ChildIdx, N::IdN>> {
        self.node.children()
    }
}

impl<H: Hash + PrimInt, U: NodeHashs<Hash = H>, N, L, T> crate::types::ErasedHolder
    for HashedCompressedNode<U, N, L, T>
{
    fn unerase_ref<TT: 'static + Send + Sync>(&self, tid: std::any::TypeId) -> Option<&TT> {
        unimplemented!("CompressedNode should be deprecated anyway")
    }
}

impl<
        H: Hash + PrimInt,
        U: NodeHashs<Hash = H>,
        N: NodeId<IdN = N> + Copy + Eq,
        L: Eq,
        T: Copy + Hash + Eq + Sync + Send + HyperType,
    > crate::types::Tree for HashedCompressedNode<U, N, L, T>
where
    N::IdN: Copy + Eq,
{
    fn has_children(&self) -> bool {
        self.node.has_children()
    }

    fn has_label(&self) -> bool {
        self.node.has_label()
    }
}

impl<U: NodeHashs, N, L, T> Hash for HashedCompressedNode<U, N, L, T>
where
    U::Hash: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hashs.hash(&Default::default()).hash(state);
    }
}

impl<H: Hash + PrimInt, U: NodeHashs<Hash = H>, N, L, T> crate::types::WithHashs
    for HashedCompressedNode<U, N, L, T>
{
    type HK = U::Kind;
    type HP = H;

    fn hash(&self, kind: impl std::ops::Deref<Target=Self::HK>) -> H {
        self.hashs.hash(&kind)
    }
}

// impl<H: Hash + PrimInt, U: NodeHashs<Hash = H>, N, L> HashedCompressedNode<U, N, L,T> {
//     pub(crate) fn new(hashs: U, node: CompressedNode<N, L>) -> Self {
//         Self { hashs, node }
//     }
// }

static ENTER: u32 = {
    let mut result = 1u32;
    result = 31 * result + 'e' as u32;
    result = 31 * result + 'n' as u32;
    result = 31 * result + 't' as u32;
    result = 31 * result + 'e' as u32;
    result = 31 * result + 'r' as u32;
    result
};
static LEAVE: u32 = {
    let mut result = 1u32;
    result = 31 * result + 'l' as u32;
    result = 31 * result + 'e' as u32;
    result = 31 * result + 'a' as u32;
    result = 31 * result + 'v' as u32;
    result = 31 * result + 'e' as u32;
    result
};
static BASE: &u32 = &33u32;

/// hash from GumtreeDiff
pub fn inner_node_hash(kind: u32, label: u32, size: u32, middle_hash: u32) -> u32 {
    let mut left = 1u32;
    left = 31 * left + kind;
    left = 31 * left + label;
    left = 31 * left + ENTER;

    let mut right = 1u32;
    right = 31 * right + kind;
    right = 31 * right + label;
    right = 31 * right + LEAVE;

    left.wrapping_add(middle_hash)
        .wrapping_add(right.wrapping_mul(hash_factor(size)))
}

fn hash_factor(exponent: u32) -> u32 {
    fast_exponentiation(*BASE, exponent)
}

fn fast_exponentiation(base: u32, exponent: u32) -> u32 {
    if exponent == 0 {
        1
    } else if exponent == 1 {
        base
    } else {
        let mut result: u32 = 1;
        let mut exponent = exponent;
        let mut base = base;
        while exponent > 0 {
            if (exponent & 1) != 0 {
                result = result.wrapping_mul(base);
            }
            exponent >>= 1;
            base = base.wrapping_mul(base);
        }
        result
    }
}
