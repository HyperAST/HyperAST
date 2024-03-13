
fn aux_aux(
    repo_handle: &impl ConfiguredRepoTrait,
    src_tr: NodeIdentifier,
    dst_tr: NodeIdentifier,
    path_to_target: Vec<u16>,
    no_spaces_path_to_target: Vec<u16>,
    flags: &Flags,
    start: usize,
    end: usize,
    partial_decomps: &PartialDecompCache,
    mappings_alone: &MappingAloneCache,
    repositories: std::sync::RwLockReadGuard<multi_preprocessed::PreProcessedRepositories>,
    dst_oid: hyper_ast_cvs_git::git::Oid,
    target_node: NodeIdentifier,
) -> MappingResult {
    let with_spaces_stores = &repositories.processor.main_stores;
    let stores = &no_space::as_nospaces(with_spaces_stores);
    let node_store = &stores.node_store;
    // NOTE: persists mappings, could also easily persist diffs,
    // but some compression on mappins could help
    // such as, not storing the decompression arenas
    // or encoding mappings more efficiently considering that most slices could simply by represented as ranges (ie. mapped identical subtrees)
    // let mapper = lazy_mapping(repos, &mut state.mappings, src_tr, dst_tr);

    dbg!(src_tr, dst_tr);
    if src_tr == dst_tr {
        let src_size = stores.node_store.resolve(src_tr).size();
        let dst_size = stores.node_store.resolve(dst_tr).size();
        let nodes = src_size + dst_size;
        let (pos, path_ids) = compute_position_and_nodes(
            dst_tr,
            &mut path_to_target.iter().copied(),
            with_spaces_stores,
        );
        dbg!();
        let range = pos.range();
        let matches = vec![PieceOfCode {
            user: repo_handle.spec().user.clone(),
            name: repo_handle.spec().name.clone(),
            commit: dst_oid.to_string(),
            file: pos.file().to_str().unwrap().to_string(),
            start: range.start,
            end: range.end,
            path: path_to_target.iter().map(|x| *x as usize).collect(),
            path_ids: path_ids.clone(),
        }];
        let src = LocalPieceOfCode {
            file: pos.file().to_string_lossy().to_string(),
            start,
            end,
            path: path_to_target.iter().map(|x| *x as usize).collect(),
            path_ids,
        };
        if flags.some() {
            return MappingResult::Skipped {
                nodes,
                src,
                next: matches,
            };
        } else {
            return MappingResult::Direct { src, matches };
        }
    }
    let pair = get_pair_simp(partial_decomps, stores, &src_tr, &dst_tr);

    if flags.some() {
        dbg!();

        let mapped = {
            let mappings_cache = mappings_alone;
            let hyperast = stores;
            let src = &src_tr;
            let dst = &dst_tr;
            let (src_arena, dst_arena) = (pair.0.get_mut(), pair.1.get_mut());
            matching::top_down(hyperast, src_arena, dst_arena)
        };
        let (mapper_src_arena, mapper_dst_arena) = (pair.0.get_mut(), pair.1.get_mut());
        let mapper_mappings = &mapped;
        let mut curr = mapper_src_arena.root();
        let mut path = &no_spaces_path_to_target[..];
        let flags: EnumSet<_> = flags.into();
        loop {
            dbg!(path);
            let dsts = mapper_mappings.get_dsts(&curr);
            let curr_flags = FlagsE::Upd | FlagsE::Child | FlagsE::SimChild; //  | FlagsE::ExactChild
            let parent_flags = curr_flags | FlagsE::Parent | FlagsE::SimParent; //  | FlagsE::ExactParent
            if dsts.is_empty() {
                // continue through path_to_target
                dbg!(curr);
            } else if path.len() == 0 {
                // need to check curr node flags
                if flags.is_subset(curr_flags) {
                    // only trigger on curr and children changed

                    let src_size = stores.node_store.resolve(src_tr).size();
                    let dst_size = stores.node_store.resolve(dst_tr).size();
                    let nodes = src_size + dst_size;
                    let nodes = 500_000;

                    return MappingResult::Skipped {
                        nodes,
                        src: {
                            let (pos, path_ids) = compute_position_and_nodes(
                                src_tr,
                                &mut path_to_target.iter().copied(),
                                with_spaces_stores,
                            );

                            LocalPieceOfCode {
                                file: pos.file().to_string_lossy().to_string(),
                                start,
                                end,
                                path: path_to_target.iter().map(|x| *x as usize).collect(),
                                path_ids,
                            }
                        },
                        next: dsts
                            .iter()
                            .map(|x| {
                                let path_dst = mapper_dst_arena.path(&mapper_dst_arena.root(), x);
                                let (path_dst,) = path_with_spaces(
                                    dst_tr,
                                    &mut path_dst.iter().copied(),
                                    &repositories.processor.main_stores,
                                );
                                let (pos, path_ids) = compute_position_and_nodes(
                                    dst_tr,
                                    &mut path_dst.iter().copied(),
                                    with_spaces_stores,
                                );
                                let range = pos.range();
                                PieceOfCode {
                                    user: repo_handle.spec().user.clone(),
                                    name: repo_handle.spec().name.clone(),
                                    commit: dst_oid.to_string(),
                                    file: pos.file().to_str().unwrap().to_string(),
                                    start: range.start,
                                    end: range.end,
                                    path: path_dst.iter().map(|x| *x as usize).collect(),
                                    path_ids,
                                }
                            })
                            .collect(),
                    };
                }
                // also the type of src and dsts
                // also check it file path changed
                // can we test if parent changed ? at least we can ckeck some attributes
            } else if path.len() == 1 {
                // need to check parent node flags
                if flags.is_subset(parent_flags) {
                    // only trigger on parent, curr and children changed
                    let src_size = stores.node_store.resolve(src_tr).size();
                    let dst_size = stores.node_store.resolve(dst_tr).size();
                    let nodes = src_size + dst_size;
                    let nodes = 500_000;
                    return MappingResult::Skipped {
                        nodes,
                        src: {
                            let (pos, path_ids) = compute_position_and_nodes(
                                src_tr,
                                &mut path_to_target.iter().copied(),
                                with_spaces_stores,
                            );

                            LocalPieceOfCode {
                                file: pos.file().to_string_lossy().to_string(),
                                start,
                                end,
                                path: path_to_target.iter().map(|x| *x as usize).collect(),
                                path_ids,
                            }
                        },
                        next: dsts
                            .iter()
                            .map(|x| {
                                let mut path_dst =
                                    mapper_dst_arena.path(&mapper_dst_arena.root(), x);
                                path_dst.extend(path); // WARN with similarity it might not be possible to simply concat path...
                                let (path_dst,) = path_with_spaces(
                                    dst_tr,
                                    &mut path_dst.iter().copied(),
                                    &repositories.processor.main_stores,
                                );
                                let (pos, path_ids) = compute_position_and_nodes(
                                    dst_tr,
                                    &mut path_dst.iter().copied(),
                                    with_spaces_stores,
                                );
                                let range = pos.range();
                                PieceOfCode {
                                    user: repo_handle.spec().user.clone(),
                                    name: repo_handle.spec().name.clone(),
                                    commit: dst_oid.to_string(),
                                    file: pos.file().to_str().unwrap().to_string(),
                                    start: range.start,
                                    end: range.end,
                                    path: path_dst.iter().map(|x| *x as usize).collect(),
                                    path_ids,
                                }
                            })
                            .collect(),
                    };
                }
                // also the type of src and dsts
                // also check if file path changed
                // can we test if parent changed ? at least we can ckeck some attributes
            } else {
                // need to check flags, the type of src and dsts
                if flags.is_subset(parent_flags) {
                    // only trigger on parent, curr and children changed
                    let src_size = stores.node_store.resolve(src_tr).size();
                    let dst_size = stores.node_store.resolve(dst_tr).size();
                    let nodes = src_size + dst_size;
                    let nodes = 500_000;
                    return MappingResult::Skipped {
                        nodes,
                        src: {
                            let (pos, path_ids) = compute_position_and_nodes(
                                src_tr,
                                &mut path_to_target.iter().copied(),
                                with_spaces_stores,
                            );

                            LocalPieceOfCode {
                                file: pos.file().to_string_lossy().to_string(),
                                start,
                                end,
                                path: path_to_target.iter().map(|x| *x as usize).collect(),
                                path_ids,
                            }
                        },
                        next: dsts
                            .iter()
                            .map(|x| {
                                let mut path_dst =
                                    mapper_dst_arena.path(&mapper_dst_arena.root(), x);
                                path_dst.extend(path); // WARN with similarity it might not be possible to simply concat path...
                                let (path_dst,) = path_with_spaces(
                                    dst_tr,
                                    &mut path_dst.iter().copied(),
                                    &repositories.processor.main_stores,
                                );
                                let (pos, path_ids) = compute_position_and_nodes(
                                    dst_tr,
                                    &mut path_dst.iter().copied(),
                                    with_spaces_stores,
                                );
                                let range = pos.range();
                                PieceOfCode {
                                    user: repo_handle.spec().user.clone(),
                                    name: repo_handle.spec().name.clone(),
                                    commit: dst_oid.to_string(),
                                    file: pos.file().to_str().unwrap().to_string(),
                                    start: range.start,
                                    end: range.end,
                                    path: path_dst.iter().map(|x| *x as usize).collect(),
                                    path_ids,
                                }
                            })
                            .collect(),
                    };
                }
                // also check if file path changed
                // can we test if parent changed ? at least we can ckeck some attributes
            }

            let Some(i) = path.get(0) else {
                break;
            };
            path = &path[1..];
            let cs = mapper_src_arena.decompress_children(node_store, &curr);
            if cs.is_empty() {
                break;
            }
            curr = cs[*i as usize];
        }
    }

    let mapped = {
        let mappings_cache = mappings_alone;
        use hyper_diff::matchers::mapping_store::MappingStore;
        use hyper_diff::matchers::mapping_store::VecStore;
        let hyperast = stores;
        use hyper_diff::matchers::Mapping;

        dbg!();
        match mappings_cache.entry((src_tr, dst_tr)) {
            dashmap::mapref::entry::Entry::Occupied(entry) => entry.into_ref().downgrade(),
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                let mappings = VecStore::default();
                let (src_arena, dst_arena) = (pair.0.get_mut(), pair.1.get_mut());
                dbg!(src_arena.len());
                dbg!(dst_arena.len());
                let src_size = stores.node_store.resolve(src_tr).size();
                let dst_size = stores.node_store.resolve(dst_tr).size();
                dbg!(src_size);
                dbg!(dst_size);
                let mut mapper = Mapper {
                    hyperast,
                    mapping: Mapping {
                        src_arena,
                        dst_arena,
                        mappings,
                    },
                };
                dbg!();
                dbg!(mapper.mapping.src_arena.len());
                dbg!(mapper.mapping.dst_arena.len());
                mapper.mapping.mappings.topit(
                    mapper.mapping.src_arena.len(),
                    mapper.mapping.dst_arena.len(),
                );
                dbg!();

                let vec_store = matching::full2(hyperast, mapper);

                dbg!();
                entry
                    .insert((crate::MappingStage::Bottomup, vec_store))
                    .downgrade()
            }
        }
    };
    let (mapper_src_arena, mapper_dst_arena) = (pair.0.get_mut(), pair.1.get_mut());
    let mapper_mappings = &mapped.1;
    let root = mapper_src_arena.root();
    let mapping_target =
        mapper_src_arena.child_decompressed(node_store, &root, &no_spaces_path_to_target);

    let mut matches = vec![];
    if let Some(mapped) = mapper_mappings.get_dst(&mapping_target) {
        let mapped = mapper_dst_arena.decompress_to(node_store, &mapped);
        let path = mapper_dst_arena.path(&mapper_dst_arena.root(), &mapped);
        let mut path_ids = vec![mapper_dst_arena.original(&mapped)];
        mapper_dst_arena
            .parents(mapped)
            .map(|i| mapper_dst_arena.original(&i))
            .collect_into(&mut path_ids);
        path_ids.pop();
        assert_eq!(path.len(), path_ids.len());
        let (path,) = path_with_spaces(
            dst_tr,
            &mut path.iter().copied(),
            &repositories.processor.main_stores,
        );
        let (pos, mapped_node) =
            compute_position(dst_tr, &mut path.iter().copied(), with_spaces_stores);
        dbg!(&pos);
        dbg!(&mapped_node);
        let mut flagged = false;
        let mut triggered = false;
        if flags.exact_child {
            flagged = true;
            dbg!();
            triggered |= target_node != mapped_node;
        }
        if flags.child || flags.sim_child {
            flagged = true;
            dbg!();

            let target_node = stores.node_store.resolve(target_node);
            let mapped_node = stores.node_store.resolve(mapped_node);
            if flags.sim_child {
                triggered |= target_node.hash(&types::HashKind::structural())
                    != mapped_node.hash(&types::HashKind::structural());
            } else {
                triggered |= target_node.hash(&types::HashKind::label())
                    != mapped_node.hash(&types::HashKind::label());
            }
        }
        if flags.upd {
            flagged = true;
            dbg!();
            // TODO need role name
            // let target_ident = child_by_type(stores, target_node, &Type::Identifier);
            // let mapped_ident = child_by_type(stores, mapped_node, &Type::Identifier);
            // if let (Some(target_ident), Some(mapped_ident)) = (target_ident, mapped_ident) {
            //     let target_node = stores.node_store.resolve(target_ident.0);
            //     let target_ident = target_node.try_get_label();
            //     let mapped_node = stores.node_store.resolve(mapped_ident.0);
            //     let mapped_ident = mapped_node.try_get_label();
            //     triggered |= target_ident != mapped_ident;
            // }
        }
        if flags.parent {
            flagged = true;
            dbg!();

            let target_parent = mapper_src_arena.parent(&mapping_target);
            let target_parent = target_parent.map(|x| mapper_src_arena.original(&x));
            let mapped_parent = mapper_dst_arena.parent(&mapped);
            let mapped_parent = mapped_parent.map(|x| mapper_dst_arena.original(&x));
            triggered |= target_parent != mapped_parent;
        }
        // if flags.meth {
        //     flagged = true;
        //     dbg!();
        // }
        // if flags.typ {
        //     flagged = true;
        //     dbg!();
        // }
        // if flags.top {
        //     flagged = true;
        //     dbg!();
        // }
        // if flags.file {
        //     flagged = true;
        //     dbg!();
        // }
        // if flags.pack {
        //     flagged = true;
        //     dbg!();
        // }
        // TODO add flags for artefacts (tests, prod code, build, lang, misc)
        // TODO add flags for similarity comps
        let range = pos.range();
        matches.push(PieceOfCode {
            user: repo_handle.spec().user.clone(),
            name: repo_handle.spec().name.clone(),
            commit: dst_oid.to_string(),
            file: pos.file().to_str().unwrap().to_string(),
            start: range.start,
            end: range.end,
            path: path.iter().map(|x| *x as usize).collect(),
            path_ids: path_ids.clone(),
        });
        if flagged && !triggered {
            use hyper_ast::types::WithStats;
            let src_size = stores.node_store.resolve(src_tr).size();
            let dst_size = stores.node_store.resolve(dst_tr).size();
            let nodes = src_size + dst_size;
            return MappingResult::Skipped {
                nodes,
                src: {
                    let (pos, path_ids) = compute_position_and_nodes(
                        src_tr,
                        &mut path_to_target.iter().copied(),
                        with_spaces_stores,
                    );

                    LocalPieceOfCode {
                        file: pos.file().to_string_lossy().to_string(),
                        start,
                        end,
                        path: path_to_target.iter().map(|x| *x as usize).collect(),
                        path_ids,
                    }
                },
                next: matches,
            };
        }
        let path = path_to_target.iter().map(|x| *x as usize).collect();
        let (target_pos, target_path_ids) = compute_position_and_nodes(
            src_tr,
            &mut path_to_target.iter().copied(),
            with_spaces_stores,
        );
        return MappingResult::Direct {
            src: LocalPieceOfCode {
                file: target_pos.file().to_string_lossy().to_string(),
                start,
                end,
                path,
                path_ids: target_path_ids,
            },
            matches,
        };
    }

    for parent_target in mapper_src_arena.parents(mapping_target) {
        if let Some(mapped_parent) = mapper_mappings.get_dst(&parent_target) {
            let mapped_parent = mapper_dst_arena.decompress_to(node_store, &mapped_parent);
            let path = mapper_dst_arena.path(&mapper_dst_arena.root(), &mapped_parent);
            let mut path_ids = vec![mapper_dst_arena.original(&mapped_parent)];
            mapper_dst_arena
                .parents(mapped_parent)
                .map(|i| mapper_dst_arena.original(&i))
                .collect_into(&mut path_ids);
            path_ids.pop();
            assert_eq!(path.len(), path_ids.len());
            let (path,) = path_with_spaces(
                dst_tr,
                &mut path.iter().copied(),
                &repositories.processor.main_stores,
            );
            let (pos, mapped_node) =
                compute_position(dst_tr, &mut path.iter().copied(), with_spaces_stores);
            dbg!(&pos);
            dbg!(&mapped_node);
            let range = pos.range();
            let fallback = PieceOfCode {
                user: repo_handle.spec().user.clone(),
                name: repo_handle.spec().name.clone(),
                commit: dst_oid.to_string(),
                file: pos.file().to_str().unwrap().to_string(),
                start: range.start,
                end: range.end,
                path: path.iter().map(|x| *x as usize).collect(),
                path_ids: path_ids.clone(),
            };

            let src = {
                let path = path_to_target.iter().map(|x| *x as usize).collect();
                let (target_pos, target_path_ids) = compute_position_and_nodes(
                    src_tr,
                    &mut path_to_target.iter().copied(),
                    with_spaces_stores,
                );
                LocalPieceOfCode {
                    file: target_pos.file().to_string_lossy().to_string(),
                    start,
                    end,
                    path,
                    path_ids: target_path_ids,
                }
            };
            return MappingResult::Missing { src, fallback };
        };
    }
    let path = path_to_target.iter().map(|x| *x as usize).collect();
    let (target_pos, target_path_ids) = compute_position_and_nodes(
        src_tr,
        &mut path_to_target.iter().copied(),
        with_spaces_stores,
    );
    // TODO what should be done if there is no match ?
    MappingResult::Direct {
        src: LocalPieceOfCode {
            file: target_pos.file().to_string_lossy().to_string(),
            start,
            end,
            path,
            path_ids: target_path_ids,
        },
        matches,
    }
}

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

mod my_dash {
    use std::{
        cell::UnsafeCell,
        collections::hash_map::RandomState,
        fmt::Debug,
        hash::{BuildHasher, Hash},
    };

    use dashmap::{DashMap, RwLockWriteGuard, SharedValue};
    use hashbrown::HashMap;

    // pub fn entries<'a, K: 'a + Eq + Hash, V: 'a, S: BuildHasher + Clone>(
    //     map: DashMap<K, V, S>,
    //     key1: K,
    //     key2: K,
    // ) -> Entry<'a, K, V, S> {
    //     let hash = map.hash_usize(&key1);

    //     let idx = map.determine_shard(hash);

    //     let shard: RwLockWriteGuard<HashMap<K, SharedValue<V>, S>> = unsafe {
    //         debug_assert!(idx < map.shards().len());

    //         map.shards().get_unchecked(idx).write()
    //     };

    //     #[repr(transparent)]
    //     struct MySharedValue<T> {
    //         value: UnsafeCell<T>,
    //     }

    //     impl<T> MySharedValue<T> {
    //         /// Get a mutable raw pointer to the underlying value
    //         fn as_ptr(&self) -> *mut T {
    //             self.value.get()
    //         }
    //     }
    //     // SAFETY: Sharded and UnsafeCell are transparent wrappers of V
    //     let shard: RwLockWriteGuard<HashMap<K, V, S>> = unsafe { std::mem::transmute(shard) };
    //     if let Some((kptr, vptr)) = shard.get_key_value(&key1) {
    //         unsafe {
    //             let kptr: *const K = kptr;
    //             // SAFETY: same memory layout because transparent and same fields
    //             let vptr: &MySharedValue<V> = std::mem::transmute(&vptr);
    //             let vptr: *mut V = vptr.as_ptr();
    //             Entry::Occupied(OccupiedEntry::new(shard, key1, (kptr, vptr)))
    //         }
    //     } else {
    //         unsafe {
    //             // SAFETY: same memory layout because transparent and same fields
    //             let shard: RwLockWriteGuard<HashMap<K, V, S>> = std::mem::transmute(shard);
    //             Entry::Vacant(VacantEntry::new(shard, key1))
    //         }
    //     }
    pub fn entries<'a, K: 'a + Eq + Hash, V: 'a, S: BuildHasher + Clone>(
        map: &'a DashMap<K, V, S>,
        key1: K,
        key2: K,
    ) -> Entry<'a, K, V, S> {
        assert!(key1 != key2, "keys should be different");
        let hash1 = map.hash_usize(&key1);
        let idx1 = map.determine_shard(hash1);
        let hash2 = map.hash_usize(&key2);
        let idx2 = map.determine_shard(hash2);

        if idx1 == idx2 {
            let shard = unsafe {
                debug_assert!(idx1 < map.shards().len());
                debug_assert!(idx2 < map.shards().len());
                map.shards().get_unchecked(idx1).write()
            };
            // SAFETY: Sharded and UnsafeCell are transparent wrappers of V
            let shard: RwLockWriteGuard<HashMap<K, V, S>> = unsafe { std::mem::transmute(shard) };
            let elem1 = shard
                .get_key_value(&key1)
                .map(|(kptr, vptr)| unsafe { as_ptr(kptr, vptr) });
            let elem2 = shard
                .get_key_value(&key2)
                .map(|(kptr, vptr)| unsafe { as_ptr(kptr, vptr) });
            Entry {
                shard1: shard,
                shard2: None,
                key1,
                key2,
                elem1,
                elem2,
            }
        } else {
            let (shard1, shard2) = unsafe {
                debug_assert!(idx1 < map.shards().len());
                debug_assert!(idx2 < map.shards().len());
                (
                    map.shards().get_unchecked(idx1).write(),
                    map.shards().get_unchecked(idx2).write(),
                )
            };
            let shard1: RwLockWriteGuard<HashMap<K, V, S>> = unsafe { std::mem::transmute(shard1) };
            let shard2: RwLockWriteGuard<HashMap<K, V, S>> = unsafe { std::mem::transmute(shard2) };
            let elem1 = shard1
                .get_key_value(&key1)
                .map(|(kptr, vptr)| unsafe { as_ptr(kptr, vptr) });
            let elem2 = shard2
                .get_key_value(&key2)
                .map(|(kptr, vptr)| unsafe { as_ptr(kptr, vptr) });
            Entry {
                shard1: shard1,
                shard2: Some(shard2),
                key1,
                key2,
                elem1,
                elem2,
            }
        }
    }

    unsafe fn as_ptr<'a, K: 'a + Eq + Hash, V: 'a>(kptr1: &K, vptr1: &V) -> (*const K, *mut V) {
        let kptr1: *const K = kptr1;
        // SAFETY: same memory layout because transparent and same fields
        let vptr1: &MySharedValue<V> = std::mem::transmute(&vptr1);
        let vptr1: *mut V = vptr1.as_ptr();
        (kptr1, vptr1)
    }

    pub(super) unsafe fn shard_as_ptr<'a, V: 'a>(vptr1: &SharedValue<V>) -> *mut V {
        // SAFETY: same memory layout because transparent and same fields
        let vptr1: &MySharedValue<V> = std::mem::transmute(&vptr1);
        let vptr1: *mut V = vptr1.as_ptr();
        vptr1
    }

    pub(super) unsafe fn shard_as_ptr2<'a, V: 'a>(vptr1: &V) -> *mut V {
        // SAFETY: same memory layout because transparent and same fields
        let vptr1: &MySharedValue<V> = std::mem::transmute(&vptr1);
        let vptr1: *mut V = vptr1.as_ptr();
        vptr1
    }

    #[repr(transparent)]
    struct MySharedValue<T> {
        value: UnsafeCell<T>,
    }

    impl<T> MySharedValue<T> {
        /// Get a mutable raw pointer to the underlying value
        fn as_ptr(&self) -> *mut T {
            self.value.get()
        }
    }
    pub struct Entry<'a, K, V, S = RandomState> {
        shard1: RwLockWriteGuard<'a, HashMap<K, V, S>>,
        shard2: Option<RwLockWriteGuard<'a, HashMap<K, V, S>>>,
        elem1: Option<(*const K, *mut V)>,
        elem2: Option<(*const K, *mut V)>,
        key1: K,
        key2: K,
    }
    unsafe impl<'a, K: Eq + Hash + Sync, V: Sync, S: BuildHasher> Send for Entry<'a, K, V, S> {}
    unsafe impl<'a, K: Eq + Hash + Sync, V: Sync, S: BuildHasher> Sync for Entry<'a, K, V, S> {}

    impl<'a, K: Clone + Eq + Hash + Debug, V: Debug, S: BuildHasher> Entry<'a, K, V, S> {
        pub fn or_insert_with(
            self,
            value: impl FnOnce((Option<()>, Option<()>)) -> (Option<V>, Option<V>),
        ) -> RefMut<'a, K, V, S> {
            match self {
                Entry {
                    shard1,
                    shard2,
                    elem1: Some((k1, v1)),
                    elem2: Some((k2, v2)),
                    ..
                } => {
                    dbg!(v1);
                    dbg!(v2);
                    RefMut {
                        guard1: shard1,
                        guard2: shard2,
                        k1,
                        k2,
                        v1,
                        v2,
                    }
                }
                Entry {
                    mut shard1,
                    shard2: None,
                    elem1,
                    elem2,
                    key1,
                    key2,
                } => {
                    let (r1, r2) = value((elem1.as_ref().map(|_| ()), elem2.as_ref().map(|_| ())));
                    let k1 = key1.clone();
                    let k2 = key2.clone();
                    if elem1.is_none() {
                        let value = r1.expect("some value");
                        let key = key1;
                        let shard = &mut shard1;
                        insert2_p1(key, shard, value)
                    }
                    if elem2.is_none() {
                        let value = r2.expect("some value");
                        let key = key2;
                        let shard = &mut shard1;
                        insert2_p1(key, shard, value)
                    }
                    let (k1, v1) = elem1.unwrap_or_else(|| {
                        let shard = &mut shard1;
                        insert2_p2(&k1, shard)
                    });
                    let (k2, v2) = elem2.unwrap_or_else(|| {
                        let shard = &mut shard1;
                        insert2_p2(&k2, shard)
                    });
                    dbg!(v1);
                    dbg!(v2);
                    RefMut {
                        guard1: shard1,
                        guard2: None,
                        k1,
                        k2,
                        v1,
                        v2,
                    }
                }
                Entry {
                    mut shard1,
                    shard2: Some(mut shard2),
                    elem1,
                    elem2,
                    key1,
                    key2,
                } => {
                    let (r1, r2) = value((elem1.as_ref().map(|_| ()), elem2.as_ref().map(|_| ())));
                    // let (k1, v1) = elem1.unwrap_or_else(|| {
                    //     let value = r1.expect("some value");
                    //     let key = key1;
                    //     let shard = &mut shard1;
                    //     println!("{:p}", shard);
                    //     println!("{:p}", &key);
                    //     println!("{}", shard.hasher().hash_one(&key));
                    //     insert2(key, shard, value)
                    // });
                    // let (k2, v2) = elem2.unwrap_or_else(|| {
                    //     let value = r2.expect("some value");
                    //     let key = key2;
                    //     let shard = &mut shard2;
                    //     insert2(key, shard, value)
                    // });
                    let k1 = key1.clone();
                    let k2 = key2.clone();
                    dbg!(&k1);
                    dbg!(&k2);
                    println!("{:p}", &k1);
                    println!("{:p}", &k2);
                    println!("{:p}", &r1);
                    println!("{:p}", &r2);
                    if elem1.is_none() {
                        let value = r1.expect("some value");
                        dbg!(&value);
                        println!("{:p}", &value);
                        let key = key1;
                        let shard = &mut shard1;
                        insert2_p1_shard(key, shard, value)
                    }
                    if elem2.is_none() {
                        let value = r2.expect("some value");
                        dbg!(&value);
                        println!("{:p}", &value);
                        let key = key2;
                        let shard = &mut shard2;
                        insert2_p1(key, shard, value)
                    }
                    let (k1, v1) = elem1.unwrap_or_else(|| {
                        let shard = &mut shard1;
                        dbg!(shard.hasher().hash_one(&k1));
                        let shard: &mut RwLockWriteGuard<HashMap<K, SharedValue<V>>> =
                            unsafe { std::mem::transmute(shard) };
                        insert2_p2_shard(&k1, shard)
                    });
                    let (k2, v2) = elem2.unwrap_or_else(|| {
                        let shard = &mut shard2;
                        insert2_p2(&k2, shard)
                    });
                    println!("{:p}", &shard1);
                    dbg!(shard1.len());
                    println!("{:p}", &shard2);
                    dbg!(shard2.len());
                    dbg!(v1);
                    dbg!(v2);
                    RefMut {
                        guard1: shard1,
                        guard2: Some(shard2),
                        k1,
                        k2,
                        v1,
                        v2,
                    }
                }
            }
        }
    }

    fn insert2<'a, K: Eq + Hash, V, S: BuildHasher>(
        key: K,
        shard: &mut RwLockWriteGuard<'a, HashMap<K, V, S>>,
        value: V,
    ) -> (*const K, *mut V) {
        let c = unsafe { std::ptr::read(&key) };
        shard.insert(key, value);
        // let shard: &mut RwLockWriteGuard<HashMap<K, SharedValue<V>>> =
        //     unsafe { std::mem::transmute(shard) };
        {
            // let shard: &'a mut RwLockWriteGuard<'a, HashMap<K, SharedValue<V>>> = shard;
            unsafe {
                use std::mem;
                dbg!();
                let (k, v) = shard.get_key_value(&c).unwrap();
                dbg!();
                let k = change_lifetime_const(k);
                dbg!();
                let v = &mut *shard_as_ptr2(v);
                dbg!();
                mem::forget(c);
                dbg!();
                (k, v)
            }
        }
    }

    fn insert2_p1<'a, K: Eq + Hash, V, S: BuildHasher>(
        key: K,
        shard: &mut RwLockWriteGuard<'a, HashMap<K, V, S>>,
        value: V,
    ) {
        println!("{:p}", &key);
        println!("{}", shard.hasher().hash_one(&key));
        println!("{:p}", &value);
        shard.insert(key, value);
    }

    fn insert2_p1_shard<'a, K: Eq + Hash, V, S: BuildHasher>(
        key: K,
        shard: &mut RwLockWriteGuard<'a, HashMap<K, V, S>>,
        value: V,
    ) {
        println!("{:p}", &key);
        println!("{}", shard.hasher().hash_one(&key));
        println!("{:p}", &value);
        // let shard: &mut RwLockWriteGuard<HashMap<K, SharedValue<V>>> =
        //             unsafe { std::mem::transmute(shard) };
        // let value: SharedValue<V> = SharedValue::new(value);
        println!("{:p}", &key);
        println!("{}", shard.hasher().hash_one(&key));
        println!("{:p}", &value);
        // todo!()
        shard.insert(key, value);
    }

    fn insert2_p2<'a, K: Eq + Hash, V, S: BuildHasher>(
        key: &K,
        shard: &mut RwLockWriteGuard<'a, HashMap<K, V, S>>,
    ) -> (*const K, *mut V) {
        unsafe {
            use std::mem;
            dbg!();
            println!("{:p}", &key);
            println!("{}", shard.hasher().hash_one(&key));
            let (k, v) = shard.get_key_value(key).unwrap();
            dbg!();
            let k = change_lifetime_const(k);
            dbg!();
            let v = &mut *shard_as_ptr2(v);
            dbg!();
            (k, v)
        }
    }

    fn insert2_p2_shard<'a, K: Eq + Hash, V, S: BuildHasher>(
        key: &K,
        shard: &mut RwLockWriteGuard<'a, HashMap<K, SharedValue<V>, S>>,
    ) -> (*const K, *mut V) {
        unsafe {
            dbg!();
            println!("{:p}", &key);
            println!("{}", shard.hasher().hash_one(&key));
            todo!();

            let (k, v) = shard.get_key_value(key).unwrap();
            dbg!();
            let k = change_lifetime_const(k);
            dbg!();
            let v = &mut *shard_as_ptr(v);
            dbg!();
            (k, v)
        }
    }

    fn insert<'a, K: Eq + Hash, V>(
        key: K,
        shard: &'a mut RwLockWriteGuard<'a, HashMap<K, SharedValue<V>>>,
        value: SharedValue<V>,
    ) -> (*const K, *mut V) {
        unsafe {
            use std::mem;
            use std::ptr;
            let c: K = ptr::read(&key);
            dbg!();
            println!("{:p}", &key);
            println!("{}", shard.hasher().hash_one(&key));
            println!("{:p}", &value);
            {
                // let shard: &mut RwLockWriteGuard<HashMap<K, V>> =
                //     unsafe { std::mem::transmute(shard) };
                // let value: V =
                //     unsafe { std::mem::transmute(value) };
                shard.insert(key, value);
            }
            dbg!();
            let (k, v) = shard.get_key_value(&c).unwrap();
            dbg!();
            let k = change_lifetime_const(k);
            dbg!();
            let v = &mut *shard_as_ptr(v);
            dbg!();
            mem::forget(c);
            dbg!();
            (k, v)
        }
    }

    /// # Safety
    ///
    /// Requires that you ensure the reference does not become invalid.
    /// The object has to outlive the reference.
    unsafe fn change_lifetime_const<'a, 'b, T>(x: &'a T) -> &'b T {
        &*(x as *const T)
    }

    pub struct RefMut<'a, K, V, S = RandomState> {
        guard1: RwLockWriteGuard<'a, HashMap<K, V, S>>,
        guard2: Option<RwLockWriteGuard<'a, HashMap<K, V, S>>>,
        k1: *const K,
        k2: *const K,
        v1: *mut V,
        v2: *mut V,
    }

    unsafe impl<'a, K: Eq + Hash + Sync, V: Sync, S: BuildHasher> Send for RefMut<'a, K, V, S> {}
    unsafe impl<'a, K: Eq + Hash + Sync, V: Sync, S: BuildHasher> Sync for RefMut<'a, K, V, S> {}

    impl<'a, K: Eq + Hash, V, S: BuildHasher> RefMut<'a, K, V, S> {
        pub fn value_mut(&mut self) -> (&mut V, &mut V) {
            unsafe { (&mut *self.v1, &mut *self.v2) }
        }
    }
}

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
        .find(|(_, x)| {
            stores.resolve_type(*x).eq(t)
        })
        .map(|(i, x)| (*x, i));
    s
}
