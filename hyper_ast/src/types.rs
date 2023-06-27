use std::borrow::Borrow;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

use num::PrimInt;
use num::ToPrimitive;
use strum::IntoEnumIterator;
use strum_macros::AsRefStr;
use strum_macros::Display;
use strum_macros::EnumCount;
use strum_macros::EnumIter;
use strum_macros::EnumString;

pub trait HashKind {
    fn structural() -> Self;
    fn label() -> Self;
}

impl Type {
    pub fn it() -> impl Iterator<Item = Type> {
        Type::iter()
    }
    pub fn parse(s: &str) -> Result<Type, strum::ParseError> {
        Type::from_str(s)
    }
}

#[repr(transparent)]
pub struct T(u16);

#[repr(u16)]
pub enum T2 {
    Java(u16),
    Cpp(u16),
}

// pub trait Lang {
//     type Factory;
//     type Type;
// }

trait TypeFactory {
    fn new() -> Self
    where
        Self: Sized;
}

macro_rules! make_type_store {
    ($kw:ty, $sh:ty, $($a:ident($l:ty)),* $(,)?) => {

        #[repr(u16)]
        pub enum CustomTypeStore {$(
            $a(u16),
        )*}

        impl CustomTypeStore {
            // fn lang<L: Lang>(&self) -> Option<L> {
            //     todo!()
            // }
            fn eq_keyword(kw: &$kw) -> bool {
                todo!()
            }
            fn eq_shared(kw: &$sh) -> bool {
                todo!()
            }
        }
    };
}

make_type_store!(Keyword, Shared, Java(java::Language), Cpp(cpp::Language),);

#[derive(Debug, Hash, Eq, PartialEq, EnumString, AsRefStr, EnumIter, EnumCount, Display)]
#[strum(serialize_all = "snake_case")]
enum Abstract {
    Expression,
    Statement,
    Executable,
    Declaration,
    Literal,
}

// only keywords (leafs with a specific unique serialized form)
// and concrete types (concrete rules) are stored.
// abtract types are found by looking at the actual node ie. metadata
// due to possibilities of concrete rules of being part of multiple abtract rules

// lang + type // no abstract stuff

mod abst {
    use std::hash::Hash;

    trait T: Hash + PartialEq + Eq {}
}

trait KeywordProvider: Sized {
    fn parse(&self, s: &str) -> Option<Self>;
    fn as_str(&'static self) -> &'static str;
    fn len(&self) -> usize;
}

/// only contains keywords such as
#[derive(Debug, EnumString, AsRefStr, EnumIter, EnumCount, Display)]
#[strum(serialize_all = "snake_case")]
#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    // While,
    // For,
    // #[strum(serialize = ";")]
    // SemiColon,
    // #[strum(serialize = ".")]
    // Dot,
    // #[strum(serialize = "{")]
    // LeftCurly,
    // #[strum(serialize = "}")]
    // RightCurly,
}

impl KeywordProvider for Keyword {
    fn parse(&self, s: &str) -> Option<Self> {
        Keyword::from_str(s).ok()
    }

    fn as_str(&'static self) -> &'static str {
        Keyword::as_ref(&self)
    }

    fn len(&self) -> usize {
        <Keyword as strum::EnumCount>::COUNT
    }
}

#[derive(Debug, EnumString, AsRefStr, EnumIter, EnumCount, Display)]
#[strum(serialize_all = "snake_case")]
#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub enum Shared {
    Comment,
    // ExpressionStatement,
    // ReturnStatement,
    // TryStatement,
    Identifier,
    TypeDeclaration,
    Other,
    // WARN do not include Abtract type/rules (should go in Abstract) ie.
    // Expression,
    // Statement,
}

mod polyglote {
    /// has statements
    struct Block;
    /// has a name
    struct Member;
}

// WARN order of fields matter in java for instantiation
// stuff where order does not matter should be sorted before erasing anything

pub enum TypeMapElement<Concrete, Abstract> {
    Keyword(Keyword),
    Concrete(Concrete),
    Abstract(Abstract),
}

pub enum ConvertResult<Concrete, Abstract> {
    Keyword(Keyword),
    Concrete(Concrete),
    Abstract(Abstract),
    Missing,
}

mod macro_test {
    macro_rules! parse_unitary_variants {
        (@as_expr $e:expr) => {$e};
        (@as_item $($i:item)+) => {$($i)+};

        // Exit rules.
        (
            @collect_unitary_variants ($callback:ident ( $($args:tt)* )),
            ($(,)*) -> ($($var_names:ident,)*)
        ) => {
            parse_unitary_variants! {
                @as_expr
                $callback!{ $($args)* ($($var_names),*) }
            }
        };

        (
            @collect_unitary_variants ($callback:ident { $($args:tt)* }),
            ($(,)*) -> ($($var_names:ident,)*)
        ) => {
            parse_unitary_variants! {
                @as_item
                $callback!{ $($args)* ($($var_names),*) }
            }
        };

        // Consume an attribute.
        (
            @collect_unitary_variants $fixed:tt,
            (#[$_attr:meta] $($tail:tt)*) -> ($($var_names:tt)*)
        ) => {
            parse_unitary_variants! {
                @collect_unitary_variants $fixed,
                ($($tail)*) -> ($($var_names)*)
            }
        };

        // Handle a variant, optionally with an with initialiser.
        (
            @collect_unitary_variants $fixed:tt,
            ($var:ident $(= $_val:expr)*, $($tail:tt)*) -> ($($var_names:tt)*)
        ) => {
            parse_unitary_variants! {
                @collect_unitary_variants $fixed,
                ($($tail)*) -> ($($var_names)* $var,)
            }
        };

        // Abort on variant with a payload.
        (
            @collect_unitary_variants $fixed:tt,
            ($var:ident $_struct:tt, $($tail:tt)*) -> ($($var_names:tt)*)
        ) => {
            const _error: () = "cannot parse unitary variants from enum with non-unitary variants";
        };

        // Entry rule.
        (enum $name:ident {$($body:tt)*} => $callback:ident $arg:tt) => {
            parse_unitary_variants! {
                @collect_unitary_variants
                ($callback $arg), ($($body)*,) -> ()
            }
        };
    }

    macro_rules! coucou {
        ( f(C, D)) => {
            struct B {}
        };
    }
    parse_unitary_variants! {
        enum A {
            C,D,
        } => coucou{ f}
    }
}

macro_rules! make_type {
    (
        Keyword {$(
            $(#[$km:meta])*
            $ka:ident
        ),* $(,)?}
        Concrete {$(
            $(#[$cm:meta])*
            $ca:ident$({$($cl:expr),+ $(,)*})?$(($($co:ident),+ $(,)*))?$([$($cx:ident),+ $(,)*])?
        ),* $(,)?}
        WithFields {$(
            $(#[$wm:meta])*
            $wa:ident{$($wb:tt)*}
        ),* $(,)?}
        Abstract {$(
            $(#[$am:meta])*
            $aa:ident($($ab:ident),* $(,)?)
        ),* $(,)?}
    ) => {

        #[derive(Debug, EnumString, AsRefStr, EnumIter, EnumCount, Display)]
        #[strum(serialize_all = "snake_case")]
        #[derive(Hash, Clone, Copy, PartialEq, Eq)]
        pub enum Type {
            // Keywords
        $(
            $( #[$km] )*
            $ka,
        )*
            // Concrete
        $(
            $ca,
        )*
            // WithFields
        $(
            $( #[$wm] )*
            $wa,
        )*
        }
        enum Abstract {
            $(
                $aa,
            )*
        }

        // #[strum(props(Teacher="Ms.Frizzle", Room="201"))]
        // pub enum WithFields {}

        pub struct Factory {
            map: Box<[u16]>,
        }

        pub struct Language;
        // impl super::Lang for Language {
        //     type Factory = Factory;
        //     type Type = Type;
        // }
    };
}

pub mod java {
    use super::*;
    // pub struct TypeMap(Vec<TypeMapElement<ConcreteType, AbstractType>>);
    // impl TypeMap {
    //     pub fn is(&self, t: &T) -> bool {
    //         !matches!(self.convert(t), ConvertResult::Missing)
    //     }
    //     pub fn convert(&self, t: &T) -> ConvertResult<ConcreteType, AbstractType> {
    //         todo!()
    //     }
    // }
    // pub fn parse(t: &str) -> ConvertResult<ConcreteType, AbstractType> {
    //     todo!()
    // }
    pub enum Field {
        Name,
        Body,
        Expression,
        Condition,
        Then,
        Else,
        Block,
        Type,
    }

    make_type! {
        Keyword{
            While,
            For,
            Public,
            Private,
            Protected,
            #[strum(serialize = ";")]
            SemiColon,
            #[strum(serialize = ".")]
            Dot,
            #[strum(serialize = "{")]
            LeftCurly,
            #[strum(serialize = "}")]
            RightCurly,
            #[strum(serialize = "(")]
            LeftParen,
            #[strum(serialize = ")")]
            RightParen,
            #[strum(serialize = "[")]
            LeftBracket,
            #[strum(serialize = "]")]
            RightBracket,
        }
        Concrete {
            Comment{r"//.\*$",r"/\*.*\*/"},
            Identifier{r"[a-zA-Z].*"},
            ExpressionStatement(Statement, Semicolon),
            ReturnStatement(Return, Expression, Semicolon),
            TryStatement(Try, Paren, Block),
        }
        WithFields {
            Class {
                name(Identifier),
                body(ClassBody),
            },
            Interface {
                name(Identifier),
                body(InterfaceBody),
            },
        }
        Abstract {
            Statement(
                StatementExpression,
                TryStatement,
            ),
            Expression(
                BinaryExpression,
                UnaryExpression,
            ),
        }
    }
}

mod cpp {
    pub struct Factory {}
}

mod ts {}

/// for now the types are shared between all languages
#[derive(Debug, EnumString, EnumIter, Display)]
#[strum(serialize_all = "snake_case")]
#[allow(non_camel_case_types)]
#[derive(Hash, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub(crate) enum Type {
    MakeDirectory,
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
    // cpp types
    #[strum(serialize = "\n")]
    cpp_TS0,
    #[strum(serialize = "!")]
    cpp_TS1,
    #[strum(serialize = "!=")]
    cpp_TS2,
    #[strum(serialize = "\"")]
    cpp_TS3,
    #[strum(serialize = "\"\"")]
    cpp_TS4,
    #[strum(serialize = "#define")]
    cpp_TS5,
    #[strum(serialize = "#elif")]
    cpp_TS6,
    #[strum(serialize = "#else")]
    cpp_TS7,
    #[strum(serialize = "#endif")]
    cpp_TS8,
    #[strum(serialize = "#if")]
    cpp_TS9,
    #[strum(serialize = "#ifdef")]
    cpp_TS10,
    #[strum(serialize = "#ifndef")]
    cpp_TS11,
    #[strum(serialize = "#include")]
    cpp_TS12,
    #[strum(serialize = "%")]
    cpp_TS13,
    #[strum(serialize = "%=")]
    cpp_TS14,
    #[strum(serialize = "&")]
    cpp_TS15,
    #[strum(serialize = "&&")]
    cpp_TS16,
    #[strum(serialize = "&=")]
    cpp_TS17,
    #[strum(serialize = "'")]
    cpp_TS18,
    #[strum(serialize = "(")]
    cpp_TS19,
    #[strum(serialize = "()")]
    cpp_TS20,
    #[strum(serialize = ")")]
    cpp_TS21,
    #[strum(serialize = "*")]
    cpp_TS22,
    #[strum(serialize = "*=")]
    cpp_TS23,
    #[strum(serialize = "+")]
    cpp_TS24,
    #[strum(serialize = "++")]
    cpp_TS25,
    #[strum(serialize = "+=")]
    cpp_TS26,
    #[strum(serialize = ",")]
    cpp_TS27,
    #[strum(serialize = "-")]
    cpp_TS28,
    #[strum(serialize = "--")]
    cpp_TS29,
    #[strum(serialize = "-=")]
    cpp_TS30,
    #[strum(serialize = "->")]
    cpp_TS31,
    #[strum(serialize = "->*")]
    cpp_TS32,
    #[strum(serialize = ".")]
    cpp_TS33,
    #[strum(serialize = "...")]
    cpp_TS34,
    #[strum(serialize = "/")]
    cpp_TS35,
    #[strum(serialize = "/=")]
    cpp_TS36,
    #[strum(serialize = ":")]
    cpp_TS37,
    #[strum(serialize = "::")]
    cpp_TS38,
    #[strum(serialize = ";")]
    cpp_TS39,
    #[strum(serialize = "<")]
    cpp_TS40,
    #[strum(serialize = "<<")]
    cpp_TS41,
    #[strum(serialize = "<<=")]
    cpp_TS42,
    #[strum(serialize = "<=")]
    cpp_TS43,
    #[strum(serialize = "=")]
    cpp_TS44,
    #[strum(serialize = "==")]
    cpp_TS45,
    #[strum(serialize = ">")]
    cpp_TS46,
    #[strum(serialize = ">=")]
    cpp_TS47,
    #[strum(serialize = ">>")]
    cpp_TS48,
    #[strum(serialize = ">>=")]
    cpp_TS49,
    #[strum(serialize = "?")]
    cpp_TS50,
    #[strum(serialize = "L\"")]
    cpp_TS51,
    #[strum(serialize = "L'")]
    cpp_TS52,
    #[strum(serialize = "U\"")]
    cpp_TS53,
    #[strum(serialize = "U'")]
    cpp_TS54,
    #[strum(serialize = "[")]
    cpp_TS55,
    #[strum(serialize = "[[")]
    cpp_TS56,
    #[strum(serialize = "[]")]
    cpp_TS57,
    #[strum(serialize = "]")]
    cpp_TS58,
    #[strum(serialize = "]]")]
    cpp_TS59,
    #[strum(serialize = "^")]
    cpp_TS60,
    #[strum(serialize = "^=")]
    cpp_TS61,
    #[strum(serialize = "_Atomic")]
    cpp_TS62,
    #[strum(serialize = "__attribute__")]
    cpp_TS63,
    #[strum(serialize = "__based")]
    cpp_TS64,
    #[strum(serialize = "__cdecl")]
    cpp_TS65,
    #[strum(serialize = "__clrcall")]
    cpp_TS66,
    #[strum(serialize = "__declspec")]
    cpp_TS67,
    #[strum(serialize = "__fastcall")]
    cpp_TS68,
    #[strum(serialize = "__stdcall")]
    cpp_TS69,
    #[strum(serialize = "__thiscall")]
    cpp_TS70,
    #[strum(serialize = "__unaligned")]
    cpp_TS71,
    #[strum(serialize = "__vectorcall")]
    cpp_TS72,
    #[strum(serialize = "_abstract_declarator")]
    cpp_AbstractDeclarator,
    #[strum(serialize = "_declarator")]
    cpp_Declarator,
    #[strum(serialize = "_expression")]
    cpp_Expression,
    #[strum(serialize = "_field_declarator")]
    cpp_FieldDeclarator,
    #[strum(serialize = "_statement")]
    cpp_Statement,
    #[strum(serialize = "_type_declarator")]
    cpp_TypeDeclarator,
    #[strum(serialize = "_type_specifier")]
    cpp_TypeSpecifier,
    #[strum(serialize = "_unaligned")]
    cpp_TS73,
    #[strum(serialize = "abstract_array_declarator")]
    cpp_AbstractArrayDeclarator,
    #[strum(serialize = "abstract_function_declarator")]
    cpp_AbstractFunctionDeclarator,
    #[strum(serialize = "abstract_parenthesized_declarator")]
    cpp_AbstractParenthesizedDeclarator,
    #[strum(serialize = "abstract_pointer_declarator")]
    cpp_AbstractPointerDeclarator,
    #[strum(serialize = "abstract_reference_declarator")]
    cpp_AbstractReferenceDeclarator,
    #[strum(serialize = "access_specifier")]
    cpp_AccessSpecifier,
    #[strum(serialize = "alias_declaration")]
    cpp_AliasDeclaration,
    #[strum(serialize = "argument_list")]
    cpp_ArgumentList,
    #[strum(serialize = "array_declarator")]
    cpp_ArrayDeclarator,
    #[strum(serialize = "assignment_expression")]
    cpp_AssignmentExpression,
    #[strum(serialize = "attribute")]
    cpp_Attribute,
    #[strum(serialize = "attribute_declaration")]
    cpp_AttributeDeclaration,
    #[strum(serialize = "attribute_specifier")]
    cpp_AttributeSpecifier,
    #[strum(serialize = "attributed_declarator")]
    cpp_AttributedDeclarator,
    #[strum(serialize = "attributed_statement")]
    cpp_AttributedStatement,
    #[strum(serialize = "auto")]
    cpp_Auto,
    #[strum(serialize = "base_class_clause")]
    cpp_BaseClassClause,
    #[strum(serialize = "binary_expression")]
    cpp_BinaryExpression,
    #[strum(serialize = "bitfield_clause")]
    cpp_BitfieldClause,
    #[strum(serialize = "break")]
    cpp_TS74,
    #[strum(serialize = "break_statement")]
    cpp_BreakStatement,
    #[strum(serialize = "call_expression")]
    cpp_CallExpression,
    #[strum(serialize = "case")]
    cpp_TS75,
    #[strum(serialize = "case_statement")]
    cpp_CaseStatement,
    #[strum(serialize = "cast_expression")]
    cpp_CastExpression,
    #[strum(serialize = "catch")]
    cpp_TS76,
    #[strum(serialize = "catch_clause")]
    cpp_CatchClause,
    #[strum(serialize = "char_literal")]
    cpp_CharLiteral,
    #[strum(serialize = "class")]
    cpp_TS77,
    #[strum(serialize = "class_specifier")]
    cpp_ClassSpecifier,
    #[strum(serialize = "co_await")]
    cpp_TS78,
    #[strum(serialize = "co_await_expression")]
    cpp_CoAwaitExpression,
    #[strum(serialize = "co_return")]
    cpp_TS79,
    #[strum(serialize = "co_return_statement")]
    cpp_CoReturnStatement,
    #[strum(serialize = "co_yield")]
    cpp_TS80,
    #[strum(serialize = "co_yield_statement")]
    cpp_CoYieldStatement,
    #[strum(serialize = "comma_expression")]
    cpp_CommaExpression,
    #[strum(serialize = "comment")]
    cpp_Comment,
    #[strum(serialize = "compound_literal_expression")]
    cpp_CompoundLiteralExpression,
    #[strum(serialize = "compound_statement")]
    cpp_CompoundStatement,
    #[strum(serialize = "concatenated_string")]
    cpp_ConcatenatedString,
    #[strum(serialize = "condition_clause")]
    cpp_ConditionClause,
    #[strum(serialize = "conditional_expression")]
    cpp_ConditionalExpression,
    #[strum(serialize = "const")]
    cpp_TS81,
    #[strum(serialize = "constexpr")]
    cpp_TS82,
    #[strum(serialize = "continue")]
    cpp_TS83,
    #[strum(serialize = "continue_statement")]
    cpp_ContinueStatement,
    #[strum(serialize = "declaration")]
    cpp_Declaration,
    #[strum(serialize = "declaration_list")]
    cpp_DeclarationList,
    #[strum(serialize = "decltype")]
    cpp_TS84,
    #[strum(serialize = "default")]
    cpp_TS85,
    #[strum(serialize = "default_method_clause")]
    cpp_DefaultMethodClause,
    #[strum(serialize = "defined")]
    cpp_TS86,
    #[strum(serialize = "delete")]
    cpp_TS87,
    #[strum(serialize = "delete_expression")]
    cpp_DeleteExpression,
    #[strum(serialize = "delete_method_clause")]
    cpp_DeleteMethodClause,
    #[strum(serialize = "dependent_name")]
    cpp_DependentName,
    #[strum(serialize = "dependent_type")]
    cpp_DependentType,
    #[strum(serialize = "destructor_name")]
    cpp_DestructorName,
    #[strum(serialize = "do")]
    cpp_TS88,
    #[strum(serialize = "do_statement")]
    cpp_DoStatement,
    #[strum(serialize = "else")]
    cpp_TS89,
    #[strum(serialize = "enum")]
    cpp_TS90,
    #[strum(serialize = "enum_specifier")]
    cpp_EnumSpecifier,
    #[strum(serialize = "enumerator")]
    cpp_Enumerator,
    #[strum(serialize = "enumerator_list")]
    cpp_EnumeratorList,
    #[strum(serialize = "escape_sequence")]
    cpp_EscapeSequence,
    #[strum(serialize = "explicit")]
    cpp_TS91,
    #[strum(serialize = "explicit_function_specifier")]
    cpp_ExplicitFunctionSpecifier,
    #[strum(serialize = "expression_statement")]
    cpp_ExpressionStatement,
    #[strum(serialize = "extern")]
    cpp_TS92,
    #[strum(serialize = "false")]
    cpp_False,
    #[strum(serialize = "field_declaration")]
    cpp_FieldDeclaration,
    #[strum(serialize = "field_declaration_list")]
    cpp_FieldDeclarationList,
    #[strum(serialize = "field_designator")]
    cpp_FieldDesignator,
    #[strum(serialize = "field_expression")]
    cpp_FieldExpression,
    #[strum(serialize = "field_identifier")]
    cpp_FieldIdentifier,
    #[strum(serialize = "field_initializer")]
    cpp_FieldInitializer,
    #[strum(serialize = "field_initializer_list")]
    cpp_FieldInitializerList,
    #[strum(serialize = "final")]
    cpp_TS93,
    #[strum(serialize = "for")]
    cpp_TS94,
    #[strum(serialize = "for_range_loop")]
    cpp_ForRangeLoop,
    #[strum(serialize = "for_statement")]
    cpp_ForStatement,
    #[strum(serialize = "friend")]
    cpp_TS95,
    #[strum(serialize = "friend_declaration")]
    cpp_FriendDeclaration,
    #[strum(serialize = "function_declarator")]
    cpp_FunctionDeclarator,
    #[strum(serialize = "function_definition")]
    cpp_FunctionDefinition,
    #[strum(serialize = "goto")]
    cpp_TS96,
    #[strum(serialize = "goto_statement")]
    cpp_GotoStatement,
    #[strum(serialize = "identifier")]
    cpp_Identifier,
    #[strum(serialize = "if")]
    cpp_TS97,
    #[strum(serialize = "if_statement")]
    cpp_IfStatement,
    #[strum(serialize = "init_declarator")]
    cpp_InitDeclarator,
    #[strum(serialize = "initializer_list")]
    cpp_InitializerList,
    #[strum(serialize = "initializer_pair")]
    cpp_InitializerPair,
    #[strum(serialize = "inline")]
    cpp_TS98,
    #[strum(serialize = "labeled_statement")]
    cpp_LabeledStatement,
    #[strum(serialize = "lambda_capture_specifier")]
    cpp_LambdaCaptureSpecifier,
    #[strum(serialize = "lambda_default_capture")]
    cpp_LambdaDefaultCapture,
    #[strum(serialize = "lambda_expression")]
    cpp_LambdaExpression,
    #[strum(serialize = "linkage_specification")]
    cpp_LinkageSpecification,
    #[strum(serialize = "literal_suffix")]
    cpp_LiteralSuffix,
    #[strum(serialize = "long")]
    cpp_TS99,
    #[strum(serialize = "ms_based_modifier")]
    cpp_MsBasedModifier,
    #[strum(serialize = "ms_call_modifier")]
    cpp_MsCallModifier,
    #[strum(serialize = "ms_declspec_modifier")]
    cpp_MsDeclspecModifier,
    #[strum(serialize = "ms_pointer_modifier")]
    cpp_MsPointerModifier,
    #[strum(serialize = "ms_restrict_modifier")]
    cpp_MsRestrictModifier,
    #[strum(serialize = "ms_signed_ptr_modifier")]
    cpp_MsSignedPtrModifier,
    #[strum(serialize = "ms_unaligned_ptr_modifier")]
    cpp_MsUnalignedPtrModifier,
    #[strum(serialize = "ms_unsigned_ptr_modifier")]
    cpp_MsUnsignedPtrModifier,
    #[strum(serialize = "mutable")]
    cpp_TS100,
    #[strum(serialize = "namespace")]
    cpp_TS101,
    #[strum(serialize = "namespace_definition")]
    cpp_NamespaceDefinition,
    #[strum(serialize = "namespace_definition_name")]
    cpp_NamespaceDefinitionName,
    #[strum(serialize = "namespace_identifier")]
    cpp_NamespaceIdentifier,
    #[strum(serialize = "new")]
    cpp_TS102,
    #[strum(serialize = "new_declarator")]
    cpp_NewDeclarator,
    #[strum(serialize = "new_expression")]
    cpp_NewExpression,
    #[strum(serialize = "noexcept")]
    cpp_TS103,
    #[strum(serialize = "null")]
    cpp_Null,
    #[strum(serialize = "nullptr")]
    cpp_Nullptr,
    #[strum(serialize = "number_literal")]
    cpp_NumberLiteral,
    #[strum(serialize = "operator")]
    cpp_TS104,
    #[strum(serialize = "operator_cast")]
    cpp_OperatorCast,
    #[strum(serialize = "operator_name")]
    cpp_OperatorName,
    #[strum(serialize = "optional_parameter_declaration")]
    cpp_OptionalParameterDeclaration,
    #[strum(serialize = "optional_type_parameter_declaration")]
    cpp_OptionalTypeParameterDeclaration,
    #[strum(serialize = "override")]
    cpp_TS105,
    #[strum(serialize = "parameter_declaration")]
    cpp_ParameterDeclaration,
    #[strum(serialize = "parameter_list")]
    cpp_ParameterList,
    #[strum(serialize = "parameter_pack_expansion")]
    cpp_ParameterPackExpansion,
    #[strum(serialize = "parenthesized_declarator")]
    cpp_ParenthesizedDeclarator,
    #[strum(serialize = "parenthesized_expression")]
    cpp_ParenthesizedExpression,
    #[strum(serialize = "pointer_declarator")]
    cpp_PointerDeclarator,
    #[strum(serialize = "pointer_expression")]
    cpp_PointerExpression,
    #[strum(serialize = "preproc_arg")]
    cpp_PreprocArg,
    #[strum(serialize = "nested_namespace_specifier")]
    cpp_Nested_Namespace_Specifier,
    #[strum(serialize = "placeholder_type_specifier")]
    cpp_PlaceholderTypeSpecifier,
    #[strum(serialize = "namespace_alias_definition")]
    cpp_NamespaceAliasDefinition,
    #[strum(serialize = "raw_string_delimiter")]
    cpp_RawStringDelimiter,
    #[strum(serialize = "init_statement")]
    cpp_InitStatement,
    #[strum(serialize = "asm")]
    cpp_Asm,
    #[strum(serialize = "preproc_call")]
    cpp_PreprocCall,
    #[strum(serialize = "preproc_def")]
    cpp_PreprocDef,
    #[strum(serialize = "preproc_defined")]
    cpp_PreprocDefined,
    #[strum(serialize = "preproc_directive")]
    cpp_PreprocDirective,
    #[strum(serialize = "preproc_elif")]
    cpp_PreprocElif,
    #[strum(serialize = "preproc_else")]
    cpp_PreprocElse,
    #[strum(serialize = "preproc_function_def")]
    cpp_PreprocFunctionDef,
    #[strum(serialize = "preproc_if")]
    cpp_PreprocIf,
    #[strum(serialize = "preproc_ifdef")]
    cpp_PreprocIfdef,
    #[strum(serialize = "preproc_include")]
    cpp_PreprocInclude,
    #[strum(serialize = "preproc_params")]
    cpp_PreprocParams,
    #[strum(serialize = "primitive_type")]
    cpp_PrimitiveType,
    #[strum(serialize = "private")]
    cpp_TS106,
    #[strum(serialize = "protected")]
    cpp_TS107,
    #[strum(serialize = "public")]
    cpp_TS108,
    #[strum(serialize = "qualified_identifier")]
    cpp_QualifiedIdentifier,
    #[strum(serialize = "raw_string_literal")]
    cpp_RawStringLiteral,
    #[strum(serialize = "ref_qualifier")]
    cpp_RefQualifier,
    #[strum(serialize = "reference_declarator")]
    cpp_ReferenceDeclarator,
    #[strum(serialize = "register")]
    cpp_TS109,
    #[strum(serialize = "restrict")]
    cpp_TS110,
    #[strum(serialize = "return")]
    cpp_TS111,
    #[strum(serialize = "return_statement")]
    cpp_ReturnStatement,
    #[strum(serialize = "short")]
    cpp_TS112,
    #[strum(serialize = "signed")]
    cpp_TS113,
    #[strum(serialize = "sized_type_specifier")]
    cpp_SizedTypeSpecifier,
    #[strum(serialize = "sizeof")]
    cpp_TS114,
    #[strum(serialize = "sizeof_expression")]
    cpp_SizeofExpression,
    #[strum(serialize = "statement_identifier")]
    cpp_StatementIdentifier,
    #[strum(serialize = "static")]
    cpp_TS115,
    #[strum(serialize = "static_assert")]
    cpp_TS116,
    #[strum(serialize = "static_assert_declaration")]
    cpp_StaticAssertDeclaration,
    #[strum(serialize = "storage_class_specifier")]
    cpp_StorageClassSpecifier,
    #[strum(serialize = "string_literal")]
    cpp_StringLiteral,
    #[strum(serialize = "struct")]
    cpp_TS117,
    #[strum(serialize = "struct_specifier")]
    cpp_StructSpecifier,
    #[strum(serialize = "structured_binding_declarator")]
    cpp_StructuredBindingDeclarator,
    #[strum(serialize = "subscript_designator")]
    cpp_SubscriptDesignator,
    #[strum(serialize = "subscript_expression")]
    cpp_SubscriptExpression,
    #[strum(serialize = "switch")]
    cpp_TS118,
    #[strum(serialize = "switch_statement")]
    cpp_SwitchStatement,
    #[strum(serialize = "system_lib_string")]
    cpp_SystemLibString,
    #[strum(serialize = "template")]
    cpp_TS119,
    #[strum(serialize = "template_argument_list")]
    cpp_TemplateArgumentList,
    #[strum(serialize = "template_declaration")]
    cpp_TemplateDeclaration,
    #[strum(serialize = "template_function")]
    cpp_TemplateFunction,
    #[strum(serialize = "template_instantiation")]
    cpp_TemplateInstantiation,
    #[strum(serialize = "template_method")]
    cpp_TemplateMethod,
    #[strum(serialize = "template_parameter_list")]
    cpp_TemplateParameterList,
    #[strum(serialize = "template_template_parameter_declaration")]
    cpp_TemplateTemplateParameterDeclaration,
    #[strum(serialize = "template_type")]
    cpp_TemplateType,
    #[strum(serialize = "this")]
    cpp_This,
    #[strum(serialize = "thread_local")]
    cpp_TS120,
    #[strum(serialize = "throw")]
    cpp_TS121,
    #[strum(serialize = "throw_specifier")]
    cpp_ThrowSpecifier,
    #[strum(serialize = "throw_statement")]
    cpp_ThrowStatement,
    #[strum(serialize = "trailing_return_type")]
    cpp_TrailingReturnType,
    #[strum(serialize = "translation_unit")]
    cpp_TranslationUnit,
    #[strum(serialize = "true")]
    cpp_True,
    #[strum(serialize = "try")]
    cpp_TS122,
    #[strum(serialize = "try_statement")]
    cpp_TryStatement,
    #[strum(serialize = "type_definition")]
    cpp_TypeDefinition,
    #[strum(serialize = "type_descriptor")]
    cpp_TypeDescriptor,
    #[strum(serialize = "type_identifier")]
    cpp_TypeIdentifier,
    #[strum(serialize = "type_parameter_declaration")]
    cpp_TypeParameterDeclaration,
    #[strum(serialize = "type_qualifier")]
    cpp_TypeQualifier,
    #[strum(serialize = "typedef")]
    cpp_TS123,
    #[strum(serialize = "typename")]
    cpp_TS124,
    #[strum(serialize = "u\"")]
    cpp_TS125,
    #[strum(serialize = "u'")]
    cpp_TS126,
    #[strum(serialize = "u8\"")]
    cpp_TS127,
    #[strum(serialize = "u8'")]
    cpp_TS128,
    #[strum(serialize = "unary_expression")]
    cpp_UnaryExpression,
    #[strum(serialize = "union")]
    cpp_TS129,
    #[strum(serialize = "union_specifier")]
    cpp_UnionSpecifier,
    #[strum(serialize = "unsigned")]
    cpp_TS130,
    #[strum(serialize = "update_expression")]
    cpp_UpdateExpression,
    #[strum(serialize = "user_defined_literal")]
    cpp_UserDefinedLiteral,
    #[strum(serialize = "using")]
    cpp_TS131,
    #[strum(serialize = "using_declaration")]
    cpp_UsingDeclaration,
    #[strum(serialize = "variadic_declarator")]
    cpp_VariadicDeclarator,
    #[strum(serialize = "variadic_parameter_declaration")]
    cpp_VariadicParameterDeclaration,
    #[strum(serialize = "variadic_type_parameter_declaration")]
    cpp_VariadicTypeParameterDeclaration,
    #[strum(serialize = "virtual")]
    cpp_TS132,
    #[strum(serialize = "virtual_function_specifier")]
    cpp_VirtualFunctionSpecifier,
    #[strum(serialize = "virtual_specifier")]
    cpp_VirtualSpecifier,
    #[strum(serialize = "volatile")]
    cpp_TS133,
    #[strum(serialize = "while")]
    cpp_TS134,
    #[strum(serialize = "while_statement")]
    cpp_WhileStatement,
    #[strum(serialize = "{")]
    cpp_TS135,
    #[strum(serialize = "|")]
    cpp_TS136,
    #[strum(serialize = "|=")]
    cpp_TS137,
    #[strum(serialize = "||")]
    cpp_TS138,
    #[strum(serialize = "}")]
    cpp_TS139,
    #[strum(serialize = "~")]
    cpp_TS140,
    #[strum(serialize = ".*")]
    cpp_TS141,
    #[strum(serialize = "inline_asm_expression")]
    cpp_InlineAsmExpression,
    #[strum(serialize = "inline_asm_operand")]
    cpp_InlineAsmOperand,
    #[strum(serialize = "translation_unit_repeat1")]
    cpp_TranslationUnitRepeat1,
    #[strum(serialize = "_declaration_specifiers")]
    cpp_DeclarationSpecifiers,
    #[strum(serialize = "_declaration_specifiers_repeat1")]
    cpp_DeclarationSpecifiers_repeat1,
}
pub trait Lang<T>: LangRef<T> {
    fn make(t: u16) -> &'static T;
    fn to_u16(t: T) -> u16;
}

pub trait LangRef<T> {
    fn name(&self) -> &'static str;
    fn make(&self, t: u16) -> &'static T;
    fn to_u16(&self, t: T) -> u16;
}
pub struct LangWrapper<T: 'static>(&'static dyn LangRef<T>);

impl<T> From<&'static (dyn LangRef<T> + 'static)> for LangWrapper<T> {
    fn from(value: &'static (dyn LangRef<T> + 'static)) -> Self {
        LangWrapper(value)
    }
}

impl<T> LangRef<T> for LangWrapper<T> {
    fn make(&self, t: u16) -> &'static T {
        self.0.make(t)
    }

    fn to_u16(&self, t: T) -> u16 {
        self.0.to_u16(t)
    }

    fn name(&self) -> &'static str {
        self.0.name()
    }
}

pub trait HyperType: Display + Debug {
    fn as_shared(&self) -> Shared;
    fn as_any(&self) -> &dyn std::any::Any;
    fn generic_eq(&self, other: &dyn HyperType) -> bool
    where
        Self: 'static + PartialEq + Sized,
    {
        // Do a type-safe casting. If the types are different,
        // return false, otherwise test the values for equality.
        other
            .as_any()
            .downcast_ref::<Self>()
            .map_or(false, |a| self == a)
    }
    fn is_file(&self) -> bool;
    fn is_directory(&self) -> bool;
    fn is_spaces(&self) -> bool;
    fn is_syntax(&self) -> bool;
    fn get_lang(&self) -> LangWrapper<Self>
    where
        Self: Sized;
}
impl HyperType for u8 {
    fn as_shared(&self) -> Shared {
        todo!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        todo!()
    }

    fn is_file(&self) -> bool {
        todo!()
    }

    fn is_directory(&self) -> bool {
        todo!()
    }

    fn is_spaces(&self) -> bool {
        todo!()
    }

    fn is_syntax(&self) -> bool {
        todo!()
    }

    fn get_lang(&self) -> LangWrapper<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}
pub trait TypeTrait: HyperType + Hash + Copy + Eq + Send + Sync {
    type Lang: Lang<Self>;
    fn is_fork(&self) -> bool;

    fn is_literal(&self) -> bool;
    fn is_primitive(&self) -> bool;
    fn is_type_declaration(&self) -> bool;
    fn is_identifier(&self) -> bool;
    fn is_instance_ref(&self) -> bool;

    fn is_type_body(&self) -> bool;

    fn is_value_member(&self) -> bool;

    fn is_executable_member(&self) -> bool;

    fn is_statement(&self) -> bool;

    fn is_declarative_statement(&self) -> bool;

    fn is_structural_statement(&self) -> bool;

    fn is_block_related(&self) -> bool;

    fn is_simple_statement(&self) -> bool;

    fn is_local_declare(&self) -> bool;

    fn is_parameter(&self) -> bool;

    fn is_parameter_list(&self) -> bool;

    fn is_argument_list(&self) -> bool;

    fn is_expression(&self) -> bool;
    fn is_comment(&self) -> bool;
}

impl Type {
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
}

// TODO remove
pub struct Old;

impl Lang<Type> for Old {
    fn make(t: u16) -> &'static Type {
        Old.make(t)
    }
    fn to_u16(t: Type) -> u16 {
        Old.to_u16(t)
    }
}

impl LangRef<Type> for Old {
    fn make(&self, t: u16) -> &'static Type {
        let t: Type = unsafe { std::mem::transmute(t) };
        todo!()
    }
    fn to_u16(&self, t: Type) -> u16 {
        t as u16
    }

    fn name(&self) -> &'static str {
        todo!()
    }
}

impl HyperType for Type {
    fn is_directory(&self) -> bool {
        self == &Type::Directory || self == &Type::MavenDirectory || self == &Type::MakeDirectory
    }

    fn is_file(&self) -> bool {
        self == &Type::Program
            || self == &Type::xml_SourceFile
            || self == &Type::cpp_TranslationUnit
    }

    fn is_spaces(&self) -> bool {
        self == &Type::Spaces
    }

    fn is_syntax(&self) -> bool {
        todo!()
    }

    fn as_shared(&self) -> Shared {
        panic!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        todo!()
    }

    fn get_lang(&self) -> LangWrapper<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}
// TODO remove, and also Type
impl TypeTrait for Type {
    type Lang = Old;
    fn is_fork(&self) -> bool {
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

    fn is_literal(&self) -> bool {
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
    fn is_primitive(&self) -> bool {
        match self {
            Self::BooleanType => true,
            Self::VoidType => true,
            Self::FloatingPointType => true,
            Self::IntegralType => true,
            _ => false,
        }
    }
    fn is_type_declaration(&self) -> bool {
        match self {
            Self::ClassDeclaration => true,
            Self::EnumDeclaration => true,
            Self::InterfaceDeclaration => true,
            Self::AnnotationTypeDeclaration => true,
            Self::EnumConstant => true, // TODO need more eval
            _ => false,
        }
    }
    // fn primitive_to_str(&self) -> &str {
    //     match self {
    //         Self::BooleanType => "boolean",
    //         Self::VoidType => "void",
    //         Self::FloatingPointType => "float",
    //         Self::IntegralType => "int",
    //         _ => panic!(),
    //     }
    // }
    fn is_identifier(&self) -> bool {
        match self {
            Self::Identifier => true,
            Self::TypeIdentifier => true,
            Self::ScopedIdentifier => true,
            Self::ScopedTypeIdentifier => true,
            _ => false,
        }
    }
    fn is_instance_ref(&self) -> bool {
        match self {
            Self::This => true,
            Self::Super => true,
            _ => false,
        }
    }

    fn is_type_body(&self) -> bool {
        self == &Type::ClassBody
            || self == &Type::InterfaceBody
            || self == &Type::AnnotationTypeBody
            || self == &Type::EnumBody
            || self == &Type::EnumBodyDeclarations
    }

    fn is_value_member(&self) -> bool {
        self == &Type::FieldDeclaration
        || self == &Type::ConstantDeclaration
        // || self == &Type::EnumConstant
        || self == &Type::AnnotationTypeElementDeclaration
    }

    fn is_executable_member(&self) -> bool {
        self == &Type::MethodDeclaration || self == &Type::ConstructorDeclaration
    }

    fn is_statement(&self) -> bool {
        self.is_declarative_statement()
            || self.is_structural_statement()
            || self.is_simple_statement()
            || self.is_block_related()
    }

    fn is_declarative_statement(&self) -> bool {
        self == &Type::LocalVariableDeclaration
            || self == &Type::TryWithResourcesStatement
            || self == &Type::CatchClause
            || self == &Type::ForStatement
            || self == &Type::EnhancedForStatement
    }

    fn is_structural_statement(&self) -> bool {
        self == &Type::SwitchStatement
            || self == &Type::WhileStatement
            || self == &Type::DoStatement
            || self == &Type::IfStatement
            || self == &Type::TryStatement
            || self == &Type::FinallyClause
            || self == &Type::TryWithResourcesExtendedStatement
    }

    fn is_block_related(&self) -> bool {
        self == &Type::StaticInitializer
            || self == &Type::ConstructorBody
            || self == &Type::Block
            || self == &Type::SwitchBlock
            || self == &Type::SwitchBlockStatementGroup
    }

    fn is_simple_statement(&self) -> bool {
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

    fn is_local_declare(&self) -> bool {
        self == &Type::LocalVariableDeclaration
            || self == &Type::EnhancedForVariable
            || self == &Type::Resource
    }

    fn is_parameter(&self) -> bool {
        self == &Type::Resource
            || self == &Type::FormalParameter
            || self == &Type::SpreadParameter
            || self == &Type::CatchFormalParameter
            || self == &Type::TypeParameter
    }

    fn is_parameter_list(&self) -> bool {
        self == &Type::ResourceSpecification
            || self == &Type::FormalParameters
            || self == &Type::TypeParameters
    }

    fn is_argument_list(&self) -> bool {
        self == &Type::ArgumentList
            || self == &Type::TypeArguments
            || self == &Type::AnnotationArgumentList
    }

    fn is_expression(&self) -> bool {
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

    fn is_comment(&self) -> bool {
        self == &Type::Comment || self == &Type::cpp_Comment || self == &Type::xml_Comment
    }
    // fn is_type_propagating_expression(&self) -> bool {
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
    type TreeId: NodeId;
}

pub trait Typed {
    type Type: HyperType + Eq + Copy + Send + Sync; // todo try remove Hash and copy
    fn get_type(&self) -> Self::Type; // TODO add TypeTrait bound on Self::Type to forbid AnyType from being given
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
    type Children<'a>: Children<Self::ChildIdx, <Self::TreeId as NodeId>::IdN> + ?Sized
    where
        Self: 'a;
    // type Children<'a>: std::ops::Index<Self::ChildIdx, Output = Self::TreeId> + IntoIterator<Item = Self::TreeId>
    // where
    //     Self: 'a;

    fn child_count(&self) -> Self::ChildIdx;
    fn child(&self, idx: &Self::ChildIdx) -> Option<<Self::TreeId as NodeId>::IdN>;
    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<<Self::TreeId as NodeId>::IdN>;
    fn children(&self) -> Option<&Self::Children<'_>>;
    // unsafe fn children_unchecked(&self) -> <Self::Children as std::ops::Deref>::Target
    // where
    //     <Self::Children as std::ops::Deref>::Target: std::ops::Index<<Self as WithChildren>::ChildIdx, Output = <Self as Stored>::TreeId>
    //         + Sized;
    // fn get_children_cpy(&self) -> Self::Children;
}

pub trait WithChildrenSameLang: WithChildren
// where
//     <Self::Children as std::ops::Deref>::Target: std::ops::Index<<Self as WithChildren>::ChildIdx, Output = <Self as Stored>::TreeId>
//         + Sized,
{
    type TChildren<'a>: Children<Self::ChildIdx, Self::TreeId> + ?Sized
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
pub trait WithMetaData<C> {
    fn get_metadata(&self) -> Option<&C>;
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
    fn get_label_unchecked<'a>(&'a self) -> &'a Self::Label;
    fn try_get_label<'a>(&'a self) -> Option<&'a Self::Label>;
}

pub trait Tree: Typed + Labeled + WithChildren
// where
//     <Self::Children as std::ops::Deref>::Target: std::ops::Index<<Self as WithChildren>::ChildIdx, Output = <Self as Stored>::TreeId>
//         + Sized,
{
    fn has_children(&self) -> bool;
    fn has_label(&self) -> bool;
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

pub trait NodeId: Eq + Clone {
    type IdN: Eq + NodeId;
    fn as_id(&self) -> &Self::IdN;
    // fn as_ty(&self) -> &Self::Ty;
    unsafe fn from_id(id: Self::IdN) -> Self;
    unsafe fn from_ref_id(id: &Self::IdN) -> &Self;
}

impl NodeId for u16 {
    type IdN = u16;
    fn as_id(&self) -> &Self::IdN {
        self
    }
    unsafe fn from_id(id: Self::IdN) -> Self {
        id
    }

    unsafe fn from_ref_id(id: &Self::IdN) -> &Self {
        id
    }
}

pub trait TypedNodeId: NodeId {
    type Ty: HyperType + Hash + Copy + Eq + Send + Sync;
}

pub trait TypedNodeStore<IdN: TypedNodeId> {
    type R<'a>: Typed<Type = IdN::Ty>
    where
        Self: 'a;
    fn try_typed(&self, id: &IdN::IdN) -> Option<IdN>;
    fn try_resolve(&self, id: &IdN::IdN) -> Option<(Self::R<'_>, IdN)> {
        self.try_typed(id).map(|x| (self.resolve(&x), x))
    }
    fn resolve(&self, id: &IdN) -> Self::R<'_>;
}

pub trait DecompressedSubtree<'a, T: Stored> {
    type Out: DecompressedSubtree<'a, T>;
    fn decompress<S>(store: &'a S, id: &T::TreeId) -> Self::Out
    where
        S: NodeStore<T::TreeId, R<'a> = T>;
}

pub trait DecompressibleNodeStore<IdN>: NodeStore<IdN> {
    fn decompress<'a, D: DecompressedSubtree<'a, Self::R<'a>>>(
        &'a self,
        id: &IdN,
    ) -> (&'a Self, D::Out)
    where
        Self: Sized,
        Self::R<'a>: Stored<TreeId = IdN>,
    {
        (self, D::decompress(self, id))
    }

    fn decompress_pair<'a, D1, D2>(&'a self, id1: &IdN, id2: &IdN) -> (&'a Self, (D1::Out, D2::Out))
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
pub trait NodeStoreExt<T: Tree> {
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

type TypeInternalSize = u16;
pub trait TypeStore<T> {
    type Ty: 'static
        + HyperType
        + Eq
        + std::hash::Hash
        + Copy
        + std::marker::Send
        + std::marker::Sync;
    const MASK: TypeInternalSize;
    fn resolve_type(&self, n: &T) -> Self::Ty;
    fn resolve_lang(&self, n: &T) -> LangWrapper<Self::Ty>;
    type Marshaled;
    fn marshal_type(&self, n: &T) -> Self::Marshaled;
}

pub trait SpecializedTypeStore<T: Typed>: TypeStore<T> {}

pub trait HyperAST<'store> {
    type IdN: NodeId<IdN = Self::IdN>;
    type Idx: PrimInt;
    type Label;
    type T: Tree<TreeId = Self::IdN, Label = Self::Label, ChildIdx = Self::Idx>;
    type NS: 'store + NodeStore<Self::IdN, R<'store> = Self::T>;
    fn node_store(&self) -> &Self::NS;

    type LS: LabelStore<str, I = Self::Label>;
    fn label_store(&self) -> &Self::LS;

    type TS: TypeStore<Self::T, Ty = <Self::T as Typed>::Type>;
    fn type_store(&self) -> &Self::TS;

    fn decompress<D: DecompressedSubtree<'store, Self::T, Out = D>>(
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
        D1: DecompressedSubtree<'store, Self::T, Out = D1>,
        D2: DecompressedSubtree<'store, Self::T, Out = D2>,
    {
        {
            (
                self,
                (
                    D1::decompress(self.node_store(), id1),
                    D2::decompress(self.node_store(), id2),
                ),
            )
        }
    }
}
pub trait TypedHyperAST<'store, TIdN: TypedNodeId<IdN = Self::IdN>>: HyperAST<'store> {
    type T: Tree<Type = TIdN::Ty, TreeId = Self::IdN, Label = Self::Label>;
    type NS: 'store + TypedNodeStore<TIdN, R<'store> = <Self as TypedHyperAST<'store, TIdN>>::T>;
    fn typed_node_store(&self) -> &<Self as TypedHyperAST<'store, TIdN>>::NS;
}

pub struct SimpleHyperAST<T, TS, NS, LS> {
    pub type_store: TS,
    pub node_store: NS,
    pub label_store: LS,
    pub _phantom: std::marker::PhantomData<T>,
}

impl<T, TS: Default, NS: Default, LS: Default> Default for SimpleHyperAST<T, TS, NS, LS> {
    fn default() -> Self {
        Self {
            type_store: Default::default(),
            node_store: Default::default(),
            label_store: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<T, TS, NS, LS> NodeStore<T::TreeId> for SimpleHyperAST<T, TS, NS, LS>
where
    T: Tree,
    T::TreeId: NodeId<IdN = T::TreeId>,
    T::Type: 'static,
    NS: NodeStore<T::TreeId>,
{
    type R<'a> = NS::R<'a>
    where
        Self: 'a;

    fn resolve(&self, id: &T::TreeId) -> Self::R<'_> {
        self.node_store.resolve(id)
    }
}

impl<'store, T, TS, NS, LS> LabelStore<str> for SimpleHyperAST<T, TS, NS, LS>
where
    T: Tree,
    T::TreeId: NodeId<IdN = T::TreeId>,
    T::Type: 'static,
    LS: LabelStore<str, I = T::Label>,
    <T as Labeled>::Label: Copy,
{
    type I = LS::I;

    fn get_or_insert<U: Borrow<str>>(&mut self, node: U) -> Self::I {
        self.label_store.get_or_insert(node)
    }

    fn get<U: Borrow<str>>(&self, node: U) -> Option<Self::I> {
        self.label_store.get(node)
    }

    fn resolve(&self, id: &Self::I) -> &str {
        self.label_store.resolve(id)
    }
}

impl<'store, T, TS, NS, LS> TypeStore<T> for SimpleHyperAST<T, TS, NS, LS>
where
    T: Tree,
    T::TreeId: NodeId<IdN = T::TreeId>,
    T::Type: 'static + std::hash::Hash,
    TS: TypeStore<T, Ty = T::Type>,
{
    type Ty = TS::Ty;

    const MASK: u16 = TS::MASK;

    fn resolve_type(&self, n: &T) -> Self::Ty {
        self.type_store.resolve_type(n)
    }

    fn resolve_lang(&self, n: &T) -> LangWrapper<Self::Ty> {
        self.type_store.resolve_lang(n)
    }

    type Marshaled = TS::Marshaled;

    fn marshal_type(&self, n: &T) -> Self::Marshaled {
        self.type_store.marshal_type(n)
    }
}

pub struct TypeIndex {
    pub lang: &'static str,
    pub ty: u16,
}

impl<'store, T, TS, NS, LS> HyperAST<'store> for SimpleHyperAST<T, TS, NS, LS>
where
    T: Tree,
    T::TreeId: NodeId<IdN = T::TreeId>,
    T::Type: 'static,
    TS: TypeStore<T, Ty = T::Type>,
    NS: 'store + NodeStore<T::TreeId, R<'store> = T>,
    LS: LabelStore<str, I = T::Label>,
{
    type IdN = T::TreeId;

    type Idx = T::ChildIdx;

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

    type TS = TS;

    fn type_store(&self) -> &Self::TS {
        &self.type_store
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

impl Type {
    pub fn parse_cpp(t: &str) -> Self {
        Self::try_parse_cpp(t).expect(t)
    }
    pub fn try_parse_cpp(t: &str) -> Option<Self> {
        let t = match t {
            "\n" => Self::cpp_TS0,
            "!" => Self::cpp_TS1,
            "!=" => Self::cpp_TS2,
            "\"" => Self::cpp_TS3,
            "\"\"" => Self::cpp_TS4,
            "#define" => Self::cpp_TS5,
            "#elif" => Self::cpp_TS6,
            "#else" => Self::cpp_TS7,
            "#endif" => Self::cpp_TS8,
            "#if" => Self::cpp_TS9,
            "#ifdef" => Self::cpp_TS10,
            "#ifndef" => Self::cpp_TS11,
            "#include" => Self::cpp_TS12,
            "%" => Self::cpp_TS13,
            "%=" => Self::cpp_TS14,
            "&" => Self::cpp_TS15,
            "&&" => Self::cpp_TS16,
            "&=" => Self::cpp_TS17,
            "'" => Self::cpp_TS18,
            "(" => Self::cpp_TS19,
            "()" => Self::cpp_TS20,
            ")" => Self::cpp_TS21,
            "*" => Self::cpp_TS22,
            "*=" => Self::cpp_TS23,
            "+" => Self::cpp_TS24,
            "++" => Self::cpp_TS25,
            "+=" => Self::cpp_TS26,
            "," => Self::cpp_TS27,
            "-" => Self::cpp_TS28,
            "--" => Self::cpp_TS29,
            "-=" => Self::cpp_TS30,
            "->" => Self::cpp_TS31,
            "->*" => Self::cpp_TS32,
            "." => Self::cpp_TS33,
            "..." => Self::cpp_TS34,
            "/" => Self::cpp_TS35,
            "/=" => Self::cpp_TS36,
            ":" => Self::cpp_TS37,
            "::" => Self::cpp_TS38,
            ";" => Self::cpp_TS39,
            "<" => Self::cpp_TS40,
            "<<" => Self::cpp_TS41,
            "<<=" => Self::cpp_TS42,
            "<=" => Self::cpp_TS43,
            "=" => Self::cpp_TS44,
            "==" => Self::cpp_TS45,
            ">" => Self::cpp_TS46,
            ">=" => Self::cpp_TS47,
            ">>" => Self::cpp_TS48,
            ">>=" => Self::cpp_TS49,
            "?" => Self::cpp_TS50,
            "L\"" => Self::cpp_TS51,
            "L'" => Self::cpp_TS52,
            "U\"" => Self::cpp_TS53,
            "U'" => Self::cpp_TS54,
            "[" => Self::cpp_TS55,
            "[[" => Self::cpp_TS56,
            "[]" => Self::cpp_TS57,
            "]" => Self::cpp_TS58,
            "]]" => Self::cpp_TS59,
            "^" => Self::cpp_TS60,
            "^=" => Self::cpp_TS61,
            "_Atomic" => Self::cpp_TS62,
            "__attribute__" => Self::cpp_TS63,
            "__based" => Self::cpp_TS64,
            "__cdecl" => Self::cpp_TS65,
            "__clrcall" => Self::cpp_TS66,
            "__declspec" => Self::cpp_TS67,
            "__fastcall" => Self::cpp_TS68,
            "__stdcall" => Self::cpp_TS69,
            "__thiscall" => Self::cpp_TS70,
            "__unaligned" => Self::cpp_TS71,
            "__vectorcall" => Self::cpp_TS72,
            "_abstract_declarator" => Self::cpp_AbstractDeclarator,
            "_declarator" => Self::cpp_Declarator,
            "_expression" => Self::cpp_Expression,
            "_field_declarator" => Self::cpp_FieldDeclarator,
            "_statement" => Self::cpp_Statement,
            "_type_declarator" => Self::cpp_TypeDeclarator,
            "_type_specifier" => Self::cpp_TypeSpecifier,
            "_unaligned" => Self::cpp_TS73,
            "abstract_array_declarator" => Self::cpp_AbstractArrayDeclarator,
            "abstract_function_declarator" => Self::cpp_AbstractFunctionDeclarator,
            "abstract_parenthesized_declarator" => Self::cpp_AbstractParenthesizedDeclarator,
            "abstract_pointer_declarator" => Self::cpp_AbstractPointerDeclarator,
            "abstract_reference_declarator" => Self::cpp_AbstractReferenceDeclarator,
            "access_specifier" => Self::cpp_AccessSpecifier,
            "alias_declaration" => Self::cpp_AliasDeclaration,
            "argument_list" => Self::cpp_ArgumentList,
            "array_declarator" => Self::cpp_ArrayDeclarator,
            "assignment_expression" => Self::cpp_AssignmentExpression,
            "attribute" => Self::cpp_Attribute,
            "attribute_declaration" => Self::cpp_AttributeDeclaration,
            "attribute_specifier" => Self::cpp_AttributeSpecifier,
            "attributed_declarator" => Self::cpp_AttributedDeclarator,
            "attributed_statement" => Self::cpp_AttributedStatement,
            "auto" => Self::cpp_Auto,
            "base_class_clause" => Self::cpp_BaseClassClause,
            "binary_expression" => Self::cpp_BinaryExpression,
            "bitfield_clause" => Self::cpp_BitfieldClause,
            "break" => Self::cpp_TS74,
            "break_statement" => Self::cpp_BreakStatement,
            "call_expression" => Self::cpp_CallExpression,
            "case" => Self::cpp_TS75,
            "case_statement" => Self::cpp_CaseStatement,
            "cast_expression" => Self::cpp_CastExpression,
            "catch" => Self::cpp_TS76,
            "catch_clause" => Self::cpp_CatchClause,
            "char_literal" => Self::cpp_CharLiteral,
            "class" => Self::cpp_TS77,
            "class_specifier" => Self::cpp_ClassSpecifier,
            "co_await" => Self::cpp_TS78,
            "co_await_expression" => Self::cpp_CoAwaitExpression,
            "co_return" => Self::cpp_TS79,
            "co_return_statement" => Self::cpp_CoReturnStatement,
            "co_yield" => Self::cpp_TS80,
            "co_yield_statement" => Self::cpp_CoYieldStatement,
            "comma_expression" => Self::cpp_CommaExpression,
            "comment" => Self::cpp_Comment,
            "compound_literal_expression" => Self::cpp_CompoundLiteralExpression,
            "compound_statement" => Self::cpp_CompoundStatement,
            "concatenated_string" => Self::cpp_ConcatenatedString,
            "condition_clause" => Self::cpp_ConditionClause,
            "conditional_expression" => Self::cpp_ConditionalExpression,
            "const" => Self::cpp_TS81,
            "constexpr" => Self::cpp_TS82,
            "continue" => Self::cpp_TS83,
            "continue_statement" => Self::cpp_ContinueStatement,
            "declaration" => Self::cpp_Declaration,
            "declaration_list" => Self::cpp_DeclarationList,
            "decltype" => Self::cpp_TS84,
            "default" => Self::cpp_TS85,
            "default_method_clause" => Self::cpp_DefaultMethodClause,
            "defined" => Self::cpp_TS86,
            "delete" => Self::cpp_TS87,
            "delete_expression" => Self::cpp_DeleteExpression,
            "delete_method_clause" => Self::cpp_DeleteMethodClause,
            "dependent_name" => Self::cpp_DependentName,
            "dependent_type" => Self::cpp_DependentType,
            "destructor_name" => Self::cpp_DestructorName,
            "do" => Self::cpp_TS88,
            "do_statement" => Self::cpp_DoStatement,
            "else" => Self::cpp_TS89,
            "enum" => Self::cpp_TS90,
            "enum_specifier" => Self::cpp_EnumSpecifier,
            "enumerator" => Self::cpp_Enumerator,
            "enumerator_list" => Self::cpp_EnumeratorList,
            "escape_sequence" => Self::cpp_EscapeSequence,
            "explicit" => Self::cpp_TS91,
            "explicit_function_specifier" => Self::cpp_ExplicitFunctionSpecifier,
            "expression_statement" => Self::cpp_ExpressionStatement,
            "extern" => Self::cpp_TS92,
            "false" => Self::cpp_False,
            "field_declaration" => Self::cpp_FieldDeclaration,
            "field_declaration_list" => Self::cpp_FieldDeclarationList,
            "field_designator" => Self::cpp_FieldDesignator,
            "field_expression" => Self::cpp_FieldExpression,
            "field_identifier" => Self::cpp_FieldIdentifier,
            "field_initializer" => Self::cpp_FieldInitializer,
            "field_initializer_list" => Self::cpp_FieldInitializerList,
            "final" => Self::cpp_TS93,
            "for" => Self::cpp_TS94,
            "for_range_loop" => Self::cpp_ForRangeLoop,
            "for_statement" => Self::cpp_ForStatement,
            "friend" => Self::cpp_TS95,
            "friend_declaration" => Self::cpp_FriendDeclaration,
            "function_declarator" => Self::cpp_FunctionDeclarator,
            "function_definition" => Self::cpp_FunctionDefinition,
            "goto" => Self::cpp_TS96,
            "goto_statement" => Self::cpp_GotoStatement,
            "identifier" => Self::cpp_Identifier,
            "if" => Self::cpp_TS97,
            "if_statement" => Self::cpp_IfStatement,
            "init_declarator" => Self::cpp_InitDeclarator,
            "initializer_list" => Self::cpp_InitializerList,
            "initializer_pair" => Self::cpp_InitializerPair,
            "inline" => Self::cpp_TS98,
            "labeled_statement" => Self::cpp_LabeledStatement,
            "lambda_capture_specifier" => Self::cpp_LambdaCaptureSpecifier,
            "lambda_default_capture" => Self::cpp_LambdaDefaultCapture,
            "lambda_expression" => Self::cpp_LambdaExpression,
            "linkage_specification" => Self::cpp_LinkageSpecification,
            "literal_suffix" => Self::cpp_LiteralSuffix,
            "long" => Self::cpp_TS99,
            "ms_based_modifier" => Self::cpp_MsBasedModifier,
            "ms_call_modifier" => Self::cpp_MsCallModifier,
            "ms_declspec_modifier" => Self::cpp_MsDeclspecModifier,
            "ms_pointer_modifier" => Self::cpp_MsPointerModifier,
            "ms_restrict_modifier" => Self::cpp_MsRestrictModifier,
            "ms_signed_ptr_modifier" => Self::cpp_MsSignedPtrModifier,
            "ms_unaligned_ptr_modifier" => Self::cpp_MsUnalignedPtrModifier,
            "ms_unsigned_ptr_modifier" => Self::cpp_MsUnsignedPtrModifier,
            "mutable" => Self::cpp_TS100,
            "namespace" => Self::cpp_TS101,
            "namespace_definition" => Self::cpp_NamespaceDefinition,
            "namespace_definition_name" => Self::cpp_NamespaceDefinitionName,
            "namespace_identifier" => Self::cpp_NamespaceIdentifier,
            "new" => Self::cpp_TS102,
            "new_declarator" => Self::cpp_NewDeclarator,
            "new_expression" => Self::cpp_NewExpression,
            "noexcept" => Self::cpp_TS103,
            "null" => Self::cpp_Null,
            "nullptr" => Self::cpp_Nullptr,
            "number_literal" => Self::cpp_NumberLiteral,
            "operator" => Self::cpp_TS104,
            "operator_cast" => Self::cpp_OperatorCast,
            "operator_name" => Self::cpp_OperatorName,
            "optional_parameter_declaration" => Self::cpp_OptionalParameterDeclaration,
            "optional_type_parameter_declaration" => Self::cpp_OptionalTypeParameterDeclaration,
            "override" => Self::cpp_TS105,
            "parameter_declaration" => Self::cpp_ParameterDeclaration,
            "parameter_list" => Self::cpp_ParameterList,
            "parameter_pack_expansion" => Self::cpp_ParameterPackExpansion,
            "parenthesized_declarator" => Self::cpp_ParenthesizedDeclarator,
            "parenthesized_expression" => Self::cpp_ParenthesizedExpression,
            "pointer_declarator" => Self::cpp_PointerDeclarator,
            "pointer_expression" => Self::cpp_PointerExpression,
            "preproc_arg" => Self::cpp_PreprocArg,
            "nested_namespace_specifier" => Self::cpp_Nested_Namespace_Specifier,
            "placeholder_type_specifier" => Self::cpp_PlaceholderTypeSpecifier,
            "namespace_alias_definition" => Self::cpp_NamespaceAliasDefinition,
            "raw_string_delimiter" => Self::cpp_RawStringDelimiter,
            "preproc_call" => Self::cpp_PreprocCall,
            "preproc_def" => Self::cpp_PreprocDef,
            "preproc_defined" => Self::cpp_PreprocDefined,
            "preproc_directive" => Self::cpp_PreprocDirective,
            "preproc_elif" => Self::cpp_PreprocElif,
            "preproc_else" => Self::cpp_PreprocElse,
            "preproc_function_def" => Self::cpp_PreprocFunctionDef,
            "preproc_if" => Self::cpp_PreprocIf,
            "preproc_ifdef" => Self::cpp_PreprocIfdef,
            "preproc_include" => Self::cpp_PreprocInclude,
            "preproc_params" => Self::cpp_PreprocParams,
            "primitive_type" => Self::cpp_PrimitiveType,
            "private" => Self::cpp_TS106,
            "protected" => Self::cpp_TS107,
            "public" => Self::cpp_TS108,
            "qualified_identifier" => Self::cpp_QualifiedIdentifier,
            "raw_string_literal" => Self::cpp_RawStringLiteral,
            "ref_qualifier" => Self::cpp_RefQualifier,
            "reference_declarator" => Self::cpp_ReferenceDeclarator,
            "register" => Self::cpp_TS109,
            "restrict" => Self::cpp_TS110,
            "return" => Self::cpp_TS111,
            "return_statement" => Self::cpp_ReturnStatement,
            "short" => Self::cpp_TS112,
            "signed" => Self::cpp_TS113,
            "sized_type_specifier" => Self::cpp_SizedTypeSpecifier,
            "sizeof" => Self::cpp_TS114,
            "sizeof_expression" => Self::cpp_SizeofExpression,
            "statement_identifier" => Self::cpp_StatementIdentifier,
            "static" => Self::cpp_TS115,
            "static_assert" => Self::cpp_TS116,
            "static_assert_declaration" => Self::cpp_StaticAssertDeclaration,
            "storage_class_specifier" => Self::cpp_StorageClassSpecifier,
            "string_literal" => Self::cpp_StringLiteral,
            "struct" => Self::cpp_TS117,
            "struct_specifier" => Self::cpp_StructSpecifier,
            "structured_binding_declarator" => Self::cpp_StructuredBindingDeclarator,
            "subscript_designator" => Self::cpp_SubscriptDesignator,
            "subscript_expression" => Self::cpp_SubscriptExpression,
            "switch" => Self::cpp_TS118,
            "switch_statement" => Self::cpp_SwitchStatement,
            "system_lib_string" => Self::cpp_SystemLibString,
            "template" => Self::cpp_TS119,
            "template_argument_list" => Self::cpp_TemplateArgumentList,
            "template_declaration" => Self::cpp_TemplateDeclaration,
            "template_function" => Self::cpp_TemplateFunction,
            "template_instantiation" => Self::cpp_TemplateInstantiation,
            "template_method" => Self::cpp_TemplateMethod,
            "template_parameter_list" => Self::cpp_TemplateParameterList,
            "template_template_parameter_declaration" => {
                Self::cpp_TemplateTemplateParameterDeclaration
            }
            "template_type" => Self::cpp_TemplateType,
            "this" => Self::cpp_This,
            "thread_local" => Self::cpp_TS120,
            "throw" => Self::cpp_TS121,
            "throw_specifier" => Self::cpp_ThrowSpecifier,
            "throw_statement" => Self::cpp_ThrowStatement,
            "trailing_return_type" => Self::cpp_TrailingReturnType,
            "translation_unit" => Self::cpp_TranslationUnit,
            "true" => Self::cpp_True,
            "try" => Self::cpp_TS122,
            "try_statement" => Self::cpp_TryStatement,
            "type_definition" => Self::cpp_TypeDefinition,
            "type_descriptor" => Self::cpp_TypeDescriptor,
            "type_identifier" => Self::cpp_TypeIdentifier,
            "type_parameter_declaration" => Self::cpp_TypeParameterDeclaration,
            "type_qualifier" => Self::cpp_TypeQualifier,
            "typedef" => Self::cpp_TS123,
            "typename" => Self::cpp_TS124,
            "u\"" => Self::cpp_TS125,
            "u'" => Self::cpp_TS126,
            "u8\"" => Self::cpp_TS127,
            "u8'" => Self::cpp_TS128,
            "unary_expression" => Self::cpp_UnaryExpression,
            "union" => Self::cpp_TS129,
            "union_specifier" => Self::cpp_UnionSpecifier,
            "unsigned" => Self::cpp_TS130,
            "update_expression" => Self::cpp_UpdateExpression,
            "user_defined_literal" => Self::cpp_UserDefinedLiteral,
            "using" => Self::cpp_TS131,
            "using_declaration" => Self::cpp_UsingDeclaration,
            "variadic_declarator" => Self::cpp_VariadicDeclarator,
            "variadic_parameter_declaration" => Self::cpp_VariadicParameterDeclaration,
            "variadic_type_parameter_declaration" => Self::cpp_VariadicTypeParameterDeclaration,
            "virtual" => Self::cpp_TS132,
            "virtual_function_specifier" => Self::cpp_VirtualFunctionSpecifier,
            "virtual_specifier" => Self::cpp_VirtualSpecifier,
            "volatile" => Self::cpp_TS133,
            "while" => Self::cpp_TS134,
            "while_statement" => Self::cpp_WhileStatement,
            "{" => Self::cpp_TS135,
            "|" => Self::cpp_TS136,
            "|=" => Self::cpp_TS137,
            "||" => Self::cpp_TS138,
            "}" => Self::cpp_TS139,
            "~" => Self::cpp_TS140,
            "ERROR" => Self::Error,
            ".*" => Self::cpp_TS141,
            "asm" => Self::cpp_Asm,
            "init_statement" => Self::cpp_InitStatement,
            "inline_asm_expression" => Self::cpp_InlineAsmExpression,
            "inline_asm_operand" => Self::cpp_InlineAsmOperand,
            "translation_unit_repeat1" => Self::cpp_TranslationUnitRepeat1,
            "_declaration_specifiers" => Self::cpp_DeclarationSpecifiers,
            "_declaration_specifiers_repeat1" => Self::cpp_DeclarationSpecifiers_repeat1,
            _ => return None,
        };
        Some(t)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AnyType(&'static dyn HyperType);

unsafe impl Send for AnyType {}
unsafe impl Sync for AnyType {}
impl PartialEq for AnyType {
    fn eq(&self, other: &Self) -> bool {
        self.generic_eq(other.0)
    }
}
// impl Default for AnyType {}
impl Eq for AnyType {}
impl Hash for AnyType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_shared().hash(state);
    }
}
impl Display for AnyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}
impl From<&'static dyn HyperType> for AnyType {
    fn from(value: &'static dyn HyperType) -> Self {
        Self(value)
    }
}

impl HyperType for AnyType {
    fn is_file(&self) -> bool {
        self.0.is_file()
    }

    fn is_directory(&self) -> bool {
        self.0.is_directory()
    }

    fn is_spaces(&self) -> bool {
        self.0.is_spaces()
    }

    fn is_syntax(&self) -> bool {
        self.0.is_syntax()
    }

    fn as_shared(&self) -> Shared {
        self.0.as_shared()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self.0.as_any()
    }

    fn get_lang(&self) -> LangWrapper<Self>
    where
        Self: Sized,
    {
        // self.0.get_lang()
        panic!()
    }
}

// impl Lang<AnyType> for AnyType {
//     fn make(t: u16) -> AnyType {
//         todo!()
//     }

//     fn to_u16(t: AnyType) -> u16 {
//         todo!()
//     }
// }
// impl TypeTrait for AnyType {
//     type Lang = AnyType;

//     fn is_fork(&self) -> bool {
//         todo!()
//     }

//     fn is_literal(&self) -> bool {
//         todo!()
//     }

//     fn is_primitive(&self) -> bool {
//         todo!()
//     }

//     fn is_type_declaration(&self) -> bool {
//         todo!()
//     }

//     fn is_identifier(&self) -> bool {
//         todo!()
//     }

//     fn is_instance_ref(&self) -> bool {
//         todo!()
//     }

//     fn is_type_body(&self) -> bool {
//         todo!()
//     }

//     fn is_value_member(&self) -> bool {
//         todo!()
//     }

//     fn is_executable_member(&self) -> bool {
//         todo!()
//     }

//     fn is_statement(&self) -> bool {
//         todo!()
//     }

//     fn is_declarative_statement(&self) -> bool {
//         todo!()
//     }

//     fn is_structural_statement(&self) -> bool {
//         todo!()
//     }

//     fn is_block_related(&self) -> bool {
//         todo!()
//     }

//     fn is_simple_statement(&self) -> bool {
//         todo!()
//     }

//     fn is_local_declare(&self) -> bool {
//         todo!()
//     }

//     fn is_parameter(&self) -> bool {
//         todo!()
//     }

//     fn is_parameter_list(&self) -> bool {
//         todo!()
//     }

//     fn is_argument_list(&self) -> bool {
//         todo!()
//     }

//     fn is_expression(&self) -> bool {
//         todo!()
//     }

//     fn is_comment(&self) -> bool {
//         todo!()
//     }
// }
