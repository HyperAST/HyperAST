use super::*;

macro_rules! mk_enum {
    ( $( $camel:ident ),* ) => {
        #[derive(Clone, Debug, PartialEq)]
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
    ( $( ($camel:ident, $name:ident) ),* ) => {
        pub fn get_language(lang: &Lang) -> Language {
            match lang {
                Lang::Kotlin => tree_sitter_kotlin::language(),
                Lang::Java => tree_sitter_java::language(),
                Lang::Typescript => tree_sitter_typescript::language_typescript(),
                // Lang::Tsx => tree_sitter_typescript::language_tsx(),
                // Lang::Javascript => tree_sitter_javascript::language(),
                Lang::Python => tree_sitter_python::language(),
                Lang::Rust => tree_sitter_rust::language(),
                // Lang::Preproc => tree_sitter_preproc::language(),
                // Lang::Ccomment => tree_sitter_ccomment::language(),
                Lang::Cpp => tree_sitter_cpp::language(),
                // Lang::Cpp => tree_sitter_mozcpp::language(),
                // Lang::Mozjs => tree_sitter_mozjs::language(),
                Lang::Xml => tree_sitter_xml::language(),
            }
        }
    };
}

macro_rules! mk_get_language_name {
    ( $( $camel:ident ),* ) => {
        pub fn get_language_name(lang: &Lang) -> &'static str {
            match lang {
                $(
                    Lang::$camel => stringify!($camel),
                )*
            }
        }
    };
}

macro_rules! mk_langs {
    ( $( ($camel:ident, $name:ident) ),* $(,)?) => {
        mk_enum!($( $camel ),*);
        mk_get_language!($( ($camel, $name) ),*);
        mk_get_language_name!($( $camel ),*);
    };
}

mk_langs!(
    // 1) Name for enum 2) tree-sitter function to call to get a Language
    (Kotlin, tree_sitter_kotlin),
    (Java, tree_sitter_java),
    (Rust, tree_sitter_rust),
    (Cpp, tree_sitter_cpp),
    (Python, tree_sitter_python),
    // (Tsx, tree_sitter_tsx),
    // (Ccomment, tree_sitter_ccomment),
    // (Preproc, tree_sitter_preproc),
    // (Mozjs, tree_sitter_mozjs),
    // (Javascript, tree_sitter_javascript),
    (Typescript, tree_sitter_typescript),
    (Xml, tree_sitter_xml),
);


impl Lang {
    pub fn get_language(&self) -> Language {
        get_language(self)
    }
}