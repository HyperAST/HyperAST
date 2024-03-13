use hyper_ast::position::position_accessors::{self, SolvedPosition};
use hyper_ast_cvs_git::no_space::{NoSpaceMarker, NoSpaceWrapper};

use crate::utils::LPO;

use super::*;

struct MappingTracker<R: ConfiguredRepoTrait> {
    repo_handle: R,
}

impl<R: ConfiguredRepoTrait> MappingTracker<R> {
    fn with_flags(self, flags: &Flags) -> Self {
        self
    }
}

struct MappingTracker2<'x, 'y, HAST, R: ConfiguredRepoTrait> {
    repo_handle: &'x R,
    stores: &'y HAST,
    // no_spaces_path_to_target: Vec<HAST::Idx>,
}

impl<'x, 'store, HAST, R: ConfiguredRepoTrait> MappingTracker2<'x, 'store, HAST, R> {
    fn new(&self, repo_handle: &'x R, stores: &'store HAST) -> Self {
        Self {
            repo_handle,
            stores,
        }
    }
    fn change_store(&self, repo_handle: &'x R, stores: &'store HAST) -> Self {
        Self {
            repo_handle,
            stores,
        }
    }
}

struct MappingTracker3<'store, HAST> {
    stores: &'store HAST,
    // no_spaces_path_to_target: Vec<HAST::Idx>,
}

impl<'store, HAST> MappingTracker3<'store, HAST> {
    fn new(stores: &'store HAST) -> Self {
        Self { stores }
    }
    fn size(&self, other_tr: &HAST::IdN, current_tr: &HAST::IdN) -> usize
    where
        HAST: HyperAST<'store>,
        HAST::T: types::WithStats,
    {
        let node_store = self.stores.node_store();
        let src_size: usize = node_store.resolve(&other_tr).size();
        let dst_size: usize = node_store.resolve(&current_tr).size();
        src_size + dst_size
    }
}

impl<'x, 'store, HAST, R: ConfiguredRepoTrait> MappingTracker2<'x, 'store, HAST, R> {
    fn size_not_working(&self, other_tr: &HAST::IdN, current_tr: &HAST::IdN) -> usize
    where
        HAST: HyperAST<'store> + NoSpaceMarker,
        HAST: no_space::AsNoSpace,
        for<'s> HAST::T: types::WithSerialization,
        HAST::R: HyperAST<'store, IdN = HAST::IdN>,
        for<'s> <HAST::R as HyperAST<'store>>::T: types::WithStats,
    {
        let stores = self.stores.as_nospaces();
        let node_store = stores.node_store();

        let src_size: usize = node_store.resolve(&other_tr).size();
        let dst_size: usize = node_store.resolve(&current_tr).size();
        src_size + dst_size
    }
    fn compute_matches<P>(
        &self,
        current_tr: HAST::IdN,
        target: &P,
    ) -> (
        LocalPieceOfCode<HAST::IdN, HAST::Idx>,
        Vec<LocalPieceOfCode<HAST::IdN, HAST::Idx>>,
    )
    where
        HAST: HyperAST<'store>,
        HAST::T: types::WithSerialization,
        for<'a> (&'store HAST, &'a P): Into<LocalPieceOfCode<HAST::IdN, HAST::Idx>>,
    {
        // let path_to_target = target.iter();

        let c: LocalPieceOfCode<_, _> = Into::into((self.stores, target));

        // let (pos, path_ids) = hyper_ast::position::compute_position_and_nodes(
        //     current_tr,
        //     &mut path_to_target.map(|x| *x),
        //     self.stores,
        // );
        // dbg!();
        // let range = pos.range();
        // let c = LocalPieceOfCode::from_pos(&pos);
        let matches = vec![c.clone()];
        let src = c;
        (src, matches)
    }
}

pub(super) fn do_tracking<'store, 'p, P>(
    repo_handle: &impl ConfiguredRepoTrait,
    current_tr: NodeIdentifier,
    other_tr: NodeIdentifier,
    flags: &Flags,
    partial_decomps: &PartialDecompCache,
    mappings_alone: &MappingAloneCache,
    repositories: &'store multi_preprocessed::PreProcessedRepositories,
    dst_oid: impl ToString,
    no_spaces_path_to_target: Vec<super::Idx>,
    target: &'p P,
) -> MappingResult<NodeIdentifier, super::Idx>
where
    P: SolvedPosition<NodeIdentifier>
        + position_accessors::WithPreOrderOffsets<Idx = super::Idx>
        + position_accessors::OffsetPostionT<NodeIdentifier, IdO = usize>,
    P::It<'p>: Clone,
{
    // let (target_node, target_range, path_to_target) = (
    //     target.node(),
    //     target.start()..target.end(),
    //     target.iter_offsets(),
    // );
    let with_spaces_stores = &repositories.processor.main_stores;
    let tracker_nospace = MappingTracker3 {
        stores: &hyper_ast_cvs_git::no_space::IntoNoSpaceGAT::as_nospaces(with_spaces_stores),
    };
    let postprocess_matching = |p: LocalPieceOfCode<super::IdN, super::Idx>| {
        p.globalize(repo_handle.spec().clone(), dst_oid.to_string())
    };

    // NOTE: persists mappings, could also easily persist diffs,
    // but some compression on mappings could help
    // such as, not storing the decompression arenas
    // or encoding mappings more efficiently considering that most slices could simply by represented as ranges (ie. mapped identical subtrees)
    // or only storing the mapping costly to compute
    // let mapper = lazy_mapping(repos, &mut state.mappings, src_tr, dst_tr);

    // dbg!(src_tr, dst_tr);
    if current_tr == other_tr {
        let nodes = tracker_nospace.size(&current_tr, &other_tr);
        let (src, matches) = {
            let c = compute_local2(other_tr, target, with_spaces_stores);
            let matches = vec![c.clone()];
            let src = c;
            (src, matches)
        };

        let matches = matches.into_iter().map(postprocess_matching).collect();
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
    let stores = &no_space::as_nospaces(with_spaces_stores);
    let node_store = &stores.node_store;
    let mut pair = get_pair_simp(partial_decomps, stores, &current_tr, &other_tr);

    if flags.some() {
        dbg!();
        if let Some(value) = track_top_down(
            current_tr,
            other_tr,
            &mut pair,
            &no_spaces_path_to_target,
            flags,
            target,
            with_spaces_stores,
            stores,
            postprocess_matching,
        ) {
            return value;
        }
    }

    let mapped =
        compute_mappings_bottom_up(mappings_alone, stores, current_tr, other_tr, &mut pair);
    let (mapper_src_arena, mapper_dst_arena) = (pair.0.get_mut(), pair.1.get_mut());
    let mapper_mappings = &mapped.1;
    let root = mapper_src_arena.root();
    let mapping_target =
        mapper_src_arena.child_decompressed(node_store, &root, &no_spaces_path_to_target);

    if let Some(mapped) = mapper_mappings.get_dst(&mapping_target) {
        return track_with_mappings(
            mapper_dst_arena,
            mapped,
            other_tr,
            repositories,
            with_spaces_stores,
            flags,
            stores,
            mapper_src_arena,
            mapping_target,
            current_tr,
            target,
            postprocess_matching,
        );
    }

    for parent_target in mapper_src_arena.parents(mapping_target) {
        if let Some(mapped_parent) = mapper_mappings.get_dst(&parent_target) {
            let mapped_parent = mapper_dst_arena.decompress_to(node_store, &mapped_parent);
            assert_eq!(
                other_tr,
                mapper_dst_arena.original(&mapper_dst_arena.root())
            );
            let path = mapper_dst_arena.path(&mapper_dst_arena.root(), &mapped_parent);
            let mut path_ids = vec![mapper_dst_arena.original(&mapped_parent)];
            mapper_dst_arena
                .parents(mapped_parent)
                .map(|i| mapper_dst_arena.original(&i))
                .collect_into(&mut path_ids);
            path_ids.pop();
            assert_eq!(path.len(), path_ids.len());
            let (path,) = path_with_spaces(
                other_tr,
                &mut path.iter().copied(),
                &repositories.processor.main_stores,
            );
            let (pos, mapped_node) =
                compute_position(other_tr, &mut path.iter().copied(), with_spaces_stores);
            dbg!(&mapped_node);
            assert_eq!(&mapped_node, path_ids.last().unwrap());
            let fallback = LocalPieceOfCode::from_file_and_range(
                pos.file(),
                target.start()..target.end(),
                path,
                path_ids,
            )
            .globalize(repo_handle.spec().clone(), dst_oid.to_string());

            let src = {
                let path = target.iter_offsets().copied().collect();
                let (target_pos, target_path_ids) = compute_position_and_nodes(
                    current_tr,
                    &mut target.iter_offsets().copied(),
                    with_spaces_stores,
                );
                LocalPieceOfCode::from_file_and_range(
                    target_pos.file(),
                    target.start()..target.end(),
                    path,
                    target_path_ids,
                )
            };
            return MappingResult::Missing { src, fallback };
        };
    }
    // lets try
    unreachable!()
    // let path = path_to_target.clone();
    // let (target_pos, target_path_ids) = compute_position_and_nodes(
    //     other_tr,
    //     &mut path_to_target.iter().copied(),
    //     with_spaces_stores,
    // );
    // // TODO what should be done if there is no match ?
    // MappingResult::Direct {
    //     src: LocalPieceOfCode::from_file_and_range(
    //         target_pos.file(),
    //         target_range,
    //         path,
    //         target_path_ids,
    //     ),
    //     matches: vec![],
    // }
}

type IdD = u32;

fn track_with_mappings<'store, C, P>(
    mapper_dst_arena: &mut hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder<
        NoSpaceWrapper<'store, super::IdN>,
        IdD,
    >,
    mapped: IdD,
    other_tr: super::IdN,
    repositories: &multi_preprocessed::PreProcessedRepositories,
    with_spaces_stores: &SimpleStores<TStore>,
    flags: &Flags,
    stores: &'store types::SimpleHyperAST<
        NoSpaceWrapper<'store, NodeIdentifier>,
        &TStore,
        no_space::NoSpaceNodeStoreWrapper<'store>,
        &hyper_ast::store::labels::LabelStore,
    >,
    mapper_src_arena: &mut hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder<
        NoSpaceWrapper<'store, super::IdN>,
        IdD,
    >,
    mapping_target: IdD,
    current_tr: super::IdN,
    target: &P,
    postprocess_matching: impl Fn(LocalPieceOfCode<super::IdN, super::Idx>) -> C,
) -> MappingResult<super::IdN, super::Idx, C>
where
    P: SolvedPosition<NodeIdentifier>
        + position_accessors::WithPreOrderOffsets<Idx = super::Idx>
        + position_accessors::OffsetPostionT<NodeIdentifier, IdO = usize>,
{
    let mut matches = vec![];
    let node_store = &stores.node_store;
    let mapped = mapper_dst_arena.decompress_to(node_store, &mapped);
    assert_eq!(
        other_tr,
        mapper_dst_arena.original(&mapper_dst_arena.root())
    );
    let path = mapper_dst_arena.path(&mapper_dst_arena.root(), &mapped);
    let path_ids = {
        let mut path_ids = vec![mapper_dst_arena.original(&mapped)];
        mapper_dst_arena
            .parents(mapped)
            .map(|i| mapper_dst_arena.original(&i))
            .collect_into(&mut path_ids);
        path_ids.pop();
        path_ids
    };
    assert_eq!(path.len(), path_ids.len());
    let (path,) = path_with_spaces(
        other_tr,
        &mut path.iter().copied(),
        &repositories.processor.main_stores,
    );
    let (pos, mapped_node) =
        compute_position(other_tr, &mut path.iter().copied(), with_spaces_stores);
    dbg!(&pos);
    dbg!(&mapped_node);
    let mut flagged = false;
    let mut triggered = false;
    if flags.exact_child {
        flagged = true;
        dbg!();
        triggered |= target.node() != mapped_node;
    }
    if flags.child || flags.sim_child {
        flagged = true;
        dbg!();

        let target_node = stores.node_store.resolve(target.node());
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
    matches.push(postprocess_matching(LocalPieceOfCode::from_position(
        &pos,
        path.clone(),
        path_ids.clone(),
    )));
    if flagged && !triggered {
        let src_size = stores.node_store.resolve(current_tr).size();
        let dst_size = stores.node_store.resolve(other_tr).size();
        let nodes = src_size + dst_size;
        MappingResult::Skipped {
            nodes,
            src: {
                let (pos, path_ids) = compute_position_and_nodes(
                    current_tr,
                    &mut target.iter_offsets().copied(),
                    with_spaces_stores,
                );

                LocalPieceOfCode::from_file_and_range(
                    pos.file(),
                    target.start()..target.end(),
                    target.iter_offsets().copied().collect(),
                    path_ids,
                )
            },
            next: matches,
        }
    } else {
        let path = target.iter_offsets().copied().collect();
        let (target_pos, target_path_ids) = compute_position_and_nodes(
            current_tr,
            &mut target.iter_offsets().copied(),
            with_spaces_stores,
        );
        MappingResult::Direct {
            src: LocalPieceOfCode::from_file_and_range(
                target_pos.file(),
                target.start()..target.end(),
                path,
                target_path_ids,
            ),
            matches,
        }
    }
}

fn compute_mappings_bottom_up<'store, 'a, S>(
    mappings_alone: &'a dashmap::DashMap<
        (super::IdN, super::IdN),
        (crate::MappingStage, mapping_store::VecStore<IdD>),
        S,
    >,
    stores: &'store types::SimpleHyperAST<
        NoSpaceWrapper<'store, NodeIdentifier>,
        &TStore,
        no_space::NoSpaceNodeStoreWrapper<'store>,
        &hyper_ast::store::labels::LabelStore,
    >,
    other_tr: super::IdN,
    current_tr: super::IdN,
    pair: &mut (
        &mut LPO<NoSpaceWrapper<'store, super::IdN>>,
        &mut LPO<NoSpaceWrapper<'store, super::IdN>>,
    ),
) -> dashmap::mapref::one::Ref<
    'a,
    (super::IdN, super::IdN),
    (crate::MappingStage, mapping_store::VecStore<IdD>),
    S,
>
where
    S: BuildHasher,
    S: Clone,
{
    let mappings_cache = mappings_alone;
    use hyper_diff::matchers::mapping_store::MappingStore;
    use hyper_diff::matchers::mapping_store::VecStore;
    let hyperast = stores;
    use hyper_diff::matchers::Mapping;
    dbg!();
    match mappings_cache.entry((other_tr, current_tr)) {
        dashmap::mapref::entry::Entry::Occupied(entry) => entry.into_ref().downgrade(),
        dashmap::mapref::entry::Entry::Vacant(entry) => {
            let mappings = VecStore::default();
            let (src_arena, dst_arena) = (pair.0.get_mut(), pair.1.get_mut());
            let mut mapper = Mapper {
                hyperast,
                mapping: Mapping {
                    src_arena,
                    dst_arena,
                    mappings,
                },
            };
            mapper.mapping.mappings.topit(
                mapper.mapping.src_arena.len(),
                mapper.mapping.dst_arena.len(),
            );

            let vec_store = matching::full2(hyperast, mapper);

            entry
                .insert((crate::MappingStage::Bottomup, vec_store))
                .downgrade()
        }
    }
}

fn track_top_down<'store, C, P>(
    current_tr: super::IdN,
    other_tr: super::IdN,
    pair: &mut (
        &mut LPO<NoSpaceWrapper<'store, super::IdN>>,
        &mut LPO<NoSpaceWrapper<'store, super::IdN>>,
    ),
    no_spaces_path_to_target: &[u16],
    flags: &Flags,
    target: &P,
    with_spaces_stores: &'store SimpleStores<TStore>,
    stores: &'store types::SimpleHyperAST<
        NoSpaceWrapper<'store, super::IdN>,
        &TStore,
        no_space::NoSpaceNodeStoreWrapper<'store>,
        &hyper_ast::store::labels::LabelStore,
    >,
    postprocess_matching: impl Fn(LocalPieceOfCode<super::IdN, super::Idx>) -> C,
) -> Option<MappingResult<super::IdN, super::Idx, C>>
where
    P: SolvedPosition<NodeIdentifier>
        + position_accessors::WithPreOrderOffsets<Idx = super::Idx>
        + position_accessors::OffsetPostionT<NodeIdentifier, IdO = usize>,
{
    let node_store = &stores.node_store;
    let tracker_nospace = MappingTracker3 {
        stores: &hyper_ast_cvs_git::no_space::IntoNoSpaceGAT::as_nospaces(with_spaces_stores),
    };
    let mapped = {
        let hyperast = stores;
        let src = &current_tr;
        let dst = &other_tr;
        let (src_arena, dst_arena) = (pair.0.get_mut(), pair.1.get_mut());
        matching::top_down(hyperast, src_arena, dst_arena)
    };
    let (mapper_src_arena, mapper_dst_arena) = (pair.0.get_mut(), pair.1.get_mut());
    let mapper_mappings = &mapped;
    let mut curr = mapper_src_arena.root();
    let mut path = no_spaces_path_to_target;
    let flags: EnumSet<_> = flags.into();
    loop {
        // dbg!(path);
        let dsts = mapper_mappings.get_dsts(&curr);
        let curr_flags = FlagsE::Upd | FlagsE::Child | FlagsE::SimChild; //  | FlagsE::ExactChild
        let parent_flags = curr_flags | FlagsE::Parent | FlagsE::SimParent; //  | FlagsE::ExactParent
        if dsts.is_empty() {
            // continue through path_to_target
            // dbg!(curr);
        } else if path.len() == 0 {
            // need to check curr node flags
            if flags.is_subset(curr_flags) {
                // only trigger on curr and children changed
                let nodes = tracker_nospace.size(&current_tr, &other_tr);
                let nodes = 500_000;

                return Some(MappingResult::Skipped {
                    nodes,
                    src: {
                        let (pos, path_ids) = compute_position_and_nodes(
                            current_tr,
                            &mut target.iter_offsets().copied(),
                            with_spaces_stores,
                        );

                        LocalPieceOfCode::from_file_and_range(
                            pos.file(),
                            target.start()..target.end(),
                            target.iter_offsets().copied().collect(),
                            path_ids,
                        )
                    },
                    next: dsts
                        .iter()
                        .map(|x| {
                            assert_eq!(
                                other_tr,
                                mapper_dst_arena.original(&mapper_dst_arena.root())
                            );
                            let path_dst = mapper_dst_arena.path(&mapper_dst_arena.root(), x);
                            let (path_dst,) = path_with_spaces(
                                other_tr,
                                &mut path_dst.iter().copied(),
                                with_spaces_stores,
                            );
                            postprocess_matching(compute_local(
                                other_tr,
                                &path_dst,
                                with_spaces_stores,
                            ))
                        })
                        .collect(),
                });
            }
            // also the type of src and dsts
            // also check it file path changed
            // can we test if parent changed ? at least we can ckeck some attributes
        } else if path.len() == 1 {
            // need to check parent node flags
            if flags.is_subset(parent_flags) {
                // only trigger on parent, curr and children changed
                let nodes = tracker_nospace.size(&current_tr, &other_tr);
                let nodes = 500_000;
                return Some(MappingResult::Skipped {
                    nodes,
                    src: {
                        let (pos, path_ids) = compute_position_and_nodes(
                            current_tr,
                            &mut target.iter_offsets().copied(),
                            with_spaces_stores,
                        );

                        let path = target.iter_offsets().copied().collect();
                        LocalPieceOfCode::from_position(&pos, path, path_ids)
                    },
                    next: dsts
                        .iter()
                        .map(|x| {
                            assert_eq!(
                                other_tr,
                                mapper_dst_arena.original(&mapper_dst_arena.root())
                            );
                            let mut path_dst = mapper_dst_arena.path(&mapper_dst_arena.root(), x);
                            path_dst.extend(path); // WARN with similarity it might not be possible to simply concat path...
                            let (path_dst,) = path_with_spaces(
                                other_tr,
                                &mut path_dst.iter().copied(),
                                with_spaces_stores,
                            );
                            postprocess_matching(compute_local(
                                other_tr,
                                &path_dst,
                                with_spaces_stores,
                            ))
                        })
                        .collect(),
                });
            }
            // also the type of src and dsts
            // also check if file path changed
            // can we test if parent changed ? at least we can ckeck some attributes
        } else {
            // need to check flags, the type of src and dsts
            if flags.is_subset(parent_flags) {
                // only trigger on parent, curr and children changed
                let nodes = tracker_nospace.size(&current_tr, &other_tr);
                let nodes = 500_000;
                return Some(MappingResult::Skipped {
                    nodes,
                    src: compute_local2(current_tr, target, with_spaces_stores),
                    next: dsts
                        .iter()
                        .map(|x| {
                            assert_eq!(
                                other_tr,
                                mapper_dst_arena.original(&mapper_dst_arena.root())
                            );
                            let mut path_dst = mapper_dst_arena.path(&mapper_dst_arena.root(), x);
                            path_dst.extend(path); // WARN with similarity it might not be possible to simply concat path...
                            let (path_dst,) = path_with_spaces(
                                other_tr,
                                &mut path_dst.iter().copied(),
                                with_spaces_stores,
                            );
                            postprocess_matching(compute_local(
                                other_tr,
                                &path_dst,
                                with_spaces_stores,
                            ))
                        })
                        .collect(),
                });
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
    None
}

fn compute_local(
    tr: super::IdN,
    path: &[super::Idx],
    with_spaces_stores: &SimpleStores<TStore>,
) -> LocalPieceOfCode<super::IdN, super::Idx> {
    let (pos, path_ids) =
        compute_position_and_nodes(tr, &mut path.iter().copied(), with_spaces_stores);
    let path = path.to_vec();
    LocalPieceOfCode::from_position(&pos, path, path_ids)
}

fn compute_local2(
    tr: super::IdN,
    path: &impl position_accessors::WithPreOrderOffsets<Idx = super::Idx>,
    with_spaces_stores: &SimpleStores<TStore>,
) -> LocalPieceOfCode<super::IdN, super::Idx> {
    let (pos, path_ids) =
        compute_position_and_nodes(tr, &mut path.iter_offsets().copied(), with_spaces_stores);
    let path = path.iter_offsets().copied().collect();
    LocalPieceOfCode::from_position(&pos, path, path_ids)
}
