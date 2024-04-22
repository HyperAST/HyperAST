use std::fmt::Display;

use hyper_ast::{
    store::defaults::NodeIdentifier,
    tree_gen::parser::NodeWithU16TypeId,
    types::{AnyType, HyperType, Lang, LangRef, NodeId, TypeStore, TypeTrait, TypedNodeId},
};

#[cfg(feature = "legion")]
mod legion_impls {
    use super::*;

    use crate::TNode;

    impl<'a> TNode<'a> {
        pub fn obtain_type<T>(&self, _: &mut impl TsQueryEnabledTypeStore<T>) -> Type {
            let t = self.kind_id();
            Type::from_u16(t)
        }
    }

    use hyper_ast::{store::nodes::legion::HashedNodeRef, types::TypeIndex};

    impl<'a> TypeStore<HashedNodeRef<'a, TIdN<NodeIdentifier>>> for TStore {
        type Ty = Type;
        const MASK: TypeInternalSize = 0b1000_0000_0000_0000;
        fn resolve_type(&self, n: &HashedNodeRef<'a, TIdN<NodeIdentifier>>) -> Self::Ty {
            n.get_component::<Type>().unwrap().clone()
        }

        fn resolve_lang(
            &self,
            n: &HashedNodeRef<'a, TIdN<NodeIdentifier>>,
        ) -> hyper_ast::types::LangWrapper<Self::Ty> {
            From::<&'static (dyn LangRef<Type>)>::from(&TsQuery)
        }

        type Marshaled = TypeIndex;

        fn marshal_type(&self, n: &HashedNodeRef<'a, TIdN<NodeIdentifier>>) -> Self::Marshaled {
            TypeIndex {
                lang: LangRef::<Type>::name(&TsQuery),
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
    impl<'a> TsQueryEnabledTypeStore<HashedNodeRef<'a, TIdN<NodeIdentifier>>> for TStore {
        const LANG: TypeInternalSize = Self::Ts as u16;

        fn _intern(l: u16, t: u16) -> Self::Ty {
            // T((u16::MAX - l as u16) | t)
            todo!()
        }
        fn intern(&self, t: Type) -> Self::Ty {
            t
        }

        fn resolve(&self, t: Self::Ty) -> Type {
            t
            // let t = t.0 as u16;
            // let t = t & !TStore::MASK;
            // Type::resolve(t)
        }
    }
    impl<'a> TypeStore<HashedNodeRef<'a, NodeIdentifier>> for TStore {
        type Ty = AnyType;
        const MASK: TypeInternalSize = 0b1000_0000_0000_0000;
        fn resolve_type(&self, n: &HashedNodeRef<'a, NodeIdentifier>) -> Self::Ty {
            From::<&'static (dyn HyperType)>::from(LangRef::<Type>::make(
                &TsQuery,
                *n.get_component::<Type>().unwrap() as u16,
            ))
        }

        fn resolve_lang(
            &self,
            n: &HashedNodeRef<'a, NodeIdentifier>,
        ) -> hyper_ast::types::LangWrapper<Self::Ty> {
            From::<&'static (dyn LangRef<AnyType>)>::from(&TsQuery)
        }

        type Marshaled = TypeIndex;

        fn marshal_type(&self, n: &HashedNodeRef<'a, NodeIdentifier>) -> Self::Marshaled {
            TypeIndex {
                lang: LangRef::<Type>::name(&TsQuery),
                ty: *n.get_component::<Type>().unwrap() as u16,
            }
        }
        fn type_eq(
            &self,
            n: &HashedNodeRef<'a, NodeIdentifier>,
            m: &HashedNodeRef<'a, NodeIdentifier>,
        ) -> bool {
            todo!()
        }
    }
}

pub trait TsQueryEnabledTypeStore<T>: TypeStore<T> {
    const LANG: u16;
    fn intern(&self, t: Type) -> Self::Ty {
        let t = t as u16;
        Self::_intern(Self::LANG, t)
    }
    fn _intern(l: u16, t: u16) -> Self::Ty;
    fn resolve(&self, t: Self::Ty) -> Type;
}

impl Type {
    pub fn resolve(t: u16) -> Self {
        assert!(t < COUNT);
        unsafe { std::mem::transmute(t) }
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
        todo!()
    }
}

impl<IdN: Clone + Eq + NodeId> TypedNodeId for TIdN<IdN> {
    type Ty = Type;
}

#[repr(u8)]
pub enum TStore {
    Ts = 0,
}

impl Default for TStore {
    fn default() -> Self {
        Self::Ts
    }
}

type TypeInternalSize = u16;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct T(TypeInternalSize);

pub struct TsQuery;

impl LangRef<AnyType> for TsQuery {
    fn make(&self, t: u16) -> &'static AnyType {
        panic!()
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
}

impl Lang<Type> for TsQuery {
    fn make(t: u16) -> &'static Type {
        TsQuery.make(t)
    }
    fn to_u16(t: Type) -> u16 {
        TsQuery.to_u16(t)
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

    fn as_shared(&self) -> hyper_ast::types::Shared {
        use hyper_ast::types::Shared;

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
        let t = <TsQuery as Lang<Type>>::to_u16(*self);
        let t = <TsQuery as Lang<Type>>::make(t);
        t
    }

    fn get_lang(&self) -> hyper_ast::types::LangWrapper<Self>
    where
        Self: Sized,
    {
        From::<&'static (dyn LangRef<Self>)>::from(&TsQuery)
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

#[repr(u16)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Type {
    End,
    Dot,
    Quote,
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
            2u16 => Type::Quote,
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
            17u16 => Type::Colon,
            18u16 => Type::Bang,
            19u16 => Type::Sharp,
            20u16 => Type::PredicateType,
            21u16 => Type::Program,
            22u16 => Type::_Definition,
            23u16 => Type::_GroupExpression,
            24u16 => Type::_NamedNodeExpression,
            25u16 => Type::_String,
            26u16 => Type::Quantifier,
            27u16 => Type::_ImmediateIdentifier,
            28u16 => Type::_NodeIdentifier,
            29u16 => Type::Capture,
            30u16 => Type::String,
            31u16 => Type::Parameters,
            32u16 => Type::List,
            33u16 => Type::Grouping,
            34u16 => Type::AnonymousNode,
            35u16 => Type::NamedNode,
            36u16 => Type::_FieldName,
            37u16 => Type::FieldDefinition,
            38u16 => Type::NegatedField,
            39u16 => Type::Predicate,
            40u16 => Type::ProgramRepeat1,
            41u16 => Type::_StringRepeat1,
            42u16 => Type::ParametersRepeat1,
            43u16 => Type::ListRepeat1,
            44u16 => Type::GroupingRepeat1,
            45u16 => Type::NamedNodeRepeat1,
            46u16 => Type::ERROR,
            u16::MAX => Type::ERROR,
            x => panic!("{}", x),
        }
    }
    pub fn from_str(s: &str) -> Option<Type> {
        Some(match s {
            "end" => Type::End,
            "." => Type::Dot,
            "\"" => Type::Quote,
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
            x => return None,
        })
    }
    pub fn to_str(&self) -> &'static str {
        match self {
            Type::End => "end",
            Type::Dot => ".",
            Type::Quote => "\"",
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
}
// Type::End => "end",
// Type::Dot => ".",
// Type::TS0 => "\"",
// Type::TS1 => "_string_token1",
// Type::EscapeSequence => "escape_sequence",
// Type::Star => "*",
// Type::Plus => "+",
// Type::QMark => "?",
// Type::Identifier => "identifier",
// Type::TS2 => "_",
// Type::At => "@",
// Type::Comment => "comment",
// Type::LBracket => "[",
// Type::RBracket => "]",
// Type::LParen => "(",
// Type::RParen => ")",
// Type::Colon => ":",
// Type::Bang => "!",
// Type::TS3 => "#",
// Type::PredicateType => "predicate_type",
// Type::Program => "program",
// Type::TS4 => "_definition",
// Type::TS5 => "_group_expression",
// Type::TS6 => "_named_node_expression",
// Type::TS7 => "_string",
// Type::Quantifier => "quantifier",
// Type::TS8 => "_immediate_identifier",
// Type::TS9 => "_node_identifier",
// Type::Capture => "capture",
// Type::String => "string",
// Type::Parameters => "parameters",
// Type::List => "list",
// Type::Grouping => "grouping",
// Type::AnonymousNode => "anonymous_node",
// Type::NamedNode => "named_node",
// Type::TS10 => "_field_name",
// Type::FieldDefinition => "field_definition",
// Type::NegatedField => "negated_field",
// Type::Predicate => "predicate",
// Type::ProgramRepeat1 => "program_repeat1",
// Type::TS11 => "_string_repeat1",
// Type::ParametersRepeat1 => "parameters_repeat1",
// Type::ListRepeat1 => "list_repeat1",
// Type::GroupingRepeat1 => "grouping_repeat1",
// Type::NamedNodeRepeat1 => "named_node_repeat1",
// Type::Spaces => "Spaces",
// Type::Directory => "Directory",
// Type::ERROR => "ERROR",

const S_T_L: &'static [Type] = &[
    Type::End,
    Type::Dot,
    Type::Quote,
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
