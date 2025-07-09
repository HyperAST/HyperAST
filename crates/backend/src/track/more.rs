use super::*;

use hyper_diff::{
    decompressed_tree_store::lazy_post_order::LazyPostOrder,
    matchers::{Decompressible, Mapping},
};

// WARN lazy subtrees are not complete
fn lazy_mapping<'a>(
    repositories: &'a multi_preprocessed::PreProcessedRepositories,
    mappings: &'a crate::MappingCache,
    src_tr: NodeIdentifier,
    dst_tr: NodeIdentifier,
) -> dashmap::mapref::one::RefMut<
    'a,
    (NodeIdentifier, NodeIdentifier),
    hyper_diff::matchers::Mapping<
        LazyPostOrder<NodeIdentifier, u32>,
        LazyPostOrder<NodeIdentifier, u32>,
        hyper_diff::matchers::mapping_store::VecStore<u32>,
    >,
> {
    use hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder;
    use hyper_diff::matchers::heuristic::gt::{
        lazy2_greedy_bottom_up_matcher::LazyGreedyBottomUpMatcher,
        lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher,
    };
    use hyper_diff::matchers::mapping_store::DefaultMultiMappingStore;
    use hyper_diff::matchers::mapping_store::MappingStore;
    use hyper_diff::matchers::mapping_store::VecStore;
    use hyperast::types::HyperAST;
    mappings.entry((src_tr, dst_tr)).or_insert_with(|| {
        let hyperast = &repositories.processor.main_stores;
        let src = &src_tr;
        let dst = &dst_tr;
        let now = Instant::now();
        let mut _mapper: Mapper<
            _,
            Decompressible<_, LazyPostOrder<_, u32>>,
            Decompressible<_, LazyPostOrder<_, u32>>,
            VecStore<_>,
        > = hyperast.decompress_pair(src, dst).into();
        // TODO factor
        let mapper = Mapper {
            hyperast,
            mapping: Mapping {
                src_arena: _mapper.mapping.src_arena.as_mut(),
                dst_arena: _mapper.mapping.dst_arena.as_mut(),
                mappings: _mapper.mapping.mappings,
            },
        };
        let subtree_prepare_t = now.elapsed().as_secs_f64();
        let now = Instant::now();
        let mapper =
            LazyGreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
        let subtree_matcher_t = now.elapsed().as_secs_f64();
        let subtree_mappings_s = mapper.mappings().len();
        dbg!(&subtree_matcher_t, &subtree_mappings_s);
        let bottomup_prepare_t = 0.;
        let now = Instant::now();
        let mapper = LazyGreedyBottomUpMatcher::<_, _, _, _, VecStore<_>>::match_it(mapper);
        dbg!(&now.elapsed().as_secs_f64());
        let bottomup_matcher_t = now.elapsed().as_secs_f64();
        let bottomup_mappings_s = mapper.mappings().len();
        dbg!(&bottomup_matcher_t, &bottomup_mappings_s);
        // let now = Instant::now();

        // NOTE could also have completed trees
        // let node_store = hyperast.node_store();
        // let mapper = mapper.map(
        //     |src_arena| CompletePostOrder::from(src_arena.complete(node_store)),
        //     |dst_arena| {
        //         let complete = CompletePostOrder::from(dst_arena.complete(node_store));
        //         SimpleBfsMapper::from(node_store, complete)
        //     },
        // );

        // NOTE we do not use edit scripts here
        // let prepare_gen_t = now.elapsed().as_secs_f64();
        // let now = Instant::now();
        // let actions = ScriptGenerator::compute_actions(mapper.hyperast, &mapper.mapping).ok();
        // let gen_t = now.elapsed().as_secs_f64();
        // dbg!(gen_t);
        // let mapper = mapper.map(|x| x, |dst_arena| dst_arena.back);
        // Mapper::<_, LazyPostOrder<_, _>, LazyPostOrder<_, _>, _>::persist(mapper)
        Mapping {
            mappings: mapper.mapping.mappings,
            src_arena: _mapper.mapping.src_arena.decomp,
            dst_arena: _mapper.mapping.dst_arena.decomp,
        }
    })
}

// WARN lazy subtrees are not complete
fn lazy_subtree_mapping<'a, 'b>(
    repositories: &'a multi_preprocessed::PreProcessedRepositories,
    partial_comp_cache: &'a crate::PartialDecompCache,
    src_tr: NodeIdentifier,
    dst_tr: NodeIdentifier,
) -> hyper_diff::matchers::Mapping<
    clashmap::mapref::one::RefMut<'a, NodeIdentifier, LazyPostOrder<NodeIdentifier, u32>>,
    clashmap::mapref::one::RefMut<'a, NodeIdentifier, LazyPostOrder<NodeIdentifier, u32>>,
    mapping_store::MultiVecStore<u32>,
> {
    use hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder;
    use hyper_diff::matchers::Mapping;
    use hyper_diff::matchers::heuristic::gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher;
    use hyper_diff::matchers::mapping_store::DefaultMultiMappingStore;
    use hyper_diff::matchers::mapping_store::MappingStore;
    use hyper_diff::matchers::mapping_store::VecStore;

    let hyperast = &repositories.processor.main_stores;
    let src = &src_tr;
    let dst = &dst_tr;
    let now = Instant::now();
    assert_ne!(src, dst);
    let (mut decompress_src, mut decompress_dst) = {
        use hyperast::types::DecompressedFrom;
        let cached_decomp = |id: &NodeIdentifier| -> Option<
            clashmap::mapref::one::RefMut<NodeIdentifier, LazyPostOrder<NodeIdentifier, u32>>,
        > {
            let decompress = partial_comp_cache
                .try_entry(*id)?
                .or_insert_with(|| LazyPostOrder::<_, u32>::decompress(hyperast, id));
            Some(decompress)
        };
        loop {
            match (cached_decomp(src), cached_decomp(dst)) {
                (Some(decompress_src), Some(decompress_dst)) => {
                    break (decompress_src, decompress_dst);
                }
                (None, None) => {
                    dbg!();
                }
                _ => {
                    dbg!(
                        partial_comp_cache.hash_usize(src),
                        partial_comp_cache.hash_usize(dst)
                    );
                    dbg!(
                        partial_comp_cache.determine_shard(partial_comp_cache.hash_usize(src)),
                        partial_comp_cache.determine_shard(partial_comp_cache.hash_usize(dst))
                    );
                }
            }
            sleep(Duration::from_secs(2));
        }
    };
    hyperast
        .node_store
        .resolve(decompress_src.original(&decompress_src.root()));
    hyperast
        .node_store
        .resolve(decompress_dst.original(&decompress_dst.root()));

    let mappings = VecStore::default();
    let mut mapper = Mapper {
        hyperast,
        mapping: Mapping {
            src_arena: Decompressible {
                hyperast,
                decomp: decompress_src.value_mut(),
            },
            dst_arena: Decompressible {
                hyperast,
                decomp: decompress_dst.value_mut(),
            },
            mappings,
        },
    };
    mapper.mapping.mappings.topit(
        mapper.mapping.src_arena.len(),
        mapper.mapping.dst_arena.len(),
    );
    dbg!();

    let mm = LazyGreedySubtreeMatcher::<_, _, _, VecStore<_>>::compute_multi_mapping::<
        DefaultMultiMappingStore<_>,
    >(&mut mapper);
    dbg!();

    hyper_diff::matchers::Mapping {
        src_arena: decompress_src,
        dst_arena: decompress_dst,
        mappings: mm,
    }
}

pub fn child_by_type<'store, HAST: HyperAST<IdN = NodeIdentifier>>(
    stores: &'store HAST,
    d: NodeIdentifier,
    t: &<HAST::TS as types::TypeStore>::Ty,
) -> Option<(NodeIdentifier, usize)> {
    let n = stores.node_store().resolve(&d);
    let s = n
        .children()
        .unwrap()
        .iter_children()
        .enumerate()
        .find(|(_, x)| stores.resolve_type(x).eq(t))
        .map(|(i, x)| (x, i));
    s
}
