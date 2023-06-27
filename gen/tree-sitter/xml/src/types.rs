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
        pub fn obtain_type<T>(&self, _: &mut impl XmlEnabledTypeStore<T>) -> Type {
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
            From::<&'static (dyn LangRef<Type>)>::from(&Xml)
        }

        type Marshaled = TypeIndex;

        fn marshal_type(&self, n: &HashedNodeRef<'a, TIdN<NodeIdentifier>>) -> Self::Marshaled {
            TypeIndex {
                lang: LangRef::<Type>::name(&Xml),
                ty: self.resolve_type(n) as u16,
            }
        }
    }
    impl<'a> XmlEnabledTypeStore<HashedNodeRef<'a, TIdN<NodeIdentifier>>> for TStore {
        const LANG: TypeInternalSize = Self::Cpp as u16;

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
                lang: LangRef::<Type>::name(&Xml),
                ty: *n.get_component::<Type>().unwrap() as u16,
            }
        }
    }
}
pub trait XmlEnabledTypeStore<T>: TypeStore<T> {
    const LANG: u16;
    // fn obtain(&self, n: &TNode) -> Type {
    //     let t = n.kind_id();
    //     Type::from_u16(t)
    // }
    fn intern(&self, t: Type) -> Self::Ty {
        let t = t as u16;
        Self::_intern(Self::LANG, t)
    }
    fn _intern(l: u16, t: u16) -> Self::Ty;
    fn resolve(&self, t: Self::Ty) -> Type;
}

#[repr(u8)]
pub(crate) enum TStore {
    Cpp = 0,
}

impl Default for TStore {
    fn default() -> Self {
        Self::Cpp
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

type TypeInternalSize = u16;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct T(TypeInternalSize);

pub struct Xml;

impl Lang<Type> for Xml {
    fn make(t: u16) -> &'static Type {
        Xml.make(t)
    }
    fn to_u16(t: Type) -> u16 {
        Xml.to_u16(t)
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
}

impl LangRef<AnyType> for Xml {
    fn name(&self) -> &'static str {
        std::any::type_name::<Xml>()
    }

    fn make(&self, t: u16) -> &'static AnyType {
        todo!()
    }

    fn to_u16(&self, t: AnyType) -> u16 {
        todo!()
    }
}
impl HyperType for Type {
    fn as_shared(&self) -> hyper_ast::types::Shared {
        use hyper_ast::types::Shared;
        match self {
            _ => Shared::Other,
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn is_file(&self) -> bool {
        self == &Type::SourceFile
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

    fn get_lang(&self) -> hyper_ast::types::LangWrapper<Self>
    where
        Self: Sized,
    {
        todo!()
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
        f.write_str(Type::to_str(*self))
    }
}

// #[repr(u16)]
// #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
// pub enum Type {
//     Spaces,
//     SourceFile,
//     MavenDirectory,
//     Directory,
//     ERROR,
// }
// impl Type {
//     pub fn from_u16(t: u16) -> Type {
//         todo!()
//     }
// }

// const S_T_L: &'static [Type] = &[
//     Type::Spaces,
//     Type::SourceFile,
//     Type::MavenDirectory,
//     Type::Directory,
//     Type::ERROR,
// ];

#[repr(u16)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Type {
    End,
    TS0,
    TS1,
    Name,
    TS2,
    Nmtoken,
    TS3,
    TS4,
    TS5,
    TS6,
    TS7,
    SystemLiteral,
    PubidLiteral,
    CharData,
    Comment,
    TS8,
    TS9,
    CdSect,
    TS10,
    Version,
    Eq,
    VersionNum,
    TS11,
    LBracket,
    RBracket,
    GT,
    Standalone,
    Yes,
    No,
    LT,
    TS12,
    TS13,
    TS14,
    TS15,
    TS16,
    QMark,
    Star,
    Plus,
    LParen,
    Pipe,
    RParen,
    Comma,
    TS17,
    TS18,
    TS19,
    StringType,
    TS20,
    TS21,
    TS22,
    TS23,
    TS24,
    TS25,
    TS26,
    TS27,
    TS28,
    TS29,
    TS30,
    CharRef,
    Amp,
    SemiColon,
    Percent,
    TS31,
    TS32,
    TS33,
    TS34,
    Encoding,
    EncName,
    TS35,
    SourceFile,
    EntityValue,
    AttValue,
    Sep1,
    Sep2,
    Sep3,
    Text,
    Pi,
    TS36,
    Prolog,
    XmlDecl,
    VersionInfo,
    TS37,
    TS38,
    Doctypedecl,
    TS39,
    TS40,
    TS41,
    SdDecl,
    Element,
    STag,
    Attribute,
    ETag,
    TS42,
    EmptyElemTag,
    Elementdecl,
    Contentspec,
    Children,
    TS43,
    TS44,
    TS45,
    Mixed,
    AttlistDecl,
    TS46,
    TS47,
    TokenizedType,
    TS48,
    NotationType,
    Enumeration,
    DefaultDecl,
    TS49,
    EntityRef,
    PeReference,
    TS50,
    GeDecl,
    PeDecl,
    TS51,
    TS52,
    ExternalId,
    NDataDecl,
    EncodingDecl,
    NotationDecl,
    PublicId,
    SourceFileRepeat1,
    TS53,
    TS54,
    TS55,
    TS56,
    TS57,
    TS58,
    TS59,
    ElementRepeat1,
    TS60,
    TS61,
    TS62,
    TS63,
    TS64,
    TS65,
    Spaces,
    MavenDirectory, // NOTE maven specific
    Directory,
    ERROR,
}
impl Type {
    pub fn from_u16(t: u16) -> Type {
        match t {
            0u16 => Type::End,
            1u16 => Type::TS0,
            2u16 => Type::TS1,
            3u16 => Type::Name,
            4u16 => Type::TS2,
            5u16 => Type::Nmtoken,
            6u16 => Type::TS3,
            7u16 => Type::TS4,
            8u16 => Type::TS5,
            9u16 => Type::TS6,
            10u16 => Type::TS7,
            11u16 => Type::SystemLiteral,
            12u16 => Type::PubidLiteral,
            13u16 => Type::CharData,
            14u16 => Type::Comment,
            15u16 => Type::TS8,
            16u16 => Type::TS9,
            17u16 => Type::CdSect,
            18u16 => Type::TS10,
            19u16 => Type::Version,
            20u16 => Type::Eq,
            21u16 => Type::VersionNum,
            22u16 => Type::TS11,
            23u16 => Type::LBracket,
            24u16 => Type::RBracket,
            25u16 => Type::GT,
            26u16 => Type::Standalone,
            27u16 => Type::Yes,
            28u16 => Type::No,
            29u16 => Type::LT,
            30u16 => Type::TS12,
            31u16 => Type::TS13,
            32u16 => Type::TS14,
            33u16 => Type::TS15,
            34u16 => Type::TS16,
            35u16 => Type::QMark,
            36u16 => Type::Star,
            37u16 => Type::Plus,
            38u16 => Type::LParen,
            39u16 => Type::Pipe,
            40u16 => Type::RParen,
            41u16 => Type::Comma,
            42u16 => Type::TS17,
            43u16 => Type::TS18,
            44u16 => Type::TS19,
            45u16 => Type::StringType,
            46u16 => Type::TS20,
            47u16 => Type::TS21,
            48u16 => Type::TS22,
            49u16 => Type::TS23,
            50u16 => Type::TS24,
            51u16 => Type::TS25,
            52u16 => Type::TS26,
            53u16 => Type::TS27,
            54u16 => Type::TS28,
            55u16 => Type::TS29,
            56u16 => Type::TS30,
            57u16 => Type::CharRef,
            58u16 => Type::Amp,
            59u16 => Type::SemiColon,
            60u16 => Type::Percent,
            61u16 => Type::TS31,
            62u16 => Type::TS32,
            63u16 => Type::TS33,
            64u16 => Type::TS34,
            65u16 => Type::Encoding,
            66u16 => Type::EncName,
            67u16 => Type::TS35,
            68u16 => Type::SourceFile,
            69u16 => Type::EntityValue,
            70u16 => Type::AttValue,
            71u16 => Type::Sep1,
            72u16 => Type::Sep2,
            73u16 => Type::Sep3,
            74u16 => Type::Text,
            75u16 => Type::Pi,
            76u16 => Type::TS36,
            77u16 => Type::Prolog,
            78u16 => Type::XmlDecl,
            79u16 => Type::VersionInfo,
            80u16 => Type::TS37,
            81u16 => Type::TS38,
            82u16 => Type::Doctypedecl,
            83u16 => Type::TS39,
            84u16 => Type::TS40,
            85u16 => Type::TS41,
            86u16 => Type::SdDecl,
            87u16 => Type::Element,
            88u16 => Type::STag,
            89u16 => Type::Attribute,
            90u16 => Type::ETag,
            91u16 => Type::TS42,
            92u16 => Type::EmptyElemTag,
            93u16 => Type::Elementdecl,
            94u16 => Type::Contentspec,
            95u16 => Type::Children,
            96u16 => Type::TS43,
            97u16 => Type::TS44,
            98u16 => Type::TS45,
            99u16 => Type::Mixed,
            100u16 => Type::AttlistDecl,
            101u16 => Type::TS46,
            102u16 => Type::TS47,
            103u16 => Type::TokenizedType,
            104u16 => Type::TS48,
            105u16 => Type::NotationType,
            106u16 => Type::Enumeration,
            107u16 => Type::DefaultDecl,
            108u16 => Type::TS49,
            109u16 => Type::EntityRef,
            110u16 => Type::PeReference,
            111u16 => Type::TS50,
            112u16 => Type::GeDecl,
            113u16 => Type::PeDecl,
            114u16 => Type::TS51,
            115u16 => Type::TS52,
            116u16 => Type::ExternalId,
            117u16 => Type::NDataDecl,
            118u16 => Type::EncodingDecl,
            119u16 => Type::NotationDecl,
            120u16 => Type::PublicId,
            121u16 => Type::SourceFileRepeat1,
            122u16 => Type::TS53,
            123u16 => Type::TS54,
            124u16 => Type::TS55,
            125u16 => Type::TS56,
            126u16 => Type::TS57,
            127u16 => Type::TS58,
            128u16 => Type::TS59,
            129u16 => Type::ElementRepeat1,
            130u16 => Type::TS60,
            131u16 => Type::TS61,
            132u16 => Type::TS62,
            133u16 => Type::TS63,
            134u16 => Type::TS64,
            135u16 => Type::TS65,
            // 136u16 => Type::ERROR,
            u16::MAX => Type::ERROR,
            x => panic!("{}", x),
        }
    }
    pub fn from_str(t: &str) -> Option<Type> {
        Some(match t {
            "end" => Type::End,
            "_Char" => Type::TS0,
            "_S" => Type::TS1,
            "Name" => Type::Name,
            " " => Type::TS2,
            "Nmtoken" => Type::Nmtoken,
            "\"" => Type::TS3,
            "'" => Type::TS4,
            "Sep1_token1" => Type::TS5,
            "Sep2_token1" => Type::TS6,
            "Sep3_token1" => Type::TS7,
            "SystemLiteral" => Type::SystemLiteral,
            "PubidLiteral" => Type::PubidLiteral,
            "CharData" => Type::CharData,
            "Comment" => Type::Comment,
            "<?" => Type::TS8,
            "?>" => Type::TS9,
            "CDSect" => Type::CdSect,
            "<?xml" => Type::TS10,
            "version" => Type::Version,
            "=" => Type::Eq,
            "VersionNum" => Type::VersionNum,
            "<!DOCTYPE" => Type::TS11,
            "[" => Type::LBracket,
            "]" => Type::RBracket,
            ">" => Type::GT,
            "standalone" => Type::Standalone,
            "yes" => Type::Yes,
            "no" => Type::No,
            "<" => Type::LT,
            "</" => Type::TS12,
            "/>" => Type::TS13,
            "<!ELEMENT" => Type::TS14,
            "EMPTY" => Type::TS15,
            "ANY" => Type::TS16,
            "?" => Type::QMark,
            "*" => Type::Star,
            "+" => Type::Plus,
            "(" => Type::LParen,
            "|" => Type::Pipe,
            ")" => Type::RParen,
            "," => Type::Comma,
            "#PCDATA" => Type::TS17,
            ")*" => Type::TS18,
            "<!ATTLIST" => Type::TS19,
            "StringType" => Type::StringType,
            "ID" => Type::TS20,
            "IDREF" => Type::TS21,
            "IDREFS" => Type::TS22,
            "ENTITY" => Type::TS23,
            "ENTITIES" => Type::TS24,
            "NMTOKEN" => Type::TS25,
            "NMTOKENS" => Type::TS26,
            "NOTATION" => Type::TS27,
            "#REQUIRED" => Type::TS28,
            "#IMPLIED" => Type::TS29,
            "#FIXED" => Type::TS30,
            "CharRef" => Type::CharRef,
            "&" => Type::Amp,
            ";" => Type::SemiColon,
            "%" => Type::Percent,
            "<!ENTITY" => Type::TS31,
            "SYSTEM" => Type::TS32,
            "PUBLIC" => Type::TS33,
            "NDATA" => Type::TS34,
            "encoding" => Type::Encoding,
            "EncName" => Type::EncName,
            "<!NOTATION" => Type::TS35,
            "source_file" => Type::SourceFile,
            "EntityValue" => Type::EntityValue,
            "AttValue" => Type::AttValue,
            "Sep1" => Type::Sep1,
            "Sep2" => Type::Sep2,
            "Sep3" => Type::Sep3,
            "Text" => Type::Text,
            "PI" => Type::Pi,
            "_PITarget" => Type::TS36,
            "prolog" => Type::Prolog,
            "XMLDecl" => Type::XmlDecl,
            "VersionInfo" => Type::VersionInfo,
            "_Eq" => Type::TS37,
            "_Misc" => Type::TS38,
            "doctypedecl" => Type::Doctypedecl,
            "_DeclSep" => Type::TS39,
            "_intSubset" => Type::TS40,
            "_markupdecl" => Type::TS41,
            "SDDecl" => Type::SdDecl,
            "element" => Type::Element,
            "STag" => Type::STag,
            "Attribute" => Type::Attribute,
            "ETag" => Type::ETag,
            "_content" => Type::TS42,
            "EmptyElemTag" => Type::EmptyElemTag,
            "elementdecl" => Type::Elementdecl,
            "contentspec" => Type::Contentspec,
            "children" => Type::Children,
            "_cp" => Type::TS43,
            "_choice" => Type::TS44,
            "_seq" => Type::TS45,
            "Mixed" => Type::Mixed,
            "AttlistDecl" => Type::AttlistDecl,
            "_AttDef" => Type::TS46,
            "_AttType" => Type::TS47,
            "TokenizedType" => Type::TokenizedType,
            "_EnumeratedType" => Type::TS48,
            "NotationType" => Type::NotationType,
            "Enumeration" => Type::Enumeration,
            "DefaultDecl" => Type::DefaultDecl,
            "_Reference" => Type::TS49,
            "EntityRef" => Type::EntityRef,
            "PEReference" => Type::PeReference,
            "_EntityDecl" => Type::TS50,
            "GEDecl" => Type::GeDecl,
            "PEDecl" => Type::PeDecl,
            "_EntityDef" => Type::TS51,
            "_PEDef" => Type::TS52,
            "ExternalID" => Type::ExternalId,
            "NDataDecl" => Type::NDataDecl,
            "EncodingDecl" => Type::EncodingDecl,
            "NotationDecl" => Type::NotationDecl,
            "PublicID" => Type::PublicId,
            "source_file_repeat1" => Type::SourceFileRepeat1,
            "EntityValue_repeat1" => Type::TS53,
            "AttValue_repeat1" => Type::TS54,
            "AttValue_repeat2" => Type::TS55,
            "Sep1_repeat1" => Type::TS56,
            "Sep2_repeat1" => Type::TS57,
            "Sep3_repeat1" => Type::TS58,
            "Text_repeat1" => Type::TS59,
            "element_repeat1" => Type::ElementRepeat1,
            "STag_repeat1" => Type::TS60,
            "_choice_repeat1" => Type::TS61,
            "_seq_repeat1" => Type::TS62,
            "Mixed_repeat1" => Type::TS63,
            "AttlistDecl_repeat1" => Type::TS64,
            "Enumeration_repeat1" => Type::TS65,
            "ERROR" => Type::ERROR,
            x => return None,
        })
    }
    pub fn to_str(t: Type) -> &'static str {
        match t {
            Type::End => "end",
            Type::TS0 => "_Char",
            Type::TS1 => "_S",
            Type::Name => "Name",
            Type::TS2 => " ",
            Type::Nmtoken => "Nmtoken",
            Type::TS3 => "\"",
            Type::TS4 => "'",
            Type::TS5 => "Sep1_token1",
            Type::TS6 => "Sep2_token1",
            Type::TS7 => "Sep3_token1",
            Type::SystemLiteral => "SystemLiteral",
            Type::PubidLiteral => "PubidLiteral",
            Type::CharData => "CharData",
            Type::Comment => "Comment",
            Type::TS8 => "<?",
            Type::TS9 => "?>",
            Type::CdSect => "CDSect",
            Type::TS10 => "<?xml",
            Type::Version => "version",
            Type::Eq => "=",
            Type::VersionNum => "VersionNum",
            Type::TS11 => "<!DOCTYPE",
            Type::LBracket => "[",
            Type::RBracket => "]",
            Type::GT => ">",
            Type::Standalone => "standalone",
            Type::Yes => "yes",
            Type::No => "no",
            Type::LT => "<",
            Type::TS12 => "</",
            Type::TS13 => "/>",
            Type::TS14 => "<!ELEMENT",
            Type::TS15 => "EMPTY",
            Type::TS16 => "ANY",
            Type::QMark => "?",
            Type::Star => "*",
            Type::Plus => "+",
            Type::LParen => "(",
            Type::Pipe => "|",
            Type::RParen => ")",
            Type::Comma => ",",
            Type::TS17 => "#PCDATA",
            Type::TS18 => ")*",
            Type::TS19 => "<!ATTLIST",
            Type::StringType => "StringType",
            Type::TS20 => "ID",
            Type::TS21 => "IDREF",
            Type::TS22 => "IDREFS",
            Type::TS23 => "ENTITY",
            Type::TS24 => "ENTITIES",
            Type::TS25 => "NMTOKEN",
            Type::TS26 => "NMTOKENS",
            Type::TS27 => "NOTATION",
            Type::TS28 => "#REQUIRED",
            Type::TS29 => "#IMPLIED",
            Type::TS30 => "#FIXED",
            Type::CharRef => "CharRef",
            Type::Amp => "&",
            Type::SemiColon => ";",
            Type::Percent => "%",
            Type::TS31 => "<!ENTITY",
            Type::TS32 => "SYSTEM",
            Type::TS33 => "PUBLIC",
            Type::TS34 => "NDATA",
            Type::Encoding => "encoding",
            Type::EncName => "EncName",
            Type::TS35 => "<!NOTATION",
            Type::SourceFile => "source_file",
            Type::EntityValue => "EntityValue",
            Type::AttValue => "AttValue",
            Type::Sep1 => "Sep1",
            Type::Sep2 => "Sep2",
            Type::Sep3 => "Sep3",
            Type::Text => "Text",
            Type::Pi => "PI",
            Type::TS36 => "_PITarget",
            Type::Prolog => "prolog",
            Type::XmlDecl => "XMLDecl",
            Type::VersionInfo => "VersionInfo",
            Type::TS37 => "_Eq",
            Type::TS38 => "_Misc",
            Type::Doctypedecl => "doctypedecl",
            Type::TS39 => "_DeclSep",
            Type::TS40 => "_intSubset",
            Type::TS41 => "_markupdecl",
            Type::SdDecl => "SDDecl",
            Type::Element => "element",
            Type::STag => "STag",
            Type::Attribute => "Attribute",
            Type::ETag => "ETag",
            Type::TS42 => "_content",
            Type::EmptyElemTag => "EmptyElemTag",
            Type::Elementdecl => "elementdecl",
            Type::Contentspec => "contentspec",
            Type::Children => "children",
            Type::TS43 => "_cp",
            Type::TS44 => "_choice",
            Type::TS45 => "_seq",
            Type::Mixed => "Mixed",
            Type::AttlistDecl => "AttlistDecl",
            Type::TS46 => "_AttDef",
            Type::TS47 => "_AttType",
            Type::TokenizedType => "TokenizedType",
            Type::TS48 => "_EnumeratedType",
            Type::NotationType => "NotationType",
            Type::Enumeration => "Enumeration",
            Type::DefaultDecl => "DefaultDecl",
            Type::TS49 => "_Reference",
            Type::EntityRef => "EntityRef",
            Type::PeReference => "PEReference",
            Type::TS50 => "_EntityDecl",
            Type::GeDecl => "GEDecl",
            Type::PeDecl => "PEDecl",
            Type::TS51 => "_EntityDef",
            Type::TS52 => "_PEDef",
            Type::ExternalId => "ExternalID",
            Type::NDataDecl => "NDataDecl",
            Type::EncodingDecl => "EncodingDecl",
            Type::NotationDecl => "NotationDecl",
            Type::PublicId => "PublicID",
            Type::SourceFileRepeat1 => "source_file_repeat1",
            Type::TS53 => "EntityValue_repeat1",
            Type::TS54 => "AttValue_repeat1",
            Type::TS55 => "AttValue_repeat2",
            Type::TS56 => "Sep1_repeat1",
            Type::TS57 => "Sep2_repeat1",
            Type::TS58 => "Sep3_repeat1",
            Type::TS59 => "Text_repeat1",
            Type::ElementRepeat1 => "element_repeat1",
            Type::TS60 => "STag_repeat1",
            Type::TS61 => "_choice_repeat1",
            Type::TS62 => "_seq_repeat1",
            Type::TS63 => "Mixed_repeat1",
            Type::TS64 => "AttlistDecl_repeat1",
            Type::TS65 => "Enumeration_repeat1",
            Type::Spaces => "Spaces",
            Type::MavenDirectory => "MavenDirectory", // NOTE maven specific
            Type::Directory => "Directory",
            Type::ERROR => "ERROR",
        }
    }
}

const S_T_L: &'static [Type] = &[
    Type::End,
    Type::TS0,
    Type::TS1,
    Type::Name,
    Type::TS2,
    Type::Nmtoken,
    Type::TS3,
    Type::TS4,
    Type::TS5,
    Type::TS6,
    Type::TS7,
    Type::SystemLiteral,
    Type::PubidLiteral,
    Type::CharData,
    Type::Comment,
    Type::TS8,
    Type::TS9,
    Type::CdSect,
    Type::TS10,
    Type::Version,
    Type::Eq,
    Type::VersionNum,
    Type::TS11,
    Type::LBracket,
    Type::RBracket,
    Type::GT,
    Type::Standalone,
    Type::Yes,
    Type::No,
    Type::LT,
    Type::TS12,
    Type::TS13,
    Type::TS14,
    Type::TS15,
    Type::TS16,
    Type::QMark,
    Type::Star,
    Type::Plus,
    Type::LParen,
    Type::Pipe,
    Type::RParen,
    Type::Comma,
    Type::TS17,
    Type::TS18,
    Type::TS19,
    Type::StringType,
    Type::TS20,
    Type::TS21,
    Type::TS22,
    Type::TS23,
    Type::TS24,
    Type::TS25,
    Type::TS26,
    Type::TS27,
    Type::TS28,
    Type::TS29,
    Type::TS30,
    Type::CharRef,
    Type::Amp,
    Type::SemiColon,
    Type::Percent,
    Type::TS31,
    Type::TS32,
    Type::TS33,
    Type::TS34,
    Type::Encoding,
    Type::EncName,
    Type::TS35,
    Type::SourceFile,
    Type::EntityValue,
    Type::AttValue,
    Type::Sep1,
    Type::Sep2,
    Type::Sep3,
    Type::Text,
    Type::Pi,
    Type::TS36,
    Type::Prolog,
    Type::XmlDecl,
    Type::VersionInfo,
    Type::TS37,
    Type::TS38,
    Type::Doctypedecl,
    Type::TS39,
    Type::TS40,
    Type::TS41,
    Type::SdDecl,
    Type::Element,
    Type::STag,
    Type::Attribute,
    Type::ETag,
    Type::TS42,
    Type::EmptyElemTag,
    Type::Elementdecl,
    Type::Contentspec,
    Type::Children,
    Type::TS43,
    Type::TS44,
    Type::TS45,
    Type::Mixed,
    Type::AttlistDecl,
    Type::TS46,
    Type::TS47,
    Type::TokenizedType,
    Type::TS48,
    Type::NotationType,
    Type::Enumeration,
    Type::DefaultDecl,
    Type::TS49,
    Type::EntityRef,
    Type::PeReference,
    Type::TS50,
    Type::GeDecl,
    Type::PeDecl,
    Type::TS51,
    Type::TS52,
    Type::ExternalId,
    Type::NDataDecl,
    Type::EncodingDecl,
    Type::NotationDecl,
    Type::PublicId,
    Type::SourceFileRepeat1,
    Type::TS53,
    Type::TS54,
    Type::TS55,
    Type::TS56,
    Type::TS57,
    Type::TS58,
    Type::TS59,
    Type::ElementRepeat1,
    Type::TS60,
    Type::TS61,
    Type::TS62,
    Type::TS63,
    Type::TS64,
    Type::TS65,
    Type::Spaces,
    Type::MavenDirectory,
    Type::Directory,
    Type::ERROR,
];
