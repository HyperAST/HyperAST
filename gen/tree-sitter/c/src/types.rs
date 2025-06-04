use std::fmt::Display;

use hyperast::{
    tree_gen::utils_ts::TsEnableTS,
    types::{
        AAAA, AnyType, HyperType, LangRef, NodeId, TypeStore, TypeTrait, TypeU16, TypedNodeId,
    },
};

#[cfg(feature = "impl")]
mod legion_impls {

    use super::*;
    use hyperast::tree_gen::utils_ts::TsEnableTS;
    use hyperast::tree_gen::utils_ts::TsType;

    use hyperast::types::{LangWrapper, RoleStore};

    impl TsEnableTS for TStore {
        fn obtain_type<'a, N: hyperast::tree_gen::parser::NodeWithU16TypeId>(
            n: &N,
        ) -> <Self as hyperast::types::ETypeStore>::Ty2 {
            let k = n.kind_id();
            Type::from_u16(k)
        }

        fn try_obtain_type<N: hyperast::tree_gen::parser::NodeWithU16TypeId>(
            n: &N,
        ) -> Option<Self::Ty2> {
            let k = n.kind_id();
            static LEN: u16 = S_T_L.len() as u16;
            if LEN <= k && k < TStore::LOWEST_RESERVED {
                return None;
            }
            Some(Type::from_u16(k))
        }
    }

    impl TsType for Type {
        fn spaces() -> Self {
            Self::Spaces
        }

        fn is_repeat(&self) -> bool {
            self.is_repeat()
        }
    }

    impl TypeStore for TStore {
        type Ty = TypeU16<C>;
    }
    impl<'a> CEnabledTypeStore for TStore {
        fn resolve(t: Self::Ty) -> Type {
            t.e()
        }
    }

    impl<'a> hyperast::types::ETypeStore for TStore {
        type Ty2 = Type;

        fn intern(ty: Self::Ty2) -> Self::Ty {
            TType::new(ty)
        }
    }

    impl RoleStore for TStore {
        type IdF = u16;

        type Role = hyperast::types::Role;

        fn resolve_field(_lang: LangWrapper<Self::Ty>, field_id: Self::IdF) -> Self::Role {
            let s = crate::language()
                .field_name_for_id(field_id)
                .ok_or_else(|| format!("{}", field_id))
                .unwrap();
            hyperast::types::Role::try_from(s).expect(s)
        }

        fn intern_role(_lang: LangWrapper<Self::Ty>, role: Self::Role) -> Self::IdF {
            let field_name = role.to_string();
            crate::language()
                .field_id_for_name(field_name)
                .unwrap()
                .into()
        }
    }
}

#[cfg(feature = "impl")]
fn id_for_node_kind(kind: &str, named: bool) -> u16 {
    crate::language().id_for_node_kind(kind, named)
}
#[cfg(not(feature = "impl"))]
fn id_for_node_kind(kind: &str, named: bool) -> u16 {
    unimplemented!("need treesitter grammar")
}

#[cfg(feature = "impl")]
pub trait CEnabledTypeStore:
    hyperast::types::ETypeStore<Ty2 = Type> + Clone + hyperast::tree_gen::utils_ts::TsEnableTS
{
    // fn intern(t: Type) -> Self::Ty;
    fn resolve(t: Self::Ty) -> Type;
}

#[cfg(not(feature = "impl"))]
pub trait CEnabledTypeStore: TypeStore {
    // fn intern(t: Type) -> Self::Ty;
    fn resolve(t: Self::Ty) -> Type;
}

impl Type {
    pub fn resolve(t: u16) -> Self {
        assert!(t < COUNT);
        unsafe { std::mem::transmute(t) }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TIdN<IdN>(IdN);

impl<IdN: Clone + Eq + hyperast::types::AAAA> NodeId for TIdN<IdN> {
    type IdN = IdN;

    fn as_id(&self) -> &Self::IdN {
        &self.0
    }

    unsafe fn from_id(id: Self::IdN) -> Self {
        Self(id)
    }

    unsafe fn from_ref_id(_id: &Self::IdN) -> &Self {
        todo!()
    }
}

impl<IdN: Clone + Eq + AAAA> TypedNodeId for TIdN<IdN> {
    type Ty = Type;
    type TyErazed = TType;
    fn unerase(ty: Self::TyErazed) -> Self::Ty {
        ty.e()
    }
}

#[derive(Clone, Copy)]
pub struct TStore;

impl Default for TStore {
    fn default() -> Self {
        Self
    }
}

type TypeInternalSize = u16;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct T(TypeInternalSize);

#[derive(Debug)]
pub struct Lang;
pub type C = Lang;

impl C {
    pub const INST: C = Lang;
}

pub fn as_any(t: &Type) -> AnyType {
    let t = <C as hyperast::types::Lang<Type>>::to_u16(*t);
    let t = <C as hyperast::types::Lang<Type>>::make(t);
    let t: &'static dyn HyperType = t;
    t.into()
}

impl LangRef<AnyType> for C {
    fn make(&self, _t: u16) -> &'static AnyType {
        panic!()
        // &From::<&'static dyn HyperType>::from(&S_T_L[t as usize])
    }
    fn to_u16(&self, t: AnyType) -> u16 {
        // t as u16
        let t = t.as_any().downcast_ref::<Type>().unwrap();
        *t as u16
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<C>()
    }

    fn ts_symbol(&self, t: AnyType) -> u16 {
        // TODO check lang
        id_for_node_kind(t.as_static_str(), t.is_named())
    }
}

impl LangRef<Type> for C {
    fn make(&self, t: u16) -> &'static Type {
        if t == TStore::ERROR {
            &Type::ERROR
        } else if t == TStore::_ERROR {
            &Type::_ERROR
        } else if t == TStore::SPACES {
            &Type::Spaces
        } else if t == TStore::DIRECTORY {
            &Type::Directory
        } else {
            &S_T_L[t as usize]
        }
    }
    fn to_u16(&self, t: Type) -> u16 {
        t as u16
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<C>()
    }

    fn ts_symbol(&self, t: Type) -> u16 {
        id_for_node_kind(t.as_static_str(), t.is_named())
    }
}

impl LangRef<TType> for Lang {
    fn make(&self, t: u16) -> &'static TType {
        // TODO could make one safe, but not priority
        unsafe { std::mem::transmute(&S_T_L[t as usize]) }
    }
    fn to_u16(&self, t: TType) -> u16 {
        t.e() as u16
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<Lang>()
    }

    fn ts_symbol(&self, t: TType) -> u16 {
        id_for_node_kind(t.as_static_str(), t.is_named())
    }
}

impl hyperast::types::Lang<Type> for C {
    fn make(t: u16) -> &'static Type {
        Lang.make(t)
    }
    fn to_u16(t: Type) -> u16 {
        Lang.to_u16(t)
    }
}

pub use hyperast::types::Role;

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
        self == &Type::TranslationUnit
    }

    fn is_spaces(&self) -> bool {
        self == &Type::Spaces
        // setting TS0 as space is causing an issue with global_pos_with_spaces
        // and TS0 is end list of tokens, so maybe other issues.
        // Actual fix is to skip TS0 in skipable_pre in the generator,
        // thus TSO should not appear anymore in generated ast.
        // || self == &Type::TS0
    }

    fn is_syntax(&self) -> bool {
        self == &Type::LParen
        || self == &Type::RParen
        || self == &Type::HashInclude // "#include",
        ||self == &Type::NewLine // "\n",
        ||self == &Type::HashDefine // "#define",
        ||self == &Type::LParen // "(",
        ||self == &Type::DotDotDot // "...",
        ||self == &Type::Comma // ",",
        ||self == &Type::RParen // ")",
        // ||self == &Type::HashIf // "#if",
        // ||self == &Type::HashEndif // "#endif",
        // ||self == &Type::HashIfdef // "#ifdef",
        // ||self == &Type::HashIfndef // "#ifndef",
        // ||self == &Type::HashElse // "#else",
        // ||self == &Type::HashElif // "#elif",
        ||self == &Type::Bang // "!",
        ||self == &Type::Tilde // "~",
        ||self == &Type::Dash // "-",
        ||self == &Type::Plus // "+",
        ||self == &Type::Star // "*",
        ||self == &Type::Slash // "/",
        ||self == &Type::Percent // "%",
        ||self == &Type::PipePipe // "||",
        ||self == &Type::AmpAmp // "&&",
        ||self == &Type::Pipe // "|",
        ||self == &Type::Caret // "^",
        ||self == &Type::Amp // "&",
        ||self == &Type::EqEq // "==",
        ||self == &Type::BangEq // "!=",
        ||self == &Type::GT // ">",
        ||self == &Type::GTEq // ">=",
        ||self == &Type::LTEq // "<=",
        ||self == &Type::LT // "<",
        ||self == &Type::LtLt // "<<",
        ||self == &Type::GtGt // ">>",
        ||self == &Type::SemiColon // ";",
        // ||self == &Type::Typedef // "typedef",
        ||self == &Type::Extern // "extern",
        ||self == &Type::TS1 // "__attribute__",
        ||self == &Type::ColonColon // "::",
        ||self == &Type::TS2 // "[[",
        ||self == &Type::TS3 // "]]",
        // ||self == &Type::TS4 // "__declspec",
        // ||self == &Type::TS5 // "__based",
        // ||self == &Type::TS6 // "__cdecl",
        // ||self == &Type::TS7 // "__clrcall",
        // ||self == &Type::TS8 // "__stdcall",
        // ||self == &Type::TS9 // "__fastcall",
        // ||self == &Type::TS10 // "__thiscall",
        // ||self == &Type::TS11 // "__vectorcall",
        // ||self == &Type::MsRestrictModifier // "ms_restrict_modifier",
        // ||self == &Type::MsUnsignedPtrModifier // "ms_unsigned_ptr_modifier",
        // ||self == &Type::MsSignedPtrModifier // "ms_signed_ptr_modifier",
        // ||self == &Type::TS12 // "_unaligned",
        // ||self == &Type::TS13 // "__unaligned",
        ||self == &Type::LBrace // "{",
        ||self == &Type::RBrace // "}",
        ||self == &Type::LBracket // "[",
        ||self == &Type::RBracket // "]",
        ||self == &Type::Eq // "=",
        // ||self == &Type::Static // "static",
        // ||self == &Type::Register // "register",
        // ||self == &Type::Inline // "inline",
        // ||self == &Type::ThreadLocal // "thread_local",
        // ||self == &Type::Const // "const",
        // ||self == &Type::Volatile // "volatile",
        // ||self == &Type::Restrict // "restrict",
        // ||self == &Type::TS14 // "_Atomic",
        // ||self == &Type::Mutable // "mutable",
        // ||self == &Type::Constexpr // "constexpr",
        // ||self == &Type::Constinit // "constinit",
        // ||self == &Type::Consteval // "consteval",
        // ||self == &Type::Signed // "signed",
        // ||self == &Type::Unsigned // "unsigned",
        // ||self == &Type::Long // "long",
        // ||self == &Type::Short // "short",
        ||self == &Type::Enum // "enum",
        ||self == &Type::Struct // "struct",
        ||self == &Type::Union // "union",
        ||self == &Type::Colon // ":",
        ||self == &Type::If // "if",
        ||self == &Type::Else // "else",
        ||self == &Type::Switch // "switch",
        ||self == &Type::Case // "case",
        ||self == &Type::Default // "default",
        ||self == &Type::While // "while",
        ||self == &Type::Do // "do",
        ||self == &Type::For // "for",
        ||self == &Type::Return // "return",
        // ||self == &Type::Break // "break",
        // ||self == &Type::Continue // "continue",
        // ||self == &Type::Goto // "goto",
        ||self == &Type::QMark // "?",
        // ||self == &Type::StarEq // "*=",
        // ||self == &Type::SlashEq // "/=",
        // ||self == &Type::PercentEq // "%=",
        // ||self == &Type::PlusEq // "+=",
        // ||self == &Type::DashEq // "-=",
        // ||self == &Type::LtLtEq // "<<=",
        // ||self == &Type::GtGtEq // ">>=",
        // ||self == &Type::AmpEq // "&=",
        // ||self == &Type::CaretEq // "^=",
        // ||self == &Type::PipeEq // "|=",
        // ||self == &Type::AndEq // "and_eq",
        // ||self == &Type::OrEq // "or_eq",
        // ||self == &Type::XorEq // "xor_eq",
        // ||self == &Type::Not // "not",
        // ||self == &Type::Compl // "compl",
        // ||self == &Type::TS15 // "<=>",
        // ||self == &Type::Or // "or",
        // ||self == &Type::And // "and",
        // ||self == &Type::Bitor // "bitor",
        // ||self == &Type::Xor // "xor",
        // ||self == &Type::Bitand // "bitand",
        // ||self == &Type::NotEq // "not_eq",
        // ||self == &Type::DashDash // "--",
        // ||self == &Type::PlusPlus // "++",
        // ||self == &Type::Sizeof // "sizeof",
        // ||self == &Type::Asm // "asm",
        ||self == &Type::Dot // ".",
        ||self == &Type::DashGt // "->",
    }

    fn as_shared(&self) -> hyperast::types::Shared {
        use hyperast::types::Shared;
        match self {
            Type::EnumSpecifier => Shared::TypeDeclaration,
            // Type::_TypeSpecifier => Shared::TypeDeclaration, // abstract
            Type::PrimitiveType => Shared::TypeDeclaration,
            Type::SizedTypeSpecifier => Shared::TypeDeclaration,
            Type::StructSpecifier => Shared::TypeDeclaration,
            Type::TypeIdentifier => Shared::TypeDeclaration,
            Type::UnionSpecifier => Shared::TypeDeclaration,
            Type::Comment => Shared::Comment,
            Type::Identifier => Shared::Identifier,
            _ => Shared::Other,
        }
    }

    fn as_abstract(&self) -> hyperast::types::Abstracts {
        use hyperast::types::Abstract;
        Abstract::Expression.when(self.is_expression())
            | Abstract::Statement.when(self.is_statement())
            | Abstract::Executable.when(self.is_executable_member())
            | Abstract::Declaration.when(self.is_type_declaration())
            | Abstract::Literal.when(self.is_literal())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_static(&self) -> &'static dyn HyperType {
        let t = <C as hyperast::types::Lang<Type>>::to_u16(*self);
        let t = <C as hyperast::types::Lang<Type>>::make(t);
        t
    }

    fn as_static_str(&self) -> &'static str {
        self.to_str()
    }

    fn is_hidden(&self) -> bool {
        self.is_hidden()
    }

    fn is_supertype(&self) -> bool {
        self.is_supertype()
    }

    fn is_named(&self) -> bool {
        self.is_named()
    }
    fn get_lang(&self) -> hyperast::types::LangWrapper<Self>
    where
        Self: Sized,
    {
        hyperast::types::LangWrapper::from(&Lang as &(dyn LangRef<Self> + 'static))
    }

    fn lang_ref(&self) -> hyperast::types::LangWrapper<AnyType> {
        hyperast::types::LangWrapper::from(&Lang as &(dyn LangRef<AnyType> + 'static))
    }
}
impl TypeTrait for Type {
    type Lang = C;

    fn is_fork(&self) -> bool {
        todo!()
    }

    fn is_literal(&self) -> bool {
        todo!()
    }

    fn is_primitive(&self) -> bool {
        todo!()
    }

    fn is_type_declaration(&self) -> bool {
        todo!()
    }

    fn is_identifier(&self) -> bool {
        todo!()
    }

    fn is_instance_ref(&self) -> bool {
        todo!()
    }

    fn is_type_body(&self) -> bool {
        todo!()
    }

    fn is_value_member(&self) -> bool {
        todo!()
    }

    fn is_executable_member(&self) -> bool {
        todo!()
    }

    fn is_statement(&self) -> bool {
        todo!()
    }

    fn is_declarative_statement(&self) -> bool {
        todo!()
    }

    fn is_structural_statement(&self) -> bool {
        todo!()
    }

    fn is_block_related(&self) -> bool {
        todo!()
    }

    fn is_simple_statement(&self) -> bool {
        todo!()
    }

    fn is_local_declare(&self) -> bool {
        todo!()
    }

    fn is_parameter(&self) -> bool {
        todo!()
    }

    fn is_parameter_list(&self) -> bool {
        todo!()
    }

    fn is_argument_list(&self) -> bool {
        todo!()
    }

    fn is_expression(&self) -> bool {
        todo!()
    }

    fn is_comment(&self) -> bool {
        todo!()
    }
}

const COUNT: u16 = 542;
impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

impl TryFrom<&str> for Type {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Type::from_str(value).ok_or(())
    }
}

impl Type {
    pub(crate) fn is_repeat(&self) -> bool {
        *self == Type::TranslationUnitRepeat1
            || *self == Type::PreprocParamsRepeat1
            || *self == Type::PreprocIfRepeat1
            || *self == Type::PreprocIfInFieldDeclarationListRepeat1
            || *self == Type::PreprocIfInEnumeratorListRepeat1
            || *self == Type::PreprocIfInEnumeratorListNoCommaRepeat1
            || *self == Type::PreprocArgumentListRepeat1
            || *self == Type::DeclarationRepeat1
            || *self == Type::TypeDefinitionRepeat1
            || *self == Type::_TypeDefinitionTypeRepeat1
            || *self == Type::_TypeDefinitionDeclaratorsRepeat1
            || *self == Type::_DeclarationSpecifiersRepeat1
            || *self == Type::AttributeDeclarationRepeat1
            || *self == Type::AttributedDeclaratorRepeat1
            || *self == Type::PointerDeclaratorRepeat1
            || *self == Type::ArrayDeclaratorRepeat1
            || *self == Type::SizedTypeSpecifierRepeat1
            || *self == Type::EnumeratorListRepeat1
            || *self == Type::ParameterListRepeat1
            || *self == Type::CaseStatementRepeat1
            || *self == Type::GenericExpressionRepeat1
            || *self == Type::GnuAsmExpressionRepeat1
            || *self == Type::GnuAsmOutputOperandListRepeat1
            || *self == Type::GnuAsmInputOperandListRepeat1
            || *self == Type::GnuAsmClobberListRepeat1
            || *self == Type::GnuAsmGotoListRepeat1
            || *self == Type::ArgumentListRepeat1
            || *self == Type::InitializerListRepeat1
            || *self == Type::InitializerPairRepeat1
            || *self == Type::CharLiteralRepeat1
            || *self == Type::ConcatenatedStringRepeat1
            || *self == Type::StringLiteralRepeat1
    }
}

impl hyperast::types::LLang<TType> for C {
    type I = u16;

    type E = Type;

    const TE: &[Self::E] = S_T_L;

    fn as_lang_wrapper() -> hyperast::types::LangWrapper<TType> {
        From::<&'static (dyn LangRef<_>)>::from(&Lang)
    }
}

pub type TType = TypeU16<Lang>;

impl From<u16> for Type {
    fn from(value: u16) -> Self {
        debug_assert_eq!(Self::from_u16(value), S_T_L[value as usize]);
        S_T_L[value as usize]
    }
}
impl Into<TypeU16<C>> for Type {
    fn into(self) -> TypeU16<C> {
        TypeU16::new(self)
    }
}

impl Into<u16> for Type {
    fn into(self) -> u16 {
        self as u16
    }
}

#[repr(u16)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Type {
    End,
    Identifier,
    HashInclude,
    PreprocIncludeToken2,
    HashDefine,
    LParen,
    DotDotDot,
    Comma,
    RParen,
    HashIf,
    NewLine,
    HashEndif,
    HashIfdef,
    HashIfndef,
    HashElse,
    HashElif,
    HashElifdef,
    HashElifndef,
    PreprocArg,
    PreprocDirective,
    Defined,
    Bang,
    Tilde,
    Dash,
    Plus,
    Star,
    Slash,
    Percent,
    PipePipe,
    AmpAmp,
    Pipe,
    Caret,
    Amp,
    EqEq,
    BangEq,
    GT,
    GTEq,
    LTEq,
    LT,
    LtLt,
    GtGt,
    SemiColon,
    TS0,
    Typedef,
    Extern,
    TS1,
    __Attribute,
    ColonColon,
    TS2,
    TS3,
    __Declspec,
    __Based,
    __Cdecl,
    __Clrcall,
    __Stdcall,
    __Fastcall,
    __Thiscall,
    __Vectorcall,
    MsRestrictModifier,
    MsUnsignedPtrModifier,
    MsSignedPtrModifier,
    _Unaligned,
    __Unaligned,
    LBrace,
    RBrace,
    Signed,
    Unsigned,
    Long,
    Short,
    LBracket,
    Static,
    RBracket,
    Eq,
    Auto,
    Register,
    Inline,
    __Inline,
    TS4,
    __Forceinline,
    ThreadLocal,
    __Thread,
    Const,
    Constexpr,
    Volatile,
    Restrict,
    TS5,
    TS6,
    TS7,
    Noreturn,
    TS8,
    Alignas,
    TS9,
    PrimitiveType,
    Enum,
    Colon,
    Struct,
    Union,
    If,
    Else,
    Switch,
    Case,
    Default,
    While,
    Do,
    For,
    Return,
    Break,
    Continue,
    Goto,
    __Try,
    __Except,
    __Finally,
    __Leave,
    QMark,
    StarEq,
    SlashEq,
    PercentEq,
    PlusEq,
    DashEq,
    LtLtEq,
    GtGtEq,
    AmpEq,
    CaretEq,
    PipeEq,
    DashDash,
    PlusPlus,
    Sizeof,
    TS10,
    __Alignof,
    _Alignof,
    Alignof,
    TS11,
    Offsetof,
    TS12,
    Asm,
    TS13,
    __Asm,
    TS14,
    Dot,
    DashGt,
    NumberLiteral,
    TS15,
    TS16,
    TS17,
    TS18,
    SQuote,
    Character,
    TS19,
    TS20,
    TS21,
    TS22,
    DQuote,
    StringContent,
    EscapeSequence,
    SystemLibString,
    True,
    False,
    TS23,
    Nullptr,
    Comment,
    TranslationUnit,
    _TopLevelItem,
    _BlockItem,
    PreprocInclude,
    PreprocDef,
    PreprocFunctionDef,
    PreprocParams,
    PreprocCall,
    PreprocIf,
    PreprocIfdef,
    PreprocElse,
    PreprocElif,
    PreprocElifdef,
    _PreprocExpression,
    ParenthesizedExpression,
    PreprocDefined,
    UnaryExpression,
    CallExpression,
    ArgumentList,
    BinaryExpression,
    FunctionDefinition,
    Declaration,
    TypeDefinition,
    _TypeDefinitionType,
    _TypeDefinitionDeclarators,
    _DeclarationModifiers,
    _DeclarationSpecifiers,
    LinkageSpecification,
    AttributeSpecifier,
    Attribute,
    AttributeDeclaration,
    MsDeclspecModifier,
    MsBasedModifier,
    MsCallModifier,
    MsUnalignedPtrModifier,
    MsPointerModifier,
    DeclarationList,
    _Declarator,
    _DeclarationDeclarator,
    _FieldDeclarator,
    _TypeDeclarator,
    _AbstractDeclarator,
    ParenthesizedDeclarator,
    AbstractParenthesizedDeclarator,
    AttributedDeclarator,
    PointerDeclarator,
    AbstractPointerDeclarator,
    FunctionDeclarator,
    AbstractFunctionDeclarator,
    ArrayDeclarator,
    AbstractArrayDeclarator,
    InitDeclarator,
    CompoundStatement,
    StorageClassSpecifier,
    TypeQualifier,
    AlignasQualifier,
    TypeSpecifier,
    SizedTypeSpecifier,
    EnumSpecifier,
    EnumeratorList,
    StructSpecifier,
    UnionSpecifier,
    FieldDeclarationList,
    _FieldDeclarationListItem,
    FieldDeclaration,
    _FieldDeclarationDeclarator,
    BitfieldClause,
    Enumerator,
    VariadicParameter,
    ParameterList,
    ParameterDeclaration,
    AttributedStatement,
    Statement,
    _TopLevelStatement,
    LabeledStatement,
    ExpressionStatement,
    IfStatement,
    ElseClause,
    SwitchStatement,
    CaseStatement,
    WhileStatement,
    DoStatement,
    ForStatement,
    _ForStatementBody,
    ReturnStatement,
    BreakStatement,
    ContinueStatement,
    GotoStatement,
    SehTryStatement,
    SehExceptClause,
    SehFinallyClause,
    SehLeaveStatement,
    Expression,
    _String,
    CommaExpression,
    ConditionalExpression,
    AssignmentExpression,
    PointerExpression,
    UpdateExpression,
    CastExpression,
    TypeDescriptor,
    SizeofExpression,
    AlignofExpression,
    OffsetofExpression,
    GenericExpression,
    SubscriptExpression,
    GnuAsmExpression,
    GnuAsmQualifier,
    GnuAsmOutputOperandList,
    GnuAsmOutputOperand,
    GnuAsmInputOperandList,
    GnuAsmInputOperand,
    GnuAsmClobberList,
    GnuAsmGotoList,
    ExtensionExpression,
    FieldExpression,
    CompoundLiteralExpression,
    InitializerList,
    InitializerPair,
    SubscriptDesignator,
    SubscriptRangeDesignator,
    FieldDesignator,
    CharLiteral,
    ConcatenatedString,
    StringLiteral,
    Null,
    _EmptyDeclaration,
    MacroTypeSpecifier,
    TranslationUnitRepeat1,
    PreprocParamsRepeat1,
    PreprocIfRepeat1,
    PreprocIfInFieldDeclarationListRepeat1,
    PreprocIfInEnumeratorListRepeat1,
    PreprocIfInEnumeratorListNoCommaRepeat1,
    PreprocArgumentListRepeat1,
    _OldStyleFunctionDefinitionRepeat1,
    DeclarationRepeat1,
    TypeDefinitionRepeat1,
    _TypeDefinitionTypeRepeat1,
    _TypeDefinitionDeclaratorsRepeat1,
    _DeclarationSpecifiersRepeat1,
    AttributeDeclarationRepeat1,
    AttributedDeclaratorRepeat1,
    PointerDeclaratorRepeat1,
    FunctionDeclaratorRepeat1,
    ArrayDeclaratorRepeat1,
    SizedTypeSpecifierRepeat1,
    EnumeratorListRepeat1,
    _FieldDeclarationDeclaratorRepeat1,
    ParameterListRepeat1,
    _OldStyleParameterListRepeat1,
    CaseStatementRepeat1,
    GenericExpressionRepeat1,
    GnuAsmExpressionRepeat1,
    GnuAsmOutputOperandListRepeat1,
    GnuAsmInputOperandListRepeat1,
    GnuAsmClobberListRepeat1,
    GnuAsmGotoListRepeat1,
    ArgumentListRepeat1,
    InitializerListRepeat1,
    InitializerPairRepeat1,
    CharLiteralRepeat1,
    ConcatenatedStringRepeat1,
    StringLiteralRepeat1,
    FieldIdentifier,
    StatementIdentifier,
    TypeIdentifier,
    Directory = TStore::DIRECTORY,
    Spaces = TStore::SPACES,
    _ERROR = TStore::_ERROR,
    ERROR = TStore::ERROR,
}

impl Type {
    pub fn from_u16(t: u16) -> Type {
        match t {
            0u16 => Type::End,
            1u16 => Type::Identifier,
            2u16 => Type::HashInclude,
            3u16 => Type::PreprocIncludeToken2,
            4u16 => Type::HashDefine,
            5u16 => Type::LParen,
            6u16 => Type::DotDotDot,
            7u16 => Type::Comma,
            8u16 => Type::RParen,
            9u16 => Type::HashIf,
            10u16 => Type::NewLine,
            11u16 => Type::HashEndif,
            12u16 => Type::HashIfdef,
            13u16 => Type::HashIfndef,
            14u16 => Type::HashElse,
            15u16 => Type::HashElif,
            16u16 => Type::HashElifdef,
            17u16 => Type::HashElifndef,
            18u16 => Type::PreprocArg,
            19u16 => Type::PreprocDirective,
            20u16 => Type::LParen,
            21u16 => Type::Defined,
            22u16 => Type::Bang,
            23u16 => Type::Tilde,
            24u16 => Type::Dash,
            25u16 => Type::Plus,
            26u16 => Type::Star,
            27u16 => Type::Slash,
            28u16 => Type::Percent,
            29u16 => Type::PipePipe,
            30u16 => Type::AmpAmp,
            31u16 => Type::Pipe,
            32u16 => Type::Caret,
            33u16 => Type::Amp,
            34u16 => Type::EqEq,
            35u16 => Type::BangEq,
            36u16 => Type::GT,
            37u16 => Type::GTEq,
            38u16 => Type::LTEq,
            39u16 => Type::LT,
            40u16 => Type::LtLt,
            41u16 => Type::GtGt,
            42u16 => Type::SemiColon,
            43u16 => Type::TS0,
            44u16 => Type::Typedef,
            45u16 => Type::Extern,
            46u16 => Type::TS1,
            47u16 => Type::__Attribute,
            48u16 => Type::ColonColon,
            49u16 => Type::TS2,
            50u16 => Type::TS3,
            51u16 => Type::__Declspec,
            52u16 => Type::__Based,
            53u16 => Type::__Cdecl,
            54u16 => Type::__Clrcall,
            55u16 => Type::__Stdcall,
            56u16 => Type::__Fastcall,
            57u16 => Type::__Thiscall,
            58u16 => Type::__Vectorcall,
            59u16 => Type::MsRestrictModifier,
            60u16 => Type::MsUnsignedPtrModifier,
            61u16 => Type::MsSignedPtrModifier,
            62u16 => Type::_Unaligned,
            63u16 => Type::__Unaligned,
            64u16 => Type::LBrace,
            65u16 => Type::RBrace,
            66u16 => Type::Signed,
            67u16 => Type::Unsigned,
            68u16 => Type::Long,
            69u16 => Type::Short,
            70u16 => Type::LBracket,
            71u16 => Type::Static,
            72u16 => Type::RBracket,
            73u16 => Type::Eq,
            74u16 => Type::Auto,
            75u16 => Type::Register,
            76u16 => Type::Inline,
            77u16 => Type::__Inline,
            78u16 => Type::TS4,
            79u16 => Type::__Forceinline,
            80u16 => Type::ThreadLocal,
            81u16 => Type::__Thread,
            82u16 => Type::Const,
            83u16 => Type::Constexpr,
            84u16 => Type::Volatile,
            85u16 => Type::Restrict,
            86u16 => Type::TS5,
            87u16 => Type::TS6,
            88u16 => Type::TS7,
            89u16 => Type::Noreturn,
            90u16 => Type::TS8,
            91u16 => Type::Alignas,
            92u16 => Type::TS9,
            93u16 => Type::PrimitiveType,
            94u16 => Type::Enum,
            95u16 => Type::Colon,
            96u16 => Type::Struct,
            97u16 => Type::Union,
            98u16 => Type::If,
            99u16 => Type::Else,
            100u16 => Type::Switch,
            101u16 => Type::Case,
            102u16 => Type::Default,
            103u16 => Type::While,
            104u16 => Type::Do,
            105u16 => Type::For,
            106u16 => Type::Return,
            107u16 => Type::Break,
            108u16 => Type::Continue,
            109u16 => Type::Goto,
            110u16 => Type::__Try,
            111u16 => Type::__Except,
            112u16 => Type::__Finally,
            113u16 => Type::__Leave,
            114u16 => Type::QMark,
            115u16 => Type::StarEq,
            116u16 => Type::SlashEq,
            117u16 => Type::PercentEq,
            118u16 => Type::PlusEq,
            119u16 => Type::DashEq,
            120u16 => Type::LtLtEq,
            121u16 => Type::GtGtEq,
            122u16 => Type::AmpEq,
            123u16 => Type::CaretEq,
            124u16 => Type::PipeEq,
            125u16 => Type::DashDash,
            126u16 => Type::PlusPlus,
            127u16 => Type::Sizeof,
            128u16 => Type::TS10,
            129u16 => Type::__Alignof,
            130u16 => Type::_Alignof,
            131u16 => Type::Alignof,
            132u16 => Type::TS11,
            133u16 => Type::Offsetof,
            134u16 => Type::TS12,
            135u16 => Type::Asm,
            136u16 => Type::TS13,
            137u16 => Type::__Asm,
            138u16 => Type::TS14,
            139u16 => Type::Dot,
            140u16 => Type::DashGt,
            141u16 => Type::NumberLiteral,
            142u16 => Type::TS15,
            143u16 => Type::TS16,
            144u16 => Type::TS17,
            145u16 => Type::TS18,
            146u16 => Type::SQuote,
            147u16 => Type::Character,
            148u16 => Type::TS19,
            149u16 => Type::TS20,
            150u16 => Type::TS21,
            151u16 => Type::TS22,
            152u16 => Type::DQuote,
            153u16 => Type::StringContent,
            154u16 => Type::EscapeSequence,
            155u16 => Type::SystemLibString,
            156u16 => Type::True,
            157u16 => Type::False,
            158u16 => Type::TS23,
            159u16 => Type::Nullptr,
            160u16 => Type::Comment,
            161u16 => Type::TranslationUnit,
            162u16 => Type::_TopLevelItem,
            163u16 => Type::_BlockItem,
            164u16 => Type::PreprocInclude,
            165u16 => Type::PreprocDef,
            166u16 => Type::PreprocFunctionDef,
            167u16 => Type::PreprocParams,
            168u16 => Type::PreprocCall,
            169u16 => Type::PreprocIf,
            170u16 => Type::PreprocIfdef,
            171u16 => Type::PreprocElse,
            172u16 => Type::PreprocElif,
            173u16 => Type::PreprocElifdef,
            174u16 => Type::PreprocIf,
            175u16 => Type::PreprocIfdef,
            176u16 => Type::PreprocElse,
            177u16 => Type::PreprocElif,
            178u16 => Type::PreprocElifdef,
            179u16 => Type::PreprocIf,
            180u16 => Type::PreprocIfdef,
            181u16 => Type::PreprocElse,
            182u16 => Type::PreprocElif,
            183u16 => Type::PreprocElifdef,
            184u16 => Type::PreprocIf,
            185u16 => Type::PreprocIfdef,
            186u16 => Type::PreprocElse,
            187u16 => Type::PreprocElif,
            188u16 => Type::PreprocElifdef,
            189u16 => Type::_PreprocExpression,
            190u16 => Type::ParenthesizedExpression,
            191u16 => Type::PreprocDefined,
            192u16 => Type::UnaryExpression,
            193u16 => Type::CallExpression,
            194u16 => Type::ArgumentList,
            195u16 => Type::BinaryExpression,
            196u16 => Type::FunctionDefinition,
            197u16 => Type::FunctionDefinition,
            198u16 => Type::Declaration,
            199u16 => Type::TypeDefinition,
            200u16 => Type::_TypeDefinitionType,
            201u16 => Type::_TypeDefinitionDeclarators,
            202u16 => Type::_DeclarationModifiers,
            203u16 => Type::_DeclarationSpecifiers,
            204u16 => Type::LinkageSpecification,
            205u16 => Type::AttributeSpecifier,
            206u16 => Type::Attribute,
            207u16 => Type::AttributeDeclaration,
            208u16 => Type::MsDeclspecModifier,
            209u16 => Type::MsBasedModifier,
            210u16 => Type::MsCallModifier,
            211u16 => Type::MsUnalignedPtrModifier,
            212u16 => Type::MsPointerModifier,
            213u16 => Type::DeclarationList,
            214u16 => Type::_Declarator,
            215u16 => Type::_DeclarationDeclarator,
            216u16 => Type::_FieldDeclarator,
            217u16 => Type::_TypeDeclarator,
            218u16 => Type::_AbstractDeclarator,
            219u16 => Type::ParenthesizedDeclarator,
            220u16 => Type::ParenthesizedDeclarator,
            221u16 => Type::ParenthesizedDeclarator,
            222u16 => Type::AbstractParenthesizedDeclarator,
            223u16 => Type::AttributedDeclarator,
            224u16 => Type::AttributedDeclarator,
            225u16 => Type::AttributedDeclarator,
            226u16 => Type::PointerDeclarator,
            227u16 => Type::PointerDeclarator,
            228u16 => Type::PointerDeclarator,
            229u16 => Type::AbstractPointerDeclarator,
            230u16 => Type::FunctionDeclarator,
            231u16 => Type::FunctionDeclarator,
            232u16 => Type::FunctionDeclarator,
            233u16 => Type::FunctionDeclarator,
            234u16 => Type::AbstractFunctionDeclarator,
            235u16 => Type::FunctionDeclarator,
            236u16 => Type::ArrayDeclarator,
            237u16 => Type::ArrayDeclarator,
            238u16 => Type::ArrayDeclarator,
            239u16 => Type::AbstractArrayDeclarator,
            240u16 => Type::InitDeclarator,
            241u16 => Type::CompoundStatement,
            242u16 => Type::StorageClassSpecifier,
            243u16 => Type::TypeQualifier,
            244u16 => Type::AlignasQualifier,
            245u16 => Type::TypeSpecifier,
            246u16 => Type::SizedTypeSpecifier,
            247u16 => Type::EnumSpecifier,
            248u16 => Type::EnumeratorList,
            249u16 => Type::StructSpecifier,
            250u16 => Type::UnionSpecifier,
            251u16 => Type::FieldDeclarationList,
            252u16 => Type::_FieldDeclarationListItem,
            253u16 => Type::FieldDeclaration,
            254u16 => Type::_FieldDeclarationDeclarator,
            255u16 => Type::BitfieldClause,
            256u16 => Type::Enumerator,
            257u16 => Type::VariadicParameter,
            258u16 => Type::ParameterList,
            259u16 => Type::ParameterList,
            260u16 => Type::ParameterDeclaration,
            261u16 => Type::AttributedStatement,
            262u16 => Type::Statement,
            263u16 => Type::_TopLevelStatement,
            264u16 => Type::LabeledStatement,
            265u16 => Type::ExpressionStatement,
            266u16 => Type::ExpressionStatement,
            267u16 => Type::IfStatement,
            268u16 => Type::ElseClause,
            269u16 => Type::SwitchStatement,
            270u16 => Type::CaseStatement,
            271u16 => Type::WhileStatement,
            272u16 => Type::DoStatement,
            273u16 => Type::ForStatement,
            274u16 => Type::_ForStatementBody,
            275u16 => Type::ReturnStatement,
            276u16 => Type::BreakStatement,
            277u16 => Type::ContinueStatement,
            278u16 => Type::GotoStatement,
            279u16 => Type::SehTryStatement,
            280u16 => Type::SehExceptClause,
            281u16 => Type::SehFinallyClause,
            282u16 => Type::SehLeaveStatement,
            283u16 => Type::Expression,
            284u16 => Type::_String,
            285u16 => Type::CommaExpression,
            286u16 => Type::ConditionalExpression,
            287u16 => Type::AssignmentExpression,
            288u16 => Type::PointerExpression,
            289u16 => Type::UnaryExpression,
            290u16 => Type::BinaryExpression,
            291u16 => Type::UpdateExpression,
            292u16 => Type::CastExpression,
            293u16 => Type::TypeDescriptor,
            294u16 => Type::SizeofExpression,
            295u16 => Type::AlignofExpression,
            296u16 => Type::OffsetofExpression,
            297u16 => Type::GenericExpression,
            298u16 => Type::SubscriptExpression,
            299u16 => Type::CallExpression,
            300u16 => Type::GnuAsmExpression,
            301u16 => Type::GnuAsmQualifier,
            302u16 => Type::GnuAsmOutputOperandList,
            303u16 => Type::GnuAsmOutputOperand,
            304u16 => Type::GnuAsmInputOperandList,
            305u16 => Type::GnuAsmInputOperand,
            306u16 => Type::GnuAsmClobberList,
            307u16 => Type::GnuAsmGotoList,
            308u16 => Type::ExtensionExpression,
            309u16 => Type::ArgumentList,
            310u16 => Type::FieldExpression,
            311u16 => Type::CompoundLiteralExpression,
            312u16 => Type::ParenthesizedExpression,
            313u16 => Type::InitializerList,
            314u16 => Type::InitializerPair,
            315u16 => Type::SubscriptDesignator,
            316u16 => Type::SubscriptRangeDesignator,
            317u16 => Type::FieldDesignator,
            318u16 => Type::CharLiteral,
            319u16 => Type::ConcatenatedString,
            320u16 => Type::StringLiteral,
            321u16 => Type::Null,
            322u16 => Type::_EmptyDeclaration,
            323u16 => Type::MacroTypeSpecifier,
            324u16 => Type::TranslationUnitRepeat1,
            325u16 => Type::PreprocParamsRepeat1,
            326u16 => Type::PreprocIfRepeat1,
            327u16 => Type::PreprocIfInFieldDeclarationListRepeat1,
            328u16 => Type::PreprocIfInEnumeratorListRepeat1,
            329u16 => Type::PreprocIfInEnumeratorListNoCommaRepeat1,
            330u16 => Type::PreprocArgumentListRepeat1,
            331u16 => Type::_OldStyleFunctionDefinitionRepeat1,
            332u16 => Type::DeclarationRepeat1,
            333u16 => Type::TypeDefinitionRepeat1,
            334u16 => Type::_TypeDefinitionTypeRepeat1,
            335u16 => Type::_TypeDefinitionDeclaratorsRepeat1,
            336u16 => Type::_DeclarationSpecifiersRepeat1,
            337u16 => Type::AttributeDeclarationRepeat1,
            338u16 => Type::AttributedDeclaratorRepeat1,
            339u16 => Type::PointerDeclaratorRepeat1,
            340u16 => Type::FunctionDeclaratorRepeat1,
            341u16 => Type::ArrayDeclaratorRepeat1,
            342u16 => Type::SizedTypeSpecifierRepeat1,
            343u16 => Type::EnumeratorListRepeat1,
            344u16 => Type::_FieldDeclarationDeclaratorRepeat1,
            345u16 => Type::ParameterListRepeat1,
            346u16 => Type::_OldStyleParameterListRepeat1,
            347u16 => Type::CaseStatementRepeat1,
            348u16 => Type::GenericExpressionRepeat1,
            349u16 => Type::GnuAsmExpressionRepeat1,
            350u16 => Type::GnuAsmOutputOperandListRepeat1,
            351u16 => Type::GnuAsmInputOperandListRepeat1,
            352u16 => Type::GnuAsmClobberListRepeat1,
            353u16 => Type::GnuAsmGotoListRepeat1,
            354u16 => Type::ArgumentListRepeat1,
            355u16 => Type::InitializerListRepeat1,
            356u16 => Type::InitializerPairRepeat1,
            357u16 => Type::CharLiteralRepeat1,
            358u16 => Type::ConcatenatedStringRepeat1,
            359u16 => Type::StringLiteralRepeat1,
            360u16 => Type::FieldIdentifier,
            361u16 => Type::StatementIdentifier,
            362u16 => Type::TypeIdentifier,
            TStore::DIRECTORY => Type::Directory,
            TStore::SPACES => Type::Spaces,
            TStore::_ERROR => Type::_ERROR,
            TStore::ERROR => Type::ERROR,
            x => panic!("{}", x),
        }
    }
    #[allow(unreachable_patterns)]
    pub fn from_str(t: &str) -> Option<Type> {
        Some(match t {
            "end" => Type::End,
            "identifier" => Type::Identifier,
            "#include" => Type::HashInclude,
            "preproc_include_token2" => Type::PreprocIncludeToken2,
            "#define" => Type::HashDefine,
            "(" => Type::LParen,
            "..." => Type::DotDotDot,
            "," => Type::Comma,
            ")" => Type::RParen,
            "#if" => Type::HashIf,
            "\n" => Type::NewLine,
            "#endif" => Type::HashEndif,
            "#ifdef" => Type::HashIfdef,
            "#ifndef" => Type::HashIfndef,
            "#else" => Type::HashElse,
            "#elif" => Type::HashElif,
            "#elifdef" => Type::HashElifdef,
            "#elifndef" => Type::HashElifndef,
            "preproc_arg" => Type::PreprocArg,
            "preproc_directive" => Type::PreprocDirective,
            "defined" => Type::Defined,
            "!" => Type::Bang,
            "~" => Type::Tilde,
            "-" => Type::Dash,
            "+" => Type::Plus,
            "*" => Type::Star,
            "/" => Type::Slash,
            "%" => Type::Percent,
            "||" => Type::PipePipe,
            "&&" => Type::AmpAmp,
            "|" => Type::Pipe,
            "^" => Type::Caret,
            "&" => Type::Amp,
            "==" => Type::EqEq,
            "!=" => Type::BangEq,
            ">" => Type::GT,
            ">=" => Type::GTEq,
            "<=" => Type::LTEq,
            "<" => Type::LT,
            "<<" => Type::LtLt,
            ">>" => Type::GtGt,
            ";" => Type::SemiColon,
            "__extension__" => Type::TS0,
            "typedef" => Type::Typedef,
            "extern" => Type::Extern,
            "__attribute__" => Type::TS1,
            "__attribute" => Type::__Attribute,
            "::" => Type::ColonColon,
            "[[" => Type::TS2,
            "]]" => Type::TS3,
            "__declspec" => Type::__Declspec,
            "__based" => Type::__Based,
            "__cdecl" => Type::__Cdecl,
            "__clrcall" => Type::__Clrcall,
            "__stdcall" => Type::__Stdcall,
            "__fastcall" => Type::__Fastcall,
            "__thiscall" => Type::__Thiscall,
            "__vectorcall" => Type::__Vectorcall,
            "ms_restrict_modifier" => Type::MsRestrictModifier,
            "ms_unsigned_ptr_modifier" => Type::MsUnsignedPtrModifier,
            "ms_signed_ptr_modifier" => Type::MsSignedPtrModifier,
            "_unaligned" => Type::_Unaligned,
            "__unaligned" => Type::__Unaligned,
            "{" => Type::LBrace,
            "}" => Type::RBrace,
            "signed" => Type::Signed,
            "unsigned" => Type::Unsigned,
            "long" => Type::Long,
            "short" => Type::Short,
            "[" => Type::LBracket,
            "static" => Type::Static,
            "]" => Type::RBracket,
            "=" => Type::Eq,
            "auto" => Type::Auto,
            "register" => Type::Register,
            "inline" => Type::Inline,
            "__inline" => Type::__Inline,
            "__inline__" => Type::TS4,
            "__forceinline" => Type::__Forceinline,
            "thread_local" => Type::ThreadLocal,
            "__thread" => Type::__Thread,
            "const" => Type::Const,
            "constexpr" => Type::Constexpr,
            "volatile" => Type::Volatile,
            "restrict" => Type::Restrict,
            "__restrict__" => Type::TS5,
            "_Atomic" => Type::TS6,
            "_Noreturn" => Type::TS7,
            "noreturn" => Type::Noreturn,
            "_Nonnull" => Type::TS8,
            "alignas" => Type::Alignas,
            "_Alignas" => Type::TS9,
            "primitive_type" => Type::PrimitiveType,
            "enum" => Type::Enum,
            ":" => Type::Colon,
            "struct" => Type::Struct,
            "union" => Type::Union,
            "if" => Type::If,
            "else" => Type::Else,
            "switch" => Type::Switch,
            "case" => Type::Case,
            "default" => Type::Default,
            "while" => Type::While,
            "do" => Type::Do,
            "for" => Type::For,
            "return" => Type::Return,
            "break" => Type::Break,
            "continue" => Type::Continue,
            "goto" => Type::Goto,
            "__try" => Type::__Try,
            "__except" => Type::__Except,
            "__finally" => Type::__Finally,
            "__leave" => Type::__Leave,
            "?" => Type::QMark,
            "*=" => Type::StarEq,
            "/=" => Type::SlashEq,
            "%=" => Type::PercentEq,
            "+=" => Type::PlusEq,
            "-=" => Type::DashEq,
            "<<=" => Type::LtLtEq,
            ">>=" => Type::GtGtEq,
            "&=" => Type::AmpEq,
            "^=" => Type::CaretEq,
            "|=" => Type::PipeEq,
            "--" => Type::DashDash,
            "++" => Type::PlusPlus,
            "sizeof" => Type::Sizeof,
            "__alignof__" => Type::TS10,
            "__alignof" => Type::__Alignof,
            "_alignof" => Type::_Alignof,
            "alignof" => Type::Alignof,
            "_Alignof" => Type::TS11,
            "offsetof" => Type::Offsetof,
            "_Generic" => Type::TS12,
            "asm" => Type::Asm,
            "__asm__" => Type::TS13,
            "__asm" => Type::__Asm,
            "__volatile__" => Type::TS14,
            "." => Type::Dot,
            "->" => Type::DashGt,
            "number_literal" => Type::NumberLiteral,
            "L'" => Type::TS15,
            "u'" => Type::TS16,
            "U'" => Type::TS17,
            "u8'" => Type::TS18,
            "'" => Type::SQuote,
            "character" => Type::Character,
            "L\"" => Type::TS19,
            "u\"" => Type::TS20,
            "U\"" => Type::TS21,
            "u8\"" => Type::TS22,
            "\"" => Type::DQuote,
            "string_content" => Type::StringContent,
            "escape_sequence" => Type::EscapeSequence,
            "system_lib_string" => Type::SystemLibString,
            "true" => Type::True,
            "false" => Type::False,
            "NULL" => Type::TS23,
            "nullptr" => Type::Nullptr,
            "comment" => Type::Comment,
            "translation_unit" => Type::TranslationUnit,
            "_top_level_item" => Type::_TopLevelItem,
            "_block_item" => Type::_BlockItem,
            "preproc_include" => Type::PreprocInclude,
            "preproc_def" => Type::PreprocDef,
            "preproc_function_def" => Type::PreprocFunctionDef,
            "preproc_params" => Type::PreprocParams,
            "preproc_call" => Type::PreprocCall,
            "preproc_if" => Type::PreprocIf,
            "preproc_ifdef" => Type::PreprocIfdef,
            "preproc_else" => Type::PreprocElse,
            "preproc_elif" => Type::PreprocElif,
            "preproc_elifdef" => Type::PreprocElifdef,
            "_preproc_expression" => Type::_PreprocExpression,
            "parenthesized_expression" => Type::ParenthesizedExpression,
            "preproc_defined" => Type::PreprocDefined,
            "unary_expression" => Type::UnaryExpression,
            "call_expression" => Type::CallExpression,
            "argument_list" => Type::ArgumentList,
            "binary_expression" => Type::BinaryExpression,
            "function_definition" => Type::FunctionDefinition,
            "declaration" => Type::Declaration,
            "type_definition" => Type::TypeDefinition,
            "_type_definition_type" => Type::_TypeDefinitionType,
            "_type_definition_declarators" => Type::_TypeDefinitionDeclarators,
            "_declaration_modifiers" => Type::_DeclarationModifiers,
            "_declaration_specifiers" => Type::_DeclarationSpecifiers,
            "linkage_specification" => Type::LinkageSpecification,
            "attribute_specifier" => Type::AttributeSpecifier,
            "attribute" => Type::Attribute,
            "attribute_declaration" => Type::AttributeDeclaration,
            "ms_declspec_modifier" => Type::MsDeclspecModifier,
            "ms_based_modifier" => Type::MsBasedModifier,
            "ms_call_modifier" => Type::MsCallModifier,
            "ms_unaligned_ptr_modifier" => Type::MsUnalignedPtrModifier,
            "ms_pointer_modifier" => Type::MsPointerModifier,
            "declaration_list" => Type::DeclarationList,
            "_declarator" => Type::_Declarator,
            "_declaration_declarator" => Type::_DeclarationDeclarator,
            "_field_declarator" => Type::_FieldDeclarator,
            "_type_declarator" => Type::_TypeDeclarator,
            "_abstract_declarator" => Type::_AbstractDeclarator,
            "parenthesized_declarator" => Type::ParenthesizedDeclarator,
            "abstract_parenthesized_declarator" => Type::AbstractParenthesizedDeclarator,
            "attributed_declarator" => Type::AttributedDeclarator,
            "pointer_declarator" => Type::PointerDeclarator,
            "abstract_pointer_declarator" => Type::AbstractPointerDeclarator,
            "function_declarator" => Type::FunctionDeclarator,
            "abstract_function_declarator" => Type::AbstractFunctionDeclarator,
            "array_declarator" => Type::ArrayDeclarator,
            "abstract_array_declarator" => Type::AbstractArrayDeclarator,
            "init_declarator" => Type::InitDeclarator,
            "compound_statement" => Type::CompoundStatement,
            "storage_class_specifier" => Type::StorageClassSpecifier,
            "type_qualifier" => Type::TypeQualifier,
            "alignas_qualifier" => Type::AlignasQualifier,
            "type_specifier" => Type::TypeSpecifier,
            "sized_type_specifier" => Type::SizedTypeSpecifier,
            "enum_specifier" => Type::EnumSpecifier,
            "enumerator_list" => Type::EnumeratorList,
            "struct_specifier" => Type::StructSpecifier,
            "union_specifier" => Type::UnionSpecifier,
            "field_declaration_list" => Type::FieldDeclarationList,
            "_field_declaration_list_item" => Type::_FieldDeclarationListItem,
            "field_declaration" => Type::FieldDeclaration,
            "_field_declaration_declarator" => Type::_FieldDeclarationDeclarator,
            "bitfield_clause" => Type::BitfieldClause,
            "enumerator" => Type::Enumerator,
            "variadic_parameter" => Type::VariadicParameter,
            "parameter_list" => Type::ParameterList,
            "parameter_declaration" => Type::ParameterDeclaration,
            "attributed_statement" => Type::AttributedStatement,
            "statement" => Type::Statement,
            "_top_level_statement" => Type::_TopLevelStatement,
            "labeled_statement" => Type::LabeledStatement,
            "expression_statement" => Type::ExpressionStatement,
            "if_statement" => Type::IfStatement,
            "else_clause" => Type::ElseClause,
            "switch_statement" => Type::SwitchStatement,
            "case_statement" => Type::CaseStatement,
            "while_statement" => Type::WhileStatement,
            "do_statement" => Type::DoStatement,
            "for_statement" => Type::ForStatement,
            "_for_statement_body" => Type::_ForStatementBody,
            "return_statement" => Type::ReturnStatement,
            "break_statement" => Type::BreakStatement,
            "continue_statement" => Type::ContinueStatement,
            "goto_statement" => Type::GotoStatement,
            "seh_try_statement" => Type::SehTryStatement,
            "seh_except_clause" => Type::SehExceptClause,
            "seh_finally_clause" => Type::SehFinallyClause,
            "seh_leave_statement" => Type::SehLeaveStatement,
            "expression" => Type::Expression,
            "_string" => Type::_String,
            "comma_expression" => Type::CommaExpression,
            "conditional_expression" => Type::ConditionalExpression,
            "assignment_expression" => Type::AssignmentExpression,
            "pointer_expression" => Type::PointerExpression,
            "update_expression" => Type::UpdateExpression,
            "cast_expression" => Type::CastExpression,
            "type_descriptor" => Type::TypeDescriptor,
            "sizeof_expression" => Type::SizeofExpression,
            "alignof_expression" => Type::AlignofExpression,
            "offsetof_expression" => Type::OffsetofExpression,
            "generic_expression" => Type::GenericExpression,
            "subscript_expression" => Type::SubscriptExpression,
            "gnu_asm_expression" => Type::GnuAsmExpression,
            "gnu_asm_qualifier" => Type::GnuAsmQualifier,
            "gnu_asm_output_operand_list" => Type::GnuAsmOutputOperandList,
            "gnu_asm_output_operand" => Type::GnuAsmOutputOperand,
            "gnu_asm_input_operand_list" => Type::GnuAsmInputOperandList,
            "gnu_asm_input_operand" => Type::GnuAsmInputOperand,
            "gnu_asm_clobber_list" => Type::GnuAsmClobberList,
            "gnu_asm_goto_list" => Type::GnuAsmGotoList,
            "extension_expression" => Type::ExtensionExpression,
            "field_expression" => Type::FieldExpression,
            "compound_literal_expression" => Type::CompoundLiteralExpression,
            "initializer_list" => Type::InitializerList,
            "initializer_pair" => Type::InitializerPair,
            "subscript_designator" => Type::SubscriptDesignator,
            "subscript_range_designator" => Type::SubscriptRangeDesignator,
            "field_designator" => Type::FieldDesignator,
            "char_literal" => Type::CharLiteral,
            "concatenated_string" => Type::ConcatenatedString,
            "string_literal" => Type::StringLiteral,
            "null" => Type::Null,
            "_empty_declaration" => Type::_EmptyDeclaration,
            "macro_type_specifier" => Type::MacroTypeSpecifier,
            "translation_unit_repeat1" => Type::TranslationUnitRepeat1,
            "preproc_params_repeat1" => Type::PreprocParamsRepeat1,
            "preproc_if_repeat1" => Type::PreprocIfRepeat1,
            "preproc_if_in_field_declaration_list_repeat1" => {
                Type::PreprocIfInFieldDeclarationListRepeat1
            }
            "preproc_if_in_enumerator_list_repeat1" => Type::PreprocIfInEnumeratorListRepeat1,
            "preproc_if_in_enumerator_list_no_comma_repeat1" => {
                Type::PreprocIfInEnumeratorListNoCommaRepeat1
            }
            "preproc_argument_list_repeat1" => Type::PreprocArgumentListRepeat1,
            "_old_style_function_definition_repeat1" => Type::_OldStyleFunctionDefinitionRepeat1,
            "declaration_repeat1" => Type::DeclarationRepeat1,
            "type_definition_repeat1" => Type::TypeDefinitionRepeat1,
            "_type_definition_type_repeat1" => Type::_TypeDefinitionTypeRepeat1,
            "_type_definition_declarators_repeat1" => Type::_TypeDefinitionDeclaratorsRepeat1,
            "_declaration_specifiers_repeat1" => Type::_DeclarationSpecifiersRepeat1,
            "attribute_declaration_repeat1" => Type::AttributeDeclarationRepeat1,
            "attributed_declarator_repeat1" => Type::AttributedDeclaratorRepeat1,
            "pointer_declarator_repeat1" => Type::PointerDeclaratorRepeat1,
            "function_declarator_repeat1" => Type::FunctionDeclaratorRepeat1,
            "array_declarator_repeat1" => Type::ArrayDeclaratorRepeat1,
            "sized_type_specifier_repeat1" => Type::SizedTypeSpecifierRepeat1,
            "enumerator_list_repeat1" => Type::EnumeratorListRepeat1,
            "_field_declaration_declarator_repeat1" => Type::_FieldDeclarationDeclaratorRepeat1,
            "parameter_list_repeat1" => Type::ParameterListRepeat1,
            "_old_style_parameter_list_repeat1" => Type::_OldStyleParameterListRepeat1,
            "case_statement_repeat1" => Type::CaseStatementRepeat1,
            "generic_expression_repeat1" => Type::GenericExpressionRepeat1,
            "gnu_asm_expression_repeat1" => Type::GnuAsmExpressionRepeat1,
            "gnu_asm_output_operand_list_repeat1" => Type::GnuAsmOutputOperandListRepeat1,
            "gnu_asm_input_operand_list_repeat1" => Type::GnuAsmInputOperandListRepeat1,
            "gnu_asm_clobber_list_repeat1" => Type::GnuAsmClobberListRepeat1,
            "gnu_asm_goto_list_repeat1" => Type::GnuAsmGotoListRepeat1,
            "argument_list_repeat1" => Type::ArgumentListRepeat1,
            "initializer_list_repeat1" => Type::InitializerListRepeat1,
            "initializer_pair_repeat1" => Type::InitializerPairRepeat1,
            "char_literal_repeat1" => Type::CharLiteralRepeat1,
            "concatenated_string_repeat1" => Type::ConcatenatedStringRepeat1,
            "string_literal_repeat1" => Type::StringLiteralRepeat1,
            "field_identifier" => Type::FieldIdentifier,
            "statement_identifier" => Type::StatementIdentifier,
            "type_identifier" => Type::TypeIdentifier,
            "Directory" => Type::Directory,
            "Spaces" => Type::Spaces,
            "_ERROR" => Type::_ERROR,
            "ERROR" => Type::ERROR,
            _ => return None,
        })
    }
    pub fn to_str(&self) -> &'static str {
        match self {
            Type::End => "end",
            Type::Identifier => "identifier",
            Type::HashInclude => "#include",
            Type::PreprocIncludeToken2 => "preproc_include_token2",
            Type::HashDefine => "#define",
            Type::LParen => "(",
            Type::DotDotDot => "...",
            Type::Comma => ",",
            Type::RParen => ")",
            Type::HashIf => "#if",
            Type::NewLine => "\n",
            Type::HashEndif => "#endif",
            Type::HashIfdef => "#ifdef",
            Type::HashIfndef => "#ifndef",
            Type::HashElse => "#else",
            Type::HashElif => "#elif",
            Type::HashElifdef => "#elifdef",
            Type::HashElifndef => "#elifndef",
            Type::PreprocArg => "preproc_arg",
            Type::PreprocDirective => "preproc_directive",
            Type::Defined => "defined",
            Type::Bang => "!",
            Type::Tilde => "~",
            Type::Dash => "-",
            Type::Plus => "+",
            Type::Star => "*",
            Type::Slash => "/",
            Type::Percent => "%",
            Type::PipePipe => "||",
            Type::AmpAmp => "&&",
            Type::Pipe => "|",
            Type::Caret => "^",
            Type::Amp => "&",
            Type::EqEq => "==",
            Type::BangEq => "!=",
            Type::GT => ">",
            Type::GTEq => ">=",
            Type::LTEq => "<=",
            Type::LT => "<",
            Type::LtLt => "<<",
            Type::GtGt => ">>",
            Type::SemiColon => ";",
            Type::TS0 => "__extension__",
            Type::Typedef => "typedef",
            Type::Extern => "extern",
            Type::TS1 => "__attribute__",
            Type::__Attribute => "__attribute",
            Type::ColonColon => "::",
            Type::TS2 => "[[",
            Type::TS3 => "]]",
            Type::__Declspec => "__declspec",
            Type::__Based => "__based",
            Type::__Cdecl => "__cdecl",
            Type::__Clrcall => "__clrcall",
            Type::__Stdcall => "__stdcall",
            Type::__Fastcall => "__fastcall",
            Type::__Thiscall => "__thiscall",
            Type::__Vectorcall => "__vectorcall",
            Type::MsRestrictModifier => "ms_restrict_modifier",
            Type::MsUnsignedPtrModifier => "ms_unsigned_ptr_modifier",
            Type::MsSignedPtrModifier => "ms_signed_ptr_modifier",
            Type::_Unaligned => "_unaligned",
            Type::__Unaligned => "__unaligned",
            Type::LBrace => "{",
            Type::RBrace => "}",
            Type::Signed => "signed",
            Type::Unsigned => "unsigned",
            Type::Long => "long",
            Type::Short => "short",
            Type::LBracket => "[",
            Type::Static => "static",
            Type::RBracket => "]",
            Type::Eq => "=",
            Type::Auto => "auto",
            Type::Register => "register",
            Type::Inline => "inline",
            Type::__Inline => "__inline",
            Type::TS4 => "__inline__",
            Type::__Forceinline => "__forceinline",
            Type::ThreadLocal => "thread_local",
            Type::__Thread => "__thread",
            Type::Const => "const",
            Type::Constexpr => "constexpr",
            Type::Volatile => "volatile",
            Type::Restrict => "restrict",
            Type::TS5 => "__restrict__",
            Type::TS6 => "_Atomic",
            Type::TS7 => "_Noreturn",
            Type::Noreturn => "noreturn",
            Type::TS8 => "_Nonnull",
            Type::Alignas => "alignas",
            Type::TS9 => "_Alignas",
            Type::PrimitiveType => "primitive_type",
            Type::Enum => "enum",
            Type::Colon => ":",
            Type::Struct => "struct",
            Type::Union => "union",
            Type::If => "if",
            Type::Else => "else",
            Type::Switch => "switch",
            Type::Case => "case",
            Type::Default => "default",
            Type::While => "while",
            Type::Do => "do",
            Type::For => "for",
            Type::Return => "return",
            Type::Break => "break",
            Type::Continue => "continue",
            Type::Goto => "goto",
            Type::__Try => "__try",
            Type::__Except => "__except",
            Type::__Finally => "__finally",
            Type::__Leave => "__leave",
            Type::QMark => "?",
            Type::StarEq => "*=",
            Type::SlashEq => "/=",
            Type::PercentEq => "%=",
            Type::PlusEq => "+=",
            Type::DashEq => "-=",
            Type::LtLtEq => "<<=",
            Type::GtGtEq => ">>=",
            Type::AmpEq => "&=",
            Type::CaretEq => "^=",
            Type::PipeEq => "|=",
            Type::DashDash => "--",
            Type::PlusPlus => "++",
            Type::Sizeof => "sizeof",
            Type::TS10 => "__alignof__",
            Type::__Alignof => "__alignof",
            Type::_Alignof => "_alignof",
            Type::Alignof => "alignof",
            Type::TS11 => "_Alignof",
            Type::Offsetof => "offsetof",
            Type::TS12 => "_Generic",
            Type::Asm => "asm",
            Type::TS13 => "__asm__",
            Type::__Asm => "__asm",
            Type::TS14 => "__volatile__",
            Type::Dot => ".",
            Type::DashGt => "->",
            Type::NumberLiteral => "number_literal",
            Type::TS15 => "L'",
            Type::TS16 => "u'",
            Type::TS17 => "U'",
            Type::TS18 => "u8'",
            Type::SQuote => "'",
            Type::Character => "character",
            Type::TS19 => "L\"",
            Type::TS20 => "u\"",
            Type::TS21 => "U\"",
            Type::TS22 => "u8\"",
            Type::DQuote => "\"",
            Type::StringContent => "string_content",
            Type::EscapeSequence => "escape_sequence",
            Type::SystemLibString => "system_lib_string",
            Type::True => "true",
            Type::False => "false",
            Type::TS23 => "NULL",
            Type::Nullptr => "nullptr",
            Type::Comment => "comment",
            Type::TranslationUnit => "translation_unit",
            Type::_TopLevelItem => "_top_level_item",
            Type::_BlockItem => "_block_item",
            Type::PreprocInclude => "preproc_include",
            Type::PreprocDef => "preproc_def",
            Type::PreprocFunctionDef => "preproc_function_def",
            Type::PreprocParams => "preproc_params",
            Type::PreprocCall => "preproc_call",
            Type::PreprocIf => "preproc_if",
            Type::PreprocIfdef => "preproc_ifdef",
            Type::PreprocElse => "preproc_else",
            Type::PreprocElif => "preproc_elif",
            Type::PreprocElifdef => "preproc_elifdef",
            Type::_PreprocExpression => "_preproc_expression",
            Type::ParenthesizedExpression => "parenthesized_expression",
            Type::PreprocDefined => "preproc_defined",
            Type::UnaryExpression => "unary_expression",
            Type::CallExpression => "call_expression",
            Type::ArgumentList => "argument_list",
            Type::BinaryExpression => "binary_expression",
            Type::FunctionDefinition => "function_definition",
            Type::Declaration => "declaration",
            Type::TypeDefinition => "type_definition",
            Type::_TypeDefinitionType => "_type_definition_type",
            Type::_TypeDefinitionDeclarators => "_type_definition_declarators",
            Type::_DeclarationModifiers => "_declaration_modifiers",
            Type::_DeclarationSpecifiers => "_declaration_specifiers",
            Type::LinkageSpecification => "linkage_specification",
            Type::AttributeSpecifier => "attribute_specifier",
            Type::Attribute => "attribute",
            Type::AttributeDeclaration => "attribute_declaration",
            Type::MsDeclspecModifier => "ms_declspec_modifier",
            Type::MsBasedModifier => "ms_based_modifier",
            Type::MsCallModifier => "ms_call_modifier",
            Type::MsUnalignedPtrModifier => "ms_unaligned_ptr_modifier",
            Type::MsPointerModifier => "ms_pointer_modifier",
            Type::DeclarationList => "declaration_list",
            Type::_Declarator => "_declarator",
            Type::_DeclarationDeclarator => "_declaration_declarator",
            Type::_FieldDeclarator => "_field_declarator",
            Type::_TypeDeclarator => "_type_declarator",
            Type::_AbstractDeclarator => "_abstract_declarator",
            Type::ParenthesizedDeclarator => "parenthesized_declarator",
            Type::AbstractParenthesizedDeclarator => "abstract_parenthesized_declarator",
            Type::AttributedDeclarator => "attributed_declarator",
            Type::PointerDeclarator => "pointer_declarator",
            Type::AbstractPointerDeclarator => "abstract_pointer_declarator",
            Type::FunctionDeclarator => "function_declarator",
            Type::AbstractFunctionDeclarator => "abstract_function_declarator",
            Type::ArrayDeclarator => "array_declarator",
            Type::AbstractArrayDeclarator => "abstract_array_declarator",
            Type::InitDeclarator => "init_declarator",
            Type::CompoundStatement => "compound_statement",
            Type::StorageClassSpecifier => "storage_class_specifier",
            Type::TypeQualifier => "type_qualifier",
            Type::AlignasQualifier => "alignas_qualifier",
            Type::TypeSpecifier => "type_specifier",
            Type::SizedTypeSpecifier => "sized_type_specifier",
            Type::EnumSpecifier => "enum_specifier",
            Type::EnumeratorList => "enumerator_list",
            Type::StructSpecifier => "struct_specifier",
            Type::UnionSpecifier => "union_specifier",
            Type::FieldDeclarationList => "field_declaration_list",
            Type::_FieldDeclarationListItem => "_field_declaration_list_item",
            Type::FieldDeclaration => "field_declaration",
            Type::_FieldDeclarationDeclarator => "_field_declaration_declarator",
            Type::BitfieldClause => "bitfield_clause",
            Type::Enumerator => "enumerator",
            Type::VariadicParameter => "variadic_parameter",
            Type::ParameterList => "parameter_list",
            Type::ParameterDeclaration => "parameter_declaration",
            Type::AttributedStatement => "attributed_statement",
            Type::Statement => "statement",
            Type::_TopLevelStatement => "_top_level_statement",
            Type::LabeledStatement => "labeled_statement",
            Type::ExpressionStatement => "expression_statement",
            Type::IfStatement => "if_statement",
            Type::ElseClause => "else_clause",
            Type::SwitchStatement => "switch_statement",
            Type::CaseStatement => "case_statement",
            Type::WhileStatement => "while_statement",
            Type::DoStatement => "do_statement",
            Type::ForStatement => "for_statement",
            Type::_ForStatementBody => "_for_statement_body",
            Type::ReturnStatement => "return_statement",
            Type::BreakStatement => "break_statement",
            Type::ContinueStatement => "continue_statement",
            Type::GotoStatement => "goto_statement",
            Type::SehTryStatement => "seh_try_statement",
            Type::SehExceptClause => "seh_except_clause",
            Type::SehFinallyClause => "seh_finally_clause",
            Type::SehLeaveStatement => "seh_leave_statement",
            Type::Expression => "expression",
            Type::_String => "_string",
            Type::CommaExpression => "comma_expression",
            Type::ConditionalExpression => "conditional_expression",
            Type::AssignmentExpression => "assignment_expression",
            Type::PointerExpression => "pointer_expression",
            Type::UpdateExpression => "update_expression",
            Type::CastExpression => "cast_expression",
            Type::TypeDescriptor => "type_descriptor",
            Type::SizeofExpression => "sizeof_expression",
            Type::AlignofExpression => "alignof_expression",
            Type::OffsetofExpression => "offsetof_expression",
            Type::GenericExpression => "generic_expression",
            Type::SubscriptExpression => "subscript_expression",
            Type::GnuAsmExpression => "gnu_asm_expression",
            Type::GnuAsmQualifier => "gnu_asm_qualifier",
            Type::GnuAsmOutputOperandList => "gnu_asm_output_operand_list",
            Type::GnuAsmOutputOperand => "gnu_asm_output_operand",
            Type::GnuAsmInputOperandList => "gnu_asm_input_operand_list",
            Type::GnuAsmInputOperand => "gnu_asm_input_operand",
            Type::GnuAsmClobberList => "gnu_asm_clobber_list",
            Type::GnuAsmGotoList => "gnu_asm_goto_list",
            Type::ExtensionExpression => "extension_expression",
            Type::FieldExpression => "field_expression",
            Type::CompoundLiteralExpression => "compound_literal_expression",
            Type::InitializerList => "initializer_list",
            Type::InitializerPair => "initializer_pair",
            Type::SubscriptDesignator => "subscript_designator",
            Type::SubscriptRangeDesignator => "subscript_range_designator",
            Type::FieldDesignator => "field_designator",
            Type::CharLiteral => "char_literal",
            Type::ConcatenatedString => "concatenated_string",
            Type::StringLiteral => "string_literal",
            Type::Null => "null",
            Type::_EmptyDeclaration => "_empty_declaration",
            Type::MacroTypeSpecifier => "macro_type_specifier",
            Type::TranslationUnitRepeat1 => "translation_unit_repeat1",
            Type::PreprocParamsRepeat1 => "preproc_params_repeat1",
            Type::PreprocIfRepeat1 => "preproc_if_repeat1",
            Type::PreprocIfInFieldDeclarationListRepeat1 => {
                "preproc_if_in_field_declaration_list_repeat1"
            }
            Type::PreprocIfInEnumeratorListRepeat1 => "preproc_if_in_enumerator_list_repeat1",
            Type::PreprocIfInEnumeratorListNoCommaRepeat1 => {
                "preproc_if_in_enumerator_list_no_comma_repeat1"
            }
            Type::PreprocArgumentListRepeat1 => "preproc_argument_list_repeat1",
            Type::_OldStyleFunctionDefinitionRepeat1 => "_old_style_function_definition_repeat1",
            Type::DeclarationRepeat1 => "declaration_repeat1",
            Type::TypeDefinitionRepeat1 => "type_definition_repeat1",
            Type::_TypeDefinitionTypeRepeat1 => "_type_definition_type_repeat1",
            Type::_TypeDefinitionDeclaratorsRepeat1 => "_type_definition_declarators_repeat1",
            Type::_DeclarationSpecifiersRepeat1 => "_declaration_specifiers_repeat1",
            Type::AttributeDeclarationRepeat1 => "attribute_declaration_repeat1",
            Type::AttributedDeclaratorRepeat1 => "attributed_declarator_repeat1",
            Type::PointerDeclaratorRepeat1 => "pointer_declarator_repeat1",
            Type::FunctionDeclaratorRepeat1 => "function_declarator_repeat1",
            Type::ArrayDeclaratorRepeat1 => "array_declarator_repeat1",
            Type::SizedTypeSpecifierRepeat1 => "sized_type_specifier_repeat1",
            Type::EnumeratorListRepeat1 => "enumerator_list_repeat1",
            Type::_FieldDeclarationDeclaratorRepeat1 => "_field_declaration_declarator_repeat1",
            Type::ParameterListRepeat1 => "parameter_list_repeat1",
            Type::_OldStyleParameterListRepeat1 => "_old_style_parameter_list_repeat1",
            Type::CaseStatementRepeat1 => "case_statement_repeat1",
            Type::GenericExpressionRepeat1 => "generic_expression_repeat1",
            Type::GnuAsmExpressionRepeat1 => "gnu_asm_expression_repeat1",
            Type::GnuAsmOutputOperandListRepeat1 => "gnu_asm_output_operand_list_repeat1",
            Type::GnuAsmInputOperandListRepeat1 => "gnu_asm_input_operand_list_repeat1",
            Type::GnuAsmClobberListRepeat1 => "gnu_asm_clobber_list_repeat1",
            Type::GnuAsmGotoListRepeat1 => "gnu_asm_goto_list_repeat1",
            Type::ArgumentListRepeat1 => "argument_list_repeat1",
            Type::InitializerListRepeat1 => "initializer_list_repeat1",
            Type::InitializerPairRepeat1 => "initializer_pair_repeat1",
            Type::CharLiteralRepeat1 => "char_literal_repeat1",
            Type::ConcatenatedStringRepeat1 => "concatenated_string_repeat1",
            Type::StringLiteralRepeat1 => "string_literal_repeat1",
            Type::FieldIdentifier => "field_identifier",
            Type::StatementIdentifier => "statement_identifier",
            Type::TypeIdentifier => "type_identifier",
            Type::Spaces => "Spaces",
            Type::Directory => "Directory",
            Type::_ERROR => "_ERROR",
            Type::ERROR => "ERROR",
        }
    }
    pub fn is_hidden(&self) -> bool {
        match self {
            Type::End => true,
            Type::PreprocIncludeToken2 => true,
            Type::_TopLevelItem => true,
            Type::_BlockItem => true,
            Type::_PreprocExpression => true,
            Type::_TypeDefinitionType => true,
            Type::_TypeDefinitionDeclarators => true,
            Type::_DeclarationModifiers => true,
            Type::_DeclarationSpecifiers => true,
            Type::_Declarator => true,
            Type::_DeclarationDeclarator => true,
            Type::_FieldDeclarator => true,
            Type::_TypeDeclarator => true,
            Type::_AbstractDeclarator => true,
            Type::TypeSpecifier => true,
            Type::_FieldDeclarationListItem => true,
            Type::_FieldDeclarationDeclarator => true,
            Type::Statement => true,
            Type::_TopLevelStatement => true,
            Type::_ForStatementBody => true,
            Type::Expression => true,
            Type::_String => true,
            Type::_EmptyDeclaration => true,
            Type::TranslationUnitRepeat1 => true,
            Type::PreprocParamsRepeat1 => true,
            Type::PreprocIfRepeat1 => true,
            Type::PreprocIfInFieldDeclarationListRepeat1 => true,
            Type::PreprocIfInEnumeratorListRepeat1 => true,
            Type::PreprocIfInEnumeratorListNoCommaRepeat1 => true,
            Type::PreprocArgumentListRepeat1 => true,
            Type::_OldStyleFunctionDefinitionRepeat1 => true,
            Type::DeclarationRepeat1 => true,
            Type::TypeDefinitionRepeat1 => true,
            Type::_TypeDefinitionTypeRepeat1 => true,
            Type::_TypeDefinitionDeclaratorsRepeat1 => true,
            Type::_DeclarationSpecifiersRepeat1 => true,
            Type::AttributeDeclarationRepeat1 => true,
            Type::AttributedDeclaratorRepeat1 => true,
            Type::PointerDeclaratorRepeat1 => true,
            Type::FunctionDeclaratorRepeat1 => true,
            Type::ArrayDeclaratorRepeat1 => true,
            Type::SizedTypeSpecifierRepeat1 => true,
            Type::EnumeratorListRepeat1 => true,
            Type::_FieldDeclarationDeclaratorRepeat1 => true,
            Type::ParameterListRepeat1 => true,
            Type::_OldStyleParameterListRepeat1 => true,
            Type::CaseStatementRepeat1 => true,
            Type::GenericExpressionRepeat1 => true,
            Type::GnuAsmExpressionRepeat1 => true,
            Type::GnuAsmOutputOperandListRepeat1 => true,
            Type::GnuAsmInputOperandListRepeat1 => true,
            Type::GnuAsmClobberListRepeat1 => true,
            Type::GnuAsmGotoListRepeat1 => true,
            Type::ArgumentListRepeat1 => true,
            Type::InitializerListRepeat1 => true,
            Type::InitializerPairRepeat1 => true,
            Type::CharLiteralRepeat1 => true,
            Type::ConcatenatedStringRepeat1 => true,
            Type::StringLiteralRepeat1 => true,
            _ => false,
        }
    }
    pub fn is_supertype(&self) -> bool {
        match self {
            Type::_Declarator => true,
            Type::_FieldDeclarator => true,
            Type::_TypeDeclarator => true,
            Type::_AbstractDeclarator => true,
            Type::TypeSpecifier => true,
            Type::Statement => true,
            Type::Expression => true,
            _ => false,
        }
    }
    pub fn is_named(&self) -> bool {
        match self {
            Type::Identifier => true,
            Type::PreprocArg => true,
            Type::PreprocDirective => true,
            Type::MsRestrictModifier => true,
            Type::MsUnsignedPtrModifier => true,
            Type::MsSignedPtrModifier => true,
            Type::PrimitiveType => true,
            Type::NumberLiteral => true,
            Type::Character => true,
            Type::StringContent => true,
            Type::EscapeSequence => true,
            Type::SystemLibString => true,
            Type::True => true,
            Type::False => true,
            Type::Comment => true,
            Type::TranslationUnit => true,
            Type::PreprocInclude => true,
            Type::PreprocDef => true,
            Type::PreprocFunctionDef => true,
            Type::PreprocParams => true,
            Type::PreprocCall => true,
            Type::PreprocIf => true,
            Type::PreprocIfdef => true,
            Type::PreprocElse => true,
            Type::PreprocElif => true,
            Type::PreprocElifdef => true,
            Type::ParenthesizedExpression => true,
            Type::PreprocDefined => true,
            Type::UnaryExpression => true,
            Type::CallExpression => true,
            Type::ArgumentList => true,
            Type::BinaryExpression => true,
            Type::FunctionDefinition => true,
            Type::Declaration => true,
            Type::TypeDefinition => true,
            Type::LinkageSpecification => true,
            Type::AttributeSpecifier => true,
            Type::Attribute => true,
            Type::AttributeDeclaration => true,
            Type::MsDeclspecModifier => true,
            Type::MsBasedModifier => true,
            Type::MsCallModifier => true,
            Type::MsUnalignedPtrModifier => true,
            Type::MsPointerModifier => true,
            Type::DeclarationList => true,
            Type::_Declarator => true,
            Type::_FieldDeclarator => true,
            Type::_TypeDeclarator => true,
            Type::_AbstractDeclarator => true,
            Type::ParenthesizedDeclarator => true,
            Type::AbstractParenthesizedDeclarator => true,
            Type::AttributedDeclarator => true,
            Type::PointerDeclarator => true,
            Type::AbstractPointerDeclarator => true,
            Type::FunctionDeclarator => true,
            Type::AbstractFunctionDeclarator => true,
            Type::ArrayDeclarator => true,
            Type::AbstractArrayDeclarator => true,
            Type::InitDeclarator => true,
            Type::CompoundStatement => true,
            Type::StorageClassSpecifier => true,
            Type::TypeQualifier => true,
            Type::AlignasQualifier => true,
            Type::TypeSpecifier => true,
            Type::SizedTypeSpecifier => true,
            Type::EnumSpecifier => true,
            Type::EnumeratorList => true,
            Type::StructSpecifier => true,
            Type::UnionSpecifier => true,
            Type::FieldDeclarationList => true,
            Type::FieldDeclaration => true,
            Type::BitfieldClause => true,
            Type::Enumerator => true,
            Type::VariadicParameter => true,
            Type::ParameterList => true,
            Type::ParameterDeclaration => true,
            Type::AttributedStatement => true,
            Type::Statement => true,
            Type::LabeledStatement => true,
            Type::ExpressionStatement => true,
            Type::IfStatement => true,
            Type::ElseClause => true,
            Type::SwitchStatement => true,
            Type::CaseStatement => true,
            Type::WhileStatement => true,
            Type::DoStatement => true,
            Type::ForStatement => true,
            Type::ReturnStatement => true,
            Type::BreakStatement => true,
            Type::ContinueStatement => true,
            Type::GotoStatement => true,
            Type::SehTryStatement => true,
            Type::SehExceptClause => true,
            Type::SehFinallyClause => true,
            Type::SehLeaveStatement => true,
            Type::Expression => true,
            Type::CommaExpression => true,
            Type::ConditionalExpression => true,
            Type::AssignmentExpression => true,
            Type::PointerExpression => true,
            Type::UpdateExpression => true,
            Type::CastExpression => true,
            Type::TypeDescriptor => true,
            Type::SizeofExpression => true,
            Type::AlignofExpression => true,
            Type::OffsetofExpression => true,
            Type::GenericExpression => true,
            Type::SubscriptExpression => true,
            Type::GnuAsmExpression => true,
            Type::GnuAsmQualifier => true,
            Type::GnuAsmOutputOperandList => true,
            Type::GnuAsmOutputOperand => true,
            Type::GnuAsmInputOperandList => true,
            Type::GnuAsmInputOperand => true,
            Type::GnuAsmClobberList => true,
            Type::GnuAsmGotoList => true,
            Type::ExtensionExpression => true,
            Type::FieldExpression => true,
            Type::CompoundLiteralExpression => true,
            Type::InitializerList => true,
            Type::InitializerPair => true,
            Type::SubscriptDesignator => true,
            Type::SubscriptRangeDesignator => true,
            Type::FieldDesignator => true,
            Type::CharLiteral => true,
            Type::ConcatenatedString => true,
            Type::StringLiteral => true,
            Type::Null => true,
            Type::MacroTypeSpecifier => true,
            Type::FieldIdentifier => true,
            Type::StatementIdentifier => true,
            Type::TypeIdentifier => true,
            _ => false,
        }
    }
}

#[test]
fn test_tslanguage_and_type_identity() {
    let l = crate::language();
    assert_eq!(l.node_kind_count(), S_T_L.len());
    for id in 0..l.node_kind_count() {
        let kind = l.node_kind_for_id(id as u16).unwrap();
        let ty = Type::from_u16(id as u16);
        assert_eq!(ty.to_str(), kind);
    }
}

const S_T_L: &'static [Type] = &[
    Type::End,
    Type::Identifier,
    Type::HashInclude,
    Type::PreprocIncludeToken2,
    Type::HashDefine,
    Type::LParen,
    Type::DotDotDot,
    Type::Comma,
    Type::RParen,
    Type::HashIf,
    Type::NewLine,
    Type::HashEndif,
    Type::HashIfdef,
    Type::HashIfndef,
    Type::HashElse,
    Type::HashElif,
    Type::HashElifdef,
    Type::HashElifndef,
    Type::PreprocArg,
    Type::PreprocDirective,
    Type::Defined,
    Type::Bang,
    Type::Tilde,
    Type::Dash,
    Type::Plus,
    Type::Star,
    Type::Slash,
    Type::Percent,
    Type::PipePipe,
    Type::AmpAmp,
    Type::Pipe,
    Type::Caret,
    Type::Amp,
    Type::EqEq,
    Type::BangEq,
    Type::GT,
    Type::GTEq,
    Type::LTEq,
    Type::LT,
    Type::LtLt,
    Type::GtGt,
    Type::SemiColon,
    Type::TS0,
    Type::Typedef,
    Type::Extern,
    Type::TS1,
    Type::__Attribute,
    Type::ColonColon,
    Type::TS2,
    Type::TS3,
    Type::__Declspec,
    Type::__Based,
    Type::__Cdecl,
    Type::__Clrcall,
    Type::__Stdcall,
    Type::__Fastcall,
    Type::__Thiscall,
    Type::__Vectorcall,
    Type::MsRestrictModifier,
    Type::MsUnsignedPtrModifier,
    Type::MsSignedPtrModifier,
    Type::_Unaligned,
    Type::__Unaligned,
    Type::LBrace,
    Type::RBrace,
    Type::Signed,
    Type::Unsigned,
    Type::Long,
    Type::Short,
    Type::LBracket,
    Type::Static,
    Type::RBracket,
    Type::Eq,
    Type::Auto,
    Type::Register,
    Type::Inline,
    Type::__Inline,
    Type::TS4,
    Type::__Forceinline,
    Type::ThreadLocal,
    Type::__Thread,
    Type::Const,
    Type::Constexpr,
    Type::Volatile,
    Type::Restrict,
    Type::TS5,
    Type::TS6,
    Type::TS7,
    Type::Noreturn,
    Type::TS8,
    Type::Alignas,
    Type::TS9,
    Type::PrimitiveType,
    Type::Enum,
    Type::Colon,
    Type::Struct,
    Type::Union,
    Type::If,
    Type::Else,
    Type::Switch,
    Type::Case,
    Type::Default,
    Type::While,
    Type::Do,
    Type::For,
    Type::Return,
    Type::Break,
    Type::Continue,
    Type::Goto,
    Type::__Try,
    Type::__Except,
    Type::__Finally,
    Type::__Leave,
    Type::QMark,
    Type::StarEq,
    Type::SlashEq,
    Type::PercentEq,
    Type::PlusEq,
    Type::DashEq,
    Type::LtLtEq,
    Type::GtGtEq,
    Type::AmpEq,
    Type::CaretEq,
    Type::PipeEq,
    Type::DashDash,
    Type::PlusPlus,
    Type::Sizeof,
    Type::TS10,
    Type::__Alignof,
    Type::_Alignof,
    Type::Alignof,
    Type::TS11,
    Type::Offsetof,
    Type::TS12,
    Type::Asm,
    Type::TS13,
    Type::__Asm,
    Type::TS14,
    Type::Dot,
    Type::DashGt,
    Type::NumberLiteral,
    Type::TS15,
    Type::TS16,
    Type::TS17,
    Type::TS18,
    Type::SQuote,
    Type::Character,
    Type::TS19,
    Type::TS20,
    Type::TS21,
    Type::TS22,
    Type::DQuote,
    Type::StringContent,
    Type::EscapeSequence,
    Type::SystemLibString,
    Type::True,
    Type::False,
    Type::TS23,
    Type::Nullptr,
    Type::Comment,
    Type::TranslationUnit,
    Type::_TopLevelItem,
    Type::_BlockItem,
    Type::PreprocInclude,
    Type::PreprocDef,
    Type::PreprocFunctionDef,
    Type::PreprocParams,
    Type::PreprocCall,
    Type::PreprocIf,
    Type::PreprocIfdef,
    Type::PreprocElse,
    Type::PreprocElif,
    Type::PreprocElifdef,
    Type::_PreprocExpression,
    Type::ParenthesizedExpression,
    Type::PreprocDefined,
    Type::UnaryExpression,
    Type::CallExpression,
    Type::ArgumentList,
    Type::BinaryExpression,
    Type::FunctionDefinition,
    Type::Declaration,
    Type::TypeDefinition,
    Type::_TypeDefinitionType,
    Type::_TypeDefinitionDeclarators,
    Type::_DeclarationModifiers,
    Type::_DeclarationSpecifiers,
    Type::LinkageSpecification,
    Type::AttributeSpecifier,
    Type::Attribute,
    Type::AttributeDeclaration,
    Type::MsDeclspecModifier,
    Type::MsBasedModifier,
    Type::MsCallModifier,
    Type::MsUnalignedPtrModifier,
    Type::MsPointerModifier,
    Type::DeclarationList,
    Type::_Declarator,
    Type::_DeclarationDeclarator,
    Type::_FieldDeclarator,
    Type::_TypeDeclarator,
    Type::_AbstractDeclarator,
    Type::ParenthesizedDeclarator,
    Type::AbstractParenthesizedDeclarator,
    Type::AttributedDeclarator,
    Type::PointerDeclarator,
    Type::AbstractPointerDeclarator,
    Type::FunctionDeclarator,
    Type::AbstractFunctionDeclarator,
    Type::ArrayDeclarator,
    Type::AbstractArrayDeclarator,
    Type::InitDeclarator,
    Type::CompoundStatement,
    Type::StorageClassSpecifier,
    Type::TypeQualifier,
    Type::AlignasQualifier,
    Type::TypeSpecifier,
    Type::SizedTypeSpecifier,
    Type::EnumSpecifier,
    Type::EnumeratorList,
    Type::StructSpecifier,
    Type::UnionSpecifier,
    Type::FieldDeclarationList,
    Type::_FieldDeclarationListItem,
    Type::FieldDeclaration,
    Type::_FieldDeclarationDeclarator,
    Type::BitfieldClause,
    Type::Enumerator,
    Type::VariadicParameter,
    Type::ParameterList,
    Type::ParameterDeclaration,
    Type::AttributedStatement,
    Type::Statement,
    Type::_TopLevelStatement,
    Type::LabeledStatement,
    Type::ExpressionStatement,
    Type::IfStatement,
    Type::ElseClause,
    Type::SwitchStatement,
    Type::CaseStatement,
    Type::WhileStatement,
    Type::DoStatement,
    Type::ForStatement,
    Type::_ForStatementBody,
    Type::ReturnStatement,
    Type::BreakStatement,
    Type::ContinueStatement,
    Type::GotoStatement,
    Type::SehTryStatement,
    Type::SehExceptClause,
    Type::SehFinallyClause,
    Type::SehLeaveStatement,
    Type::Expression,
    Type::_String,
    Type::CommaExpression,
    Type::ConditionalExpression,
    Type::AssignmentExpression,
    Type::PointerExpression,
    Type::UpdateExpression,
    Type::CastExpression,
    Type::TypeDescriptor,
    Type::SizeofExpression,
    Type::AlignofExpression,
    Type::OffsetofExpression,
    Type::GenericExpression,
    Type::SubscriptExpression,
    Type::GnuAsmExpression,
    Type::GnuAsmQualifier,
    Type::GnuAsmOutputOperandList,
    Type::GnuAsmOutputOperand,
    Type::GnuAsmInputOperandList,
    Type::GnuAsmInputOperand,
    Type::GnuAsmClobberList,
    Type::GnuAsmGotoList,
    Type::ExtensionExpression,
    Type::FieldExpression,
    Type::CompoundLiteralExpression,
    Type::InitializerList,
    Type::InitializerPair,
    Type::SubscriptDesignator,
    Type::SubscriptRangeDesignator,
    Type::FieldDesignator,
    Type::CharLiteral,
    Type::ConcatenatedString,
    Type::StringLiteral,
    Type::Null,
    Type::_EmptyDeclaration,
    Type::MacroTypeSpecifier,
    Type::TranslationUnitRepeat1,
    Type::PreprocParamsRepeat1,
    Type::PreprocIfRepeat1,
    Type::PreprocIfInFieldDeclarationListRepeat1,
    Type::PreprocIfInEnumeratorListRepeat1,
    Type::PreprocIfInEnumeratorListNoCommaRepeat1,
    Type::PreprocArgumentListRepeat1,
    Type::_OldStyleFunctionDefinitionRepeat1,
    Type::DeclarationRepeat1,
    Type::TypeDefinitionRepeat1,
    Type::_TypeDefinitionTypeRepeat1,
    Type::_TypeDefinitionDeclaratorsRepeat1,
    Type::_DeclarationSpecifiersRepeat1,
    Type::AttributeDeclarationRepeat1,
    Type::AttributedDeclaratorRepeat1,
    Type::PointerDeclaratorRepeat1,
    Type::FunctionDeclaratorRepeat1,
    Type::ArrayDeclaratorRepeat1,
    Type::SizedTypeSpecifierRepeat1,
    Type::EnumeratorListRepeat1,
    Type::_FieldDeclarationDeclaratorRepeat1,
    Type::ParameterListRepeat1,
    Type::_OldStyleParameterListRepeat1,
    Type::CaseStatementRepeat1,
    Type::GenericExpressionRepeat1,
    Type::GnuAsmExpressionRepeat1,
    Type::GnuAsmOutputOperandListRepeat1,
    Type::GnuAsmInputOperandListRepeat1,
    Type::GnuAsmClobberListRepeat1,
    Type::GnuAsmGotoListRepeat1,
    Type::ArgumentListRepeat1,
    Type::InitializerListRepeat1,
    Type::InitializerPairRepeat1,
    Type::CharLiteralRepeat1,
    Type::ConcatenatedStringRepeat1,
    Type::StringLiteralRepeat1,
    Type::FieldIdentifier,
    Type::StatementIdentifier,
    Type::TypeIdentifier,
];
