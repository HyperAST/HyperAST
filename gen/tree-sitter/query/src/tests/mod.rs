use std::io::{stdout, Write};

use hyperast::store::SimpleStores;

use crate::{legion::TsQueryTreeGen, types::TStore};

#[test]
fn simple() {
    let case0 = r#"(binary_expression (number_literal) (number_literal))"#;

    run(case0.as_bytes())
}

fn run(text: &[u8]) {
    let mut stores = SimpleStores::<TStore>::default();
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
        hyperast::nodes::SyntaxSerializer::<_, _, true>::new(
            &*java_tree_gen.stores,
            full_node.local.compressed_node
        )
    );
    stdout().flush().unwrap();
    println!(
        "{}",
        hyperast::nodes::JsonSerializer::<_, _, false>::new(
            &*java_tree_gen.stores,
            full_node.local.compressed_node
        )
    )
}

mod auto;
mod search;

fn cpp_tree(
    text: &[u8],
) -> (
    SimpleStores<hyperast_gen_ts_cpp::types::TStore>,
    legion::Entity,
) {
    use hyperast_gen_ts_cpp::legion::CppTreeGen;
    use hyperast_gen_ts_cpp::types::TStore;
    let tree = match hyperast_gen_ts_cpp::legion::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    // println!("{:#?}", tree.root_node().to_sexp());
    let mut stores: SimpleStores<TStore> = SimpleStores::default();
    let mut md_cache = Default::default();
    let mut tree_gen = CppTreeGen::new(&mut stores, &mut md_cache);
    let x = tree_gen.generate_file(b"", text, tree.walk()).local;
    let entity = x.compressed_node;
    // println!(
    //     "{}",
    //     hyperast::nodes::SyntaxSerializer::<_, _, true>::new(&stores, entity)
    // );
    (stores, entity)
}
fn xml_tree(
    text: &[u8],
) -> (
    SimpleStores<hyperast_gen_ts_xml::types::TStore>,
    legion::Entity,
) {
    use hyperast_gen_ts_xml::legion::XmlTreeGen;
    use hyperast_gen_ts_xml::types::TStore;
    let tree = match hyperast_gen_ts_xml::legion::tree_sitter_parse_xml(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    // println!("{:#?}", tree.root_node().to_sexp());
    let mut stores: SimpleStores<TStore> = SimpleStores::default();
    let mut tree_gen = XmlTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
    };
    let x = tree_gen.generate_file(b"", text, tree.walk()).local;
    let entity = x.compressed_node;
    // println!(
    //     "{}",
    //     hyperast::nodes::SyntaxSerializer::<_, _, true>::new(&stores, entity)
    // );
    (stores, entity)
}
