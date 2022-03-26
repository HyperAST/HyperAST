use std::path::Path;

use crate::parse_types::{gen_enum_from_ts_json, gen_types_from_ts_json};



#[test]
fn test1() {
    gen_types_from_ts_json(&Path::new(
        "/home/quentin/rusted_gumtree/gen/tree-sitter/java/tree-sitter-java/src/node-types.json",
    ),Path::new("out.rs")).unwrap();
}

#[test]
fn test1_2() {
    gen_types_from_ts_json(&Path::new(
        "/home/quentin/rusted_gumtree/gen/tree-sitter/xml/tree-sitter-xml/src/node-types.json",
    ),Path::new("out.rs")).unwrap();
}

#[test]
fn test2() {
    gen_enum_from_ts_json(&Path::new(
        "/home/quentin/rusted_gumtree/gen/tree-sitter/java/tree-sitter-java/src/node-types.json",
    ),Path::new("out.rs")).unwrap();
}

#[test]
fn test2_2() {
    gen_enum_from_ts_json(&Path::new(
        "/home/quentin/rusted_gumtree/gen/tree-sitter/xml/tree-sitter-xml/src/node-types.json",
    ),Path::new("out.rs")).unwrap();
}