use std::{fmt::Display, u16};

use hyperast::types::{
    AnyType, HyperType, LangRef, NodeId, TypeStore, TypeTrait, TypeU16, TypedNodeId, AAAA,
};

#[cfg(feature = "impl")]
mod impls {
    use super::*;
    use hyperast::tree_gen::utils_ts::{TsEnableTS, TsType};

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

    use hyperast::types::{LangWrapper, RoleStore};

    impl TypeStore for TStore {
        type Ty = TypeU16<Xml>;
    }
    impl TypeStore for &TStore {
        type Ty = TypeU16<Xml>;
    }

    impl XmlEnabledTypeStore for TStore {
        fn resolve(t: Self::Ty) -> Type {
            t.e()
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

pub fn as_any(t: &Type) -> AnyType {
    let t = <Xml as hyperast::types::Lang<Type>>::to_u16(*t);
    let t = <Xml as hyperast::types::Lang<Type>>::make(t);
    let t: &'static dyn HyperType = t;
    t.into()
}

#[cfg(not(feature = "impl"))]
pub trait XmlEnabledTypeStore: hyperast::types::ETypeStore<Ty2 = Type> {
    fn resolve(t: Self::Ty) -> Type;
}

#[cfg(feature = "impl")]
pub trait XmlEnabledTypeStore:
    hyperast::types::ETypeStore<Ty2 = Type> + hyperast::tree_gen::utils_ts::TsEnableTS
{
    fn resolve(t: Self::Ty) -> Type;
}

#[derive(Clone, Copy)]
pub struct TStore;

impl Default for TStore {
    fn default() -> Self {
        Self
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

type TypeInternalSize = u16;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct T(TypeInternalSize);

#[derive(Debug)]
pub struct Lang;

pub type Xml = Lang;

impl hyperast::types::Lang<Type> for Xml {
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

impl LangRef<hyperast::types::TypeU16<Self>> for Lang {
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

    fn as_shared(&self) -> hyperast::types::Shared {
        use hyperast::types::Shared;
        match self {
            _ => Shared::Other,
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_static(&self) -> &'static dyn HyperType {
        let t = <Xml as hyperast::types::Lang<Type>>::to_u16(*self);
        let t = <Xml as hyperast::types::Lang<Type>>::make(t);
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

impl hyperast::types::LLang<hyperast::types::TypeU16<Self>> for Xml {
    type I = u16;

    type E = Type;

    const TE: &[Self::E] = S_T_L;

    fn as_lang_wrapper() -> hyperast::types::LangWrapper<hyperast::types::TypeU16<Self>> {
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
    Xml,
    TS1,
    Standalone,
    SQuote,
    Yes,
    No,
    DQuote,
    TS2,
    TS3,
    LBracket,
    RBracket,
    GT,
    LT,
    TS4,
    TS5,
    TS6,
    TS7,
    TS8,
    TS9,
    TS10,
    TS11,
    TS12,
    TS13,
    TS14,
    TS15,
    LParen,
    TS16,
    Pipe,
    RParen,
    Star,
    QMark,
    Plus,
    Comma,
    TS17,
    TokenizedType,
    TS18,
    TS19,
    TS20,
    TS21,
    TS22,
    Percent,
    TS23,
    TS24,
    TS25,
    SemiColon,
    TS26,
    Nmtoken,
    Amp,
    TS27,
    TS28,
    TS29,
    TS30,
    TS31,
    TS32,
    Uri,
    TS33,
    TS34,
    Version,
    VersionNum,
    Encoding,
    EncName,
    Eq,
    PiTarget,
    _PiContent,
    Comment,
    CharData,
    CData,
    _ErroneousEndName,
    Document,
    Prolog,
    TS35,
    XmlDecl,
    TS36,
    Doctypedecl,
    TS37,
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
    TS39,
    TS40,
    Pi,
    TS41,
    DocumentRepeat1,
    TS42,
    ContentRepeat1,
    TS43,
    TS44,
    TS45,
    TS46,
    TS47,
    _ChoiceRepeat1,
    _ChoiceRepeat2,
    TS48,
    TS49,
    TS50,
    TS51,
    TS52,
    Spaces,
    MavenDirectory,
    Directory,
    ERROR,
}
impl Type {
    pub fn from_u16(t: u16) -> Type {
        match t {
            0u16 => Type::End,
            1u16 => Type::Name,
            2u16 => Type::TS0,
            3u16 => Type::Xml,
            4u16 => Type::TS1,
            5u16 => Type::Standalone,
            6u16 => Type::SQuote,
            7u16 => Type::Yes,
            8u16 => Type::No,
            9u16 => Type::DQuote,
            10u16 => Type::TS2,
            11u16 => Type::TS3,
            12u16 => Type::LBracket,
            13u16 => Type::RBracket,
            14u16 => Type::GT,
            15u16 => Type::LT,
            16u16 => Type::TS4,
            17u16 => Type::TS5,
            18u16 => Type::TS6,
            19u16 => Type::TS7,
            20u16 => Type::TS8,
            21u16 => Type::TS9,
            22u16 => Type::TS10,
            23u16 => Type::TS11,
            24u16 => Type::TS12,
            25u16 => Type::TS13,
            26u16 => Type::TS14,
            27u16 => Type::TS15,
            28u16 => Type::LParen,
            29u16 => Type::TS16,
            30u16 => Type::Pipe,
            31u16 => Type::RParen,
            32u16 => Type::Star,
            33u16 => Type::QMark,
            34u16 => Type::Plus,
            35u16 => Type::Comma,
            36u16 => Type::TS17,
            37u16 => Type::TokenizedType,
            38u16 => Type::TS18,
            39u16 => Type::TS19,
            40u16 => Type::TS20,
            41u16 => Type::TS21,
            42u16 => Type::TS22,
            43u16 => Type::Percent,
            44u16 => Type::TS23,
            45u16 => Type::TS24,
            46u16 => Type::TS25,
            47u16 => Type::SemiColon,
            48u16 => Type::TS26,
            49u16 => Type::Nmtoken,
            50u16 => Type::Amp,
            51u16 => Type::TS27,
            52u16 => Type::TS28,
            53u16 => Type::TS29,
            54u16 => Type::TS30,
            55u16 => Type::TS31,
            56u16 => Type::TS32,
            57u16 => Type::Uri,
            58u16 => Type::Uri,
            59u16 => Type::TS33,
            60u16 => Type::TS34,
            61u16 => Type::Version,
            62u16 => Type::VersionNum,
            63u16 => Type::Encoding,
            64u16 => Type::EncName,
            65u16 => Type::Eq,
            66u16 => Type::PiTarget,
            67u16 => Type::_PiContent,
            68u16 => Type::Comment,
            69u16 => Type::CharData,
            70u16 => Type::CData,
            71u16 => Type::Name,
            72u16 => Type::Name,
            73u16 => Type::_ErroneousEndName,
            74u16 => Type::Document,
            75u16 => Type::Prolog,
            76u16 => Type::TS35,
            77u16 => Type::XmlDecl,
            78u16 => Type::TS36,
            79u16 => Type::Doctypedecl,
            80u16 => Type::TS37,
            81u16 => Type::Element,
            82u16 => Type::EmptyElemTag,
            83u16 => Type::Attribute,
            84u16 => Type::STag,
            85u16 => Type::ETag,
            86u16 => Type::Content,
            87u16 => Type::CdSect,
            88u16 => Type::CdStart,
            89u16 => Type::StyleSheetPi,
            90u16 => Type::XmlModelPi,
            91u16 => Type::PseudoAtt,
            92u16 => Type::PseudoAttValue,
            93u16 => Type::_Markupdecl,
            94u16 => Type::TS38,
            95u16 => Type::Elementdecl,
            96u16 => Type::Contentspec,
            97u16 => Type::Mixed,
            98u16 => Type::Children,
            99u16 => Type::_Cp,
            100u16 => Type::_Choice,
            101u16 => Type::AttlistDecl,
            102u16 => Type::AttDef,
            103u16 => Type::AttType,
            104u16 => Type::StringType,
            105u16 => Type::EnumeratedType,
            106u16 => Type::NotationType,
            107u16 => Type::Enumeration,
            108u16 => Type::DefaultDecl,
            109u16 => Type::EntityDecl,
            110u16 => Type::GeDecl,
            111u16 => Type::PeDecl,
            112u16 => Type::EntityValue,
            113u16 => Type::NDataDecl,
            114u16 => Type::NotationDecl,
            115u16 => Type::PeReference,
            116u16 => Type::Reference,
            117u16 => Type::EntityRef,
            118u16 => Type::CharRef,
            119u16 => Type::AttValue,
            120u16 => Type::ExternalId,
            121u16 => Type::PublicId,
            122u16 => Type::SystemLiteral,
            123u16 => Type::PubidLiteral,
            124u16 => Type::TS39,
            125u16 => Type::TS40,
            126u16 => Type::Pi,
            127u16 => Type::TS41,
            128u16 => Type::DocumentRepeat1,
            129u16 => Type::TS42,
            130u16 => Type::ContentRepeat1,
            131u16 => Type::TS43,
            132u16 => Type::TS44,
            133u16 => Type::TS45,
            134u16 => Type::TS46,
            135u16 => Type::TS47,
            136u16 => Type::_ChoiceRepeat1,
            137u16 => Type::_ChoiceRepeat2,
            138u16 => Type::TS48,
            139u16 => Type::TS49,
            140u16 => Type::TS50,
            141u16 => Type::TS51,
            142u16 => Type::TS52,
            u16::MAX => Type::ERROR,
            x => panic!("{}", x),
        }
    }
    pub fn from_str(t: &str) -> Option<Type> {
        Some(match t {
            "end" => Type::End,
            "Name" => Type::Name,
            "<?" => Type::TS0,
            "xml" => Type::Xml,
            "?>" => Type::TS1,
            "standalone" => Type::Standalone,
            "'" => Type::SQuote,
            "yes" => Type::Yes,
            "no" => Type::No,
            "\"" => Type::DQuote,
            "<!" => Type::TS2,
            "DOCTYPE" => Type::TS3,
            "[" => Type::LBracket,
            "]" => Type::RBracket,
            ">" => Type::GT,
            "<" => Type::LT,
            "/>" => Type::TS4,
            "</" => Type::TS5,
            "]]>" => Type::TS6,
            "<![" => Type::TS7,
            "CDATA" => Type::TS8,
            "xml-stylesheet" => Type::TS9,
            "xml-model" => Type::TS10,
            "PseudoAttValue_token1" => Type::TS11,
            "PseudoAttValue_token2" => Type::TS12,
            "ELEMENT" => Type::TS13,
            "EMPTY" => Type::TS14,
            "ANY" => Type::TS15,
            "(" => Type::LParen,
            "#PCDATA" => Type::TS16,
            "|" => Type::Pipe,
            ")" => Type::RParen,
            "*" => Type::Star,
            "?" => Type::QMark,
            "+" => Type::Plus,
            "," => Type::Comma,
            "ATTLIST" => Type::TS17,
            "TokenizedType" => Type::TokenizedType,
            "NOTATION" => Type::TS18,
            "#REQUIRED" => Type::TS19,
            "#IMPLIED" => Type::TS20,
            "#FIXED" => Type::TS21,
            "ENTITY" => Type::TS22,
            "%" => Type::Percent,
            "EntityValue_token1" => Type::TS23,
            "EntityValue_token2" => Type::TS24,
            "NDATA" => Type::TS25,
            ";" => Type::SemiColon,
            "_S" => Type::TS26,
            "Nmtoken" => Type::Nmtoken,
            "&" => Type::Amp,
            "&#" => Type::TS27,
            "CharRef_token1" => Type::TS28,
            "&#x" => Type::TS29,
            "CharRef_token2" => Type::TS30,
            "SYSTEM" => Type::TS31,
            "PUBLIC" => Type::TS32,
            "URI" => Type::Uri,
            "PubidLiteral_token1" => Type::TS33,
            "PubidLiteral_token2" => Type::TS34,
            "version" => Type::Version,
            "VersionNum" => Type::VersionNum,
            "encoding" => Type::Encoding,
            "EncName" => Type::EncName,
            "=" => Type::Eq,
            "PITarget" => Type::PiTarget,
            "_pi_content" => Type::_PiContent,
            "Comment" => Type::Comment,
            "CharData" => Type::CharData,
            "CData" => Type::CData,
            "_erroneous_end_name" => Type::_ErroneousEndName,
            "document" => Type::Document,
            "prolog" => Type::Prolog,
            "_Misc" => Type::TS35,
            "XMLDecl" => Type::XmlDecl,
            "_SDDecl" => Type::TS36,
            "doctypedecl" => Type::Doctypedecl,
            "_intSubset" => Type::TS37,
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
            "_VersionInfo" => Type::TS39,
            "_EncodingDecl" => Type::TS40,
            "PI" => Type::Pi,
            "_Eq" => Type::TS41,
            "document_repeat1" => Type::DocumentRepeat1,
            "EmptyElemTag_repeat1" => Type::TS42,
            "content_repeat1" => Type::ContentRepeat1,
            "StyleSheetPI_repeat1" => Type::TS43,
            "PseudoAttValue_repeat1" => Type::TS44,
            "PseudoAttValue_repeat2" => Type::TS45,
            "Mixed_repeat1" => Type::TS46,
            "Mixed_repeat2" => Type::TS47,
            "_choice_repeat1" => Type::_ChoiceRepeat1,
            "_choice_repeat2" => Type::_ChoiceRepeat2,
            "AttlistDecl_repeat1" => Type::TS48,
            "NotationType_repeat1" => Type::TS49,
            "Enumeration_repeat1" => Type::TS50,
            "EntityValue_repeat1" => Type::TS51,
            "EntityValue_repeat2" => Type::TS52,
            "Spaces" => Type::Spaces,
            "MavenDirectory" => Type::MavenDirectory,
            "Directory" => Type::Directory,
            "ERROR" => Type::ERROR,
            _ => return None,
        })
    }
    pub fn to_str(&self) -> &'static str {
        match self {
            Type::End => "end",
            Type::Name => "Name",
            Type::TS0 => "<?",
            Type::Xml => "xml",
            Type::TS1 => "?>",
            Type::Standalone => "standalone",
            Type::SQuote => "'",
            Type::Yes => "yes",
            Type::No => "no",
            Type::DQuote => "\"",
            Type::TS2 => "<!",
            Type::TS3 => "DOCTYPE",
            Type::LBracket => "[",
            Type::RBracket => "]",
            Type::GT => ">",
            Type::LT => "<",
            Type::TS4 => "/>",
            Type::TS5 => "</",
            Type::TS6 => "]]>",
            Type::TS7 => "<![",
            Type::TS8 => "CDATA",
            Type::TS9 => "xml-stylesheet",
            Type::TS10 => "xml-model",
            Type::TS11 => "PseudoAttValue_token1",
            Type::TS12 => "PseudoAttValue_token2",
            Type::TS13 => "ELEMENT",
            Type::TS14 => "EMPTY",
            Type::TS15 => "ANY",
            Type::LParen => "(",
            Type::TS16 => "#PCDATA",
            Type::Pipe => "|",
            Type::RParen => ")",
            Type::Star => "*",
            Type::QMark => "?",
            Type::Plus => "+",
            Type::Comma => ",",
            Type::TS17 => "ATTLIST",
            Type::TokenizedType => "TokenizedType",
            Type::TS18 => "NOTATION",
            Type::TS19 => "#REQUIRED",
            Type::TS20 => "#IMPLIED",
            Type::TS21 => "#FIXED",
            Type::TS22 => "ENTITY",
            Type::Percent => "%",
            Type::TS23 => "EntityValue_token1",
            Type::TS24 => "EntityValue_token2",
            Type::TS25 => "NDATA",
            Type::SemiColon => ";",
            Type::TS26 => "_S",
            Type::Nmtoken => "Nmtoken",
            Type::Amp => "&",
            Type::TS27 => "&#",
            Type::TS28 => "CharRef_token1",
            Type::TS29 => "&#x",
            Type::TS30 => "CharRef_token2",
            Type::TS31 => "SYSTEM",
            Type::TS32 => "PUBLIC",
            Type::Uri => "URI",
            Type::TS33 => "PubidLiteral_token1",
            Type::TS34 => "PubidLiteral_token2",
            Type::Version => "version",
            Type::VersionNum => "VersionNum",
            Type::Encoding => "encoding",
            Type::EncName => "EncName",
            Type::Eq => "=",
            Type::PiTarget => "PITarget",
            Type::_PiContent => "_pi_content",
            Type::Comment => "Comment",
            Type::CharData => "CharData",
            Type::CData => "CData",
            Type::_ErroneousEndName => "_erroneous_end_name",
            Type::Document => "document",
            Type::Prolog => "prolog",
            Type::TS35 => "_Misc",
            Type::XmlDecl => "XMLDecl",
            Type::TS36 => "_SDDecl",
            Type::Doctypedecl => "doctypedecl",
            Type::TS37 => "_intSubset",
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
            Type::TS39 => "_VersionInfo",
            Type::TS40 => "_EncodingDecl",
            Type::Pi => "PI",
            Type::TS41 => "_Eq",
            Type::DocumentRepeat1 => "document_repeat1",
            Type::TS42 => "EmptyElemTag_repeat1",
            Type::ContentRepeat1 => "content_repeat1",
            Type::TS43 => "StyleSheetPI_repeat1",
            Type::TS44 => "PseudoAttValue_repeat1",
            Type::TS45 => "PseudoAttValue_repeat2",
            Type::TS46 => "Mixed_repeat1",
            Type::TS47 => "Mixed_repeat2",
            Type::_ChoiceRepeat1 => "_choice_repeat1",
            Type::_ChoiceRepeat2 => "_choice_repeat2",
            Type::TS48 => "AttlistDecl_repeat1",
            Type::TS49 => "NotationType_repeat1",
            Type::TS50 => "Enumeration_repeat1",
            Type::TS51 => "EntityValue_repeat1",
            Type::TS52 => "EntityValue_repeat2",
            Type::Spaces => "Spaces",
            Type::MavenDirectory => "MavenDirectory",
            Type::Directory => "Directory",
            Type::ERROR => "ERROR",
        }
    }
    pub fn is_hidden(&self) -> bool {
        match self {
            Type::End => true,
            Type::TS11 => true,
            Type::TS12 => true,
            Type::TS23 => true,
            Type::TS24 => true,
            Type::TS26 => true,
            Type::TS28 => true,
            Type::TS30 => true,
            Type::TS33 => true,
            Type::TS34 => true,
            Type::_PiContent => true,
            Type::_ErroneousEndName => true,
            Type::TS35 => true,
            Type::TS36 => true,
            Type::TS37 => true,
            Type::_Markupdecl => true,
            Type::TS38 => true,
            Type::_Cp => true,
            Type::_Choice => true,
            Type::AttType => true,
            Type::EnumeratedType => true,
            Type::EntityDecl => true,
            Type::Reference => true,
            Type::TS39 => true,
            Type::TS40 => true,
            Type::TS41 => true,
            Type::DocumentRepeat1 => true,
            Type::TS42 => true,
            Type::ContentRepeat1 => true,
            Type::TS43 => true,
            Type::TS44 => true,
            Type::TS45 => true,
            Type::TS46 => true,
            Type::TS47 => true,
            Type::_ChoiceRepeat1 => true,
            Type::_ChoiceRepeat2 => true,
            Type::TS48 => true,
            Type::TS49 => true,
            Type::TS50 => true,
            Type::TS51 => true,
            Type::TS52 => true,
            _ => false,
        }
    }
    pub fn is_supertype(&self) -> bool {
        match self {
            Type::_Markupdecl => true,
            Type::AttType => true,
            Type::EnumeratedType => true,
            Type::EntityDecl => true,
            Type::Reference => true,
            _ => false,
        }
    }
    pub fn is_named(&self) -> bool {
        match self {
            Type::Name => true,
            Type::TokenizedType => true,
            Type::Nmtoken => true,
            Type::Uri => true,
            Type::VersionNum => true,
            Type::EncName => true,
            Type::PiTarget => true,
            Type::Comment => true,
            Type::CharData => true,
            Type::CData => true,
            Type::Document => true,
            Type::Prolog => true,
            Type::XmlDecl => true,
            Type::Doctypedecl => true,
            Type::Element => true,
            Type::EmptyElemTag => true,
            Type::Attribute => true,
            Type::STag => true,
            Type::ETag => true,
            Type::Content => true,
            Type::CdSect => true,
            Type::CdStart => true,
            Type::StyleSheetPi => true,
            Type::XmlModelPi => true,
            Type::PseudoAtt => true,
            Type::PseudoAttValue => true,
            Type::_Markupdecl => true,
            Type::Elementdecl => true,
            Type::Contentspec => true,
            Type::Mixed => true,
            Type::Children => true,
            Type::AttlistDecl => true,
            Type::AttDef => true,
            Type::AttType => true,
            Type::StringType => true,
            Type::EnumeratedType => true,
            Type::NotationType => true,
            Type::Enumeration => true,
            Type::DefaultDecl => true,
            Type::EntityDecl => true,
            Type::GeDecl => true,
            Type::PeDecl => true,
            Type::EntityValue => true,
            Type::NDataDecl => true,
            Type::NotationDecl => true,
            Type::PeReference => true,
            Type::Reference => true,
            Type::EntityRef => true,
            Type::CharRef => true,
            Type::AttValue => true,
            Type::ExternalId => true,
            Type::PublicId => true,
            Type::SystemLiteral => true,
            Type::PubidLiteral => true,
            Type::Pi => true,
            _ => false,
        }
    }
    pub fn is_repeat(&self) -> bool {
        todo!("need to generate with the polyglote crate")
    }
}

const S_T_L: &'static [Type] = &[
    Type::End,
    Type::Name,
    Type::TS0,
    Type::Xml,
    Type::TS1,
    Type::Standalone,
    Type::SQuote,
    Type::Yes,
    Type::No,
    Type::DQuote,
    Type::TS2,
    Type::TS3,
    Type::LBracket,
    Type::RBracket,
    Type::GT,
    Type::LT,
    Type::TS4,
    Type::TS5,
    Type::TS6,
    Type::TS7,
    Type::TS8,
    Type::TS9,
    Type::TS10,
    Type::TS11,
    Type::TS12,
    Type::TS13,
    Type::TS14,
    Type::TS15,
    Type::LParen,
    Type::TS16,
    Type::Pipe,
    Type::RParen,
    Type::Star,
    Type::QMark,
    Type::Plus,
    Type::Comma,
    Type::TS17,
    Type::TokenizedType,
    Type::TS18,
    Type::TS19,
    Type::TS20,
    Type::TS21,
    Type::TS22,
    Type::Percent,
    Type::TS23,
    Type::TS24,
    Type::TS25,
    Type::SemiColon,
    Type::TS26,
    Type::Nmtoken,
    Type::Amp,
    Type::TS27,
    Type::TS28,
    Type::TS29,
    Type::TS30,
    Type::TS31,
    Type::TS32,
    Type::Uri,
    Type::TS33,
    Type::TS34,
    Type::Version,
    Type::VersionNum,
    Type::Encoding,
    Type::EncName,
    Type::Eq,
    Type::PiTarget,
    Type::_PiContent,
    Type::Comment,
    Type::CharData,
    Type::CData,
    Type::_ErroneousEndName,
    Type::Document,
    Type::Prolog,
    Type::TS35,
    Type::XmlDecl,
    Type::TS36,
    Type::Doctypedecl,
    Type::TS37,
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
    Type::TS39,
    Type::TS40,
    Type::Pi,
    Type::TS41,
    Type::DocumentRepeat1,
    Type::TS42,
    Type::ContentRepeat1,
    Type::TS43,
    Type::TS44,
    Type::TS45,
    Type::TS46,
    Type::TS47,
    Type::_ChoiceRepeat1,
    Type::_ChoiceRepeat2,
    Type::TS48,
    Type::TS49,
    Type::TS50,
    Type::TS51,
    Type::TS52,
    Type::Spaces,
    Type::MavenDirectory,
    Type::Directory,
    Type::ERROR,
];
