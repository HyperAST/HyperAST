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

// use crate::decompressed_tree_store::lazy_post_order::LazyPostOrder;
use super::DS;

use hyperast::store::nodes::compo;
use hyperast::types::WithMetaData;

use crate::matchers::heuristic::cd::bottom_up_matcher::BottomUpMatcher;
use crate::matchers::heuristic::cd::lazy_leaves_matcher::LazyLeavesMatcher;

type M = VecStore<u32>;

pub fn diff<HAST: HyperAST + Copy>(
    hyperast: HAST,
    src: &HAST::IdN,
    dst: &HAST::IdN,
) -> DiffRes<HAST>
where
    HAST::IdN: Copy + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Label: Clone + Copy + Eq + Debug,
    HAST::Idx: hyperast::PrimInt,
    <HAST::TS as types::TypeStore>::Ty: Eq + Debug,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs
        + types::WithStats
        + WithMetaData<compo::MemberImportCount>
        + WithMetaData<compo::StmtCount>,
{
    let mem = get_allocated_memory();
    let now = Instant::now();
    let mut mapper_owned: (DS<HAST>, DS<HAST>) = hyperast.decompress_pair(src, dst).1;
    let mapper = Mapper::with_mut_decompressible(&mut mapper_owned);
    let subtree_prepare_t = now.elapsed().into();
    tr!(subtree_prepare_t);

    let now = Instant::now();
    let mapper = LazyLeavesMatcher::<_, _, _, M>::match_it(mapper);
    let subtree_matcher_t = now.elapsed().into();
    let subtree_mappings_s = mapper.mappings().len();
    let subtree_matcher_m = get_allocated_memory().saturating_sub(mem);
    tr!(subtree_matcher_t, subtree_mappings_s);

    let now = Instant::now();
    // Must fully decompress the subtrees to compute the non-lazy bottomup
    let mapper = Mapper::new(hyperast, mapper.mapping.mappings, mapper_owned);
    let mapper = mapper.map(
        |src_arena| CDS::<_>::from(src_arena.map(|x| x.complete(hyperast))),
        |dst_arena| CDS::<_>::from(dst_arena.map(|x| x.complete(hyperast))),
    );
    let bottomup_prepare_t = now.elapsed().into();
    tr!(bottomup_prepare_t);

    let mem = get_allocated_memory();
    let now = Instant::now();
    let mapper = BottomUpMatcher::<_, _, _, _>::match_it(mapper);
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
    let actions = ScriptGenerator::compute_actions(mapper.hyperast, &mapper.mapping).ok();
    let gen_t = now.elapsed().into();
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
