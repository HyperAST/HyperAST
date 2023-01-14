use std::{fmt::Debug, time::Instant};

use hyper_ast::types::{self, HyperAST};
use hyper_gumtree::{
    actions::script_generator2::{ScriptGenerator, SimpleAction},
    decompressed_tree_store::{
        bfs_wrapper::SimpleBfsMapper, lazy_post_order::LazyPostOrder, CompletePostOrder,
    },
    matchers::{
        heuristic::gt::{
            lazy2_greedy_bottom_up_matcher::GreedyBottomUpMatcher,
            lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher,
        },
        mapping_store::{DefaultMultiMappingStore, MappingStore, VecStore},
        Mapper,
    },
    tree::tree_path::CompressedTreePath,
};

type DS<T> = LazyPostOrder<T, u32>;
type CDS<T> = CompletePostOrder<T, u32>;

use crate::algorithms::MappingDurations;

use super::{DiffResult, PreparedMappingDurations};

pub fn diff<'store, HAST: HyperAST<'store>>(
    hyperast: &'store HAST,
    src: &HAST::IdN,
    dst: &HAST::IdN,
) -> DiffResult<
    SimpleAction<
        HAST::Label,
        CompressedTreePath<<HAST::T as types::WithChildren>::ChildIdx>,
        HAST::IdN,
    >,
    Mapper<'store, HAST, CDS<HAST::T>, CDS<HAST::T>, VecStore<u32>>,
    PreparedMappingDurations<2>,
>
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::Label: Clone + Copy + Eq + Debug,
    <HAST::T as types::Typed>::Type: Debug,
    <HAST::T as types::WithChildren>::ChildIdx: Debug,
    HAST::T: 'store + types::WithHashs + types::WithStats,
{
    let now = Instant::now();
    let mapper: Mapper<_, DS<HAST::T>, DS<HAST::T>, VecStore<_>> =
        hyperast.decompress_pair(src, dst).into();
    let subtree_prepare_t = now.elapsed().as_secs_f64();
    let now = Instant::now();
    let mapper =
        LazyGreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
    let subtree_matcher_t = now.elapsed().as_secs_f64();
    let subtree_mappings_s = mapper.mappings().len();
    dbg!(&subtree_matcher_t, &subtree_mappings_s);
    let bottomup_prepare_t = 0.;
    let now = Instant::now();
    let mapper = GreedyBottomUpMatcher::<_, _, _, _, VecStore<_>>::match_it(mapper);
    dbg!(&now.elapsed().as_secs_f64());
    let bottomup_matcher_t = now.elapsed().as_secs_f64();
    let bottomup_mappings_s = mapper.mappings().len();
    dbg!(&bottomup_matcher_t, &bottomup_mappings_s);
    let now = Instant::now();

    let node_store = hyperast.node_store();
    let mapper = mapper.map(
        |src_arena| CompletePostOrder::from(src_arena.complete(node_store)),
        |dst_arena| {
            let complete = CompletePostOrder::from(dst_arena.complete(node_store));
            SimpleBfsMapper::from(node_store, complete)
        },
    );

    let prepare_gen_t = now.elapsed().as_secs_f64();
    let now = Instant::now();
    let actions = ScriptGenerator::compute_actions(mapper.hyperast, &mapper.mapping).ok();
    let gen_t = now.elapsed().as_secs_f64();
    dbg!(gen_t);
    let mapper = mapper.map(|x| x, |dst_arena| dst_arena.back);
    DiffResult {
        mapping_durations: PreparedMappingDurations {
            mappings: MappingDurations([subtree_matcher_t, bottomup_matcher_t]),
            preparation: [subtree_prepare_t, bottomup_prepare_t],
        },
        mapper,
        actions,
        prepare_gen_t,
        gen_t,
    }
}
