use std::borrow::Borrow;
use std::cell::Ref;
use std::hash::Hash;
use std::str::FromStr;

use num_traits::PrimInt;
use strum_macros::EnumString;
use strum_macros::ToString;

pub trait HashKind {
    fn structural() -> Self;
    fn label() -> Self;
}

/// for now the types shared between all languages
#[derive(Debug, EnumString, ToString)]
#[strum(serialize_all = "snake_case")]
#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub enum Type {}

// impl std::fmt::Display for Type {
// }

pub trait Node {}

pub trait Stored: Node {
    type TreeId: Eq;
}

pub trait Typed {
    type Type: Eq + Hash + Copy; // todo try remove Hash and copy
    fn get_type(&self) -> Self::Type;
}

pub trait WithChildren: Node + Stored {
    type ChildIdx: PrimInt;

    fn child_count(&self) -> Self::ChildIdx;
    fn get_child(&self, idx: &Self::ChildIdx) -> Self::TreeId;
    fn get_child_rev(&self, idx: &Self::ChildIdx) -> Self::TreeId;
    fn get_children(&self) -> &[Self::TreeId];
}

/// just to show that it is not efficient
mod owned {
    use std::cell::RefMut;

    use super::*;

    pub trait WithChildren: Node {
        type ChildIdx: PrimInt;

        fn child_count(&self) -> Self::ChildIdx;
        fn get_child(&self, idx: &Self::ChildIdx) -> RefMut<Self>;
        fn get_child_mut(&mut self, idx: &Self::ChildIdx) -> Ref<Self>;
    }
    pub trait WithParent: Node {
        fn get_parent(&self) -> Ref<Self>;
        fn get_parent_mut(&mut self) -> RefMut<Self>;
    }
}

pub trait WithStats {
    fn size(&self) -> usize;
    fn height(&self) -> usize;
}

pub trait WithHashs {
    type HK: HashKind;
    type HP: PrimInt + PartialEq + Eq;
    fn hash(&self, kind: &Self::HK) -> Self::HP;
}

pub trait Labeled {
    type Label: Eq;
    fn get_label<'a>(&'a self) -> &'a Self::Label;
}

pub trait Tree: Typed + Labeled + WithChildren {
    fn has_children(&self) -> bool;
    fn has_label(&self) -> bool;
}

impl Type {
    pub fn new(kind: &str) -> Type {
        Type::from_str(kind)
            .map_err(|x| format!("{} for '{}'", x, kind))
            .unwrap()
    }
}

pub trait TreePath {}

pub trait NodeStore<'a, IdN, D> {
    fn resolve(&'a self, id: &IdN) -> D;

    // fn size(&self, id: &T::TreeId) -> usize;
    // fn height(&self, id: &T::TreeId) -> usize;
}

pub trait NodeStoreMut<'a, T: Stored, D>: NodeStore<'a, T::TreeId, D> {
    fn get_or_insert(&mut self, node: T) -> T::TreeId;
}

pub trait VersionedNodeStore<'a, IdN: Eq + Clone, D>: NodeStore<'a, IdN, D> {
    fn resolve_root(&self, version: (u8, u8, u8), node: IdN);
}

pub trait VersionedNodeStoreMut<'a, T: Stored, D>: NodeStoreMut<'a, T, D>
where
    T::TreeId: Clone,
{
    fn insert_as_root(&mut self, version: (u8, u8, u8), node: T) -> T::TreeId;
    //  {
    //     let r = self.get_or_insert(node);
    //     self.as_root(version, r.clone());
    //     r
    // }

    fn as_root(&mut self, version: (u8, u8, u8), node: T::TreeId);
}

pub type OwnedLabel = Vec<u8>;
pub type SlicedLabel = [u8];

pub trait LabelStore<L: ?Sized> {
    type I: Copy + Eq;

    fn get_or_insert<T: Borrow<L>>(&mut self, node: T) -> Self::I;
    
    fn get<T: Borrow<L>>(&self, node: T) -> Option<Self::I>;

    fn resolve(&self, id: &Self::I) -> &L;
}
