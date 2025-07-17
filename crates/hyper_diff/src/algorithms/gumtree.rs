use super::DiffResult;
use super::tr;
use std::fmt::Debug;

use super::CDS;
use super::DiffRes;
use crate::actions::script_generator2::ScriptGenerator;
use crate::algorithms::check_oneshot_decompressed_against_lazy;
use crate::decompressed_tree_store::bfs_wrapper::SimpleBfsMapper;
use crate::matchers::Mapper;
use crate::matchers::mapping_store::{DefaultMultiMappingStore, MappingStore, VecStore};
use hyperast::types::{self, HyperAST, NodeId};

use crate::matchers::heuristic::gt::greedy_bottom_up_matcher::GreedyBottomUpMatcher;
use crate::matchers::heuristic::gt::greedy_subtree_matcher::GreedySubtreeMatcher;

type MM = DefaultMultiMappingStore<u32>;

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
    let measure = super::DefaultMetricSetup::prepare();
    let mapper: Mapper<_, CDS<HAST>, CDS<HAST>, VecStore<_>> =
        hyperast.decompress_pair(src, dst).into();
    if cfg!(debug_assertions) {
        check_oneshot_decompressed_against_lazy(hyperast, src, dst, &mapper);
    }
    let measure = measure.start();

    let mapper = GreedySubtreeMatcher::<_, _, _, _>::match_it::<MM>(mapper);
    let subtree_mappings_s = mapper.mappings().len();
    tr!(subtree_mappings_s);

    let measure = measure.stop_then_skip_prepare();

    let mapper = GreedyBottomUpMatcher::<_, _, _, _>::match_it(mapper);
    let bottomup_mappings_s = mapper.mappings().len();
    tr!(bottomup_mappings_s);

    let measure = measure.stop_then_prepare();

    let mapper = mapper.map(
        |x| x,
        // the dst side has to be traversed in bfs for chawathe
        |dst_arena| SimpleBfsMapper::with_store(hyperast, dst_arena),
    );

    let measure = measure.start();
    let actions = ScriptGenerator::compute_actions(hyperast, &mapper.mapping).ok();
    let mapper = mapper.map(|x| x, |dst_arena| dst_arena.back);

    let exec_data = measure.stop();

    DiffResult {
        mapper,
        actions,
        exec_data,
    }
}

pub fn diff_100<HAST: HyperAST + Copy>(
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
    let measure = super::DefaultMetricSetup::prepare();
    let mapper: Mapper<_, CDS<HAST>, CDS<HAST>, VecStore<_>> =
        hyperast.decompress_pair(src, dst).into();
    if cfg!(debug_assertions) {
        check_oneshot_decompressed_against_lazy(hyperast, src, dst, &mapper);
    }
    let measure = measure.start();

    let mapper =
        GreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
    let subtree_mappings_s = mapper.mappings().len();
    tr!(subtree_mappings_s);

    let measure = measure.stop_then_skip_prepare();

    let mapper = GreedyBottomUpMatcher::<_, _, _, _, 100>::match_it(mapper);

    let measure = measure.stop_then_prepare();

    let mapper = mapper.map(
        |x| x,
        // the dst side has to be traversed in bfs for chawathe
        |dst_arena| SimpleBfsMapper::with_store(hyperast, dst_arena),
    );

    let measure = measure.start();
    let actions = ScriptGenerator::compute_actions(hyperast, &mapper.mapping).ok();
    let mapper = mapper.map(|x| x, |dst_arena| dst_arena.back);

    let exec_data = measure.stop();

    DiffResult {
        mapper,
        actions,
        exec_data,
    }
}

pub fn diff_subtree<HAST: HyperAST + Copy>(
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
    let measure = super::DefaultMetricSetup::prepare();
    let mapper: Mapper<_, CDS<HAST>, CDS<HAST>, VecStore<_>> =
        hyperast.decompress_pair(src, dst).into();
    if cfg!(debug_assertions) {
        check_oneshot_decompressed_against_lazy(hyperast, src, dst, &mapper);
    }
    let measure = measure.start();

    let mapper =
        GreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
    let subtree_mappings_s = mapper.mappings().len();
    tr!(subtree_mappings_s);

    let measure = measure.stop_then_skip_prepare();
    let measure = measure.stop_then_prepare();

    let mapper = mapper.map(
        |x| x,
        // the dst side has to be traversed in bfs for chawathe
        |dst_arena| SimpleBfsMapper::with_store(hyperast, dst_arena),
    );

    let measure = measure.start();
    let actions = ScriptGenerator::compute_actions(hyperast, &mapper.mapping).ok();
    let mapper = mapper.map(|x| x, |dst_arena| dst_arena.back);

    let exec_data = measure.stop();

    DiffResult {
        mapper,
        actions,
        exec_data,
    }
}
