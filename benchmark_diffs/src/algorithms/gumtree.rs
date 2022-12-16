use std::time::Instant;

use hyper_ast::store::{
    defaults::NodeIdentifier,
    nodes::legion::{HashedNodeRef},
    SimpleStores,
};
use hyper_gumtree::{
    actions::script_generator2::ScriptGenerator,
    decompressed_tree_store::{bfs_wrapper::SimpleBfsMapper, CompletePostOrder},
    matchers::{
        heuristic::gt::{
            bottom_up_matcher::BottomUpMatcher,
            greedy_bottom_up_matcher::GreedyBottomUpMatcher,
            greedy_subtree_matcher::{GreedySubtreeMatcher, SubtreeMatcher},
        },
        mapping_store::{MappingStore, VecStore},
    },
};

type DS<'store> = CompletePostOrder<HashedNodeRef<'store>, u32>;

use super::DiffResult;

pub fn diff<'store>(stores: &'store SimpleStores, src: &'store NodeIdentifier, dst: &'store NodeIdentifier) -> DiffResult<u32, DS<'store>, DS<'store>,2> {
    let mappings = VecStore::default();
    let now = Instant::now();
    let mapper = GreedySubtreeMatcher::<DS, DS, _, HashedNodeRef, _, _>::matchh(
        &stores.node_store,
        &src,
        &dst,
        mappings,
    );
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
    let mut mapper = GreedyBottomUpMatcher::<DS, DS, _, HashedNodeRef, _, _, _>::new(
        &stores.node_store,
        &stores.label_store,
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
    let dst_arena_bfs = SimpleBfsMapper::from(&stores.node_store, dst_arena);
    let ScriptGenerator { actions, .. } = ScriptGenerator::precompute_actions(
        &stores.node_store,
        &src_arena,
        &dst_arena_bfs,
        &mappings,
    )
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
