#![feature(iter_collect_into)]
#![feature(drain_filter)]
mod macr;
mod preprocess;
mod render_fdg;
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
use preprocess::{TypeSys, consider_highlights};
use serde::Deserialize;
use tree_sitter::Language;

use crate::{
    macr::{get_language, get_language_name, Lang},
    preprocess::{get_token_hierarchy, TsType, consider_tags},
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

#[macroquad::main(window_conf)]
async fn main() -> std::io::Result<()> {
    use macroquad::prelude::*;
    println!("Screen {}x{}", screen_width(), screen_height());
    let types = preprocess()?;
    let graph = render_fdg_custom::init_graph(types);
    render_macroquad::live(graph).await;
    Ok(())
}

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
//     let tags = r#"
// (new_expression
//     constructor: (identifier) @name) @reference.class
// (new_expression
//     constructor: (identifier) @name1) @reference1.class
// (new_expression
//     constructor: (identifier) @name2) @reference2.class
// (
// (identifier) @constant
// (#match? @constant "^[A-Z][A-Z_]+")
// )
//     "#;
//     let query = tree_sitter::Query::new(lang.get_language(), tags);
//     let q = &query.as_ref().unwrap().capture_names()[6];
//     dbg!(q);
//     let q = query.as_ref().unwrap().capture_index_for_name(q).unwrap();
//     dbg!(q);
//     let q = &query.as_ref().unwrap().capture_quantifiers(3);
//     dbg!(q);
//     let q = &query.as_ref().unwrap().start_byte_for_pattern(3);
//     dbg!(q);
//     dbg!(query);

    let tags = tree_sitter_typescript::TAGGING_QUERY;
    let hi = tree_sitter_typescript::HIGHLIGHT_QUERY;
    let sgraph = r#"
    (new_expression
        constructor: (identifier) @name) @reference.class {}
    (new_expression
        constructor: (identifier) @name1) @reference1.class {}
    "#;
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
    let mut file = File::create(path)?;
    let names = get_token_names(&language, false);
    let mut typesys = get_token_hierarchy(types, false);

    if let Some(tags) = tags {
        consider_tags(tags, &mut typesys);
    }
    if let Some(hi) = hi {
        consider_highlights(hi, &mut typesys);
    }
    Ok(typesys)
}

// mod deserialize_query;
