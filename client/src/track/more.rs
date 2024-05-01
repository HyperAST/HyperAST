use super::*;

// fn diff<'a>(
//     repositories: &'a multi_preprocessed::PreProcessedRepositories,
//     mappings: &'a mut crate::MappingCache,
//     src_tr: NodeIdentifier,
//     dst_tr: NodeIdentifier,
// ) -> &'a hyper_diff::matchers::Mapping<
//     hyper_diff::decompressed_tree_store::CompletePostOrder<
//         hyper_ast::store::nodes::legion::HashedNodeRef<'a>,
//         u32,
//     >,
//     hyper_diff::decompressed_tree_store::CompletePostOrder<
//         hyper_ast::store::nodes::legion::HashedNodeRef<'a>,
//         u32,
//     >,
//     hyper_diff::matchers::mapping_store::VecStore<u32>,
// > {
//     use hyper_diff::decompressed_tree_store::CompletePostOrder;
//     let mapped = mappings.entry((src_tr, dst_tr)).or_insert_with(|| {
//         hyper_diff::algorithms::gumtree_lazy::diff(
//             &repositories.processor.main_stores,
//             &src_tr,
//             &dst_tr,
//         )
//         .mapper
//         .persist()
//     });
//     unsafe { Mapper::<_,CompletePostOrder<_,_>,CompletePostOrder<_,_>,_>::unpersist(&repositories.processor.main_stores, &*mapped) }
// }

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
        hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder<
            hyper_ast::store::nodes::legion::HashedNodeRef<'a, NodeIdentifier>,
            u32,
        >,
        hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder<
            hyper_ast::store::nodes::legion::HashedNodeRef<'a, NodeIdentifier>,
            u32,
        >,
        hyper_diff::matchers::mapping_store::VecStore<u32>,
    >,
> {
    use hyper_ast::types::HyperAST;
    use hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder;
    use hyper_diff::matchers::heuristic::gt::{
        lazy2_greedy_bottom_up_matcher::GreedyBottomUpMatcher,
        lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher,
    };
    use hyper_diff::matchers::mapping_store::DefaultMultiMappingStore;
    use hyper_diff::matchers::mapping_store::MappingStore;
    use hyper_diff::matchers::mapping_store::VecStore;
    let mapped = mappings.entry((src_tr, dst_tr)).or_insert_with(|| {
        let hyperast = &repositories.processor.main_stores;
        let src = &src_tr;
        let dst = &dst_tr;
        let now = Instant::now();
        let mapper: Mapper<_, LazyPostOrder<_, u32>, LazyPostOrder<_, u32>, VecStore<_>> =
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
        Mapper::<_, LazyPostOrder<_, _>, LazyPostOrder<_, _>, _>::persist(mapper)
    });
    pub unsafe fn unpersist<'a>(
        _hyperast: &'a SimpleStores<TStore>,
        p: dashmap::mapref::one::RefMut<
            'a,
            (NodeIdentifier, NodeIdentifier),
            hyper_diff::matchers::Mapping<
                LazyPostOrder<
                    hyper_diff::decompressed_tree_store::PersistedNode<NodeIdentifier>,
                    u32,
                >,
                LazyPostOrder<
                    hyper_diff::decompressed_tree_store::PersistedNode<NodeIdentifier>,
                    u32,
                >,
                VecStore<u32>,
            >,
        >,
    ) -> dashmap::mapref::one::RefMut<
        'a,
        (NodeIdentifier, NodeIdentifier),
        hyper_diff::matchers::Mapping<
            LazyPostOrder<HashedNodeRef<'a, NodeIdentifier>, u32>,
            LazyPostOrder<HashedNodeRef<'a, NodeIdentifier>, u32>,
            VecStore<u32>,
        >,
    > {
        unsafe { std::mem::transmute(p) }
    }
    unsafe { unpersist(&repositories.processor.main_stores, mapped) }
}

struct RRR<'a>(
    dashmap::mapref::one::Ref<
        'a,
        (NodeIdentifier, NodeIdentifier),
        (
            crate::MappingStage,
            hyper_diff::matchers::mapping_store::VecStore<u32>,
        ),
    >,
);

// WARN lazy subtrees are not complete
fn lazy_subtree_mapping<'a, 'b>(
    repositories: &'a multi_preprocessed::PreProcessedRepositories,
    partial_comp_cache: &'a crate::PartialDecompCache,
    src_tr: NodeIdentifier,
    dst_tr: NodeIdentifier,
) -> hyper_diff::matchers::Mapping<
    dashmap::mapref::one::RefMut<
        'a,
        NodeIdentifier,
        hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder<
            hyper_ast::store::nodes::legion::HashedNodeRef<'a, NodeIdentifier>,
            u32,
        >,
    >,
    dashmap::mapref::one::RefMut<
        'a,
        NodeIdentifier,
        hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder<
            hyper_ast::store::nodes::legion::HashedNodeRef<'a, NodeIdentifier>,
            u32,
        >,
    >,
    mapping_store::MultiVecStore<u32>,
> {
    use hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder;
    use hyper_diff::matchers::heuristic::gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher;
    use hyper_diff::matchers::mapping_store::DefaultMultiMappingStore;
    use hyper_diff::matchers::mapping_store::MappingStore;
    use hyper_diff::matchers::mapping_store::VecStore;
    use hyper_diff::matchers::Mapping;

    let hyperast = &repositories.processor.main_stores;
    let src = &src_tr;
    let dst = &dst_tr;
    let now = Instant::now();
    assert_ne!(src, dst);
    let (mut decompress_src, mut decompress_dst) = {
        use hyper_ast::types::DecompressedSubtree;
        let mut cached_decomp = |id: &NodeIdentifier| -> Option<
            dashmap::mapref::one::RefMut<NodeIdentifier, LazyPostOrder<HashedNodeRef<'a>, u32>>,
        > {
            let decompress = partial_comp_cache
                .try_entry(*id)?
                .or_insert_with(|| unsafe {
                    std::mem::transmute(LazyPostOrder::<_, u32>::decompress(
                        hyperast.node_store(),
                        id,
                    ))
                });
            Some(unsafe { std::mem::transmute(decompress) })
        };
        loop {
            match (cached_decomp(src), cached_decomp(dst)) {
                (Some(decompress_src), Some(decompress_dst)) => {
                    break (decompress_src, decompress_dst)
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
            src_arena: decompress_src.value_mut(),
            dst_arena: decompress_dst.value_mut(),
            mappings,
        },
    };
    mapper.mapping.mappings.topit(
        mapper.mapping.src_arena.len(),
        mapper.mapping.dst_arena.len(),
    );
    dbg!();
    let mm = LazyGreedySubtreeMatcher::<
        'a,
        SimpleStores<TStore>,
        &mut LazyPostOrder<HashedNodeRef<'a>, u32>,
        &mut LazyPostOrder<HashedNodeRef<'a>, u32>,
        VecStore<_>,
    >::compute_multi_mapping::<DefaultMultiMappingStore<_>>(&mut mapper);
    dbg!();

    hyper_diff::matchers::Mapping {
        src_arena: decompress_src,
        dst_arena: decompress_dst,
        mappings: mm,
    }
}

pub fn child_by_type<'store, HAST: HyperAST<'store, IdN = NodeIdentifier>>(
    stores: &'store HAST,
    d: NodeIdentifier,
    t: &<HAST::TS as types::TypeStore<HAST::T>>::Ty,
) -> Option<(NodeIdentifier, usize)> {
    let n = stores.node_store().resolve(&d);
    let s = n
        .children()
        .unwrap()
        .iter_children()
        .enumerate()
        .find(|(_, x)| stores.resolve_type(*x).eq(t))
        .map(|(i, x)| (*x, i));
    s
}
