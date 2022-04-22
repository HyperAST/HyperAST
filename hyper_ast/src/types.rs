use std::borrow::Borrow;
use std::hash::Hash;
use std::str::FromStr;

use num::PrimInt;
use strum_macros::EnumString;
use strum_macros::ToString;

pub trait HashKind {
    fn structural() -> Self;
    fn label() -> Self;
}

/// for now the types are shared between all languages
#[derive(Debug, EnumString, ToString)]
#[strum(serialize_all = "snake_case")]
#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    MavenDirectory,
    Directory,
    // FileName,
    Spaces,
    // File,
    xml_File,
    #[strum(serialize = "ERROR")]
    Error,

    // # autogen from java grammar
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
    EnhancedForVariable,
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
    ScopedAbsoluteIdentifier,
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
    BinaryIntegerLiteral,
    BooleanType,
    CharacterLiteral,
    Comment,
    DecimalFloatingPointLiteral,
    DecimalIntegerLiteral,
    False,
    HexFloatingPointLiteral,
    HexIntegerLiteral,
    Identifier,
    NullLiteral,
    OctalIntegerLiteral,
    StringLiteral,
    Super,
    TypeIdentifier,
    VoidType,
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
    #[strum(serialize = "class")]
    TS52,
    #[strum(serialize = "continue")]
    TS53,
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
    #[strum(serialize = "final")]
    TS61,
    #[strum(serialize = "finally")]
    TS62,
    #[strum(serialize = "float")]
    TS63,
    #[strum(serialize = "for")]
    TS64,
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
    #[strum(serialize = "switch")]
    TS88,
    #[strum(serialize = "synchronized")]
    TS89,
    This,
    #[strum(serialize = "throw")]
    TS90,
    #[strum(serialize = "throws")]
    TS91, // TODO check this keyword as it collide with a grammar rule
    #[strum(serialize = "to")]
    TS92,
    #[strum(serialize = "transient")]
    TS93,
    #[strum(serialize = "transitive")]
    TS94,
    True,
    #[strum(serialize = "try")]
    TS95,
    #[strum(serialize = "uses")]
    TS96,
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

    // from XML grammar, temporary solution to handling multiple languages
    // an advanced solution would be to make a macro that choose the memory layout
    // and where:
    // - we can add types to languages or,
    // - we can add laguages to types or,
    // - we can rename types per languages
    xml_XMLDecl,
    xml_AttValue,
    xml_AttlistDecl,
    xml_Attribute,
    xml_CData,
    xml_Comment,
    xml_DefaultDecl,
    xml_ETag,
    xml_EmptyElemTag,
    xml_EncodingDecl,
    xml_EntityRef,
    xml_Enumeration,
    xml_ExternalId,
    xml_GeDecl,
    xml_Ignore,
    xml_Mixed,
    xml_NDataDecl,
    xml_NotationDecl,
    xml_NotationType,
    xml_PEDecl,
    xml_PEReference,
    xml_Pi,
    xml_PublicId,
    xml_SDDecl,
    xml_STag,
    xml_TextDecl,
    xml_TokenizedType,
    xml_VersionInfo,
    xml_Children,
    xml_Contentspec,
    xml_Doctypedecl,
    xml_Element,
    xml_Elementdecl,
    xml_IgnoreSect,
    xml_IgnoreSectContents,
    xml_IncludeSect,
    xml_Prolog,
    xml_SourceFile,
    xml_CharData,
    xml_CharRef,
    xml_EncName,
    xml_Name,
    xml_Nmtoken,
    xml_PubidLiteral,
    xml_StringType,
    xml_SystemLiteral,
    xml_VersionNum,
    #[strum(serialize = " ")]
    xml_TS0,
    #[strum(serialize = "\"")]
    xml_TS1,
    #[strum(serialize = "#FIXED")]
    xml_TS2,
    #[strum(serialize = "#IMPLIED")]
    xml_TS3,
    #[strum(serialize = "#PCDATA")]
    xml_TS4,
    #[strum(serialize = "#REQUIRED")]
    xml_TS5,
    #[strum(serialize = "%")]
    xml_TS6,
    #[strum(serialize = "&")]
    xml_TS7,
    #[strum(serialize = "'")]
    xml_TS8,
    #[strum(serialize = "(")]
    xml_TS9,
    #[strum(serialize = ")")]
    xml_TS10,
    #[strum(serialize = ")*")]
    xml_TS11,
    #[strum(serialize = "*")]
    xml_TS12,
    #[strum(serialize = "+")]
    xml_TS13,
    #[strum(serialize = ",")]
    xml_TS14,
    #[strum(serialize = "-->")]
    xml_TS15,
    #[strum(serialize = "/>")]
    xml_TS16,
    #[strum(serialize = ";")]
    xml_TS17,
    #[strum(serialize = "<")]
    xml_TS18,
    #[strum(serialize = "<!--")]
    xml_TS19,
    #[strum(serialize = "<!ATTLIST")]
    xml_TS20,
    #[strum(serialize = "<!DOCTYPE")]
    xml_TS21,
    #[strum(serialize = "<!ELEMENT")]
    xml_TS22,
    #[strum(serialize = "<!ENTITY")]
    xml_TS23,
    #[strum(serialize = "<!NOTATION")]
    xml_TS24,
    #[strum(serialize = "<![")]
    xml_TS25,
    #[strum(serialize = "</")]
    xml_TS26,
    #[strum(serialize = "<?")]
    xml_TS27,
    #[strum(serialize = "<?xml")]
    xml_TS28,
    #[strum(serialize = "=")]
    xml_TS29,
    #[strum(serialize = ">")]
    xml_TS30,
    #[strum(serialize = "?")]
    xml_TS31,
    #[strum(serialize = "?>")]
    xml_TS32,
    #[strum(serialize = "ANY")]
    xml_TS33,
    #[strum(serialize = "EMPTY")]
    xml_TS34,
    #[strum(serialize = "ENTITIES")]
    xml_TS35,
    #[strum(serialize = "ENTITY")]
    xml_TS36,
    #[strum(serialize = "ID")]
    xml_TS37,
    #[strum(serialize = "IDREF")]
    xml_TS38,
    #[strum(serialize = "IDREFS")]
    xml_TS39,
    #[strum(serialize = "IGNORE")]
    xml_TS40,
    #[strum(serialize = "INCLUDE")]
    xml_TS41,
    #[strum(serialize = "NDATA")]
    xml_TS42,
    #[strum(serialize = "NMTOKEN")]
    xml_TS43,
    #[strum(serialize = "NMTOKENS")]
    xml_TS44,
    #[strum(serialize = "NOTATION")]
    xml_TS45,
    #[strum(serialize = "PUBLIC")]
    xml_TS46,
    #[strum(serialize = "SYSTEM")]
    xml_TS47,
    #[strum(serialize = "[")]
    xml_TS48,
    #[strum(serialize = "]")]
    xml_TS49,
    #[strum(serialize = "]]>")]
    xml_TS50,
    #[strum(serialize = "encoding")]
    xml_TS51,
    #[strum(serialize = "no")]
    xml_TS52,
    #[strum(serialize = "standalone")]
    xml_TS53,
    #[strum(serialize = "version")]
    xml_TS54,
    #[strum(serialize = "yes")]
    xml_TS55,
    #[strum(serialize = "|")]
    xml_TS56,
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
        // TODO make the difference btw int/long and float/double
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
    pub fn is_type_declaration(&self) -> bool {
        match self {
            Self::ClassDeclaration => true,
            Self::EnumDeclaration => true,
            Self::InterfaceDeclaration => true,
            Self::AnnotationTypeDeclaration => true,
            Self::EnumConstant => true, // TODO need more eval
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

    pub fn is_directory(&self) -> bool {
        self == &Type::Directory || self == &Type::MavenDirectory
    }

    pub fn is_type_body(&self) -> bool {
        self == &Type::ClassBody
            || self == &Type::InterfaceBody
            || self == &Type::AnnotationTypeBody
            || self == &Type::EnumBody
            || self == &Type::EnumBodyDeclarations
    }

    pub fn is_value_member(&self) -> bool {
        self == &Type::FieldDeclaration
        || self == &Type::ConstantDeclaration
        // || self == &Type::EnumConstant
        || self == &Type::AnnotationTypeElementDeclaration
    }

    pub fn is_executable_member(&self) -> bool {
        self == &Type::MethodDeclaration || self == &Type::ConstructorDeclaration
    }

    pub fn is_statement(&self) -> bool {
        self.is_declarative_statement()
            || self.is_structural_statement()
            || self.is_simple_statement()
            || self.is_block_related()
    }

    pub fn is_declarative_statement(&self) -> bool {
        self == &Type::LocalVariableDeclaration
            || self == &Type::TryWithResourcesStatement
            || self == &Type::CatchClause
            || self == &Type::ForStatement
            || self == &Type::EnhancedForStatement
    }

    pub fn is_structural_statement(&self) -> bool {
        self == &Type::SwitchStatement
            || self == &Type::WhileStatement
            || self == &Type::DoStatement
            || self == &Type::IfStatement
            || self == &Type::TryStatement
            || self == &Type::FinallyClause
            || self == &Type::TryWithResourcesExtendedStatement
    }

    pub fn is_block_related(&self) -> bool {
        self == &Type::StaticInitializer
            || self == &Type::ConstructorBody
            || self == &Type::Block
            || self == &Type::SwitchBlock
            || self == &Type::SwitchBlockStatementGroup
    }

    pub fn is_simple_statement(&self) -> bool {
        self == &Type::ExpressionStatement
            || self == &Type::AssertStatement
            || self == &Type::ThrowStatement
            || self == &Type::ReturnStatement
            || self == &Type::LabeledStatement
            || self == &Type::SynchronizedStatement
            || self == &Type::ContinueStatement
            || self == &Type::BreakStatement
            || self == &Type::SynchronizedStatement
    }

    pub fn is_local_declare(&self) -> bool {
        self == &Type::LocalVariableDeclaration
            || self == &Type::EnhancedForVariable
            || self == &Type::Resource
    }


    pub fn is_parameter(&self) -> bool {
        self == &Type::Resource
            || self == &Type::FormalParameter
            || self == &Type::SpreadParameter
            || self == &Type::CatchFormalParameter
            || self == &Type::TypeParameter
    }

    pub fn is_parameter_list(&self) -> bool {
        self == &Type::ResourceSpecification
            || self == &Type::FormalParameters
            || self == &Type::TypeParameters
    }

    pub fn is_argument_list(&self) -> bool {
        self == &Type::ArgumentList
            || self == &Type::TypeArguments
            || self == &Type::AnnotationArgumentList
    }

    pub fn is_expression(&self) -> bool {
        self == &Type::TernaryExpression
        || self == &Type::BinaryExpression
        || self == &Type::UnaryExpression
        || self == &Type::AssignmentExpression
        // || self == &Type::VariableDeclarator
        || self == &Type::InstanceofExpression
        || self == &Type::ArrayCreationExpression
        || self == &Type::ObjectCreationExpression
        || self == &Type::LambdaExpression
        || self == &Type::CastExpression
        || self == &Type::UpdateExpression
        || self == &Type::ParenthesizedExpression
        || self == &Type::MethodInvocation
        || self == &Type::MethodReference
        || self == &Type::ExplicitConstructorInvocation
        || self == &Type::ClassLiteral
        || self == &Type::FieldAccess
        || self == &Type::ArrayAccess
    }
    // pub fn is_type_propagating_expression(&self) -> bool {
    //     self == &Type::TernaryExpression
    //     || self == &Type::BinaryExpression
    //     || self == &Type::UnaryExpression
    //     || self == &Type::AssignmentExpression
    //     || self == &Type::ArrayCreationExpression
    //     || self == &Type::ObjectCreationExpression
    //     // || self == &Type::LambdaExpression
    //     || self == &Type::CastExpression
    //     || self == &Type::UpdateExpression
    //     || self == &Type::ParenthesizedExpression
    //     || self == &Type::MethodInvocation
    //     // || self == &Type::MethodReference
    //     || self == &Type::ExplicitConstructorInvocation
    //     // || self == &Type::ClassLiteral
    //     || self == &Type::FieldAccess
    //     || self == &Type::ArrayAccess
    // }
}

impl Type {
    pub fn new(kind: &str) -> Type {
        Type::from_str(kind)
            .map_err(|x| format!("{} for '{}'", x, kind))
            .unwrap()
    }
}

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
mod Owned {
    use std::cell::{Ref, RefMut};

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
    fn try_get_label<'a>(&'a self) -> Option<&'a Self::Label> {
        self.has_label().then(|| self.get_label())
    }
}
pub trait DeCompressedTree<T: PrimInt>: Tree {
    fn get_parent(&self) -> T;
    // fn has_parent(&self) -> bool;
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

impl Type {
    pub fn parse_xml(t: &str) -> Self {
        match t {
            "file" => Self::xml_File,
            "source_file" => Self::xml_SourceFile,
            "XMLDecl" => Self::xml_XMLDecl,
            "AttValue" => Self::xml_AttValue,
            "AttlistDecl" => Self::xml_AttlistDecl,
            "Attribute" => Self::xml_Attribute,
            "CData" => Self::xml_CData,
            "Comment" => Self::xml_Comment,
            "DefaultDecl" => Self::xml_DefaultDecl,
            "ETag" => Self::xml_ETag,
            "EmptyElemTag" => Self::xml_EmptyElemTag,
            "EncodingDecl" => Self::xml_EncodingDecl,
            "EntityRef" => Self::xml_EntityRef,
            "Enumeration" => Self::xml_Enumeration,
            "ExternalId" => Self::xml_ExternalId,
            "GeDecl" => Self::xml_GeDecl,
            "Ignore" => Self::xml_Ignore,
            "Mixed" => Self::xml_Mixed,
            "NDataDecl" => Self::xml_NDataDecl,
            "NotationDecl" => Self::xml_NotationDecl,
            "NotationType" => Self::xml_NotationType,
            "PEDecl" => Self::xml_PEDecl,
            "PEReference" => Self::xml_PEReference,
            "Pi" => Self::xml_Pi,
            "PublicId" => Self::xml_PublicId,
            "SDDecl" => Self::xml_SDDecl,
            "STag" => Self::xml_STag,
            "TextDecl" => Self::xml_TextDecl,
            "TokenizedType" => Self::xml_TokenizedType,
            "VersionInfo" => Self::xml_VersionInfo,
            "XMLDecl" => Self::xml_XMLDecl,
            "children" => Self::xml_Children,
            "contentspec" => Self::xml_Contentspec,
            "doctypedecl" => Self::xml_Doctypedecl,
            "element" => Self::xml_Element,
            "elementdecl" => Self::xml_Elementdecl,
            "ignoreSect" => Self::xml_IgnoreSect,
            "IgnoreSectContents" => Self::xml_IgnoreSectContents,
            "includeSect" => Self::xml_IncludeSect,
            "prolog" => Self::xml_Prolog,
            "CharData" => Self::xml_CharData,
            "CharRef" => Self::xml_CharRef,
            "EncName" => Self::xml_EncName,
            "Name" => Self::xml_Name,
            "Nmtoken" => Self::xml_Nmtoken,
            "PubidLiteral" => Self::xml_PubidLiteral,
            "StringType" => Self::xml_StringType,
            "SystemLiteral" => Self::xml_SystemLiteral,
            "VersionNum" => Self::xml_VersionNum,
            " " => Self::xml_TS0,
            "\"" => Self::xml_TS1,
            "#FIXED" => Self::xml_TS2,
            "#IMPLIED" => Self::xml_TS3,
            "#PCDATA" => Self::xml_TS4,
            "#REQUIRED" => Self::xml_TS5,
            "%" => Self::xml_TS6,
            "&" => Self::xml_TS7,
            "'" => Self::xml_TS8,
            "(" => Self::xml_TS9,
            ")" => Self::xml_TS10,
            ")*" => Self::xml_TS11,
            "*" => Self::xml_TS12,
            "+" => Self::xml_TS13,
            "," => Self::xml_TS14,
            "-->" => Self::xml_TS15,
            "/>" => Self::xml_TS16,
            ";" => Self::xml_TS17,
            "<" => Self::xml_TS18,
            "<!--" => Self::xml_TS19,
            "<!ATTLIST" => Self::xml_TS20,
            "<!DOCTYPE" => Self::xml_TS21,
            "<!ELEMENT" => Self::xml_TS22,
            "<!ENTITY" => Self::xml_TS23,
            "<!NOTATION" => Self::xml_TS24,
            "<![" => Self::xml_TS25,
            "</" => Self::xml_TS26,
            "<?" => Self::xml_TS27,
            "<?xml" => Self::xml_TS28,
            "=" => Self::xml_TS29,
            ">" => Self::xml_TS30,
            "?" => Self::xml_TS31,
            "?>" => Self::xml_TS32,
            "ANY" => Self::xml_TS33,
            "EMPTY" => Self::xml_TS34,
            "ENTITIES" => Self::xml_TS35,
            "ENTITY" => Self::xml_TS36,
            "ID" => Self::xml_TS37,
            "IDREF" => Self::xml_TS38,
            "IDREFS" => Self::xml_TS39,
            "IGNORE" => Self::xml_TS40,
            "INCLUDE" => Self::xml_TS41,
            "NDATA" => Self::xml_TS42,
            "NMTOKEN" => Self::xml_TS43,
            "NMTOKENS" => Self::xml_TS44,
            "NOTATION" => Self::xml_TS45,
            "PUBLIC" => Self::xml_TS46,
            "SYSTEM" => Self::xml_TS47,
            "[" => Self::xml_TS48,
            "]" => Self::xml_TS49,
            "]]>" => Self::xml_TS50,
            "encoding" => Self::xml_TS51,
            "no" => Self::xml_TS52,
            "standalone" => Self::xml_TS53,
            "version" => Self::xml_TS54,
            "yes" => Self::xml_TS55,
            "|" => Self::xml_TS56,
            "ERROR" => Self::Error,
            x => panic!("{}", x),
        }
    }
    pub fn serialize_xml(&self) -> &str {
        match self {
            Self::xml_XMLDecl => "XMLDecl",
            Self::xml_AttValue => "AttValue",
            Self::xml_AttlistDecl => "AttlistDecl",
            Self::xml_Attribute => "Attribute",
            Self::xml_CData => "CData",
            Self::xml_Comment => "Comment",
            Self::xml_DefaultDecl => "DefaultDecl",
            Self::xml_ETag => "ETag",
            Self::xml_EmptyElemTag => "EmptyElemTag",
            Self::xml_EncodingDecl => "EncodingDecl",
            Self::xml_EntityRef => "EntityRef",
            Self::xml_Enumeration => "Enumeration",
            Self::xml_ExternalId => "ExternalId",
            Self::xml_GeDecl => "GeDecl",
            Self::xml_Ignore => "Ignore",
            Self::xml_Mixed => "Mixed",
            Self::xml_NDataDecl => "NDataDecl",
            Self::xml_NotationDecl => "NotationDecl",
            Self::xml_NotationType => "NotationType",
            Self::xml_PEDecl => "PEDecl",
            Self::xml_PEReference => "PEReference",
            Self::xml_Pi => "Pi",
            Self::xml_PublicId => "PublicId",
            Self::xml_SDDecl => "SDDecl",
            Self::xml_STag => "STag",
            Self::xml_TextDecl => "TextDecl",
            Self::xml_TokenizedType => "TokenizedType",
            Self::xml_VersionInfo => "VersionInfo",
            Self::xml_XMLDecl => "XMLDecl",
            Self::xml_Children => "children",
            Self::xml_Contentspec => "contentspec",
            Self::xml_Doctypedecl => "doctypedecl",
            Self::xml_Element => "element",
            Self::xml_Elementdecl => "elementdecl",
            Self::xml_IgnoreSect => "ignoreSect",
            Self::xml_IgnoreSectContents => "IgnoreSectContents",
            Self::xml_IncludeSect => "includeSect",
            Self::xml_Prolog => "prolog",
            Self::xml_SourceFile => "SourceFile",
            Self::xml_CharData => "CharData",
            Self::xml_CharRef => "CharRef",
            Self::xml_EncName => "EncName",
            Self::xml_Name => "Name",
            Self::xml_Nmtoken => "Nmtoken",
            Self::xml_PubidLiteral => "PubidLiteral",
            Self::xml_StringType => "StringType",
            Self::xml_SystemLiteral => "SystemLiteral",
            Self::xml_VersionNum => "VersionNum",
            Self::xml_TS0 => " ",
            Self::xml_TS1 => "\"",
            Self::xml_TS2 => "#FIXED",
            Self::xml_TS3 => "#IMPLIED",
            Self::xml_TS4 => "#PCDATA",
            Self::xml_TS5 => "#REQUIRED",
            Self::xml_TS6 => "%",
            Self::xml_TS7 => "&",
            Self::xml_TS8 => "'",
            Self::xml_TS9 => "(",
            Self::xml_TS10 => ")",
            Self::xml_TS11 => ")*",
            Self::xml_TS12 => "*",
            Self::xml_TS13 => "+",
            Self::xml_TS14 => ",",
            Self::xml_TS15 => "-->",
            Self::xml_TS16 => "/>",
            Self::xml_TS17 => ";",
            Self::xml_TS18 => "<",
            Self::xml_TS19 => "<!--",
            Self::xml_TS20 => "<!ATTLIST",
            Self::xml_TS21 => "<!DOCTYPE",
            Self::xml_TS22 => "<!ELEMENT",
            Self::xml_TS23 => "<!ENTITY",
            Self::xml_TS24 => "<!NOTATION",
            Self::xml_TS25 => "<![",
            Self::xml_TS26 => "</",
            Self::xml_TS27 => "<?",
            Self::xml_TS28 => "<?xml",
            Self::xml_TS29 => "=",
            Self::xml_TS30 => ">",
            Self::xml_TS31 => "?",
            Self::xml_TS32 => "?>",
            Self::xml_TS33 => "ANY",
            Self::xml_TS34 => "EMPTY",
            Self::xml_TS35 => "ENTITIES",
            Self::xml_TS36 => "ENTITY",
            Self::xml_TS37 => "ID",
            Self::xml_TS38 => "IDREF",
            Self::xml_TS39 => "IDREFS",
            Self::xml_TS40 => "IGNORE",
            Self::xml_TS41 => "INCLUDE",
            Self::xml_TS42 => "NDATA",
            Self::xml_TS43 => "NMTOKEN",
            Self::xml_TS44 => "NMTOKENS",
            Self::xml_TS45 => "NOTATION",
            Self::xml_TS46 => "PUBLIC",
            Self::xml_TS47 => "SYSTEM",
            Self::xml_TS48 => "[",
            Self::xml_TS49 => "]",
            Self::xml_TS50 => "]]>",
            Self::xml_TS51 => "encoding",
            Self::xml_TS52 => "no",
            Self::xml_TS53 => "standalone",
            Self::xml_TS54 => "version",
            Self::xml_TS55 => "yes",
            Self::xml_TS56 => "|",
            Self::Error => "ERROR",
            x => panic!("{:?}", x),
        }
    }
}
