use super::MappingDurations;
use super::{DiffResult, PreparedMappingDurations};
use crate::{
    actions::script_generator2::{ScriptGenerator, SimpleAction},
    decompressed_tree_store::{bfs_wrapper::SimpleBfsMapper, CompletePostOrder},
    matchers::{
        heuristic::gt::{
            greedy_bottom_up_matcher::GreedyBottomUpMatcher,
            greedy_subtree_matcher::GreedySubtreeMatcher,
        },
        mapping_store::{DefaultMultiMappingStore, MappingStore, VecStore},
        Decompressible, Mapper,
    },
    tree::tree_path::CompressedTreePath,
};
use hyperast::types::{self, HyperAST, HyperASTShared, NodeId};
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
    HAST::Label: Debug + Clone + Copy + Eq,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
    let now = Instant::now();
    let mapper: Mapper<_, CDS<HAST>, CDS<HAST>, VecStore<_>> =
        hyperast.decompress_pair(src, dst).into();
    let subtree_prepare_t = now.elapsed().as_secs_f64();
    let now = Instant::now();
    let mapper =
        GreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
    let subtree_matcher_t = now.elapsed().as_secs_f64();
    let subtree_mappings_s = mapper.mappings().len();
    dbg!(&subtree_matcher_t, &subtree_mappings_s);
    let now = Instant::now();
    let mapper = GreedyBottomUpMatcher::<_, _, _, _>::match_it(mapper);
    dbg!(&now.elapsed().as_secs_f64());
    let bottomup_matcher_t = now.elapsed().as_secs_f64();
    let bottomup_mappings_s = mapper.mappings().len();
    dbg!(&bottomup_matcher_t, &bottomup_mappings_s);
    let now = Instant::now();

    let mapper = mapper.map(
        |x| x,
        |dst_arena| SimpleBfsMapper::with_store(hyperast, dst_arena),
    );
    let prepare_gen_t = now.elapsed().as_secs_f64();
    let now = Instant::now();
    let actions = ScriptGenerator::compute_actions(hyperast, &mapper.mapping).ok();
    let gen_t = now.elapsed().as_secs_f64();
    dbg!(gen_t);
    let mapper = mapper.map(|x| x, |dst_arena| dst_arena.back);
    DiffResult {
        mapping_durations: PreparedMappingDurations {
            mappings: MappingDurations([subtree_matcher_t, bottomup_matcher_t]),
            preparation: [subtree_prepare_t, 0.0],
        },
        mapper,
        actions,
        prepare_gen_t,
        gen_t,
    }
}
