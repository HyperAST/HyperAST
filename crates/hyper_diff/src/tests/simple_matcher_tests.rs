use crate::tree::simple_tree::SimpleTree;
use crate::algorithms;
use crate::actions::Actions;
use hyperast::{
    full::FullNode, nodes::SyntaxSerializer, store::SimpleStores, tree_gen::StatsGlobalData, types::NodeId
};
use hyperast::test_utils::simple_tree::vpair_to_stores;
use hyperast_gen_ts_java::{
    legion_with_refs::{self, JavaTreeGen, Local},
    types::TStore,
};
use crate::tests::simple_matcher_examples::*;

fn prepare_tree_print<'a>(
    stores: &'a SimpleStores<TStore>,
) -> impl Fn(&FullNode<StatsGlobalData, Local>) -> () + 'a {
    return |tree: &FullNode<StatsGlobalData, Local>| {
        println!();
        println!(
            "{}",
            SyntaxSerializer::new(stores, tree.local.compressed_node)
        );
    };
}



#[test]
fn test_gumtree_simple_java_simple() {
    let (stores, src, dst) = vpair_to_stores(example_from_gumtree_java_simple());

    // Perform the diff using gumtree lazy
    let _diff_result = algorithms::gumtree::diff(
        &stores,
        &src,
        &dst,
    );

    println!("{}", _diff_result);

    let actions = _diff_result.actions.expect("Expected a result");

    let hyperast_actions_len = actions.len();
    let hyperast_matches_len = _diff_result.mapper.mappings.src_to_dst.iter().filter(|a| **a != 0).count();

    assert_eq!(hyperast_actions_len, 1);
    assert_eq!(hyperast_matches_len, 5);
}


#[test]
fn test_gumtree_simple_java_method() {
    let (stores, src, dst) = vpair_to_stores(example_from_gumtree_java_method());

    // Perform the diff using gumtree lazy
    let _diff_result = algorithms::gumtree::diff(
        &stores,
        &src,
        &dst,
    );

    println!("{}", _diff_result);

    let actions = _diff_result.actions.expect("Expected a result");

    let hyperast_actions_len = actions.len();
    let hyperast_matches_len = _diff_result.mapper.mappings.src_to_dst.iter().filter(|a| **a != 0).count();

    assert_eq!(hyperast_matches_len, 21);
    assert_eq!(hyperast_actions_len, 12);
}

#[test]
fn test_gumtree_simple_reorder_children() {
    let (stores, src, dst) = vpair_to_stores(example_reorder_children());

    // Perform the diff using gumtree lazy
    let _diff_result = algorithms::gumtree::diff(
        &stores,
        &src,
        &dst,
    );

    println!("{}", _diff_result);

    let actions = _diff_result.actions.expect("Expected a result");

    let hyperast_actions_len = actions.len();
    let hyperast_matches_len = _diff_result.mapper.mappings.src_to_dst.iter().filter(|a| **a != 0).count();

    assert_eq!(hyperast_matches_len, 25);
    assert_eq!(hyperast_actions_len, 1);
}

#[test]
fn test_gumtree_simple_move_method() {
    let (stores, src, dst) = vpair_to_stores(example_move_method());

    // Perform the diff using gumtree lazy
    let _diff_result = algorithms::gumtree::diff(
        &stores,
        &src,
        &dst,
    );

    println!("{}", _diff_result);

    let actions = _diff_result.actions.expect("Expected a result");

    let hyperast_actions_len = actions.len();
    let hyperast_matches_len = _diff_result.mapper.mappings.src_to_dst.iter().filter(|a| **a != 0).count();

    assert_eq!(hyperast_matches_len, 35);
    assert_eq!(hyperast_actions_len, 4);
}