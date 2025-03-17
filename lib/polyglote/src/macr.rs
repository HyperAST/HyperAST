use strum_macros::AsRefStr;
use tree_sitter::Language;

use polyglote::LanguageCompo;

macro_rules! mk_enum {
    ( $( $camel:ident ),* ) => {
        #[derive(Clone, Debug, PartialEq, AsRefStr)]
        pub enum Lang {
            $(
                $camel,
            )*
        }
        impl Lang {
            pub fn into_enum_iter() -> impl Iterator<Item=Lang> {
                use Lang::*;
                [$( $camel, )*].into_iter()
            }
        }
    };
}

macro_rules! mk_get_language {
    (@default $camel:ident { $crat:ident, lang: $lang:ident } ) => {
        fn name(&self) -> &str {
            stringify!($camel)
        }
        fn language(&self) -> Language {
            $crat::$lang()
        }
    };
    (@default $camel:ident { $crat:ident, lang_fn: $lang:ident } ) => {
        fn name(&self) -> &str {
            stringify!($camel)
        }
        fn language(&self) -> Language {
            Language::new($crat::$lang)
        }
    };
    (@each $camel:ident { $crat:ident, lang: $lang:ident, $($attrs:tt)* } ) => {
        mk_get_language!{ @default $camel { $crat, lang: $lang } }
        mk_get_language!{ @others $crat, $($attrs)* }
    };
    (@each $camel:ident { $crat:ident, lang_fn: $lang:ident, $($attrs:tt)* } ) => {
        mk_get_language!{ @default $camel { $crat, lang_fn: $lang } }
        mk_get_language!{ @others $crat, $($attrs)* }
    };
    (@each $camel:ident { $crat:ident, $($attrs:tt)* } ) => {
        mk_get_language!{ @default $camel { $crat, lang_fn: LANGUAGE } }
        mk_get_language!{ @others $crat, $($attrs)* }
    };
    (@each $camel:ident { $crat:ident } ) => {
        mk_get_language!{ @default $camel { $crat, lang: language } }
    };
    (@others $crat:ident, ) => {};
    (@others $crat:ident, tags: $tags:ident, $($attrs:tt)* ) => {
        fn tags(&self) -> &str {
            $crat::$tags
        }
        mk_get_language!{@others $crat, $($attrs)* }
    };
    (@others $crat:ident, injects: $injects:ident, $($attrs:tt)* ) => {
        fn injects(&self) -> &str {
            $crat::$injects
        }
        mk_get_language!{@others $crat, $($attrs)* }
    };
    (@others $crat:ident, hi: $hi:ident, $($attrs:tt)* ) => {
        fn highlights(&self) -> &str {
            $crat::$hi
        }
        mk_get_language!{@others $crat, $($attrs)* }
    };
    (@others $crat:ident, n_types: $n_types:ident, $($attrs:tt)* ) => {
        fn node_types(&self) -> &str {
            $crat::$n_types
        }
        mk_get_language!{@others $crat, $($attrs)* }
    };
    ($( $camel:ident { $($attrs:tt)* } ),*) => {
        $(
            pub struct $camel;
            impl LanguageCompo for $camel {
                mk_get_language!{@each $camel { $($attrs)* }}
            }
        )*


        impl LanguageCompo for Lang {
            fn language(&self) -> Language {
                match self {
                    $(Lang::$camel => $camel.language(),)*
                    // Lang::Kotlin => tree_sitter_kotlin::language(),
                    // Lang::Tsx => tree_sitter_typescript::language_tsx(),
                    // Lang::Javascript => tree_sitter_javascript::language(),
                    // Lang::Rust => tree_sitter_rust::language(),
                    // Lang::Preproc => tree_sitter_preproc::language(),
                    // Lang::Ccomment => tree_sitter_ccomment::language(),
                    // Lang::Cpp => tree_sitter_mozcpp::language(),
                    // Lang::Mozjs => tree_sitter_mozjs::language(),
                }
            }

            fn name(&self) -> &str {
                match self {
                    $(Lang::$camel => $camel.name(),)*
                }
            }

            fn tags(&self) -> &str {
                match self {
                    $(Lang::$camel => $camel.tags(),)*
                }
            }

            fn highlights(&self) -> &str {
                match self {
                    $(Lang::$camel => $camel.highlights(),)*
                }
            }

            fn node_types(&self) -> &str {
                match self {
                    $(Lang::$camel => $camel.node_types(),)*
                }
            }
        }
    };
}

macro_rules! mk_langs {
    ( $( $camel:ident {$($rest:tt)*} ),* $(,)?) => {
        mk_enum!($( $camel ),*);
        mk_get_language!($( $camel {$($rest)*}),*);
    };
}

mk_langs!(
    // 1) Name for enum 2) tree-sitter function to call to get a Language
    Query {
        tree_sitter_query,
        lang: language,
        injects: INJECTIONS_QUERY,
        hi: HIGHLIGHTS_QUERY,
        n_types: NODE_TYPES,
    },
    Java {
        tree_sitter_java,
        tags: TAGS_QUERY,
        hi: HIGHLIGHTS_QUERY,
        n_types: NODE_TYPES,
    },
    // Rust {
    //     tree_sitter_rust,
    //     tags: TAGS_QUERY,
    //     hi: HIGHLIGHT_QUERY,
    //     n_types: NODE_TYPES,
    // },
    Cpp {
        tree_sitter_cpp,
        tags: TAGS_QUERY,
        hi: HIGHLIGHT_QUERY,
        n_types: NODE_TYPES,
    },
    C {
        tree_sitter_c,
        tags: TAGS_QUERY,
        hi: HIGHLIGHT_QUERY,
        n_types: NODE_TYPES,
    },
    // Tsx { tree_sitter_tsx},
    // Ccomment { tree_sitter_ccomment},
    // Preproc { tree_sitter_preproc},
    // Mozjs { tree_sitter_mozjs},
    // Javascript { tree_sitter_javascript},
    Xml {
        tree_sitter_xml,
        lang_fn: LANGUAGE_XML,
        hi: XML_HIGHLIGHT_QUERY,
        n_types: XML_NODE_TYPES,
    },
    Typescript {
        tree_sitter_typescript,
        lang_fn: LANGUAGE_TYPESCRIPT,
        tags: TAGS_QUERY,
        hi: HIGHLIGHTS_QUERY,
        n_types: TYPESCRIPT_NODE_TYPES,
    },
    Python {
        tree_sitter_python,
        tags: TAGS_QUERY,
        hi: HIGHLIGHTS_QUERY,
        n_types: NODE_TYPES,
    },
    // Kotlin {
    //     tree_sitter_kotlin,
    //     lang: language,
    //     n_types: NODE_TYPES,
    // },
    TsQuery {
        tree_sitter_query,
        lang: language,
        hi: HIGHLIGHTS_QUERY,
        injects: INJECTIONS_QUERY,
        n_types: NODE_TYPES,
    },
);
