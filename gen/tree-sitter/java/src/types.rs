use std::fmt::Display;

use hyper_ast::{
    store::defaults::NodeIdentifier,
    tree_gen::parser::NodeWithU16TypeId,
    types::{AnyType, HyperType, Lang, LangRef, NodeId, TypeStore, TypeTrait, TypedNodeId},
};

pub struct Single {
    mask: TypeInternalSize,
    lang: TypeInternalSize,
}

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
            todo!()
        }

        fn resolve_lang(
            &self,
            n: &HashedNodeRef<'a, TIdN<NodeIdentifier>>,
        ) -> hyper_ast::types::LangWrapper<Self::Ty> {
            todo!()
        }

        type Marshaled = TypeIndex;

        fn marshal_type(&self, n: &HashedNodeRef<'a, TIdN<NodeIdentifier>>) -> Self::Marshaled {
            TypeIndex {
                lang: LangRef::<Type>::name(&Java),
                ty: *n.get_component::<Type>().unwrap() as u16,
            }
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
            todo!()
        }

        fn resolve_lang(
            &self,
            n: &HashedNodeRef<'a, NodeIdentifier>,
        ) -> hyper_ast::types::LangWrapper<Self::Ty> {
            todo!()
        }

        type Marshaled = TypeIndex;

        fn marshal_type(&self, n: &HashedNodeRef<'a, NodeIdentifier>) -> Self::Marshaled {
            TypeIndex {
                lang: LangRef::<Type>::name(&Java),
                ty: *n.get_component::<Type>().unwrap() as u16,
            }
        }
    }
}
pub trait JavaEnabledTypeStore<T>: TypeStore<T> {}

// impl Single {
//     fn from<TS: JavaEnabledTypeStore>(value: TS) -> Self {
//         Self {
//             mask: TS::MASK,
//             lang: todo!(),
//         }
//     }
// }

impl Type {
    fn resolve(t: u16) -> Self {
        assert!(t < COUNT);
        unsafe { std::mem::transmute(t) }
    }
}

#[repr(u8)]
pub(crate) enum TStore {
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
pub struct Java;

impl Lang<Type> for Java {
    fn make(t: u16) -> &'static Type {
        Java.make(t)
    }
    fn to_u16(t: Type) -> u16 {
        Java.to_u16(t)
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
        todo!()
    }
    fn to_u16(&self, t: AnyType) -> u16 {
        todo!()
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<Java>()
    }
}
impl HyperType for Type {
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
            Type::ClassDeclaration => Shared::TypeDeclaration,
            Type::InterfaceDeclaration => Shared::TypeDeclaration,
            Type::EnumDeclaration => Shared::TypeDeclaration,
            Type::Comment => Shared::Comment,
            Type::Identifier => Shared::Identifier,
            Type::TypeIdentifier => Shared::Identifier,
            Type::ScopedIdentifier => Shared::Identifier,
            _ => Shared::Other,
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
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
        self == &Type::Comment
    }
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

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

const COUNT: u16 = 286 + 1 + 2;
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
    StringLiteral,
    NullLiteral,
    LParen,
    Amp,
    RParen,
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
    RBrace,
    Case,
    Default,
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
    Exports,
    To,
    Opens,
    Uses,
    Provides,
    With,
    Transitive,
    Static,
    Package,
    Import,
    Enum,
    Public,
    Protected,
    Private,
    Abstract,
    Final,
    Strictfp,
    Native,
    Transient,
    Volatile,
    Implements,
    Record,
    TS0,
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
    Comment,
    Program,
    Literal,
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
    TS1,
    FieldAccess,
    ArrayAccess,
    MethodInvocation,
    ArgumentList,
    MethodReference,
    TypeArguments,
    Wildcard,
    WildcardExtends,
    WildcardSuper,
    Dimensions,
    SwitchExpression,
    SwitchStatement,
    SwitchBlock,
    SwitchBlockStatementGroup,
    SwitchRule,
    SwitchLabel,
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
    TryWithResourcesExtendedStatement,
    TryWithResourcesStatement,
    ResourceSpecification,
    Resource,
    IfStatement,
    WhileStatement,
    ForStatement,
    EnhancedForStatement,
    EnhancedForVariable,
    TS2,
    MarkerAnnotation,
    Annotation,
    AnnotationArgumentList,
    ElementValuePair,
    TS3,
    ElementValueArrayInitializer,
    TypeDeclaration,
    ModuleDeclaration,
    ModuleBody,
    ModuleDirective,
    RequiresModifier,
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
    ClassBody,
    StaticInitializer,
    ConstructorDeclaration,
    TS4,
    ConstructorBody,
    ExplicitConstructorInvocation,
    ScopedIdentifier,
    ScopedAbsoluteIdentifier,
    FieldDeclaration,
    RecordDeclaration,
    AnnotationTypeDeclaration,
    AnnotationTypeBody,
    AnnotationTypeElementDeclaration,
    TS5,
    InterfaceDeclaration,
    ExtendsInterfaces,
    InterfaceBody,
    ConstantDeclaration,
    TS6,
    VariableDeclarator,
    TS7,
    ArrayInitializer,
    Type,
    UnannotatedType,
    AnnotatedType,
    ScopedTypeIdentifier,
    GenericType,
    ArrayType,
    IntegralType,
    FloatingPointType,
    TS8,
    TS9,
    FormalParameters,
    FormalParameter,
    ReceiverParameter,
    SpreadParameter,
    LocalVariableDeclaration,
    MethodDeclaration,
    ProgramRepeat1,
    ProgramRepeat2,
    ProgramRepeat3,
    CastExpressionRepeat1,
    InferredParametersRepeat1,
    ArrayCreationExpressionRepeat1,
    DimensionsExprRepeat1,
    ArgumentListRepeat1,
    TypeArgumentsRepeat1,
    DimensionsRepeat1,
    SwitchBlockRepeat1,
    SwitchBlockRepeat2,
    SwitchBlockStatementGroupRepeat1,
    TryStatementRepeat1,
    CatchTypeRepeat1,
    ResourceSpecificationRepeat1,
    ForStatementRepeat1,
    ForStatementRepeat2,
    AnnotationArgumentListRepeat1,
    ElementValueArrayInitializerRepeat1,
    ModuleBodyRepeat1,
    ModuleDirectiveRepeat1,
    ModuleDirectiveRepeat2,
    EnumBodyRepeat1,
    EnumBodyDeclarationsRepeat1,
    ModifiersRepeat1,
    TypeParametersRepeat1,
    TypeBoundRepeat1,
    TS10,
    AnnotationTypeBodyRepeat1,
    InterfaceBodyRepeat1,
    TS11,
    ArrayInitializerRepeat1,
    FormalParametersRepeat1,
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
            11u16 => Type::StringLiteral,
            12u16 => Type::NullLiteral,
            13u16 => Type::LParen,
            14u16 => Type::Amp,
            15u16 => Type::RParen,
            16u16 => Type::Eq,
            17u16 => Type::PlusEq,
            18u16 => Type::DashEq,
            19u16 => Type::StarEq,
            20u16 => Type::SlashEq,
            21u16 => Type::AmpEq,
            22u16 => Type::PipeEq,
            23u16 => Type::CaretEq,
            24u16 => Type::PercentEq,
            25u16 => Type::LtLtEq,
            26u16 => Type::GtGtEq,
            27u16 => Type::GtGtGtEq,
            28u16 => Type::GT,
            29u16 => Type::LT,
            30u16 => Type::GTEq,
            31u16 => Type::LTEq,
            32u16 => Type::EqEq,
            33u16 => Type::BangEq,
            34u16 => Type::AmpAmp,
            35u16 => Type::PipePipe,
            36u16 => Type::Plus,
            37u16 => Type::Dash,
            38u16 => Type::Star,
            39u16 => Type::Slash,
            40u16 => Type::Pipe,
            41u16 => Type::Caret,
            42u16 => Type::Percent,
            43u16 => Type::LtLt,
            44u16 => Type::GtGt,
            45u16 => Type::GtGtGt,
            46u16 => Type::Instanceof,
            47u16 => Type::DashGt,
            48u16 => Type::Comma,
            49u16 => Type::QMark,
            50u16 => Type::Colon,
            51u16 => Type::Bang,
            52u16 => Type::Tilde,
            53u16 => Type::PlusPlus,
            54u16 => Type::DashDash,
            55u16 => Type::New,
            56u16 => Type::LBracket,
            57u16 => Type::RBracket,
            58u16 => Type::Dot,
            59u16 => Type::Class,
            60u16 => Type::ColonColon,
            61u16 => Type::Extends,
            62u16 => Type::Switch,
            63u16 => Type::LBrace,
            64u16 => Type::RBrace,
            65u16 => Type::Case,
            66u16 => Type::Default,
            67u16 => Type::SemiColon,
            68u16 => Type::Assert,
            69u16 => Type::Do,
            70u16 => Type::While,
            71u16 => Type::Break,
            72u16 => Type::Continue,
            73u16 => Type::Return,
            74u16 => Type::Yield,
            75u16 => Type::Synchronized,
            76u16 => Type::Throw,
            77u16 => Type::Try,
            78u16 => Type::Catch,
            79u16 => Type::Finally,
            80u16 => Type::If,
            81u16 => Type::Else,
            82u16 => Type::For,
            83u16 => Type::At,
            84u16 => Type::Open,
            85u16 => Type::Module,
            86u16 => Type::Requires,
            87u16 => Type::Exports,
            88u16 => Type::To,
            89u16 => Type::Opens,
            90u16 => Type::Uses,
            91u16 => Type::Provides,
            92u16 => Type::With,
            93u16 => Type::Transitive,
            94u16 => Type::Static,
            95u16 => Type::Package,
            96u16 => Type::Import,
            97u16 => Type::Enum,
            98u16 => Type::Public,
            99u16 => Type::Protected,
            100u16 => Type::Private,
            101u16 => Type::Abstract,
            102u16 => Type::Final,
            103u16 => Type::Strictfp,
            104u16 => Type::Native,
            105u16 => Type::Transient,
            106u16 => Type::Volatile,
            107u16 => Type::Implements,
            108u16 => Type::Record,
            109u16 => Type::TS0,
            110u16 => Type::Interface,
            111u16 => Type::Byte,
            112u16 => Type::Short,
            113u16 => Type::Int,
            114u16 => Type::Long,
            115u16 => Type::Char,
            116u16 => Type::Float,
            117u16 => Type::Double,
            118u16 => Type::BooleanType,
            119u16 => Type::VoidType,
            120u16 => Type::DotDotDot,
            121u16 => Type::Throws,
            122u16 => Type::This,
            123u16 => Type::Super,
            124u16 => Type::Comment,
            125u16 => Type::Program,
            126u16 => Type::Literal,
            127u16 => Type::Expression,
            128u16 => Type::CastExpression,
            129u16 => Type::AssignmentExpression,
            130u16 => Type::BinaryExpression,
            131u16 => Type::InstanceofExpression,
            132u16 => Type::LambdaExpression,
            133u16 => Type::InferredParameters,
            134u16 => Type::TernaryExpression,
            135u16 => Type::UnaryExpression,
            136u16 => Type::UpdateExpression,
            137u16 => Type::PrimaryExpression,
            138u16 => Type::ArrayCreationExpression,
            139u16 => Type::DimensionsExpr,
            140u16 => Type::ParenthesizedExpression,
            141u16 => Type::ClassLiteral,
            142u16 => Type::ObjectCreationExpression,
            143u16 => Type::TS1,
            144u16 => Type::FieldAccess,
            145u16 => Type::ArrayAccess,
            146u16 => Type::MethodInvocation,
            147u16 => Type::ArgumentList,
            148u16 => Type::MethodReference,
            149u16 => Type::TypeArguments,
            150u16 => Type::Wildcard,
            151u16 => Type::WildcardExtends,
            152u16 => Type::WildcardSuper,
            153u16 => Type::Dimensions,
            154u16 => Type::SwitchExpression,
            155u16 => Type::SwitchStatement,
            156u16 => Type::SwitchBlock,
            157u16 => Type::SwitchBlockStatementGroup,
            158u16 => Type::SwitchRule,
            159u16 => Type::SwitchLabel,
            160u16 => Type::Statement,
            161u16 => Type::Block,
            162u16 => Type::ExpressionStatement,
            163u16 => Type::LabeledStatement,
            164u16 => Type::AssertStatement,
            165u16 => Type::DoStatement,
            166u16 => Type::BreakStatement,
            167u16 => Type::ContinueStatement,
            168u16 => Type::ReturnStatement,
            169u16 => Type::YieldStatement,
            170u16 => Type::SynchronizedStatement,
            171u16 => Type::ThrowStatement,
            172u16 => Type::TryStatement,
            173u16 => Type::CatchClause,
            174u16 => Type::CatchFormalParameter,
            175u16 => Type::CatchType,
            176u16 => Type::FinallyClause,
            177u16 => Type::TryWithResourcesExtendedStatement,
            178u16 => Type::TryWithResourcesStatement,
            179u16 => Type::ResourceSpecification,
            180u16 => Type::Resource,
            181u16 => Type::IfStatement,
            182u16 => Type::WhileStatement,
            183u16 => Type::ForStatement,
            184u16 => Type::EnhancedForStatement,
            185u16 => Type::EnhancedForVariable,
            186u16 => Type::TS2,
            187u16 => Type::MarkerAnnotation,
            188u16 => Type::Annotation,
            189u16 => Type::AnnotationArgumentList,
            190u16 => Type::ElementValuePair,
            191u16 => Type::TS3,
            192u16 => Type::ElementValueArrayInitializer,
            193u16 => Type::TypeDeclaration,
            194u16 => Type::ModuleDeclaration,
            195u16 => Type::ModuleBody,
            196u16 => Type::ModuleDirective,
            197u16 => Type::RequiresModifier,
            198u16 => Type::PackageDeclaration,
            199u16 => Type::ImportDeclaration,
            200u16 => Type::Asterisk,
            201u16 => Type::EnumDeclaration,
            202u16 => Type::EnumBody,
            203u16 => Type::EnumBodyDeclarations,
            204u16 => Type::EnumConstant,
            205u16 => Type::ClassDeclaration,
            206u16 => Type::Modifiers,
            207u16 => Type::TypeParameters,
            208u16 => Type::TypeParameter,
            209u16 => Type::TypeBound,
            210u16 => Type::Superclass,
            211u16 => Type::SuperInterfaces,
            212u16 => Type::ClassBody,
            213u16 => Type::StaticInitializer,
            214u16 => Type::ConstructorDeclaration,
            215u16 => Type::TS4,
            216u16 => Type::ConstructorBody,
            217u16 => Type::ExplicitConstructorInvocation,
            218u16 => Type::ScopedIdentifier,
            219u16 => Type::ScopedAbsoluteIdentifier,
            220u16 => Type::FieldDeclaration,
            221u16 => Type::RecordDeclaration,
            222u16 => Type::AnnotationTypeDeclaration,
            223u16 => Type::AnnotationTypeBody,
            224u16 => Type::AnnotationTypeElementDeclaration,
            225u16 => Type::TS5,
            226u16 => Type::InterfaceDeclaration,
            227u16 => Type::ExtendsInterfaces,
            228u16 => Type::InterfaceBody,
            229u16 => Type::ConstantDeclaration,
            230u16 => Type::TS6,
            231u16 => Type::VariableDeclarator,
            232u16 => Type::TS7,
            233u16 => Type::ArrayInitializer,
            234u16 => Type::Type,
            235u16 => Type::UnannotatedType,
            236u16 => Type::AnnotatedType,
            237u16 => Type::ScopedTypeIdentifier,
            238u16 => Type::GenericType,
            239u16 => Type::ArrayType,
            240u16 => Type::IntegralType,
            241u16 => Type::FloatingPointType,
            242u16 => Type::TS8,
            243u16 => Type::TS9,
            244u16 => Type::FormalParameters,
            245u16 => Type::FormalParameter,
            246u16 => Type::ReceiverParameter,
            247u16 => Type::SpreadParameter,
            248u16 => Type::Throws,
            249u16 => Type::LocalVariableDeclaration,
            250u16 => Type::MethodDeclaration,
            251u16 => Type::ProgramRepeat1,
            252u16 => Type::ProgramRepeat2,
            253u16 => Type::ProgramRepeat3,
            254u16 => Type::CastExpressionRepeat1,
            255u16 => Type::InferredParametersRepeat1,
            256u16 => Type::ArrayCreationExpressionRepeat1,
            257u16 => Type::DimensionsExprRepeat1,
            258u16 => Type::ArgumentListRepeat1,
            259u16 => Type::TypeArgumentsRepeat1,
            260u16 => Type::DimensionsRepeat1,
            261u16 => Type::SwitchBlockRepeat1,
            262u16 => Type::SwitchBlockRepeat2,
            263u16 => Type::SwitchBlockStatementGroupRepeat1,
            264u16 => Type::TryStatementRepeat1,
            265u16 => Type::CatchTypeRepeat1,
            266u16 => Type::ResourceSpecificationRepeat1,
            267u16 => Type::ForStatementRepeat1,
            268u16 => Type::ForStatementRepeat2,
            269u16 => Type::AnnotationArgumentListRepeat1,
            270u16 => Type::ElementValueArrayInitializerRepeat1,
            271u16 => Type::ModuleBodyRepeat1,
            272u16 => Type::ModuleDirectiveRepeat1,
            273u16 => Type::ModuleDirectiveRepeat2,
            274u16 => Type::EnumBodyRepeat1,
            275u16 => Type::EnumBodyDeclarationsRepeat1,
            276u16 => Type::ModifiersRepeat1,
            277u16 => Type::TypeParametersRepeat1,
            278u16 => Type::TypeBoundRepeat1,
            279u16 => Type::TS10,
            280u16 => Type::AnnotationTypeBodyRepeat1,
            281u16 => Type::InterfaceBodyRepeat1,
            282u16 => Type::TS11,
            283u16 => Type::ArrayInitializerRepeat1,
            284u16 => Type::FormalParametersRepeat1,
            285u16 => Type::TypeIdentifier,
            // 286u16 => Type::ERROR,
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
            "string_literal" => Type::StringLiteral,
            "null_literal" => Type::NullLiteral,
            "(" => Type::LParen,
            "&" => Type::Amp,
            ")" => Type::RParen,
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
            "}" => Type::RBrace,
            "case" => Type::Case,
            "default" => Type::Default,
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
            "exports" => Type::Exports,
            "to" => Type::To,
            "opens" => Type::Opens,
            "uses" => Type::Uses,
            "provides" => Type::Provides,
            "with" => Type::With,
            "transitive" => Type::Transitive,
            "static" => Type::Static,
            "package" => Type::Package,
            "import" => Type::Import,
            "enum" => Type::Enum,
            "public" => Type::Public,
            "protected" => Type::Protected,
            "private" => Type::Private,
            "abstract" => Type::Abstract,
            "final" => Type::Final,
            "strictfp" => Type::Strictfp,
            "native" => Type::Native,
            "transient" => Type::Transient,
            "volatile" => Type::Volatile,
            "implements" => Type::Implements,
            "record" => Type::Record,
            "@interface" => Type::TS0,
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
            "comment" => Type::Comment,
            "program" => Type::Program,
            "_literal" => Type::Literal,
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
            "_unqualified_object_creation_expression" => Type::TS1,
            "field_access" => Type::FieldAccess,
            "array_access" => Type::ArrayAccess,
            "method_invocation" => Type::MethodInvocation,
            "argument_list" => Type::ArgumentList,
            "method_reference" => Type::MethodReference,
            "type_arguments" => Type::TypeArguments,
            "wildcard" => Type::Wildcard,
            "wildcard_extends" => Type::WildcardExtends,
            "wildcard_super" => Type::WildcardSuper,
            "dimensions" => Type::Dimensions,
            "switch_expression" => Type::SwitchExpression,
            "switch_statement" => Type::SwitchStatement,
            "switch_block" => Type::SwitchBlock,
            "switch_block_statement_group" => Type::SwitchBlockStatementGroup,
            "switch_rule" => Type::SwitchRule,
            "switch_label" => Type::SwitchLabel,
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
            "try_with_resources_extended_statement" => Type::TryWithResourcesExtendedStatement,
            "try_with_resources_statement" => Type::TryWithResourcesStatement,
            "resource_specification" => Type::ResourceSpecification,
            "resource" => Type::Resource,
            "if_statement" => Type::IfStatement,
            "while_statement" => Type::WhileStatement,
            "for_statement" => Type::ForStatement,
            "enhanced_for_statement" => Type::EnhancedForStatement,
            "enhanced_for_variable" => Type::EnhancedForVariable,
            "_annotation" => Type::TS2,
            "marker_annotation" => Type::MarkerAnnotation,
            "annotation" => Type::Annotation,
            "annotation_argument_list" => Type::AnnotationArgumentList,
            "element_value_pair" => Type::ElementValuePair,
            "_element_value" => Type::TS3,
            "element_value_array_initializer" => Type::ElementValueArrayInitializer,
            "_type_declaration" => Type::TypeDeclaration,
            "module_declaration" => Type::ModuleDeclaration,
            "module_body" => Type::ModuleBody,
            "module_directive" => Type::ModuleDirective,
            "requires_modifier" => Type::RequiresModifier,
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
            "class_body" => Type::ClassBody,
            "static_initializer" => Type::StaticInitializer,
            "constructor_declaration" => Type::ConstructorDeclaration,
            "_constructor_declarator" => Type::TS4,
            "constructor_body" => Type::ConstructorBody,
            "explicit_constructor_invocation" => Type::ExplicitConstructorInvocation,
            "scoped_identifier" => Type::ScopedIdentifier,
            "scoped_absolute_identifier" => Type::ScopedAbsoluteIdentifier,
            "field_declaration" => Type::FieldDeclaration,
            "record_declaration" => Type::RecordDeclaration,
            "annotation_type_declaration" => Type::AnnotationTypeDeclaration,
            "annotation_type_body" => Type::AnnotationTypeBody,
            "annotation_type_element_declaration" => Type::AnnotationTypeElementDeclaration,
            "_default_value" => Type::TS5,
            "interface_declaration" => Type::InterfaceDeclaration,
            "extends_interfaces" => Type::ExtendsInterfaces,
            "interface_body" => Type::InterfaceBody,
            "constant_declaration" => Type::ConstantDeclaration,
            "_variable_declarator_list" => Type::TS6,
            "variable_declarator" => Type::VariableDeclarator,
            "_variable_declarator_id" => Type::TS7,
            "array_initializer" => Type::ArrayInitializer,
            "_type" => Type::Type,
            "_unannotated_type" => Type::UnannotatedType,
            "annotated_type" => Type::AnnotatedType,
            "scoped_type_identifier" => Type::ScopedTypeIdentifier,
            "generic_type" => Type::GenericType,
            "array_type" => Type::ArrayType,
            "integral_type" => Type::IntegralType,
            "floating_point_type" => Type::FloatingPointType,
            "_method_header" => Type::TS8,
            "_method_declarator" => Type::TS9,
            "formal_parameters" => Type::FormalParameters,
            "formal_parameter" => Type::FormalParameter,
            "receiver_parameter" => Type::ReceiverParameter,
            "spread_parameter" => Type::SpreadParameter,
            "local_variable_declaration" => Type::LocalVariableDeclaration,
            "method_declaration" => Type::MethodDeclaration,
            "program_repeat1" => Type::ProgramRepeat1,
            "program_repeat2" => Type::ProgramRepeat2,
            "program_repeat3" => Type::ProgramRepeat3,
            "cast_expression_repeat1" => Type::CastExpressionRepeat1,
            "inferred_parameters_repeat1" => Type::InferredParametersRepeat1,
            "array_creation_expression_repeat1" => Type::ArrayCreationExpressionRepeat1,
            "dimensions_expr_repeat1" => Type::DimensionsExprRepeat1,
            "argument_list_repeat1" => Type::ArgumentListRepeat1,
            "type_arguments_repeat1" => Type::TypeArgumentsRepeat1,
            "dimensions_repeat1" => Type::DimensionsRepeat1,
            "switch_block_repeat1" => Type::SwitchBlockRepeat1,
            "switch_block_repeat2" => Type::SwitchBlockRepeat2,
            "switch_block_statement_group_repeat1" => Type::SwitchBlockStatementGroupRepeat1,
            "try_statement_repeat1" => Type::TryStatementRepeat1,
            "catch_type_repeat1" => Type::CatchTypeRepeat1,
            "resource_specification_repeat1" => Type::ResourceSpecificationRepeat1,
            "for_statement_repeat1" => Type::ForStatementRepeat1,
            "for_statement_repeat2" => Type::ForStatementRepeat2,
            "annotation_argument_list_repeat1" => Type::AnnotationArgumentListRepeat1,
            "element_value_array_initializer_repeat1" => Type::ElementValueArrayInitializerRepeat1,
            "module_body_repeat1" => Type::ModuleBodyRepeat1,
            "module_directive_repeat1" => Type::ModuleDirectiveRepeat1,
            "module_directive_repeat2" => Type::ModuleDirectiveRepeat2,
            "enum_body_repeat1" => Type::EnumBodyRepeat1,
            "enum_body_declarations_repeat1" => Type::EnumBodyDeclarationsRepeat1,
            "modifiers_repeat1" => Type::ModifiersRepeat1,
            "type_parameters_repeat1" => Type::TypeParametersRepeat1,
            "type_bound_repeat1" => Type::TypeBoundRepeat1,
            "_interface_type_list_repeat1" => Type::TS10,
            "annotation_type_body_repeat1" => Type::AnnotationTypeBodyRepeat1,
            "interface_body_repeat1" => Type::InterfaceBodyRepeat1,
            "_variable_declarator_list_repeat1" => Type::TS11,
            "array_initializer_repeat1" => Type::ArrayInitializerRepeat1,
            "formal_parameters_repeat1" => Type::FormalParametersRepeat1,
            "type_identifier" => Type::TypeIdentifier,
            "Spaces" => Type::Spaces,
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
            Type::StringLiteral => "string_literal",
            Type::NullLiteral => "null_literal",
            Type::LParen => "(",
            Type::Amp => "&",
            Type::RParen => ")",
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
            Type::RBrace => "}",
            Type::Case => "case",
            Type::Default => "default",
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
            Type::Exports => "exports",
            Type::To => "to",
            Type::Opens => "opens",
            Type::Uses => "uses",
            Type::Provides => "provides",
            Type::With => "with",
            Type::Transitive => "transitive",
            Type::Static => "static",
            Type::Package => "package",
            Type::Import => "import",
            Type::Enum => "enum",
            Type::Public => "public",
            Type::Protected => "protected",
            Type::Private => "private",
            Type::Abstract => "abstract",
            Type::Final => "final",
            Type::Strictfp => "strictfp",
            Type::Native => "native",
            Type::Transient => "transient",
            Type::Volatile => "volatile",
            Type::Implements => "implements",
            Type::Record => "record",
            Type::TS0 => "@interface",
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
            Type::Comment => "comment",
            Type::Program => "program",
            Type::Literal => "_literal",
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
            Type::TS1 => "_unqualified_object_creation_expression",
            Type::FieldAccess => "field_access",
            Type::ArrayAccess => "array_access",
            Type::MethodInvocation => "method_invocation",
            Type::ArgumentList => "argument_list",
            Type::MethodReference => "method_reference",
            Type::TypeArguments => "type_arguments",
            Type::Wildcard => "wildcard",
            Type::WildcardExtends => "wildcard_extends",
            Type::WildcardSuper => "wildcard_super",
            Type::Dimensions => "dimensions",
            Type::SwitchExpression => "switch_expression",
            Type::SwitchStatement => "switch_statement",
            Type::SwitchBlock => "switch_block",
            Type::SwitchBlockStatementGroup => "switch_block_statement_group",
            Type::SwitchRule => "switch_rule",
            Type::SwitchLabel => "switch_label",
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
            Type::TryWithResourcesExtendedStatement => "try_with_resources_extended_statement",
            Type::TryWithResourcesStatement => "try_with_resources_statement",
            Type::ResourceSpecification => "resource_specification",
            Type::Resource => "resource",
            Type::IfStatement => "if_statement",
            Type::WhileStatement => "while_statement",
            Type::ForStatement => "for_statement",
            Type::EnhancedForStatement => "enhanced_for_statement",
            Type::EnhancedForVariable => "enhanced_for_variable",
            Type::TS2 => "_annotation",
            Type::MarkerAnnotation => "marker_annotation",
            Type::Annotation => "annotation",
            Type::AnnotationArgumentList => "annotation_argument_list",
            Type::ElementValuePair => "element_value_pair",
            Type::TS3 => "_element_value",
            Type::ElementValueArrayInitializer => "element_value_array_initializer",
            Type::TypeDeclaration => "_type_declaration",
            Type::ModuleDeclaration => "module_declaration",
            Type::ModuleBody => "module_body",
            Type::ModuleDirective => "module_directive",
            Type::RequiresModifier => "requires_modifier",
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
            Type::ClassBody => "class_body",
            Type::StaticInitializer => "static_initializer",
            Type::ConstructorDeclaration => "constructor_declaration",
            Type::TS4 => "_constructor_declarator",
            Type::ConstructorBody => "constructor_body",
            Type::ExplicitConstructorInvocation => "explicit_constructor_invocation",
            Type::ScopedIdentifier => "scoped_identifier",
            Type::ScopedAbsoluteIdentifier => "scoped_absolute_identifier",
            Type::FieldDeclaration => "field_declaration",
            Type::RecordDeclaration => "record_declaration",
            Type::AnnotationTypeDeclaration => "annotation_type_declaration",
            Type::AnnotationTypeBody => "annotation_type_body",
            Type::AnnotationTypeElementDeclaration => "annotation_type_element_declaration",
            Type::TS5 => "_default_value",
            Type::InterfaceDeclaration => "interface_declaration",
            Type::ExtendsInterfaces => "extends_interfaces",
            Type::InterfaceBody => "interface_body",
            Type::ConstantDeclaration => "constant_declaration",
            Type::TS6 => "_variable_declarator_list",
            Type::VariableDeclarator => "variable_declarator",
            Type::TS7 => "_variable_declarator_id",
            Type::ArrayInitializer => "array_initializer",
            Type::Type => "_type",
            Type::UnannotatedType => "_unannotated_type",
            Type::AnnotatedType => "annotated_type",
            Type::ScopedTypeIdentifier => "scoped_type_identifier",
            Type::GenericType => "generic_type",
            Type::ArrayType => "array_type",
            Type::IntegralType => "integral_type",
            Type::FloatingPointType => "floating_point_type",
            Type::TS8 => "_method_header",
            Type::TS9 => "_method_declarator",
            Type::FormalParameters => "formal_parameters",
            Type::FormalParameter => "formal_parameter",
            Type::ReceiverParameter => "receiver_parameter",
            Type::SpreadParameter => "spread_parameter",
            Type::LocalVariableDeclaration => "local_variable_declaration",
            Type::MethodDeclaration => "method_declaration",
            Type::ProgramRepeat1 => "program_repeat1",
            Type::ProgramRepeat2 => "program_repeat2",
            Type::ProgramRepeat3 => "program_repeat3",
            Type::CastExpressionRepeat1 => "cast_expression_repeat1",
            Type::InferredParametersRepeat1 => "inferred_parameters_repeat1",
            Type::ArrayCreationExpressionRepeat1 => "array_creation_expression_repeat1",
            Type::DimensionsExprRepeat1 => "dimensions_expr_repeat1",
            Type::ArgumentListRepeat1 => "argument_list_repeat1",
            Type::TypeArgumentsRepeat1 => "type_arguments_repeat1",
            Type::DimensionsRepeat1 => "dimensions_repeat1",
            Type::SwitchBlockRepeat1 => "switch_block_repeat1",
            Type::SwitchBlockRepeat2 => "switch_block_repeat2",
            Type::SwitchBlockStatementGroupRepeat1 => "switch_block_statement_group_repeat1",
            Type::TryStatementRepeat1 => "try_statement_repeat1",
            Type::CatchTypeRepeat1 => "catch_type_repeat1",
            Type::ResourceSpecificationRepeat1 => "resource_specification_repeat1",
            Type::ForStatementRepeat1 => "for_statement_repeat1",
            Type::ForStatementRepeat2 => "for_statement_repeat2",
            Type::AnnotationArgumentListRepeat1 => "annotation_argument_list_repeat1",
            Type::ElementValueArrayInitializerRepeat1 => "element_value_array_initializer_repeat1",
            Type::ModuleBodyRepeat1 => "module_body_repeat1",
            Type::ModuleDirectiveRepeat1 => "module_directive_repeat1",
            Type::ModuleDirectiveRepeat2 => "module_directive_repeat2",
            Type::EnumBodyRepeat1 => "enum_body_repeat1",
            Type::EnumBodyDeclarationsRepeat1 => "enum_body_declarations_repeat1",
            Type::ModifiersRepeat1 => "modifiers_repeat1",
            Type::TypeParametersRepeat1 => "type_parameters_repeat1",
            Type::TypeBoundRepeat1 => "type_bound_repeat1",
            Type::TS10 => "_interface_type_list_repeat1",
            Type::AnnotationTypeBodyRepeat1 => "annotation_type_body_repeat1",
            Type::InterfaceBodyRepeat1 => "interface_body_repeat1",
            Type::TS11 => "_variable_declarator_list_repeat1",
            Type::ArrayInitializerRepeat1 => "array_initializer_repeat1",
            Type::FormalParametersRepeat1 => "formal_parameters_repeat1",
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
    Type::StringLiteral,
    Type::NullLiteral,
    Type::LParen,
    Type::Amp,
    Type::RParen,
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
    Type::RBrace,
    Type::Case,
    Type::Default,
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
    Type::Exports,
    Type::To,
    Type::Opens,
    Type::Uses,
    Type::Provides,
    Type::With,
    Type::Transitive,
    Type::Static,
    Type::Package,
    Type::Import,
    Type::Enum,
    Type::Public,
    Type::Protected,
    Type::Private,
    Type::Abstract,
    Type::Final,
    Type::Strictfp,
    Type::Native,
    Type::Transient,
    Type::Volatile,
    Type::Implements,
    Type::Record,
    Type::TS0,
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
    Type::Comment,
    Type::Program,
    Type::Literal,
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
    Type::TS1,
    Type::FieldAccess,
    Type::ArrayAccess,
    Type::MethodInvocation,
    Type::ArgumentList,
    Type::MethodReference,
    Type::TypeArguments,
    Type::Wildcard,
    Type::WildcardExtends,
    Type::WildcardSuper,
    Type::Dimensions,
    Type::SwitchExpression,
    Type::SwitchStatement,
    Type::SwitchBlock,
    Type::SwitchBlockStatementGroup,
    Type::SwitchRule,
    Type::SwitchLabel,
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
    Type::TryWithResourcesExtendedStatement,
    Type::TryWithResourcesStatement,
    Type::ResourceSpecification,
    Type::Resource,
    Type::IfStatement,
    Type::WhileStatement,
    Type::ForStatement,
    Type::EnhancedForStatement,
    Type::EnhancedForVariable,
    Type::TS2,
    Type::MarkerAnnotation,
    Type::Annotation,
    Type::AnnotationArgumentList,
    Type::ElementValuePair,
    Type::TS3,
    Type::ElementValueArrayInitializer,
    Type::TypeDeclaration,
    Type::ModuleDeclaration,
    Type::ModuleBody,
    Type::ModuleDirective,
    Type::RequiresModifier,
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
    Type::ClassBody,
    Type::StaticInitializer,
    Type::ConstructorDeclaration,
    Type::TS4,
    Type::ConstructorBody,
    Type::ExplicitConstructorInvocation,
    Type::ScopedIdentifier,
    Type::ScopedAbsoluteIdentifier,
    Type::FieldDeclaration,
    Type::RecordDeclaration,
    Type::AnnotationTypeDeclaration,
    Type::AnnotationTypeBody,
    Type::AnnotationTypeElementDeclaration,
    Type::TS5,
    Type::InterfaceDeclaration,
    Type::ExtendsInterfaces,
    Type::InterfaceBody,
    Type::ConstantDeclaration,
    Type::TS6,
    Type::VariableDeclarator,
    Type::TS7,
    Type::ArrayInitializer,
    Type::Type,
    Type::UnannotatedType,
    Type::AnnotatedType,
    Type::ScopedTypeIdentifier,
    Type::GenericType,
    Type::ArrayType,
    Type::IntegralType,
    Type::FloatingPointType,
    Type::TS8,
    Type::TS9,
    Type::FormalParameters,
    Type::FormalParameter,
    Type::ReceiverParameter,
    Type::SpreadParameter,
    Type::LocalVariableDeclaration,
    Type::MethodDeclaration,
    Type::ProgramRepeat1,
    Type::ProgramRepeat2,
    Type::ProgramRepeat3,
    Type::CastExpressionRepeat1,
    Type::InferredParametersRepeat1,
    Type::ArrayCreationExpressionRepeat1,
    Type::DimensionsExprRepeat1,
    Type::ArgumentListRepeat1,
    Type::TypeArgumentsRepeat1,
    Type::DimensionsRepeat1,
    Type::SwitchBlockRepeat1,
    Type::SwitchBlockRepeat2,
    Type::SwitchBlockStatementGroupRepeat1,
    Type::TryStatementRepeat1,
    Type::CatchTypeRepeat1,
    Type::ResourceSpecificationRepeat1,
    Type::ForStatementRepeat1,
    Type::ForStatementRepeat2,
    Type::AnnotationArgumentListRepeat1,
    Type::ElementValueArrayInitializerRepeat1,
    Type::ModuleBodyRepeat1,
    Type::ModuleDirectiveRepeat1,
    Type::ModuleDirectiveRepeat2,
    Type::EnumBodyRepeat1,
    Type::EnumBodyDeclarationsRepeat1,
    Type::ModifiersRepeat1,
    Type::TypeParametersRepeat1,
    Type::TypeBoundRepeat1,
    Type::TS10,
    Type::AnnotationTypeBodyRepeat1,
    Type::InterfaceBodyRepeat1,
    Type::TS11,
    Type::ArrayInitializerRepeat1,
    Type::FormalParametersRepeat1,
    Type::TypeIdentifier,
    Type::Spaces,
    Type::Directory,
    Type::ERROR,
];
