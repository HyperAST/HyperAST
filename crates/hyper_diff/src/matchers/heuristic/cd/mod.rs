use std::time::Duration;

use serde::{Serialize, Serializer};

use crate::{actions::action_vec::ActionsVec, algorithms::DiffResult};

pub mod bottom_up_matcher;
pub mod iterator;
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
    #[deprecated]
    pub enable_type_grouping: bool,
    /// Only iterate to the highest statement level nodes
    pub statement_level_iteration: bool,
    /// Use binary heap instead of vector + sort for mappings
    #[deprecated]
    pub use_binary_heap: bool,
    /// Reuse QGram object for string distance computation
    #[deprecated]
    pub reuse_qgram_object: bool,
}

impl Default for OptimizedLeavesMatcherConfig {
    /// Create a default configuration with all optimizations enabled
    fn default() -> Self {
        Self {
            base_config: LeavesMatcherConfig::default(),
            enable_label_caching: true,
            enable_type_grouping: false,
            statement_level_iteration: true,
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
    #[deprecated]
    pub enable_type_grouping: bool,
    /// Pre-compute leaf counts in single traversal
    pub enable_leaf_count_precomputation: bool,
    /// Only iterate up to the highest statement level nodes
    pub statement_level_iteration: bool,
}

impl Default for OptimizedBottomUpMatcherConfig {
    /// Create a default configuration with all optimizations enabled
    fn default() -> Self {
        Self {
            base_config: BottomUpMatcherConfig::default(),
            enable_type_grouping: false,
            enable_leaf_count_precomputation: true,
            statement_level_iteration: true,
        }
    }
}

/// Detailed metrics collected during leaves matching
#[derive(Debug, Clone, Default, Serialize)]
pub struct LeavesMatcherMetrics {
    #[serde(serialize_with = "duration_as_millis")]
    pub total_time: Duration,
    /// Total number of leaf-to-leaf comparisons attempted (including both successful and unsuccessful).
    pub total_comparisons: usize,
    /// Number of successful matches found between source and destination leaves.
    pub successful_matches: usize,
    /// Total time spent computing hashes for leaves (for hash-based optimizations).
    #[serde(serialize_with = "duration_as_millis")]
    pub hash_computation_time: Duration,
    /// Total time spent serializing leaf node text for comparison.
    #[serde(serialize_with = "duration_as_millis")]
    pub text_serialization_time: Duration,
    /// Total time spent performing string similarity calculations.
    #[serde(serialize_with = "duration_as_millis")]
    pub similarity_time: Duration,
    /// Total number of characters compared during similarity checks.
    pub characters_compared: usize,
    /// Number of times a cached label or serialized node was successfully reused.
    pub cache_hits: usize,
    /// Number of times a cache lookup failed and a value had to be recomputed.
    pub cache_misses: usize,
    /// Number of exact matches found based on identical label-hash.
    pub exact_matches: usize,
    /// Number of expensive similarity checks performed (may be less than total comparisons if some are skipped).
    pub similarity_checks: usize,
    /// Number of destination leaves skipped due to pre-matching.
    pub skipped_dst: usize,
}

/// Detailed metrics collected during bottom-up matching
#[derive(Debug, Clone, Default, Serialize)]
pub struct BottomUpMatcherMetrics {
    #[serde(serialize_with = "duration_as_millis")]
    pub total_time: Duration,
    pub total_comparisons: usize,
    pub successful_matches: usize,
    #[serde(serialize_with = "duration_as_millis")]
    pub similarity_time: Duration,
}

/// Detailed metrics and result from Change Distiller algorithm
#[derive(Debug)]
pub struct CDResult<A, M, MD> {
    pub mapping_durations: MD,
    pub mapper: M,
    pub actions: Option<ActionsVec<A>>,
    pub prepare_gen_t: f64,
    pub gen_t: f64,
    pub total_time: Duration,

    pub produced_mappings: usize,
    pub subtree_prepare_time: Duration,

    pub leaves_matcher_metrics: LeavesMatcherMetrics,
    pub bottomup_matcher_metrics: BottomUpMatcherMetrics,
}

impl<A, M, MD> CDResult<A, M, MD> {
    /// Convert CDResult to standard DiffResult for backward compatibility
    pub fn into_diff_result(self) -> DiffResult<A, M, MD> {
        DiffResult {
            mapping_durations: self.mapping_durations,
            mapper: self.mapper,
            actions: self.actions,
            prepare_gen_t: self.prepare_gen_t,
            gen_t: self.gen_t,
        }
    }
}

/// Serializable version of detailed CDResult metrics
#[derive(Debug, Serialize)]
pub struct DiffResultSummary {
    mappings: usize,
    actions: Option<usize>,
    prepare_gen_t: f64,
    gen_t: f64,

    leaves: LeavesMatcherMetrics,
    bottomup: BottomUpMatcherMetrics,
}

impl<A, M, MD> Into<DiffResultSummary> for CDResult<A, M, MD> {
    fn into(self) -> DiffResultSummary {
        DiffResultSummary {
            mappings: self.produced_mappings,
            actions: self.actions.as_ref().map(|x| x.0.len()),
            prepare_gen_t: self.prepare_gen_t,
            gen_t: self.gen_t,

            leaves: self.leaves_matcher_metrics,
            bottomup: self.bottomup_matcher_metrics,
        }
    }
}

fn duration_as_millis<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_f32(duration.as_secs_f32() * 1000.0)
}
