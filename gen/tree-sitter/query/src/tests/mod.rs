use std::io::{stdout, Write};

use hyper_ast::store::{labels::LabelStore, nodes::legion::NodeStore, SimpleStores};

use crate::{legion::TsQueryTreeGen, types::TStore};




#[test]
fn simple() {
    let case0 = r#"(binary_expression (number_literal) (number_literal))"#;

    run(case0.as_bytes())
}


fn run(text: &[u8]) {
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TStore::default(),
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = TsQueryTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };

    let tree = match crate::legion::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text, tree.walk());

    println!();
    println!(
        "{}",
        hyper_ast::nodes::SyntaxSerializer::<_,_,true>::new(
            &*java_tree_gen.stores,
            full_node.local.compressed_node
        )
    );
    stdout().flush().unwrap();
    println!(
        "{}",
        hyper_ast::nodes::JsonSerializer::<_,_,false>::new(
            &*java_tree_gen.stores,
            full_node.local.compressed_node
        )
    )
}

mod search;
mod auto;