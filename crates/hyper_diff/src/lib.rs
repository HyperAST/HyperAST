pub mod actions;
pub mod decompressed_tree_store;
#[cfg(feature = "experimental")]
pub mod mapping;
pub mod matchers;
pub mod tree;
pub mod utils;
// TODO rename to helpers
/// helpers
pub mod algorithms;

// // Re-export optimized diff API for convenience
// pub use algorithms::change_distiller_optimized::{
//     diff_baseline, diff_with_all_optimizations, diff_baseline,
// };
pub use matchers::heuristic::cd::{
    OptimizedBottomUpMatcherConfig, OptimizedDiffConfig, OptimizedLeavesMatcherConfig,
};

#[cfg(test)]
mod tests;
