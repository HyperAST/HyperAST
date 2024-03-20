use hyper_ast::position::position_accessors::{self, SolvedPosition};
use hyper_ast_cvs_git::no_space::NoSpaceWrapper;
use hyper_diff::decompressed_tree_store::lazy_post_order;

use crate::MappingAloneCacheRef;

use super::*;

type IdD = u32;

type DecompressedTree<'store> =
    lazy_post_order::LazyPostOrder<NoSpaceWrapper<'store, super::IdN>, IdD>;

struct MappingTracker<'store, HAST> {
    stores: &'store HAST,
    // no_spaces_path_to_target: Vec<HAST::Idx>,
}

impl<'store, HAST> MappingTracker<'store, HAST> {
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

pub trait WithPreOrderOffsetsNoSpaces: WithOffsets {
    // type Path: Iterator;
    // fn path(&self) -> Self::Path;
    type It<'a>: Iterator<Item = &'a Self::Idx>
    where
        Self: 'a,
        Self::Idx: 'a;
    fn iter_offsets_nospaces(&self) -> Self::It<'_>;
}

pub(super) fn do_tracking<'store, 'p, P, C>(
    repositories: &'store multi_preprocessed::PreProcessedRepositories,
    partial_decomps: &PartialDecompCache,
    mappings_alone: &MappingAloneCache,
    flags: &Flags,
    // no_spaces_path_to_target: Vec<super::Idx>,
    target: &'p P,
    other_tr: super::IdN,
    postprocess_matching: &impl Fn(LocalPieceOfCode<super::IdN, super::Idx>) -> C,
) -> MappingResult<super::IdN, super::Idx, C>
where
    P: position_accessors::SolvedPosition<super::IdN>
        + position_accessors::RootedPosition<super::IdN>
        + position_accessors::WithPreOrderOffsets<Idx = super::Idx>
        + position_accessors::OffsetPostionT<super::IdN, IdO = usize>
        + self::WithPreOrderOffsetsNoSpaces,
{
    // let (target_node, target_range, path_to_target) = (
    //     target.node(),
    //     target.start()..target.end(),
    //     target.iter_offsets(),
    // );
    let with_spaces_stores = &repositories.processor.main_stores;
    let tracker_nospace = MappingTracker {
        stores: &hyper_ast_cvs_git::no_space::IntoNoSpaceGAT::as_nospaces(with_spaces_stores),
    };

    let current_tr = target.root();

    // NOTE: persists mappings, could also easily persist diffs,
    // but some compression on mappings could help
    // such as, not storing the decompression arenas
    // or encoding mappings more efficiently considering that most slices could simply by represented as ranges (ie. mapped identical subtrees)
    // or only storing the mapping costly to compute

    if current_tr == other_tr {
        // if both tree are identical lets just take a shortcut
        let nodes = tracker_nospace.size(&current_tr, &other_tr);
        let (src, matches) = {
            let c = compute_local2(target, with_spaces_stores);
            // the shortcut, it's ok because roots/trees are identical
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
    let (src_tree, dst_tree) =
        crate::utils::get_pair_simp(partial_decomps, stores, &current_tr, &other_tr);
    let (src_tree, dst_tree) = (src_tree.get_mut(), dst_tree.get_mut());

    if flags.some() {
        dbg!();
        if let Some(value) = track_top_down(
            with_spaces_stores,
            stores,
            src_tree,
            dst_tree,
            flags,
            target,
            postprocess_matching,
        ) {
            return value;
        }
    }

    let mapper_mappings = compute_mappings_bottom_up(
        stores,
        mappings_alone,
        src_tree,
        dst_tree,
    );
    let mapper_mappings = &mapper_mappings.1;

    let root = src_tree.root();
    let mapping_target = src_tree.child_decompressed(
        &stores.node_store,
        &root,
        target.iter_offsets_nospaces().copied(),
    );

    if let Some(mapped) = mapper_mappings.get_dst(&mapping_target) {
        return track_with_mappings(
            with_spaces_stores,
            stores,
            src_tree,
            dst_tree,
            flags,
            target,
            mapping_target,
            mapped,
            postprocess_matching,
        );
    }

    for parent_target in src_tree.parents(mapping_target) {
        if let Some(mapped_parent) = mapper_mappings.get_dst(&parent_target) {
            let fallback = {
                let (path, path_ids) = {
                    let dst_tree = dst_tree;
                    let mapped_parent = dst_tree.decompress_to(&stores.node_store, &mapped_parent);
                    assert_eq!(other_tr, dst_tree.original(&dst_tree.root()));
                    let path = dst_tree.path(&dst_tree.root(), &mapped_parent);
                    let mut path_ids = vec![dst_tree.original(&mapped_parent)];
                    dst_tree
                        .parents(mapped_parent)
                        .map(|i| dst_tree.original(&i))
                        .collect_into(&mut path_ids);
                    path_ids.pop();
                    assert_eq!(path.len(), path_ids.len());
                    (
                        path_with_spaces(other_tr, &mut path.iter().copied(), with_spaces_stores).0,
                        path_ids,
                    )
                };
                let (pos, mapped_node) =
                    compute_position(other_tr, &mut path.iter().copied(), with_spaces_stores);
                dbg!(&mapped_node);
                assert_eq!(&mapped_node, path_ids.last().unwrap()); // if it holds then ok to take the ids from the nospace repr.
                LocalPieceOfCode::from_position(&pos, path, path_ids)
            };
            return MappingResult::Missing {
                src: compute_local2(target, with_spaces_stores),
                fallback: postprocess_matching(fallback),
            };
        };
    }
    // lets try
    unreachable!("At least roots should have been mapped")
    // RATIONAL: For now I consider that mapping roots is part of the hypothesis when tracking a value between a tree pair,
    //           it is not necessary for all mapping algorithms,
    //           but providing a fallback is still useful,
    //           so that the user can descide if the element is really not there.
    //           The disaperance of an element should probably be descibed using a window of versions.
    //           Actually, relaxing the mapping process could always find a match for a given code element.
    //           Moreover, a mapping algorithm does not give an absolute result (would not mean much),
    //           it is just a process that minimizes the number of actions to go from one version to the other.
    //           In this context falling back to a clone detection approach seem more adapted.

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

fn track_with_mappings<'store, C, P>(
    with_spaces_stores: &SimpleStores<TStore>,
    stores: &'store NoSpaceStore<'_, 'store>,
    src_tree: &mut DecompressedTree<'store>,
    dst_tree: &mut DecompressedTree<'store>,
    flags: &Flags,
    target: &P,
    mapping_target: IdD,
    mapped: IdD,
    postprocess_matching: &impl Fn(LocalPieceOfCode<super::IdN, super::Idx>) -> C,
) -> MappingResult<super::IdN, super::Idx, C>
where
    P: SolvedPosition<super::IdN>
        + position_accessors::RootedPosition<super::IdN>
        + position_accessors::WithPreOrderOffsets<Idx = super::Idx>
        + position_accessors::OffsetPostionT<super::IdN, IdO = usize>,
{
    let other_tr = dst_tree.original(&dst_tree.root());
    let mapped = dst_tree.decompress_to(&stores.node_store, &mapped);
    let path_no_spaces = dst_tree.path_rooted(&mapped);
    let path_ids = {
        let mut path_ids = vec![dst_tree.original(&mapped)];
        dst_tree
            .parents(mapped)
            .map(|i| dst_tree.original(&i))
            .collect_into(&mut path_ids);
        path_ids.pop();
        path_ids
    };
    assert_eq!(path_no_spaces.len(), path_ids.len());
    let (path,) = path_with_spaces(
        other_tr,
        &mut path_no_spaces.iter().copied(),
        with_spaces_stores,
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
        let target_parent = src_tree.parent(&mapping_target);
        let target_parent = target_parent.map(|x| src_tree.original(&x));
        let mapped_parent = dst_tree.parent(&mapped);
        let mapped_parent = mapped_parent.map(|x| dst_tree.original(&x));
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
    let matches = vec![postprocess_matching(LocalPieceOfCode::from_position(
        &pos,
        path.clone(),
        path_ids.clone(),
    ))];
    let src = compute_local2(target, with_spaces_stores);
    if flagged && !triggered {
        let nodes = MappingTracker::new(stores).size(&other_tr, &target.root());
        MappingResult::Skipped {
            nodes,
            src,
            next: matches,
        }
    } else {
        MappingResult::Direct { src, matches }
    }
}

type NoSpaceStore<'a, 'store> = types::SimpleHyperAST<
    NoSpaceWrapper<'store, super::IdN>,
    &'a TStore,
    no_space::NoSpaceNodeStoreWrapper<'store>,
    &'a hyper_ast::store::labels::LabelStore,
>;

fn compute_mappings_bottom_up<'store, 'a>(
    stores: &'store NoSpaceStore<'_, 'store>,
    mappings_alone: &'a MappingAloneCache,
    src_tree: &mut DecompressedTree<'store>,
    dst_tree: &mut DecompressedTree<'store>,
) -> MappingAloneCacheRef<'a> {
    let mappings_cache = mappings_alone;
    use hyper_diff::matchers::mapping_store::MappingStore;
    use hyper_diff::matchers::mapping_store::VecStore;
    let hyperast = stores;
    use hyper_diff::matchers::Mapping;
    match mappings_cache.entry((
        src_tree.original(&src_tree.root()),
        dst_tree.original(&dst_tree.root()),
    )) {
        dashmap::mapref::entry::Entry::Occupied(entry) => entry.into_ref().downgrade(),
        dashmap::mapref::entry::Entry::Vacant(entry) => {
            let mappings = VecStore::default();
            let mut mapper = Mapper {
                hyperast,
                mapping: Mapping {
                    src_arena: src_tree,
                    dst_arena: dst_tree,
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

const CONST_NODE_COUNTING: Option<usize> = Some(500_000);

fn track_top_down<'store, C, P>(
    with_spaces_stores: &'store SimpleStores<TStore>,
    stores: &'store NoSpaceStore<'_, 'store>,
    src_tree: &mut DecompressedTree<'store>,
    dst_tree: &mut DecompressedTree<'store>,
    // no_spaces_path_to_target: &[u16],
    flags: &Flags,
    target: &P,
    postprocess_matching: &impl Fn(LocalPieceOfCode<super::IdN, super::Idx>) -> C,
) -> Option<MappingResult<super::IdN, super::Idx, C>>
where
    P: position_accessors::SolvedPosition<super::IdN>
        + position_accessors::RootedPosition<super::IdN>
        + position_accessors::WithPreOrderOffsets<Idx = super::Idx>
        + position_accessors::OffsetPostionT<super::IdN, IdO = usize>
        + self::WithPreOrderOffsetsNoSpaces,
{
    let current_tr = target.root();
    let other_tr = dst_tree.original(&dst_tree.root());
    assert_eq!(current_tr, src_tree.original(&src_tree.root()));
    let node_store = &stores.node_store;
    let tracker_nospace = MappingTracker {
        stores: &hyper_ast_cvs_git::no_space::IntoNoSpaceGAT::as_nospaces(with_spaces_stores),
    };
    let mapped = {
        let hyperast = stores;
        let src = current_tr;
        let dst = other_tr;
        matching::top_down(hyperast, src_tree, dst_tree)
    };
    let mapper_mappings = &mapped;
    let mut curr = src_tree.root();
    let path = target.iter_offsets_nospaces().copied().collect::<Vec<_>>();
    let mut path = &path[..];
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
                let nodes = CONST_NODE_COUNTING
                    .unwrap_or_else(|| tracker_nospace.size(&other_tr, &current_tr));

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
                            let path_dst = dst_tree.path_rooted(x);
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
                let nodes = CONST_NODE_COUNTING
                    .unwrap_or_else(|| tracker_nospace.size(&other_tr, &current_tr));
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
                            let mut path_dst = dst_tree.path_rooted(x);
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
                let nodes = CONST_NODE_COUNTING
                    .unwrap_or_else(|| tracker_nospace.size(&other_tr, &current_tr));
                return Some(MappingResult::Skipped {
                    nodes,
                    src: compute_local2(target, with_spaces_stores),
                    next: dsts
                        .iter()
                        .map(|x| {
                            let mut path_dst = dst_tree.path_rooted(x);
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
        let cs = src_tree.decompress_children(node_store, &curr);
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
    path: &(impl position_accessors::WithPreOrderOffsets<Idx = super::Idx>
          + position_accessors::RootedPosition<super::IdN>),
    with_spaces_stores: &SimpleStores<TStore>,
) -> LocalPieceOfCode<super::IdN, super::Idx> {
    let tr = path.root();
    let (pos, path_ids) =
        compute_position_and_nodes(tr, &mut path.iter_offsets().copied(), with_spaces_stores);
    let path = path.iter_offsets().copied().collect();
    LocalPieceOfCode::from_position(&pos, path, path_ids)
}

#[derive(EnumSetType, Debug)]
pub enum FlagsE {
    Upd,
    Child,
    Parent,
    ExactChild,
    ExactParent,
    SimChild,
    SimParent,
    Meth,
    Typ,
    Top,
    File,
    Pack,
    Dependency,
    Dependent,
    References,
    Declaration,
}

impl Into<EnumSet<FlagsE>> for &Flags {
    fn into(self) -> EnumSet<FlagsE> {
        let mut r = EnumSet::new();
        if self.upd {
            r.insert(FlagsE::Upd);
        }
        if self.child {
            r.insert(FlagsE::Child);
        }
        if self.parent {
            r.insert(FlagsE::Parent);
        }
        if self.exact_child {
            r.insert(FlagsE::ExactChild);
        }
        if self.exact_parent {
            r.insert(FlagsE::ExactParent);
        }
        if self.sim_child {
            r.insert(FlagsE::SimChild);
        }
        if self.sim_parent {
            r.insert(FlagsE::SimParent);
        }
        if self.meth {
            r.insert(FlagsE::Meth);
        }
        if self.typ {
            r.insert(FlagsE::Typ);
        }
        if self.top {
            r.insert(FlagsE::Top);
        }
        if self.file {
            r.insert(FlagsE::File);
        }
        if self.pack {
            r.insert(FlagsE::Pack);
        }
        if self.dependency {
            r.insert(FlagsE::Dependency);
        }
        if self.dependent {
            r.insert(FlagsE::Dependent);
        }
        if self.references {
            r.insert(FlagsE::References);
        }
        if self.declaration {
            r.insert(FlagsE::Declaration);
        }
        r
    }
}
