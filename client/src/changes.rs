use hyper_ast_cvs_git::SimpleStores;
use serde::{Deserialize, Serialize};

use std::fmt::Debug;

use hyper_ast::{
    store::defaults::NodeIdentifier,
    types::{HyperAST, HyperType, IterableChildren, TypeStore, WithChildren, WithStats},
};
use hyper_diff::{decompressed_tree_store::ShallowDecompressedTreeStore, matchers::Mapper};

use crate::{matching, no_space, utils::get_pair_simp};

#[derive(Deserialize, Serialize)]
pub struct SrcChanges {
    user: String,
    name: String,
    commit: String,
    /// Global position of deleted elements
    deletions: Vec<u32>, // TODO diff encode
}
#[derive(Deserialize, Serialize)]
pub struct DstChanges {
    user: String,
    name: String,
    commit: String,
    /// Global position of added elements
    additions: Vec<u32>, // TODO diff encode
}

pub(crate) fn added_deleted(
    state: std::sync::Arc<crate::AppState>,
    src_oid: hyper_ast_cvs_git::git::Oid,
    dst_oid: hyper_ast_cvs_git::git::Oid,
    user: &String,
    name: &String,
) -> Result<(SrcChanges, DstChanges), String> {
    let repositories = state.repositories.read().unwrap();
    let commit_src = repositories.commits.get_key_value(&src_oid).unwrap();
    let src_tr = commit_src.1.ast_root;
    let commit_dst = repositories.commits.get_key_value(&dst_oid).unwrap();
    let dst_tr = commit_dst.1.ast_root;
    let with_spaces_stores = &repositories.processor.main_stores;
    let stores = &no_space::as_nospaces(with_spaces_stores);

    if src_tr == dst_tr {
        return Ok((
            SrcChanges {
                user: user.to_string(),
                name: name.to_string(),
                commit: src_oid.to_string(),
                deletions: Default::default(),
            },
            DstChanges {
                user: user.to_string(),
                name: name.to_string(),
                commit: dst_oid.to_string(),
                additions: Default::default(),
            },
        ));
    }

    let pair = get_pair_simp(&state.partial_decomps, stores, &src_tr, &dst_tr);

    let mapped = {
        let mappings_cache = &state.mappings_alone;
        use hyper_ast::types::HyperAST;
        use hyper_diff::matchers::heuristic::gt::{
            lazy2_greedy_bottom_up_matcher::GreedyBottomUpMatcher,
            lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher,
        };
        use hyper_diff::matchers::mapping_store::DefaultMultiMappingStore;
        use hyper_diff::matchers::mapping_store::MappingStore;
        use hyper_diff::matchers::mapping_store::VecStore;
        let hyperast = stores;
        use hyper_diff::matchers::Mapping;

        dbg!();
        match mappings_cache.entry((src_tr, dst_tr)) {
            dashmap::mapref::entry::Entry::Occupied(entry) => {
                // std::collections::hash_map::Entry::Occupied(entry) => {
                // src_arena = decompress_src;
                // dst_arena = decompress_dst;

                // entry.into_mut()
                entry.into_ref().downgrade()
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                // std::collections::hash_map::Entry::Vacant(entry) => {
                let mappings = VecStore::default();
                dbg!();
                // let (src_arena, dst_arena) = pair.value_mut();
                dbg!();
                // let (src_arena, dst_arena) = (pair.0.value_mut(), pair.1.value_mut());
                let (src_arena, dst_arena) = (pair.0.get_mut(), pair.1.get_mut());
                // let src_arena = decompress_src.value_mut();
                // let dst_arena = decompress_dst.value_mut();
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

                let vec_store = matching::full2(hyperast, mapper);
                // GreedyBottomUpMatcher::<_, _, _, _, VecStore<_>>::execute(
                //     &mut mapper,
                //     hyperast.label_store(),
                // );
                // dbg!();

                // // src_arena = mapper.mapping.src_arena;
                // // dst_arena = mapper.mapping.dst_arena;
                // let vec_store = mapper.mapping.mappings;
                // drop(mapper.mapping.dst_arena);
                // drop(mapper.mapping.src_arena);

                dbg!();
                entry
                    .insert((crate::MappingStage::Bottomup, vec_store))
                    .downgrade()
            }
        }
    };

    pub fn global_pos_with_spaces<'store, It: Iterator<Item = u32>>(
        root: NodeIdentifier,
        // increasing order
        mut no_spaces: It,
        stores: &'store SimpleStores,
    ) -> Vec<It::Item> {
        let mut offset_with_spaces: u32 = 0;
        let mut offset_without_spaces: u32 = 0;
        // let mut x = root;
        let mut res = vec![];
        let (children, pos_no_s, pos_w_s) = {
            let b = stores.node_store().resolve(root);
            let cs = b.children();
            (
                cs.unwrap().iter_children().map(|x| *x).collect::<Vec<_>>(),
                offset_without_spaces + b.size_no_spaces() as u32,
                offset_with_spaces + b.size() as u32,
            )
        };
        #[derive(Debug)]
        struct Ele {
            id: NodeIdentifier,
            pos_no_s: u32,
            pos_w_s: u32,
            idx: usize,
            children: Vec<NodeIdentifier>,
        }
        let mut stack = vec![Ele {
            id: root,
            pos_no_s,
            pos_w_s,
            idx: 0,
            children,
        }];
        while let Some(curr_no_space) = no_spaces.next() {
            loop {
                // dbg!(stack.len());
                let mut ele = stack.pop().unwrap();
                // dbg!(
                //     curr_no_space,
                //     offset_with_spaces,
                //     offset_without_spaces,
                //     &ele
                // );
                assert!(offset_without_spaces <= offset_with_spaces);
                if curr_no_space < offset_without_spaces {
                    panic!()
                } else if curr_no_space < ele.pos_no_s {
                    // need to go down
                    let id = ele.children[ele.idx];
                    let b = stores.node_store().resolve(id);
                    if stores.type_store().resolve_type(&b).is_spaces() {
                        ele.idx += 1;
                        stack.push(ele);
                        offset_with_spaces += 1;
                        // dbg!();
                        continue;
                    }
                    let cs = b.children();
                    let value = if let Some(cs) = cs {
                        // dbg!(b.size_no_spaces(), b.size());
                        Ele {
                            id,
                            children: cs.iter_children().map(|x| *x).collect::<Vec<_>>(),
                            pos_no_s: offset_without_spaces + b.size_no_spaces() as u32 - 1,
                            pos_w_s: offset_with_spaces + b.size() as u32 - 1,
                            idx: 0,
                        }
                    // } else if curr_no_space == offset_without_spaces {
                    //     ele.idx += 1;
                    //     stack.push(ele);
                    //     res.push(offset_with_spaces);
                    //     offset_without_spaces += 1;
                    //     offset_with_spaces += 1;
                    //     break;
                    } else {
                        // ele.idx += 1;
                        // stack.push(ele);
                        // offset_without_spaces += 1;
                        // offset_with_spaces += 1;
                        // continue;
                        // dbg!();
                        Ele {
                            id,
                            children: vec![],
                            pos_no_s: offset_without_spaces,
                            pos_w_s: offset_with_spaces,
                            idx: 0,
                        }
                    };
                    ele.idx += 1;
                    stack.push(ele);
                    stack.push(value);
                } else if curr_no_space == ele.pos_no_s {
                    res.push(offset_with_spaces);
                    offset_without_spaces = ele.pos_no_s + 1;
                    offset_with_spaces = ele.pos_w_s + 1;
                    // dbg!();
                    break;
                } else {
                    // offset_without_spaces + ele.size_no_s < curr_no_space
                    // we can skip the current node
                    // we already poped ele
                    offset_without_spaces = ele.pos_no_s + 1;
                    offset_with_spaces = ele.pos_w_s + 1;
                    // dbg!();
                }
            }
        }
        res
    }

    // TODO remap mappings by counting spaces
    // let unmapped_dst: Vec<_> = mapped
    //     .1
    //     .dst_to_src
    //     .iter()
    //     .enumerate()
    //     .filter_map(|(i, x)| if *x == 0 { Some(i as u32) } else { None })
    //     .collect();
    let unmapped_dst: Vec<_> = global_pos_with_spaces(
        dst_tr,
        mapped.1.dst_to_src.iter().enumerate().filter_map(|(i, x)| {
            if *x == 0 {
                Some(i as u32)
            } else {
                None
            }
        }),
        &repositories.processor.main_stores,
    );
    let unmapped_src: Vec<_> = global_pos_with_spaces(
        src_tr,
        mapped.1.src_to_dst.iter().enumerate().filter_map(|(i, x)| {
            if *x == 0 {
                Some(i as u32)
            } else {
                None
            }
        }),
        &repositories.processor.main_stores,
    );
    // let unmapped_src: Vec<_> = mapped
    //     .1
    //     .src_to_dst
    //     .iter()
    //     .enumerate()
    //     .filter_map(|(i, x)| if *x == 0 { Some(i as u32) } else { None })
    //     .collect();

    // let (src_arena, dst_arena) = (pair.0.get_mut(), pair.1.get_mut());
    // unmapped_src.iter().for_each(|i| {
    //     let path = src_arena.path(&src_arena.root(), i);
    //     let ori = src_arena.original(i);
    //     let (pos, ent, _) = compute_position_with_no_spaces(
    //         src_tr,
    //         &mut path.iter().copied(),
    //         &stores.node_store,
    //         &stores.label_store,
    //     );
    //     dbg!(ori, ent);
    //     dbg!(pos);
    // });

    Ok((
        SrcChanges {
            user: user.to_string(),
            name: name.to_string(),
            commit: src_oid.to_string(),
            deletions: unmapped_src,
        },
        DstChanges {
            user: user.to_string(),
            name: name.to_string(),
            commit: dst_oid.to_string(),
            additions: unmapped_dst,
        },
    ))
}
