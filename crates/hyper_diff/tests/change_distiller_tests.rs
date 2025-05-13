use hyper_diff::{algorithms, matchers::mapping_store::MonoMappingStore};
use hyperast::test_utils::simple_tree::vpair_to_stores;

macro_rules! tree {
    ( $k:expr ) => {
        hyperast::test_utils::simple_tree::SimpleTree::new($k, None, vec![])
    };
    ( $k:expr, $l:expr) => {
        hyperast::test_utils::simple_tree::SimpleTree::new($k, Some($l), vec![])
    };
    ( $k:expr, $l:expr; [$($x:expr),+ $(,)?]) => {
        hyperast::test_utils::simple_tree::SimpleTree::new($k, Some($l), vec![$($x),+])
    };
    ( $k:expr; [$($x:expr),+ $(,)?]) => {
        hyperast::test_utils::simple_tree::SimpleTree::new($k, None, vec![$($x),+])
    };
}

#[test]
fn test_base_cd_vs_lazy_cd_same() {
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

    // Create stores for the test trees
    let (stores, src, dst) = vpair_to_stores((src, dst));

    let result_base = algorithms::change_distiller::diff(&stores, &src, &dst);
    let result_lazy = algorithms::change_distiller_lazy::diff(&stores, &src, &dst);

    // Convert mappings to vectors for comparison
    let base_mappings: Vec<_> = result_base.mapper.mappings.iter().collect();
    let lazy_mappings: Vec<_> = result_lazy.mapper.mappings.iter().collect();

    assert_eq!(base_mappings, lazy_mappings);
}
