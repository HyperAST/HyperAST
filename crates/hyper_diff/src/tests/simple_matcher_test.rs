use crate::actions::Actions;
use crate::algorithms;
use crate::tests::simple_matcher_examples::*;
use hyperast::test_utils::simple_tree::{SimpleTree, vpair_to_stores};

#[test]
fn test_for_mappings() {
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

struct DiffInfo {
    num_actions_normal: usize,
    num_actions_lazy: usize,
    num_matches_normal: usize,
    num_matches_lazy: usize,
}

impl DiffInfo {
    fn assert_correctness(&self, target_num_matches: usize, target_num_actions: usize) {
        assert_eq!(
            self.num_matches_normal, self.num_matches_lazy,
            "Number of matches normal and lazy were not equal"
        );
        assert!(
            self.num_matches_normal >= target_num_matches,
            "Number of matches did not match, target"
        );
        assert_eq!(
            self.num_actions_normal, self.num_actions_lazy,
            "Number of actions normal and lazy were not equal"
        );
        assert!(
            self.num_actions_normal <= target_num_actions,
            "Number of actions did not match target"
        );
    }
}

fn get_diff_info(example: (SimpleTree<u8>, SimpleTree<u8>)) -> DiffInfo {
    let (stores, src, dst) = vpair_to_stores(example);

    // Apply diff with both gumtree simple and simple_lazy
    let _diff_result_normal = algorithms::gumtree_simple::diff(&stores, &src, &dst);
    let _diff_result_lazy = algorithms::gumtree_simple_lazy::diff(&stores, &src, &dst);
    let _diff_result_greedy = algorithms::gumtree::diff(&stores, &src, &dst);

    // Get the number of generated actions
    let num_actions_normal = _diff_result_normal
        .actions
        .expect("ASTs were not equal, but no actions were found")
        .len();
    let num_actions_lazy = _diff_result_lazy
        .actions
        .expect("ASTs were not equal, but no actions were found")
        .len();
    let num_actions_greedy = _diff_result_greedy
        .actions
        .expect("ASTs were not equal, but no actions were found")
        .len();

    // Get the number of mappings found
    let num_matches_normal = _diff_result_normal
        .mapper
        .mappings
        .src_to_dst
        .iter()
        .filter(|a| **a != 0)
        .count();
    let num_matches_lazy = _diff_result_lazy
        .mapper
        .mappings
        .src_to_dst
        .iter()
        .filter(|a| **a != 0)
        .count();
    let num_matches_greedy = _diff_result_greedy
        .mapper
        .mappings
        .src_to_dst
        .iter()
        .filter(|a| **a != 0)
        .count();

    println!(
        "Greedy found {} mappings, simple found {} mappings and lazy simple found: {} mappings",
        num_matches_greedy, num_matches_normal, num_matches_lazy
    );
    println!(
        "Greedy found {} actions, simple found {} actions and lazy simple found: {} actions",
        num_actions_greedy, num_actions_normal, num_actions_lazy
    );

    return DiffInfo {
        num_actions_normal,
        num_actions_lazy,
        num_matches_normal,
        num_matches_lazy,
    };
}

#[test]
fn test_gumtree_simple_java_simple() {
    let diff_info = get_diff_info(example_from_gumtree_java_simple());
    diff_info.assert_correctness(5, 1);
}

#[test]
fn test_gumtree_simple_java_method() {
    let diff_info = get_diff_info(example_from_gumtree_java_method());
    diff_info.assert_correctness(21, 12);
}

#[test]
fn test_gumtree_simple_reorder_children() {
    let diff_info = get_diff_info(example_reorder_children());
    diff_info.assert_correctness(25, 1);
}

#[test]
fn test_gumtree_simple_move_method() {
    // Here Gumtree Simple actually finds more matches than Gumtree Greedy
    let diff_info = get_diff_info(example_move_method());
    diff_info.assert_correctness(31, 7);
}
