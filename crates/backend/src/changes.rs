use hyperast_vcs_git::{processing::ConfiguredRepoTrait, SimpleStores};
use serde::{Deserialize, Serialize};

use std::fmt::Debug;

use hyperast::{
    store::defaults::NodeIdentifier,
    types::{HyperAST, HyperType, Childrn, TypeStore, WithChildren, WithStats},
};
use hyper_diff::{decompressed_tree_store::ShallowDecompressedTreeStore, matchers::{Decompressible, Mapper}};

use crate::{matching, no_space, utils::get_pair_simp};

#[derive(Deserialize, Serialize, Debug)]
pub struct SrcChanges {
    user: String,
    name: String,
    commit: String,
    /// Global position of deleted elements
    deletions: Vec<u32>, // TODO diff encode
}
#[derive(Deserialize, Serialize, Debug)]
pub struct DstChanges {
    user: String,
    name: String,
    commit: String,
    /// Global position of added elements
    additions: Vec<u32>, // TODO diff encode
}

pub(crate) fn added_deleted(
    state: std::sync::Arc<crate::AppState>,
    repo_handle: &impl ConfiguredRepoTrait<
        Config = hyperast_vcs_git::processing::ParametrizedCommitProcessorHandle,
    >,
    src_oid: hyperast_vcs_git::git::Oid,
    dst_oid: hyperast_vcs_git::git::Oid,
) -> Result<(SrcChanges, DstChanges), String> {
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
    let stores = &no_space::as_nospaces2(with_spaces_stores);

    if src_tr == dst_tr {
        return Ok((
            SrcChanges {
                user: repo_handle.spec().user().to_string(),
                name: repo_handle.spec().name().to_string(),
                commit: src_oid.to_string(),
                deletions: Default::default(),
            },
            DstChanges {
                user: repo_handle.spec().user().to_string(),
                name: repo_handle.spec().name().to_string(),
                commit: dst_oid.to_string(),
                additions: Default::default(),
            },
        ));
    }

    let pair = get_pair_simp(&state.partial_decomps, stores, &src_tr, &dst_tr);

    let mapped = {
        let mappings_cache = &state.mappings_alone;
        use hyper_diff::matchers::mapping_store::MappingStore;
        use hyper_diff::matchers::mapping_store::VecStore;

        // unsucceful attempt using a type specific Typestore to improve efficiency of diff
        // #[repr(u8)]
        // pub enum TStore {
        //     Maven = 0,
        //     Java = 1,
        //     Cpp = 2,
        // }

        // impl Default for TStore {
        //     fn default() -> Self {
        //         Self::Maven
        //     }
        // }

        // impl<'a> TypeStore<no_space::NoSpaceWrapper<'a, NodeIdentifier>> for &TStore {
        //     type Ty = hyperast_vcs_git::MultiType;
        //     const MASK: u16 = 0b1000_0000_0000_0000;

        //     fn resolve_type(&self, n: &no_space::NoSpaceWrapper<'a, NodeIdentifier>) -> Self::Ty {
        //         use hyperast::types::Typed;
        //         n.get_type()
        //     }

        //     fn resolve_lang(
        //         &self,
        //         n: &no_space::NoSpaceWrapper<'a, NodeIdentifier>,
        //     ) -> hyperast::types::LangWrapper<Self::Ty> {
        //         todo!()
        //     }

        //     type Marshaled = hyperast::types::TypeIndex;

        //     fn marshal_type(
        //         &self,
        //         n: &no_space::NoSpaceWrapper<'a, NodeIdentifier>,
        //     ) -> Self::Marshaled {
        //         todo!()
        //     }

        //     fn type_eq(
        //         &self,
        //         n: &no_space::NoSpaceWrapper<'a, NodeIdentifier>,
        //         m: &no_space::NoSpaceWrapper<'a, NodeIdentifier>,
        //     ) -> bool {
        //         n.as_ref()
        //             .get_component::<hyperast_gen_ts_cpp::types::Type>()
        //             == m.as_ref()
        //                 .get_component::<hyperast_gen_ts_cpp::types::Type>()
        //     }
        // }
        // let tstore2 = TStore::default();
        let hyperast = stores;
        // let hyperast = hyperast.change_type_store_ref(&tstore2);
        // let hyperast = &hyperast;
        use hyper_diff::matchers::Mapping;

        dbg!();
        match mappings_cache.entry((src_tr, dst_tr)) {
            dashmap::mapref::entry::Entry::Occupied(entry) => entry.into_ref().downgrade(),
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                // std::collections::hash_map::Entry::Vacant(entry) => {
                let mappings = VecStore::default();
                let (src_arena, dst_arena) = (pair.0.get_mut(), pair.1.get_mut());
                let src_arena = Decompressible{hyperast, decomp: src_arena};
                let dst_arena = Decompressible{hyperast, decomp: dst_arena};
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
                matching::full2(&mut mapper);
                let vec_store = mapper.mappings.clone();

                dbg!();
                entry
                    .insert((crate::MappingStage::Bottomup, vec_store))
                    .downgrade()
            }
        }
    };
    let unmapped_dst: Vec<_> = global_pos_with_spaces(
        &repositories.processor.main_stores,
        dst_tr,
        mapped.1.dst_to_src.iter().enumerate().filter_map(|(i, x)| {
            if *x == 0 {
                Some(i as u32)
            } else {
                None
            }
        }),
    );
    let unmapped_src: Vec<_> = global_pos_with_spaces(
        &repositories.processor.main_stores,
        src_tr,
        mapped.1.src_to_dst.iter().enumerate().filter_map(|(i, x)| {
            if *x == 0 {
                Some(i as u32)
            } else {
                None
            }
        }),
    );

    Ok((
        SrcChanges {
            user: repo_handle.spec().user().to_string(),
            name: repo_handle.spec().name().to_string(),
            commit: src_oid.to_string(),
            deletions: unmapped_src,
        },
        DstChanges {
            user: repo_handle.spec().user().to_string(),
            name: repo_handle.spec().name().to_string(),
            commit: dst_oid.to_string(),
            additions: unmapped_dst,
        },
    ))
}

// TODO try to move it in hyperast::position
/// no_spaces gives topolgical indexes, topologically ordered,
/// it maps onto a tree without spaces
pub fn global_pos_with_spaces<'store, It: Iterator<Item = u32>>(
    stores: &'store SimpleStores,
    root: NodeIdentifier,
    // increasing order
    mut no_spaces: It,
) -> Vec<It::Item> {
    #[derive(Debug)]
    struct Ele {
        id: NodeIdentifier,
        i_no_s: u32,
        i_w_s: u32,
        idx: usize,
        children: Vec<NodeIdentifier>,
        d1_no_s: u32,
    }
    let mut res = vec![];
    let mut stack = {
        let b = stores.node_store().resolve(root);
        let cs = b.children().unwrap();
        let children = cs.iter_children().collect();
        let i_no_s = b.size_no_spaces() as u32;
        let i_w_s = b.size() as u32;
        vec![Ele {
            id: root,
            i_no_s,
            i_w_s,
            idx: 0,
            children,
            d1_no_s: 0,
        }]
    };
    let mut index_with_spaces: u32 = 0;
    let mut index_no_spaces: u32 = 0;
    while let Some(curr_no_space) = no_spaces.next() {
        loop {
            // dbg!(stack.len());
            let mut ele = stack.pop().unwrap();
            // dbg!(
            //     curr_no_space,
            //     index_with_spaces,
            //     index_no_spaces,
            //     &ele
            // );
            // TODO add debug assertion about size_no_space and is_space being compatible
            assert!(index_no_spaces <= index_with_spaces);
            if curr_no_space < index_no_spaces {
                panic!()
            } else if curr_no_space < ele.i_no_s {
                // need to go down
                let Some(&id) = ele.children.get(ele.idx) else {
                    for x in ele.children {
                        let b = stores.node_store().resolve(x);
                        dbg!(stores.resolve_type(&x));
                        dbg!(b.size_no_spaces());
                    }
                    panic!()
                };
                let b = stores.node_store().resolve(id);
                if stores.resolve_type(&id).is_spaces() {
                    ele.idx += 1;
                    ele.d1_no_s += b.size_no_spaces() as u32;
                    stack.push(ele);
                    index_with_spaces += 1;
                    // dbg!(b.size_no_spaces());
                    continue;
                }
                let cs = b.children();
                let value = if let Some(cs) = cs {
                    // dbg!(b.size_no_spaces(), b.size());
                    Ele {
                        id,
                        children: cs.iter_children().collect(),
                        i_no_s: index_no_spaces + b.size_no_spaces() as u32 - 1,
                        i_w_s: index_with_spaces + b.size() as u32 - 1,
                        idx: 0,
                        d1_no_s: index_no_spaces,
                    }
                } else {
                    // dbg!();
                    Ele {
                        id,
                        children: vec![],
                        i_no_s: index_no_spaces,
                        i_w_s: index_with_spaces,
                        idx: 0,
                        d1_no_s: index_no_spaces,
                    }
                };
                ele.idx += 1;
                if ele.idx >= ele.children.len() {
                    // dbg!(ele.idx);
                }
                stack.push(ele);
                stack.push(value);
            } else if curr_no_space == ele.i_no_s {
                let b = stores.node_store().resolve(ele.id);
                if stores.resolve_type(&ele.id).is_spaces() {
                    panic!();
                }
                res.push(index_with_spaces);
                if let Some(e) = stack.last_mut() {
                    e.d1_no_s += b.size_no_spaces() as u32;
                }
                index_no_spaces = ele.i_no_s + 1;
                index_with_spaces = ele.i_w_s + 1;
                // dbg!();
                break;
            } else {
                // index_no_spaces + ele.size_no_s < curr_no_space
                // we can skip the current node
                // we already poped ele
                index_no_spaces = ele.i_no_s + 1;
                index_with_spaces = ele.i_w_s + 1;
                // dbg!();
                let b = stores.node_store().resolve(ele.id);
                if let Some(e) = stack.last_mut() {
                    e.d1_no_s += b.size_no_spaces() as u32;
                }
            }
        }
    }
    res
}
