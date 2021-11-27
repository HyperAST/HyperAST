use std::cell::Ref;
use std::hash::Hash;
use std::ops::Deref;
use std::str::FromStr;

use num_traits::PrimInt;
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
    DecimalIntegerLiteral,
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

    // to cat
    ExpressionStatement,
}

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
    fn get_label<'a>(&'a self) -> &'a Self::Label;
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

// mod a {
//     use super::*;
//     use std::{borrow::Borrow, marker::PhantomData, ops::Deref, rc::Rc};

//     fn f() {
//         let r: Rc<u32> = Rc::new(3);

//         let a: &u32 = r.borrow();
//     }

//     pub trait NodeHandle:Deref {
//     }

//     pub trait NodeStore<T: Stored> {
//         type H:NodeHandle<Target=T>;

//         fn get_or_insert(&mut self, node: T) -> T::TreeId;

//         fn resolve(&self, id: &T::TreeId) -> &Self::H;
//     }

//     struct NH<T> {

//         _phantom:PhantomData<*const T>,
//     }

//     struct NS<T,U> {
//         pending:(),
//         /// given a threshold, nodes are put here and shared between all trees
//         /// extension of it is simple just allocate a new subVec
//         compressed: Vec<[U;256]>,
//         compressed_len:usize,
//         _phantom:PhantomData<*const T>,
//     }

//     trait Trait<'a,T> {}

//     struct Tr<'a, T> {
//         phantom:PhantomData<*const &'a T>,
//     }

//     impl<'a,T> Trait<'a,T> for Tr<'a,T> {
//     }

//     trait Foo<T> {
//         type Assoc<'a>: Trait<'a,T>;
//     }

//     struct Bar<T> {
//         phantom:PhantomData<*const T>,
//     }

//     // impl<T:'a> Foo<T> for Bar<T> where for <'a> T:'a {
//     //     type Assoc<'a> = Tr<'a, T>;
//     // }

// }
pub trait NodeStore<'a, T: Stored> {
    type D: Deref<Target = T>;

    fn get_or_insert(&mut self, node: T) -> T::TreeId;

    fn resolve(&'a self, id: &T::TreeId) -> Self::D;

    // fn size(&self, id: &T::TreeId) -> usize;
    // fn height(&self, id: &T::TreeId) -> usize;
}

pub trait VersionedNodeStore<'a, T: Stored>: NodeStore<'a, T>
where
    T::TreeId: Clone,
{
    fn insert_as_root(&mut self, version: (u8, u8, u8), node: T) -> T::TreeId {
        let r = self.get_or_insert(node);
        self.as_root(version, r.clone());
        r
    }

    fn as_root(&mut self, version: (u8, u8, u8), node: T::TreeId);
}

pub type OwnedLabel = Vec<u8>;

pub trait LabelStore<L: ?Sized> {
    type I: Copy + Eq;

    fn get_or_insert<T: AsRef<L>>(&mut self, node: T) -> Self::I;

    fn resolve(&self, id: &Self::I) -> &L;
}
