use super::tr;
use super::{DiffResult, MappingDurations, PreparedMappingDurations};
use super::{MappingMemoryUsages, get_allocated_memory};
use std::{fmt::Debug, time::Instant};

use super::CDS;
use super::DiffRes;
use crate::actions::script_generator2::ScriptGenerator;
use crate::algorithms::check_oneshot_decompressed_against_lazy;
use crate::decompressed_tree_store::bfs_wrapper::SimpleBfsMapper;
use crate::matchers::Mapper;
use crate::matchers::mapping_store::{DefaultMultiMappingStore, MappingStore, VecStore};
use hyperast::types::{self, HyperAST, NodeId};

use crate::matchers::heuristic::gt::greedy_subtree_matcher::GreedySubtreeMatcher;
use crate::matchers::heuristic::gt::hybrid_bottom_up_matcher::HybridBottomUpMatcher;

type M = VecStore<u32>;

const DEFAULT_MIN_HEIGHT: usize = 1; // todo: make min_height adjustable?

pub fn diff<HAST: HyperAST + Copy>(
    hyperast: HAST,
    src: &HAST::IdN,
    dst: &HAST::IdN,
) -> DiffRes<HAST>
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Idx: hyperast::PrimInt,
    HAST::Label: Debug + Clone + Copy + Eq,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
    diff_hybrid_minheight::<HAST, DEFAULT_MIN_HEIGHT>(hyperast, src, dst)
}

pub fn diff_with_hyperparameters<
    HAST: HyperAST + Copy,
    const MIN_HEIGHT: usize,
    const SIZE_THRESHOLD: usize,
    const SIM_THRESHOLD_NUM: u64,
    const SIM_THRESHOLD_DEN: u64,
>(
    hyperast: HAST,
    src: &HAST::IdN,
    dst: &HAST::IdN,
) -> DiffRes<HAST>
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Idx: hyperast::PrimInt,
    HAST::Label: Debug + Clone + Copy + Eq,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
    let mem = get_allocated_memory();
    let now = Instant::now();
    let mapper: Mapper<_, CDS<HAST>, CDS<HAST>, VecStore<_>> =
        hyperast.decompress_pair(src, dst).into();
    if cfg!(debug_assertions) {
        check_oneshot_decompressed_against_lazy(hyperast, src, dst, &mapper);
    }
    let subtree_prepare_t = now.elapsed().into();
    tr!(subtree_prepare_t);

    let now = Instant::now();
    let mapper = GreedySubtreeMatcher::<_, _, _, _, MIN_HEIGHT>::match_it::<
        DefaultMultiMappingStore<_>,
    >(mapper);
    let subtree_matcher_t = now.elapsed().into();
    let subtree_mappings_s = mapper.mappings().len();
    let subtree_matcher_m = get_allocated_memory().saturating_sub(mem);
    tr!(subtree_matcher_t, subtree_mappings_s);

    let bottomup_prepare_t = std::time::Duration::ZERO.into(); // nothing to prepare

    let mem = get_allocated_memory();
    let now = Instant::now();
    let mapper = HybridBottomUpMatcher::<
        _,
        _,
        _,
        _,
        M,
        SIZE_THRESHOLD,
        SIM_THRESHOLD_NUM,
        SIM_THRESHOLD_DEN,
    >::match_it(mapper);
    dbg!(&now.elapsed().as_secs_f64());
    let bottomup_matcher_t = now.elapsed().into();
    let bottomup_mappings_s = mapper.mappings().len();
    let bottomup_matcher_m = get_allocated_memory().saturating_sub(mem);

    tr!(bottomup_matcher_t, bottomup_mappings_s);
    let mapping_durations = PreparedMappingDurations {
        mappings: MappingDurations([subtree_matcher_t, bottomup_matcher_t]),
        preparation: [subtree_prepare_t, bottomup_prepare_t],
    };
    let mapping_memory_usages = MappingMemoryUsages {
        memory: [subtree_matcher_m, bottomup_matcher_m],
    };

    let now = Instant::now();
    let mapper = mapper.map(
        |x| x,
        // the dst side has to be traversed in bfs for chawathe
        |dst_arena| SimpleBfsMapper::with_store(hyperast, dst_arena),
    );
    let prepare_gen_t = now.elapsed().into();
    tr!(prepare_gen_t);
    let now = Instant::now();
    let actions = ScriptGenerator::compute_actions(hyperast, &mapper.mapping).ok();
    let gen_t = now.elapsed().into();
    tr!(gen_t);
    let mapper = mapper.map(|x| x, |dst_arena| dst_arena.back);
    DiffResult {
        mapping_durations,
        mapping_memory_usages,
        mapper,
        actions,
        prepare_gen_t,
        gen_t,
    }
}

pub fn diff_hybrid_minheight<HAST: HyperAST + Copy, const MIN_HEIGHT: usize>(
    hyperast: HAST,
    src: &HAST::IdN,
    dst: &HAST::IdN,
) -> DiffRes<HAST>
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Idx: hyperast::PrimInt,
    HAST::Label: Debug + Clone + Copy + Eq,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
    diff_with_hyperparameters::<HAST, MIN_HEIGHT, 100, 1, 2>(hyperast, src, dst)
}
