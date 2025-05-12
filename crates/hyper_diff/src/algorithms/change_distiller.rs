use super::MappingDurations;
use super::{DiffResult, PreparedMappingDurations};
use crate::matchers::heuristic::cd::bottom_up_matcher::BottomUpMatcher;
use crate::matchers::heuristic::cd::leaves_matcher::LeavesMatcher;
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
use log::debug;
use std::{fmt::Debug, time::Instant};

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
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
    log::debug!("Starting ChangeDistiller Algorithm. Preparing subtrees");
    let now = Instant::now();
    let mapper: Mapper<_, CDS<HAST>, CDS<HAST>, VecStore<_>> =
        hyperast.decompress_pair(src, dst).into();
    let subtree_prepare_t = now.elapsed().as_secs_f64();
    log::debug!("Subtree prepare time: {}", subtree_prepare_t);
    log::debug!("Starting LeavesMatcher");
    let now = Instant::now();
    let mapper = LeavesMatcher::<_, _, _, _>::match_it(mapper);
    let leaves_matcher_t = now.elapsed().as_secs_f64();
    let leaves_mappings_s = mapper.mappings().len();
    log::debug!(
        "LeavesMatcher time: {}, Leaves mappings: {}",
        leaves_matcher_t,
        leaves_mappings_s
    );
    log::debug!("Starting BottomUpMatcher");
    let now = Instant::now();
    let mapper = BottomUpMatcher::<_, _, _, _>::match_it(mapper);
    let bottomup_matcher_t = now.elapsed().as_secs_f64();
    let bottomup_mappings_s = mapper.mappings().len();
    log::debug!(
        "Bottom-up matcher time: {}, Bottom-up mappings: {}",
        bottomup_matcher_t,
        bottomup_mappings_s
    );
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
