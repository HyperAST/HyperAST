pub mod generate_types;
mod keywords;
pub mod preprocess;
pub mod ts_metadata;

use std::{ops::Not, path::Path};

use preprocess::{consider_highlights, TypeSys};

use crate::preprocess::{consider_tags, TsType};

pub trait LanguageCompo {
    fn language(&self) -> tree_sitter::Language;
    fn name(&self) -> &str;
    fn node_types(&self) -> &str {
        ""
    }
    fn highlights(&self) -> &str {
        ""
    }
    fn tags(&self) -> &str {
        ""
    }
    fn injects(&self) -> &str {
        ""
    }
}

pub struct Lang {
    pub language: tree_sitter::Language,
    pub name: &'static str,
    pub node_types: &'static str,
    pub highlights: &'static str,
    pub tags: &'static str,
    pub injects: &'static str,
}

impl LanguageCompo for Lang {
    fn language(&self) -> tree_sitter::Language {
        self.language.clone()
    }

    fn name(&self) -> &str {
        self.name
    }
    fn node_types(&self) -> &str {
        self.node_types
    }
    fn highlights(&self) -> &str {
        self.highlights
    }
    fn tags(&self) -> &str {
        self.tags
    }
    fn injects(&self) -> &str {
        self.injects
    }
}

pub fn preprocess_aux(lang: &impl LanguageCompo) -> Result<TypeSys, std::io::Error> {
    dbg!(lang.name());
    let tags = lang.tags();
    let tags = tags.is_empty().not().then_some(tags);
    let hi = lang.highlights();
    let hi = hi.is_empty().not().then_some(hi);
    let _ = lang.injects(); // TODO process injections
    _preprocess_aux(lang.name(), lang.language(), lang.node_types(), tags, hi)
}

fn _preprocess_aux(
    name: &str,
    language: tree_sitter::Language,
    n_types: &str,
    tags: Option<&str>,
    hi: Option<&str>,
) -> Result<TypeSys, std::io::Error> {
    let types: Vec<TsType> = if n_types.is_empty() {
        vec![]
    } else {
        serde_json::from_str(n_types).unwrap()
    };
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
        let tags: ts_metadata::tags::Tags = tags.parse().unwrap();
        println!("{}", tags);
        Some(tags)
    } else {
        None
    };
    let hi = if let Some(hi) = hi {
        let hi: ts_metadata::highlights::HighLights = hi.parse().unwrap();
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
    let c_name = camel_case(name);
    let file_name = format!("{}.rs", file_template.replace('$', &c_name.to_lowercase()));
    // dbg!(&file_name);
    let _path = output.join(file_name);
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

// use enums::{camel_case, get_token_names};

pub fn camel_case(name: impl AsRef<str>) -> String {
    let name = name.as_ref();
    let mut result = String::with_capacity(name.len());
    let mut cap = true;
    for c in name.chars() {
        if c == '_' {
            cap = true;
        } else if cap {
            result.extend(c.to_uppercase().collect::<Vec<char>>());
            cap = false;
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
#[allow(unused)]
mod refl {
    use std::marker::PhantomData;

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
