use hyper_diff::algorithms;
use hyperast::full::FullNode;
use hyperast::nodes::SyntaxSerializer;
use hyperast::store::SimpleStores;
use hyperast::tree_gen::StatsGlobalData;
use hyperast_gen_ts_java::legion_with_refs::{JavaTreeGen, Local, tree_sitter_parse};
use hyperast_gen_ts_java::types::TStore;

//Parses the provided bytes to a java syntax tree
fn preprocess_for_diff(
    src: &[u8],
    dst: &[u8],
) -> (
    SimpleStores<TStore>,
    FullNode<StatsGlobalData, Local>,
    FullNode<StatsGlobalData, Local>,
) {
    let mut stores = SimpleStores::<TStore>::default();
    let mut md_cache = Default::default(); // [cite: 133, 139]
    let mut java_tree_gen = JavaTreeGen::new(&mut stores, &mut md_cache);
    let tree = match tree_sitter_parse(src) {
        Ok(t) => t,
        Err(t) => t,
    };
    let src = java_tree_gen.generate_file(b"", src, tree.walk());
    let tree = match tree_sitter_parse(dst) {
        Ok(t) => t,
        Err(t) => t,
    };
    let dst = java_tree_gen.generate_file(b"", dst, tree.walk());
    return (stores, src, dst);
}

fn prepare_tree_print<'a>(
    stores: &'a SimpleStores<TStore>,
) -> impl Fn(&FullNode<StatsGlobalData, Local>) -> () + 'a {
    return |tree: &FullNode<StatsGlobalData, Local>| {
        println!();
        let id = tree.local.compressed_node;
        println!("{}", SyntaxSerializer::new(stores, id));
    };
}

#[test]
fn change_class_name_test() {
    let src = "class A {}".as_bytes();
    let dst = "class B {}".as_bytes();

    let (stores, src, dst) = preprocess_for_diff(src, dst);

    let diff_result = algorithms::gumtree::diff(
        &stores,
        &src.local.compressed_node,
        &dst.local.compressed_node,
        1000,
        0.5f64,
    );

    let print_tree = prepare_tree_print(&stores);
    print_tree(&src);
    print_tree(&dst);

    println!("stats from diffing: \n{:?}", &diff_result.summarize());

    println!("{}", diff_result);
}

#[test]
fn add_inner_class_test() {
    let src = "class A {}".as_bytes();
    let dst = "class A { class B {} }".as_bytes();

    let (stores, src, dst) = preprocess_for_diff(src, dst);
    let diff_result = algorithms::gumtree::diff(
        &stores,
        &src.local.compressed_node,
        &dst.local.compressed_node,
        1000,
        0.5f64,
    );

    let print_tree = prepare_tree_print(&stores);
    print_tree(&src);
    print_tree(&dst);

    println!("stats from diffing: \n{:?}", &diff_result.summarize());
    for a in diff_result.actions.unwrap().iter() {
        println!("{:?}", a)
    }
}
