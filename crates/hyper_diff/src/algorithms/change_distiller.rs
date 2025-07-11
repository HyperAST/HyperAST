use super::tr;
use super::{DiffResult, MappingDurations, PreparedMappingDurations};
use super::{MappingMemoryUsages, get_allocated_memory};
use std::{fmt::Debug, time::Instant};

use super::CDS;
use super::DiffRes;
use crate::actions::script_generator2::ScriptGenerator;
use crate::decompressed_tree_store::bfs_wrapper::SimpleBfsMapper;
use crate::matchers::Mapper;
use crate::matchers::mapping_store::{MappingStore, VecStore};
use hyperast::types::{self, HyperAST, NodeId};

use hyperast::store::nodes::compo;
use hyperast::types::WithMetaData;

use crate::matchers::heuristic::cd::bottom_up_matcher::BottomUpMatcher;
use crate::matchers::heuristic::cd::leaves_matcher::LeavesMatcher;

pub fn diff<HAST: HyperAST + Copy>(
    hyperast: HAST,
    src: &HAST::IdN,
    dst: &HAST::IdN,
) -> DiffRes<HAST>
where
    HAST::IdN: Copy + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Idx: hyperast::PrimInt,
    HAST::Label: Debug + Clone + Copy + Eq,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs
        + types::WithStats
        + WithMetaData<compo::MemberImportCount>
        + WithMetaData<compo::StmtCount>,
{
    let mem = get_allocated_memory();
    let now = Instant::now();
    let mapper: Mapper<_, CDS<HAST>, CDS<HAST>, VecStore<_>> =
        hyperast.decompress_pair(src, dst).into();
    let subtree_prepare_t = now.elapsed().into();
    tr!(subtree_prepare_t);

    let now = Instant::now();
    let mapper = LeavesMatcher::<_, _, _, _>::match_it(mapper);
    let subtree_matcher_t = now.elapsed().into();
    let subtree_mappings_s = mapper.mappings().len();
    let subtree_matcher_m = get_allocated_memory().saturating_sub(mem);
    tr!(subtree_matcher_t, subtree_mappings_s);

    let bottomup_prepare_t = std::time::Duration::ZERO.into(); // nothing to prepare

    let mem = get_allocated_memory();
    let now = Instant::now();
    let mapper = BottomUpMatcher::<_, _, _, _>::match_it(mapper);
    dbg!(&now.elapsed());
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
