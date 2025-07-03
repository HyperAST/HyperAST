use super::MappingDurations;
use super::PreparedMappingDurations;
use crate::actions::script_generator2::{ScriptGenerator, SimpleAction};
use crate::algorithms::tr;
use crate::decompressed_tree_store::lazy_post_order::LazyPostOrder;
use crate::decompressed_tree_store::{CompletePostOrder, bfs_wrapper::SimpleBfsMapper};
use crate::matchers::heuristic::cd::bottom_up_matcher::BottomUpMatcher;
use crate::matchers::heuristic::cd::leaves_matcher::LeavesMatcher;
use crate::matchers::heuristic::cd::optimized_bottom_up_matcher::OptimizedBottomUpMatcher;
use crate::matchers::heuristic::cd::optimized_leaves_matcher::OptimizedLeavesMatcher;
use crate::matchers::heuristic::cd::{CDResult, OptimizedDiffConfig};
use crate::matchers::mapping_store::{MappingStore, VecStore};
use crate::matchers::{Decompressible, Mapper};
use crate::tree::tree_path::CompressedTreePath;
use hyperast::types::{self, HyperAST, HyperASTShared, NodeId};
use std::{fmt::Debug, time::Instant};

#[allow(type_alias_bounds)]
type DS<HAST: HyperASTShared> = Decompressible<HAST, LazyPostOrder<HAST::IdN, u32>>;
#[allow(type_alias_bounds)]
type CDS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;

pub fn diff_with_lazy_decompression<HAST: HyperAST + Copy>(
    hyperast: HAST,
    src: &HAST::IdN,
    dst: &HAST::IdN,
    config: OptimizedDiffConfig,
) -> CDResult<
    SimpleAction<HAST::Label, CompressedTreePath<HAST::Idx>, HAST::IdN>,
    Mapper<HAST, CDS<HAST>, CDS<HAST>, VecStore<u32>>,
    PreparedMappingDurations<2>,
>
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Idx: hyperast::PrimInt,
    HAST::Label: Debug + Copy + Eq,
    for<'t> types::LendT<'t, HAST>: types::WithHashs + types::WithStats,
{
    let start = Instant::now();
    let now = Instant::now();

    let mapper: (HAST, (DS<HAST>, DS<HAST>)) = hyperast.decompress_pair(src, dst);
    let mut mapper_owned: Mapper<_, DS<HAST>, DS<HAST>, VecStore<_>> = mapper.into();

    let mapper = Mapper {
        hyperast,
        mapping: crate::matchers::Mapping {
            src_arena: mapper_owned.mapping.src_arena.as_mut(),
            dst_arena: mapper_owned.mapping.dst_arena.as_mut(),
            mappings: mapper_owned.mapping.mappings,
        },
    };
    let subtree_prepare_time = now.elapsed();
    let subtree_prepare_t = subtree_prepare_time.as_secs_f64();
    tr!(subtree_prepare_t);
    let leaves_start = Instant::now();
    let (mapper, leaves_matcher_metrics) =
        OptimizedLeavesMatcher::with_config_and_metrics(mapper, config.leaves_matcher);
    let leaves_matcher_time = leaves_start.elapsed();
    let leaves_matcher_t = leaves_matcher_time.as_secs_f64();
    let leaves_mappings_s = mapper.mappings().len();
    tr!(leaves_matcher_t, leaves_mappings_s);

    let bottomup_start = Instant::now();
    let (mapper, bottomup_matcher_metrics) =
        OptimizedBottomUpMatcher::with_config_and_metrics(mapper, config.bottom_up_matcher);
    let bottomup_matcher_time = bottomup_start.elapsed();
    let bottomup_matcher_t = bottomup_matcher_time.as_secs_f64();
    let bottomup_mappings_s = mapper.mappings().len();
    tr!(bottomup_matcher_t, bottomup_mappings_s);

    let (actions, prepare_gen_t, gen_t, mapper) = if config.calculate_script {
        let now = Instant::now();

        let mapper = mapper.map(
            |x| x,
            |dst_arena| SimpleBfsMapper::with_store(hyperast, dst_arena),
        );
        let prepare_gen_t = now.elapsed().as_secs_f64();
        let now = Instant::now();
        let actions = ScriptGenerator::compute_actions(hyperast, &mapper.mapping).ok();
        let gen_t = now.elapsed().as_secs_f64();
        tr!(prepare_gen_t, gen_t);

        let mapper = Mapper {
            hyperast,
            mapping: crate::matchers::Mapping {
                mappings: mapper.mapping.mappings,
                src_arena: mapper_owned.mapping.src_arena,
                dst_arena: mapper_owned.mapping.dst_arena,
            },
        };
        let mapper = mapper.map(
            |src_arena| {
                Decompressible::<HAST, CompletePostOrder<HAST::IdN, _>>::from(
                    src_arena.map(|x| x.complete(hyperast)),
                )
            },
            |dst_arena| {
                let complete = Decompressible::<HAST, CompletePostOrder<HAST::IdN, _>>::from(
                    dst_arena.map(|x| x.complete(hyperast)),
                );
                SimpleBfsMapper::with_store(hyperast, complete)
            },
        );

        let mapper = mapper.map(|x| x, |dst_arena| dst_arena.back);
        (actions, prepare_gen_t, gen_t, mapper)
    } else {
        let mapper = Mapper {
            hyperast,
            mapping: crate::matchers::Mapping {
                mappings: mapper.mapping.mappings,
                src_arena: mapper_owned.mapping.src_arena,
                dst_arena: mapper_owned.mapping.dst_arena,
            },
        };
        let mapper = mapper.map(
            |src_arena| {
                Decompressible::<HAST, CompletePostOrder<HAST::IdN, _>>::from(
                    src_arena.map(|x| x.complete(hyperast)),
                )
            },
            |dst_arena| {
                let complete = Decompressible::<HAST, CompletePostOrder<HAST::IdN, _>>::from(
                    dst_arena.map(|x| x.complete(hyperast)),
                );
                SimpleBfsMapper::with_store(hyperast, complete)
            },
        );

        let mapper = mapper.map(|x| x, |dst_arena| dst_arena.back);
        (None, 0.0, 0.0, mapper)
    };

    CDResult {
        total_time: start.elapsed(),
        mapping_durations: PreparedMappingDurations {
            mappings: MappingDurations([leaves_matcher_t, bottomup_matcher_t]),
            preparation: [subtree_prepare_t, 0.0],
        },
        mapper,
        actions,
        prepare_gen_t,
        gen_t,

        // Detailed timing metrics from actual measurements
        leaves_matcher_metrics,
        bottomup_matcher_metrics,

        produced_mappings: bottomup_mappings_s,
        subtree_prepare_time,
    }
}

/// Execute diff with complete decompression (verbose baseline algorithm)
pub fn diff_with_complete_decompression<HAST: HyperAST + Copy>(
    hyperast: HAST,
    src: &HAST::IdN,
    dst: &HAST::IdN,
    config: OptimizedDiffConfig,
) -> CDResult<
    SimpleAction<HAST::Label, CompressedTreePath<HAST::Idx>, HAST::IdN>,
    Mapper<HAST, CDS<HAST>, CDS<HAST>, VecStore<u32>>,
    PreparedMappingDurations<2>,
>
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Idx: hyperast::PrimInt,
    HAST::Label: Debug + Copy + Eq,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
    let start = Instant::now();
    log::debug!(
        "Starting Optimized ChangeDistiller Algorithm with complete decompression. Preparing subtrees"
    );
    let now = Instant::now();
    let mapper: Mapper<_, CDS<HAST>, CDS<HAST>, VecStore<_>> =
        hyperast.decompress_pair(src, dst).into();
    let subtree_prepare_time = now.elapsed();
    let subtree_prepare_t = subtree_prepare_time.as_secs_f64();
    log::debug!("Subtree prepare time: {}", subtree_prepare_t);

    log::debug!("Starting LeavesMatcher (baseline)");
    let leaves_start = Instant::now();
    let (mapper, leaves_matcher_metrics) =
        LeavesMatcher::with_config_and_metrics(mapper, config.leaves_matcher);
    let leaves_matcher_time = leaves_start.elapsed();
    let leaves_matcher_t = leaves_matcher_time.as_secs_f64();
    let leaves_mappings_s = mapper.mappings().len();
    log::debug!(
        "LeavesMatcher time: {}, Leaves mappings: {}",
        leaves_matcher_t,
        leaves_mappings_s
    );

    log::debug!("Starting BottomUpMatcher (baseline)");
    let bottomup_start = Instant::now();
    let (mapper, bottomup_matcher_metrics) =
        BottomUpMatcher::with_config_and_metrics(mapper, config.bottom_up_matcher);
    let bottomup_matcher_time = bottomup_start.elapsed();
    let bottomup_matcher_t = bottomup_matcher_time.as_secs_f64();
    let bottomup_mappings_s = mapper.mappings().len();
    log::debug!(
        "Bottom-up matcher time: {}, Bottom-up mappings: {}",
        bottomup_matcher_t,
        bottomup_mappings_s
    );

    let (actions, prepare_gen_t, gen_t, mapper) = if config.calculate_script {
        log::debug!("Starting script generation");
        let now = Instant::now();

        let mapper = mapper.map(
            |x| x,
            |dst_arena| SimpleBfsMapper::with_store(hyperast, dst_arena),
        );
        let prepare_gen_t = now.elapsed().as_secs_f64();
        let now = Instant::now();
        let actions = ScriptGenerator::compute_actions(hyperast, &mapper.mapping).ok();
        let gen_t = now.elapsed().as_secs_f64();
        log::debug!("Script generator time: {}", gen_t);
        log::debug!("Prepare generator time: {}", prepare_gen_t);
        let mapper = mapper.map(|x| x, |dst_arena| dst_arena.back);
        (actions, prepare_gen_t, gen_t, mapper)
    } else {
        (None, 0.0, 0.0, mapper)
    };

    let total_time = start.elapsed();

    CDResult {
        mapping_durations: PreparedMappingDurations {
            mappings: MappingDurations([leaves_matcher_t, bottomup_matcher_t]),
            preparation: [subtree_prepare_t, 0.0],
        },
        mapper,
        actions,
        prepare_gen_t,
        gen_t,
        total_time,

        // Detailed timing metrics - baseline algorithm has limited metrics
        leaves_matcher_metrics,

        bottomup_matcher_metrics,

        produced_mappings: bottomup_mappings_s,
        subtree_prepare_time,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matchers::heuristic::cd::{BottomUpMatcherConfig, LeavesMatcherConfig};
    use crate::tests::examples::example_simple;
    use crate::tree::simple_tree::vpair_to_stores;
    use crate::{OptimizedBottomUpMatcherConfig, OptimizedLeavesMatcherConfig};

    #[test]
    fn test_optimized_diff_all_optimizations() {
        let (stores, src, dst) = vpair_to_stores(example_simple());

        let hyperast = &stores;
        let src = &src;
        let dst = &dst;
        let config = OptimizedDiffConfig::default();
        let result = if config.use_lazy_decompression {
            diff_with_lazy_decompression(hyperast, src, dst, config)
        } else {
            diff_with_complete_decompression(hyperast, src, dst, config)
        }
        .into_diff_result();

        assert!(result.mapper.mappings().len() > 0);
        assert!(result.actions.is_some());
    }

    #[test]
    fn test_optimized_diff_baseline() {
        let (stores, src, dst) = vpair_to_stores(example_simple());

        let hyperast = &stores;
        let src = &src;
        let dst = &dst;
        let config = OptimizedDiffConfig::baseline();
        let result = if config.use_lazy_decompression {
            diff_with_lazy_decompression(hyperast, src, dst, config)
        } else {
            diff_with_complete_decompression(hyperast, src, dst, config)
        }
        .into_diff_result();

        assert!(result.mapper.mappings().len() > 0);
        assert!(result.actions.is_some());
    }

    #[test]
    fn test_optimized_diff_custom_config() {
        let (stores, src, dst) = vpair_to_stores(example_simple());

        let config = OptimizedDiffConfig {
            use_lazy_decompression: true,
            use_ranged_similarity: true,
            calculate_script: false,
            leaves_matcher: OptimizedLeavesMatcherConfig {
                base_config: LeavesMatcherConfig::default(),
                enable_label_caching: true,
                enable_deep_leaves: false,
                enable_ngram_caching: false,
                statement_level_iteration: true,
            },
            bottom_up_matcher: OptimizedBottomUpMatcherConfig {
                base: BottomUpMatcherConfig::default(),
                enable_deep_leaves: false,
                statement_level_iteration: false,
                enable_leaf_count_precomputation: false,
            },
        };

        let hyperast = &stores;
        let src = &src;
        let dst = &dst;
        let result = if config.use_lazy_decompression {
            diff_with_lazy_decompression(hyperast, src, dst, config)
        } else {
            diff_with_complete_decompression(hyperast, src, dst, config)
        }
        .into_diff_result();

        assert!(result.mapper.mappings().len() > 0);
        assert!(result.actions.is_none()); // Script generation disabled
    }
}
