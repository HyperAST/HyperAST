use std::fmt::Display;

use hyperast::{
    store::defaults::NodeIdentifier,
    tree_gen::utils_ts::TsEnableTS,
    types::{
        AnyType, HyperType, LangRef, NodeId, RoleStore, TypeStore, TypeTrait, TypeU16, TypedNodeId, AAAA,
    },
};

#[cfg(feature = "legion")]
mod legion_impls {
    use super::*;

    impl<'a> hyperast::types::ETypeStore for TStore {
        type Ty2 = Type;

        fn intern(ty: Self::Ty2) -> Self::Ty {
            TType::new(ty)
        }
    }

    impl TsEnableTS for TStore {
        fn obtain_type<'a, N: hyperast::tree_gen::parser::NodeWithU16TypeId>(
            n: &N,
        ) -> <Self as hyperast::types::ETypeStore>::Ty2 {
            let k = n.kind_id();
            Type::from_u16(k)
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

    use hyperast::{
        store::nodes::legion::HashedNodeRef,
        tree_gen::utils_ts::{TsEnableTS, TsType},
        types::LangWrapper,
    };

    impl TypeStore for TStore {
        type Ty = TypeU16<TsQuery>;
    }
    impl TypeStore for _TStore {
        type Ty = Type;
        fn decompress_type(
            erazed: &impl hyperast::types::ErasedHolder,
            tid: std::any::TypeId,
        ) -> Self::Ty {
            erazed
                .unerase_ref::<TypeU16<TsQuery>>(std::any::TypeId::of::<TypeU16<TsQuery>>())
                .unwrap()
                .e()
        }
    }

    impl RoleStore for TStore {
        type IdF = u16;

        type Role = hyperast::types::Role;

        fn resolve_field(_lang: LangWrapper<Self::Ty>, field_id: Self::IdF) -> Self::Role {
            let s = tree_sitter_query::language()
                .field_name_for_id(field_id)
                .ok_or_else(|| format!("{}", field_id))
                .unwrap();
            hyperast::types::Role::try_from(s).expect(s)
        }

        fn intern_role(_lang: LangWrapper<Self::Ty>, role: Self::Role) -> Self::IdF {
            let field_name = role.to_string();
            tree_sitter_query::language()
                .field_id_for_name(field_name)
                .unwrap()
                .into()
        }
    }

    // impl<'a> TsQueryEnabledTypeStore<HashedNodeRef<'a, TIdN<NodeIdentifier>>> for TStore {
    //     fn intern(t: Type) -> Self::Ty {
    //         t.into()
    //     }

    //     fn resolve(t: Self::Ty) -> Type {
    //         t.e()
    //     }
    // }

    impl<'a> TsQueryEnabledTypeStore<HashedNodeRef<'a, NodeIdentifier>> for TStore {
        fn resolve(t: Self::Ty) -> Type {
            t.e()
        }
    }
}

#[cfg(feature = "impl")]
fn id_for_node_kind(kind: &str, named: bool) -> u16 {
    tree_sitter_query::language().id_for_node_kind(kind, named)
}
#[cfg(not(feature = "impl"))]
fn id_for_node_kind(kind: &str, named: bool) -> u16 {
    unimplemented!("need treesitter grammar")
}

pub trait TsQueryEnabledTypeStore<T>:
    hyperast::types::ETypeStore<Ty2 = Type> + Clone + TsEnableTS
{
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

impl<IdN: Clone + Eq + AAAA> NodeId for TIdN<IdN> {
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

pub struct _TStore;

impl hyperast::store::TyDown<_TStore> for TStore {}

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
pub type TsQuery = Lang;

impl LangRef<AnyType> for TsQuery {
    fn make(&self, t: u16) -> &'static AnyType {
        panic!("{}", t)
        // &From::<&'static dyn HyperType>::from(&S_T_L[t as usize])
    }
    fn to_u16(&self, t: AnyType) -> u16 {
        // t as u16
        let t = t.as_any().downcast_ref::<Type>().unwrap();
        *t as u16
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<TsQuery>()
    }

    fn ts_symbol(&self, t: AnyType) -> u16 {
        id_for_node_kind(t.as_static_str(), t.is_named())
    }
}

impl LangRef<Type> for TsQuery {
    fn make(&self, t: u16) -> &'static Type {
        &S_T_L[t as usize]
    }
    fn to_u16(&self, t: Type) -> u16 {
        t as u16
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<TsQuery>()
    }

    fn ts_symbol(&self, t: Type) -> u16 {
        id_for_node_kind(t.as_static_str(), t.is_named())
    }
}

type TType = hyperast::types::TypeU16<Lang>;

impl LangRef<TType> for TsQuery {
    fn make(&self, t: u16) -> &'static TType {
        // TODO could make one safe, but not priority
        unsafe { std::mem::transmute(&S_T_L[t as usize]) }
    }
    fn to_u16(&self, t: TType) -> u16 {
        t.e() as u16
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<TsQuery>()
    }

    fn ts_symbol(&self, t: TType) -> u16 {
        id_for_node_kind(t.as_static_str(), t.is_named())
    }
}

impl hyperast::types::Lang<Type> for TsQuery {
    fn make(t: u16) -> &'static Type {
        Lang.make(t)
    }
    fn to_u16(t: Type) -> u16 {
        Lang.to_u16(t)
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
        todo!()
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
        todo!()
    }

    fn as_shared(&self) -> hyperast::types::Shared {
        use hyperast::types::Shared;

        match self {
            Type::Comment => Shared::Comment,
            Type::Identifier => Shared::Identifier,
            _ => Shared::Other,
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_static(&self) -> &'static dyn HyperType {
        let t = <TsQuery as hyperast::types::Lang<Type>>::to_u16(*self);
        let t = <TsQuery as hyperast::types::Lang<Type>>::make(t);
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
        From::<&'static (dyn LangRef<Self>)>::from(&Lang)
    }
    fn lang_ref(&self) -> hyperast::types::LangWrapper<AnyType> {
        hyperast::types::LangWrapper::from(&Lang as &(dyn LangRef<AnyType> + 'static))
    }
}

impl TypeTrait for Type {
    type Lang = TsQuery;

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

// 356 + directory  + spaces
const COUNT: u16 = 46 + 1 + 1;

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

impl TryFrom<&str> for Type {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Type::from_str(value).ok_or_else(|| value.to_owned())
    }
}

impl hyperast::types::LLang<hyperast::types::TypeU16<Self>> for TsQuery {
    type I = u16;

    type E = Type;

    const TE: &[Self::E] = S_T_L;

    fn as_lang_wrapper() -> hyperast::types::LangWrapper<hyperast::types::TypeU16<Self>> {
        From::<&'static (dyn LangRef<_>)>::from(&Lang)
    }
}

impl From<u16> for Type {
    fn from(value: u16) -> Self {
        debug_assert_eq!(Self::from_u16(value), S_T_L[value as usize]);
        S_T_L[value as usize]
    }
}
impl Into<TypeU16<TsQuery>> for Type {
    fn into(self) -> TypeU16<TsQuery> {
        TypeU16::new(self)
    }
}

impl Into<u16> for Type {
    fn into(self) -> u16 {
        self as u8 as u16
    }
}

#[repr(u16)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Type {
    End,
    Dot,
    DQuote,
    _StringToken1,
    EscapeSequence,
    Star,
    Plus,
    QMark,
    Identifier,
    Inderscore,
    At,
    Comment,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Slash,
    Colon,
    Bang,
    Sharp,
    PredicateType,
    Program,
    _Definition,
    _GroupExpression,
    _NamedNodeExpression,
    _String,
    Quantifier,
    _ImmediateIdentifier,
    _NodeIdentifier,
    Capture,
    String,
    Parameters,
    List,
    Grouping,
    AnonymousNode,
    NamedNode,
    _FieldName,
    FieldDefinition,
    NegatedField,
    Predicate,
    ProgramRepeat1,
    _StringRepeat1,
    ParametersRepeat1,
    ListRepeat1,
    GroupingRepeat1,
    NamedNodeRepeat1,
    Spaces,
    Directory,
    ERROR,
}
impl Type {
    pub fn from_u16(t: u16) -> Type {
        match t {
            0u16 => Type::End,
            1u16 => Type::Dot,
            2u16 => Type::DQuote,
            3u16 => Type::_StringToken1,
            4u16 => Type::EscapeSequence,
            5u16 => Type::Star,
            6u16 => Type::Plus,
            7u16 => Type::QMark,
            8u16 => Type::Identifier,
            9u16 => Type::Identifier,
            10u16 => Type::Inderscore,
            11u16 => Type::At,
            12u16 => Type::Comment,
            13u16 => Type::LBracket,
            14u16 => Type::RBracket,
            15u16 => Type::LParen,
            16u16 => Type::RParen,
            17u16 => Type::Slash,
            18u16 => Type::Colon,
            19u16 => Type::Bang,
            20u16 => Type::Sharp,
            21u16 => Type::PredicateType,
            22u16 => Type::Program,
            23u16 => Type::_Definition,
            24u16 => Type::_GroupExpression,
            25u16 => Type::_NamedNodeExpression,
            26u16 => Type::_String,
            27u16 => Type::Quantifier,
            28u16 => Type::_ImmediateIdentifier,
            29u16 => Type::_NodeIdentifier,
            30u16 => Type::Capture,
            31u16 => Type::String,
            32u16 => Type::Parameters,
            33u16 => Type::List,
            34u16 => Type::Grouping,
            35u16 => Type::AnonymousNode,
            36u16 => Type::NamedNode,
            37u16 => Type::_FieldName,
            38u16 => Type::FieldDefinition,
            39u16 => Type::NegatedField,
            40u16 => Type::Predicate,
            41u16 => Type::ProgramRepeat1,
            42u16 => Type::_StringRepeat1,
            43u16 => Type::ParametersRepeat1,
            44u16 => Type::ListRepeat1,
            45u16 => Type::GroupingRepeat1,
            46u16 => Type::NamedNodeRepeat1,
            u16::MAX => Type::ERROR,
            x => panic!("{}", x),
        }
    }
    pub fn from_str(t: &str) -> Option<Type> {
        Some(match t {
            "end" => Type::End,
            "." => Type::Dot,
            "\"" => Type::DQuote,
            "_string_token1" => Type::_StringToken1,
            "escape_sequence" => Type::EscapeSequence,
            "*" => Type::Star,
            "+" => Type::Plus,
            "?" => Type::QMark,
            "identifier" => Type::Identifier,
            "_" => Type::Inderscore,
            "@" => Type::At,
            "comment" => Type::Comment,
            "[" => Type::LBracket,
            "]" => Type::RBracket,
            "(" => Type::LParen,
            ")" => Type::RParen,
            "/" => Type::Slash,
            ":" => Type::Colon,
            "!" => Type::Bang,
            "#" => Type::Sharp,
            "predicate_type" => Type::PredicateType,
            "program" => Type::Program,
            "_definition" => Type::_Definition,
            "_group_expression" => Type::_GroupExpression,
            "_named_node_expression" => Type::_NamedNodeExpression,
            "_string" => Type::_String,
            "quantifier" => Type::Quantifier,
            "_immediate_identifier" => Type::_ImmediateIdentifier,
            "_node_identifier" => Type::_NodeIdentifier,
            "capture" => Type::Capture,
            "string" => Type::String,
            "parameters" => Type::Parameters,
            "list" => Type::List,
            "grouping" => Type::Grouping,
            "anonymous_node" => Type::AnonymousNode,
            "named_node" => Type::NamedNode,
            "_field_name" => Type::_FieldName,
            "field_definition" => Type::FieldDefinition,
            "negated_field" => Type::NegatedField,
            "predicate" => Type::Predicate,
            "program_repeat1" => Type::ProgramRepeat1,
            "_string_repeat1" => Type::_StringRepeat1,
            "parameters_repeat1" => Type::ParametersRepeat1,
            "list_repeat1" => Type::ListRepeat1,
            "grouping_repeat1" => Type::GroupingRepeat1,
            "named_node_repeat1" => Type::NamedNodeRepeat1,
            "Spaces" => Type::Spaces,
            "Directory" => Type::Directory,
            "ERROR" => Type::ERROR,
            _ => return None,
        })
    }
    pub fn to_str(&self) -> &'static str {
        match self {
            Type::End => "end",
            Type::Dot => ".",
            Type::DQuote => "\"",
            Type::_StringToken1 => "_string_token1",
            Type::EscapeSequence => "escape_sequence",
            Type::Star => "*",
            Type::Plus => "+",
            Type::QMark => "?",
            Type::Identifier => "identifier",
            Type::Inderscore => "_",
            Type::At => "@",
            Type::Comment => "comment",
            Type::LBracket => "[",
            Type::RBracket => "]",
            Type::LParen => "(",
            Type::RParen => ")",
            Type::Slash => "/",
            Type::Colon => ":",
            Type::Bang => "!",
            Type::Sharp => "#",
            Type::PredicateType => "predicate_type",
            Type::Program => "program",
            Type::_Definition => "_definition",
            Type::_GroupExpression => "_group_expression",
            Type::_NamedNodeExpression => "_named_node_expression",
            Type::_String => "_string",
            Type::Quantifier => "quantifier",
            Type::_ImmediateIdentifier => "_immediate_identifier",
            Type::_NodeIdentifier => "_node_identifier",
            Type::Capture => "capture",
            Type::String => "string",
            Type::Parameters => "parameters",
            Type::List => "list",
            Type::Grouping => "grouping",
            Type::AnonymousNode => "anonymous_node",
            Type::NamedNode => "named_node",
            Type::_FieldName => "_field_name",
            Type::FieldDefinition => "field_definition",
            Type::NegatedField => "negated_field",
            Type::Predicate => "predicate",
            Type::ProgramRepeat1 => "program_repeat1",
            Type::_StringRepeat1 => "_string_repeat1",
            Type::ParametersRepeat1 => "parameters_repeat1",
            Type::ListRepeat1 => "list_repeat1",
            Type::GroupingRepeat1 => "grouping_repeat1",
            Type::NamedNodeRepeat1 => "named_node_repeat1",
            Type::Spaces => "Spaces",
            Type::Directory => "Directory",
            Type::ERROR => "ERROR",
        }
    }

    pub fn is_hidden(&self) -> bool {
        match self {
            Type::End => true,
            Type::_StringToken1 => true,
            Type::_Definition => true,
            Type::_GroupExpression => true,
            Type::_NamedNodeExpression => true,
            Type::_String => true,
            Type::_ImmediateIdentifier => true,
            Type::_NodeIdentifier => true,
            Type::_FieldName => true,
            Type::ProgramRepeat1 => true,
            Type::_StringRepeat1 => true,
            Type::ParametersRepeat1 => true,
            Type::ListRepeat1 => true,
            Type::GroupingRepeat1 => true,
            Type::NamedNodeRepeat1 => true,
            _ => false,
        }
    }
    pub fn is_supertype(&self) -> bool {
        match self {
            _ => false,
        }
    }
    pub fn is_repeat(&self) -> bool {
        todo!("generate this with polyglote")
    }
    pub fn is_named(&self) -> bool {
        match self {
            Type::EscapeSequence => true,
            Type::Identifier => true,
            Type::Comment => true,
            Type::PredicateType => true,
            Type::Program => true,
            Type::Quantifier => true,
            Type::Capture => true,
            Type::String => true,
            Type::Parameters => true,
            Type::List => true,
            Type::Grouping => true,
            Type::AnonymousNode => true,
            Type::NamedNode => true,
            Type::FieldDefinition => true,
            Type::NegatedField => true,
            Type::Predicate => true,
            _ => false,
        }
    }
}

const S_T_L: &'static [Type] = &[
    Type::End,
    Type::Dot,
    Type::DQuote,
    Type::_StringToken1,
    Type::EscapeSequence,
    Type::Star,
    Type::Plus,
    Type::QMark,
    Type::Identifier,
    Type::Inderscore,
    Type::At,
    Type::Comment,
    Type::LBracket,
    Type::RBracket,
    Type::LParen,
    Type::RParen,
    Type::Slash,
    Type::Colon,
    Type::Bang,
    Type::Sharp,
    Type::PredicateType,
    Type::Program,
    Type::_Definition,
    Type::_GroupExpression,
    Type::_NamedNodeExpression,
    Type::_String,
    Type::Quantifier,
    Type::_ImmediateIdentifier,
    Type::_NodeIdentifier,
    Type::Capture,
    Type::String,
    Type::Parameters,
    Type::List,
    Type::Grouping,
    Type::AnonymousNode,
    Type::NamedNode,
    Type::_FieldName,
    Type::FieldDefinition,
    Type::NegatedField,
    Type::Predicate,
    Type::ProgramRepeat1,
    Type::_StringRepeat1,
    Type::ParametersRepeat1,
    Type::ListRepeat1,
    Type::GroupingRepeat1,
    Type::NamedNodeRepeat1,
    Type::Spaces,
    Type::Directory,
    Type::ERROR,
];
