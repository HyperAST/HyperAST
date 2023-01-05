use std::{fmt::Debug, time::Instant};

use hyper_ast::types;
use hyper_gumtree::{
    actions::script_generator2::ScriptGenerator,
    decompressed_tree_store::{
        bfs_wrapper::SimpleBfsMapper, lazy_post_order::LazyPostOrder, CompletePostOrder,
    },
    matchers::{
        heuristic::gt::{
            lazy_bottom_up_matcher::BottomUpMatcher,
            lazy_greedy_bottom_up_matcher::GreedyBottomUpMatcher,
            lazy_greedy_subtree_matcher::{LazyGreedySubtreeMatcher, SubtreeMatcher},
        },
        mapping_store::{DefaultMultiMappingStore, MappingStore, VecStore},
    },
    tree::tree_path::CompressedTreePath,
};

type DS<T> = LazyPostOrder<T, u32>;
type CDS<T> = CompletePostOrder<T, u32>;

use super::DiffResult;

pub fn diff<'store, IdN, NS, LS>(
    node_store: &'store NS,
    label_store: &'store LS,
    src: &'store IdN,
    dst: &'store IdN,
) -> DiffResult<
    IdN,
    LS::I,
    CompressedTreePath<<NS::R<'store> as types::WithChildren>::ChildIdx>,
    u32,
    CDS<NS::R<'store>>,
    CDS<NS::R<'store>>,
    2,
>
where
    IdN: Clone + Debug + Eq,
    LS::I: Debug,
    <NS::R<'store> as types::Typed>::Type: Debug,
    <NS::R<'store> as types::WithChildren>::ChildIdx: Debug,
    NS: types::NodeStore<IdN>,
    LS: types::LabelStore<str>,
    NS::R<'store>: types::Tree<Type = types::Type, TreeId = IdN, Label = LS::I>
        + types::WithHashs
        + types::WithStats,
{
    let mappings = VecStore::default();
    let now = Instant::now();
    let mapper = LazyGreedySubtreeMatcher::<DS<NS::R<'store>>, DS<NS::R<'store>>, _, _, _>::matchh::<
        DefaultMultiMappingStore<_>,
    >(node_store, src, dst, mappings);
    let SubtreeMatcher {
        src_arena,
        dst_arena,
        mappings,
        ..
    } = mapper.into();
    let subtree_matcher_t = now.elapsed().as_secs_f64();
    let subtree_mappings_s = mappings.len();
    dbg!(&subtree_matcher_t, &subtree_mappings_s);
    let now = Instant::now();
    let mut mapper = GreedyBottomUpMatcher::<_, _, _, _, _, _, VecStore<_>>::new(
        node_store,
        label_store,
        src_arena,
        dst_arena,
        mappings,
    );
    dbg!(&now.elapsed().as_secs_f64());
    mapper.execute();
    dbg!(&now.elapsed().as_secs_f64());
    let BottomUpMatcher {
        src_arena,
        dst_arena,
        mappings,
        ..
    } = mapper.into();
    dbg!(&now.elapsed().as_secs_f64());
    let bottomup_matcher_t = now.elapsed().as_secs_f64();
    let bottomup_mappings_s = mappings.len();
    dbg!(&bottomup_matcher_t, &bottomup_mappings_s);
    let now = Instant::now();
    let src_arena = src_arena.complete(node_store);
    let src_arena = CompletePostOrder::from(src_arena);
    let dst_arena = dst_arena.complete(node_store);
    let dst_arena = CompletePostOrder::from(dst_arena);
    let dst_arena_bfs = SimpleBfsMapper::from(node_store, dst_arena);
    let ScriptGenerator { actions, .. } =
        ScriptGenerator::precompute_actions(node_store, &src_arena, &dst_arena_bfs, &mappings)
            .generate();
    let gen_t = now.elapsed().as_secs_f64();
    dbg!(gen_t);
    let dst_arena = dst_arena_bfs.back;
    DiffResult {
        mapping_durations: [subtree_matcher_t, bottomup_matcher_t],
        src_arena,
        dst_arena,
        mappings,
        actions,
        gen_t,
    }
}
