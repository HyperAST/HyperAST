macro_rules! tree {
    ( $k:expr ) => {
        SimpleTree::new($k, None, vec![])
    };
    ( $k:expr, $l:expr) => {
        SimpleTree::new($k, Some($l), vec![])
    };
    ( $k:expr, $l:expr; [$($x:expr),+ $(,)?]) => {
        SimpleTree::new($k, Some($l), vec![$($x),+])
    };
    ( $k:expr; [$($x:expr),+ $(,)?]) => {
        SimpleTree::new($k, None, vec![$($x),+])
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
