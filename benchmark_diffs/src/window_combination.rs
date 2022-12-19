use std::{fs::File, io::BufWriter, io::Write, path::PathBuf};

use hyper_ast::{
    store::{
        defaults::NodeIdentifier,
        labels::DefaultLabelIdentifier,
        nodes::legion::{HashedNodeRef, NodeStore},
    },
    types::{self, Children, MySlice, WithStats},
    utils::memusage_linux,
};
use hyper_ast_cvs_git::{git::fetch_github_repository, preprocessed::PreProcessedRepository};

use crate::{
    algorithms::{self, DiffResult},
    other_tools,
    postprocess::{CompressedBfPostProcess, PathJsonPostProcess},
};

use hyper_gumtree::actions::Actions;

pub fn windowed_commits_compare(
    window_size: usize,
    mut preprocessed: PreProcessedRepository,
    (before, after): (&str, &str),
    dir_path: &str,
    out: Option<PathBuf>,
) {
    assert!(window_size > 1);

    let batch_id = format!("{}:({},{})", &preprocessed.name, before, after);
    preprocessed.pre_process_with_limit(
        &mut fetch_github_repository(&preprocessed.name),
        before,
        after,
        dir_path,
        1000,
    );
    log::warn!("batch_id: {batch_id}");
    let mu = memusage_linux();
    log::warn!("total memory used {mu}");
    preprocessed.purge_caches();
    let mu = mu - memusage_linux();
    log::warn!("cache size: {mu}");
    log::warn!(
        "commits ({}): {:?}",
        preprocessed.commits.len(),
        preprocessed.processing_ordered_commits
    );
    let mut i = 0;
    let c_len = preprocessed.processing_ordered_commits.len();

    // let mappings_store = NodeStore::new();
    // let h = 0;
    // let insertion = mappings_store.prepare_insertion(&h, |a,b| 0==0);

    // let mappings: HashMap<(git::Oid,git::Oid),NodeIdentifier> = Default::default();
    let mut file = out.map(|out| File::create(out).unwrap());
    let (mut buf, out_to_file): (Box<dyn Write>, bool) = if let Some(ref mut file) = file {
        (Box::new(BufWriter::with_capacity(4 * 8 * 1024, file)), true)
    } else {
        (Box::new(std::io::stdout()), false)
    };
    for c in (0..c_len - 1)
        .map(|c| &preprocessed.processing_ordered_commits[c..(c + window_size).min(c_len)])
    {
        let oid_src = c[0];
        for oid_dst in &c[1..] {
            log::warn!("diff of {oid_src} and {oid_dst}");

            let stores = &preprocessed.main_stores;

            let commit_src = preprocessed.commits.get_key_value(&oid_src).unwrap();
            let src_tr = commit_src.1.ast_root;
            // let src_tr = preprocessed.child_by_name(src_tr, "hadoop-common-project").unwrap();
            let src_s = stores.node_store.resolve(src_tr).size();

            dbg!(src_s, stores.node_store.resolve(src_tr).size_no_spaces());

            let commit_dst = preprocessed.commits.get_key_value(&oid_dst).unwrap();
            let dst_tr = commit_dst.1.ast_root;
            // let dst_tr = preprocessed.child_by_name(dst_tr, "hadoop-common-project").unwrap();
            let dst_s = stores.node_store.resolve(dst_tr).size();
            dbg!(dst_s, stores.node_store.resolve(dst_tr).size_no_spaces());

            let label_store = &stores.label_store;
            let node_store = &stores.node_store;
            let node_store = &NoSpaceNodeStoreWrapper { s: node_store };

            let mu = memusage_linux();

            let DiffResult {
                mapping_durations: [subtree_matcher_t, bottomup_matcher_t],
                src_arena,
                dst_arena,
                mappings,
                actions,
                gen_t,
            } = algorithms::gumtree::diff(node_store, label_store, &src_tr, &dst_tr);
            let hast_actions = actions.len();
            log::warn!("ed+mappings size: {}", memusage_linux() - mu);

            let gt_out_format = "COMPRESSED"; //"COMPRESSED"; // JSON
            let gt_out = other_tools::gumtree::subprocess(
                node_store,
                label_store,
                src_tr,
                dst_tr,
                "gumtree",
                gt_out_format,
            );

            let timings = vec![subtree_matcher_t, bottomup_matcher_t, gen_t];

            dbg!(&timings);
            let res = if gt_out_format == "COMPRESSED" {
                let pp = CompressedBfPostProcess::create(&gt_out);
                let (pp, counts) = pp.counts();
                let (pp, gt_timings) = pp.performances();
                let valid = pp.validity_mappings(
                    node_store,
                    label_store,
                    &src_arena,
                    src_tr,
                    &dst_arena,
                    dst_tr,
                    &mappings,
                );
                Some((gt_timings, counts, valid))
            } else if gt_out_format == "JSON" {
                let pp = PathJsonPostProcess::new(&gt_out);
                let gt_timings = pp.performances();
                let counts = pp.counts();
                let valid = pp.validity_mappings(
                    node_store,
                    label_store,
                    &src_arena,
                    src_tr,
                    &dst_arena,
                    dst_tr,
                    &mappings,
                );
                // let pp = SimpleJsonPostProcess::new(&gt_out);
                // let gt_timings = pp.performances();
                // let counts = pp.counts();
                // let valid = pp.validity_mappings(
                //     node_store,
                //     label_store,
                //     &src_arena,
                //     src_tr,
                //     &dst_arena,
                //     dst_tr,
                //     &mappings,
                // );
                // dbg!(&valid.missing_mappings.iter().filter(|x|x.src.start<500).collect::<Vec<_>>());
                // dbg!(&valid.additional_mappings.iter().filter(|x|x.src.start<500).collect::<Vec<_>>());
                Some((gt_timings, counts, valid.map(|x| x.len())))
            } else {
                unimplemented!("gt_out_format {} is not implemented", gt_out_format)
            };
            if out_to_file {
                if let Some((gt_timings, gt_counts, valid)) = res {
                    dbg!(&gt_timings);
                    writeln!(
                        buf,
                        "{oid_src}/{oid_dst},{},{},{},{},{},{},{},{},{},{},{},{}",
                        src_s,
                        dst_s,
                        hast_actions,
                        gt_counts.actions,
                        valid.missing_mappings,
                        valid.additional_mappings,
                        &timings[0],
                        &timings[1],
                        &timings[2],
                        &gt_timings[0],
                        &gt_timings[1],
                        &gt_timings[2],
                    )
                    .unwrap();
                    buf.flush().unwrap();
                }
            } else {
                if let Some((gt_timings, gt_counts, valid)) = res {
                    dbg!(
                        &src_s,
                        &dst_s,
                        &hast_actions,
                        &gt_counts.actions,
                        &valid.missing_mappings,
                        &valid.additional_mappings,
                        &timings[0],
                        &timings[1],
                        &timings[2],
                        &gt_timings[0],
                        &gt_timings[1],
                        &gt_timings[2],
                    );
                    writeln!(
                        buf,
                        "{oid_src}/{oid_dst},{},{},{},{},{},{},{},{},{},{},{},{}",
                        src_s,
                        dst_s,
                        hast_actions,
                        gt_counts.actions,
                        valid.missing_mappings,
                        valid.additional_mappings,
                        &timings[0],
                        &timings[1],
                        &timings[2],
                        &gt_timings[0],
                        &gt_timings[1],
                        &gt_timings[2],
                    )
                    .unwrap()
                }
            }
        }
        log::warn!("done computing diff {i}");
        i += 1;
    }
    let mu = memusage_linux();
    drop(preprocessed);
    log::warn!("hyperAST size: {}", mu - memusage_linux());
}

#[cfg(test)]
mod test {

    use super::*;

    use hyper_ast::{store::nodes::legion::HashedNodeRef, types::WithChildren};
    use hyper_gumtree::{
        decompressed_tree_store::CompletePostOrder,
        matchers::{
            heuristic::gt::greedy_subtree_matcher::{GreedySubtreeMatcher, SubtreeMatcher},
            mapping_store::VecStore,
        },
    };

    use crate::postprocess::{print_mappings, SimpleJsonPostProcess};

    #[test]
    fn issue_mappings_pomxml_spoon_pom() {
        // INRIA/spoon 7c7f094bb22a350fa64289a94880cc3e7231468f 78d88752a9f4b5bc490f5e6fb0e31dc9c2cf4bcd "spoon-pom" "" 2
        let preprocessed = PreProcessedRepository::new("INRIA/spoon");
        let window_size = 2;
        let mut preprocessed = preprocessed;
        let (before, after) = (
            "7c7f094bb22a350fa64289a94880cc3e7231468f",
            "78d88752a9f4b5bc490f5e6fb0e31dc9c2cf4bcd",
        );
        assert!(window_size > 1);

        preprocessed.pre_process_with_limit(
            &mut fetch_github_repository(&preprocessed.name),
            before,
            after,
            "spoon-pom",
            1000,
        );
        preprocessed.purge_caches();
        let c_len = preprocessed.processing_ordered_commits.len();
        let c = (0..c_len - 1)
            .map(|c| &preprocessed.processing_ordered_commits[c..(c + window_size).min(c_len)])
            .next()
            .unwrap();
        let oid_src = &c[0];
        let oid_dst = &c[1];

        let commit_src = preprocessed.commits.get_key_value(&oid_src).unwrap();
        let src_tr = commit_src.1.ast_root;
        // let src_tr = preprocessed.child_by_name(src_tr, "hadoop-common-project").unwrap();

        let commit_dst = preprocessed.commits.get_key_value(&oid_dst).unwrap();
        let dst_tr = commit_dst.1.ast_root;
        // let dst_tr = preprocessed.child_by_name(dst_tr, "hadoop-common-project").unwrap();
        let stores = &preprocessed.main_stores;
        let src = &src_tr;
        let dst = &dst_tr;
        let mappings = VecStore::default();
        type DS<'a> = CompletePostOrder<HashedNodeRef<'a>, u32>;
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
        print_mappings(
            &dst_arena,
            &src_arena,
            &stores.node_store,
            &stores.label_store,
            &mappings,
        );
        // let subtree_matcher_t = now.elapsed().as_secs_f64();
        // let subtree_mappings_s = mappings.len();
        // dbg!(&subtree_matcher_t, &subtree_mappings_s);
        // let now = Instant::now();
        // let mut mapper = GreedyBottomUpMatcher::<DS, DS, _, HashedNodeRef, _, _, _>::new(
        //     &stores.node_store,
        //     &stores.label_store,
        //     src_arena,
        //     dst_arena,
        //     mappings,
        // );
        // dbg!(&now.elapsed().as_secs_f64());
        // mapper.execute();
        // dbg!(&now.elapsed().as_secs_f64());
        // let BottomUpMatcher {
        //     src_arena,
        //     dst_arena,
        //     mappings,
        //     ..
        // } = mapper.into();
        // dbg!(&now.elapsed().as_secs_f64());
        // let bottomup_matcher_t = now.elapsed().as_secs_f64();
        // let bottomup_mappings_s = mappings.len();
        // dbg!(&bottomup_matcher_t, &bottomup_mappings_s);
        // let now = Instant::now();
        // let dst_arena_bfs = SimpleBfsMapper::from(&stores.node_store, &dst_arena);
        // let script_gen =
        //     ScriptGenerator::<_, HashedNodeRef, _, _, NodeStore, _>::precompute_actions(
        //         &stores.node_store,
        //         &src_arena,
        //         &dst_arena_bfs,
        //         &mappings,
        //     )
        //     .generate();
        // let ScriptGenerator { actions, .. } = script_gen;
        // let gen_t = now.elapsed().as_secs_f64();
        // dbg!(gen_t);

        // let gt_out_format = "JSON"; //"COMPRESSED"; // JSON
        // let gt_out = other_tools::gumtree::subprocess(
        //     &preprocessed.main_stores,
        //     src_tr,
        //     dst_tr,
        //     "gumtree",
        //     gt_out_format,
        // );

        // let timings = vec![subtree_matcher_t, bottomup_matcher_t, gen_t];

        // dbg!(&timings);
        // let pp = SimpleJsonPostProcess::new(&gt_out);
        // let gt_timings = pp.performances();
        // let counts = pp.counts();
        // let valid = pp.validity_mappings(
        //     &preprocessed.main_stores,
        //     &src_arena,
        //     src_tr,
        //     &dst_arena,
        //     dst_tr,
        //     &mappings,
        // );
    }

    #[test]
    fn issue_mappings_pomxml_spoon_pom_2() {
        // INRIA/spoon 76ffd3353a535b0ce6edf0bf961a05236a40d3a1 74ee133f4fe25d8606e0775ade577cd8e8b5cbfd "spoon-pom" "" 2
        // hast, gt evolutions: 517,517,
        // missing, additional mappings: 43,10,
        // 1.089578603,2.667414915,1.76489064,1.59514709,2.984131976,35.289540009
        let preprocessed = PreProcessedRepository::new("INRIA/spoon");
        let window_size = 2;
        let mut preprocessed = preprocessed;
        let (before, after) = (
            "76ffd3353a535b0ce6edf0bf961a05236a40d3a1",
            "74ee133f4fe25d8606e0775ade577cd8e8b5cbfd",
        );
        assert!(window_size > 1);

        preprocessed.pre_process_with_limit(
            &mut fetch_github_repository(&preprocessed.name),
            before,
            after,
            "spoon-pom",
            1000,
        );
        preprocessed.purge_caches();
        let c_len = preprocessed.processing_ordered_commits.len();
        let c = (0..c_len - 1)
            .map(|c| &preprocessed.processing_ordered_commits[c..(c + window_size).min(c_len)])
            .next()
            .unwrap();
        let oid_src = &c[0];
        let oid_dst = &c[1];
        let stores = &preprocessed.main_stores;

        let commit_src = preprocessed.commits.get_key_value(&oid_src).unwrap();
        let src_tr = commit_src.1.ast_root;
        let src_tr = preprocessed.child_by_name(src_tr, "spoon-pom").unwrap();
        let src_tr = preprocessed.child_by_name(src_tr, "pom.xml").unwrap();
        // let src_tr = stores.node_store.resolve(src_tr).get_child(&0);
        dbg!(stores.node_store.resolve(src_tr).child_count());

        let commit_dst = preprocessed.commits.get_key_value(&oid_dst).unwrap();
        let dst_tr = commit_dst.1.ast_root;
        let dst_tr = preprocessed.child_by_name(dst_tr, "spoon-pom").unwrap();
        let dst_tr = preprocessed.child_by_name(dst_tr, "pom.xml").unwrap();
        // let dst_tr = stores.node_store.resolve(dst_tr).get_child(&0);

        let src = &src_tr;
        let dst = &dst_tr;
        let mappings = VecStore::default();
        type DS<'a> = CompletePostOrder<HashedNodeRef<'a>, u32>;
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
        print_mappings(
            &dst_arena,
            &src_arena,
            &stores.node_store,
            &stores.label_store,
            &mappings,
        );
        // let subtree_matcher_t = now.elapsed().as_secs_f64();
        // let subtree_mappings_s = mappings.len();
        // dbg!(&subtree_matcher_t, &subtree_mappings_s);
        // let now = Instant::now();
        // let mut mapper = GreedyBottomUpMatcher::<DS, DS, _, HashedNodeRef, _, _, _>::new(
        //     &stores.node_store,
        //     &stores.label_store,
        //     src_arena,
        //     dst_arena,
        //     mappings,
        // );
        // dbg!(&now.elapsed().as_secs_f64());
        // mapper.execute();
        // dbg!(&now.elapsed().as_secs_f64());
        // let BottomUpMatcher {
        //     src_arena,
        //     dst_arena,
        //     mappings,
        //     ..
        // } = mapper.into();
        // dbg!(&now.elapsed().as_secs_f64());
        // let bottomup_matcher_t = now.elapsed().as_secs_f64();
        // let bottomup_mappings_s = mappings.len();
        // dbg!(&bottomup_matcher_t, &bottomup_mappings_s);
        // let now = Instant::now();
        // let dst_arena_bfs = SimpleBfsMapper::from(&stores.node_store, &dst_arena);
        // let script_gen =
        //     ScriptGenerator::<_, HashedNodeRef, _, _, NodeStore, _>::precompute_actions(
        //         &stores.node_store,
        //         &src_arena,
        //         &dst_arena_bfs,
        //         &mappings,
        //     )
        //     .generate();
        // let ScriptGenerator { actions, .. } = script_gen;
        // let gen_t = now.elapsed().as_secs_f64();
        // dbg!(gen_t);

        let gt_out_format = "JSON"; //"COMPRESSED"; // JSON
        let gt_out = other_tools::gumtree::subprocess(
            &preprocessed.main_stores.node_store,
            &preprocessed.main_stores.label_store,
            src_tr,
            dst_tr,
            "gumtree-subtree",
            gt_out_format,
        );

        let pp = SimpleJsonPostProcess::new(&gt_out);
        let gt_timings = pp.performances();
        let counts = pp.counts();
        dbg!(gt_timings, counts.mappings, counts.actions);
        let valid = pp.validity_mappings(
            &preprocessed.main_stores.node_store,
            &preprocessed.main_stores.label_store,
            &src_arena,
            src_tr,
            &dst_arena,
            dst_tr,
            &mappings,
        );
        dbg!(valid.additional_mappings, valid.missing_mappings);
    }
}

pub(crate) struct NoSpaceNodeStoreWrapper<'a> {
    pub(crate) s: &'a NodeStore,
}

pub(crate) struct NoSpaceWrapper<'a> {
    inner: HashedNodeRef<'a>,
}

impl<'a> types::Typed for NoSpaceWrapper<'a> {
    type Type = types::Type;

    fn get_type(&self) -> types::Type {
        self.inner.get_type()
    }
}

impl<'a> types::WithStats for NoSpaceWrapper<'a> {
    fn size(&self) -> usize {
        self.inner.size_no_spaces()
    }

    fn height(&self) -> usize {
        self.inner.height()
    }
}

// impl<'a> types::WithSerialization for NoSpaceWrapper<'a> {
//     /// WARN return the len with spaces ?
//     fn try_bytes_len(&self) -> Option<usize> {
//         self.inner.try_bytes_len()
//     }
// }

impl<'a> types::Labeled for NoSpaceWrapper<'a> {
    type Label = DefaultLabelIdentifier;

    fn get_label(&self) -> &DefaultLabelIdentifier {
        self.inner.get_label()
    }
}

impl<'a> types::Node for NoSpaceWrapper<'a> {}

impl<'a> types::Stored for NoSpaceWrapper<'a> {
    type TreeId = NodeIdentifier;
}

// impl<'a> NoSpaceWrapper<'a> {
//     fn cs(&self) -> Option<&NoSpaceSlice<<Self as types::Stored>::TreeId>> {
//         self.inner.cs().map(|x|x.into()).ok()
//     }
// }

impl<'a> types::WithChildren for NoSpaceWrapper<'a> {
    type ChildIdx = u16;
    type Children<'b> = MySlice<Self::TreeId> where Self: 'b;

    fn child_count(&self) -> u16 {
        self.inner.no_spaces().map_or(0, |x| x.child_count())
    }

    fn child(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
        self.inner
            .no_spaces()
            .ok()
            .and_then(|x| x.get(*idx).copied())
    }

    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
        self.inner
            .no_spaces()
            .ok()
            .and_then(|x| x.rev(*idx).copied())
    }

    fn children(&self) -> Option<&Self::Children<'_>> {
        self.inner.no_spaces().ok()
    }
}

impl<'a> types::WithHashs for NoSpaceWrapper<'a> {
    type HK = hyper_ast::hashed::SyntaxNodeHashsKinds;
    type HP = hyper_ast::nodes::HashSize;

    fn hash(&self, kind: &Self::HK) -> Self::HP {
        self.inner.hash(kind)
    }
}

impl<'a> types::Tree for NoSpaceWrapper<'a> {
    fn has_children(&self) -> bool {
        self.inner.has_children()
    }

    fn has_label(&self) -> bool {
        self.inner.has_label()
    }

    fn try_get_label(&self) -> Option<&Self::Label> {
        self.inner.try_get_label()
    }
}

impl<'store> types::NodeStore<NodeIdentifier> for NoSpaceNodeStoreWrapper<'store> {
    type R<'a> = NoSpaceWrapper<'a> where Self: 'a;
    fn resolve(&self, id: &NodeIdentifier) -> Self::R<'_> {
        NoSpaceWrapper {
            inner: types::NodeStore::resolve(self.s, id),
        }
    }
}

// TODO materialize nodes type in the handle ie. NodeIdentier, 
// to allow filtering spaces in a slice,
// without having to access the node store.

// #[repr(transparent)]
// pub struct NoSpaceSlice<T>(pub [T]);

// impl<'a, T> From<&'a [T]> for &'a NoSpaceSlice<T> {
//     fn from(value: &'a [T]) -> Self {
//         unsafe { std::mem::transmute(value) }
//     }
// }

// impl<'a, T> From<&'a MySlice<T>> for &'a NoSpaceSlice<T> {
//     fn from(value: &'a MySlice<T>) -> Self {
//         unsafe { std::mem::transmute(value) }
//     }
// }

// impl<T> std::ops::Index<u16> for NoSpaceSlice<T> {
//     type Output = T;

//     fn index(&self, index: u16) -> &Self::Output {
//         &self.0[index as usize]
//     }
// }

// impl<T> std::ops::Index<u8> for NoSpaceSlice<T> {
//     type Output = T;

//     fn index(&self, index: u8) -> &Self::Output {
//         &self.0[index as usize]
//     }
// }

// impl<T> std::ops::Index<usize> for NoSpaceSlice<T> {
//     type Output = T;

//     fn index(&self, index: usize) -> &Self::Output {
//         &self.0[index]
//     }
// }

// impl<T: Clone> From<&NoSpaceSlice<T>> for Vec<T> {
//     fn from(value: &NoSpaceSlice<T>) -> Self {
//         value.0.to_vec()
//     }
// }

// impl<T: Debug> Debug for NoSpaceSlice<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         Debug::fmt(&self.0, f)
//     }
// }

// impl<T: Debug> Default for &NoSpaceSlice<T> {
//     fn default() -> Self {
//         let r: &[T] = &[];
//         r.into()
//     }
// }

// impl<T> IterableChildren<T> for NoSpaceSlice<T> {
//     type ChildrenIter<'a> = core::slice::Iter<'a, T> where T: 'a;

//     fn iter_children(&self) -> Self::ChildrenIter<'_> {
//         <[T]>::iter(&self.0)
//     }

//     fn is_empty(&self) -> bool {
//         <[T]>::is_empty(&self.0)
//     }
// }
// impl<'a> NoSpaceWrapper<'a> {
//     fn skip_spaces(&self) -> usize {
//         self.cs().map_or(0,|x| x.child_count())
//     }
// }

// impl<T> Children<u16, T> for NoSpaceSlice<T> {
//     fn child_count(&self) -> u16 {
//         <[T]>::len(&self.0).to_u16().unwrap()
//     }

//     fn get(&self, i: u16) -> Option<&T> {
//         self.0.get(usize::from(i))
//     }

//     fn rev(&self, idx: u16) -> Option<&T> {
//         let c: u16 = self.child_count();
//         let c = c.checked_sub(idx.checked_add(1)?)?;
//         self.get(c)
//     }

//     fn after(&self, i: u16) -> &Self {
//         (&self.0[i.into()..]).into()
//     }

//     fn before(&self, i: u16) -> &Self {
//         (&self.0[..i.into()]).into()
//     }

//     fn between(&self, start: u16, end: u16) -> &Self {
//         (&self.0[start.into()..end.into()]).into()
//     }

//     fn inclusive(&self, start: u16, end: u16) -> &Self {
//         (&self.0[start.into()..=end.into()]).into()
//     }
// }

// impl<T> Children<u8, T> for NoSpaceSlice<T> {
//     fn child_count(&self) -> u8 {
//         <[T]>::len(&self.0).to_u8().unwrap()
//     }

//     fn get(&self, i: u8) -> Option<&T> {
//         self.0.get(usize::from(i))
//     }

//     fn rev(&self, idx: u8) -> Option<&T> {
//         let c: u8 = self.child_count();
//         let c = c.checked_sub(idx.checked_add(1)?)?;
//         self.get(c)
//     }

//     fn after(&self, i: u8) -> &Self {
//         (&self.0[i.into()..]).into()
//     }

//     fn before(&self, i: u8) -> &Self {
//         (&self.0[..i.into()]).into()
//     }

//     fn between(&self, start: u8, end: u8) -> &Self {
//         (&self.0[start.into()..end.into()]).into()
//     }

//     fn inclusive(&self, start: u8, end: u8) -> &Self {
//         (&self.0[start.into()..=end.into()]).into()
//     }
// }
