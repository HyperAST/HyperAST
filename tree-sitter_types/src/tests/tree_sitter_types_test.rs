use std::path::Path;

use crate::parse_types::{gen_enum_from_ts_json, gen_types_from_ts_json};



#[test]
fn test() {
    gen_types_from_ts_json(&Path::new(
        "/home/quentin/rusted_gumtree/gen/tree-sitter/java/tree-sitter-java/src/node-types.json",
    ),Path::new("out.rs")).unwrap();
}

#[test]
fn test2() {
    gen_enum_from_ts_json(&Path::new(
        "/home/quentin/rusted_gumtree/gen/tree-sitter/java/tree-sitter-java/src/node-types.json",
    ),Path::new("out.rs")).unwrap();
}