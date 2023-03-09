use std::{fmt::Display, rc::Rc, ops::DerefMut};

use axum::{body::HttpBody, Json};
use hyper_ast::{
    position::{compute_position, compute_range, resolve_range, Position},
    types::{LabelStore, WithChildren},
};
use hyper_ast_cvs_git::{
    git::{fetch_github_repository, read_position},
    preprocessed::{child_at_path, child_at_path_tracked},
};
use hyper_diff::{
    decompressed_tree_store::{DecompressedWithParent, ShallowDecompressedTreeStore},
    matchers::{mapping_store::MonoMappingStore, Mapper},
};
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use crate::SharedState;

#[derive(Deserialize, Clone, Debug)]
pub struct TrackingParam {
    user: String,
    name: String,
    commit: String,
    file: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TrackingQuery {
    start: Option<usize>,
    end: Option<usize>,
}

#[derive(Deserialize, Serialize)]
pub struct TrackingResult {
    pub compute_time: f64,
    src: PieceOfCode,
    matched: Vec<PieceOfCode>,
}

// impl Display for TrackingResult {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         writeln!()
//     }
// }

#[derive(Deserialize, Serialize)]
pub struct PieceOfCode {
    user: String,
    name: String,
    commit: String,
    file: String,
    start: usize,
    end: usize,
}

// impl Display for PieceOfCode {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         todo!()
//     }
// }

pub fn track_code(
    state: SharedState,
    path: TrackingParam,
    query: TrackingQuery,
) -> Result<Json<TrackingResult>, String> {
    let now = Instant::now();
    let TrackingParam {
        user,
        name,
        commit,
        file,
    } = path.clone();
    let TrackingQuery { start, end } = query.clone();
    let mut repo = fetch_github_repository(&format!("{}/{}", user, name));
    log::warn!("done cloning {user}/{name}");
    let mut get_mut = state.write().unwrap();
    let state = get_mut.deref_mut();
    let commits = state
        .repositories
        .pre_process_with_limit(&mut repo, "", &commit, "", 2)
        .map_err(|e| e.to_string())?;
    log::warn!("done construction of {commits:?} in {user}/{name}");
    let commit_src = state
        .repositories
        .commits
        .get_key_value(&commits[0])
        .unwrap();
    let src_tr = commit_src.1.ast_root;
    let commit_dst = state
        .repositories
        .commits
        .get_key_value(&commits[1])
        .unwrap();
    let dst_tr = commit_dst.1.ast_root;
    let node_store = &state.repositories.processor.main_stores.node_store;

    // let size = node_store.resolve(src_tr).size();
    log::error!("searching for {file}");
    let file_node = child_at_path_tracked(
        &state.repositories.processor.main_stores,
        src_tr,
        file.split("/"),
    );

    let Some((file_node, offsets_to_file)) = file_node else {
        return Err("not found".to_string());
    };

    dbg!(&offsets_to_file);

    let stores = &state.repositories.processor.main_stores;
    let mut path = vec![];
    let (node, offsets_in_file) = resolve_range(file_node, start.unwrap_or(0), end, stores);
    path.extend(offsets_to_file.iter().map(|x| *x as u16));
    dbg!(&node);
    dbg!(&offsets_in_file);
    let aaa = node_store.resolve(file_node);
    dbg!(aaa.try_get_bytes_len(0));
    path.extend(offsets_in_file.iter().map(|x| *x as u16));
    let (start, end, node) = compute_range(file_node, &mut offsets_in_file.into_iter(), stores);
    dbg!(start, end);
    dbg!(&node);

    // persists mappings, could also easily persist diffs,
    // but some compression on mappins could help
    // such as, not storing the decompression arenas
    // or encoding mappings more efficiently considering that most slices could simply by represented as ranges (ie. mapped identical subtrees)
    let mapper = {
        let repos = &state.repositories;
        let aaa = state.mappings.entry((src_tr, dst_tr)).or_insert_with(|| {
            let aaa = hyper_diff::algorithms::gumtree_lazy::diff(
                &repos.processor.main_stores,
                &src_tr,
                &dst_tr,
            )
            .mapper
            .persist();
            aaa
        });
        unsafe { Mapper::unpersist(&repos.processor.main_stores, &*aaa) }
    };
    let aaa = mapper
        .src_arena
        .child(node_store, &mapper.src_arena.root(), &path);

    let bbb = mapper.mappings.get_dst(&aaa);

    dbg!(bbb);

    let mut matched = vec![];
    if let Some(bbb) = bbb {
        let path = mapper.dst_arena.path(&mapper.dst_arena.root(), &bbb);
        let (pos, node) = compute_position(
            dst_tr,
            &mut path.into_iter(),
            &stores.node_store,
            &stores.label_store,
        );
        dbg!(&pos);
        dbg!(&node);
        let range = pos.range();
        matched.push(PieceOfCode {
            user: user.clone(),
            name: name.clone(),
            commit: commit_dst.0.to_string(),
            file: pos.file().to_str().unwrap().to_string(),
            start: range.start,
            end: range.end,
        })
    }

    Ok(TrackingResult {
        compute_time: now.elapsed().as_secs_f64(),
        src: PieceOfCode {
            user,
            name,
            commit,
            file,
            start,
            end,
        },
        matched,
    }
    .into())
}
