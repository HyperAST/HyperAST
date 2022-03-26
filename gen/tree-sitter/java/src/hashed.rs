use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

use num::{traits::WrappingAdd, PrimInt};
use rusted_gumtree_core::tree::tree::{HashKind, Type};

use crate::nodes::{CompressedNode, HashSize, LabelIdentifier, NodeIdentifier};

pub type HashedNode =
    HashedCompressedNode<SyntaxNodeHashs<HashSize>, NodeIdentifier, LabelIdentifier>;

pub trait NodeHashs {
    type Hash: PrimInt;
    type Kind: Default + HashKind;
    fn hash(&self, kind: &Self::Kind) -> Self::Hash;
    fn acc(&mut self, other: &Self);
}

#[derive(Default, Clone, Copy, Eq)]
pub struct SyntaxNodeHashs<T: PrimInt> {
    pub structt: T,
    pub label: T,
    pub syntax: T,
}

pub enum SyntaxNodeHashsKinds {
    Struct,
    Label,
    Syntax,
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
        // f.debug_struct("SyntaxNodeHashs")
        //     .field("structt", &self.structt)
        //     .field("label", &self.label)
        //     .field("syntax", &self.syntax)
        //     .finish()
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

#[derive(Debug)]
pub struct HashedCompressedNode<U: NodeHashs, N, L> {
    pub(crate) hashs: U,
    pub(crate) node: CompressedNode<N, L>,
}

impl<T: Hash + PrimInt, U: NodeHashs<Hash = T>, N, L> rusted_gumtree_core::tree::tree::Node
    for HashedCompressedNode<U, N, L>
{
}

impl<T: Hash + PrimInt, U: NodeHashs<Hash = T>, N: Eq, L> rusted_gumtree_core::tree::tree::Stored
    for HashedCompressedNode<U, N, L>
{
    type TreeId = N;
}

impl<U: NodeHashs + PartialEq, N: PartialEq, L: PartialEq> PartialEq
    for HashedCompressedNode<U, N, L>
{
    fn eq(&self, other: &Self) -> bool {
        self.hashs.eq(&other.hashs) && self.node.eq(&other.node)
    }
}

impl<U: NodeHashs + PartialEq, N: Eq, L: Eq> Eq for HashedCompressedNode<U, N, L> {}

impl<T: Hash + PrimInt, U: NodeHashs<Hash = T>, N, L> rusted_gumtree_core::tree::tree::Typed
    for HashedCompressedNode<U, N, L>
{
    type Type = Type;

    fn get_type(&self) -> Type {
        self.node.get_type()
    }
}

impl<T: Hash + PrimInt, U: NodeHashs<Hash = T>, N, L: Eq> rusted_gumtree_core::tree::tree::Labeled
    for HashedCompressedNode<U, N, L>
{
    type Label = L;

    fn get_label(&self) -> &L {
        self.node.get_label()
    }
}

impl<T: Hash + PrimInt, U: NodeHashs<Hash = T>, N: Eq + Clone, L>
    rusted_gumtree_core::tree::tree::WithChildren for HashedCompressedNode<U, N, L>
{
    type ChildIdx = u16;

    fn child_count(&self) -> u16 {
        self.node.child_count()
    }

    fn get_child(&self, idx: &Self::ChildIdx) -> N {
        self.node.get_child(idx)
    }

    fn get_child_rev(&self, idx: &Self::ChildIdx) -> Self::TreeId {
    self.node.get_child_rev(idx)
    }

    // fn descendants_count(&self) -> Self::TreeId {
    //     self.node.descendants_count()
    // }

    fn get_children<'a>(&'a self) -> &'a [Self::TreeId] {
        self.node.get_children()
    }
}

impl<T: Hash + PrimInt, U: NodeHashs<Hash = T>, N: Eq + Clone, L: Eq>
    rusted_gumtree_core::tree::tree::Tree for HashedCompressedNode<U, N, L>
{
    fn has_children(&self) -> bool {
        self.node.has_children()
    }

    fn has_label(&self) -> bool {
        self.node.has_label()
    }
}

impl<U: NodeHashs, N, L> Hash for HashedCompressedNode<U, N, L>
where
    U::Hash: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hashs.hash(&Default::default()).hash(state);
    }
}

impl<T: Hash + PrimInt, U: NodeHashs<Hash = T>, N, L> rusted_gumtree_core::tree::tree::WithHashs
    for HashedCompressedNode<U, N, L>
{
    type HK = U::Kind;
    type HP = T;

    fn hash(&self, kind: &Self::HK) -> T {
        self.hashs.hash(kind)
    }
}

impl<T: Hash + PrimInt, U: NodeHashs<Hash = T>, N, L> HashedCompressedNode<U, N, L> {
    pub(crate) fn new(hashs: U, node: CompressedNode<N, L>) -> Self {
        Self { hashs, node }
    }
}

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

pub fn inner_node_hash(kind: &u32, label: &u32, size: &u32, middle_hash: &u32) -> u32 {
    let mut left = 1u32;
    left = 31 * left + kind;
    left = 31 * left + label;
    left = 31 * left + ENTER;

    let mut right = 1u32;
    right = 31 * right + kind;
    right = 31 * right + label;
    right = 31 * right + LEAVE;

    left.wrapping_add(*middle_hash)
        .wrapping_add(right.wrapping_mul(hash_factor(size)))
}

fn hash_factor(exponent: &u32) -> u32 {
    fast_exponentiation(BASE, exponent)
}

fn fast_exponentiation(base: &u32, exponent: &u32) -> u32 {
    if exponent == &0 {
        1
    } else if exponent == &1 {
        *base
    } else {
        let mut result: u32 = 1;
        let mut exponent = *exponent;
        let mut base = *base;
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
