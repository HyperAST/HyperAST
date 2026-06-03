use hyperast::{
    nodes::{SyntaxSerializer, TextSerializer},
    store::SimpleStores,
    tree_gen::{self, utils_ts},
};

use crate::types::{TStore, Type};

// type CppGen<'store, 'b> = tree_gen::zipped_ts_simp::TsTreeGen<'store, 'store, crate::types::TStore>;

#[test]
fn simple() {
    type CppGen<'store, 'b, More> =
        tree_gen::zipped_ts_simp::TsTreeGen<'store, 'b, TStore, More, true>;
    let mut stores = Default::default();
    let mut md_cache = Default::default();
    let mut r#gen = CppGen::new(&mut stores, &mut md_cache);
    let text = EXAMPLE_SPACING;
    let tree = match utils_ts::tree_sitter_parse(text.as_bytes(), &crate::language()) {
        Ok(t) => t,
        Err(t) => t,
    };
    eprintln!("{}", tree.root_node().to_sexp());
    let name = b"";
    let f = r#gen.generate_file(name, text.as_bytes(), tree.walk());
    assert_eq!(f.local._ty, Type::TranslationUnit);
    dbg!(&f.local.metrics);
}
#[test]
fn simple0() {
    type CppGen<'store, 'b, More> =
        tree_gen::zipped_ts_simp::TsTreeGen<'store, 'store, TStore, More, true>;
    let mut stores = Default::default();
    let mut md_cache = Default::default();
    let mut r#gen = CppGen::new(&mut stores, &mut md_cache);
    let text = EXAMPLE_SPACING;
    let tree = match utils_ts::tree_sitter_parse(text.as_bytes(), &crate::language()) {
        Ok(t) => t,
        Err(t) => t,
    };
    eprintln!("{}", tree.root_node().to_sexp());
    let name = b"";
    let f = r#gen.generate_file(name, text.as_bytes(), tree.walk());
    assert_eq!(f.local._ty, Type::TranslationUnit);
    dbg!(&f.local.metrics);
    let id = f.local.compressed_node;
    println!("{}", SyntaxSerializer::new(&stores, id));
    println!("\n{}", TextSerializer::new(&stores, id));
    assert_eq!(text, TextSerializer::new(&stores, id).to_string());
}

#[test]
fn simple1() {
    use tree_gen::zipped_ts_simp1::TsTreeGen;
    type CppGen<'store, 'b, More> = TsTreeGen<'store, 'store, crate::types::TStore, More, true>;
    let mut stores = Default::default();
    let mut md_cache = Default::default();
    let mut r#gen = CppGen::new(&mut stores, &mut md_cache);
    let text = EXAMPLE_SPACING;
    let tree = match utils_ts::tree_sitter_parse(text.as_bytes(), &crate::language()) {
        Ok(t) => t,
        Err(t) => t,
    };
    eprintln!("{}", tree.root_node().to_sexp());
    let name = b"";
    let f = r#gen.generate_file(name, text.as_bytes(), tree.walk());
    assert_eq!(f.local._ty, Type::TranslationUnit);
    dbg!(&f.local.metrics);
    let id = f.local.compressed_node;
    println!("{}", SyntaxSerializer::new(&stores, id));
    println!("\n{}", TextSerializer::new(&stores, id));
    assert_eq!(text, TextSerializer::new(&stores, id).to_string());
}

#[test]
fn no_goto_parent() {
    use tree_gen::zipped_ts_no_goto_parent::TsTreeGen;
    type CppGen<'store, 'b, More> = TsTreeGen<'store, 'store, crate::types::TStore, More, true>;
    let mut stores = Default::default();
    let mut md_cache = Default::default();
    let mut r#gen = CppGen::new(&mut stores, &mut md_cache);
    let text = EXAMPLE_SPACING;
    let tree = match utils_ts::tree_sitter_parse(text.as_bytes(), &crate::language()) {
        Ok(t) => t,
        Err(t) => t,
    };
    eprintln!("{}", tree.root_node().to_sexp());
    let name = b"";
    let f = r#gen.generate_file(name, text.as_bytes(), tree.walk());
    assert_eq!(f.local._ty, Type::TranslationUnit);
    dbg!(&f.local.metrics);
    let id = f.local.compressed_node;
    println!("{}", SyntaxSerializer::new(&stores, id));
    println!("\n{}", TextSerializer::new(&stores, id));
    assert_eq!(text, TextSerializer::new(&stores, id).to_string());
}

#[test]
fn no_goto_parent_a() {
    use tree_gen::zipped_ts_no_goto_parent_a::TsTreeGen;
    type CppGen<'store, 'b, More> = TsTreeGen<'store, 'store, crate::types::TStore, More, true>;
    let mut stores = Default::default();
    let mut md_cache = Default::default();
    let mut r#gen = CppGen::new(&mut stores, &mut md_cache);
    let text = EXAMPLE_SPACING;
    let tree = match utils_ts::tree_sitter_parse(text.as_bytes(), &crate::language()) {
        Ok(t) => t,
        Err(t) => t,
    };
    eprintln!("{}", tree.root_node().to_sexp());
    let name = b"";
    let f = r#gen.generate_file(name, text.as_bytes(), tree.walk());
    assert_eq!(f.local._ty, Type::TranslationUnit);
    dbg!(&f.local.metrics);
    let id = f.local.compressed_node;
    println!("{}", SyntaxSerializer::new(&stores, id));
    println!("\n{}", TextSerializer::new(&stores, id));
    assert_eq!(text, TextSerializer::new(&stores, id).to_string());
}

#[test]
fn not_simple() {
    let mut stores = SimpleStores::<TStore>::default();
    let mut md_cache = Default::default();
    let mut r#gen = crate::legion::CppTreeGen::new(&mut stores, &mut md_cache);
    let text = EXAMPLE_SPACING;
    let tree = match utils_ts::tree_sitter_parse(text.as_bytes(), &crate::language()) {
        Ok(t) => t,
        Err(t) => t,
    };
    eprintln!("{}", tree.root_node().to_sexp());
    let name = b"";
    let f = r#gen.generate_file(name, text.as_bytes(), tree.walk());
    dbg!(&f.local.metrics);
    let id = f.local.compressed_node;
    println!("{}", SyntaxSerializer::new(&stores, id));
    println!("\n{}", TextSerializer::new(&stores, id));
    assert_eq!(text, TextSerializer::new(&stores, id).to_string());
}

/// the parsing ERROR comes from the second `A()`
static EXAMPLE_SPACING: &str = r#"
class G {};
bool operator==(const G& x, t spec);
namespace d {
class A() A() B {
  B& operator=(const B&) = delete;
};
}  // namespace d
"#;
