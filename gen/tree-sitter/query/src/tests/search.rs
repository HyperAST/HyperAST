use hyper_ast::{
    position::TreePath,
    store::{
        defaults::{NodeIdentifier},
        nodes::legion::NodeStore,
        SimpleStores,
    },
    types::{IterableChildren, Labeled, Typed, WithChildren},
};
use hyper_ast_gen_ts_cpp::legion::CppTreeGen;
use std::{
    io::{Write},
    ops::Deref,
    sync::Arc,
};

use crate::legion::TsQueryTreeGen;

const Q0: &str = r#"(binary_expression (number_literal) "+" (number_literal))"#;
const C0: &str = r#"int f() {
    return 21 + 21;
}"#;

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

#[test]
fn simple() {
    use hyper_ast::types::{LabelStore, NodeStore};
    let (code_store, code) = cpp_tree(C0.as_bytes());
    let (query_store, query) = crate::search::ts_query(Q0.as_bytes());
    let path = hyper_ast::position::StructuralPosition::new(code);
    let prepared_matcher =
        crate::search::PreparedMatcher::<_,hyper_ast_gen_ts_cpp::types::Type>::new(&query_store, query);
    for e in hyper_ast_gen_ts_cpp::iter::IterAll::new(&code_store, path, code) {
        if prepared_matcher.is_matching::<_, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>(&code_store, *e.node().unwrap()) {
            type T = hyper_ast_gen_ts_cpp::types::TIdN<hyper_ast::store::defaults::NodeIdentifier>;
            let n = code_store
                .node_store
                .try_resolve_typed::<T>(e.node().unwrap())
                .unwrap()
                .0;
            let t = n.get_type();
            dbg!(t);
        }
        // for i in 0..prepared_matcher.quick_trigger.root_types.deref().len() {
        //     let tt = prepared_matcher.quick_trigger.root_types[i];
        //     let pat = &prepared_matcher.patterns[i];
        //     if t == tt {
        //         dbg!("", pat.is_matching(&code_store, e.node().unwrap()));

        //     }
    }
}

fn cpp_tree(
    text: &[u8],
) -> (
    SimpleStores<hyper_ast_gen_ts_cpp::types::TStore>,
    legion::Entity,
) {
    use hyper_ast_gen_ts_cpp::types::TStore;
    let tree = match CppTreeGen::<TStore>::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    println!("{:#?}", tree.root_node().to_sexp());
    let mut stores: SimpleStores<TStore> = SimpleStores::default();
    let mut md_cache = Default::default();
    let mut tree_gen = CppTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    let x = tree_gen.generate_file(b"", text, tree.walk()).local;
    let entity = x.compressed_node;
    println!(
        "{}",
        hyper_ast::nodes::SyntaxSerializer::<_, _, true>::new(&stores, entity)
    );
    (stores, entity)
}
