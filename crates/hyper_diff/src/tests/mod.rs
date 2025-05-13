/// Macro for constructing simple trees.
///
/// # Arguments
///
/// * `$k` - The type/value of the node
/// * `$l` - Optional label for the node
///
/// # Examples
///
/// ```
/// // Create a single node with no label or children
/// tree!(type);
///
/// // Create a node with a label but no children
/// tree!(type, label);
///
/// // Create a node with children but no label
/// tree!(type; [child1, child2]);
///
/// // Create a node with both label and children
/// tree!(type, label; [child1, child2]);
/// ```
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
pub(crate) use tree;

pub mod action_generator2_simple_tests;
pub mod action_generator2_tests;
pub mod action_generator_tests;
#[cfg(test)]
pub mod examples;
pub mod hungarian_tests;
#[cfg(test)]
pub mod lazy_decompression_tests;
pub mod pair_tests;
pub mod simple_examples;
