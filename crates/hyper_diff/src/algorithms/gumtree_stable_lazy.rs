use super::tr;
use super::{DiffResult, MappingDurations, PreparedMappingDurations};
use super::{MappingMemoryUsages, get_allocated_memory};
use std::{fmt::Debug, time::Instant};

use super::CDS;
use super::DiffRes;
use crate::actions::script_generator2::ScriptGenerator;
use crate::decompressed_tree_store::bfs_wrapper::SimpleBfsMapper;
use crate::matchers::Mapper;
use crate::matchers::mapping_store::{DefaultMultiMappingStore, MappingStore, VecStore};
use hyperast::types::{self, HyperAST, NodeId};

// use crate::decompressed_tree_store::lazy_post_order::LazyPostOrder;
use super::DS;

use crate::matchers::heuristic::gt::lazy_marriage_bottom_up_matcher::LazyMarriageBottomUpMatcher;
use crate::matchers::heuristic::gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher;

type M = VecStore<u32>;
type MM = DefaultMultiMappingStore<u32>;

pub fn diff<HAST: HyperAST + Copy>(
    hyperast: HAST,
    src: &HAST::IdN,
    dst: &HAST::IdN,
) -> DiffRes<HAST>
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Label: Clone + Copy + Eq + Debug,
    HAST::Idx: hyperast::PrimInt,
    for<'t> types::LendT<'t, HAST>: types::WithHashs + types::WithStats,
{
    let now = Instant::now();
    let mut mapper_owned: (DS<HAST>, DS<HAST>) = hyperast.decompress_pair(src, dst).1;
    let mapper = Mapper::with_mut_decompressible(&mut mapper_owned);
    let subtree_prepare_t = now.elapsed();
    tr!(subtree_prepare_t);

    let mem = get_allocated_memory();
    let now = Instant::now();
    let mapper = LazyGreedySubtreeMatcher::<_, _, _, M>::match_it::<MM>(mapper);
    let subtree_matcher_t = now.elapsed();
    let subtree_mappings_s = mapper.mappings().len();
    let subtree_matcher_m = get_allocated_memory().saturating_sub(mem);
    tr!(subtree_matcher_t, subtree_mappings_s);

    let bottomup_prepare_t = std::time::Duration::ZERO; // nothing to prepare

    let mem = get_allocated_memory();
    let now = Instant::now();
    let mapper = LazyMarriageBottomUpMatcher::<_, _, _, _, M, 300>::match_it(mapper);
    let bottomup_matcher_t = now.elapsed();
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
    // Must fully decompress the subtrees to compute default chawathe
    let mapper = Mapper::new(hyperast, mapper.mapping.mappings, mapper_owned);
    let mapper = mapper.map(
        |src_arena| CDS::<_>::from(src_arena.map(|x| x.complete(hyperast))),
        |dst_arena| {
            let complete = CDS::<_>::from(dst_arena.map(|x| x.complete(hyperast)));
            // the dst side has to be traversed in bfs for chawathe
            SimpleBfsMapper::with_store(hyperast, complete)
        },
    );
    let prepare_gen_t = now.elapsed();
    tr!(prepare_gen_t);

    let now = Instant::now();
    let actions = ScriptGenerator::compute_actions(mapper.hyperast, &mapper.mapping).ok();
    let gen_t = now.elapsed();
    tr!(gen_t);

    // drop the bfs wrapper
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
