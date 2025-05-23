pub mod bottom_up_matcher;
pub mod lazy_bottom_up_matcher;
pub mod lazy_bottom_up_matcher_2;
pub mod lazy_leaves_matcher;
pub mod lazy_leaves_matcher_2;
pub mod leaves_matcher;
pub mod optimized_bottom_up_matcher;
pub mod optimized_leaves_matcher;

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

/// Configuration for the optimized diff algorithm with granular control over optimizations
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizedDiffConfig {
    /// Use lazy decompression for better memory efficiency. If lazy decompression is disabled, the algorithm will fall back to the baseline implementation and most other optimizations will be disabled aswell.
    pub use_lazy_decompression: bool,
    /// Use ranged similarity computation instead of chawathe similarity
    pub use_ranged_similarity: bool,
    /// Whether to calculate the script generation (can be disabled for performance)
    pub calculate_script: bool,
    /// Configuration for leaves matcher optimizations
    pub leaves_matcher: OptimizedLeavesMatcherConfig,
    /// Configuration for bottom-up matcher optimizations
    pub bottom_up_matcher: OptimizedBottomUpMatcherConfig,
}

impl Default for OptimizedDiffConfig {
    /// Create a default configuration with all optimizations enabled except script calculation
    fn default() -> Self {
        Self {
            use_lazy_decompression: true,
            use_ranged_similarity: true,
            calculate_script: false,
            leaves_matcher: OptimizedLeavesMatcherConfig::default(),
            bottom_up_matcher: OptimizedBottomUpMatcherConfig::default(),
        }
    }
}

/// Configuration for optimized leaves matcher with individual optimization flags
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizedLeavesMatcherConfig {
    /// Base configuration (label similarity threshold)
    pub base_config: LeavesMatcherConfig,
    /// Cache label strings to avoid repeated resolution. This is automatically enabled when using type grouping.
    pub enable_label_caching: bool,
    /// Group leaves by type before comparison. This automatically enables label caching.
    pub enable_type_grouping: bool,
    /// Use binary heap instead of vector + sort for mappings
    pub use_binary_heap: bool,
    /// Reuse QGram object for string distance computation
    pub reuse_qgram_object: bool,
}

impl Default for OptimizedLeavesMatcherConfig {
    /// Create a default configuration with all optimizations enabled
    fn default() -> Self {
        Self {
            base_config: LeavesMatcherConfig::default(),
            enable_label_caching: true,
            enable_type_grouping: true,
            use_binary_heap: true,
            reuse_qgram_object: true,
        }
    }
}

/// Configuration for optimized bottom-up matcher with individual optimization flags
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizedBottomUpMatcherConfig {
    /// Base configuration (thresholds and max_leaves)
    pub base_config: BottomUpMatcherConfig,
    /// Group nodes by type before comparison
    pub enable_type_grouping: bool,
    /// Pre-compute leaf counts in single traversal
    pub enable_leaf_count_precomputation: bool,
}

impl Default for OptimizedBottomUpMatcherConfig {
    /// Create a default configuration with all optimizations enabled
    fn default() -> Self {
        Self {
            base_config: BottomUpMatcherConfig::default(),
            enable_type_grouping: true,
            enable_leaf_count_precomputation: true,
        }
    }
}
