use std::borrow::Borrow;
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;

use num::PrimInt;
use num::ToPrimitive;
use strum_macros::Display;
use strum_macros::EnumString;

pub trait HashKind {
    fn structural() -> Self;
    fn label() -> Self;
}

/// for now the types are shared between all languages
#[derive(Debug, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
#[allow(non_camel_case_types)]
#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    MavenDirectory,
    Directory,
    // FileName,
    Spaces,
    // File,
    // xml_File,
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
    Throws, // TODO change rule in grammar to throws_modifier
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
    TS91, // TODO check this keyword as it collides with a grammar rule
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
    xml_CDSect,
    xml_CData,
    xml_CDStart,
    xml_CDEnd,
    xml_Text,
    xml_Sep1,
    xml_Sep2,
    xml_Sep3,
    xml_Comment,
    xml_DefaultDecl,
    xml_ETag,
    xml_EmptyElemTag,
    xml_EncodingDecl,
    xml_EntityRef,
    xml_EntityValue,
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
    pub fn is_fork(&self) -> bool {
        match self {
            Self::TernaryExpression => true,
            Self::IfStatement => true,
            Self::ForStatement => true,
            Self::EnhancedForStatement => true,
            Self::WhileStatement => true,
            Self::CatchClause => true,
            Self::SwitchLabel => true,
            Self::TryStatement => true,
            Self::TryWithResourcesStatement => true,
            Self::DoStatement => true,
            _ => false,
        }
    }

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

    pub fn is_file(&self) -> bool {
        self == &Type::Program || self == &Type::xml_SourceFile
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

pub trait AsTreeRef<T> {
    fn as_tree_ref(&self) -> T;
}

pub trait Stored: Node {
    type TreeId: Eq;
}

pub trait Typed {
    type Type: Eq + Hash + Copy; // todo try remove Hash and copy
    fn get_type(&self) -> Self::Type;
}

// impl<T, A: Allocator> ops::Deref for Vec<T, A> {
//     type Target = [T];

//     #[inline]
//     fn deref(&self) -> &[T] {
//         unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
//     }
// }
pub trait WithChildren: Node + Stored
// where
//     <Self::Children as std::ops::Deref>::Target: std::ops::Index<<Self as WithChildren>::ChildIdx, Output = <Self as Stored>::TreeId>
//         + Sized,
{
    type ChildIdx: PrimInt;
    type Children<'a>: Children<Self::ChildIdx, Self::TreeId> + ?Sized
    where
        Self: 'a;
    // type Children<'a>: std::ops::Index<Self::ChildIdx, Output = Self::TreeId> + IntoIterator<Item = Self::TreeId>
    // where
    //     Self: 'a;

    fn child_count(&self) -> Self::ChildIdx;
    fn child(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId>;
    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId>;
    fn children(&self) -> Option<&Self::Children<'_>>;
    // unsafe fn children_unchecked(&self) -> <Self::Children as std::ops::Deref>::Target
    // where
    //     <Self::Children as std::ops::Deref>::Target: std::ops::Index<<Self as WithChildren>::ChildIdx, Output = <Self as Stored>::TreeId>
    //         + Sized;
    // fn get_children_cpy(&self) -> Self::Children;
}
pub trait IterableChildren<T> {
    type ChildrenIter<'a>: Iterator<Item = &'a T>
    where
        T: 'a,
        Self: 'a;
    fn iter_children(&self) -> Self::ChildrenIter<'_>;
    fn is_empty(&self) -> bool;
}

pub trait Children<IdX, T>: std::ops::Index<IdX, Output = T> + IterableChildren<T> {
    fn child_count(&self) -> IdX;
    fn get(&self, i: IdX) -> Option<&T>;
    fn rev(&self, i: IdX) -> Option<&T>;
    fn after(&self, i: IdX) -> &Self;
    fn before(&self, i: IdX) -> &Self;
    fn between(&self, start: IdX, end: IdX) -> &Self;
    fn inclusive(&self, start: IdX, end: IdX) -> &Self;
}

// pub trait AsSlice<'a, IdX, T: 'a> {
//     type Slice: std::ops::Index<IdX, Output = [T]> + ?Sized;

//     fn as_slice(&self) -> &Self::Slice;
// }

impl<T> IterableChildren<T> for [T] {
    type ChildrenIter<'a> = core::slice::Iter<'a, T> where T: 'a;

    fn iter_children(&self) -> Self::ChildrenIter<'_> {
        <[T]>::iter(&self)
    }

    fn is_empty(&self) -> bool {
        <[T]>::is_empty(&self)
    }
}

impl<IdX: num::NumCast, T> Children<IdX, T> for [T]
where
    IdX: std::slice::SliceIndex<[T], Output = T>,
{
    fn child_count(&self) -> IdX {
        IdX::from(<[T]>::len(&self)).unwrap()
        // num::cast::<_, IdX>(<[T]>::len(&self)).unwrap()
    }

    fn get(&self, i: IdX) -> Option<&T> {
        self.get(i.to_usize()?)
    }

    fn rev(&self, idx: IdX) -> Option<&T> {
        let c = <[T]>::len(&self);
        let c = c.checked_sub(idx.to_usize()?.checked_add(1)?)?;
        self.get(c.to_usize()?)
    }

    fn after(&self, i: IdX) -> &Self {
        (&self[i.to_usize().unwrap()..]).into()
    }

    fn before(&self, i: IdX) -> &Self {
        (&self[..i.to_usize().unwrap()]).into()
    }

    fn between(&self, start: IdX, end: IdX) -> &Self {
        (&self[start.to_usize().unwrap()..end.to_usize().unwrap()]).into()
    }

    fn inclusive(&self, start: IdX, end: IdX) -> &Self {
        (&self[start.to_usize().unwrap()..=end.to_usize().unwrap()]).into()
    }
}

#[repr(transparent)]
pub struct MySlice<T>(pub [T]);

impl<'a, T> From<&'a [T]> for &'a MySlice<T> {
    fn from(value: &'a [T]) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl<T> std::ops::Index<u16> for MySlice<T> {
    type Output = T;

    fn index(&self, index: u16) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl<T> std::ops::Index<u8> for MySlice<T> {
    type Output = T;

    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl<T> std::ops::Index<usize> for MySlice<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T: Clone> From<&MySlice<T>> for Vec<T> {
    fn from(value: &MySlice<T>) -> Self {
        value.0.to_vec()
    }
}

impl<T: Debug> Debug for MySlice<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T: Debug> Default for &MySlice<T> {
    fn default() -> Self {
        let r: &[T] = &[];
        r.into()
    }
}

// impl<T> std::ops::Index<core::ops::RangeTo<usize>> for MySlice<T> {
//     type Output=[T];

//     fn index(&self, index: core::ops::RangeTo<usize>) -> &Self::Output {
//         &self.0[index]
//     }
// }

// impl<T> std::ops::Index<core::ops::Range<usize>> for MySlice<T> {
//     type Output=[T];

//     fn index(&self, index: core::ops::Range<usize>) -> &Self::Output {
//         &self.0[index]
//     }
// }

// impl<IdX, T> std::ops::Index<IdX> for MySlice<T> where IdX: std::slice::SliceIndex<[T], Output = T> {
//     type Output=T;

//     fn index(&self, index: usize) -> &Self::Output {
//         &self.0[index as usize]
//     }
// }

impl<T> IterableChildren<T> for MySlice<T> {
    type ChildrenIter<'a> = core::slice::Iter<'a, T> where T: 'a;

    fn iter_children(&self) -> Self::ChildrenIter<'_> {
        <[T]>::iter(&self.0)
    }

    fn is_empty(&self) -> bool {
        <[T]>::is_empty(&self.0)
    }
}

impl<T> Children<u16, T> for MySlice<T> {
    fn child_count(&self) -> u16 {
        <[T]>::len(&self.0).to_u16().unwrap()
    }

    fn get(&self, i: u16) -> Option<&T> {
        self.0.get(usize::from(i))
    }

    fn rev(&self, idx: u16) -> Option<&T> {
        let c: u16 = self.child_count();
        let c = c.checked_sub(idx.checked_add(1)?)?;
        self.get(c)
    }

    fn after(&self, i: u16) -> &Self {
        (&self.0[i.into()..]).into()
    }

    fn before(&self, i: u16) -> &Self {
        (&self.0[..i.into()]).into()
    }

    fn between(&self, start: u16, end: u16) -> &Self {
        (&self.0[start.into()..end.into()]).into()
    }

    fn inclusive(&self, start: u16, end: u16) -> &Self {
        (&self.0[start.into()..=end.into()]).into()
    }
}

impl<T> Children<u8, T> for MySlice<T> {
    fn child_count(&self) -> u8 {
        <[T]>::len(&self.0).to_u8().unwrap()
    }

    fn get(&self, i: u8) -> Option<&T> {
        self.0.get(usize::from(i))
    }

    fn rev(&self, idx: u8) -> Option<&T> {
        let c: u8 = self.child_count();
        let c = c.checked_sub(idx.checked_add(1)?)?;
        self.get(c)
    }

    fn after(&self, i: u8) -> &Self {
        (&self.0[i.into()..]).into()
    }

    fn before(&self, i: u8) -> &Self {
        (&self.0[..i.into()]).into()
    }

    fn between(&self, start: u8, end: u8) -> &Self {
        (&self.0[start.into()..end.into()]).into()
    }

    fn inclusive(&self, start: u8, end: u8) -> &Self {
        (&self.0[start.into()..=end.into()]).into()
    }
}

// impl<'a, T: 'a> AsSlice<'a, core::ops::RangeTo<usize>, T> for MySlice<T> {
//     type Slice = MySlice<T>;

//     fn as_slice(&self) -> &Self::Slice {
//         self
//     }
// }

// impl<'a, T: 'a> AsSlice<'a, core::ops::Range<usize>, T> for MySlice<T> {
//     type Slice = MySlice<T>;

//     fn as_slice(&self) -> &Self::Slice {
//         self
//     }
// }

// fn f () {
//     let v = vec![];
//     v.get_unchecked(index)
// }
// #[repr(transparent)]
// struct Children<'a, I>(&'a [I]);

// impl<'a, I> Children<'a, I> {
//     fn new(data: Vec<u8>) -> Self {
//         Children { data }
//     }
// }

// impl<'a, I, Idx> std::ops::Index<Idx> for Children<'a, I>
// where
//     Idx: std::slice::SliceIndex<[I], Output = I>,
// {
//     type Output = I;

//     fn index(&self, index: Idx) -> &Self::Output {
//         &self.0[index]
//     }
// }

// impl<'a, I, Idx: PrimInt> std::ops::Index<Idx> for Children<'a, I> {
//     type Output = I;

//     fn index(&self, index: Idx) -> &Self::Output {
//         &self.0[index.to_usize().unwrap()]
//     }
// }

/// just to show that it is not efficient
mod owned {
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

// pub trait WithChildrenAndStore<IdN,S: NodeStore<T>> : WithChildren {
//     fn size(&self, store: &S) -> usize;
//     fn height(&self, store: &S) -> usize;
// }

pub trait WithStats {
    fn size(&self) -> usize;
    fn height(&self) -> usize;
}

pub trait WithSerialization {
    fn try_bytes_len(&self) -> Option<usize>;
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

pub trait Tree: Typed + Labeled + WithChildren
// where
//     <Self::Children as std::ops::Deref>::Target: std::ops::Index<<Self as WithChildren>::ChildIdx, Output = <Self as Stored>::TreeId>
//         + Sized,
{
    fn has_children(&self) -> bool;
    fn has_label(&self) -> bool;
    fn try_get_label<'a>(&'a self) -> Option<&'a Self::Label> {
        self.has_label().then(|| self.get_label())
    }
}
pub trait DeCompressedTree<T: PrimInt>: Tree
// where
//     <Self::Children as std::ops::Deref>::Target: std::ops::Index<<Self as WithChildren>::ChildIdx, Output = <Self as Stored>::TreeId>
//         + Sized,
{
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
// pub trait NodeStore<'a, IdN, R> {
//     fn resolve(&'a self, id: &IdN) -> R;

//     // fn size(&self, id: &T::TreeId) -> usize;
//     // fn height(&self, id: &T::TreeId) -> usize;
// }
// pub trait NodeStore2<IdN>
// where
//     for<'a> Self::R<'a>: Stored<TreeId = IdN>,
// {
//     type R<'a>
//     where
//         Self: 'a;
//     fn resolve(&self, id: &IdN) -> Self::R<'_>;
// }
pub trait GenericItem<'a> {
    type Item; //:WithChildren;
}
// pub trait NodeStore3<IdN> {
//     type R: ?Sized + for<'any> GenericItem<'any>;
//     fn resolve(&self, id: &IdN) -> <Self::R as GenericItem>::Item;
// }
// pub trait NodeStore4<'a, IdN> {
//     type R:'a;
//     fn resolve(&'a self, id: &IdN) -> Self::R;
// }
pub trait NodeStore<IdN> {
    type R<'a>
    where
        Self: 'a;
    fn resolve(&self, id: &IdN) -> Self::R<'_>;
}

pub trait DecompressedSubtree<'a, T: Stored> {
    fn decompress<S>(store: &'a S, id: &T::TreeId) -> Self
    where
        S: NodeStore<T::TreeId, R<'a> = T>;
}

pub trait DecompressibleNodeStore<IdN>: NodeStore<IdN> {
    fn decompress<'a, D: DecompressedSubtree<'a, Self::R<'a>>>(&'a self, id: &IdN) -> (&'a Self, D)
    where
        Self: Sized,
        Self::R<'a>: Stored<TreeId = IdN>,
    {
        (self, D::decompress(self, id))
    }

    fn decompress_pair<'a, D1, D2>(&'a self, id1: &IdN, id2: &IdN) -> (&'a Self, (D1, D2))
    where
        Self: Sized,
        Self::R<'a>: Stored<TreeId = IdN>,
        D1: DecompressedSubtree<'a, Self::R<'a>>,
        D2: DecompressedSubtree<'a, Self::R<'a>>,
    {
        (self, (D1::decompress(self, id1), D2::decompress(self, id2)))
    }
}

impl<IdN, S> DecompressibleNodeStore<IdN> for S where S: NodeStore<IdN> {}

// pub trait NodeStoreMut<'a, T: Stored, D: 'a>: NodeStore<'a, T::TreeId, D> {
//     fn get_or_insert(&mut self, node: T) -> T::TreeId;
// }
// pub trait NodeStoreExt<'a, T: Stored, D: 'a + Tree<TreeId = T::TreeId>>:
//     NodeStore<'a, T::TreeId, D>
// {
//     fn build_then_insert(&mut self, t: D::Type, l: D::Label, cs: Vec<T::TreeId>) -> T::TreeId;
// }

pub trait NodeStoreMut<T: Stored> {
    fn get_or_insert(&mut self, node: T) -> T::TreeId;
}
pub trait NodeStoreExt<T: Tree>
// where
//     <T::Children as std::ops::Deref>::Target:
//         std::ops::Index<<T as WithChildren>::ChildIdx, Output = <T as Stored>::TreeId> + Sized,
{
    fn build_then_insert(
        &mut self,
        i: T::TreeId,
        t: T::Type,
        l: Option<T::Label>,
        cs: Vec<T::TreeId>,
    ) -> T::TreeId;
}

pub trait VersionedNodeStore<'a, IdN>: NodeStore<IdN> {
    fn resolve_root(&self, version: (u8, u8, u8), node: IdN);
}

pub trait VersionedNodeStoreMut<'a, T: Stored>: NodeStoreMut<T>
where
    T::TreeId: Clone,
{
    // fn insert_as_root(&mut self, version: (u8, u8, u8), node: T) -> T::TreeId;
    //  {
    //     let r = self.get_or_insert(node);
    //     self.as_root(version, r.clone());
    //     r
    // }

    fn as_root(&mut self, version: (u8, u8, u8), node: T::TreeId);
}

pub type OwnedLabel = String;
pub type SlicedLabel = str;

pub trait LabelStore<L: ?Sized> {
    type I: Copy + Eq;

    fn get_or_insert<T: Borrow<L>>(&mut self, node: T) -> Self::I;

    fn get<T: Borrow<L>>(&self, node: T) -> Option<Self::I>;

    fn resolve(&self, id: &Self::I) -> &L;
}

pub trait HyperAST<'store> {
    type IdN;
    type Label;
    type T: Tree<TreeId = Self::IdN, Label = Self::Label>;
    type NS: 'store + NodeStore<Self::IdN, R<'store> = Self::T>;
    fn node_store(&self) -> &Self::NS;

    type LS: LabelStore<str, I = Self::Label>;
    fn label_store(&self) -> &Self::LS;

    fn decompress<D: DecompressedSubtree<'store, Self::T>>(
        &'store self,
        id: &Self::IdN,
    ) -> (&'store Self, D)
    where
        Self: Sized,
    {
        {
            (self, D::decompress(self.node_store(), id))
        }
    }

    fn decompress_pair<D1, D2>(
        &'store self,
        id1: &Self::IdN,
        id2: &Self::IdN,
    ) -> (&'store Self, (D1, D2))
    where
        Self: Sized,
        D1: DecompressedSubtree<'store, Self::T>,
        D2: DecompressedSubtree<'store, Self::T>,
    {
        {
            (self, (D1::decompress(self.node_store(), id1), D2::decompress(self.node_store(), id2)))
        }
    }
}

pub struct SimpleHyperAST<T, NS, LS> {
    pub node_store: NS,
    pub label_store: LS,
    pub _phantom: std::marker::PhantomData<T>,
}

impl<'store, T, NS, LS> HyperAST<'store> for SimpleHyperAST<T, NS, LS>
where
    T: Tree,
    NS: 'store + NodeStore<T::TreeId, R<'store> = T>,
    LS: LabelStore<str, I = T::Label>,
{
    type IdN = T::TreeId;

    type Label = T::Label;

    type T = T;

    type NS = NS;

    fn node_store(&self) -> &Self::NS {
        &self.node_store
    }

    type LS = LS;

    fn label_store(&self) -> &Self::LS {
        &self.label_store
    }
}

impl Type {
    pub fn parse_xml(t: &str) -> Self {
        match t {
            // "file" => Self::xml_File,
            "source_file" => Self::xml_SourceFile,
            "XMLDecl" => Self::xml_XMLDecl,
            "AttValue" => Self::xml_AttValue,
            "AttlistDecl" => Self::xml_AttlistDecl,
            "Attribute" => Self::xml_Attribute,
            "CDSect" => Self::xml_CDSect,
            "CData" => Self::xml_CData,
            "CDStart" => Self::xml_CDStart,
            "CDEnd" => Self::xml_CDEnd,
            "Text" => Self::xml_Text,
            "Sep1" => Self::xml_Sep1,
            "Sep2" => Self::xml_Sep2,
            "Sep3" => Self::xml_Sep3,
            "Comment" => Self::xml_Comment,
            "DefaultDecl" => Self::xml_DefaultDecl,
            "ETag" => Self::xml_ETag,
            "EmptyElemTag" => Self::xml_EmptyElemTag,
            "EncodingDecl" => Self::xml_EncodingDecl,
            "EntityRef" => Self::xml_EntityRef,
            "EntityValue" => Self::xml_EntityValue,
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
            Self::xml_CDSect => "CDSect",
            Self::xml_CData => "CData",
            Self::xml_CDStart => "<![CDATA",
            Self::xml_CDEnd => "]]>",
            Self::xml_Text => "Text",
            Self::xml_Sep1 => "Sep1",
            Self::xml_Sep2 => "Sep2",
            Self::xml_Sep3 => "Sep3",
            Self::xml_Comment => "Comment",
            Self::xml_DefaultDecl => "DefaultDecl",
            Self::xml_ETag => "ETag",
            Self::xml_EmptyElemTag => "EmptyElemTag",
            Self::xml_EncodingDecl => "EncodingDecl",
            Self::xml_EntityRef => "EntityRef",
            Self::xml_EntityValue => "EntityValue",
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
