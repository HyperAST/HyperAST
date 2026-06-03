#![allow(unused)] // TODO maintain those tests
use crate::tests::cpp_tree;
use hyperast::nodes::SyntaxSerializer;
use hyperast::nodes::TextSerializer;
use hyperast::{
    position::{TreePath, structural_pos::AAA},
    store::defaults::NodeIdentifier,
    types::Typed,
};

use hyperast_gen_ts_cpp::iter::IterAll as CppIter;

const Q0: &str =
    r#"(binary_expression (_expression (number_literal)) "+" (_expression (number_literal)))"#; // TODO make _expression optional
const C0: &str = r#"int f() {
    return 21 + 21;
}"#;

const C1: &str = r#"int f() {
    return 21 - 21;
}"#;

const C2: &str = r#"int f() {
    int a = 21;
    return a + a;
}"#;
const Q1: &str = r#"(binary_expression (_expression (identifier) @first) "+" (_expression (identifier) @second)) (#eq? @first @second)"#; // TODO make _expression optional

// Possible useful stuff:
// - test if subtree is conforming to ts query
//   - initially for each node in subtree, do the test
//     - terminate on wrong root type as fast as possible
//   - after that find different oracles
//     - type oracle
//     - structure hash oracle
//     - filtered structure hash oracle
//     - other convolutions (including prev hashes)
//     - labels through bags of words and defered bloom filters computing
// - edit distance between query and subtree
// - acceleration related to extracting entropy from basic constructs

// type CppTIdN = hyperast_gen_ts_cpp::types::TIdN<NodeIdentifier>;

// type Cpp = hyperast_gen_ts_cpp::types::Type;

// #[test]
// fn simple() {
//     let (code_store, code) = cpp_tree(C0.as_bytes());
//     let (query_store, query) = crate::search::ts_query(Q0.as_bytes());
//     let path = hyperast::position::StructuralPosition::new(code);
//     let prepared_matcher = crate::search::PreparedMatcher::<Cpp>::new(&query_store, query);
//     let mut matched = false;
//     for e in CppIter::new(&code_store, path, code) {
//         if prepared_matcher.is_matching::<_, CppTIdN>(&code_store, *e.node().unwrap()) {
//             let n = code_store
//                 .node_store
//                 .try_resolve_typed::<CppTIdN>(e.node().unwrap())
//                 .unwrap()
//                 .0;
//             let t = n.get_type();
//             dbg!(t);
//             matched = true;
//         }
//     }
//     assert!(matched);
//     let (code_store1, code1) = cpp_tree(C1.as_bytes());
//     let path = hyperast::position::StructuralPosition::new(code1);
//     let prepared_matcher = crate::search::PreparedMatcher::<Cpp>::new(&query_store, query);
//     for e in CppIter::new(&code_store1, path, code1) {
//         if prepared_matcher.is_matching::<_, CppTIdN>(&code_store1, *e.node().unwrap()) {
//             panic!("should not match")
//         }
//     }
// }

// #[test]
// fn named() {
//     let (code_store, code) = cpp_tree(C2.as_bytes());
//     let (query_store, query) = crate::search::ts_query(Q1.as_bytes());
//     let path = hyperast::position::StructuralPosition::new(code);
//     let prepared_matcher = crate::search::PreparedMatcher::<Cpp>::new(&query_store, query);
//     let mut matched = false;
//     for e in CppIter::new(&code_store, path, code) {
//         if let Some(c) =
//             prepared_matcher.is_matching_and_capture::<_, CppTIdN>(&code_store, *e.node().unwrap())
//         {
//             dbg!(c);
//             matched = true;
//         }
//     }
//     assert!(matched);
// }

#[test]
fn match_xml() {
    use crate::tests::xml_tree;
    use TextSerializer;
    use hyperast::types::WithChildren;
    let path: std::path::PathBuf =
        std::path::Path::new("../../../gen/tree-sitter/xml/src/tests/pom.xml.test").to_path_buf();

    let text = std::fs::read(path).unwrap();
    let (code_store, code) = xml_tree(&text);
    let pat = code;
    let pat = code_store.node_store.resolve(pat).child(&2).unwrap();
    let pat = code_store.node_store.resolve(pat).child(&1).unwrap();
    let deps = code_store.node_store.resolve(pat).child(&19).unwrap();
    let pat = code_store.node_store.resolve(deps).child(&1).unwrap();
    let pat = code_store.node_store.resolve(pat).child(&1).unwrap();
    let pat = code_store.node_store.resolve(pat).child(&1).unwrap();
    let pat = code_store.node_store.resolve(pat).child(&3).unwrap();
    println!(
        "{}",
        &SyntaxSerializer::<_, _, true>::new(&code_store, pat).to_string()[..70]
    );
    println!();
    println!("{}", TextSerializer::new(&code_store, pat));
    let q0 = format!("(element (STag (Name) @id (#eq? @id \"artifactId\")))");
    let (query_store, query) = crate::search::ts_query(q0.as_bytes());
    use hyperast_gen_ts_xml::types::Type as Xml;
    type XmlTIdN = hyperast_gen_ts_xml::types::TIdN<NodeIdentifier>;
    use hyperast_gen_ts_xml::iter::IterAll as XmlIter;
    let path = hyperast::position::StructuralPosition::new(code);
    let prepared_matcher = crate::search::PreparedMatcher::<Xml>::new(&query_store, query);
    for e in XmlIter::new(&code_store, path, code) {
        if prepared_matcher.is_matching::<_, XmlTIdN>(&code_store, *e.node().unwrap()) {
            eprintln!("{}", TextSerializer::new(&code_store, *e.node().unwrap()));
        }
    }
}

#[test]
fn test_new_matcher_for_xml_element() -> Result<(), Box<dyn std::error::Error>> {
    let path: std::path::PathBuf =
        std::path::Path::new("../../../gen/tree-sitter/xml/src/tests/pom.xml.test").to_path_buf();

    let text = std::fs::read(&path).unwrap();
    eprintln!("{}", std::fs::read_to_string(&path).unwrap());
    let (code_store, code) = crate::tests::xml_tree(&text);
    eprintln!("{}", SyntaxSerializer::new(&code_store, code));

    let query = r#"(element) @root"#;
    let qqq = hyperast_tsquery::Query::new(query, hyperast_gen_ts_xml::language())
        .map_err(|e| e.to_string())?;
    let pos = hyperast::position::structural_pos::CursorWithPersistance::new(code);
    let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(&code_store, pos);
    let root_cap = qqq.capture_index_for_name("root").unwrap();
    let qcursor = qqq.matches(cursor);
    for m in qcursor {
        let pid = m.pattern_index;
        let i = qqq.enabled_pattern_index(pid).unwrap();
        assert_eq!(i, 0);
        dbg!(pid, i);
        let mut root_cap = m.nodes_for_capture_index(root_cap);
        let root_cap = root_cap.next().unwrap().pos.node();
        eprintln!("{}", SyntaxSerializer::new(&code_store, root_cap));
        eprintln!("{}", TextSerializer::new(&code_store, root_cap));
    }
    Ok(())
}

#[test]
fn test_new_matcher_for_xml_eq() -> Result<(), Box<dyn std::error::Error>> {
    let path: std::path::PathBuf =
        std::path::Path::new("../../../gen/tree-sitter/xml/src/tests/pom.xml.test").to_path_buf();

    let text = std::fs::read(&path).unwrap();
    eprintln!("{}", std::fs::read_to_string(&path).unwrap());
    let (code_store, code) = crate::tests::xml_tree(&text);
    eprintln!("{}", SyntaxSerializer::new(&code_store, code));

    let query = r#"(element (STag (Name) @id (#eq? @id "artifactId"))) @root"#;
    let qqq = hyperast_tsquery::Query::new(query, hyperast_gen_ts_xml::language())
        .map_err(|e| e.to_string())?;
    let pos = hyperast::position::structural_pos::CursorWithPersistance::new(code);
    let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(&code_store, pos);
    let root_cap = qqq.capture_index_for_name("root").unwrap();
    let qcursor = qqq.matches(cursor);
    for m in qcursor {
        let pid = m.pattern_index;
        let i = qqq.enabled_pattern_index(pid).unwrap();
        assert_eq!(i, 0);
        dbg!(pid, i);
        let mut root_cap = m.nodes_for_capture_index(root_cap);
        let root_cap = root_cap.next().unwrap().pos.node();
        eprintln!("{}", SyntaxSerializer::new(&code_store, root_cap));
        eprintln!("{}", TextSerializer::new(&code_store, root_cap));
    }
    Ok(())
}

#[test]
fn test_new_matcher_for_xml_imm_eq() -> Result<(), Box<dyn std::error::Error>> {
    let path: std::path::PathBuf =
        std::path::Path::new("../../../gen/tree-sitter/xml/src/tests/pom.xml.test").to_path_buf();

    let text = std::fs::read(&path).unwrap();
    eprintln!("{}", std::fs::read_to_string(&path).unwrap());
    let (code_store, code) = crate::tests::xml_tree(&text);
    eprintln!("{}", SyntaxSerializer::new(&code_store, code));

    let query = r#"(element (STag (Name) (#EQ? "artifactId"))) @root"#;
    // let query = r#"(element) @root"#;
    let qqq = hyperast_tsquery::Query::new(query, hyperast_gen_ts_xml::language())
        .map_err(|e| e.to_string())?;
    let pos = hyperast::position::structural_pos::CursorWithPersistance::new(code);
    let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(&code_store, pos);
    let root_cap = qqq.capture_index_for_name("root").unwrap();
    let qcursor = qqq.matches(cursor);
    for m in qcursor {
        let pid = m.pattern_index;
        let i = qqq.enabled_pattern_index(pid).unwrap();
        assert_eq!(i, 0);
        dbg!(pid, i);
        let mut root_cap = m.nodes_for_capture_index(root_cap);
        let root_cap = root_cap.next().unwrap().pos.node();
        eprintln!("{}", SyntaxSerializer::new(&code_store, root_cap));
        eprintln!("{}", TextSerializer::new(&code_store, root_cap));
    }
    Ok(())
}

#[test]
fn test_new_matcher_for_xml_proj_artid() -> Result<(), Box<dyn std::error::Error>> {
    let path: std::path::PathBuf =
        std::path::Path::new("../../../gen/tree-sitter/xml/src/tests/pom.xml.test").to_path_buf();

    let text = std::fs::read(&path).unwrap();
    eprintln!("{}", std::fs::read_to_string(&path).unwrap());
    let (code_store, code) = crate::tests::xml_tree(&text);
    eprintln!("{}", SyntaxSerializer::new(&code_store, code));

    let query = r#"(document (_ (_
    (element (STag (Name) (#EQ? "artifactId"))) @root
)))"#;
    let qqq = hyperast_tsquery::Query::new(query, hyperast_gen_ts_xml::language())
        .map_err(|e| e.to_string())?;
    let pos = hyperast::position::structural_pos::CursorWithPersistance::new(code);
    let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(&code_store, pos);
    let root_cap = qqq.capture_index_for_name("root").unwrap();
    let qcursor = qqq.matches(cursor);
    for m in qcursor {
        let pid = m.pattern_index;
        let i = qqq.enabled_pattern_index(pid).unwrap();
        assert_eq!(i, 0);
        dbg!(pid, i);
        let mut root_cap = m.nodes_for_capture_index(root_cap);
        let root_cap = root_cap.next().unwrap().pos.node();
        eprintln!("{}", SyntaxSerializer::new(&code_store, root_cap));
        eprintln!("{}", TextSerializer::new(&code_store, root_cap));
    }
    Ok(())
}

#[test]
fn test_new_matcher_for_xml_deps_artid() -> Result<(), Box<dyn std::error::Error>> {
    let path: std::path::PathBuf =
        std::path::Path::new("../../../gen/tree-sitter/xml/src/tests/pom.xml.test").to_path_buf();

    let text = std::fs::read(&path).unwrap();
    eprintln!("{}", std::fs::read_to_string(&path).unwrap());
    let (code_store, code) = crate::tests::xml_tree(&text);
    eprintln!("{}", SyntaxSerializer::new(&code_store, code));

    let query = r#"(element (STag (Name) (#EQ? "dependency")) (_
    (element (STag (Name) (#EQ? "artifactId"))) @root
))"#;
    let qqq = hyperast_tsquery::Query::new(query, hyperast_gen_ts_xml::language())
        .map_err(|e| e.to_string())?;
    let pos = hyperast::position::structural_pos::CursorWithPersistance::new(code);
    let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(&code_store, pos);
    let root_cap = qqq.capture_index_for_name("root").unwrap();
    let qcursor = qqq.matches(cursor);
    for m in qcursor {
        let pid = m.pattern_index;
        let i = qqq.enabled_pattern_index(pid).unwrap();
        assert_eq!(i, 0);
        dbg!(pid, i);
        let mut root_cap = m.nodes_for_capture_index(root_cap);
        let root_cap = root_cap.next().unwrap().pos.node();
        eprintln!("{}", SyntaxSerializer::new(&code_store, root_cap));
        eprintln!("{}", TextSerializer::new(&code_store, root_cap));
    }
    Ok(())
}
