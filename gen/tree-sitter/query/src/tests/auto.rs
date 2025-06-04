#![allow(unused)] // TODO maintain those tests

use hyperast::{
    nodes::TextSerializer,
    position::{StructuralPosition, TreePath, position_accessors::WithPreOrderOffsets},
    store::defaults::NodeIdentifier,
    types::{Typed, WithChildren},
};
use hyperast_gen_ts_cpp::iter::IterAll as CppIter;
use hyperast_gen_ts_xml::iter::IterAll as XmlIter;

use crate::{
    auto::{
        tsq_ser,
        tsq_ser_meta::{self, Conv},
        tsq_transform,
    },
    search::PreparedMatcher,
    tests::{cpp_tree, xml_tree},
};

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

const C3: &str = r#"int f() {
    int a = 21;
    int b = 21;
    return a + b;
}"#;

const C4: &str = r#"int f() {
    int b = 21;
    return b + b;
}"#;

type XmlTIdN = hyperast_gen_ts_xml::types::TIdN<NodeIdentifier>;
type CppTIdN = hyperast_gen_ts_cpp::types::TIdN<NodeIdentifier>;

type Cpp = hyperast_gen_ts_cpp::types::Type;
type Xml = hyperast_gen_ts_xml::types::Type;

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

// #[test]
// fn gen_match_simple() {
//     let (code_store, code) = cpp_tree(C0.as_bytes());
//     let pat = code;
//     let pat = code_store.node_store.resolve(pat).child(&0).unwrap();
//     let pat = code_store.node_store.resolve(pat).child(&4).unwrap();
//     let pat = code_store.node_store.resolve(pat).child(&2).unwrap();
//     let pat = code_store.node_store.resolve(pat).child(&2).unwrap();
//     println!();
//     println!(
//         "{}",
//         hyperast::nodes::SyntaxSerializer::<_, _, true>::new(&code_store, pat)
//     );
//     println!();
//     println!("{}", TextSerializer::new(&code_store, pat));
//     println!();
//     let q0 = tsq_ser::to_query(&code_store, pat).to_string();
//     println!("{}", q0);
//     let (query_store, query) = crate::search::ts_query(q0.as_bytes());

//     {
//         let path = StructuralPosition::new(code);
//         let prepared_matcher = PreparedMatcher::<Cpp>::new(query_store.with_ts(), query);
//         let mut matched = false;
//         for e in CppIter::new(&code_store, path, code) {
//             if prepared_matcher.is_matching::<_, CppTIdN>(&code_store, *e.node().unwrap()) {
//                 let (t, _) = code_store
//                     .node_store
//                     .resolve_with_type::<CppTIdN>(e.node().unwrap());
//                 dbg!(t);
//                 dbg!(t);
//                 matched = true;
//             }
//         }
//         assert!(matched);
//     }
//     {
//         let (code_store1, code1) = cpp_tree(C1.as_bytes());
//         let path = StructuralPosition::new(code1);
//         let prepared_matcher = PreparedMatcher::<Cpp>::new(query_store.with_ts(), query);
//         for e in CppIter::new(&code_store1, path, code1) {
//             if prepared_matcher.is_matching::<_, CppTIdN>(&code_store1, *e.node().unwrap()) {
//                 let n = code_store1
//                     .node_store
//                     .try_resolve_typed::<CppTIdN>(e.node().unwrap())
//                     .unwrap()
//                     .0;
//                 let t = n.get_type();
//                 dbg!(t);
//                 panic!("should not match")
//             }
//         }
//     }
// }

#[test]
fn gen_match_xml() {
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
        &hyperast::nodes::SyntaxSerializer::<_, _, true>::new(&code_store, pat).to_string()[..70]
    );
    println!();
    println!("{}", TextSerializer::new(&code_store, pat));
    let q0 = tsq_ser_meta::TreeToQuery::<_, XmlTIdN, Conv<Xml>, true>::with_pred(
        &code_store,
        pat,
        "(Name)",
    )
    .to_string();
    println!("{}", q0);
    // let q0 = format!("(document (element (content (element {}))))", q0);
    // let q0 = format!("(element (STag (Name) @id (#eq? @id \"artifactId\")))");
    let (query_store, query) = crate::search::ts_query(q0.as_bytes());

    {
        let path = StructuralPosition::new(code);
        let prepared_matcher = PreparedMatcher::<Xml>::new(query_store.with_ts(), query);
        let mut matched = false;
        for e in XmlIter::new(&code_store, path, code) {
            if prepared_matcher.is_matching::<_, XmlTIdN>(&code_store, *e.node().unwrap()) {
                println!("{}", TextSerializer::new(&code_store, *e.node().unwrap()));
                matched = true;
            }
        }
        assert!(matched);
    }

    {
        println!("\nsearching in build section:");
        let build = code;
        let build = code_store.node_store.resolve(build).child(&2).unwrap();
        let build = code_store.node_store.resolve(build).child(&1).unwrap();
        let build = code_store.node_store.resolve(build).child(&21).unwrap();
        let path = StructuralPosition::new(build);
        let prepared_matcher = PreparedMatcher::<Xml>::new(query_store.with_ts(), query);
        let mut matched = false;
        for e in XmlIter::new(&code_store, path, build) {
            if prepared_matcher.is_matching::<_, XmlTIdN>(&code_store, *e.node().unwrap()) {
                println!("{}", TextSerializer::new(&code_store, *e.node().unwrap()));
                matched = true;
            }
        }
        assert!(matched);
    }
    {
        let neg_subtree = code;
        let neg_subtree = code_store
            .node_store
            .resolve(neg_subtree)
            .child(&2)
            .unwrap();
        let neg_subtree = code_store
            .node_store
            .resolve(neg_subtree)
            .child(&1)
            .unwrap();
        let neg_subtree = code_store
            .node_store
            .resolve(neg_subtree)
            .child(&17)
            .unwrap();
        println!();
        println!("{}", TextSerializer::new(&code_store, neg_subtree));
        let path = StructuralPosition::new(neg_subtree);
        let prepared_matcher = PreparedMatcher::<Xml>::new(query_store.with_ts(), query);
        for e in XmlIter::new(&code_store, path, neg_subtree) {
            if prepared_matcher.is_matching::<_, XmlTIdN>(&code_store, *e.node().unwrap()) {
                let (t, _) = code_store
                    .node_store
                    .resolve_with_type::<XmlTIdN>(e.node().unwrap());
                dbg!(t);
                panic!("should not match")
            }
        }
    }
}

// #[test]
// fn gen_match_named() {
//     let (code_store, code) = cpp_tree(C2.as_bytes());
//     println!("\nThe code:\n{}", TextSerializer::new(&code_store, code));
//     let pat = code;
//     let pat = code_store.node_store.resolve(pat).child(&0).unwrap();
//     let pat = code_store.node_store.resolve(pat).child(&4).unwrap();
//     let pat = code_store.node_store.resolve(pat).child(&4).unwrap();
//     let pat = code_store.node_store.resolve(pat).child(&2).unwrap();
//     let pat = code_store.node_store.resolve(pat).child(&0).unwrap(); // when extracting hidden nodes, gets bin_op from _expr
//     println!("\nThe subtree:\n{}", TextSerializer::new(&code_store, pat));
//     let q0 = tsq_ser::to_query(&code_store, pat).to_string();
//     println!("\nThe corresponding query:\n{}", q0);
//     let (mut query_store, query) = crate::search::ts_query(q0.as_bytes());

//     let path = StructuralPosition::new(code);
//     let prepared_matcher = PreparedMatcher::<Cpp>::new(query_store.with_ts(), query);
//     let mut matched = false;
//     for e in CppIter::new(&code_store, path, code) {
//         if prepared_matcher.is_matching::<_, CppTIdN>(&code_store, *e.node().unwrap()) {
//             println!(
//                 "\nThe matched subtree:\n{}",
//                 TextSerializer::new(&code_store, *e.node().unwrap())
//             );
//             matched = true;
//         }
//     }
//     assert!(matched);
//     // Negative case
//     let (code_store1, code1) = cpp_tree(C3.as_bytes());
//     println!(
//         "\nThe second code:\n{}",
//         TextSerializer::new(&code_store1, code1)
//     );
//     let path = StructuralPosition::new(code1);
//     let prepared_matcher = PreparedMatcher::<Cpp>::new(query_store.with_ts(), query);
//     for e in CppIter::new(&code_store1, path, code1) {
//         if prepared_matcher.is_matching::<_, CppTIdN>(&code_store1, *e.node().unwrap()) {
//             panic!("should not match")
//         }
//     }

//     println!("(Good) The initial query does not match the second code");

//     let (code_store2, code2) = cpp_tree(C4.as_bytes());
//     println!(
//         "\nThe third code (initial code with a rename):\n{}",
//         TextSerializer::new(&code_store2, code2)
//     );

//     let path = StructuralPosition::new(code2);
//     let prepared_matcher = PreparedMatcher::<Cpp>::new(query_store.with_ts(), query);
//     for e in CppIter::new(&code_store2, path, code2) {
//         if prepared_matcher.is_matching::<_, CppTIdN>(&code_store2, *e.node().unwrap()) {
//             panic!("should not match")
//         }
//     }
//     println!("(Expected) The initial query does not match the third code");
//     println!("Lets make the initial query more generic");

//     const M0: &str = r#"(predicate (identifier) @op (#eq? @op "eq") (parameters (capture (identifier) @id ) (string) @label ))"#;
//     println!("");
//     println!("\nThe meta query:\n{}", M0);
//     let (query_store1, query1) = crate::search::ts_query(M0.as_bytes());

//     let path = StructuralPosition::new(query);
//     let prepared_matcher = PreparedMatcher::<crate::types::Type>::new(query_store1.with_ts(), query1);
//     let mut per_label = std::collections::HashMap::<
//         String,
//         Vec<(String, StructuralPosition<NodeIdentifier, u16>)>,
//     >::default();
//     for e in crate::iter::IterAll::new(&query_store, path, query) {
//         if let Some(capts) = prepared_matcher
//             .is_matching_and_capture::<_, crate::types::TIdN<NodeIdentifier>>(
//                 &query_store,
//                 *e.node().unwrap(),
//             )
//         {
//             dbg!(&capts);
//             let l_l = prepared_matcher
//                 .captures
//                 .iter()
//                 .position(|x| &x.name == "label")
//                 .unwrap() as u32;
//             let l_i = prepared_matcher
//                 .captures
//                 .iter()
//                 .position(|x| &x.name == "label")
//                 .unwrap() as u32;
//             let k = capts
//                 .by_capture_id(l_l)
//                 .unwrap()
//                 .clone()
//                 .try_label(&code_store)
//                 .unwrap();
//             let v = capts
//                 .by_capture_id(l_i)
//                 .unwrap()
//                 .clone()
//                 .try_label(&code_store)
//                 .unwrap();
//             let p = e;
//             per_label
//                 .entry(k.to_string())
//                 .or_insert(vec![])
//                 .push((v.to_string(), p));
//         }
//     }
//     assert_eq!(1, per_label.len());
//     dbg!(&per_label);
//     struct PerLabel(
//         std::collections::HashMap<String, Vec<(String, StructuralPosition<NodeIdentifier, u16>)>>,
//     );

//     impl std::fmt::Display for PerLabel {
//         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//             for x in self.0.values() {
//                 if x.len() == 2 {
//                     writeln!(f, "(#eq? @{} @{})", x[0].0, x[1].0)?;
//                     dbg!(x[0].1.shared_ancestors(&x[1].1));
//                 } else if x.len() == 1 {
//                     // noop
//                 } else {
//                     todo!("need to do combination")
//                 }
//             }
//             Ok(())
//         }
//     }

//     let query_bis = tsq_transform::regen_query(
//         &mut query_store,
//         query,
//         per_label
//             .values()
//             .filter(|l| l.len() == 2)
//             .flatten()
//             .map(|x| tsq_transform::Action::Delete {
//                 path: x.1.iter_offsets().collect(),
//             })
//             .collect(),
//     );

//     println!(
//         "\nAgain the third code (initial code with a rename):\n{}",
//         TextSerializer::new(&code_store2, code2)
//     );

//     let qbis = TextSerializer::<_, _>::new(query_store.with_ts(), query_bis.unwrap()).to_string();
//     let qbis = format!("{} {}", qbis, PerLabel(per_label.clone()));
//     println!("\nThe generified query:\n{}", qbis);
//     let (query_store, query) = crate::search::ts_query(qbis.as_bytes());

//     {
//         let path = StructuralPosition::new(code2);
//         let prepared_matcher = PreparedMatcher::<Cpp>::new(query_store.with_ts(), query);
//         let mut matched = false;
//         for e in CppIter::new(&code_store2, path, code2) {
//             if prepared_matcher.is_matching::<_, CppTIdN>(&code_store2, *e.node().unwrap()) {
//                 println!("\nThe matched subtree with a rename:");
//                 println!("{}", TextSerializer::new(&code_store2, *e.node().unwrap()));
//                 matched = true;
//             }
//         }
//         assert!(matched);
//     }
// }
