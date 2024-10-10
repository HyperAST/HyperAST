use super::Diff;
use super::Idx;
use super::Pos;
use super::*;
use hyper_ast::store::defaults::LabelIdentifier;
use hyper_ast::store::defaults::NodeIdentifier;
use hyper_ast::store::labels::LabelStore;
use hyper_ast::types::HyperAST;
use hyper_ast_cvs_git::no_space::NoSpaceWrapper;
use hyper_diff::actions::action_tree::ActionsTree;
use hyper_diff::actions::action_vec::ActionsVec;
use hyper_diff::actions::script_generator2::Act;
use hyper_diff::actions::script_generator2::ScriptGenerator;
use hyper_diff::actions::script_generator2::SimpleAction;
use hyper_diff::tree::tree_path::CompressedTreePath;

pub(crate) struct T;

impl hyper_ast::types::Node for T {}

impl hyper_ast::types::Stored for T {
    type TreeId = NodeIdentifier;
}

impl hyper_ast::types::WithChildren for T {
    type ChildIdx = u16;

    type Children<'a>
        = hyper_ast::types::MySlice<NodeIdentifier>
    where
        Self: 'a;

    fn child_count(&self) -> Self::ChildIdx {
        todo!()
    }

    fn child(
        &self,
        idx: &Self::ChildIdx,
    ) -> Option<<Self::TreeId as hyper_ast::types::NodeId>::IdN> {
        todo!()
    }

    fn child_rev(
        &self,
        idx: &Self::ChildIdx,
    ) -> Option<<Self::TreeId as hyper_ast::types::NodeId>::IdN> {
        todo!()
    }

    fn children(&self) -> Option<&Self::Children<'_>> {
        todo!()
    }
}

impl hyper_ast::types::Labeled for T {
    type Label = LabelIdentifier;

    fn get_label_unchecked<'a>(&'a self) -> &'a Self::Label {
        todo!()
    }

    fn try_get_label<'a>(&'a self) -> Option<&'a Self::Label> {
        todo!()
    }
}

pub(crate) fn diff(
    state: std::sync::Arc<crate::AppState>,
    repo_handle: &impl hyper_ast_cvs_git::processing::ConfiguredRepoTrait<
        Config = hyper_ast_cvs_git::processing::ParametrizedCommitProcessorHandle,
    >,
    src_oid: hyper_ast_cvs_git::git::Oid,
    dst_oid: hyper_ast_cvs_git::git::Oid,
) -> Result<Diff, String> {
    let repositories = state.repositories.read().unwrap();
    let commit_src = repositories
        .get_commit(repo_handle.config(), &src_oid)
        .unwrap();
    let src_tr = commit_src.ast_root;
    let commit_dst = repositories
        .get_commit(repo_handle.config(), &dst_oid)
        .unwrap();
    let dst_tr = commit_dst.ast_root;
    let with_spaces_stores = &repositories.processor.main_stores;
    let stores = &hyper_ast_cvs_git::no_space::as_nospaces(with_spaces_stores);

    if src_tr == dst_tr {
        return Ok(Diff {
            focuses: Default::default(),
            deletes: Default::default(),
            inserts: Default::default(),
            moves: Default::default(),
        });
    }

    let pair = crate::utils::get_pair_simp(&state.partial_decomps, stores, &src_tr, &dst_tr);
    use hyper_ast::types::WithStats;
    use hyper_diff::decompressed_tree_store::ShallowDecompressedTreeStore;
    let mapped = {
        let mappings_cache = &state.mappings_alone;
        use hyper_diff::matchers::mapping_store::MappingStore;
        use hyper_diff::matchers::mapping_store::VecStore;

        let hyperast = stores;
        use hyper_diff::matchers::Mapping;

        dbg!();
        match mappings_cache.entry((src_tr, dst_tr)) {
            dashmap::mapref::entry::Entry::Occupied(entry) => entry.into_ref().downgrade(),
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                // std::collections::hash_map::Entry::Vacant(entry) => {
                let mappings = VecStore::default();
                let (src_arena, dst_arena) = (pair.0.get_mut(), pair.1.get_mut());
                dbg!(src_arena.len());
                dbg!(dst_arena.len());
                let src_size = stores.node_store.resolve(src_tr).size();
                let dst_size = stores.node_store.resolve(dst_tr).size();
                dbg!(src_size);
                dbg!(dst_size);
                let mut mapper = hyper_diff::matchers::Mapper {
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
                crate::matching::full2(hyperast, &mut mapper);

                // TODO match decls by sig/path

                let vec_store = mapper.mappings.clone();

                dbg!();
                entry
                    .insert((crate::MappingStage::Bottomup, vec_store))
                    .downgrade()
            }
        }
    };
    let (src_arena, dst_arena) = (pair.0.get_mut(), pair.1.get_mut());
    dbg!();
    src_arena.complete_subtree(&stores.node_store, &src_arena.root());
    let src_arena =
        hyper_diff::decompressed_tree_store::complete_post_order_ref::CompletePostOrder::from(
            &*src_arena,
        );
    dbg!();
    dst_arena.complete_subtree(&stores.node_store, &dst_arena.root());
    let dst_arena =
        hyper_diff::decompressed_tree_store::complete_post_order_ref::CompletePostOrder::from(
            &*dst_arena,
        );
    dbg!();
    let dst_arena = hyper_diff::decompressed_tree_store::bfs_wrapper::SimpleBfsMapper::from(
        &stores.node_store,
        dst_arena,
    );
    dbg!();
    let ms = &mapped.1;
    let mapping = hyper_diff::matchers::Mapping {
        src_arena,
        dst_arena,
        mappings: ms.clone(),
    };
    let actions = {
        let mapping = &mapping;
        let store = stores.node_store();

        let mut this = ScriptGenerator::new(store, &mapping.src_arena, &mapping.dst_arena)
            .init_cpy(&mapping.mappings);
        this.auxilary_ins_mov_upd(&|w, x| {
            assert_eq!(stores.resolve_type(w), stores.resolve_type(x))
        })?;
        this.del();
        this.actions
    };
    // ;

    dbg!(&actions.len());

    enum Choice {
        Del,
        Mov,
        Mov2,
        Ins,
        Upd,
        Mov2_Del,
    }
    let choice = Choice::Mov2_Del;
    let mut focuses = vec![];
    let mut deletes = vec![];
    let mut inserts = vec![];
    let moves = if let Choice::Del = choice {
        extract_deletes(with_spaces_stores, stores, src_tr, dst_tr, &actions).collect()
    } else if let Choice::Ins = choice {
        extract_inserts(with_spaces_stores, stores, src_tr, dst_tr, &actions).collect()
    } else if let Choice::Upd = choice {
        extract_updates(with_spaces_stores, stores, src_tr, dst_tr, &actions).collect()
    } else if let Choice::Mov = choice {
        extract_moves(with_spaces_stores, stores, src_tr, dst_tr, &actions).collect()
    } else if let Choice::Mov2 = choice {
        extract_moves2(with_spaces_stores, stores, src_tr, dst_tr, &actions).collect()
    } else if let Choice::Mov2_Del = choice {
        let foc = extract_focuses(with_spaces_stores, stores, src_tr, dst_tr, &actions);
        focuses = foc.collect();
        let dels = extract_deletes(with_spaces_stores, stores, src_tr, dst_tr, &actions);
        deletes = dels.map(|x| x.0).collect();
        let ins = extract_inserts(with_spaces_stores, stores, src_tr, dst_tr, &actions);
        inserts = ins.map(|x| x.1).collect();
        let movs = extract_moves2(with_spaces_stores, stores, src_tr, dst_tr, &actions);
        movs.collect()
    } else {
        unreachable!()
    };

    Ok(Diff {
        focuses,
        deletes,
        inserts,
        moves,
    })
}

pub(crate) type A = SimpleAction<LabelIdentifier, CompressedTreePath<u16>, NodeIdentifier>;

pub(crate) fn extract_moves<'a>(
    with_spaces_stores: &'a hyper_ast::store::SimpleStores<hyper_ast_cvs_git::TStore>,
    stores: &'a Stores,
    src_tr: NodeIdentifier,
    dst_tr: NodeIdentifier,
    actions: &'a ActionsVec<A>,
) -> impl Iterator<Item = (Pos, Pos)> + 'a {
    let mut result = vec![];
    let mut a_tree = ActionsTree::new();
    for a in actions.0.iter() {
        if let Act::Move { from } = &a.action {
            dbg!(from.ori.iter().count(), a.path.ori.iter().count());
            let (_, w) = hyper_ast::position::path_with_spaces(
                src_tr,
                &mut from.ori.iter(),
                with_spaces_stores,
            );
            let (_, x) = hyper_ast::position::path_with_spaces(
                dst_tr,
                &mut a.path.ori.iter(),
                with_spaces_stores,
            );
            assert_eq!(
                hyper_ast::types::HyperAST::resolve_type(stores, &w),
                hyper_ast::types::HyperAST::resolve_type(stores, &x)
            );

            a_tree.merge_ori(a);
        }
    }
    // eprintln!("{:?}", a_tree.inspect());
    use hyper_ast::types::HyperType;
    go_to_files(
        stores,
        &a_tree.atomics,
        hyper_ast::position::StructuralPosition::new(dst_tr),
        &mut |p, nn, n, id| {
            let t = stores.resolve_type(&id);
            // dbg!(t.as_static_str(), p);
            // if t.is_hidden() {
            //     return false
            // }
            let Act::Move { from } = &n.action.action else {
                unreachable!();
            };
            dbg!(from.ori.iter().count(), p.iter_offsets().count());
            result.push((p.clone(), from.ori.clone()));
            false
        },
    );

    dbg!(&result.len());

    result.into_iter().filter_map(move |(to, from)| {
        dbg!(&from);
        dbg!(from.iter().count());
        let (from_path, f_id) =
            hyper_ast::position::path_with_spaces(src_tr, &mut from.iter(), with_spaces_stores);
        dbg!(from_path.iter().count());
        let (from, _from) = hyper_ast::position::compute_position(
            src_tr,
            &mut from_path.iter().copied(),
            with_spaces_stores,
        );
        dbg!(f_id);

        dbg!(to.node());
        let t_t = hyper_ast::types::HyperAST::resolve_type(stores, &to.node());
        let tr = to.root();
        dbg!(&to);
        let t0 = to.iter_offsets().count();
        let to_path =
            hyper_ast::position::path_with_spaces(tr, &mut to.iter_offsets(), with_spaces_stores).0;
        let t1 = to_path.len();
        let (to, _to) = hyper_ast::position::compute_position(
            tr,
            &mut to_path.iter().copied(),
            with_spaces_stores,
        );
        dbg!(_to);

        let t_f = hyper_ast::types::HyperAST::resolve_type(stores, &f_id);
        dbg!(t0, t1);
        assert_eq!(t_f, t_t);

        let t_f = hyper_ast::types::HyperAST::resolve_type(stores, &_from);
        let t_t = hyper_ast::types::HyperAST::resolve_type(stores, &_to);
        if t_f != t_t {
            dbg!(t_f.as_static_str(), t_t.as_static_str());
            return None;
        }
        Some(((to, to_path), (from, from_path)))
    })
}

pub(crate) fn extract_moves2<'a>(
    with_spaces_stores: &'a hyper_ast::store::SimpleStores<hyper_ast_cvs_git::TStore>,
    stores: &'a Stores,
    src_tr: NodeIdentifier,
    dst_tr: NodeIdentifier,
    actions: &'a ActionsVec<A>,
) -> impl Iterator<Item = (Pos, Pos)> + 'a {
    actions.0.iter().filter_map(move |a| {
        let from = match &a.action {
            Act::Move { from } => from,
            Act::MovUpd { from, .. } => from,
            // Act::Insert { sub } => todo!(),
            _ => return None,
        };
        let (from_path, w) =
            hyper_ast::position::path_with_spaces(src_tr, &mut from.ori.iter(), with_spaces_stores);
        let t = hyper_ast::types::HyperAST::resolve_type(stores, &w);
        use hyper_ast::types::HyperType;
        if t.is_file() || t.is_directory() {
            return None;
        }
        // if t.is_hidden() || !t.is_named() || {
        //     dbg!(t.as_static_str());
        //     return None;
        // }
        // if t.as_static_str() != "method_declaration" && t.as_static_str() != "_method_header" {
        //     dbg!(t.as_static_str());
        //     return None;
        // }
        let (to_path, x) = hyper_ast::position::path_with_spaces(
            dst_tr,
            &mut a.path.ori.iter(),
            with_spaces_stores,
        );
        let (from, _from) = hyper_ast::position::compute_position(
            src_tr,
            &mut from_path.iter().copied(),
            with_spaces_stores,
        );
        assert_eq!(w, _from);

        let (to, _to) = hyper_ast::position::compute_position(
            dst_tr,
            &mut to_path.iter().copied(),
            with_spaces_stores,
        );
        // dbg!(_to);
        assert_eq!(x, _to);

        let t_f = hyper_ast::types::HyperAST::resolve_type(stores, &_from);
        let t_t = hyper_ast::types::HyperAST::resolve_type(stores, &_to);
        if t_f != t_t {
            dbg!(t_f.as_static_str(), t_t.as_static_str());
            return None;
        }
        // dbg!(t_f.as_static_str());
        Some(((to, to_path), (from, from_path)))
    })
}

pub(crate) fn extract_updates<'a>(
    with_spaces_stores: &'a hyper_ast::store::SimpleStores<hyper_ast_cvs_git::TStore>,
    stores: &'a Stores,
    _src_tr: NodeIdentifier,
    dst_tr: NodeIdentifier,
    actions: &'a ActionsVec<A>,
) -> impl Iterator<Item = (Pos, Pos)> + 'a {
    let mut result = vec![];
    let mut a_tree = ActionsTree::new();
    for a in actions.0.iter() {
        if let Act::Update { .. } = &a.action {
            a_tree.merge_ori(a);
        }
    }
    // eprintln!("{:?}", a_tree.inspect());
    use hyper_ast::types::HyperType;
    go_to_files(
        stores,
        &a_tree.atomics,
        hyper_ast::position::StructuralPosition::new(dst_tr),
        &mut |p, nn, n, id| {
            let t = stores.resolve_type(&id);
            dbg!(t.as_static_str(), p);
            result.push(p.clone());
            false
        },
    );

    dbg!(&result.len());

    result
        .into_iter()
        .map(|path| {
            let tr = path.root();
            let path = hyper_ast::position::path_with_spaces(
                tr,
                &mut path.iter_offsets(),
                with_spaces_stores,
            )
            .0;
            let (pos, _) = hyper_ast::position::compute_position(
                tr,
                &mut path.iter().copied(),
                with_spaces_stores,
            );
            (pos, path)
        })
        .map(|x| (x.clone(), x))
}

pub(crate) fn extract_inserts<'a>(
    with_spaces_stores: &'a hyper_ast::store::SimpleStores<hyper_ast_cvs_git::TStore>,
    stores: &'a Stores,
    _src_tr: NodeIdentifier,
    dst_tr: NodeIdentifier,
    actions: &'a ActionsVec<A>,
) -> impl Iterator<Item = (Pos, Pos)> + 'a {
    let mut result = vec![];
    let mut a_tree = ActionsTree::new();
    for a in actions.0.iter() {
        if let Act::Insert { .. } = &a.action {
            a_tree.merge_ori(a);
        }
    }
    // eprintln!("{:?}", a_tree.inspect());
    use hyper_ast::types::HyperType;
    go_to_files(
        stores,
        &a_tree.atomics,
        hyper_ast::position::StructuralPosition::new(dst_tr),
        &mut |p, nn, n, id| {
            let t = stores.resolve_type(&id);
            // dbg!(t.as_static_str(), p);
            result.push(p.clone());
            false
        },
    );

    dbg!(&result.len());

    result
        .into_iter()
        .map(|path| {
            let tr = path.root();
            let path = hyper_ast::position::path_with_spaces(
                tr,
                &mut path.iter_offsets(),
                with_spaces_stores,
            )
            .0;
            let (pos, _) = hyper_ast::position::compute_position(
                tr,
                &mut path.iter().copied(),
                with_spaces_stores,
            );
            (pos, path)
        })
        .map(|x| (x.clone(), x))
}

pub(crate) fn extract_deletes<'a>(
    with_spaces_stores: &'a hyper_ast::store::SimpleStores<hyper_ast_cvs_git::TStore>,
    stores: &'a Stores,
    src_tr: NodeIdentifier,
    _dst_tr: NodeIdentifier,
    actions: &'a ActionsVec<A>,
) -> impl Iterator<Item = (Pos, Pos)> + 'a {
    let mut result = vec![];
    let mut a_tree = ActionsTree::new();
    for a in actions.0.iter().rev() {
        if let Act::Delete { .. } = &a.action {
            a_tree.merge_ori(a);
        }
    }
    // eprintln!("{:?}", a_tree.inspect());
    use hyper_ast::types::HyperType;
    go_to_files(
        stores,
        &a_tree.atomics, // , &mapping
        hyper_ast::position::StructuralPosition::new(src_tr),
        &mut |p, nn, n, id| {
            let t = stores.resolve_type(&id);
            if !t.is_hidden() {
                result.push(p.clone());
            }
            false
            // let t = stores.resolve_type(id);
            // if t.as_static_str() == "try_statement" {
            //     dbg!(t.as_static_str(), p);
            //     result.push(p.clone());
            //     true
            // } else if t.as_static_str() == "import_declaration" {
            //     dbg!(t.as_static_str(), p);
            //     result.push(p.clone());
            //     true
            // } else {
            //     false
            // }
        },
    );

    dbg!(&result.len());

    result
        .into_iter()
        .map(|path| {
            let tr = path.root();
            let path = hyper_ast::position::path_with_spaces(
                tr,
                &mut path.iter_offsets(),
                with_spaces_stores,
            )
            .0;
            let (pos, _) = hyper_ast::position::compute_position(
                tr,
                &mut path.iter().copied(),
                with_spaces_stores,
            );
            (pos, path)
        })
        .map(|x| (x.clone(), x))
}

pub(crate) fn extract_focuses<'a>(
    with_spaces_stores: &'a hyper_ast::store::SimpleStores<hyper_ast_cvs_git::TStore>,
    stores: &'a Stores,
    src_tr: NodeIdentifier,
    _dst_tr: NodeIdentifier,
    actions: &'a ActionsVec<A>,
) -> impl Iterator<Item = (Pos, Pos)> + 'a {
    let mut result = vec![];
    let mut a_tree = ActionsTree::new();
    for a in actions.0.iter().rev() {
        if let Act::Delete { .. } = &a.action {
            a_tree.merge_ori(a);
        }
    }
    // eprintln!("{:?}", a_tree.inspect());
    use hyper_ast::types::HyperType;
    go_to_files(
        stores,
        &a_tree.atomics, // , &mapping
        hyper_ast::position::StructuralPosition::new(src_tr),
        &mut |p, nn, n, id| {
            let t = stores.resolve_type(&id);
            if t.as_static_str() == "try_statement" {
                // dbg!(t.as_static_str(), p);
                result.push(p.clone());
                true
            } else if t.as_static_str() == "import_declaration" {
                // dbg!(t.as_static_str(), p);
                result.push(p.clone());
                true
            } else {
                false
            }
        },
    );

    dbg!(&result.len());

    result
        .into_iter()
        .map(|path| {
            let tr = path.root();
            let path = hyper_ast::position::path_with_spaces(
                tr,
                &mut path.iter_offsets(),
                with_spaces_stores,
            )
            .0;
            let (pos, _) = hyper_ast::position::compute_position(
                tr,
                &mut path.iter().copied(),
                with_spaces_stores,
            );
            (pos, path)
        })
        .map(|x| (x.clone(), x))
}

pub(crate) type _R = hyper_ast::position::structural_pos::StructuralPosition<NodeIdentifier, u16>;

pub(crate) type Stores<'a> = hyper_ast::types::SimpleHyperAST<
    NoSpaceWrapper<'a, NodeIdentifier>,
    &'a hyper_ast_cvs_git::TStore,
    hyper_ast_cvs_git::no_space::NoSpaceNodeStoreWrapper<'a>,
    &'a LabelStore,
>;

pub(crate) type N = hyper_diff::actions::action_tree::Node<
    SimpleAction<LabelIdentifier, CompressedTreePath<Idx>, NodeIdentifier>,
>;

pub(crate) type P = hyper_ast::position::StructuralPosition;

pub(crate) fn go_to_files<F>(
    stores: &Stores,
    cs: &[N],
    // mapping: _,
    path: P,
    result: &mut F,
) where
    F: FnMut(&P, &NoSpaceWrapper<NodeIdentifier>, &N, NodeIdentifier) -> bool,
{
    't: for n in cs {
        // n.action;
        let mut path = path.clone();
        // use hyper_ast::types::TypeStore;
        // let t = stores.resolve_type(nn);
        // if t.is_file() {
        //     dbg!();
        //     continue;
        // }
        // dbg!(&n.action.path.ori);
        let mut p_it = n.action.path.ori.iter();
        loop {
            let Some(p) = p_it.next() else {
                break;
            };
            let id = path.node();
            let nn = stores.node_store.resolve(id);
            use hyper_ast::types::TypeStore;
            let t = stores.resolve_type(&id);
            use hyper_ast::types::HyperType;
            // dbg!(t.as_static_str());
            if t.is_file() {
                got_through(stores, n, path.clone(), p, p_it, 0, result);
                // got_through_file(stores, n, path.clone(), p, p_it, 0, result);
                continue 't;
            }
            use hyper_ast::types::WithChildren;
            let cs = nn.children().unwrap();
            let node = cs.get(p).unwrap();
            path.goto(*node, p);
        }

        go_to_files(stores, &n.children, path, result);
    }
}

pub(crate) fn got_through<F>(
    stores: &Stores,
    n: &N,
    mut path: hyper_ast::position::StructuralPosition,
    mut p: u16,
    mut p_it: impl std::iter::Iterator<Item = u16> + Clone,
    mut d: usize,
    result: &mut F,
) where
    F: FnMut(&P, &NoSpaceWrapper<NodeIdentifier>, &N, NodeIdentifier) -> bool,
{
    let mut id = path.node();
    let mut nn = stores.node_store.resolve(id);
    loop {
        if result(&path, &nn, &n, id) {
            return;
        }
        // if t.as_static_str() == "try_statement" {
        //     dbg!(d, t.as_static_str(), &path);
        //     result.push(path);
        //     return;
        // }
        use hyper_ast::types::WithChildren;
        let Some(cs) = nn.children() else {
            return; // NOTE should not happen
        };

        let Some(node) = cs.get(p) else {
            return; // NOTE should not happen
        };
        path.goto(*node, p);
        d += 1;

        id = *node;
        nn = stores.node_store.resolve(*node);

        let Some(_p) = p_it.next() else {
            break;
        };
        p = _p;
    }
    if result(&path, &nn, &n, id) {
        return;
    }
    for n in &n.children {
        let mut p_it = n.action.path.ori.iter();
        // always at least one element in an action path
        let p = p_it.next().unwrap();
        got_through(stores, n, path.clone(), p, p_it, d, result);
    }
}
