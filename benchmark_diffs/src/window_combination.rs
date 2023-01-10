use std::{fs::File, io::BufWriter, io::Write, path::PathBuf};

use hyper_ast::{
    store::{
        defaults::NodeIdentifier,
        labels::DefaultLabelIdentifier,
        nodes::legion::{HashedNodeRef, NodeStore},
    },
    types::{self, Children, MySlice, SimpleHyperAST, WithStats},
    utils::memusage_linux,
};
use hyper_ast_cvs_git::{git::fetch_github_repository, preprocessed::PreProcessedRepository};
use num_traits::ToPrimitive;

use crate::{
    algorithms::{self, DiffResult, MappingDurations},
    other_tools,
    postprocess::{CompressedBfPostProcess, PathJsonPostProcess},
};

use hyper_gumtree::{
    actions::Actions,
};

pub fn windowed_commits_compare(
    window_size: usize,
    mut preprocessed: PreProcessedRepository,
    (before, after): (&str, &str),
    dir_path: &str,
    diff_algorithm: &str,
    out: Option<(PathBuf,PathBuf)>,
) {
    assert!(window_size > 1);

    let batch_id = format!("{}:({},{})", &preprocessed.name, before, after);
    let mu = memusage_linux();
    preprocessed.pre_process_with_limit(
        &mut fetch_github_repository(&preprocessed.name),
        before,
        after,
        dir_path,
        1000,
    );
    let hyperast_size = memusage_linux() - mu;
    log::warn!("hyperAST size: {}", hyperast_size);
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
    // let mut file_validity = out.map(|out| File::create(out.0).unwrap());
    // let (mut buf_validity, out_to_file): (Box<dyn Write>, bool) = if let Some(ref mut file) = file_validity {
    //     (Box::new(BufWriter::with_capacity(4 * 8 * 1024, file)), true)
    // } else {
    //     (Box::new(std::io::stdout()), false)
    // };
    let mut buf = out
    .map(|out| (File::create(out.0).unwrap(),File::create(out.1).unwrap()))
    .map(|file|(BufWriter::with_capacity(4 * 8 * 1024, file.0),BufWriter::with_capacity(4 * 8 * 1024, file.1)));
    if let Some((buf_validity, buf_perfs)) = &mut buf {
        writeln!(
            buf_validity,
            "input,gt_tool,hast_tool,src_s,dst_s,gt_m,hast_m,missing_mappings,additional_mappings,gt_c,hast_c,gt_src_heap,gt_dst_heap,hast_src_heap,hast_dst_heap,not_lazy_m,not_lazy_c,partial_lazy_m,partial_lazy_c"
        )
        .unwrap();
        writeln!(
            buf_perfs,
            "input,kind,src_s,dst_s,mappings,actions,prepare_topdown_t,topdown_t,prepare_bottomup_t,bottomup_t,prepare_gen_t,gen_t",
        )
        .unwrap();
    }
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

            let hyperast = SimpleHyperAST {
                node_store,
                label_store,
                _phantom: std::marker::PhantomData,
            };
            struct ResultsShort<MD> {
                pub mapping_durations: MD,
                pub mappings: usize,
                pub actions: Option<usize>,
                pub prepare_gen_t: f64,
                pub gen_t: f64,
            }

            // impl<IdN, IdL, P, M:MonoMappingStore, MD>  From<DiffResult<IdN, IdL, P, Mapper<>, MD>> for ResultsShort<MD> {
            //     fn from(value: DiffResult<IdN, IdL, P, M, MD>) -> Self {
            //         Self {
            //             mapping_durations: value.mapping_durations,
            //             mapper: value.mapper.mapping.mappings.len(),
            //             actions: value.actions.len(),
            //             gen_t: value.gen_t }
            //     }
            // }
            use hyper_gumtree::matchers::mapping_store::MappingStore;
            let not_lazy = algorithms::gumtree::diff(&hyperast, &src_tr, &dst_tr);
            let not_lazy = ResultsShort {
                mapping_durations: not_lazy.mapping_durations,
                mappings: not_lazy.mapper.mapping.mappings.len(),
                actions: not_lazy.actions.map(|x|x.len()),
                prepare_gen_t: not_lazy.prepare_gen_t,
                gen_t: not_lazy.gen_t,
            };
            dbg!(
                &not_lazy.mapping_durations,
                not_lazy.prepare_gen_t,
                not_lazy.gen_t
            );
            let partial_lazy = algorithms::gumtree_partial_lazy::diff(&hyperast, &src_tr, &dst_tr);
            let partial_lazy = ResultsShort {
                mapping_durations: partial_lazy.mapping_durations,
                mappings: partial_lazy.mapper.mapping.mappings.len(),
                actions: partial_lazy.actions.map(|x|x.len()),
                prepare_gen_t: partial_lazy.prepare_gen_t,
                gen_t: partial_lazy.gen_t,
            };
            dbg!(
                &partial_lazy.mapping_durations,
                partial_lazy.prepare_gen_t,
                partial_lazy.gen_t
            );
            let DiffResult {
                mapping_durations,
                mapper,
                actions,
                prepare_gen_t,
                gen_t,
            } = algorithms::gumtree_lazy::diff(&hyperast, &src_tr, &dst_tr);
            dbg!(&mapping_durations, &prepare_gen_t, &gen_t);
            let mappings_len = mapper.mapping.mappings.len();
            let actions_len = actions.map(|x|x.len());
            // assert_eq!(not_lazy.mappings, mappings_len);
            // assert_eq!(not_lazy.actions, actions_len);
            // assert_eq!(partial_lazy.mappings, mappings_len);
            // assert_eq!(partial_lazy.actions, actions_len);
            if not_lazy.mappings != mappings_len || not_lazy.actions != actions_len
            || partial_lazy.mappings != mappings_len || partial_lazy.actions != actions_len {
                dbg!(not_lazy.mappings, not_lazy.actions, 
                    partial_lazy.mappings, partial_lazy.actions, 
                    mappings_len, actions_len);
            }
            // } = algorithms::gumtree::diff(node_store, label_store, &src_tr, &dst_tr);
            let mapping_preparation_duration = mapping_durations.preparation;
            
            let MappingDurations([subtree_matcher_t, bottomup_matcher_t]) =
                mapping_durations.into();

            log::warn!("ed+mappings size: {}", memusage_linux() - mu);
            let total_lazy_t: f64 = subtree_matcher_t + bottomup_matcher_t + prepare_gen_t + gen_t + mapping_preparation_duration.iter().sum::<f64>();
            dbg!(&total_lazy_t);

            let gt_out_format = "COMPRESSED"; //"COMPRESSED"; // JSON
            let gt_out = other_tools::gumtree::subprocess(
                node_store,
                label_store,
                src_tr,
                dst_tr,
                "gumtree",
                diff_algorithm,
                (total_lazy_t*10.).ceil().to_u64().unwrap(),
                gt_out_format,
            );

            // let timings = vec![subtree_matcher_t, bottomup_matcher_t, gen_t + prepare_gen_t];

            // dbg!(&timings);
            let res = if gt_out_format == "COMPRESSED" {
                if let Some(gt_out) = &gt_out {
                    let pp = CompressedBfPostProcess::create(gt_out);
                    let (pp, counts) = pp.counts();
                    let (pp, gt_timings) = pp.performances();
                    let valid = pp.validity_mappings(&mapper);
                    Some((gt_timings, counts, valid))
                } else {
                    None
                }
            } else if gt_out_format == "JSON" {
                if let Some(gt_out) = &gt_out {
                    let pp = PathJsonPostProcess::new(&gt_out);
                    let gt_timings = pp.performances();
                    let counts = pp.counts();
                    let valid = pp.validity_mappings(&mapper);
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
                    None
                }
            } else {
                unimplemented!("gt_out_format {} is not implemented", gt_out_format)
            };
            if let Some((buf_validity, buf_perfs)) = &mut buf {
                dbg!(
                    &src_s,
                    &dst_s,
                    Into::<isize>::into(&commit_src.1.memory_used()),
                    Into::<isize>::into(&commit_dst.1.memory_used()),
                    &actions_len,
                    mapping_preparation_duration[0],
                    subtree_matcher_t, 
                    mapping_preparation_duration[1],
                    bottomup_matcher_t, 
                    gen_t, 
                    prepare_gen_t,
                    not_lazy.mapping_durations.preparation[0],
                    not_lazy.mapping_durations.mappings.0[0],
                    not_lazy.mapping_durations.preparation[1],
                    not_lazy.mapping_durations.mappings.0[1],
                    not_lazy.prepare_gen_t,
                    not_lazy.gen_t,
                    partial_lazy.mapping_durations.preparation[0],
                    partial_lazy.mapping_durations.mappings.0[0],
                    partial_lazy.mapping_durations.preparation[1],
                    partial_lazy.mapping_durations.mappings.0[1],
                    partial_lazy.prepare_gen_t,
                    partial_lazy.gen_t,
                );
                if let Some((gt_timings, gt_counts, valid)) = res {
                    dbg!(
                        &gt_counts.src_heap,
                        &gt_counts.dst_heap,
                        &gt_counts.actions,
                        &gt_counts.mappings,
                        &valid.missing_mappings,
                        &valid.additional_mappings,
                        &gt_timings[0],
                        &gt_timings[1],
                        &gt_timings[2],
                    );
                    writeln!(
                        buf_validity,
                        "{oid_src}/{oid_dst},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                        "java_gumtree",
                        "gumtree_lazy",
                        src_s,
                        dst_s,
                        gt_counts.mappings,
                        mappings_len,
                        valid.missing_mappings,
                        valid.additional_mappings,
                        gt_counts.actions,
                        actions_len.map_or(-1,|x|x as isize),
                        &gt_counts.src_heap,
                        &gt_counts.dst_heap,
                        Into::<isize>::into(&commit_src.1.memory_used()),
                        Into::<isize>::into(&commit_dst.1.memory_used()),
                        not_lazy.mappings, not_lazy.actions.map_or(-1,|x|x as isize), 
                        partial_lazy.mappings, partial_lazy.actions.map_or(-1,|x|x as isize), 
                    )
                    .unwrap();
                    writeln!(
                        buf_perfs,
                        "{oid_src}/{oid_dst},{},{},{},{},{},{},{},{},{},{},{}",
                        "java_gumtree",
                        src_s,
                        dst_s,
                        gt_counts.mappings,
                        gt_counts.actions,
                        0.0,
                        &gt_timings[0],
                        0.0,
                        &gt_timings[1],
                        0.0,
                        &gt_timings[2],
                    )
                    .unwrap();
                } else {
                    writeln!(
                        buf_validity,
                        "{oid_src}/{oid_dst},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                        "java_gumtree",
                        "gumtree_lazy",
                        src_s,
                        dst_s,
                        -1,//gt_counts.mappings,
                        mappings_len,
                        -1,//valid.missing_mappings,
                        -1,//valid.additional_mappings,
                        actions_len.map_or(-1,|x|x as isize),
                        -1,//gt_counts.actions,
                        -1,//&gt_counts.src_heap,
                        -1,//&gt_counts.dst_heap,
                        Into::<isize>::into(&commit_src.1.memory_used()),
                        Into::<isize>::into(&commit_dst.1.memory_used()),
                        not_lazy.mappings, not_lazy.actions.map_or(-1,|x|x as isize), 
                        partial_lazy.mappings, partial_lazy.actions.map_or(-1,|x|x as isize), 
                    )
                    .unwrap();
                }
                writeln!(
                    buf_perfs,
                    "{oid_src}/{oid_dst},{},{},{},{},{},{},{},{},{},{},{}",
                    "gumtree_lazy",
                    src_s,
                    dst_s,
                    mappings_len,
                    actions_len.map_or(-1,|x|x as isize),
                    mapping_preparation_duration[0],
                    subtree_matcher_t, 
                    mapping_preparation_duration[1],
                    bottomup_matcher_t, 
                    prepare_gen_t,
                    gen_t,
                )
                .unwrap();
                writeln!(
                    buf_perfs,
                    "{oid_src}/{oid_dst},{},{},{},{},{},{},{},{},{},{},{}",
                    "gumtree_not_lazy",
                    src_s,
                    dst_s,
                    not_lazy.mappings,
                    not_lazy.actions.map_or(-1,|x|x as isize),
                    not_lazy.mapping_durations.preparation[0],
                    not_lazy.mapping_durations.mappings.0[0],
                    not_lazy.mapping_durations.preparation[1],
                    not_lazy.mapping_durations.mappings.0[1],
                    not_lazy.prepare_gen_t,
                    not_lazy.gen_t,
                )
                .unwrap();
                writeln!(
                    buf_perfs,
                    "{oid_src}/{oid_dst},{},{},{},{},{},{},{},{},{},{},{}",
                    "gumtree_partial_lazy",
                    src_s,
                    dst_s,
                    partial_lazy.mappings,
                    partial_lazy.actions.map_or(-1,|x|x as isize),
                    partial_lazy.mapping_durations.preparation[0],
                    partial_lazy.mapping_durations.mappings.0[0],
                    partial_lazy.mapping_durations.preparation[1],
                    partial_lazy.mapping_durations.mappings.0[1],
                    partial_lazy.prepare_gen_t,
                    partial_lazy.gen_t,
                )
                .unwrap();
                buf_validity.flush().unwrap();
                buf_perfs.flush().unwrap();
            } else {
                if let Some((gt_timings, gt_counts, valid)) = res {
                    dbg!(&gt_timings);
                    println!(
                        "{oid_src}/{oid_dst},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                        src_s,
                        dst_s,
                        Into::<isize>::into(&commit_src.1.memory_used()),
                        Into::<isize>::into(&commit_dst.1.memory_used()),
                        actions_len.map_or(-1,|x|x as isize),
                        gt_counts.actions,
                        valid.missing_mappings,
                        valid.additional_mappings,
                        &gt_timings[0],
                        &gt_timings[1],
                        &gt_timings[2],
                        mapping_preparation_duration[0],
                        subtree_matcher_t, 
                        mapping_preparation_duration[1],
                        bottomup_matcher_t, 
                        gen_t, 
                        prepare_gen_t,
                        not_lazy.mapping_durations.preparation[0],
                        not_lazy.mapping_durations.mappings.0[0],
                        not_lazy.mapping_durations.preparation[1],
                        not_lazy.mapping_durations.mappings.0[1],
                        not_lazy.prepare_gen_t,
                        not_lazy.gen_t,
                        partial_lazy.mapping_durations.preparation[0],
                        partial_lazy.mapping_durations.mappings.0[0],
                        partial_lazy.mapping_durations.preparation[1],
                        partial_lazy.mapping_durations.mappings.0[1],
                        partial_lazy.prepare_gen_t,
                        partial_lazy.gen_t,
                    );
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
            heuristic::gt::{greedy_subtree_matcher::{GreedySubtreeMatcher, SubtreeMatcher}},
            mapping_store::{DefaultMultiMappingStore, VecStore},
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
        let mapper = GreedySubtreeMatcher::<DS, DS, _, _, _>::matchh::<DefaultMultiMappingStore<_>>(
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
        let mapper = GreedySubtreeMatcher::<DS, DS, _, _, _>::matchh::<DefaultMultiMappingStore<_>>(
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
            "Chawathe",
            60*5,
            gt_out_format,
        ).unwrap();

        let pp = SimpleJsonPostProcess::new(&gt_out);
        let gt_timings = pp.performances();
        let counts = pp.counts();
        dbg!(gt_timings, counts.mappings, counts.actions);
        let valid = pp._validity_mappings(
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
    
    #[test]
    fn issue_lazy_spark() {
        // cargo build --release && time target/release/window_combination apache/spark 14211a19f53bd0f413396582c8970e3e0a74281d 885f4733c413bdbb110946361247fbbd19f6bba9 "" validity_spark.csv perfs_spark.csv 2 Chawathe &> spark.log
        // thread 'main' panicked at 'Entity(63568) Entity(63568)', /home/quentin/rusted_gumtree3/gumtree/src/decompressed_tree_store/lazy_post_order.rs:293:17
        let preprocessed = PreProcessedRepository::new("apache/spark");
        let window_size = 2;
        let mut preprocessed = preprocessed;
        let (before, after) = (
            "a7f0adb2dd8449af6f9e9b5a25f11b5dcf5868f1", "29b9537e00d857c92378648ca7163ba0dc63da39"
        );
        // before 29b9537e00d857c92378648ca7163ba0dc63da39
        // after a7f0adb2dd8449af6f9e9b5a25f11b5dcf5868f1
        assert!(window_size > 1);

        preprocessed.pre_process_with_limit(
            &mut fetch_github_repository(&preprocessed.name),
            before,
            after,
            "",
            3,
        );
        preprocessed.purge_caches();
        let c_len = preprocessed.processing_ordered_commits.len();
        assert!(c_len> 0);
        dbg!(&preprocessed.processing_ordered_commits);
        let c = (0..c_len - 1)
            .map(|c| &preprocessed.processing_ordered_commits[c..(c + window_size).min(c_len)])
            .next()
            .unwrap();
        let oid_src = &c[0];
        let oid_dst = &c[1];
        dbg!(oid_src, oid_dst);
        let stores = &preprocessed.main_stores;

        let commit_src = preprocessed.commits.get_key_value(&oid_src).unwrap();
        let src_tr = commit_src.1.ast_root;
        // let src_tr = preprocessed.child_by_name(src_tr, "spoon-pom").unwrap();
        // let src_tr = preprocessed.child_by_name(src_tr, "pom.xml").unwrap();
        // let src_tr = stores.node_store.resolve(src_tr).get_child(&0);
        dbg!(stores.node_store.resolve(src_tr).child_count());

        let commit_dst = preprocessed.commits.get_key_value(&oid_dst).unwrap();
        let dst_tr = commit_dst.1.ast_root;
        // let dst_tr = preprocessed.child_by_name(dst_tr, "spoon-pom").unwrap();
        // let dst_tr = preprocessed.child_by_name(dst_tr, "pom.xml").unwrap();
        // let dst_tr = stores.node_store.resolve(dst_tr).get_child(&0);

        let label_store = &stores.label_store;
        let node_store = &stores.node_store;
        let node_store = &NoSpaceNodeStoreWrapper { s: node_store };

        let hyperast = SimpleHyperAST {
            node_store,
            label_store,
            _phantom: std::marker::PhantomData,
        };
        struct ResultsShort<MD> {
            pub mapping_durations: MD,
            pub mappings: usize,
            pub actions: usize,
            pub prepare_gen_t: f64,
            pub gen_t: f64,
        }

        // impl<IdN, IdL, P, M:MonoMappingStore, MD>  From<DiffResult<IdN, IdL, P, Mapper<>, MD>> for ResultsShort<MD> {
        //     fn from(value: DiffResult<IdN, IdL, P, M, MD>) -> Self {
        //         Self {
        //             mapping_durations: value.mapping_durations,
        //             mapper: value.mapper.mapping.mappings.len(),
        //             actions: value.actions.len(),
        //             gen_t: value.gen_t }
        //     }
        // }
        use hyper_gumtree::matchers::mapping_store::MappingStore;
        // let not_lazy = algorithms::gumtree::diff(&hyperast, &src_tr, &dst_tr);
        // let not_lazy = ResultsShort {
        //     mapping_durations: not_lazy.mapping_durations,
        //     mappings: not_lazy.mapper.mapping.mappings.len(),
        //     actions: not_lazy.actions.unwrap().len(),
        //     prepare_gen_t: not_lazy.prepare_gen_t,
        //     gen_t: not_lazy.gen_t,
        // };
        // dbg!(
        //     &not_lazy.mapping_durations,
        //     not_lazy.prepare_gen_t,
        //     not_lazy.gen_t
        // );
        let partial_lazy = algorithms::gumtree_partial_lazy::diff(&hyperast, &src_tr, &dst_tr);
        let partial_lazy = ResultsShort {
            mapping_durations: partial_lazy.mapping_durations,
            mappings: partial_lazy.mapper.mapping.mappings.len(),
            actions: partial_lazy.actions.unwrap().len(),
            prepare_gen_t: partial_lazy.prepare_gen_t,
            gen_t: partial_lazy.gen_t,
        };
        dbg!(
            &partial_lazy.mapping_durations,
            partial_lazy.prepare_gen_t,
            partial_lazy.gen_t
        );
        // let gt_out_format = "COMPRESSED"; //"COMPRESSED"; // JSON
        // let gt_out = other_tools::gumtree::subprocess(
        //     &preprocessed.main_stores.node_store,
        //     &preprocessed.main_stores.label_store,
        //     src_tr,
        //     dst_tr,
        //     "gumtree-subtree",
        //     "Chawathe",
        //     60*5,
        //     gt_out_format,
        // ).unwrap();

        // let pp = SimpleJsonPostProcess::new(&gt_out);
        // let gt_timings = pp.performances();
        // let counts = pp.counts();
        // dbg!(gt_timings, counts.mappings, counts.actions);
        // let valid = pp._validity_mappings(
        //     &preprocessed.main_stores.node_store,
        //     &preprocessed.main_stores.label_store,
        //     &src_arena,
        //     src_tr,
        //     &dst_arena,
        //     dst_tr,
        //     &mappings,
        // );
        // dbg!(valid.additional_mappings, valid.missing_mappings);
    }
    #[test]
    fn issue_logging_log4j2_pom() {
        // cargo build --release && time target/release/window_combination apache/logging-log4j2 7e745b42bda9bf6f8ea681d38992d18036fc021e ebfc8945a5dd77b617f4667647ed4b740323acc8 "" batch2/validity_logging-log4j2.csv batch2/perfs_logging-log4j2.csv 2 Chawathe &> batch2/logging-log4j2.log
        // thread 'main' panicked at '114 55318 "reporting"', hyper_ast/src/tree_gen/mod.rs:414:13
        let preprocessed = PreProcessedRepository::new("apache/logging-log4j2");
        let window_size = 2;
        let mut preprocessed = preprocessed;
        let (before, after) = (
            "7e745b42bda9bf6f8ea681d38992d18036fc021e", "ebfc8945a5dd77b617f4667647ed4b740323acc8"
        );
        assert!(window_size > 1);

        preprocessed.pre_process_with_limit(
            &mut fetch_github_repository(&preprocessed.name),
            before,
            after,
            "log4j-osgi",
            3,
        );
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
