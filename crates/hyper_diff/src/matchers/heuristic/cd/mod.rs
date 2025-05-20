pub mod bottom_up_matcher;
pub mod lazy_bottom_up_matcher;
pub mod lazy_bottom_up_matcher_2;
pub mod lazy_leaves_matcher;
pub mod lazy_leaves_matcher_2;
pub mod leaves_matcher;

#[derive(Debug, Clone, PartialEq)]
pub struct LeavesMatcherConfig {
    pub label_sim_threshold: f64,
}

impl Default for LeavesMatcherConfig {
    fn default() -> Self {
        Self {
            label_sim_threshold: 0.5,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BottomUpMatcherConfig {
    pub max_leaves: usize,
    pub sim_threshold_large_trees: f64,
    pub sim_threshold_small_trees: f64,
}

impl Default for BottomUpMatcherConfig {
    fn default() -> Self {
        Self {
            max_leaves: 4,
            sim_threshold_large_trees: 0.6,
            sim_threshold_small_trees: 0.4,
        }
    }
}
