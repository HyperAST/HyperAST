use super::MappingDurations;
use super::{DiffResult, PreparedMappingDurations};
use crate::decompressed_tree_store::lazy_post_order::LazyPostOrder;
use crate::matchers::heuristic::cd::lazy_bottom_up_matcher_2::LazyBottomUpMatcher;
use crate::matchers::heuristic::cd::lazy_leaves_matcher_2::LazyLeavesMatcher;
use crate::matchers::heuristic::cd::{BottomUpMatcherConfig, LeavesMatcherConfig};
use crate::{
    actions::script_generator2::{ScriptGenerator, SimpleAction},
    decompressed_tree_store::{CompletePostOrder, bfs_wrapper::SimpleBfsMapper},
    matchers::{
        Decompressible, Mapper,
        mapping_store::{MappingStore, VecStore},
    },
    tree::tree_path::CompressedTreePath,
};
use hyperast::types::{self, HyperAST, HyperASTShared, NodeId};
use std::{fmt::Debug, time::Instant};

type DS<HAST: HyperASTShared> = Decompressible<HAST, LazyPostOrder<HAST::IdN, u32>>;
type CDS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;

pub fn diff<HAST: HyperAST + Copy>(
    hyperast: HAST,
    src: &HAST::IdN,
    dst: &HAST::IdN,
) -> DiffResult<
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
    diff_with_config(
        hyperast,
        src,
        dst,
        false,
        BottomUpMatcherConfig::default(),
        LeavesMatcherConfig::default(),
    )
}

pub fn diff_with_config<HAST: HyperAST + Copy>(
    hyperast: HAST,
    src: &HAST::IdN,
    dst: &HAST::IdN,
    calculate_script: bool,
    bottom_up_config: BottomUpMatcherConfig,
    leaves_config: LeavesMatcherConfig,
) -> DiffResult<
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
    log::debug!("Starting Lazy ChangeDistiller Algorithm. Preparing subtrees");
    let now = Instant::now();
    // let mapper: Mapper<_, DS<HAST>, DS<HAST>, VecStore<_>> =
    // hyperast.decompress_pair(src, dst).into();
    let mapper: (HAST, (DS<HAST>, DS<HAST>)) = hyperast.decompress_pair(src, dst);
    let mut mapper_owned: Mapper<_, DS<HAST>, DS<HAST>, VecStore<_>> = mapper.into();
    // TODO find better way, at least make a shorthand

    let mapper = Mapper {
        hyperast,
        mapping: crate::matchers::Mapping {
            src_arena: mapper_owned.mapping.src_arena.as_mut(),
            dst_arena: mapper_owned.mapping.dst_arena.as_mut(),
            mappings: mapper_owned.mapping.mappings,
        },
    };
    let subtree_prepare_t = now.elapsed().as_secs_f64();
    log::debug!("Subtree prepare time: {}", subtree_prepare_t);
    log::debug!("Starting LazyLeavesMatcher");
    let now = Instant::now();
    let mapper = LazyLeavesMatcher::<_, _, _, _>::with_config(mapper, leaves_config);
    let leaves_matcher_t = now.elapsed().as_secs_f64();
    let leaves_mappings_s = mapper.mappings().len();
    log::debug!(
        "LeavesMatcher time: {}, Leaves mappings: {}",
        leaves_matcher_t,
        leaves_mappings_s
    );
    log::debug!("Starting LazyBottomUpMatcher");
    let now = Instant::now();
    let mapper = LazyBottomUpMatcher::<_, _, _, _>::with_config(mapper, bottom_up_config);
    let bottomup_matcher_t = now.elapsed().as_secs_f64();
    let bottomup_mappings_s = mapper.mappings().len();
    log::debug!(
        "Bottom-up matcher time: {}, Bottom-up mappings: {}",
        bottomup_matcher_t,
        bottomup_mappings_s
    );

    let (actions, prepare_gen_t, gen_t, mapper) = if calculate_script {
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

        // TODO find better way, at least make a shorthand
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
        // Skip script generation if not requested
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

    DiffResult {
        mapping_durations: PreparedMappingDurations {
            mappings: MappingDurations([leaves_matcher_t, bottomup_matcher_t]),
            preparation: [subtree_prepare_t, 0.0],
        },
        mapper,
        actions,
        prepare_gen_t,
        gen_t,
    }
}
