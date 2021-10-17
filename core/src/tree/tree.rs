use std::cell::Ref;
use std::hash::Hash;
use std::str::FromStr;

use num_traits::PrimInt;
use strum_macros::Display;
use strum_macros::EnumString;
use strum_macros::ToString;

/// for now the types shared between all languages
#[derive(Debug, EnumString, ToString)]
#[strum(serialize_all = "snake_case")]
#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Spaces,
    //structural
    File,
    Program,
    ClassBody,
    MethodBody,
    FormalParameters,
    Block,
    VariableDeclarator,
    //references
    VoidType,
    IntegralType,
    Identifier,
    //literal
    HexIntegerLiteral,
    //declarations
    ClassDeclaration,
    MethodDeclaration,
    FieldDeclaration,
    //keywords
    Class,
    Int,
    #[strum(serialize = ";")]
    SemiColon,
    #[strum(serialize = "=")]
    Equal,
    #[strum(serialize = "{")]
    LeftCurly,
    #[strum(serialize = "}")]
    RightCurly,
    #[strum(serialize = "(")]
    LeftPar,
    #[strum(serialize = ")")]
    RightPar,
    #[strum(serialize = "[")]
    LeftBrace,
    #[strum(serialize = "]")]
    RightBrace,
}

// impl std::fmt::Display for Type {
// }

pub trait Node {}

pub trait Stored: Node {
    type TreeId: PrimInt;
}

pub trait Typed {
    type Type: Eq + Hash + Copy; // todo try remove Hash and copy
    fn get_type(&self) -> Self::Type;
}

pub trait WithChildren: Node + Stored {
    type ChildIdx: PrimInt;

    fn child_count(&self) -> Self::ChildIdx;
    fn get_child(&self, idx: &Self::ChildIdx) -> Self::TreeId;
    fn get_children(&self) -> &[Self::TreeId];
}

/// just to show that it is not efficient
mod Owned {
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

// pub trait WithChildrenAndStore<T:Stored,S: NodeStore<T>> : WithChildren {
//     fn size(&self, store: &S) -> usize;
//     fn height(&self, store: &S) -> usize;
// }

pub trait WithStats {
    fn size(&self) -> usize;
    fn height(&self) -> usize;
}

pub trait HashKind {
    fn structural() -> Self;
    fn label() -> Self;
}

pub trait WithHashs {
    type HK: HashKind;
    type HP: PrimInt + PartialEq + Eq;
    fn hash(&self, kind: &Self::HK) -> Self::HP;
}

pub trait Labeled {
    type Label: Eq;
    fn get_label(&self) -> Self::Label;
}

pub trait Tree: Typed + Labeled + WithChildren {
    fn has_children(&self) -> bool;
    fn has_label(&self) -> bool;
}
pub trait DeCompressedTree<T: PrimInt>: Tree {
    fn get_parent(&self) -> T;
    // fn has_parent(&self) -> bool;
}

impl Type {
    pub fn new(kind: &str) -> Type {
        Type::from_str(kind)
            .map_err(|x| format!("{} for '{}'", x, kind))
            .unwrap()
    }
}

pub trait TreePath {}

pub trait NodeStore<T: Stored> {
    fn get_id_or_insert_node(&mut self, node: T) -> T::TreeId;

    fn get_node_at_id<'b>(&'b self, id: &T::TreeId) -> Ref<T>;

    // fn size(&self, id: &T::TreeId) -> usize;
    // fn height(&self, id: &T::TreeId) -> usize;
}

pub type OwnedLabel = Vec<u8>;

pub trait LabelStore {
    type I: PrimInt;

    fn get_id_or_insert_node(&mut self, node: OwnedLabel) -> Self::I;

    fn get_node_at_id<'b>(&'b self, id: &Self::I) -> Ref<OwnedLabel>;
}
