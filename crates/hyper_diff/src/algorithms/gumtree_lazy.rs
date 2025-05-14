use super::tr;
use super::{DiffResult, PreparedMappingDurations};
use crate::algorithms::MappingDurations;
use crate::{
    actions::script_generator2::{ScriptGenerator, SimpleAction},
    decompressed_tree_store::{
        CompletePostOrder, bfs_wrapper::SimpleBfsMapper, lazy_post_order::LazyPostOrder,
    },
    matchers::{
        Decompressible, Mapper,
        heuristic::gt::{
            lazy2_greedy_bottom_up_matcher::GreedyBottomUpMatcher,
            lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher,
        },
        mapping_store::{DefaultMultiMappingStore, MappingStore, VecStore},
    },
    tree::tree_path::CompressedTreePath,
};
use hyperast::types::{self, HyperAST, HyperASTShared, NodeId};
use std::{fmt::Debug, time::Instant};

#[allow(type_alias_bounds)]
type DS<HAST: HyperASTShared> = Decompressible<HAST, LazyPostOrder<HAST::IdN, u32>>;
#[allow(type_alias_bounds)]
type CDS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;
type M = VecStore<u32>;
type MM = DefaultMultiMappingStore<u32>;

pub fn diff<HAST: HyperAST + Copy>(
    hyperast: HAST,
    src: &HAST::IdN,
    dst: &HAST::IdN,
) -> DiffResult<
    SimpleAction<HAST::Label, CompressedTreePath<HAST::Idx>, HAST::IdN>,
    Mapper<HAST, CDS<HAST>, CDS<HAST>, M>,
    PreparedMappingDurations<2>,
>
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
    let subtree_prepare_t = now.elapsed().as_secs_f64();
    tr!(subtree_prepare_t);

    let now = Instant::now();
    let mapper = LazyGreedySubtreeMatcher::<_, _, _, M>::match_it::<MM>(mapper);
    let subtree_matcher_t = now.elapsed().as_secs_f64();
    let subtree_mappings_s = mapper.mappings().len();
    tr!(subtree_matcher_t, subtree_mappings_s);

    let bottomup_prepare_t = 0.; // nothing to prepare

    let now = Instant::now();
    let mapper = GreedyBottomUpMatcher::<_, _, _, _>::match_it(mapper);
    let bottomup_matcher_t = now.elapsed().as_secs_f64();
    let bottomup_mappings_s = mapper.mappings().len();
    tr!(bottomup_matcher_t, bottomup_mappings_s);
    let mapping_durations = PreparedMappingDurations {
        mappings: MappingDurations([subtree_matcher_t, bottomup_matcher_t]),
        preparation: [subtree_prepare_t, bottomup_prepare_t],
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
    let prepare_gen_t = now.elapsed().as_secs_f64();
    tr!(prepare_gen_t);

    let now = Instant::now();
    let actions = ScriptGenerator::compute_actions(mapper.hyperast, &mapper.mapping).ok();
    let gen_t = now.elapsed().as_secs_f64();
    tr!(gen_t);

    // drop the bfs wrapper
    let mapper = mapper.map(|x| x, |dst_arena| dst_arena.back);

    DiffResult {
        mapping_durations,
        mapper,
        actions,
        prepare_gen_t,
        gen_t,
    }
}
