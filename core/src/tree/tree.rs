use std::borrow::Borrow;
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
    // //structural
    // File,
    // Program,
    // ClassBody,
    // MethodBody,
    // FormalParameters,
    // Block,
    // VariableDeclarator,
    // //references
    // VoidType,
    // IntegralType,
    // Identifier,
    // //literal
    // HexIntegerLiteral,
    // DecimalIntegerLiteral,
    // //declarations
    // ClassDeclaration,
    // MethodDeclaration,
    // FieldDeclaration,
    // //keywords
    // Class,
    // Int,
    // #[strum(serialize = ";")]
    // SemiColon,
    // #[strum(serialize = "=")]
    // Equal,
    // #[strum(serialize = "{")]
    // LeftCurly,
    // #[strum(serialize = "}")]
    // RightCurly,
    // #[strum(serialize = "(")]
    // LeftPar,
    // #[strum(serialize = ")")]
    // RightPar,
    // #[strum(serialize = "[")]
    // LeftBrace,
    // #[strum(serialize = "]")]
    // RightBrace,

    // // to cat
    // ExpressionStatement,
    // Comment,
    // PackageDeclaration
    Literal,
    SimpleType,
    Type,
    UnannotatedType,
    Declaration,
    Expression,
    PrimaryExpression,
    Statement,
    AnnotatedType,
    Annotation,
    AnnotationArgumentList,
    AnnotationTypeBody,
    AnnotationTypeDeclaration,
    AnnotationTypeElementDeclaration,
    ArgumentList,
    ArrayAccess,
    ArrayCreationExpression,
    ArrayInitializer,
    ArrayType,
    AssertStatement,
    AssignmentExpression,
    Asterisk,
    BinaryExpression,
    Block,
    BreakStatement,
    CastExpression,
    CatchClause,
    CatchFormalParameter,
    CatchType,
    ClassBody,
    ClassDeclaration,
    ClassLiteral,
    ConstantDeclaration,
    ConstructorBody,
    ConstructorDeclaration,
    ContinueStatement,
    Dimensions,
    DimensionsExpr,
    DoStatement,
    ElementValueArrayInitializer,
    ElementValuePair,
    EnhancedForStatement,
    EnumBody,
    EnumBodyDeclarations,
    EnumConstant,
    EnumDeclaration,
    ExplicitConstructorInvocation,
    ExpressionStatement,
    ExtendsInterfaces,
    FieldAccess,
    FieldDeclaration,
    FinallyClause,
    FloatingPointType,
    ForStatement,
    FormalParameter,
    FormalParameters,
    GenericType,
    IfStatement,
    ImportDeclaration,
    InferredParameters,
    InstanceofExpression,
    IntegralType,
    InterfaceBody,
    InterfaceDeclaration,
    InterfaceTypeList,
    LabeledStatement,
    LambdaExpression,
    LocalVariableDeclaration,
    MarkerAnnotation,
    MethodDeclaration,
    MethodInvocation,
    MethodReference,
    Modifiers,
    ModuleBody,
    ModuleDeclaration,
    ModuleDirective,
    ObjectCreationExpression,
    PackageDeclaration,
    ParenthesizedExpression,
    Program,
    ReceiverParameter,
    RecordDeclaration,
    RequiresModifier,
    Resource,
    ResourceSpecification,
    ReturnStatement,
    ScopedIdentifier,
    ScopedTypeIdentifier,
    SpreadParameter,
    StaticInitializer,
    SuperInterfaces,
    Superclass,
    SwitchBlock,
    SwitchBlockStatementGroup,
    SwitchExpression,
    SwitchStatement,
    SwitchLabel,
    SwitchRule,
    SynchronizedStatement,
    TernaryExpression,
    ThrowStatement,
    Throws,
    TryStatement,
    TryWithResourcesExtendedStatement,
    TryWithResourcesStatement,
    TypeArguments,
    TypeBound,
    TypeParameter,
    TypeParameters,
    UnaryExpression,
    UpdateExpression,
    VariableDeclarator,
    WhileStatement,
    Wildcard,
    WildcardExtends,
    WildcardSuper,
    YieldStatement,
    #[strum(serialize = "!")]
    TS0,
    #[strum(serialize = "!=")]
    TS1,
    #[strum(serialize = "%")]
    TS2,
    #[strum(serialize = "%=")]
    TS3,
    #[strum(serialize = "&")]
    TS4,
    #[strum(serialize = "&&")]
    TS5,
    #[strum(serialize = "&=")]
    TS6,
    #[strum(serialize = "(")]
    TS7,
    #[strum(serialize = ")")]
    TS8,
    #[strum(serialize = "*")]
    TS9,
    #[strum(serialize = "*=")]
    TS10,
    #[strum(serialize = "+")]
    TS11,
    #[strum(serialize = "++")]
    TS12,
    #[strum(serialize = "+=")]
    TS13,
    #[strum(serialize = ",")]
    TS14,
    #[strum(serialize = "-")]
    TS15,
    #[strum(serialize = "--")]
    TS16,
    #[strum(serialize = "-=")]
    TS17,
    #[strum(serialize = "->")]
    TS18,
    #[strum(serialize = ".")]
    TS19,
    #[strum(serialize = "...")]
    TS20,
    #[strum(serialize = "/")]
    TS21,
    #[strum(serialize = "/=")]
    TS22,
    #[strum(serialize = ":")]
    TS23,
    #[strum(serialize = "::")]
    TS24,
    #[strum(serialize = ";")]
    TS25,
    #[strum(serialize = "<")]
    TS26,
    #[strum(serialize = "<<")]
    TS27,
    #[strum(serialize = "<<=")]
    TS28,
    #[strum(serialize = "<=")]
    TS29,
    #[strum(serialize = "=")]
    TS30,
    #[strum(serialize = "==")]
    TS31,
    #[strum(serialize = ">")]
    TS32,
    #[strum(serialize = ">=")]
    TS33,
    #[strum(serialize = ">>")]
    TS34,
    #[strum(serialize = ">>=")]
    TS35,
    #[strum(serialize = ">>>")]
    TS36,
    #[strum(serialize = ">>>=")]
    TS37,
    #[strum(serialize = "?")]
    TS38,
    #[strum(serialize = "@")]
    TS39,
    #[strum(serialize = "@interface")]
    TS40,
    #[strum(serialize = "[")]
    TS41,
    #[strum(serialize = "]")]
    TS42,
    #[strum(serialize = "^")]
    TS43,
    #[strum(serialize = "^=")]
    TS44,
    #[strum(serialize = "abstract")]
    TS45,
    #[strum(serialize = "assert")]
    TS46,
    BinaryIntegerLiteral,
    BooleanType,
    #[strum(serialize = "break")]
    TS47,
    #[strum(serialize = "byte")]
    TS48,
    #[strum(serialize = "case")]
    TS49,
    #[strum(serialize = "catch")]
    TS50,
    #[strum(serialize = "char")]
    TS51,
    CharacterLiteral,
    #[strum(serialize = "class")]
    TS52,
    Comment,
    #[strum(serialize = "continue")]
    TS53,
    DecimalFloatingPointLiteral,
    DecimalIntegerLiteral,
    #[strum(serialize = "default")]
    TS54,
    #[strum(serialize = "do")]
    TS55,
    #[strum(serialize = "double")]
    TS56,
    #[strum(serialize = "else")]
    TS57,
    #[strum(serialize = "enum")]
    TS58,
    #[strum(serialize = "exports")]
    TS59,
    #[strum(serialize = "extends")]
    TS60,
    False,
    #[strum(serialize = "final")]
    TS61,
    #[strum(serialize = "finally")]
    TS62,
    #[strum(serialize = "float")]
    TS63,
    #[strum(serialize = "for")]
    TS64,
    HexFloatingPointLiteral,
    HexIntegerLiteral,
    Identifier,
    #[strum(serialize = "if")]
    TS65,
    #[strum(serialize = "implements")]
    TS66,
    #[strum(serialize = "import")]
    TS67,
    #[strum(serialize = "instanceof")]
    TS68,
    #[strum(serialize = "int")]
    TS69,
    #[strum(serialize = "interface")]
    TS70,
    #[strum(serialize = "long")]
    TS71,
    #[strum(serialize = "module")]
    TS72,
    #[strum(serialize = "native")]
    TS73,
    #[strum(serialize = "new")]
    TS74,
    NullLiteral,
    OctalIntegerLiteral,
    #[strum(serialize = "open")]
    TS75,
    #[strum(serialize = "opens")]
    TS76,
    #[strum(serialize = "package")]
    TS77,
    #[strum(serialize = "private")]
    TS78,
    #[strum(serialize = "protected")]
    TS79,
    #[strum(serialize = "provides")]
    TS80,
    #[strum(serialize = "public")]
    TS81,
    #[strum(serialize = "record")]
    TS82,
    #[strum(serialize = "requires")]
    TS83,
    #[strum(serialize = "return")]
    TS84,
    #[strum(serialize = "short")]
    TS85,
    #[strum(serialize = "static")]
    TS86,
    #[strum(serialize = "strictfp")]
    TS87,
    StringLiteral,
    Super,
    #[strum(serialize = "switch")]
    TS88,
    #[strum(serialize = "synchronized")]
    TS89,
    This,
    #[strum(serialize = "throw")]
    TS90,
    #[strum(serialize = "throws")]
    TS91,
    #[strum(serialize = "to")]
    TS92,
    #[strum(serialize = "transient")]
    TS93,
    #[strum(serialize = "transitive")]
    TS94,
    True,
    #[strum(serialize = "try")]
    TS95,
    TypeIdentifier,
    #[strum(serialize = "uses")]
    TS96,
    VoidType,
    #[strum(serialize = "volatile")]
    TS97,
    #[strum(serialize = "while")]
    TS98,
    #[strum(serialize = "with")]
    TS99,
    #[strum(serialize = "yield")]
    TS100,
    #[strum(serialize = "{")]
    TS101,
    #[strum(serialize = "|")]
    TS102,
    #[strum(serialize = "|=")]
    TS103,
    #[strum(serialize = "||")]
    TS104,
    #[strum(serialize = "}")]
    TS105,
    #[strum(serialize = "~")]
    TS106,
    File,
    #[strum(serialize = "ERROR")]
    Error,
}

impl Type {
    pub fn is_literal(&self) -> bool {
        match self {
            Self::Literal => true,
            Self::True => true,
            Self::False => true,
            Self::OctalIntegerLiteral => true,
            Self::BinaryIntegerLiteral => true,
            Self::DecimalIntegerLiteral => true,
            Self::HexFloatingPointLiteral => true,
            Self::DecimalFloatingPointLiteral => true,
            Self::ClassLiteral => true,
            Self::StringLiteral => true,
            Self::CharacterLiteral => true,
            Self::HexIntegerLiteral => true,
            Self::NullLiteral => true,
            _ => false,
        }
    }
    pub fn literal_type(&self) -> &str {
        // TODO make the difference btw int/long and float/dooble
        match self {
            Self::Literal => panic!(),
            Self::True => "boolean",
            Self::False => "boolean",
            Self::OctalIntegerLiteral => "int",
            Self::BinaryIntegerLiteral => "int",
            Self::DecimalIntegerLiteral => "int",
            Self::HexFloatingPointLiteral => "float",
            Self::DecimalFloatingPointLiteral => "float",
            Self::HexIntegerLiteral => "float",
            // Self::ClassLiteral => "class",
            Self::StringLiteral => "String",
            Self::CharacterLiteral => "char",
            Self::NullLiteral => "null",
            _ => panic!(),
        }
    }
    pub fn is_primitive(&self) -> bool {
        match self {
            Self::BooleanType => true,
            Self::VoidType => true,
            Self::FloatingPointType => true,
            Self::IntegralType => true,
            _ => false,
        }
    }
    // pub fn primitive_to_str(&self) -> &str {
    //     match self {
    //         Self::BooleanType => "boolean",
    //         Self::VoidType => "void",
    //         Self::FloatingPointType => "float",
    //         Self::IntegralType => "int",
    //         _ => panic!(),
    //     }
    // }
    pub fn is_identifier(&self) -> bool {
        match self {
            Self::Identifier => true,
            Self::TypeIdentifier => true,
            Self::ScopedIdentifier => true,
            Self::ScopedTypeIdentifier => true,
            _ => false,
        }
    }
    pub fn is_instance_ref(&self) -> bool {
        match self {
            Self::This => true,
            Self::Super => true,
            _ => false,
        }
    }
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
pub trait NodeStore<'a, IdN, D> {
    fn resolve(&'a self, id: &IdN) -> D;

    // fn size(&self, id: &T::TreeId) -> usize;
    // fn height(&self, id: &T::TreeId) -> usize;
}

pub trait NodeStoreMut<'a, T: Stored, D>: NodeStore<'a, T::TreeId, D> {
    // fn get_or_insert(&mut self, node: T) -> T::TreeId;
}

pub trait VersionedNodeStore<'a, IdN: Eq+Clone, D>: NodeStore<'a, IdN, D>
where
{
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

    fn resolve(&self, id: &Self::I) -> &L;
}
