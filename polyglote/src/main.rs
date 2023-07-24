#![feature(iter_collect_into)]
#![feature(extract_if)]
mod macr;
mod preprocess;
// mod render_fdg;
mod render_fdg_custom;
mod render_layout;
mod render_macroquad;
use std::{
    collections::{BTreeMap, HashMap},
    fmt::{Debug, Display},
    fs::File,
    path::Path,
};

use enums::{camel_case, get_token_names};
use hecs::{CommandBuffer, EntityBuilder, World};
// use macroquad::miniquad::conf::{LinuxBackend, Platform};
use preprocess::{consider_highlights, TypeSys};
use serde::Deserialize;
use strum_macros::{AsRefStr, EnumString};
use tree_sitter::Language;

use strum_macros::*;

use crate::{
    macr::{get_language, get_language_name, Lang},
    preprocess::{consider_tags, get_token_hierarchy, TsType},
};

fn window_conf() -> macroquad::prelude::Conf {
    macroquad::prelude::Conf {
        window_title: "3D".to_owned(),
        // high_dpi: true,
        // fullscreen: true,
        sample_count: 4,
        window_height: 2160,
        window_width: 3840,
        window_resizable: true,
        // platform: Platform {
        //     // linux_backend: LinuxBackend::WaylandOnly,
        //     ..Default::default()
        // },
        ..Default::default()
    }
}
#[test]
fn generate_typescript_type_enum() -> std::io::Result<()> {
    let lang = Lang::Typescript;
    let tags = tree_sitter_typescript::TAGGING_QUERY;
    let hi = tree_sitter_typescript::HIGHLIGHT_QUERY;
    let n_types = tree_sitter_typescript::TSX_NODE_TYPES;
    let types = preprocess_aux(n_types, lang, Some(tags), Some(hi))?;
    generate_types::ggg(&types);
    Ok(())
}

#[macroquad::main(window_conf)]
async fn main() -> std::io::Result<()> {
    use macroquad::prelude::*;
    println!("Screen {}x{}", screen_width(), screen_height());
    let mut graph = render_fdg_custom::Graph::default();
    // let types = preprocess()?;

    let lang = Lang::Typescript;
    let tags = tree_sitter_typescript::TAGGING_QUERY;
    let hi = tree_sitter_typescript::HIGHLIGHT_QUERY;
    let n_types = tree_sitter_typescript::TSX_NODE_TYPES;
    let types = preprocess_aux(n_types, lang, Some(tags), Some(hi))?;
    graph.process_types(types);
    let lang = Lang::Xml;
    // let tags = tree_sitter_xml::TAGGING_QUERY;
    // let hi = tree_sitter_xml::HIGHLIGHT_QUERY;
    let n_types = tree_sitter_xml::NODE_TYPES;
    // dbg!(preprocess::get_token_names(&lang.get_language(), false));
    let types = preprocess_aux(n_types, lang, None, None)?;
    generate_types::ggg(&types);
    // todo!();
    let lang = Lang::Java;
    let tags = tree_sitter_java::TAGGING_QUERY;
    let hi = tree_sitter_java::HIGHLIGHT_QUERY;
    let n_types = tree_sitter_java::NODE_TYPES;
    // dbg!(preprocess::get_token_names(&lang.get_language(), false));
    let types = preprocess_aux(n_types, lang, Some(tags), Some(hi))?;
    generate_types::ggg(&types);
    // todo!();
    graph.process_types(types);
    let lang = Lang::Kotlin;
    // let tags = tree_sitter_kotlin::TAGGING_QUERY;
    // let hi = tree_sitter_kotlin::HIGHLIGHT_QUERY;
    let n_types = tree_sitter_kotlin::NODE_TYPES;
    let types = preprocess_aux(n_types, lang, None, None)?;
    graph.process_types(types);
    let lang = Lang::Python;
    let tags = tree_sitter_python::TAGGING_QUERY;
    let hi = tree_sitter_python::HIGHLIGHT_QUERY;
    let n_types = tree_sitter_python::NODE_TYPES;
    let types = preprocess_aux(n_types, lang, Some(tags), Some(hi))?;
    graph.process_types(types);
    let lang = Lang::Rust;
    let tags = tree_sitter_rust::TAGGING_QUERY;
    let hi = tree_sitter_rust::HIGHLIGHT_QUERY;
    let n_types = tree_sitter_rust::NODE_TYPES;
    let types = preprocess_aux(n_types, lang, Some(tags), Some(hi))?;
    graph.process_types(types);

    let lang = Lang::Cpp;
    // let tags = tree_sitter_cpp::TAGGING_QUERY;
    let hi = tree_sitter_cpp::HIGHLIGHT_QUERY;
    // let n_types = tree_sitter_mozcpp::NODE_TYPES;
    let n_types = tree_sitter_cpp::NODE_TYPES;
    // dbg!(preprocess::get_token_names(&lang.get_language(), false));
    let types = preprocess_aux(n_types, lang, None, Some(hi))?;
    dbg!(&types);
    generate_types::ggg(&types);
    // todo!();
    graph.process_types(types);
    let graph = graph.into();
    render_macroquad::live(graph).await;
    Ok(())
}

mod refl {
    use std::{any::Any, marker::PhantomData};

    struct Class {}
    struct Identifier {}
    struct GenericTypeIdentifier {}
    struct Node<P, C> {
        p: PhantomData<(P, C)>,
    }

    impl Node<Class, Identifier> {
        fn get_name_node<T>(&self) -> Node<Identifier, T> {
            todo! {}
        }
    }

    impl Node<Class, GenericTypeIdentifier> {
        fn get_name_node<T>(&self) -> Node<GenericTypeIdentifier, T> {
            todo! {}
        }
    }

    impl Node<Class, Identifier> {
        fn get_name(&self) -> String {
            todo! {}
        }
    }

    impl Node<Class, GenericTypeIdentifier> {
        fn get_name(&self) -> String {
            todo! {}
        }
    }
}

#[derive(Debug, EnumString, AsRefStr, EnumIter, EnumCount, Display)]
#[strum(serialize_all = "snake_case")]
#[derive(Hash, Clone, Copy, PartialEq, Eq)]
enum JavaKeyword {
    // Java::LBRACE => "{",
    #[strum(serialize = "{")]
    LBrace,
    // Java::RBRACE => "}",
    #[strum(serialize = "}")]
    RBrace,
    // Java::LPAREN => "(",
    #[strum(serialize = "(")]
    LParen,
    // Java::RPAREN => ")",
    #[strum(serialize = ")")]
    RParen,
    // Java::LBRACK => "[",
    #[strum(serialize = "[")]
    LBracket,
    // Java::RBRACK => "]",
    #[strum(serialize = "]")]
    RBracket,
    // Java::SEMI => ";",
    #[strum(serialize = ";")]
    SemiColon,
    // Java::COMMA => ",",
    #[strum(serialize = ",")]
    Comma,
    // Java::DOT => ".",
    #[strum(serialize = ".")]
    Dot,
    // Java::PLUS => "+",
    #[strum(serialize = "+")]
    Plus,
    // Java::DASH => "-",
    #[strum(serialize = "-")]
    Dash,
    // Java::STAR => "*",
    #[strum(serialize = "*")]
    Star,
    // Java::SLASH => "/",
    #[strum(serialize = "/")]
    Slash,
    // Java::PERCENT => "%",
    #[strum(serialize = "%")]
    Percent,
    // Java::BANG => "!",
    #[strum(serialize = "!")]
    Bang,
    // Java::GT => ">",
    #[strum(serialize = ">")]
    GT,
    // Java::LT => "<",
    #[strum(serialize = "<")]
    LT,
    // Java::GTEQ => ">=",
    #[strum(serialize = ">=")]
    GTEq,
    // Java::LTEQ => "<=",
    #[strum(serialize = "<=")]
    LTEq,
    // Java::EQEQ => "==",
    #[strum(serialize = "==")]
    EqEq,
    // Java::BANGEQ => "!=",
    #[strum(serialize = "!=")]
    BangEq,
    // Java::AMPAMP => "&&",
    #[strum(serialize = "&&")]
    AmpAmp,
    // Java::PIPEPIPE => "||",
    #[strum(serialize = "||")]
    PipePipe,
    // Java::QMARK => "?",
    #[strum(serialize = "?")]
    QMark,
    // Java::COLON => ":",
    #[strum(serialize = ":")]
    Colon,
    // Java::EQ => "=",
    #[strum(serialize = "=")]
    Eq,
    // Java::PLUSEQ => "+=",
    #[strum(serialize = "+=")]
    PlusEq,
    // Java::DASHEQ => "-=",
    #[strum(serialize = "-=")]
    DashEq,
    // Java::STAREQ => "*=",
    #[strum(serialize = "*=")]
    StarEq,
    // Java::SLASHEQ => "/=",
    #[strum(serialize = "/=")]
    SlashEq,
    // Java::AMPEQ => "&=",
    #[strum(serialize = "&=")]
    AmpEq,
    // Java::PIPEEQ => "|=",
    #[strum(serialize = "|=")]
    PipeEq,
    // Java::CARETEQ => "^=",
    #[strum(serialize = "^=")]
    CaretEq,
    // Java::PERCENTEQ => "%=",
    #[strum(serialize = "%=")]
    PercentEq,
    // Java::LTLTEQ => "<<=",
    #[strum(serialize = "<<=")]
    LtLtEq,
    // Java::GTGTEQ => ">>=",
    #[strum(serialize = ">>=")]
    GtGtEq,
    // Java::GTGTGTEQ => ">>>=",
    #[strum(serialize = ">>>=")]
    GtGtGtEq,
    // Java::AMP => "&",
    #[strum(serialize = "&")]
    Amp,
    // Java::PIPE => "|",
    #[strum(serialize = "|")]
    Pipe,
    // Java::CARET => "^",
    #[strum(serialize = "^")]
    Caret,
    // Java::LTLT => "<<",
    #[strum(serialize = "<<")]
    LtLt,
    // Java::GTGT => ">>",
    #[strum(serialize = ">>")]
    GtGt,
    // Java::GTGTGT => ">>>",
    #[strum(serialize = ">>>")]
    GtGtGt,
    // Java::DASHGT => "->",
    #[strum(serialize = "->")]
    DashGt,
    // Java::TILDE => "~",
    #[strum(serialize = "~")]
    Tilde,
    // Java::PLUSPLUS => "++",
    #[strum(serialize = "++")]
    PlusPlus,
    // Java::DASHDASH => "--",
    #[strum(serialize = "--")]
    DashDash,
    // Java::AT => "@",
    #[strum(serialize = "@")]
    At,
    // Java::COLONCOLON => "::",
    #[strum(serialize = "::")]
    ColonColon,
    True,
    False,
    New,
    Instanceof,
    Final,
    Class,
    Extends,
    Switch,
    Case,
    Default,
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
    End,
    // # Concrete
    // Java::Identifier => "identifier",
    // Java::DecimalIntegerLiteral => "decimal_integer_literal",
    // Java::HexIntegerLiteral => "hex_integer_literal",
    // Java::OctalIntegerLiteral => "octal_integer_literal",
    // Java::BinaryIntegerLiteral => "binary_integer_literal",
    // Java::DecimalFloatingPointLiteral => "decimal_floating_point_literal",
    // Java::HexFloatingPointLiteral => "hex_floating_point_literal",
    // Java::CharacterLiteral => "character_literal",
    // Java::StringLiteral => "string_literal",
    // Java::TextBlock => "text_block",
    // Java::NullLiteral => "null_literal",
}
#[derive(Debug, EnumString, AsRefStr, EnumIter, EnumCount, Display)]
#[strum(serialize_all = "snake_case")]
#[derive(Hash, Clone, Copy, PartialEq, Eq)]
enum CppKeyword {
    // Cpp::LBRACE => "{",
    #[strum(serialize = "{")]
    LBrace,
    // Cpp::RBRACE => "}",
    #[strum(serialize = "}")]
    RBrace,
    // Cpp::LPAREN => "(",
    #[strum(serialize = "(")]
    LParen,
    // Cpp::RPAREN => ")",
    #[strum(serialize = ")")]
    RParen,
    // Cpp::LPAREN2 => "(", // ?
    // Cpp::LBRACK => "[",
    #[strum(serialize = "[")]
    LBracket,
    // Cpp::RBRACK => "]",
    #[strum(serialize = "]")]
    RBracket,
    // Cpp::SEMI => ";",
    #[strum(serialize = ";")]
    SemiColon,
    // Cpp::COMMA => ",",
    #[strum(serialize = ",")]
    Comma,
    // Cpp::DOT => ".",
    #[strum(serialize = ".")]
    Dot,

    // Cpp::PLUS => "+",
    #[strum(serialize = "+")]
    Plus,
    // Cpp::DASH => "-",
    #[strum(serialize = "-")]
    Dash,
    // Cpp::STAR => "*",
    #[strum(serialize = "*")]
    Star,
    // Cpp::SLASH => "/",
    #[strum(serialize = "/")]
    Slash,
    // Cpp::PERCENT => "%",
    #[strum(serialize = "%")]
    Percent,
    // Cpp::BANG => "!",
    #[strum(serialize = "!")]
    Bang,
    // Cpp::GT => ">",
    #[strum(serialize = ">")]
    GT,
    // Cpp::LT => "<",
    #[strum(serialize = "<")]
    LT,
    // Cpp::GTEQ => ">=",
    #[strum(serialize = ">=")]
    GTEq,
    // Cpp::LTEQ => "<=",
    #[strum(serialize = "<=")]
    LTEq,
    // Cpp::EQEQ => "==",
    #[strum(serialize = "==")]
    EqEq,
    // Cpp::BANGEQ => "!=",
    #[strum(serialize = "!=")]
    BangEq,
    // Cpp::AMPAMP => "&&",
    #[strum(serialize = "&&")]
    AmpAmp,
    // Cpp::PIPEPIPE => "||",
    #[strum(serialize = "||")]
    PipePipe,

    // Cpp::TILDE => "~",
    #[strum(serialize = "~")]
    Tilde,
    // Cpp::AMP => "&",
    #[strum(serialize = "&")]
    Amp,
    // Cpp::PIPE => "|",
    #[strum(serialize = "|")]
    Pipe,
    // Cpp::CARET => "^",
    #[strum(serialize = "^")]
    Caret,
    // Cpp::LTLT => "<<",
    #[strum(serialize = "<<")]
    LtLt,
    // Cpp::GTGT => ">>",
    #[strum(serialize = ">>")]
    GtGt,
    // Cpp::QMARK => "?",
    #[strum(serialize = "?")]
    QMark,
    // Cpp::COLON => ":",
    #[strum(serialize = ":")]
    Colon,
    // Cpp::EQ => "=",
    #[strum(serialize = "=")]
    Eq,
    // Cpp::PLUSEQ => "+=",
    #[strum(serialize = "+=")]
    PlusEq,
    // Cpp::DASHEQ => "-=",
    #[strum(serialize = "-=")]
    DashEq,
    // Cpp::STAREQ => "*=",
    #[strum(serialize = "*=")]
    StarEq,
    // Cpp::SLASHEQ => "/=",
    #[strum(serialize = "/=")]
    SlashEq,
    // Cpp::AMPEQ => "&=",
    #[strum(serialize = "&=")]
    AmpEq,
    // Cpp::PIPEEQ => "|=",
    #[strum(serialize = "|=")]
    PipeEq,
    // Cpp::CARETEQ => "^=",
    #[strum(serialize = "^=")]
    CaretEq,
    // Cpp::PERCENTEQ => "%=",
    #[strum(serialize = "%=")]
    PercentEq,
    // Cpp::LTLTEQ => "<<=",
    #[strum(serialize = "<<=")]
    LtLtEq,
    // Cpp::GTGTEQ => ">>=",
    #[strum(serialize = ">>=")]
    GtGtEq,
    // Java::DASHGT => "->",
    #[strum(serialize = "->")]
    DashGt,
    // Cpp::PLUSPLUS => "++",
    #[strum(serialize = "++")]
    PlusPlus,
    // Cpp::DASHDASH => "--",
    #[strum(serialize = "--")]
    DashDash,
    // Java::AT => "@",
    #[strum(serialize = "@")]
    At,
    // Cpp::COLONCOLON => "::",
    #[strum(serialize = "::")]
    ColonColon,

    // Cpp::DOTDOTDOT => "...",
    #[strum(serialize = "...")]
    DotDotDot,
    // Cpp::DASHGTSTAR => "->*",
    #[strum(serialize = "->*")]
    DashGtStar,

    // Cpp::HASHif => "#if",
    #[strum(serialize = "#if")]
    HashIf,
    // Cpp::HASHinclude => "#include",
    #[strum(serialize = "#include")]
    HashInclude,
    // Cpp::HASHdefine => "#define",
    #[strum(serialize = "#define")]
    HashDefine,
    // Cpp::HASHendif => "#endif",
    #[strum(serialize = "#endif")]
    HashEndif,
    // Cpp::HASHifdef => "#ifdef",
    #[strum(serialize = "#ifdef")]
    HashIfdef,
    // Cpp::HASHifndef => "#ifndef",
    #[strum(serialize = "#ifndef")]
    HashIfndef,
    // Cpp::HASHelse => "#else",
    #[strum(serialize = "#else")]
    HashElse,
    // Cpp::HASHelif => "#elif",
    #[strum(serialize = "#elif")]
    HashElif,
    // Cpp::LSQUOTE => "L'",
    // Cpp::USQUOTE => "u'",
    // Cpp::USQUOTE2 => "U'",
    // Cpp::U8SQUOTE => "u8'",
    // Cpp::SQUOTE => "'",
    // Cpp::CharLiteralToken1 => "char_literal_token1",
    // Cpp::LDQUOTE => "L\"",
    // Cpp::UDQUOTE => "u\"",
    // Cpp::UDQUOTE2 => "U\"",
    // Cpp::U8DQUOTE => "u8\"",
    // Cpp::LBRACKLBRACK => "[[",
    // Cpp::RBRACKRBRACK => "]]",
    // Cpp::LPARENRPAREN => "()",
    // Cpp::LBRACKRBRACK => "[]",
    // Cpp::DQUOTE => "\"",
    // Cpp::DQUOTEDQUOTE => "\"\"",
    // Cpp::LF => "\n",

    // Cpp::Declspec => "__declspec",
    // Cpp::Based => "__based",
    // Cpp::Cdecl => "__cdecl",
    // Cpp::Clrcall => "__clrcall",
    // Cpp::Stdcall => "__stdcall",
    // Cpp::Fastcall => "__fastcall",
    // Cpp::Thiscall => "__thiscall",
    // Cpp::Vectorcall => "__vectorcall",
    // Cpp::Attribute2 => "__attribute__",
    // Cpp::Unaligned => "_unaligned",
    // Cpp::Unaligned2 => "__unaligned",

    // Cpp::MsRestrictModifier => "ms_restrict_modifier",
    // Cpp::MsUnsignedPtrModifier => "ms_unsigned_ptr_modifier",
    // Cpp::MsSignedPtrModifier => "ms_signed_ptr_modifier",

    // Cpp::End => "end",
    // Cpp::True => "true",
    // Cpp::False => "false",
    // Cpp::Try => "try",
    // Cpp::Catch => "catch",
    // Cpp::New => "new",
    // Cpp::This => "this",
    // Cpp::Enum => "enum",
    // Cpp::Class => "class",
    // Cpp::Struct => "struct",
    // Cpp::Union => "union",
    // Cpp::If => "if",
    // Cpp::Else => "else",
    // Cpp::Switch => "switch",
    // Cpp::Case => "case",
    // Cpp::Default => "default",
    // Cpp::While => "while",
    // Cpp::Do => "do",
    // Cpp::For => "for",
    // Cpp::Return => "return",
    // Cpp::Break => "break",
    // Cpp::Continue => "continue",
    // Cpp::Goto => "goto",
    // Cpp::Final => "final",
    // Cpp::Override => "override",
    // Cpp::Virtual => "virtual",
    // Cpp::Explicit => "explicit",
    // Cpp::Public => "public",
    // Cpp::Private => "private",
    // Cpp::Protected => "protected",
    // Cpp::Typedef => "typedef",
    // Cpp::Extern => "extern",
    // Cpp::PreprocDirective => "preproc_directive",
    // Cpp::PreprocArg => "preproc_arg",
    // Cpp::Defined => "defined",
    // Cpp::Static => "static",
    // Cpp::Register => "register",
    // Cpp::Inline => "inline",
    // Cpp::ThreadLocal => "thread_local",
    // Cpp::Const => "const",
    // Cpp::Volatile => "volatile",
    // Cpp::Restrict => "restrict",
    // Cpp::Atomic => "_Atomic",
    // Cpp::Mutable => "mutable",
    // Cpp::Constexpr => "constexpr",
    // Cpp::Signed => "signed",
    // Cpp::Unsigned => "unsigned",
    // Cpp::Long => "long",
    // Cpp::Short => "short",
    // Cpp::PrimitiveType => "primitive_type",
    // Cpp::NumberLiteral => "number_literal",
    // Cpp::StringLiteralToken1 => "string_literal_token1",
    // Cpp::EscapeSequence => "escape_sequence",
    // Cpp::SystemLibString => "system_lib_string",
    // Cpp::Sizeof => "sizeof",
    // Cpp::Null => "null",
    // Cpp::Decltype2 => "decltype",
    // Cpp::Auto => "auto",
    // Cpp::Typename => "typename",
    // Cpp::Template => "template",
    // Cpp::GT2 => ">",
    // Cpp::Operator => "operator",
    // Cpp::Delete => "delete",
    // Cpp::Friend => "friend",
    // Cpp::Noexcept2 => "noexcept",
    // Cpp::Throw => "throw",
    // Cpp::Namespace => "namespace",
    // Cpp::Using => "using",
    // Cpp::StaticAssert => "static_assert",
    // Cpp::CoReturn => "co_return",
    // Cpp::CoYield => "co_yield",
    // Cpp::CoAwait => "co_await",
    // Cpp::Nullptr => "nullptr",

    // # Concrete
    // Cpp::Identifier => "identifier",
    // Cpp::Comment => "comment",
}

mod generate_types {
    use std::collections::HashSet;

    use crate::preprocess::{DChildren, Fields, Hidden, MultipleChildren, RequiredChildren};
    use crate::preprocess::{Named, SubTypes};

    use super::*;
    use heck::CamelCase;
    use heck::SnakeCase;
    use proc_macro2::Ident;
    use quote::{__private::TokenStream, format_ident, quote};
    // use syn::{parse_macro_input, DeriveInput};

    pub(super) fn ggg(typesys: &TypeSys) {
        let mut merged = quote! {};
        let mut from_u16 = quote! {};
        let mut cat_from_u16 = quote! {};
        let mut from_str = quote! {};
        let mut to_str = quote! {};
        let mut as_vec_toks = quote! {};
        let mut hidden_toks = quote! {};
        let mut keyword_toks = quote! {};
        let mut concrete_toks = quote! {};
        let mut with_field_toks = quote! {};
        let mut abstract_toks = quote! {};
        let mut alias_dedup = HashMap::<hecs::Entity, Ident>::default();
        let mut leafs = HM::default();
        <JavaKeyword as strum::IntoEnumIterator>::iter().for_each(|x| {
            leafs.unamed.insert(x.to_string(), format!("{:?}", x));
        });
        <CppKeyword as strum::IntoEnumIterator>::iter().for_each(|x| {
            leafs.unamed.insert(x.to_string(), format!("{:?}", x));
        });
        let mut count = 0;

        for (i, e) in typesys.list.iter().enumerate() {
            let i = i as u16;
            let v = typesys.types.entity(*e).unwrap();
            let t = v.get::<&preprocess::T>().unwrap().0.to_string();
            if let Some(kind) = alias_dedup.get(e) {
                from_u16.extend(quote! {
                    #i => Type::#kind,
                });
                continue;
            }

            if !v.get::<&Named>().is_some() {
                // leaf/token
                let camel_case = t.to_camel_case();
                let is_camel2snake_lossy = *t != camel_case.to_snake_case();
                let raw = t.clone();
                let (q, kind) = if is_camel2snake_lossy {
                    let k = leafs.fmt(&t, |k| format!("TS{}", &k.to_camel_case()));
                    let kind = format_ident!("{}", &k);

                    (
                        quote! {
                            // #[strum(serialize = #raw)]
                            #kind(Raw<#raw>),
                        },
                        kind,
                    )
                } else {
                    let kind = format_ident!("{}", &camel_case);

                    (
                        quote! {
                            #kind,
                        },
                        kind,
                    )
                };

                if v.has::<Hidden>() {
                    let kind = format_ident!("{}", &camel_case);
                    hidden_toks.extend(q);
                    cat_from_u16.extend(quote! {
                        #i => TypeEnum::Hidden(Hidden::#kind),
                    });
                    as_vec_toks.extend(quote! {
                        Hidden(#kind),
                    });
                } else {
                    keyword_toks.extend(q);
                    cat_from_u16.extend(quote! {
                        // #i => TypeEnum::Keyword(Keyword::#kind),
                        #i => Type::#kind,
                    });
                    as_vec_toks.extend(quote! {
                        Keyword(#kind),
                    });
                }
                from_u16.extend(quote! {
                    #i => Type::#kind,
                });
                merged.extend(quote! {
                    #kind,
                });
                to_str.extend(quote! {
                    Type::#kind => #raw,
                });
                from_str.extend(quote! {
                    #raw => Type::#kind,
                });
                alias_dedup.insert(*e, kind);
            } else if let Some(st) = v.get::<&SubTypes>() {
                let camel_case = t.to_camel_case();
                let is_camel2snake_lossy = *t != camel_case.to_snake_case();
                let kind = format_ident!("{}", &camel_case);
                let raw = t.clone();
                let mut sub_toks = quote! {};
                for e in &st.0 {
                    let v = typesys.types.entity(*e).unwrap();
                    let t = &v.get::<&preprocess::T>().unwrap().0;
                    let camel_case = t.to_camel_case();
                    let is_camel2snake_lossy = *t != camel_case.to_snake_case();
                    if is_camel2snake_lossy {
                        let kind = if !v.get::<&Named>().is_some() {
                            let k = leafs.fmt(t, |k| format!("TS{}", &k.to_camel_case()));
                            format_ident!("{}", &k)
                        } else {
                            format_ident!("{}", &t.to_camel_case())
                        };
                        sub_toks.extend(quote! {
                            // #[strum(serialize = #raw)]
                            #kind,
                        });
                    } else {
                        let kind = format_ident!("{}", &camel_case);
                        sub_toks.extend(quote! {
                            // #[strum(serialize = #raw)]
                            #kind,
                        });
                    }
                }
                if is_camel2snake_lossy {
                    abstract_toks.extend(quote! {
                        // #[strum(serialize = #raw)]
                        #kind(Raw<#raw>, #sub_toks),
                    });
                } else {
                    abstract_toks.extend(quote! {
                        #kind(#sub_toks),
                    });
                }
                cat_from_u16.extend(quote! {
                    #i => TypeEnum::Abstract(Abstract::#kind),
                });
                as_vec_toks.extend(quote! {
                    Abstract(#kind),
                });

                merged.extend(quote! {
                    #kind,
                });
                from_u16.extend(quote! {
                    #i => Type::#kind,
                });
                to_str.extend(quote! {
                    Type::#kind => #raw,
                });
                from_str.extend(quote! {
                    #raw => Type::#kind,
                });
                alias_dedup.insert(*e, kind);
            } else if let Some(fields) = v.get::<&Fields>() {
                let camel_case = t.to_camel_case();
                let is_camel2snake_lossy = *t != camel_case.to_snake_case();
                let kind = format_ident!("{}", &camel_case);
                let raw = t.clone();
                let mut fields_toks = quote! {};
                for e in &fields.0 {
                    let v = typesys.types.entity(*e).unwrap();
                    let t = &v.get::<&preprocess::Role>().unwrap().0;
                    let camel_case = t.to_camel_case();
                    let is_camel2snake_lossy = *t != camel_case.to_snake_case();
                    dbg!(t);
                    assert_eq!(*t, camel_case.to_snake_case());
                    let t = if t == "type" { "r#type" } else { t };
                    let kind = format_ident!("{}", &t);
                    let cs = &v.get::<&preprocess::DChildren>().unwrap().0;
                    let mut cs_toks = quote! {};
                    for e in cs {
                        let v = typesys.types.entity(*e).unwrap();
                        let t = &v.get::<&preprocess::T>().unwrap().0;
                        let camel_case = t.to_camel_case();
                        let is_camel2snake_lossy = *t != camel_case.to_snake_case();
                        if is_camel2snake_lossy {
                            let kind = if !v.get::<&Named>().is_some() {
                                let k = leafs.fmt(t, |k| format!("TS{}", &k.to_camel_case()));
                                format_ident!("{}", &k)
                            } else {
                                format_ident!("{}", &t.to_camel_case())
                            };
                            cs_toks.extend(quote! {
                                #kind,
                            });
                        } else {
                            let kind = format_ident!("{}", &camel_case);
                            cs_toks.extend(quote! {
                                #kind,
                            });
                        }
                    }
                    if v.has::<RequiredChildren>() {
                        if v.has::<MultipleChildren>() {
                            fields_toks.extend(quote! {
                                #kind:MultReq<(#cs_toks)>,
                            });
                        } else {
                            fields_toks.extend(quote! {
                                #kind:Req<(#cs_toks)>,
                            });
                        }
                    } else if v.has::<MultipleChildren>() {
                        fields_toks.extend(quote! {
                            #kind:Mult<(#cs_toks)>,
                        });
                    } else {
                        fields_toks.extend(quote! {
                            #kind: (#cs_toks),
                        });
                    }
                }
                if let Some(cs) = v.get::<&preprocess::DChildren>() {
                    let mut cs_toks = quote! {};
                    for e in &cs.0 {
                        let v = typesys.types.entity(*e).unwrap();
                        let t = &v.get::<&preprocess::T>().unwrap().0;
                        let camel_case = t.to_camel_case();
                        let is_camel2snake_lossy = *t != camel_case.to_snake_case();
                        if is_camel2snake_lossy {
                            let kind = if !v.get::<&Named>().is_some() {
                                let k = leafs.fmt(t, |k| format!("TS{}", &k.to_camel_case()));
                                format_ident!("{}", &k)
                            } else {
                                format_ident!("{}", &t.to_camel_case())
                            };
                            cs_toks.extend(quote! {
                                #kind,
                            });
                        } else {
                            let kind = format_ident!("{}", &camel_case);
                            cs_toks.extend(quote! {
                                #kind,
                            });
                        }
                    }
                    // fields_toks.extend(quote! {
                    //     _cs:(#cs_toks),
                    // });

                    if v.has::<RequiredChildren>() {
                        if v.has::<MultipleChildren>() {
                            fields_toks.extend(quote! {
                                _cs:MultReq<(#cs_toks)>,
                            });
                        } else {
                            fields_toks.extend(quote! {
                                _cs:Req<(#cs_toks)>,
                            });
                        }
                    } else if v.has::<MultipleChildren>() {
                        fields_toks.extend(quote! {
                            _cs:Mult<(#cs_toks)>,
                        });
                    } else {
                        fields_toks.extend(quote! {
                            _cs: (#cs_toks),
                        });
                    }
                }
                if is_camel2snake_lossy {
                    with_field_toks.extend(quote! {
                        // #[strum(serialize = #raw)]
                        #kind{_ser: Raw<#raw>, #fields_toks},
                    });
                } else {
                    with_field_toks.extend(quote! {
                        #kind{#fields_toks},
                    });
                }
                cat_from_u16.extend(quote! {
                    #i => TypeEnum::WithFields(WithFields::#kind),
                });
                as_vec_toks.extend(quote! {
                    WithFields(#kind),
                });

                merged.extend(quote! {
                    #kind,
                });
                from_u16.extend(quote! {
                    #i => Type::#kind,
                });
                to_str.extend(quote! {
                    Type::#kind => #raw,
                });
                from_str.extend(quote! {
                    #raw => Type::#kind,
                });
                alias_dedup.insert(*e, kind);
            } else if let Some(cs) = v.get::<&DChildren>() {
                let camel_case = t.to_camel_case();
                let is_camel2snake_lossy = *t != camel_case.to_snake_case();
                let kind = format_ident!("{}", &camel_case);
                let raw = t.clone();
                let mut cs_toks = quote! {};
                for e in &cs.0 {
                    let v = typesys.types.entity(*e).unwrap();
                    let t = &v.get::<&preprocess::T>().unwrap().0;
                    let camel_case = t.to_camel_case();
                    let is_camel2snake_lossy = *t != camel_case.to_snake_case();
                    if is_camel2snake_lossy {
                        let kind = if !v.get::<&Named>().is_some() {
                            let k = leafs.fmt(t, |k| format!("TS{}", &k.to_camel_case()));
                            format_ident!("{}", &k)
                        } else {
                            format_ident!("{}", &t.to_camel_case())
                        };
                        cs_toks.extend(quote! {
                            #kind,
                        });
                    } else {
                        let kind = format_ident!("{}", &camel_case);
                        cs_toks.extend(quote! {
                            #kind,
                        });
                    }
                }
                let cs_toks = if v.has::<RequiredChildren>() {
                    if v.has::<MultipleChildren>() {
                        quote! {
                            MultReq<(#cs_toks)>,
                        }
                    } else {
                        quote! {
                            Req<(#cs_toks)>,
                        }
                    }
                } else if v.has::<MultipleChildren>() {
                    quote! {
                        Mult<(#cs_toks)>,
                    }
                } else {
                    quote! {
                        #cs_toks
                    }
                };
                if is_camel2snake_lossy {
                    concrete_toks.extend(quote! {
                        // #[strum(serialize = #raw)]
                        #kind(Raw<#raw>,#cs_toks),
                    });
                } else {
                    concrete_toks.extend(quote! {
                        #kind(#cs_toks),
                    });
                }
                cat_from_u16.extend(quote! {
                    #i => TypeEnum::Concrete(Concrete::#kind),
                });
                as_vec_toks.extend(quote! {
                    Concrete(#kind),
                });

                merged.extend(quote! {
                    #kind,
                });
                from_u16.extend(quote! {
                    #i => Type::#kind,
                });
                to_str.extend(quote! {
                    Type::#kind => #raw,
                });
                from_str.extend(quote! {
                    #raw => Type::#kind,
                });
                alias_dedup.insert(*e, kind);
            } else {
                let camel_case = t.to_camel_case();
                let is_camel2snake_lossy = *t != camel_case.to_snake_case();
                let kind = format_ident!("{}", &camel_case);
                let raw = t.clone();
                if is_camel2snake_lossy {
                    concrete_toks.extend(quote! {
                        // #[strum(serialize = #raw)]
                        #kind(Raw<#raw>),
                    });
                } else {
                    concrete_toks.extend(quote! {
                        #kind,
                    });
                }
                cat_from_u16.extend(quote! {
                    #i => TypeEnum::Concrete(Concrete::#kind),
                });
                as_vec_toks.extend(quote! {
                    Concrete(#kind),
                });
                if v.has::<Hidden>() {
                    panic!();
                }

                merged.extend(quote! {
                    #kind,
                });
                from_u16.extend(quote! {
                    #i => Type::#kind,
                });
                to_str.extend(quote! {
                    Type::#kind => #raw,
                });
                from_str.extend(quote! {
                    #raw => Type::#kind,
                });
                alias_dedup.insert(*e, kind);
            }
            // let v = self.abstract_types.entity(*e).unwrap();
            // writeln!(f, "{:?}: {:?}", t, e)?;
            // if v.get::<&Named>().is_some() {
            //     writeln!(f, "\tnamed")?;
            // }
            // if let Some(st) = v.get::<&SubTypes>() {
            //     writeln!(f, "\tsubtypes: {:?}", st.0)?;
            // }
            // if let Some(fi) = v.get::<&Fields>() {
            //     writeln!(f, "\tfields: {:?}", fi.0)?;
            // }
            // if let Some(cs) = v.get::<&DChildren>() {
            //     writeln!(f, "\tchildren: {:?}", cs.0)?;
            // }
            count += 1;
        }

        let count = typesys.list.len();
        let len = typesys.list.len() as u16;

        let res = quote! {
            enum TypeEnum {
                Keyword(Keyword),
                Concrete(Concrete),
                WithFields(WithFields),
                Abstract(Abstract),
                Hidden(Hidden),
                OutOfBound,
            }
            enum Hidden {
                #hidden_toks
            }
            enum Keyword {
                #keyword_toks
            }
            /// Type of nodes actually stored
            /// ie. what should be stored on CST nodes
            /// but anyway encode it as a number
            /// and it would be better to take the smallest numbers for concrete nodes
            /// to facilitate convertion
            enum Concrete {
                #concrete_toks
                // #named_concrete_types_toks
            }
            enum WithFields {
                #with_field_toks
            }
            enum Abstract {
                #abstract_toks
            }
            pub fn from_u16(t: u16) -> TypeResult {
                match t {
                    #cat_from_u16
                    #len => TypeEnum::ERROR
                }
            }
            const COUNT: usize = #count;
            const TS2Enum: &[()] = [
                #as_vec_toks
            ];

            enum Type {
                #merged
                ERROR,
            }
            impl Type {
                pub fn from_u16(t: u16) -> Type {
                    match t {
                        #from_u16
                        #len => Type::ERROR,
                        x => panic!("{}",x),
                    }
                }
                pub fn from_str(t: &str) -> Option<Type> {
                    Some(match t {
                        #from_str
                        "ERROR" => Type::ERROR,
                        x => return None,
                    })
                }
                pub fn to_str(t: Type) -> &'static str {
                    match t {
                        #to_str
                        Type::ERROR => "ERROR",
                    }
                }
            }
            // /// all types
            // enum Types {
            //     #types_toks
            // }
            // impl Types {
            //     // pub fn parse_xml(t: &str) -> Self {
            //     //     match t {
            //     //         #into_types_toks
            //     //     }
            //     // }
            // }
            // mod abstract_types {
            //     #abstract_types_toks
            // }
        };
        println!("{}", res);
        let res = syn::parse_file(&res.to_string()).unwrap();
        let res = prettyplease::unparse(&res);
        println!("{}", res);
    }
    pub(super) fn fff(typesys: &TypeSys) {
        let mut concrete_types_toks = quote! {};
        let mut abstract_types_toks = quote! {};
        let mut types_toks = quote! {};
        let mut into_types_toks = quote! {};
        let mut leafs = HM::default();
        let mut count = 0;

        for (t, e) in &typesys.index {
            let v = typesys.types.entity(*e).unwrap();

            if !v.get::<&Named>().is_some() {
                // leaf/token
                let k = leafs.fmt(t, |k| format!("cpp_TS{}", &k.to_camel_case()));
                let kind = format_ident!("{}", &k);
                let raw = t.clone();

                concrete_types_toks.extend(quote! {
                    #[strum(serialize = #raw)]
                    #kind,
                });
                types_toks.extend(quote! {
                    #[strum(serialize = #raw)]
                    #kind,
                });
                into_types_toks.extend(quote! {
                    #raw => #kind,
                });
            } else if let Some(st) = v.get::<&SubTypes>() {
                let kind = format_ident!("cpp_{}", &t.to_camel_case());
                let raw = t.clone();
                let mut sub_toks = quote! {};
                for e in &st.0 {
                    let v = typesys.types.entity(*e).unwrap();
                    let t = &v.get::<&preprocess::T>().unwrap().0;
                    let kind = format_ident!("{}", &t.to_camel_case());
                    let raw = t.clone();
                    sub_toks.extend(quote! {
                        #[strum(serialize = #raw)]
                        #kind,
                    });
                }
                let ty = quote! {
                    enum #kind {
                        #sub_toks
                    }
                };
                abstract_types_toks.extend(ty);
                types_toks.extend(quote! {
                    #[strum(serialize = #raw)]
                    #kind,
                });
                into_types_toks.extend(quote! {
                    #raw => #kind,
                });
            } else {
                let kind = format_ident!("cpp_{}", &t.to_camel_case());
                let raw = t.clone();
                concrete_types_toks.extend(quote! {
                    #[strum(serialize = #raw)]
                    #kind,
                });
                types_toks.extend(quote! {
                    #[strum(serialize = #raw)]
                    #kind,
                });
                into_types_toks.extend(quote! {
                    #raw => #kind,
                });
            }
            // let v = self.abstract_types.entity(*e).unwrap();
            // writeln!(f, "{:?}: {:?}", t, e)?;
            // if v.get::<&Named>().is_some() {
            //     writeln!(f, "\tnamed")?;
            // }
            // if let Some(st) = v.get::<&SubTypes>() {
            //     writeln!(f, "\tsubtypes: {:?}", st.0)?;
            // }
            // if let Some(fi) = v.get::<&Fields>() {
            //     writeln!(f, "\tfields: {:?}", fi.0)?;
            // }
            // if let Some(cs) = v.get::<&DChildren>() {
            //     writeln!(f, "\tchildren: {:?}", cs.0)?;
            // }
            count += 1;
        }

        let res = quote! {
            /// Type of nodes actually stored
            /// ie. what should be stored on CST nodes
            /// but anyway encode it as a number
            /// and it would be better to take the smallest numbers for concrete nodes
            /// to facilitate convertion
            enum ConcreteTypes {
                #concrete_types_toks
            }
            /// all types
            enum Types {
                #types_toks
            }
            impl Types {
                pub fn parse_xml(t: &str) -> Self {
                    match t {
                        #into_types_toks
                    }
                }
            }
            mod abstract_types {
                #abstract_types_toks
            }
        };
        println!("{}", res);
        let res = syn::parse_file(&res.to_string()).unwrap();
        let res = prettyplease::unparse(&res);
        println!("{}", res);
    }

    #[derive(Default)]
    struct HM {
        unamed: BTreeMap<String, String>,
        esc_c: u32,
    }

    impl HM {
        fn fmt(&mut self, x: &str, f: impl Fn(&str) -> String) -> String {
            if let Some(v) = self.unamed.get(x) {
                v
            } else {
                let value = f(&self.esc_c.to_string());
                self.unamed.insert(x.to_string(), value);
                self.esc_c += 1;
                &self.unamed.get(x).unwrap()
            }
            .to_string()
        }
    }
}

mod aaa {}

// #[macroquad::main(window_conf)]
// async fn main() -> std::io::Result<()> {
//     use fdg_sim::{petgraph::graph::NodeIndex, ForceGraph, ForceGraphHelper};

//     let types = preprocess()?;
//     let graph = render_fdg_custom::init_graph(types);

//     fdg_macroquad::run_window(&graph).await;
//     Ok(())
// }

// fn main() {
//     let types = preprocess().unwrap();
//     // dbg!(types);
// }

fn preprocess() -> Result<TypeSys, std::io::Error> {
    dbg!(env!("PWD"));
    let lang = Lang::Typescript;
    let tags = tree_sitter_typescript::TAGGING_QUERY;
    let hi = tree_sitter_typescript::HIGHLIGHT_QUERY;
    let n_types = tree_sitter_typescript::TSX_NODE_TYPES;
    let types = preprocess_aux(n_types, lang, Some(tags), Some(hi))?;
    let lang = Lang::Java;
    let tags = tree_sitter_java::TAGGING_QUERY;
    let hi = tree_sitter_java::HIGHLIGHT_QUERY;
    let n_types = tree_sitter_java::NODE_TYPES;
    let types = preprocess_aux(n_types, lang, Some(tags), Some(hi))?;
    let lang = Lang::Kotlin;
    // let tags = tree_sitter_kotlin::TAGGING_QUERY;
    // let hi = tree_sitter_kotlin::HIGHLIGHT_QUERY;
    let n_types = tree_sitter_kotlin::NODE_TYPES;
    let types = preprocess_aux(n_types, lang, None, None)?;
    let lang = Lang::Python;
    let tags = tree_sitter_python::TAGGING_QUERY;
    let hi = tree_sitter_python::HIGHLIGHT_QUERY;
    let n_types = tree_sitter_python::NODE_TYPES;
    let types = preprocess_aux(n_types, lang, Some(tags), Some(hi))?;
    let lang = Lang::Rust;
    let tags = tree_sitter_rust::TAGGING_QUERY;
    let hi = tree_sitter_rust::HIGHLIGHT_QUERY;
    let n_types = tree_sitter_rust::NODE_TYPES;
    let types = preprocess_aux(n_types, lang, Some(tags), Some(hi))?;
    let lang = Lang::Cpp;
    // let tags = tree_sitter_cpp::TAGGING_QUERY;
    let hi = tree_sitter_cpp::HIGHLIGHT_QUERY;
    let n_types = tree_sitter_mozcpp::NODE_TYPES;
    let types = preprocess_aux(n_types, lang, None, Some(hi))?;
    // dbg!(&types);
    Ok(types)
}

mod ts_metadata;

fn preprocess_aux(
    n_types: &str,
    lang: Lang,
    tags: Option<&str>,
    hi: Option<&str>,
) -> Result<TypeSys, std::io::Error> {
    let types: Vec<TsType> = serde_json::from_str(n_types).unwrap();
    // let s_graph = tree_sitter_graph::ast::File::from_str(lang.get_language(), tags.unwrap())
    // let s_graph =
    //     tree_sitter::Query::new(lang.get_language(), tags.unwrap()).expect("Cannot parse tags");
    // dbg!(&s_graph);

    // let mut query_parser = tree_sitter::Parser::new();
    // query_parser.set_language(tree_sitter_query::language()).unwrap();
    // // tree_sitter_loader::Loader::load_language_from_sources(&self, name, header_path, parser_path, scanner_path)
    // // tsq.set_language(tree_sitter_graph::parse_error);
    // let tags = query_parser.parse(tags.unwrap(), None).unwrap();
    // dbg!(tags.root_node().to_sexp());
    let tags = if let Some(tags) = tags {
        let tags: ts_metadata::Tags = tags.parse().unwrap();
        println!("{}", tags);
        Some(tags)
    } else {
        None
    };
    let hi = if let Some(hi) = hi {
        let hi: ts_metadata::HighLights = hi.parse().unwrap();
        println!("{}", hi);
        // println!("{:?}", hi.get("type"));
        // println!("{:?}", hi.get("variable"));
        // println!("{:?}", hi.get("variable.builtin"));
        // println!("{:?}", hi.get("variable.*"));
        Some(hi)
    } else {
        None
    };

    // let s_graph =
    //     tree_sitter::Query::new(lang.get_language(), hi.unwrap()).expect("Cannot parse highlights");
    // dbg!(&s_graph);

    // dbg!(&types);
    let output = Path::new("lang_types");
    let file_template = "rust";
    // dbg!(&lang);
    let language = get_language(&lang);
    let name = get_language_name(&lang);
    let c_name = camel_case(name.to_string());
    let file_name = format!("{}.rs", file_template.replace('$', &c_name.to_lowercase()));
    // dbg!(&file_name);
    let path = output.join(file_name);
    // let mut file = File::create(path)?;
    // let names = preprocess::get_token_names(&language, false);
    let mut typesys = TypeSys::new(language, types);

    if let Some(tags) = tags {
        consider_tags(tags, &mut typesys);
    }
    if let Some(hi) = hi {
        consider_highlights(hi, &mut typesys);
    }
    Ok(typesys)
}

// mod deserialize_query;
