use std::fmt::Display;

use hyper_ast::{
    store::defaults::NodeIdentifier,
    tree_gen::parser::NodeWithU16TypeId,
    types::{AnyType, HyperType, LangRef, NodeId, TypeStore, TypeTrait, TypedNodeId},
};

#[cfg(feature = "legion")]
mod legion_impls {
    use super::*;

    use crate::TNode;

    impl<'a> TNode<'a> {
        pub fn obtain_type<T>(&self, _: &mut impl JavaEnabledTypeStore<T>) -> Type {
            let t = self.kind_id();
            Type::from_u16(t)
        }
    }

    use hyper_ast::{store::nodes::legion::HashedNodeRef, types::TypeIndex};

    // impl<'a> TypeStore<HashedNodeRef<'a, AnyType>> for Single {
    //     type Ty = Type;
    //     const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

    //     fn resolve_type(&self, n: &HashedNodeRef<'a, AnyType>) -> Self::Ty {
    //         n.get_component::<Type>().unwrap().clone()
    //     }

    //     fn resolve_lang(&self, n: &HashedNodeRef<'a, AnyType>) -> hyper_ast::types::LangWrapper<Self::Ty> {
    //         From::<&'static (dyn LangRef<Type>)>::from(&Java)
    //     }

    //     type Marshaled = TypeIndex;

    //     fn marshal(&self, n: &HashedNodeRef<'a, TIdN<NodeIdentifier>>) -> Self::Marshaled {
    //         TypeIndex {
    //             lang: LangRef::<Type>::name(&Java),
    //             ty: *n.get_component::<Type>().unwrap() as u16,
    //         }
    //     }
    // }
    // impl<'a> JavaEnabledTypeStore<HashedNodeRef<'a, AnyType>> for Single {
    //     // fn intern(&self, t: Type) -> Self::Ty {
    //     //     // T((u16::MAX - self.lang) | t as u16)
    //     //     t
    //     // }

    //     // fn resolve(&self, t: Self::Ty) -> Type {
    //     //     t
    //     //     // let t = t.0 as u16;
    //     //     // let t = t & !self.mask;
    //     //     // Type::resolve(t)
    //     // }
    // }

    impl<'a> TypeStore<HashedNodeRef<'a, TIdN<NodeIdentifier>>> for TStore {
        type Ty = Type;
        const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

        fn resolve_type(&self, n: &HashedNodeRef<'a, TIdN<NodeIdentifier>>) -> Self::Ty {
            todo!("{:?}", n)
        }

        fn resolve_lang(
            &self,
            n: &HashedNodeRef<'a, TIdN<NodeIdentifier>>,
        ) -> hyper_ast::types::LangWrapper<Self::Ty> {
            todo!("{:?}", n)
        }

        type Marshaled = TypeIndex;

        fn marshal_type(&self, n: &HashedNodeRef<'a, TIdN<NodeIdentifier>>) -> Self::Marshaled {
            TypeIndex {
                lang: LangRef::<Type>::name(&Lang),
                ty: *n.get_component::<Type>().unwrap() as u16,
            }
        }
        fn type_eq(
            &self,
            n: &HashedNodeRef<'a, TIdN<NodeIdentifier>>,
            m: &HashedNodeRef<'a, TIdN<NodeIdentifier>>,
        ) -> bool {
            n.get_component::<Type>().unwrap() == m.get_component::<Type>().unwrap()
        }
    }
    impl<'a, R> TypeStore<R> for &TStore {
        type Ty = Type;
        const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

        fn resolve_type(&self, _n: &R) -> Self::Ty {
            todo!()
        }

        fn resolve_lang(&self, _n: &R) -> hyper_ast::types::LangWrapper<Self::Ty> {
            todo!()
        }

        type Marshaled = TypeIndex;

        fn marshal_type(&self, _n: &R) -> Self::Marshaled {
            todo!()
        }
        fn type_eq(&self, _n: &R, _m: &R) -> bool {
            todo!()
        }
    }
    impl<'a> JavaEnabledTypeStore<HashedNodeRef<'a, TIdN<NodeIdentifier>>> for TStore {
        // fn intern(&self, t: Type) -> Self::Ty {
        //     // T((u16::MAX - Self::Cpp as u16) | t as u16)
        //     t
        // }

        // fn resolve(&self, t: Self::Ty) -> Type {
        //     // let t = t.0 as u16;
        //     // let t = t & !TStore::MASK;
        //     // Type::resolve(t)
        //     t
        // }
    }

    impl<'a> TypeStore<HashedNodeRef<'a, NodeIdentifier>> for TStore {
        type Ty = AnyType;
        const MASK: TypeInternalSize = 0b1000_0000_0000_0000;

        fn resolve_type(&self, n: &HashedNodeRef<'a, NodeIdentifier>) -> Self::Ty {
            let t = n.get_component::<Type>().unwrap();
            as_any(t)
        }

        fn resolve_lang(
            &self,
            n: &HashedNodeRef<'a, NodeIdentifier>,
        ) -> hyper_ast::types::LangWrapper<Self::Ty> {
            todo!("{:?}", n)
        }

        type Marshaled = TypeIndex;

        fn marshal_type(&self, n: &HashedNodeRef<'a, NodeIdentifier>) -> Self::Marshaled {
            TypeIndex {
                lang: LangRef::<Type>::name(&Lang),
                ty: *n.get_component::<Type>().unwrap() as u16,
            }
        }
        fn type_eq(
            &self,
            n: &HashedNodeRef<'a, NodeIdentifier>,
            m: &HashedNodeRef<'a, NodeIdentifier>,
        ) -> bool {
            todo!("{:?} {:?}", n, m)
        }
    }
    pub fn as_any(t: &Type) -> AnyType {
        let t = <Java as hyper_ast::types::Lang<Type>>::to_u16(*t);
        let t = <Java as hyper_ast::types::Lang<Type>>::make(t);
        let t: &'static dyn HyperType = t;
        t.into()
    }
}
#[cfg(feature = "legion")]
pub use legion_impls::as_any;
pub trait JavaEnabledTypeStore<T>: TypeStore<T> {}


#[repr(u8)]
pub enum TStore {
    Java = 0,
}

impl Default for TStore {
    fn default() -> Self {
        Self::Java
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TIdN<IdN>(IdN);

impl<IdN: Clone + Eq + NodeId> NodeId for TIdN<IdN> {
    type IdN = IdN;

    fn as_id(&self) -> &Self::IdN {
        &self.0
    }

    unsafe fn from_id(id: Self::IdN) -> Self {
        Self(id)
    }

    unsafe fn from_ref_id(id: &Self::IdN) -> &Self {
        std::mem::transmute(id)
    }
}

impl<IdN: Clone + Eq + NodeId> TypedNodeId for TIdN<IdN> {
    type Ty = Type;
}

type TypeInternalSize = u16;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct T(TypeInternalSize);
pub struct Lang;
pub type Java = Lang;

impl hyper_ast::types::Lang<Type> for Java {
    fn make(t: u16) -> &'static Type {
        Lang.make(t)
    }
    fn to_u16(t: Type) -> u16 {
        Lang.to_u16(t)
    }
}
impl LangRef<Type> for Java {
    fn make(&self, t: u16) -> &'static Type {
        // unsafe { std::mem::transmute(t) }
        &S_T_L[t as usize]
    }
    fn to_u16(&self, t: Type) -> u16 {
        t as u16
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<Java>()
    }
}
impl LangRef<AnyType> for Java {
    fn make(&self, t: u16) -> &'static AnyType {
        todo!("{}", t)
    }
    fn to_u16(&self, t: AnyType) -> u16 {
        todo!("{}", t)
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<Java>()
    }
}
impl HyperType for Type {
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

    fn is_directory(&self) -> bool {
        self == &Type::Directory
    }

    fn is_file(&self) -> bool {
        self == &Type::Program
    }
    fn is_spaces(&self) -> bool {
        self == &Type::Spaces
    }
    fn is_syntax(&self) -> bool {
        self == &Type::LParen // "(",
        || self == &Type::Amp // "&",
        || self == &Type::RParen // ")",
        || self == &Type::Eq // "=",
        // || self == &Type::PlusEq // "+=",
        // || self == &Type::DashEq // "-=",
        // || self == &Type::StarEq // "*=",
        // || self == &Type::SlashEq // "/=",
        // || self == &Type::AmpEq // "&=",
        // || self == &Type::PipeEq // "|=",
        // || self == &Type::CaretEq // "^=",
        // || self == &Type::PercentEq // "%=",
        // || self == &Type::LtLtEq // "<<=",
        // || self == &Type::GtGtEq // ">>=",
        // || self == &Type::GtGtGtEq // ">>>=",
        // || self == &Type::GT // ">",
        // || self == &Type::LT // "<",
        // || self == &Type::GTEq // ">=",
        // || self == &Type::LTEq // "<=",
        // || self == &Type::EqEq // "==",
        // || self == &Type::BangEq // "!=",
        // || self == &Type::AmpAmp // "&&",
        // || self == &Type::PipePipe // "||",
        // || self == &Type::Plus // "+",
        // || self == &Type::Dash // "-",
        // || self == &Type::Star // "*",
        // || self == &Type::Slash // "/",
        // || self == &Type::Pipe // "|",
        // || self == &Type::Caret // "^",
        // || self == &Type::Percent // "%",
        // || self == &Type::LtLt // "<<",
        // || self == &Type::GtGt // ">>",
        // || self == &Type::GtGtGt // ">>>",
        // || self == &Type::Instanceof // "instanceof",
        // || self == &Type::DashGt // "->",
        || self == &Type::Comma // ",",
        || self == &Type::QMark // "?",
        || self == &Type::Colon // ":",
        // || self == &Type::Bang // "!",
        // || self == &Type::Tilde // "~",
        // || self == &Type::PlusPlus // "++",
        // || self == &Type::DashDash // "--",
        // || self == &Type::New // "new",
        || self == &Type::LBracket // "[",
        || self == &Type::RBracket // "]",
        || self == &Type::Dot // ".",
        // || self == &Type::Class // "class",
        || self == &Type::ColonColon // "::",
        // || self == &Type::Extends // "extends",
        || self == &Type::Switch // "switch",
        || self == &Type::LBrace // "{",
        || self == &Type::RBrace // "}",
        // || self == &Type::Case // "case",
        // || self == &Type::Default // "default",
        || self == &Type::SemiColon // ";",
        || self == &Type::Assert // "assert",
        || self == &Type::Do // "do",
        || self == &Type::While // "while",
        // || self == &Type::Break // "break",
        // || self == &Type::Continue // "continue",
        || self == &Type::Return // "return",
        || self == &Type::Yield // "yield",
        || self == &Type::Synchronized // "synchronized",
        || self == &Type::Throw // "throw",
        || self == &Type::Try // "try",
        // || self == &Type::Catch // "catch",
        // || self == &Type::Finally // "finally",
        || self == &Type::If // "if",
        || self == &Type::Else // "else",

        // || self == &Type::For // "for",
        // || self == &Type::At // "@",
        // || self == &Type::Open // "open",
        // || self == &Type::Module // "module",
        // || self == &Type::Requires // "requires",
        // || self == &Type::Exports // "exports",
        // || self == &Type::To // "to",
        // || self == &Type::Opens // "opens",
        // || self == &Type::Uses // "uses",
        // || self == &Type::Provides // "provides",
        // || self == &Type::With // "with",
        // || self == &Type::Transitive // "transitive",
        // || self == &Type::Static // "static",
        // || self == &Type::Package // "package",
        // || self == &Type::Import // "import",
        // || self == &Type::Enum // "enum",
        // || self == &Type::Public // "public",
        // || self == &Type::Protected // "protected",
        // || self == &Type::Private // "private",
        // || self == &Type::Abstract // "abstract",
        // || self == &Type::Final // "final",
        // || self == &Type::Strictfp // "strictfp",
        // || self == &Type::Native // "native",
        // || self == &Type::Transient // "transient",
        // || self == &Type::Volatile // "volatile",
        // || self == &Type::Implements // "implements",
        // || self == &Type::Record // "record",
        // || self == &Type::TS0 // "@interface",
        // || self == &Type::Interface // "interface",
        // || self == &Type::Byte // "byte",
        // || self == &Type::Short // "short",
        // || self == &Type::Int // "int",
        // || self == &Type::Long // "long",
        // || self == &Type::Char // "char",
        // || self == &Type::Float // "float",
        // || self == &Type::Double // "double",
        // || self == &Type::BooleanType // "boolean_type",
        // || self == &Type::VoidType // "void_type",
        // || self == &Type::DotDotDot // "...",
        // || self == &Type::Throws // "throws",
        // || self == &Type::This // "this",
        // || self == &Type::Super // "super",
    }

    fn as_shared(&self) -> hyper_ast::types::Shared {
        use hyper_ast::types::Shared;
        match self {
            x if x.is_type_declaration() => Shared::TypeDeclaration,
            Type::LineComment => Shared::Comment,
            Type::BlockComment => Shared::Comment,
            Type::Identifier => Shared::Identifier,
            Type::TypeIdentifier => Shared::Identifier,
            Type::ScopedIdentifier => Shared::Identifier,
            _ => Shared::Other,
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_static(&self) -> &'static dyn HyperType {
        todo!()
    }

    fn as_static_str(&self) -> &'static str {
        self.to_str()
    }

    fn get_lang(&self) -> hyper_ast::types::LangWrapper<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl TypeTrait for Type {
    type Lang = Java;

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
            Self::_Literal => true,
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
        self == &Type::SwitchExpression
            || self == &Type::WhileStatement
            || self == &Type::DoStatement
            || self == &Type::IfStatement
            || self == &Type::TryStatement
            || self == &Type::FinallyClause
            || self == &Type::TryWithResourcesStatement
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
            || self == &Type::EnhancedForVariable // TODO trick used to group nodes
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
        self == &Type::LineComment || self == &Type::BlockComment
    }
}
impl Type {
    pub fn literal_type(&self) -> &str {
        // TODO make the difference btw int/long and float/double
        match self {
            Self::_Literal => panic!(),
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

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

// impl<'a> TryFrom<&'a str> for Type {
//     type Error = ();

//     fn try_from(value: &'a str) -> Result<Self, Self::Error> {
//         Type::from_str(value).ok_or_else(|| ())
//     }
// }

impl<'a> From<&'a str> for Type {
    fn from(value: &'a str) -> Self {
        Type::from_str(value).unwrap()
    }
}

const COUNT: u16 = 326 + 1 + 2;
#[repr(u16)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Type {
    End,
    Identifier,
    DecimalIntegerLiteral,
    HexIntegerLiteral,
    OctalIntegerLiteral,
    BinaryIntegerLiteral,
    DecimalFloatingPointLiteral,
    HexFloatingPointLiteral,
    True,
    False,
    CharacterLiteral,
    DQuote,
    TS0,
    StringFragment,
    _MultilineStringFragmentToken1,
    _MultilineStringFragmentToken2,
    TS3,
    RBrace,
    _EscapeSequenceToken1,
    EscapeSequence,
    NullLiteral,
    LParen,
    RParen,
    Amp,
    Eq,
    PlusEq,
    DashEq,
    StarEq,
    SlashEq,
    AmpEq,
    PipeEq,
    CaretEq,
    PercentEq,
    LtLtEq,
    GtGtEq,
    GtGtGtEq,
    GT,
    LT,
    GTEq,
    LTEq,
    EqEq,
    BangEq,
    AmpAmp,
    PipePipe,
    Plus,
    Dash,
    Star,
    Slash,
    Pipe,
    Caret,
    Percent,
    LtLt,
    GtGt,
    GtGtGt,
    Instanceof,
    Final,
    DashGt,
    Comma,
    QMark,
    Colon,
    Bang,
    Tilde,
    PlusPlus,
    DashDash,
    New,
    LBracket,
    RBracket,
    Dot,
    Class,
    ColonColon,
    Extends,
    Switch,
    LBrace,
    Case,
    Default,
    UnderscorePattern,
    When,
    SemiColon,
    Assert,
    Do,
    While,
    Break,
    Continue,
    Return,
    Yield,
    Synchronized,
    Throw,
    Try,
    Catch,
    Finally,
    If,
    Else,
    For,
    At,
    Open,
    Module,
    Requires,
    Transitive,
    Static,
    Exports,
    To,
    Opens,
    Uses,
    Provides,
    With,
    Package,
    Import,
    Enum,
    Public,
    Protected,
    Private,
    Abstract,
    Strictfp,
    Native,
    Transient,
    Volatile,
    Sealed,
    TS5,
    Implements,
    Permits,
    Record,
    TS6,
    Interface,
    Byte,
    Short,
    Int,
    Long,
    Char,
    Float,
    Double,
    BooleanType,
    VoidType,
    DotDotDot,
    Throws,
    This,
    Super,
    LineComment,
    BlockComment,
    Program,
    _ToplevelStatement,
    _Literal,
    StringLiteral,
    _StringLiteral,
    _MultilineStringLiteral,
    MultilineStringFragment,
    StringInterpolation,
    _EscapeSequence,
    Expression,
    CastExpression,
    AssignmentExpression,
    BinaryExpression,
    InstanceofExpression,
    LambdaExpression,
    InferredParameters,
    TernaryExpression,
    UnaryExpression,
    UpdateExpression,
    PrimaryExpression,
    ArrayCreationExpression,
    DimensionsExpr,
    ParenthesizedExpression,
    ClassLiteral,
    ObjectCreationExpression,
    _UnqualifiedObjectCreationExpression,
    FieldAccess,
    TemplateExpression,
    ArrayAccess,
    MethodInvocation,
    ArgumentList,
    MethodReference,
    TypeArguments,
    Wildcard,
    WildcardExtends,
    WildcardSuper,
    _WildcardBounds,
    Dimensions,
    SwitchExpression,
    SwitchBlock,
    SwitchBlockStatementGroup,
    SwitchRule,
    SwitchLabel,
    Pattern,
    TypePattern,
    RecordPattern,
    RecordPatternBody,
    RecordPatternComponent,
    Guard,
    Statement,
    Block,
    ExpressionStatement,
    LabeledStatement,
    AssertStatement,
    DoStatement,
    BreakStatement,
    ContinueStatement,
    ReturnStatement,
    YieldStatement,
    SynchronizedStatement,
    ThrowStatement,
    TryStatement,
    CatchClause,
    CatchFormalParameter,
    CatchType,
    FinallyClause,
    TryWithResourcesStatement,
    ResourceSpecification,
    Resource,
    IfStatement,
    WhileStatement,
    ForStatement,
    EnhancedForStatement,
    EnhancedForVariable,
    _Annotation,
    MarkerAnnotation,
    Annotation,
    AnnotationArgumentList,
    ElementValuePair,
    _ElementValue,
    ElementValueArrayInitializer,
    Declaration,
    ModuleDeclaration,
    ModuleBody,
    ModuleDirective,
    RequiresModuleDirective,
    RequiresModifier,
    ExportsModuleDirective,
    OpensModuleDirective,
    UsesModuleDirective,
    ProvidesModuleDirective,
    PackageDeclaration,
    ImportDeclaration,
    Asterisk,
    EnumDeclaration,
    EnumBody,
    EnumBodyDeclarations,
    EnumConstant,
    ClassDeclaration,
    Modifiers,
    TypeParameters,
    TypeParameter,
    TypeBound,
    Superclass,
    SuperInterfaces,
    TypeList,
    ClassBody,
    StaticInitializer,
    ConstructorDeclaration,
    _ConstructorDeclarator,
    ConstructorBody,
    ExplicitConstructorInvocation,
    ScopedIdentifier,
    _AbsoluteName,
    ScopedAbsoluteIdentifier,
    FieldDeclaration,
    RecordDeclaration,
    AnnotationTypeDeclaration,
    AnnotationTypeBody,
    AnnotationTypeElementDeclaration,
    _DefaultValue,
    InterfaceDeclaration,
    ExtendsInterfaces,
    InterfaceBody,
    ConstantDeclaration,
    _VariableDeclaratorList,
    VariableDeclarator,
    _VariableDeclaratorId,
    ArrayInitializer,
    _Type,
    _UnannotatedType,
    AnnotatedType,
    ScopedTypeIdentifier,
    GenericType,
    ArrayType,
    IntegralType,
    FloatingPointType,
    _MethodHeader,
    _MethodDeclarator,
    FormalParameters,
    FormalParameter,
    ReceiverParameter,
    SpreadParameter,
    LocalVariableDeclaration,
    MethodDeclaration,
    CompactConstructorDeclaration,
    _ReservedIdentifier,
    ProgramRepeat1,
    _StringLiteralRepeat1,
    _MultilineStringLiteralRepeat1,
    CastExpressionRepeat1,
    InferredParametersRepeat1,
    ArrayCreationExpressionRepeat1,
    ArrayCreationExpressionRepeat2,
    ArgumentListRepeat1,
    TypeArgumentsRepeat1,
    DimensionsRepeat1,
    SwitchBlockRepeat1,
    SwitchBlockRepeat2,
    SwitchBlockStatementGroupRepeat1,
    SwitchBlockStatementGroupRepeat2,
    RecordPatternBodyRepeat1,
    TryStatementRepeat1,
    CatchTypeRepeat1,
    ResourceSpecificationRepeat1,
    ForStatementRepeat1,
    ForStatementRepeat2,
    AnnotationArgumentListRepeat1,
    ElementValueArrayInitializerRepeat1,
    ModuleBodyRepeat1,
    RequiresModuleDirectiveRepeat1,
    ExportsModuleDirectiveRepeat1,
    ProvidesModuleDirectiveRepeat1,
    EnumBodyRepeat1,
    EnumBodyDeclarationsRepeat1,
    ModifiersRepeat1,
    TypeParametersRepeat1,
    TypeBoundRepeat1,
    TypeListRepeat1,
    AnnotationTypeBodyRepeat1,
    InterfaceBodyRepeat1,
    _VariableDeclaratorListRepeat1,
    ArrayInitializerRepeat1,
    FormalParametersRepeat1,
    ReceiverParameterRepeat1,
    TypeIdentifier,
    Spaces,
    Directory,
    ERROR,
}
impl Type {
    pub fn from_u16(t: u16) -> Type {
        match t {
            0u16 => Type::End,
            1u16 => Type::Identifier,
            2u16 => Type::DecimalIntegerLiteral,
            3u16 => Type::HexIntegerLiteral,
            4u16 => Type::OctalIntegerLiteral,
            5u16 => Type::BinaryIntegerLiteral,
            6u16 => Type::DecimalFloatingPointLiteral,
            7u16 => Type::HexFloatingPointLiteral,
            8u16 => Type::True,
            9u16 => Type::False,
            10u16 => Type::CharacterLiteral,
            11u16 => Type::DQuote,
            12u16 => Type::TS0,
            13u16 => Type::StringFragment,
            14u16 => Type::_MultilineStringFragmentToken1,
            15u16 => Type::_MultilineStringFragmentToken2,
            16u16 => Type::TS3,
            17u16 => Type::RBrace,
            18u16 => Type::_EscapeSequenceToken1,
            19u16 => Type::EscapeSequence,
            20u16 => Type::NullLiteral,
            21u16 => Type::LParen,
            22u16 => Type::RParen,
            23u16 => Type::Amp,
            24u16 => Type::Eq,
            25u16 => Type::PlusEq,
            26u16 => Type::DashEq,
            27u16 => Type::StarEq,
            28u16 => Type::SlashEq,
            29u16 => Type::AmpEq,
            30u16 => Type::PipeEq,
            31u16 => Type::CaretEq,
            32u16 => Type::PercentEq,
            33u16 => Type::LtLtEq,
            34u16 => Type::GtGtEq,
            35u16 => Type::GtGtGtEq,
            36u16 => Type::GT,
            37u16 => Type::LT,
            38u16 => Type::GTEq,
            39u16 => Type::LTEq,
            40u16 => Type::EqEq,
            41u16 => Type::BangEq,
            42u16 => Type::AmpAmp,
            43u16 => Type::PipePipe,
            44u16 => Type::Plus,
            45u16 => Type::Dash,
            46u16 => Type::Star,
            47u16 => Type::Slash,
            48u16 => Type::Pipe,
            49u16 => Type::Caret,
            50u16 => Type::Percent,
            51u16 => Type::LtLt,
            52u16 => Type::GtGt,
            53u16 => Type::GtGtGt,
            54u16 => Type::Instanceof,
            55u16 => Type::Final,
            56u16 => Type::DashGt,
            57u16 => Type::Comma,
            58u16 => Type::QMark,
            59u16 => Type::Colon,
            60u16 => Type::Bang,
            61u16 => Type::Tilde,
            62u16 => Type::PlusPlus,
            63u16 => Type::DashDash,
            64u16 => Type::New,
            65u16 => Type::LBracket,
            66u16 => Type::RBracket,
            67u16 => Type::Dot,
            68u16 => Type::Class,
            69u16 => Type::ColonColon,
            70u16 => Type::Extends,
            71u16 => Type::Switch,
            72u16 => Type::LBrace,
            73u16 => Type::Case,
            74u16 => Type::Default,
            75u16 => Type::UnderscorePattern,
            76u16 => Type::When,
            77u16 => Type::SemiColon,
            78u16 => Type::Assert,
            79u16 => Type::Do,
            80u16 => Type::While,
            81u16 => Type::Break,
            82u16 => Type::Continue,
            83u16 => Type::Return,
            84u16 => Type::Yield,
            85u16 => Type::Synchronized,
            86u16 => Type::Throw,
            87u16 => Type::Try,
            88u16 => Type::Catch,
            89u16 => Type::Finally,
            90u16 => Type::If,
            91u16 => Type::Else,
            92u16 => Type::For,
            93u16 => Type::At,
            94u16 => Type::Open,
            95u16 => Type::Module,
            96u16 => Type::Requires,
            97u16 => Type::Transitive,
            98u16 => Type::Static,
            99u16 => Type::Exports,
            100u16 => Type::To,
            101u16 => Type::Opens,
            102u16 => Type::Uses,
            103u16 => Type::Provides,
            104u16 => Type::With,
            105u16 => Type::Package,
            106u16 => Type::Import,
            107u16 => Type::Enum,
            108u16 => Type::Public,
            109u16 => Type::Protected,
            110u16 => Type::Private,
            111u16 => Type::Abstract,
            112u16 => Type::Strictfp,
            113u16 => Type::Native,
            114u16 => Type::Transient,
            115u16 => Type::Volatile,
            116u16 => Type::Sealed,
            117u16 => Type::TS5,
            118u16 => Type::Implements,
            119u16 => Type::Permits,
            120u16 => Type::Record,
            121u16 => Type::TS6,
            122u16 => Type::Interface,
            123u16 => Type::Byte,
            124u16 => Type::Short,
            125u16 => Type::Int,
            126u16 => Type::Long,
            127u16 => Type::Char,
            128u16 => Type::Float,
            129u16 => Type::Double,
            130u16 => Type::BooleanType,
            131u16 => Type::VoidType,
            132u16 => Type::DotDotDot,
            133u16 => Type::Throws,
            134u16 => Type::This,
            135u16 => Type::Super,
            136u16 => Type::LineComment,
            137u16 => Type::BlockComment,
            138u16 => Type::Program,
            139u16 => Type::_ToplevelStatement,
            140u16 => Type::_Literal,
            141u16 => Type::StringLiteral,
            142u16 => Type::_StringLiteral,
            143u16 => Type::_MultilineStringLiteral,
            144u16 => Type::MultilineStringFragment,
            145u16 => Type::StringInterpolation,
            146u16 => Type::_EscapeSequence,
            147u16 => Type::Expression,
            148u16 => Type::CastExpression,
            149u16 => Type::AssignmentExpression,
            150u16 => Type::BinaryExpression,
            151u16 => Type::InstanceofExpression,
            152u16 => Type::LambdaExpression,
            153u16 => Type::InferredParameters,
            154u16 => Type::TernaryExpression,
            155u16 => Type::UnaryExpression,
            156u16 => Type::UpdateExpression,
            157u16 => Type::PrimaryExpression,
            158u16 => Type::ArrayCreationExpression,
            159u16 => Type::DimensionsExpr,
            160u16 => Type::ParenthesizedExpression,
            161u16 => Type::ClassLiteral,
            162u16 => Type::ObjectCreationExpression,
            163u16 => Type::_UnqualifiedObjectCreationExpression,
            164u16 => Type::FieldAccess,
            165u16 => Type::TemplateExpression,
            166u16 => Type::ArrayAccess,
            167u16 => Type::MethodInvocation,
            168u16 => Type::ArgumentList,
            169u16 => Type::MethodReference,
            170u16 => Type::TypeArguments,
            171u16 => Type::Wildcard,
            172u16 => Type::WildcardExtends,
            173u16 => Type::WildcardSuper,
            174u16 => Type::_WildcardBounds,
            175u16 => Type::Dimensions,
            176u16 => Type::SwitchExpression,
            177u16 => Type::SwitchBlock,
            178u16 => Type::SwitchBlockStatementGroup,
            179u16 => Type::SwitchRule,
            180u16 => Type::SwitchLabel,
            181u16 => Type::Pattern,
            182u16 => Type::TypePattern,
            183u16 => Type::RecordPattern,
            184u16 => Type::RecordPatternBody,
            185u16 => Type::RecordPatternComponent,
            186u16 => Type::Guard,
            187u16 => Type::Statement,
            188u16 => Type::Block,
            189u16 => Type::ExpressionStatement,
            190u16 => Type::LabeledStatement,
            191u16 => Type::AssertStatement,
            192u16 => Type::DoStatement,
            193u16 => Type::BreakStatement,
            194u16 => Type::ContinueStatement,
            195u16 => Type::ReturnStatement,
            196u16 => Type::YieldStatement,
            197u16 => Type::SynchronizedStatement,
            198u16 => Type::ThrowStatement,
            199u16 => Type::TryStatement,
            200u16 => Type::CatchClause,
            201u16 => Type::CatchFormalParameter,
            202u16 => Type::CatchType,
            203u16 => Type::FinallyClause,
            204u16 => Type::TryWithResourcesStatement,
            205u16 => Type::ResourceSpecification,
            206u16 => Type::Resource,
            207u16 => Type::IfStatement,
            208u16 => Type::WhileStatement,
            209u16 => Type::ForStatement,
            210u16 => Type::EnhancedForStatement,
            211u16 => Type::EnhancedForVariable,
            212u16 => Type::_Annotation,
            213u16 => Type::MarkerAnnotation,
            214u16 => Type::Annotation,
            215u16 => Type::AnnotationArgumentList,
            216u16 => Type::ElementValuePair,
            217u16 => Type::_ElementValue,
            218u16 => Type::ElementValueArrayInitializer,
            219u16 => Type::Declaration,
            220u16 => Type::ModuleDeclaration,
            221u16 => Type::ModuleBody,
            222u16 => Type::ModuleDirective,
            223u16 => Type::RequiresModuleDirective,
            224u16 => Type::RequiresModifier,
            225u16 => Type::ExportsModuleDirective,
            226u16 => Type::OpensModuleDirective,
            227u16 => Type::UsesModuleDirective,
            228u16 => Type::ProvidesModuleDirective,
            229u16 => Type::PackageDeclaration,
            230u16 => Type::ImportDeclaration,
            231u16 => Type::Asterisk,
            232u16 => Type::EnumDeclaration,
            233u16 => Type::EnumBody,
            234u16 => Type::EnumBodyDeclarations,
            235u16 => Type::EnumConstant,
            236u16 => Type::ClassDeclaration,
            237u16 => Type::Modifiers,
            238u16 => Type::TypeParameters,
            239u16 => Type::TypeParameter,
            240u16 => Type::TypeBound,
            241u16 => Type::Superclass,
            242u16 => Type::SuperInterfaces,
            243u16 => Type::TypeList,
            244u16 => Type::Permits,
            245u16 => Type::ClassBody,
            246u16 => Type::StaticInitializer,
            247u16 => Type::ConstructorDeclaration,
            248u16 => Type::_ConstructorDeclarator,
            249u16 => Type::ConstructorBody,
            250u16 => Type::ExplicitConstructorInvocation,
            251u16 => Type::ScopedIdentifier,
            252u16 => Type::_AbsoluteName,
            253u16 => Type::ScopedAbsoluteIdentifier,
            254u16 => Type::FieldDeclaration,
            255u16 => Type::RecordDeclaration,
            256u16 => Type::AnnotationTypeDeclaration,
            257u16 => Type::AnnotationTypeBody,
            258u16 => Type::AnnotationTypeElementDeclaration,
            259u16 => Type::_DefaultValue,
            260u16 => Type::InterfaceDeclaration,
            261u16 => Type::ExtendsInterfaces,
            262u16 => Type::InterfaceBody,
            263u16 => Type::ConstantDeclaration,
            264u16 => Type::_VariableDeclaratorList,
            265u16 => Type::VariableDeclarator,
            266u16 => Type::_VariableDeclaratorId,
            267u16 => Type::ArrayInitializer,
            268u16 => Type::_Type,
            269u16 => Type::_UnannotatedType,
            270u16 => Type::AnnotatedType,
            271u16 => Type::ScopedTypeIdentifier,
            272u16 => Type::GenericType,
            273u16 => Type::ArrayType,
            274u16 => Type::IntegralType,
            275u16 => Type::FloatingPointType,
            276u16 => Type::_MethodHeader,
            277u16 => Type::_MethodDeclarator,
            278u16 => Type::FormalParameters,
            279u16 => Type::FormalParameter,
            280u16 => Type::ReceiverParameter,
            281u16 => Type::SpreadParameter,
            282u16 => Type::Throws,
            283u16 => Type::LocalVariableDeclaration,
            284u16 => Type::MethodDeclaration,
            285u16 => Type::CompactConstructorDeclaration,
            286u16 => Type::_ReservedIdentifier,
            287u16 => Type::ProgramRepeat1,
            288u16 => Type::_StringLiteralRepeat1,
            289u16 => Type::_MultilineStringLiteralRepeat1,
            290u16 => Type::CastExpressionRepeat1,
            291u16 => Type::InferredParametersRepeat1,
            292u16 => Type::ArrayCreationExpressionRepeat1,
            293u16 => Type::ArrayCreationExpressionRepeat2,
            294u16 => Type::ArgumentListRepeat1,
            295u16 => Type::TypeArgumentsRepeat1,
            296u16 => Type::DimensionsRepeat1,
            297u16 => Type::SwitchBlockRepeat1,
            298u16 => Type::SwitchBlockRepeat2,
            299u16 => Type::SwitchBlockStatementGroupRepeat1,
            300u16 => Type::SwitchBlockStatementGroupRepeat2,
            301u16 => Type::RecordPatternBodyRepeat1,
            302u16 => Type::TryStatementRepeat1,
            303u16 => Type::CatchTypeRepeat1,
            304u16 => Type::ResourceSpecificationRepeat1,
            305u16 => Type::ForStatementRepeat1,
            306u16 => Type::ForStatementRepeat2,
            307u16 => Type::AnnotationArgumentListRepeat1,
            308u16 => Type::ElementValueArrayInitializerRepeat1,
            309u16 => Type::ModuleBodyRepeat1,
            310u16 => Type::RequiresModuleDirectiveRepeat1,
            311u16 => Type::ExportsModuleDirectiveRepeat1,
            312u16 => Type::ProvidesModuleDirectiveRepeat1,
            313u16 => Type::EnumBodyRepeat1,
            314u16 => Type::EnumBodyDeclarationsRepeat1,
            315u16 => Type::ModifiersRepeat1,
            316u16 => Type::TypeParametersRepeat1,
            317u16 => Type::TypeBoundRepeat1,
            318u16 => Type::TypeListRepeat1,
            319u16 => Type::AnnotationTypeBodyRepeat1,
            320u16 => Type::InterfaceBodyRepeat1,
            321u16 => Type::_VariableDeclaratorListRepeat1,
            322u16 => Type::ArrayInitializerRepeat1,
            323u16 => Type::FormalParametersRepeat1,
            324u16 => Type::ReceiverParameterRepeat1,
            325u16 => Type::TypeIdentifier,
            326u16 => Type::ERROR,
            u16::MAX => Type::ERROR,
            x => panic!("{}", x),
        }
    }
    pub fn from_str(t: &str) -> Option<Type> {
        Some(match t {
            "end" => Type::End,
            "identifier" => Type::Identifier,
            "decimal_integer_literal" => Type::DecimalIntegerLiteral,
            "hex_integer_literal" => Type::HexIntegerLiteral,
            "octal_integer_literal" => Type::OctalIntegerLiteral,
            "binary_integer_literal" => Type::BinaryIntegerLiteral,
            "decimal_floating_point_literal" => Type::DecimalFloatingPointLiteral,
            "hex_floating_point_literal" => Type::HexFloatingPointLiteral,
            "true" => Type::True,
            "false" => Type::False,
            "character_literal" => Type::CharacterLiteral,
            "\"" => Type::DQuote,
            "\"\"\"" => Type::TS0,
            "string_fragment" => Type::StringFragment,
            "_multiline_string_fragment_token1" => Type::_MultilineStringFragmentToken1,
            "_multiline_string_fragment_token2" => Type::_MultilineStringFragmentToken2,
            "\\{" => Type::TS3,
            "}" => Type::RBrace,
            "_escape_sequence_token1" => Type::_EscapeSequenceToken1,
            "escape_sequence" => Type::EscapeSequence,
            "null_literal" => Type::NullLiteral,
            "(" => Type::LParen,
            ")" => Type::RParen,
            "&" => Type::Amp,
            "=" => Type::Eq,
            "+=" => Type::PlusEq,
            "-=" => Type::DashEq,
            "*=" => Type::StarEq,
            "/=" => Type::SlashEq,
            "&=" => Type::AmpEq,
            "|=" => Type::PipeEq,
            "^=" => Type::CaretEq,
            "%=" => Type::PercentEq,
            "<<=" => Type::LtLtEq,
            ">>=" => Type::GtGtEq,
            ">>>=" => Type::GtGtGtEq,
            ">" => Type::GT,
            "<" => Type::LT,
            ">=" => Type::GTEq,
            "<=" => Type::LTEq,
            "==" => Type::EqEq,
            "!=" => Type::BangEq,
            "&&" => Type::AmpAmp,
            "||" => Type::PipePipe,
            "+" => Type::Plus,
            "-" => Type::Dash,
            "*" => Type::Star,
            "/" => Type::Slash,
            "|" => Type::Pipe,
            "^" => Type::Caret,
            "%" => Type::Percent,
            "<<" => Type::LtLt,
            ">>" => Type::GtGt,
            ">>>" => Type::GtGtGt,
            "instanceof" => Type::Instanceof,
            "final" => Type::Final,
            "->" => Type::DashGt,
            "," => Type::Comma,
            "?" => Type::QMark,
            ":" => Type::Colon,
            "!" => Type::Bang,
            "~" => Type::Tilde,
            "++" => Type::PlusPlus,
            "--" => Type::DashDash,
            "new" => Type::New,
            "[" => Type::LBracket,
            "]" => Type::RBracket,
            "." => Type::Dot,
            "class" => Type::Class,
            "::" => Type::ColonColon,
            "extends" => Type::Extends,
            "switch" => Type::Switch,
            "{" => Type::LBrace,
            "case" => Type::Case,
            "default" => Type::Default,
            "underscore_pattern" => Type::UnderscorePattern,
            "when" => Type::When,
            ";" => Type::SemiColon,
            "assert" => Type::Assert,
            "do" => Type::Do,
            "while" => Type::While,
            "break" => Type::Break,
            "continue" => Type::Continue,
            "return" => Type::Return,
            "yield" => Type::Yield,
            "synchronized" => Type::Synchronized,
            "throw" => Type::Throw,
            "try" => Type::Try,
            "catch" => Type::Catch,
            "finally" => Type::Finally,
            "if" => Type::If,
            "else" => Type::Else,
            "for" => Type::For,
            "@" => Type::At,
            "open" => Type::Open,
            "module" => Type::Module,
            "requires" => Type::Requires,
            "transitive" => Type::Transitive,
            "static" => Type::Static,
            "exports" => Type::Exports,
            "to" => Type::To,
            "opens" => Type::Opens,
            "uses" => Type::Uses,
            "provides" => Type::Provides,
            "with" => Type::With,
            "package" => Type::Package,
            "import" => Type::Import,
            "enum" => Type::Enum,
            "public" => Type::Public,
            "protected" => Type::Protected,
            "private" => Type::Private,
            "abstract" => Type::Abstract,
            "strictfp" => Type::Strictfp,
            "native" => Type::Native,
            "transient" => Type::Transient,
            "volatile" => Type::Volatile,
            "sealed" => Type::Sealed,
            "non-sealed" => Type::TS5,
            "implements" => Type::Implements,
            "permits" => Type::Permits,
            "record" => Type::Record,
            "@interface" => Type::TS6,
            "interface" => Type::Interface,
            "byte" => Type::Byte,
            "short" => Type::Short,
            "int" => Type::Int,
            "long" => Type::Long,
            "char" => Type::Char,
            "float" => Type::Float,
            "double" => Type::Double,
            "boolean_type" => Type::BooleanType,
            "void_type" => Type::VoidType,
            "..." => Type::DotDotDot,
            "throws" => Type::Throws,
            "this" => Type::This,
            "super" => Type::Super,
            "line_comment" => Type::LineComment,
            "block_comment" => Type::BlockComment,
            "program" => Type::Program,
            "_toplevel_statement" => Type::_ToplevelStatement,
            "_literal" => Type::_Literal,
            "string_literal" => Type::StringLiteral,
            "_string_literal" => Type::_StringLiteral,
            "_multiline_string_literal" => Type::_MultilineStringLiteral,
            "multiline_string_fragment" => Type::MultilineStringFragment,
            "string_interpolation" => Type::StringInterpolation,
            "_escape_sequence" => Type::_EscapeSequence,
            "expression" => Type::Expression,
            "cast_expression" => Type::CastExpression,
            "assignment_expression" => Type::AssignmentExpression,
            "binary_expression" => Type::BinaryExpression,
            "instanceof_expression" => Type::InstanceofExpression,
            "lambda_expression" => Type::LambdaExpression,
            "inferred_parameters" => Type::InferredParameters,
            "ternary_expression" => Type::TernaryExpression,
            "unary_expression" => Type::UnaryExpression,
            "update_expression" => Type::UpdateExpression,
            "primary_expression" => Type::PrimaryExpression,
            "array_creation_expression" => Type::ArrayCreationExpression,
            "dimensions_expr" => Type::DimensionsExpr,
            "parenthesized_expression" => Type::ParenthesizedExpression,
            "class_literal" => Type::ClassLiteral,
            "object_creation_expression" => Type::ObjectCreationExpression,
            "_unqualified_object_creation_expression" => Type::_UnqualifiedObjectCreationExpression,
            "field_access" => Type::FieldAccess,
            "template_expression" => Type::TemplateExpression,
            "array_access" => Type::ArrayAccess,
            "method_invocation" => Type::MethodInvocation,
            "argument_list" => Type::ArgumentList,
            "method_reference" => Type::MethodReference,
            "type_arguments" => Type::TypeArguments,
            "wildcard" => Type::Wildcard,
            "wildcard_extends" => Type::WildcardExtends,
            "wildcard_super" => Type::WildcardSuper,
            "_wildcard_bounds" => Type::_WildcardBounds,
            "dimensions" => Type::Dimensions,
            "switch_expression" => Type::SwitchExpression,
            "switch_block" => Type::SwitchBlock,
            "switch_block_statement_group" => Type::SwitchBlockStatementGroup,
            "switch_rule" => Type::SwitchRule,
            "switch_label" => Type::SwitchLabel,
            "pattern" => Type::Pattern,
            "type_pattern" => Type::TypePattern,
            "record_pattern" => Type::RecordPattern,
            "record_pattern_body" => Type::RecordPatternBody,
            "record_pattern_component" => Type::RecordPatternComponent,
            "guard" => Type::Guard,
            "statement" => Type::Statement,
            "block" => Type::Block,
            "expression_statement" => Type::ExpressionStatement,
            "labeled_statement" => Type::LabeledStatement,
            "assert_statement" => Type::AssertStatement,
            "do_statement" => Type::DoStatement,
            "break_statement" => Type::BreakStatement,
            "continue_statement" => Type::ContinueStatement,
            "return_statement" => Type::ReturnStatement,
            "yield_statement" => Type::YieldStatement,
            "synchronized_statement" => Type::SynchronizedStatement,
            "throw_statement" => Type::ThrowStatement,
            "try_statement" => Type::TryStatement,
            "catch_clause" => Type::CatchClause,
            "catch_formal_parameter" => Type::CatchFormalParameter,
            "catch_type" => Type::CatchType,
            "finally_clause" => Type::FinallyClause,
            "try_with_resources_statement" => Type::TryWithResourcesStatement,
            "resource_specification" => Type::ResourceSpecification,
            "resource" => Type::Resource,
            "if_statement" => Type::IfStatement,
            "while_statement" => Type::WhileStatement,
            "for_statement" => Type::ForStatement,
            "enhanced_for_statement" => Type::EnhancedForStatement,
            "_enhanced_for_variable" => Type::EnhancedForVariable,
            "_annotation" => Type::_Annotation,
            "marker_annotation" => Type::MarkerAnnotation,
            "annotation" => Type::Annotation,
            "annotation_argument_list" => Type::AnnotationArgumentList,
            "element_value_pair" => Type::ElementValuePair,
            "_element_value" => Type::_ElementValue,
            "element_value_array_initializer" => Type::ElementValueArrayInitializer,
            "declaration" => Type::Declaration,
            "module_declaration" => Type::ModuleDeclaration,
            "module_body" => Type::ModuleBody,
            "module_directive" => Type::ModuleDirective,
            "requires_module_directive" => Type::RequiresModuleDirective,
            "requires_modifier" => Type::RequiresModifier,
            "exports_module_directive" => Type::ExportsModuleDirective,
            "opens_module_directive" => Type::OpensModuleDirective,
            "uses_module_directive" => Type::UsesModuleDirective,
            "provides_module_directive" => Type::ProvidesModuleDirective,
            "package_declaration" => Type::PackageDeclaration,
            "import_declaration" => Type::ImportDeclaration,
            "asterisk" => Type::Asterisk,
            "enum_declaration" => Type::EnumDeclaration,
            "enum_body" => Type::EnumBody,
            "enum_body_declarations" => Type::EnumBodyDeclarations,
            "enum_constant" => Type::EnumConstant,
            "class_declaration" => Type::ClassDeclaration,
            "modifiers" => Type::Modifiers,
            "type_parameters" => Type::TypeParameters,
            "type_parameter" => Type::TypeParameter,
            "type_bound" => Type::TypeBound,
            "superclass" => Type::Superclass,
            "super_interfaces" => Type::SuperInterfaces,
            "type_list" => Type::TypeList,
            "class_body" => Type::ClassBody,
            "static_initializer" => Type::StaticInitializer,
            "constructor_declaration" => Type::ConstructorDeclaration,
            "_constructor_declarator" => Type::_ConstructorDeclarator,
            "constructor_body" => Type::ConstructorBody,
            "explicit_constructor_invocation" => Type::ExplicitConstructorInvocation,
            "scoped_identifier" => Type::ScopedIdentifier,
            "_absolute_name" => Type::_AbsoluteName,
            "scoped_absolute_identifier" => Type::ScopedAbsoluteIdentifier,
            "field_declaration" => Type::FieldDeclaration,
            "record_declaration" => Type::RecordDeclaration,
            "annotation_type_declaration" => Type::AnnotationTypeDeclaration,
            "annotation_type_body" => Type::AnnotationTypeBody,
            "annotation_type_element_declaration" => Type::AnnotationTypeElementDeclaration,
            "_default_value" => Type::_DefaultValue,
            "interface_declaration" => Type::InterfaceDeclaration,
            "extends_interfaces" => Type::ExtendsInterfaces,
            "interface_body" => Type::InterfaceBody,
            "constant_declaration" => Type::ConstantDeclaration,
            "_variable_declarator_list" => Type::_VariableDeclaratorList,
            "variable_declarator" => Type::VariableDeclarator,
            "_variable_declarator_id" => Type::_VariableDeclaratorId,
            "array_initializer" => Type::ArrayInitializer,
            "_type" => Type::_Type,
            "_unannotated_type" => Type::_UnannotatedType,
            "annotated_type" => Type::AnnotatedType,
            "scoped_type_identifier" => Type::ScopedTypeIdentifier,
            "generic_type" => Type::GenericType,
            "array_type" => Type::ArrayType,
            "integral_type" => Type::IntegralType,
            "floating_point_type" => Type::FloatingPointType,
            "_method_header" => Type::_MethodHeader,
            "_method_declarator" => Type::_MethodDeclarator,
            "formal_parameters" => Type::FormalParameters,
            "formal_parameter" => Type::FormalParameter,
            "receiver_parameter" => Type::ReceiverParameter,
            "spread_parameter" => Type::SpreadParameter,
            "local_variable_declaration" => Type::LocalVariableDeclaration,
            "method_declaration" => Type::MethodDeclaration,
            "compact_constructor_declaration" => Type::CompactConstructorDeclaration,
            "_reserved_identifier" => Type::_ReservedIdentifier,
            "program_repeat1" => Type::ProgramRepeat1,
            "_string_literal_repeat1" => Type::_StringLiteralRepeat1,
            "_multiline_string_literal_repeat1" => Type::_MultilineStringLiteralRepeat1,
            "cast_expression_repeat1" => Type::CastExpressionRepeat1,
            "inferred_parameters_repeat1" => Type::InferredParametersRepeat1,
            "array_creation_expression_repeat1" => Type::ArrayCreationExpressionRepeat1,
            "array_creation_expression_repeat2" => Type::ArrayCreationExpressionRepeat2,
            "argument_list_repeat1" => Type::ArgumentListRepeat1,
            "type_arguments_repeat1" => Type::TypeArgumentsRepeat1,
            "dimensions_repeat1" => Type::DimensionsRepeat1,
            "switch_block_repeat1" => Type::SwitchBlockRepeat1,
            "switch_block_repeat2" => Type::SwitchBlockRepeat2,
            "switch_block_statement_group_repeat1" => Type::SwitchBlockStatementGroupRepeat1,
            "switch_block_statement_group_repeat2" => Type::SwitchBlockStatementGroupRepeat2,
            "record_pattern_body_repeat1" => Type::RecordPatternBodyRepeat1,
            "try_statement_repeat1" => Type::TryStatementRepeat1,
            "catch_type_repeat1" => Type::CatchTypeRepeat1,
            "resource_specification_repeat1" => Type::ResourceSpecificationRepeat1,
            "for_statement_repeat1" => Type::ForStatementRepeat1,
            "for_statement_repeat2" => Type::ForStatementRepeat2,
            "annotation_argument_list_repeat1" => Type::AnnotationArgumentListRepeat1,
            "element_value_array_initializer_repeat1" => Type::ElementValueArrayInitializerRepeat1,
            "module_body_repeat1" => Type::ModuleBodyRepeat1,
            "requires_module_directive_repeat1" => Type::RequiresModuleDirectiveRepeat1,
            "exports_module_directive_repeat1" => Type::ExportsModuleDirectiveRepeat1,
            "provides_module_directive_repeat1" => Type::ProvidesModuleDirectiveRepeat1,
            "enum_body_repeat1" => Type::EnumBodyRepeat1,
            "enum_body_declarations_repeat1" => Type::EnumBodyDeclarationsRepeat1,
            "modifiers_repeat1" => Type::ModifiersRepeat1,
            "type_parameters_repeat1" => Type::TypeParametersRepeat1,
            "type_bound_repeat1" => Type::TypeBoundRepeat1,
            "type_list_repeat1" => Type::TypeListRepeat1,
            "annotation_type_body_repeat1" => Type::AnnotationTypeBodyRepeat1,
            "interface_body_repeat1" => Type::InterfaceBodyRepeat1,
            "_variable_declarator_list_repeat1" => Type::_VariableDeclaratorListRepeat1,
            "array_initializer_repeat1" => Type::ArrayInitializerRepeat1,
            "formal_parameters_repeat1" => Type::FormalParametersRepeat1,
            "receiver_parameter_repeat1" => Type::ReceiverParameterRepeat1,
            "type_identifier" => Type::TypeIdentifier,
            "Spaces" => Type::Spaces,
            "Directory" => Type::Directory,
            "ERROR" => Type::ERROR,
            x => return None,
        })
    }
    pub fn to_str(&self) -> &'static str {
        match self {
            Type::End => "end",
            Type::Identifier => "identifier",
            Type::DecimalIntegerLiteral => "decimal_integer_literal",
            Type::HexIntegerLiteral => "hex_integer_literal",
            Type::OctalIntegerLiteral => "octal_integer_literal",
            Type::BinaryIntegerLiteral => "binary_integer_literal",
            Type::DecimalFloatingPointLiteral => "decimal_floating_point_literal",
            Type::HexFloatingPointLiteral => "hex_floating_point_literal",
            Type::True => "true",
            Type::False => "false",
            Type::CharacterLiteral => "character_literal",
            Type::DQuote => "\"",
            Type::TS0 => "\"\"\"",
            Type::StringFragment => "string_fragment",
            Type::_MultilineStringFragmentToken1 => "_multiline_string_fragment_token1",
            Type::_MultilineStringFragmentToken2 => "_multiline_string_fragment_token2",
            Type::TS3 => "\\{",
            Type::RBrace => "}",
            Type::_EscapeSequenceToken1 => "_escape_sequence_token1",
            Type::EscapeSequence => "escape_sequence",
            Type::NullLiteral => "null_literal",
            Type::LParen => "(",
            Type::RParen => ")",
            Type::Amp => "&",
            Type::Eq => "=",
            Type::PlusEq => "+=",
            Type::DashEq => "-=",
            Type::StarEq => "*=",
            Type::SlashEq => "/=",
            Type::AmpEq => "&=",
            Type::PipeEq => "|=",
            Type::CaretEq => "^=",
            Type::PercentEq => "%=",
            Type::LtLtEq => "<<=",
            Type::GtGtEq => ">>=",
            Type::GtGtGtEq => ">>>=",
            Type::GT => ">",
            Type::LT => "<",
            Type::GTEq => ">=",
            Type::LTEq => "<=",
            Type::EqEq => "==",
            Type::BangEq => "!=",
            Type::AmpAmp => "&&",
            Type::PipePipe => "||",
            Type::Plus => "+",
            Type::Dash => "-",
            Type::Star => "*",
            Type::Slash => "/",
            Type::Pipe => "|",
            Type::Caret => "^",
            Type::Percent => "%",
            Type::LtLt => "<<",
            Type::GtGt => ">>",
            Type::GtGtGt => ">>>",
            Type::Instanceof => "instanceof",
            Type::Final => "final",
            Type::DashGt => "->",
            Type::Comma => ",",
            Type::QMark => "?",
            Type::Colon => ":",
            Type::Bang => "!",
            Type::Tilde => "~",
            Type::PlusPlus => "++",
            Type::DashDash => "--",
            Type::New => "new",
            Type::LBracket => "[",
            Type::RBracket => "]",
            Type::Dot => ".",
            Type::Class => "class",
            Type::ColonColon => "::",
            Type::Extends => "extends",
            Type::Switch => "switch",
            Type::LBrace => "{",
            Type::Case => "case",
            Type::Default => "default",
            Type::UnderscorePattern => "underscore_pattern",
            Type::When => "when",
            Type::SemiColon => ";",
            Type::Assert => "assert",
            Type::Do => "do",
            Type::While => "while",
            Type::Break => "break",
            Type::Continue => "continue",
            Type::Return => "return",
            Type::Yield => "yield",
            Type::Synchronized => "synchronized",
            Type::Throw => "throw",
            Type::Try => "try",
            Type::Catch => "catch",
            Type::Finally => "finally",
            Type::If => "if",
            Type::Else => "else",
            Type::For => "for",
            Type::At => "@",
            Type::Open => "open",
            Type::Module => "module",
            Type::Requires => "requires",
            Type::Transitive => "transitive",
            Type::Static => "static",
            Type::Exports => "exports",
            Type::To => "to",
            Type::Opens => "opens",
            Type::Uses => "uses",
            Type::Provides => "provides",
            Type::With => "with",
            Type::Package => "package",
            Type::Import => "import",
            Type::Enum => "enum",
            Type::Public => "public",
            Type::Protected => "protected",
            Type::Private => "private",
            Type::Abstract => "abstract",
            Type::Strictfp => "strictfp",
            Type::Native => "native",
            Type::Transient => "transient",
            Type::Volatile => "volatile",
            Type::Sealed => "sealed",
            Type::TS5 => "non-sealed",
            Type::Implements => "implements",
            Type::Permits => "permits",
            Type::Record => "record",
            Type::TS6 => "@interface",
            Type::Interface => "interface",
            Type::Byte => "byte",
            Type::Short => "short",
            Type::Int => "int",
            Type::Long => "long",
            Type::Char => "char",
            Type::Float => "float",
            Type::Double => "double",
            Type::BooleanType => "boolean_type",
            Type::VoidType => "void_type",
            Type::DotDotDot => "...",
            Type::Throws => "throws",
            Type::This => "this",
            Type::Super => "super",
            Type::LineComment => "line_comment",
            Type::BlockComment => "block_comment",
            Type::Program => "program",
            Type::_ToplevelStatement => "_toplevel_statement",
            Type::_Literal => "_literal",
            Type::StringLiteral => "string_literal",
            Type::_StringLiteral => "_string_literal",
            Type::_MultilineStringLiteral => "_multiline_string_literal",
            Type::MultilineStringFragment => "multiline_string_fragment",
            Type::StringInterpolation => "string_interpolation",
            Type::_EscapeSequence => "_escape_sequence",
            Type::Expression => "expression",
            Type::CastExpression => "cast_expression",
            Type::AssignmentExpression => "assignment_expression",
            Type::BinaryExpression => "binary_expression",
            Type::InstanceofExpression => "instanceof_expression",
            Type::LambdaExpression => "lambda_expression",
            Type::InferredParameters => "inferred_parameters",
            Type::TernaryExpression => "ternary_expression",
            Type::UnaryExpression => "unary_expression",
            Type::UpdateExpression => "update_expression",
            Type::PrimaryExpression => "primary_expression",
            Type::ArrayCreationExpression => "array_creation_expression",
            Type::DimensionsExpr => "dimensions_expr",
            Type::ParenthesizedExpression => "parenthesized_expression",
            Type::ClassLiteral => "class_literal",
            Type::ObjectCreationExpression => "object_creation_expression",
            Type::_UnqualifiedObjectCreationExpression => "_unqualified_object_creation_expression",
            Type::FieldAccess => "field_access",
            Type::TemplateExpression => "template_expression",
            Type::ArrayAccess => "array_access",
            Type::MethodInvocation => "method_invocation",
            Type::ArgumentList => "argument_list",
            Type::MethodReference => "method_reference",
            Type::TypeArguments => "type_arguments",
            Type::Wildcard => "wildcard",
            Type::WildcardExtends => "wildcard_extends",
            Type::WildcardSuper => "wildcard_super",
            Type::_WildcardBounds => "_wildcard_bounds",
            Type::Dimensions => "dimensions",
            Type::SwitchExpression => "switch_expression",
            Type::SwitchBlock => "switch_block",
            Type::SwitchBlockStatementGroup => "switch_block_statement_group",
            Type::SwitchRule => "switch_rule",
            Type::SwitchLabel => "switch_label",
            Type::Pattern => "pattern",
            Type::TypePattern => "type_pattern",
            Type::RecordPattern => "record_pattern",
            Type::RecordPatternBody => "record_pattern_body",
            Type::RecordPatternComponent => "record_pattern_component",
            Type::Guard => "guard",
            Type::Statement => "statement",
            Type::Block => "block",
            Type::ExpressionStatement => "expression_statement",
            Type::LabeledStatement => "labeled_statement",
            Type::AssertStatement => "assert_statement",
            Type::DoStatement => "do_statement",
            Type::BreakStatement => "break_statement",
            Type::ContinueStatement => "continue_statement",
            Type::ReturnStatement => "return_statement",
            Type::YieldStatement => "yield_statement",
            Type::SynchronizedStatement => "synchronized_statement",
            Type::ThrowStatement => "throw_statement",
            Type::TryStatement => "try_statement",
            Type::CatchClause => "catch_clause",
            Type::CatchFormalParameter => "catch_formal_parameter",
            Type::CatchType => "catch_type",
            Type::FinallyClause => "finally_clause",
            Type::TryWithResourcesStatement => "try_with_resources_statement",
            Type::ResourceSpecification => "resource_specification",
            Type::Resource => "resource",
            Type::IfStatement => "if_statement",
            Type::WhileStatement => "while_statement",
            Type::ForStatement => "for_statement",
            Type::EnhancedForStatement => "enhanced_for_statement",
            Type::EnhancedForVariable => "_enhanced_for_variable",
            Type::_Annotation => "_annotation",
            Type::MarkerAnnotation => "marker_annotation",
            Type::Annotation => "annotation",
            Type::AnnotationArgumentList => "annotation_argument_list",
            Type::ElementValuePair => "element_value_pair",
            Type::_ElementValue => "_element_value",
            Type::ElementValueArrayInitializer => "element_value_array_initializer",
            Type::Declaration => "declaration",
            Type::ModuleDeclaration => "module_declaration",
            Type::ModuleBody => "module_body",
            Type::ModuleDirective => "module_directive",
            Type::RequiresModuleDirective => "requires_module_directive",
            Type::RequiresModifier => "requires_modifier",
            Type::ExportsModuleDirective => "exports_module_directive",
            Type::OpensModuleDirective => "opens_module_directive",
            Type::UsesModuleDirective => "uses_module_directive",
            Type::ProvidesModuleDirective => "provides_module_directive",
            Type::PackageDeclaration => "package_declaration",
            Type::ImportDeclaration => "import_declaration",
            Type::Asterisk => "asterisk",
            Type::EnumDeclaration => "enum_declaration",
            Type::EnumBody => "enum_body",
            Type::EnumBodyDeclarations => "enum_body_declarations",
            Type::EnumConstant => "enum_constant",
            Type::ClassDeclaration => "class_declaration",
            Type::Modifiers => "modifiers",
            Type::TypeParameters => "type_parameters",
            Type::TypeParameter => "type_parameter",
            Type::TypeBound => "type_bound",
            Type::Superclass => "superclass",
            Type::SuperInterfaces => "super_interfaces",
            Type::TypeList => "type_list",
            Type::ClassBody => "class_body",
            Type::StaticInitializer => "static_initializer",
            Type::ConstructorDeclaration => "constructor_declaration",
            Type::_ConstructorDeclarator => "_constructor_declarator",
            Type::ConstructorBody => "constructor_body",
            Type::ExplicitConstructorInvocation => "explicit_constructor_invocation",
            Type::ScopedIdentifier => "scoped_identifier",
            Type::_AbsoluteName => "_absolute_name",
            Type::ScopedAbsoluteIdentifier => "scoped_absolute_identifier",
            Type::FieldDeclaration => "field_declaration",
            Type::RecordDeclaration => "record_declaration",
            Type::AnnotationTypeDeclaration => "annotation_type_declaration",
            Type::AnnotationTypeBody => "annotation_type_body",
            Type::AnnotationTypeElementDeclaration => "annotation_type_element_declaration",
            Type::_DefaultValue => "_default_value",
            Type::InterfaceDeclaration => "interface_declaration",
            Type::ExtendsInterfaces => "extends_interfaces",
            Type::InterfaceBody => "interface_body",
            Type::ConstantDeclaration => "constant_declaration",
            Type::_VariableDeclaratorList => "_variable_declarator_list",
            Type::VariableDeclarator => "variable_declarator",
            Type::_VariableDeclaratorId => "_variable_declarator_id",
            Type::ArrayInitializer => "array_initializer",
            Type::_Type => "_type",
            Type::_UnannotatedType => "_unannotated_type",
            Type::AnnotatedType => "annotated_type",
            Type::ScopedTypeIdentifier => "scoped_type_identifier",
            Type::GenericType => "generic_type",
            Type::ArrayType => "array_type",
            Type::IntegralType => "integral_type",
            Type::FloatingPointType => "floating_point_type",
            Type::_MethodHeader => "_method_header",
            Type::_MethodDeclarator => "_method_declarator",
            Type::FormalParameters => "formal_parameters",
            Type::FormalParameter => "formal_parameter",
            Type::ReceiverParameter => "receiver_parameter",
            Type::SpreadParameter => "spread_parameter",
            Type::LocalVariableDeclaration => "local_variable_declaration",
            Type::MethodDeclaration => "method_declaration",
            Type::CompactConstructorDeclaration => "compact_constructor_declaration",
            Type::_ReservedIdentifier => "_reserved_identifier",
            Type::ProgramRepeat1 => "program_repeat1",
            Type::_StringLiteralRepeat1 => "_string_literal_repeat1",
            Type::_MultilineStringLiteralRepeat1 => "_multiline_string_literal_repeat1",
            Type::CastExpressionRepeat1 => "cast_expression_repeat1",
            Type::InferredParametersRepeat1 => "inferred_parameters_repeat1",
            Type::ArrayCreationExpressionRepeat1 => "array_creation_expression_repeat1",
            Type::ArrayCreationExpressionRepeat2 => "array_creation_expression_repeat2",
            Type::ArgumentListRepeat1 => "argument_list_repeat1",
            Type::TypeArgumentsRepeat1 => "type_arguments_repeat1",
            Type::DimensionsRepeat1 => "dimensions_repeat1",
            Type::SwitchBlockRepeat1 => "switch_block_repeat1",
            Type::SwitchBlockRepeat2 => "switch_block_repeat2",
            Type::SwitchBlockStatementGroupRepeat1 => "switch_block_statement_group_repeat1",
            Type::SwitchBlockStatementGroupRepeat2 => "switch_block_statement_group_repeat2",
            Type::RecordPatternBodyRepeat1 => "record_pattern_body_repeat1",
            Type::TryStatementRepeat1 => "try_statement_repeat1",
            Type::CatchTypeRepeat1 => "catch_type_repeat1",
            Type::ResourceSpecificationRepeat1 => "resource_specification_repeat1",
            Type::ForStatementRepeat1 => "for_statement_repeat1",
            Type::ForStatementRepeat2 => "for_statement_repeat2",
            Type::AnnotationArgumentListRepeat1 => "annotation_argument_list_repeat1",
            Type::ElementValueArrayInitializerRepeat1 => "element_value_array_initializer_repeat1",
            Type::ModuleBodyRepeat1 => "module_body_repeat1",
            Type::RequiresModuleDirectiveRepeat1 => "requires_module_directive_repeat1",
            Type::ExportsModuleDirectiveRepeat1 => "exports_module_directive_repeat1",
            Type::ProvidesModuleDirectiveRepeat1 => "provides_module_directive_repeat1",
            Type::EnumBodyRepeat1 => "enum_body_repeat1",
            Type::EnumBodyDeclarationsRepeat1 => "enum_body_declarations_repeat1",
            Type::ModifiersRepeat1 => "modifiers_repeat1",
            Type::TypeParametersRepeat1 => "type_parameters_repeat1",
            Type::TypeBoundRepeat1 => "type_bound_repeat1",
            Type::TypeListRepeat1 => "type_list_repeat1",
            Type::AnnotationTypeBodyRepeat1 => "annotation_type_body_repeat1",
            Type::InterfaceBodyRepeat1 => "interface_body_repeat1",
            Type::_VariableDeclaratorListRepeat1 => "_variable_declarator_list_repeat1",
            Type::ArrayInitializerRepeat1 => "array_initializer_repeat1",
            Type::FormalParametersRepeat1 => "formal_parameters_repeat1",
            Type::ReceiverParameterRepeat1 => "receiver_parameter_repeat1",
            Type::TypeIdentifier => "type_identifier",
            Type::Spaces => "Spaces",
            Type::Directory => "Directory",
            Type::ERROR => "ERROR",
        }
    }
}

const S_T_L: &'static [Type] = &[
    Type::End,
    Type::Identifier,
    Type::DecimalIntegerLiteral,
    Type::HexIntegerLiteral,
    Type::OctalIntegerLiteral,
    Type::BinaryIntegerLiteral,
    Type::DecimalFloatingPointLiteral,
    Type::HexFloatingPointLiteral,
    Type::True,
    Type::False,
    Type::CharacterLiteral,
    Type::DQuote,
    Type::TS0,
    Type::StringFragment,
    Type::_MultilineStringFragmentToken1,
    Type::_MultilineStringFragmentToken2,
    Type::TS3,
    Type::RBrace,
    Type::_EscapeSequenceToken1,
    Type::EscapeSequence,
    Type::NullLiteral,
    Type::LParen,
    Type::RParen,
    Type::Amp,
    Type::Eq,
    Type::PlusEq,
    Type::DashEq,
    Type::StarEq,
    Type::SlashEq,
    Type::AmpEq,
    Type::PipeEq,
    Type::CaretEq,
    Type::PercentEq,
    Type::LtLtEq,
    Type::GtGtEq,
    Type::GtGtGtEq,
    Type::GT,
    Type::LT,
    Type::GTEq,
    Type::LTEq,
    Type::EqEq,
    Type::BangEq,
    Type::AmpAmp,
    Type::PipePipe,
    Type::Plus,
    Type::Dash,
    Type::Star,
    Type::Slash,
    Type::Pipe,
    Type::Caret,
    Type::Percent,
    Type::LtLt,
    Type::GtGt,
    Type::GtGtGt,
    Type::Instanceof,
    Type::Final,
    Type::DashGt,
    Type::Comma,
    Type::QMark,
    Type::Colon,
    Type::Bang,
    Type::Tilde,
    Type::PlusPlus,
    Type::DashDash,
    Type::New,
    Type::LBracket,
    Type::RBracket,
    Type::Dot,
    Type::Class,
    Type::ColonColon,
    Type::Extends,
    Type::Switch,
    Type::LBrace,
    Type::Case,
    Type::Default,
    Type::UnderscorePattern,
    Type::When,
    Type::SemiColon,
    Type::Assert,
    Type::Do,
    Type::While,
    Type::Break,
    Type::Continue,
    Type::Return,
    Type::Yield,
    Type::Synchronized,
    Type::Throw,
    Type::Try,
    Type::Catch,
    Type::Finally,
    Type::If,
    Type::Else,
    Type::For,
    Type::At,
    Type::Open,
    Type::Module,
    Type::Requires,
    Type::Transitive,
    Type::Static,
    Type::Exports,
    Type::To,
    Type::Opens,
    Type::Uses,
    Type::Provides,
    Type::With,
    Type::Package,
    Type::Import,
    Type::Enum,
    Type::Public,
    Type::Protected,
    Type::Private,
    Type::Abstract,
    Type::Strictfp,
    Type::Native,
    Type::Transient,
    Type::Volatile,
    Type::Sealed,
    Type::TS5,
    Type::Implements,
    Type::Permits,
    Type::Record,
    Type::TS6,
    Type::Interface,
    Type::Byte,
    Type::Short,
    Type::Int,
    Type::Long,
    Type::Char,
    Type::Float,
    Type::Double,
    Type::BooleanType,
    Type::VoidType,
    Type::DotDotDot,
    Type::Throws,
    Type::This,
    Type::Super,
    Type::LineComment,
    Type::BlockComment,
    Type::Program,
    Type::_ToplevelStatement,
    Type::_Literal,
    Type::StringLiteral,
    Type::_StringLiteral,
    Type::_MultilineStringLiteral,
    Type::MultilineStringFragment,
    Type::StringInterpolation,
    Type::_EscapeSequence,
    Type::Expression,
    Type::CastExpression,
    Type::AssignmentExpression,
    Type::BinaryExpression,
    Type::InstanceofExpression,
    Type::LambdaExpression,
    Type::InferredParameters,
    Type::TernaryExpression,
    Type::UnaryExpression,
    Type::UpdateExpression,
    Type::PrimaryExpression,
    Type::ArrayCreationExpression,
    Type::DimensionsExpr,
    Type::ParenthesizedExpression,
    Type::ClassLiteral,
    Type::ObjectCreationExpression,
    Type::_UnqualifiedObjectCreationExpression,
    Type::FieldAccess,
    Type::TemplateExpression,
    Type::ArrayAccess,
    Type::MethodInvocation,
    Type::ArgumentList,
    Type::MethodReference,
    Type::TypeArguments,
    Type::Wildcard,
    Type::WildcardExtends,
    Type::WildcardSuper,
    Type::_WildcardBounds,
    Type::Dimensions,
    Type::SwitchExpression,
    Type::SwitchBlock,
    Type::SwitchBlockStatementGroup,
    Type::SwitchRule,
    Type::SwitchLabel,
    Type::Pattern,
    Type::TypePattern,
    Type::RecordPattern,
    Type::RecordPatternBody,
    Type::RecordPatternComponent,
    Type::Guard,
    Type::Statement,
    Type::Block,
    Type::ExpressionStatement,
    Type::LabeledStatement,
    Type::AssertStatement,
    Type::DoStatement,
    Type::BreakStatement,
    Type::ContinueStatement,
    Type::ReturnStatement,
    Type::YieldStatement,
    Type::SynchronizedStatement,
    Type::ThrowStatement,
    Type::TryStatement,
    Type::CatchClause,
    Type::CatchFormalParameter,
    Type::CatchType,
    Type::FinallyClause,
    Type::TryWithResourcesStatement,
    Type::ResourceSpecification,
    Type::Resource,
    Type::IfStatement,
    Type::WhileStatement,
    Type::ForStatement,
    Type::EnhancedForStatement,
    Type::EnhancedForVariable,
    Type::_Annotation,
    Type::MarkerAnnotation,
    Type::Annotation,
    Type::AnnotationArgumentList,
    Type::ElementValuePair,
    Type::_ElementValue,
    Type::ElementValueArrayInitializer,
    Type::Declaration,
    Type::ModuleDeclaration,
    Type::ModuleBody,
    Type::ModuleDirective,
    Type::RequiresModuleDirective,
    Type::RequiresModifier,
    Type::ExportsModuleDirective,
    Type::OpensModuleDirective,
    Type::UsesModuleDirective,
    Type::ProvidesModuleDirective,
    Type::PackageDeclaration,
    Type::ImportDeclaration,
    Type::Asterisk,
    Type::EnumDeclaration,
    Type::EnumBody,
    Type::EnumBodyDeclarations,
    Type::EnumConstant,
    Type::ClassDeclaration,
    Type::Modifiers,
    Type::TypeParameters,
    Type::TypeParameter,
    Type::TypeBound,
    Type::Superclass,
    Type::SuperInterfaces,
    Type::TypeList,
    Type::ClassBody,
    Type::StaticInitializer,
    Type::ConstructorDeclaration,
    Type::_ConstructorDeclarator,
    Type::ConstructorBody,
    Type::ExplicitConstructorInvocation,
    Type::ScopedIdentifier,
    Type::_AbsoluteName,
    Type::ScopedAbsoluteIdentifier,
    Type::FieldDeclaration,
    Type::RecordDeclaration,
    Type::AnnotationTypeDeclaration,
    Type::AnnotationTypeBody,
    Type::AnnotationTypeElementDeclaration,
    Type::_DefaultValue,
    Type::InterfaceDeclaration,
    Type::ExtendsInterfaces,
    Type::InterfaceBody,
    Type::ConstantDeclaration,
    Type::_VariableDeclaratorList,
    Type::VariableDeclarator,
    Type::_VariableDeclaratorId,
    Type::ArrayInitializer,
    Type::_Type,
    Type::_UnannotatedType,
    Type::AnnotatedType,
    Type::ScopedTypeIdentifier,
    Type::GenericType,
    Type::ArrayType,
    Type::IntegralType,
    Type::FloatingPointType,
    Type::_MethodHeader,
    Type::_MethodDeclarator,
    Type::FormalParameters,
    Type::FormalParameter,
    Type::ReceiverParameter,
    Type::SpreadParameter,
    Type::LocalVariableDeclaration,
    Type::MethodDeclaration,
    Type::CompactConstructorDeclaration,
    Type::_ReservedIdentifier,
    Type::ProgramRepeat1,
    Type::_StringLiteralRepeat1,
    Type::_MultilineStringLiteralRepeat1,
    Type::CastExpressionRepeat1,
    Type::InferredParametersRepeat1,
    Type::ArrayCreationExpressionRepeat1,
    Type::ArrayCreationExpressionRepeat2,
    Type::ArgumentListRepeat1,
    Type::TypeArgumentsRepeat1,
    Type::DimensionsRepeat1,
    Type::SwitchBlockRepeat1,
    Type::SwitchBlockRepeat2,
    Type::SwitchBlockStatementGroupRepeat1,
    Type::SwitchBlockStatementGroupRepeat2,
    Type::RecordPatternBodyRepeat1,
    Type::TryStatementRepeat1,
    Type::CatchTypeRepeat1,
    Type::ResourceSpecificationRepeat1,
    Type::ForStatementRepeat1,
    Type::ForStatementRepeat2,
    Type::AnnotationArgumentListRepeat1,
    Type::ElementValueArrayInitializerRepeat1,
    Type::ModuleBodyRepeat1,
    Type::RequiresModuleDirectiveRepeat1,
    Type::ExportsModuleDirectiveRepeat1,
    Type::ProvidesModuleDirectiveRepeat1,
    Type::EnumBodyRepeat1,
    Type::EnumBodyDeclarationsRepeat1,
    Type::ModifiersRepeat1,
    Type::TypeParametersRepeat1,
    Type::TypeBoundRepeat1,
    Type::TypeListRepeat1,
    Type::AnnotationTypeBodyRepeat1,
    Type::InterfaceBodyRepeat1,
    Type::_VariableDeclaratorListRepeat1,
    Type::ArrayInitializerRepeat1,
    Type::FormalParametersRepeat1,
    Type::ReceiverParameterRepeat1,
    Type::TypeIdentifier,
    Type::Spaces,
    Type::Directory,
    Type::ERROR,
];
