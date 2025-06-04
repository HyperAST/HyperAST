use crate::actions::Actions;
use crate::algorithms;
use crate::tests::simple_matcher_examples::*;
use hyperast::test_utils::simple_tree::vpair_to_stores;

// //Parses the provided bytes to a java syntax tree
// fn preprocess_for_diff(
//     src: &[u8],
//     dst: &[u8],
// ) -> (
//     SimpleStores<TStore>,
//     FullNode<StatsGlobalData, Local>,
//     FullNode<StatsGlobalData, Local>,
// ) {
//     let mut stores = SimpleStores::<TStore>::default();
//     let mut md_cache = Default::default(); // [cite: 133, 139]
//     let mut java_tree_gen = JavaTreeGen::new(&mut stores, &mut md_cache);
//     let tree = match legion_with_refs::tree_sitter_parse(src) {
//         Ok(t) => t,
//         Err(t) => t,
//     };
//     let src = java_tree_gen.generate_file(b"", src, tree.walk());
//     let tree = match legion_with_refs::tree_sitter_parse(dst) {
//         Ok(t) => t,
//         Err(t) => t,
//     };
//     let dst = java_tree_gen.generate_file(b"", dst, tree.walk());
//     return (stores, src, dst);
// }

#[test]
fn test_for_mappings() {
    use hyperast::test_utils::simple_tree::SimpleTree;
    let src = tree!(
        0,"a"; [
            tree!(0, "e"; [
                tree!(0, "f")]),
            tree!(0, "b"; [
                tree!(0, "c"),
                tree!(0, "d")]),
    ]);
    let dst = tree!(
        0,"a"; [
            tree!(0, "e"; [
                tree!(0, "g")]),
            tree!(0, "b"; [
                tree!(0, "c"),
                tree!(0, "d")]),
    ]);

    //     r
    //   / | \
    //  /  |  \
    // x   y   z
    // gets represented as [x, y, z, r]
    // if y would have children it would be: [x, <children y>, y, z, r]
    // For the mappings in the VecStore it is as follows. If it is 0 it is unmapped, if it has an number i
    // It means it is mapped with node (i-1) of the other tree

    let (stores, src, dst) = vpair_to_stores((src, dst));
    let diff_result = algorithms::gumtree_simple::diff(&stores, &src, &dst);

    diff_result
        .actions
        .as_ref()
        .unwrap()
        .iter()
        .for_each(|a| println!("{:?}", a));

    println!("\nfinal mappings: \n{:?}", &diff_result.mapper.mappings());
    assert_eq!(
        diff_result
            .actions
            .expect("ASTs are not identical, but no actions were found")
            .len(),
        1 as usize,
        "Incorrect number of actions"
    );
}

#[test]
fn test_gumtree_simple_java_simple() {
    let (stores, src, dst) = vpair_to_stores(example_from_gumtree_java_simple());

    // Perform the diff using gumtree lazy
    let _diff_result = algorithms::gumtree_simple::diff(&stores, &src, &dst);
    let actions = _diff_result.actions.expect("Expected a result");

    let hyperast_actions_len = actions.len();
    let hyperast_matches_len = _diff_result
        .mapper
        .mappings
        .src_to_dst
        .iter()
        .filter(|a| **a != 0)
        .count();

    assert_eq!(hyperast_actions_len, 1, "Number of actions did not match");
    assert_eq!(hyperast_matches_len, 5, "Number of matches did not match");
}
