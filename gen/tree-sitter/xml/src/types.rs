use std::{fmt::Display, u16};

use hyper_ast::{
    tree_gen::parser::NodeWithU16TypeId,
    types::{AnyType, HyperType, LangRef, NodeId, TypeStore, TypeTrait, TypeU16, TypedNodeId},
};

#[cfg(feature = "legion")]
mod legion_impls {
    use super::*;

    use crate::TNode;

    impl<'a> TNode<'a> {
        pub fn obtain_type(&self) -> Type {
            let t = self.kind_id();
            Type::from_u16(t)
        }
    }

    use hyper_ast::types::{LangWrapper, RoleStore};

    impl TypeStore for TStore {
        type Ty = TypeU16<Xml>;
    }
    impl TypeStore for &TStore {
        type Ty = TypeU16<Xml>;
    }

    impl XmlEnabledTypeStore for TStore {
        fn intern(t: Type) -> Self::Ty {
            t.into()
        }

        fn resolve(t: Self::Ty) -> Type {
            t.e()
        }
    }

    impl RoleStore for TStore {
        type IdF = u16;

        type Role = hyper_ast::types::Role;

        fn resolve_field(_lang: LangWrapper<Self::Ty>, field_id: Self::IdF) -> Self::Role {
            let s = tree_sitter_xml::language_xml()
                .field_name_for_id(field_id)
                .ok_or_else(|| format!("{}", field_id))
                .unwrap();
            hyper_ast::types::Role::try_from(s).expect(s)
        }

        fn intern_role(_lang: LangWrapper<Self::Ty>, role: Self::Role) -> Self::IdF {
            let field_name = role.to_string();
            tree_sitter_xml::language_xml()
                .field_id_for_name(field_name)
                .unwrap()
                .into()
        }
    }
}

#[cfg(feature = "impl")]
fn id_for_node_kind(kind: &str, named: bool) -> u16 {
    tree_sitter_xml::language_xml().id_for_node_kind(kind, named)
}
#[cfg(not(feature = "impl"))]
fn id_for_node_kind(kind: &str, named: bool) -> u16 {
    unimplemented!("need treesitter grammar")
}

pub fn as_any(t: &Type) -> AnyType {
    let t = <Xml as hyper_ast::types::Lang<Type>>::to_u16(*t);
    let t = <Xml as hyper_ast::types::Lang<Type>>::make(t);
    let t: &'static dyn HyperType = t;
    t.into()
}

pub trait XmlEnabledTypeStore: TypeStore {
    fn intern(t: Type) -> Self::Ty;
    fn resolve(t: Self::Ty) -> Type;
}

pub struct TStore;

impl Default for TStore {
    fn default() -> Self {
        Self
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

    unsafe fn from_ref_id(_id: &Self::IdN) -> &Self {
        todo!()
    }
}

impl<IdN: Clone + Eq + NodeId> TypedNodeId for TIdN<IdN> {
    type Ty = Type;
}

type TypeInternalSize = u16;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct T(TypeInternalSize);

#[derive(Debug)]
pub struct Lang;

pub type Xml = Lang;

impl hyper_ast::types::Lang<Type> for Xml {
    fn make(t: u16) -> &'static Type {
        Lang.make(t)
    }
    fn to_u16(t: Type) -> u16 {
        Lang.to_u16(t)
    }
}

impl LangRef<Type> for Xml {
    fn name(&self) -> &'static str {
        std::any::type_name::<Xml>()
    }

    fn make(&self, t: u16) -> &'static Type {
        &S_T_L[t as usize]
    }

    fn to_u16(&self, t: Type) -> u16 {
        t as u16
    }

    fn ts_symbol(&self, t: Type) -> u16 {
        id_for_node_kind(t.as_static_str(), t.is_named())
    }
}

impl LangRef<AnyType> for Xml {
    fn name(&self) -> &'static str {
        std::any::type_name::<Xml>()
    }

    fn make(&self, _t: u16) -> &'static AnyType {
        todo!()
    }

    fn to_u16(&self, t: AnyType) -> u16 {
        let t: &Type = t.as_any().downcast_ref().unwrap();
        Lang.to_u16(*t)
    }

    fn ts_symbol(&self, t: AnyType) -> u16 {
        id_for_node_kind(t.as_static_str(), t.is_named())
    }
}

impl LangRef<hyper_ast::types::TypeU16<Self>> for Lang {
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

    fn as_shared(&self) -> hyper_ast::types::Shared {
        use hyper_ast::types::Shared;
        match self {
            _ => Shared::Other,
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_static(&self) -> &'static dyn HyperType {
        let t = <Xml as hyper_ast::types::Lang<Type>>::to_u16(*self);
        let t = <Xml as hyper_ast::types::Lang<Type>>::make(t);
        t
    }

    fn as_static_str(&self) -> &'static str {
        self.to_str()
    }

    fn is_file(&self) -> bool {
        self == &Type::Document
    }

    fn is_directory(&self) -> bool {
        self == &Type::Directory || self == &Type::MavenDirectory
    }

    fn is_spaces(&self) -> bool {
        self == &Type::Spaces
    }

    fn is_syntax(&self) -> bool {
        self == &Type::TS2 // " ",
        // || self == &Type::Nmtoken // "Nmtoken",
        || self == &Type::TS3 // "\"",
        || self == &Type::TS4 // "'",
        // || self == &Type::TS5 // "Sep1_token1",
        // || self == &Type::TS6 // "Sep2_token1",
        // || self == &Type::TS7 // "Sep3_token1",
        // || self == &Type::SystemLiteral // "SystemLiteral",
        // || self == &Type::PubidLiteral // "PubidLiteral",
        // || self == &Type::CharData // "CharData",
        // || self == &Type::Comment // "Comment",
        || self == &Type::TS8 // "<?",
        || self == &Type::TS9 // "?>",
        || self == &Type::CdSect // "CDSect",
        || self == &Type::TS10 // "<?xml",
        // || self == &Type::Version // "version",
        || self == &Type::Eq // "=",
        // || self == &Type::VersionNum // "VersionNum",
        || self == &Type::TS11 // "<!DOCTYPE",
        || self == &Type::LBracket // "[",
        || self == &Type::RBracket // "]",
        || self == &Type::GT // ">",
        // || self == &Type::Standalone // "standalone",
        // || self == &Type::Yes // "yes",
        // || self == &Type::No // "no",
        || self == &Type::LT // "<",
        || self == &Type::TS12 // "</",
        || self == &Type::TS13 // "/>",
        || self == &Type::TS14 // "<!ELEMENT",
        // || self == &Type::TS15 // "EMPTY",
        // || self == &Type::TS16 // "ANY",
        || self == &Type::QMark // "?",
        || self == &Type::Star // "*",
        || self == &Type::Plus // "+",
        || self == &Type::LParen // "(",
        || self == &Type::Pipe // "|",
        || self == &Type::RParen // ")",
        || self == &Type::Comma // ",",
        // || self == &Type::TS17 // "#PCDATA",
        || self == &Type::TS18 // ")*",
        || self == &Type::TS19 // "<!ATTLIST",
        || self == &Type::StringType // "StringType",
        // || self == &Type::TS20 // "ID",
        // || self == &Type::TS21 // "IDREF",
        // || self == &Type::TS22 // "IDREFS",
        // || self == &Type::TS23 // "ENTITY",
        // || self == &Type::TS24 // "ENTITIES",
        // || self == &Type::TS25 // "NMTOKEN",
        // || self == &Type::TS26 // "NMTOKENS",
        // || self == &Type::TS27 // "NOTATION",
        // || self == &Type::TS28 // "#REQUIRED",
        // || self == &Type::TS29 // "#IMPLIED",
        // || self == &Type::TS30 // "#FIXED",
        // || self == &Type::CharRef // "CharRef",
        || self == &Type::Amp // "&",
        || self == &Type::SemiColon // ";",
        || self == &Type::Percent // "%",
    }

    fn is_hidden(&self) -> bool {
        false // TODO
    }

    fn is_supertype(&self) -> bool {
        false // TODO
    }

    fn is_named(&self) -> bool {
        false // TODO
    }

    fn get_lang(&self) -> hyper_ast::types::LangWrapper<Self>
    where
        Self: Sized,
    {
        hyper_ast::types::LangWrapper::from(&Lang as &(dyn LangRef<Self> + 'static))
    }

    fn lang_ref(&self) -> hyper_ast::types::LangWrapper<AnyType> {
        hyper_ast::types::LangWrapper::from(&Lang as &(dyn LangRef<AnyType> + 'static))
    }
}

impl TypeTrait for Type {
    type Lang = Xml;

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

impl Type {
    pub fn resolve(t: u16) -> Self {
        assert!(t < COUNT);
        unsafe { std::mem::transmute(t) }
    }
}
const COUNT: u16 = 136 + 1 + 3;

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

impl TryFrom<&str> for Type {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, <Self as TryFrom<&str>>::Error> {
        Type::from_str(value).ok_or_else(|| value.to_owned())
    }
}

impl hyper_ast::types::LLang<hyper_ast::types::TypeU16<Self>> for Xml {
    type I = u16;

    type E = Type;

    const TE: &[Self::E] = S_T_L;

    fn as_lang_wrapper() -> hyper_ast::types::LangWrapper<hyper_ast::types::TypeU16<Self>> {
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
impl Into<TypeU16<Xml>> for Type {
    fn into(self) -> TypeU16<Xml> {
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
    Name,
    TS0,
    TS1,
    TS2,
    LBracket,
    TS3,
    TS4,
    TS5,
    GT,
    TS6,
    TS7,
    LParen,
    TS8,
    Pipe,
    RParen,
    Star,
    QMark,
    Plus,
    Comma,
    TS9,
    TS10,
    TokenizedType,
    TS11,
    TS12,
    TS13,
    TS14,
    TS15,
    Percent,
    DQuote,
    TS16,
    SQuote,
    TS17,
    TS18,
    SemiColon,
    TS19,
    Nmtoken,
    Amp,
    TS20,
    TS21,
    TS22,
    TS23,
    TS24,
    TS25,
    TS26,
    TS27,
    Uri,
    TS28,
    TS29,
    TS30,
    Xml,
    TS31,
    Version,
    VersionNum,
    Encoding,
    EncName,
    Eq,
    Standalone,
    Yes,
    No,
    TS32,
    RBracket,
    LT,
    TS33,
    TS34,
    TS35,
    TS36,
    PiTarget,
    _PiContent,
    Comment,
    CharData,
    CData,
    Error,
    Document,
    _Markupdecl,
    TS38,
    Elementdecl,
    Contentspec,
    Mixed,
    Children,
    _Cp,
    _Choice,
    AttlistDecl,
    AttDef,
    AttType,
    StringType,
    EnumeratedType,
    NotationType,
    Enumeration,
    DefaultDecl,
    EntityDecl,
    GeDecl,
    PeDecl,
    EntityValue,
    NDataDecl,
    NotationDecl,
    PeReference,
    Reference,
    EntityRef,
    CharRef,
    AttValue,
    ExternalId,
    PublicId,
    SystemLiteral,
    PubidLiteral,
    XmlDecl,
    TS41,
    TS42,
    Pi,
    TS43,
    Prolog,
    TS44,
    TS45,
    Doctypedecl,
    TS46,
    Element,
    EmptyElemTag,
    Attribute,
    STag,
    ETag,
    Content,
    CdSect,
    CdStart,
    StyleSheetPi,
    XmlModelPi,
    PseudoAtt,
    PseudoAttValue,
    DocumentRepeat1,
    TS48,
    TS49,
    _ChoiceRepeat1,
    _ChoiceRepeat2,
    TS52,
    TS53,
    TS54,
    TS55,
    TS56,
    TS57,
    TS58,
    TS59,
    ContentRepeat1,
    TS61,
    Spaces,
    MavenDirectory, // NOTE maven specific
    Directory,
    ERROR,
}
impl Type {
    pub fn from_u16(t: u16) -> Type {
        match t {
            0u16 => Type::End,
            1u16 => Type::Name,
            2u16 => Type::TS0,
            3u16 => Type::TS1,
            4u16 => Type::TS2,
            5u16 => Type::LBracket,
            6u16 => Type::TS3,
            7u16 => Type::TS4,
            8u16 => Type::TS5,
            9u16 => Type::GT,
            10u16 => Type::TS6,
            11u16 => Type::TS7,
            12u16 => Type::LParen,
            13u16 => Type::TS8,
            14u16 => Type::Pipe,
            15u16 => Type::RParen,
            16u16 => Type::Star,
            17u16 => Type::QMark,
            18u16 => Type::Plus,
            19u16 => Type::Comma,
            20u16 => Type::TS9,
            21u16 => Type::TS10,
            22u16 => Type::TokenizedType,
            23u16 => Type::TS11,
            24u16 => Type::TS12,
            25u16 => Type::TS13,
            26u16 => Type::TS14,
            27u16 => Type::TS15,
            28u16 => Type::Percent,
            29u16 => Type::DQuote,
            30u16 => Type::TS16,
            31u16 => Type::SQuote,
            32u16 => Type::TS17,
            33u16 => Type::TS18,
            34u16 => Type::SemiColon,
            35u16 => Type::TS19,
            36u16 => Type::Nmtoken,
            37u16 => Type::Amp,
            38u16 => Type::TS20,
            39u16 => Type::TS21,
            40u16 => Type::TS22,
            41u16 => Type::TS23,
            42u16 => Type::TS24,
            43u16 => Type::TS25,
            44u16 => Type::TS26,
            45u16 => Type::TS27,
            46u16 => Type::Uri,
            47u16 => Type::Uri,
            48u16 => Type::TS28,
            49u16 => Type::TS29,
            50u16 => Type::TS30,
            51u16 => Type::Xml,
            52u16 => Type::TS31,
            53u16 => Type::Version,
            54u16 => Type::VersionNum,
            55u16 => Type::Encoding,
            56u16 => Type::EncName,
            57u16 => Type::Eq,
            58u16 => Type::Standalone,
            59u16 => Type::Yes,
            60u16 => Type::No,
            61u16 => Type::TS32,
            62u16 => Type::RBracket,
            63u16 => Type::LT,
            64u16 => Type::TS33,
            65u16 => Type::TS34,
            66u16 => Type::TS35,
            67u16 => Type::TS36,
            68u16 => Type::PiTarget,
            69u16 => Type::_PiContent,
            70u16 => Type::Comment,
            71u16 => Type::CharData,
            72u16 => Type::CData,
            73u16 => Type::Name,
            74u16 => Type::Name,
            75u16 => Type::Error,
            76u16 => Type::Document,
            77u16 => Type::_Markupdecl,
            78u16 => Type::TS38,
            79u16 => Type::Elementdecl,
            80u16 => Type::Contentspec,
            81u16 => Type::Mixed,
            82u16 => Type::Children,
            83u16 => Type::_Cp,
            84u16 => Type::_Choice,
            85u16 => Type::AttlistDecl,
            86u16 => Type::AttDef,
            87u16 => Type::AttType,
            88u16 => Type::StringType,
            89u16 => Type::EnumeratedType,
            90u16 => Type::NotationType,
            91u16 => Type::Enumeration,
            92u16 => Type::DefaultDecl,
            93u16 => Type::EntityDecl,
            94u16 => Type::GeDecl,
            95u16 => Type::PeDecl,
            96u16 => Type::EntityValue,
            97u16 => Type::NDataDecl,
            98u16 => Type::NotationDecl,
            99u16 => Type::PeReference,
            100u16 => Type::Reference,
            101u16 => Type::EntityRef,
            102u16 => Type::CharRef,
            103u16 => Type::AttValue,
            104u16 => Type::ExternalId,
            105u16 => Type::PublicId,
            106u16 => Type::SystemLiteral,
            107u16 => Type::PubidLiteral,
            108u16 => Type::XmlDecl,
            109u16 => Type::TS41,
            110u16 => Type::TS42,
            111u16 => Type::Pi,
            112u16 => Type::TS43,
            113u16 => Type::Prolog,
            114u16 => Type::TS44,
            115u16 => Type::TS45,
            116u16 => Type::Doctypedecl,
            117u16 => Type::TS46,
            118u16 => Type::Element,
            119u16 => Type::EmptyElemTag,
            120u16 => Type::Attribute,
            121u16 => Type::STag,
            122u16 => Type::ETag,
            123u16 => Type::Content,
            124u16 => Type::CdSect,
            125u16 => Type::CdStart,
            126u16 => Type::StyleSheetPi,
            127u16 => Type::XmlModelPi,
            128u16 => Type::PseudoAtt,
            129u16 => Type::PseudoAttValue,
            130u16 => Type::DocumentRepeat1,
            131u16 => Type::TS48,
            132u16 => Type::TS49,
            133u16 => Type::_ChoiceRepeat1,
            134u16 => Type::_ChoiceRepeat2,
            135u16 => Type::TS52,
            136u16 => Type::TS53,
            137u16 => Type::TS54,
            138u16 => Type::TS55,
            139u16 => Type::TS56,
            140u16 => Type::TS57,
            141u16 => Type::TS58,
            142u16 => Type::TS59,
            143u16 => Type::ContentRepeat1,
            144u16 => Type::TS61,
            145u16 => Type::ERROR,
            u16::MAX => Type::ERROR,
            x => panic!("{}", x),
        }
    }
    pub fn from_str(t: &str) -> Option<Type> {
        Some(match t {
            "end" => Type::End,
            "Name" => Type::Name,
            "<![" => Type::TS0,
            "IGNORE" => Type::TS1,
            "INCLUDE" => Type::TS2,
            "[" => Type::LBracket,
            "]]>" => Type::TS3,
            "<!" => Type::TS4,
            "ELEMENT" => Type::TS5,
            ">" => Type::GT,
            "EMPTY" => Type::TS6,
            "ANY" => Type::TS7,
            "(" => Type::LParen,
            "#PCDATA" => Type::TS8,
            "|" => Type::Pipe,
            ")" => Type::RParen,
            "*" => Type::Star,
            "?" => Type::QMark,
            "+" => Type::Plus,
            "," => Type::Comma,
            "ATTLIST" => Type::TS9,
            "CDATA" => Type::TS10,
            "TokenizedType" => Type::TokenizedType,
            "NOTATION" => Type::TS11,
            "#REQUIRED" => Type::TS12,
            "#IMPLIED" => Type::TS13,
            "#FIXED" => Type::TS14,
            "ENTITY" => Type::TS15,
            "%" => Type::Percent,
            "\"" => Type::DQuote,
            "EntityValue_token1" => Type::TS16,
            "'" => Type::SQuote,
            "EntityValue_token2" => Type::TS17,
            "NDATA" => Type::TS18,
            ";" => Type::SemiColon,
            "_S" => Type::TS19,
            "Nmtoken" => Type::Nmtoken,
            "&" => Type::Amp,
            "&#" => Type::TS20,
            "CharRef_token1" => Type::TS21,
            "&#x" => Type::TS22,
            "CharRef_token2" => Type::TS23,
            "AttValue_token1" => Type::TS24,
            "AttValue_token2" => Type::TS25,
            "SYSTEM" => Type::TS26,
            "PUBLIC" => Type::TS27,
            "URI" => Type::Uri,
            "PubidLiteral_token1" => Type::TS28,
            "PubidLiteral_token2" => Type::TS29,
            "<?" => Type::TS30,
            "xml" => Type::Xml,
            "?>" => Type::TS31,
            "version" => Type::Version,
            "VersionNum" => Type::VersionNum,
            "encoding" => Type::Encoding,
            "EncName" => Type::EncName,
            "=" => Type::Eq,
            "standalone" => Type::Standalone,
            "yes" => Type::Yes,
            "no" => Type::No,
            "DOCTYPE" => Type::TS32,
            "]" => Type::RBracket,
            "<" => Type::LT,
            "/>" => Type::TS33,
            "</" => Type::TS34,
            "xml-stylesheet" => Type::TS35,
            "xml-model" => Type::TS36,
            "PITarget" => Type::PiTarget,
            "_pi_content" => Type::_PiContent,
            "Comment" => Type::Comment,
            "CharData" => Type::CharData,
            "CData" => Type::CData,
            "ERROR" => Type::Error,
            "document" => Type::Document,
            "_markupdecl" => Type::_Markupdecl,
            "_DeclSep" => Type::TS38,
            "elementdecl" => Type::Elementdecl,
            "contentspec" => Type::Contentspec,
            "Mixed" => Type::Mixed,
            "children" => Type::Children,
            "_cp" => Type::_Cp,
            "_choice" => Type::_Choice,
            "AttlistDecl" => Type::AttlistDecl,
            "AttDef" => Type::AttDef,
            "_AttType" => Type::AttType,
            "StringType" => Type::StringType,
            "_EnumeratedType" => Type::EnumeratedType,
            "NotationType" => Type::NotationType,
            "Enumeration" => Type::Enumeration,
            "DefaultDecl" => Type::DefaultDecl,
            "_EntityDecl" => Type::EntityDecl,
            "GEDecl" => Type::GeDecl,
            "PEDecl" => Type::PeDecl,
            "EntityValue" => Type::EntityValue,
            "NDataDecl" => Type::NDataDecl,
            "NotationDecl" => Type::NotationDecl,
            "PEReference" => Type::PeReference,
            "_Reference" => Type::Reference,
            "EntityRef" => Type::EntityRef,
            "CharRef" => Type::CharRef,
            "AttValue" => Type::AttValue,
            "ExternalID" => Type::ExternalId,
            "PublicID" => Type::PublicId,
            "SystemLiteral" => Type::SystemLiteral,
            "PubidLiteral" => Type::PubidLiteral,
            "XMLDecl" => Type::XmlDecl,
            "_VersionInfo" => Type::TS41,
            "_EncodingDecl" => Type::TS42,
            "PI" => Type::Pi,
            "_Eq" => Type::TS43,
            "prolog" => Type::Prolog,
            "_Misc" => Type::TS44,
            "_SDDecl" => Type::TS45,
            "doctypedecl" => Type::Doctypedecl,
            "_intSubset" => Type::TS46,
            "element" => Type::Element,
            "EmptyElemTag" => Type::EmptyElemTag,
            "Attribute" => Type::Attribute,
            "STag" => Type::STag,
            "ETag" => Type::ETag,
            "content" => Type::Content,
            "CDSect" => Type::CdSect,
            "CDStart" => Type::CdStart,
            "StyleSheetPI" => Type::StyleSheetPi,
            "XmlModelPI" => Type::XmlModelPi,
            "PseudoAtt" => Type::PseudoAtt,
            "PseudoAttValue" => Type::PseudoAttValue,
            "document_repeat1" => Type::DocumentRepeat1,
            "Mixed_repeat1" => Type::TS48,
            "Mixed_repeat2" => Type::TS49,
            "_choice_repeat1" => Type::_ChoiceRepeat1,
            "_choice_repeat2" => Type::_ChoiceRepeat2,
            "AttlistDecl_repeat1" => Type::TS52,
            "NotationType_repeat1" => Type::TS53,
            "Enumeration_repeat1" => Type::TS54,
            "EntityValue_repeat1" => Type::TS55,
            "EntityValue_repeat2" => Type::TS56,
            "AttValue_repeat1" => Type::TS57,
            "AttValue_repeat2" => Type::TS58,
            "EmptyElemTag_repeat1" => Type::TS59,
            "content_repeat1" => Type::ContentRepeat1,
            "StyleSheetPI_repeat1" => Type::TS61,
            "Spaces" => Type::Spaces,
            "Directory" => Type::Directory,
            "MavenDirectory" => Type::MavenDirectory,
            "ERROR" => Type::ERROR,
            _x => return None,
        })
    }
    pub fn to_str(&self) -> &'static str {
        match self {
            Type::End => "end",
            Type::Name => "Name",
            Type::TS0 => "<![",
            Type::TS1 => "IGNORE",
            Type::TS2 => "INCLUDE",
            Type::LBracket => "[",
            Type::TS3 => "]]>",
            Type::TS4 => "<!",
            Type::TS5 => "ELEMENT",
            Type::GT => ">",
            Type::TS6 => "EMPTY",
            Type::TS7 => "ANY",
            Type::LParen => "(",
            Type::TS8 => "#PCDATA",
            Type::Pipe => "|",
            Type::RParen => ")",
            Type::Star => "*",
            Type::QMark => "?",
            Type::Plus => "+",
            Type::Comma => ",",
            Type::TS9 => "ATTLIST",
            Type::TS10 => "CDATA",
            Type::TokenizedType => "TokenizedType",
            Type::TS11 => "NOTATION",
            Type::TS12 => "#REQUIRED",
            Type::TS13 => "#IMPLIED",
            Type::TS14 => "#FIXED",
            Type::TS15 => "ENTITY",
            Type::Percent => "%",
            Type::DQuote => "\"",
            Type::TS16 => "EntityValue_token1",
            Type::SQuote => "'",
            Type::TS17 => "EntityValue_token2",
            Type::TS18 => "NDATA",
            Type::SemiColon => ";",
            Type::TS19 => "_S",
            Type::Nmtoken => "Nmtoken",
            Type::Amp => "&",
            Type::TS20 => "&#",
            Type::TS21 => "CharRef_token1",
            Type::TS22 => "&#x",
            Type::TS23 => "CharRef_token2",
            Type::TS24 => "AttValue_token1",
            Type::TS25 => "AttValue_token2",
            Type::TS26 => "SYSTEM",
            Type::TS27 => "PUBLIC",
            Type::Uri => "URI",
            Type::TS28 => "PubidLiteral_token1",
            Type::TS29 => "PubidLiteral_token2",
            Type::TS30 => "<?",
            Type::Xml => "xml",
            Type::TS31 => "?>",
            Type::Version => "version",
            Type::VersionNum => "VersionNum",
            Type::Encoding => "encoding",
            Type::EncName => "EncName",
            Type::Eq => "=",
            Type::Standalone => "standalone",
            Type::Yes => "yes",
            Type::No => "no",
            Type::TS32 => "DOCTYPE",
            Type::RBracket => "]",
            Type::LT => "<",
            Type::TS33 => "/>",
            Type::TS34 => "</",
            Type::TS35 => "xml-stylesheet",
            Type::TS36 => "xml-model",
            Type::PiTarget => "PITarget",
            Type::_PiContent => "_pi_content",
            Type::Comment => "Comment",
            Type::CharData => "CharData",
            Type::CData => "CData",
            Type::Error => "ERROR",
            Type::Document => "document",
            Type::_Markupdecl => "_markupdecl",
            Type::TS38 => "_DeclSep",
            Type::Elementdecl => "elementdecl",
            Type::Contentspec => "contentspec",
            Type::Mixed => "Mixed",
            Type::Children => "children",
            Type::_Cp => "_cp",
            Type::_Choice => "_choice",
            Type::AttlistDecl => "AttlistDecl",
            Type::AttDef => "AttDef",
            Type::AttType => "_AttType",
            Type::StringType => "StringType",
            Type::EnumeratedType => "_EnumeratedType",
            Type::NotationType => "NotationType",
            Type::Enumeration => "Enumeration",
            Type::DefaultDecl => "DefaultDecl",
            Type::EntityDecl => "_EntityDecl",
            Type::GeDecl => "GEDecl",
            Type::PeDecl => "PEDecl",
            Type::EntityValue => "EntityValue",
            Type::NDataDecl => "NDataDecl",
            Type::NotationDecl => "NotationDecl",
            Type::PeReference => "PEReference",
            Type::Reference => "_Reference",
            Type::EntityRef => "EntityRef",
            Type::CharRef => "CharRef",
            Type::AttValue => "AttValue",
            Type::ExternalId => "ExternalID",
            Type::PublicId => "PublicID",
            Type::SystemLiteral => "SystemLiteral",
            Type::PubidLiteral => "PubidLiteral",
            Type::XmlDecl => "XMLDecl",
            Type::TS41 => "_VersionInfo",
            Type::TS42 => "_EncodingDecl",
            Type::Pi => "PI",
            Type::TS43 => "_Eq",
            Type::Prolog => "prolog",
            Type::TS44 => "_Misc",
            Type::TS45 => "_SDDecl",
            Type::Doctypedecl => "doctypedecl",
            Type::TS46 => "_intSubset",
            Type::Element => "element",
            Type::EmptyElemTag => "EmptyElemTag",
            Type::Attribute => "Attribute",
            Type::STag => "STag",
            Type::ETag => "ETag",
            Type::Content => "content",
            Type::CdSect => "CDSect",
            Type::CdStart => "CDStart",
            Type::StyleSheetPi => "StyleSheetPI",
            Type::XmlModelPi => "XmlModelPI",
            Type::PseudoAtt => "PseudoAtt",
            Type::PseudoAttValue => "PseudoAttValue",
            Type::DocumentRepeat1 => "document_repeat1",
            Type::TS48 => "Mixed_repeat1",
            Type::TS49 => "Mixed_repeat2",
            Type::_ChoiceRepeat1 => "_choice_repeat1",
            Type::_ChoiceRepeat2 => "_choice_repeat2",
            Type::TS52 => "AttlistDecl_repeat1",
            Type::TS53 => "NotationType_repeat1",
            Type::TS54 => "Enumeration_repeat1",
            Type::TS55 => "EntityValue_repeat1",
            Type::TS56 => "EntityValue_repeat2",
            Type::TS57 => "AttValue_repeat1",
            Type::TS58 => "AttValue_repeat2",
            Type::TS59 => "EmptyElemTag_repeat1",
            Type::ContentRepeat1 => "content_repeat1",
            Type::TS61 => "StyleSheetPI_repeat1",
            Type::Spaces => "Spaces",
            Type::Directory => "Directory",
            Type::MavenDirectory => "MavenDirectory",
            Type::ERROR => "ERROR",
        }
    }
}

const S_T_L: &'static [Type] = &[
    Type::End,
    Type::Name,
    Type::TS0,
    Type::TS1,
    Type::TS2,
    Type::LBracket,
    Type::TS3,
    Type::TS4,
    Type::TS5,
    Type::GT,
    Type::TS6,
    Type::TS7,
    Type::LParen,
    Type::TS8,
    Type::Pipe,
    Type::RParen,
    Type::Star,
    Type::QMark,
    Type::Plus,
    Type::Comma,
    Type::TS9,
    Type::TS10,
    Type::TokenizedType,
    Type::TS11,
    Type::TS12,
    Type::TS13,
    Type::TS14,
    Type::TS15,
    Type::Percent,
    Type::DQuote,
    Type::TS16,
    Type::SQuote,
    Type::TS17,
    Type::TS18,
    Type::SemiColon,
    Type::TS19,
    Type::Nmtoken,
    Type::Amp,
    Type::TS20,
    Type::TS21,
    Type::TS22,
    Type::TS23,
    Type::TS24,
    Type::TS25,
    Type::TS26,
    Type::TS27,
    Type::Uri,
    Type::TS28,
    Type::TS29,
    Type::TS30,
    Type::Xml,
    Type::TS31,
    Type::Version,
    Type::VersionNum,
    Type::Encoding,
    Type::EncName,
    Type::Eq,
    Type::Standalone,
    Type::Yes,
    Type::No,
    Type::TS32,
    Type::RBracket,
    Type::LT,
    Type::TS33,
    Type::TS34,
    Type::TS35,
    Type::TS36,
    Type::PiTarget,
    Type::_PiContent,
    Type::Comment,
    Type::CharData,
    Type::CData,
    Type::Error,
    Type::Document,
    Type::_Markupdecl,
    Type::TS38,
    Type::Elementdecl,
    Type::Contentspec,
    Type::Mixed,
    Type::Children,
    Type::_Cp,
    Type::_Choice,
    Type::AttlistDecl,
    Type::AttDef,
    Type::AttType,
    Type::StringType,
    Type::EnumeratedType,
    Type::NotationType,
    Type::Enumeration,
    Type::DefaultDecl,
    Type::EntityDecl,
    Type::GeDecl,
    Type::PeDecl,
    Type::EntityValue,
    Type::NDataDecl,
    Type::NotationDecl,
    Type::PeReference,
    Type::Reference,
    Type::EntityRef,
    Type::CharRef,
    Type::AttValue,
    Type::ExternalId,
    Type::PublicId,
    Type::SystemLiteral,
    Type::PubidLiteral,
    Type::XmlDecl,
    Type::TS41,
    Type::TS42,
    Type::Pi,
    Type::TS43,
    Type::Prolog,
    Type::TS44,
    Type::TS45,
    Type::Doctypedecl,
    Type::TS46,
    Type::Element,
    Type::EmptyElemTag,
    Type::Attribute,
    Type::STag,
    Type::ETag,
    Type::Content,
    Type::CdSect,
    Type::CdStart,
    Type::StyleSheetPi,
    Type::XmlModelPi,
    Type::PseudoAtt,
    Type::PseudoAttValue,
    Type::DocumentRepeat1,
    Type::TS48,
    Type::TS49,
    Type::_ChoiceRepeat1,
    Type::_ChoiceRepeat2,
    Type::TS52,
    Type::TS53,
    Type::TS54,
    Type::TS55,
    Type::TS56,
    Type::TS57,
    Type::TS58,
    Type::TS59,
    Type::ContentRepeat1,
    Type::TS61,
    Type::Spaces,
    Type::MavenDirectory, // NOTE maven specific
    Type::Directory,
    Type::ERROR,
];
