use std::{fmt::Display, fs::File, io::BufWriter, io::Write, path::PathBuf};

use crate::{
    other_tools,
    postprocess::{CompressedBfPostProcess, PathJsonPostProcess},
};
use hyper_diff::algorithms::{self, ComputeTime};
use hyperast::{types::WithStats, utils::memusage_linux};
use hyperast_vcs_git::{
    git::fetch_github_repository, no_space::as_nospaces2 as as_nospaces,
    preprocessed::PreProcessedRepository,
};
use num_traits::ToPrimitive;

pub fn windowed_commits_compare(
    window_size: usize,
    mut preprocessed: PreProcessedRepository,
    (before, after): (&str, &str),
    dir_path: &str,
    diff_algorithm: &str,
    out: Option<(PathBuf, PathBuf)>,
) {
    assert!(window_size > 1);

    let batch_id = format!("{}:({},{})", &preprocessed.name, before, after);
    let mu = memusage_linux();
    let processing_ordered_commits = preprocessed.pre_process_with_limit(
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
        processing_ordered_commits
    );
    let mut i = 0;
    let c_len = processing_ordered_commits.len();

    let mut buf = out
        .map(|out| (File::create(out.0).unwrap(), File::create(out.1).unwrap()))
        .map(|file| {
            (
                BufWriter::with_capacity(4 * 8 * 1024, file.0),
                BufWriter::with_capacity(4 * 8 * 1024, file.1),
            )
        });
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
    for c in (0..c_len - 1).map(|c| &processing_ordered_commits[c..(c + window_size).min(c_len)]) {
        let oid_src = c[0];
        for oid_dst in &c[1..] {
            log::warn!("diff of {oid_src} and {oid_dst}");

            let stores = &preprocessed.processor.main_stores;

            let commit_src = preprocessed.commits.get_key_value(&oid_src).unwrap();
            let src_tr = commit_src.1.ast_root;
            let src_s = stores.node_store.resolve(src_tr).size();
            dbg!(src_s, stores.node_store.resolve(src_tr).size_no_spaces());

            let commit_dst = preprocessed.commits.get_key_value(&oid_dst).unwrap();
            let dst_tr = commit_dst.1.ast_root;
            let dst_s = stores.node_store.resolve(dst_tr).size();
            dbg!(dst_s, stores.node_store.resolve(dst_tr).size_no_spaces());

            let hyperast = as_nospaces(stores);

            let mu = memusage_linux();
            let not_lazy = algorithms::gumtree::diff(&hyperast, &src_tr, &dst_tr);
            let not_lazy = not_lazy.summarize();
            dbg!(&not_lazy);
            let partial_lazy = algorithms::gumtree_partial_lazy::diff(&hyperast, &src_tr, &dst_tr);
            let partial_lazy = partial_lazy.summarize();
            dbg!(&partial_lazy);
            let lazy = algorithms::gumtree_lazy::diff(&hyperast, &src_tr, &dst_tr);
            let summarized_lazy = &lazy.summarize();
            dbg!(summarized_lazy);
            if summarized_lazy.compare_results(&not_lazy)
                || summarized_lazy.compare_results(&partial_lazy)
            {
                log::error!("there is an difference between the optimisations");
            }

            log::warn!("ed+mappings size: {}", memusage_linux() - mu);
            let total_lazy_t: f64 = summarized_lazy.time();
            dbg!(&total_lazy_t);

            let gt_out_format = "COMPRESSED"; // JSON
            let gt_out = other_tools::gumtree::subprocess(
                &hyperast,
                src_tr,
                dst_tr,
                "gumtree",
                diff_algorithm,
                (total_lazy_t * 10.).ceil().to_u64().unwrap(),
                gt_out_format,
            );
            let res = if gt_out_format == "COMPRESSED" {
                if let Some(gt_out) = &gt_out {
                    let pp = CompressedBfPostProcess::create(gt_out);
                    let (pp, counts) = pp.counts();
                    let (pp, gt_timings) = pp.performances();
                    let valid = pp.validity_mappings(&lazy.mapper);
                    Some((gt_timings, counts, valid))
                } else {
                    None
                }
            } else if gt_out_format == "JSON" {
                if let Some(gt_out) = &gt_out {
                    let pp = PathJsonPostProcess::new(&gt_out);
                    let gt_timings = pp.performances();
                    let counts = pp.counts();
                    let valid = pp.validity_mappings(&lazy.mapper);
                    Some((gt_timings, counts, valid.map(|x| x.len())))
                } else {
                    None
                }
            } else {
                unimplemented!("gt_out_format {} is not implemented", gt_out_format)
            };

            // let MappingDurations([subtree_matcher_t, bottomup_matcher_t]) =
            //     summarized_lazy.mapping_durations.clone().into();
            if let Some((buf_validity, buf_perfs)) = &mut buf {
                dbg!(
                    &src_s,
                    &dst_s,
                    Into::<isize>::into(&commit_src.1.memory_used()),
                    Into::<isize>::into(&commit_dst.1.memory_used()),
                    &summarized_lazy,
                    &not_lazy,
                    &partial_lazy,
                );
                if let Some((gt_timings, gt_counts, valid)) = res {
                    dbg!(&gt_counts, &valid, &gt_timings,);
                    writeln!(
                        buf_validity,
                        "{oid_src}/{oid_dst},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                        "java_gumtree",
                        "gumtree_lazy",
                        src_s,
                        dst_s,
                        gt_counts.mappings,
                        summarized_lazy.mappings,
                        valid.missing_mappings,
                        valid.additional_mappings,
                        gt_counts.actions,
                        summarized_lazy.actions.map_or(-1, |x| x as isize),
                        &gt_counts.src_heap,
                        &gt_counts.dst_heap,
                        Into::<isize>::into(&commit_src.1.memory_used()),
                        Into::<isize>::into(&commit_dst.1.memory_used()),
                        not_lazy.mappings,
                        not_lazy.actions.map_or(-1, |x| x as isize),
                        partial_lazy.mappings,
                        partial_lazy.actions.map_or(-1, |x| x as isize),
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
                        -1, //gt_counts.mappings,
                        summarized_lazy.mappings,
                        -1, //valid.missing_mappings,
                        -1, //valid.additional_mappings,
                        -1, //gt_counts.actions,
                        summarized_lazy.actions.map_or(-1, |x| x as isize),
                        -1, //&gt_counts.src_heap,
                        -1, //&gt_counts.dst_heap,
                        Into::<isize>::into(&commit_src.1.memory_used()),
                        Into::<isize>::into(&commit_dst.1.memory_used()),
                        not_lazy.mappings,
                        not_lazy.actions.map_or(-1, |x| x as isize),
                        partial_lazy.mappings,
                        partial_lazy.actions.map_or(-1, |x| x as isize),
                    )
                    .unwrap();
                }

                write_perfs(
                    buf_perfs,
                    "gumtree_lazy",
                    &oid_src,
                    oid_dst,
                    src_s,
                    dst_s,
                    summarized_lazy,
                )
                .unwrap();
                write_perfs(
                    buf_perfs,
                    "gumtree_not_lazy",
                    &oid_src,
                    oid_dst,
                    src_s,
                    dst_s,
                    &not_lazy,
                )
                .unwrap();
                write_perfs(
                    buf_perfs,
                    "gumtree_partial_lazy",
                    &oid_src,
                    oid_dst,
                    src_s,
                    dst_s,
                    &partial_lazy,
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
                        summarized_lazy.actions.map_or(-1,|x|x as isize),
                        gt_counts.actions,
                        valid.missing_mappings,
                        valid.additional_mappings,
                        &gt_timings[0],
                        &gt_timings[1],
                        &gt_timings[2],
                        summarized_lazy.mapping_durations.preparation[0],
                        summarized_lazy.mapping_durations.mappings.0[0],
                        summarized_lazy.mapping_durations.preparation[1],
                        summarized_lazy.mapping_durations.mappings.0[1],
                        summarized_lazy.gen_t,
                        summarized_lazy.prepare_gen_t,
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

pub(crate) fn write_perfs<Id: Display>(
    buf_perfs: &mut BufWriter<File>,
    kind: &str,
    oid_src: &Id,
    oid_dst: &Id,
    src_s: usize,
    dst_s: usize,
    summarized_lazy: &hyper_diff::algorithms::ResultsSummary<
        hyper_diff::algorithms::PreparedMappingDurations<2>,
    >,
) -> Result<(), std::io::Error> {
    writeln!(
        buf_perfs,
        "{}/{},{},{},{},{},{},{},{},{},{},{},{}",
        oid_src,
        oid_dst,
        kind,
        src_s,
        dst_s,
        summarized_lazy.mappings,
        summarized_lazy.actions.map_or(-1, |x| x as isize),
        summarized_lazy.mapping_durations.preparation[0],
        summarized_lazy.mapping_durations.mappings.0[0],
        summarized_lazy.mapping_durations.preparation[1],
        summarized_lazy.mapping_durations.mappings.0[1],
        summarized_lazy.prepare_gen_t,
        summarized_lazy.gen_t,
    )
}

#[cfg(test)]
mod test {

    use super::*;

    use hyper_diff::{
        decompressed_tree_store::{lazy_post_order::LazyPostOrder, CompletePostOrder},
        matchers::{
            heuristic::gt::greedy_subtree_matcher::{GreedySubtreeMatcher, SubtreeMatcher},
            mapping_store::{DefaultMultiMappingStore, VecStore},
            Decompressible,
        },
    };
    use hyperast::{
        store::nodes::legion::HashedNodeRef,
        types::{HyperASTShared, WithChildren},
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

        let processing_ordered_commits = preprocessed.pre_process_with_limit(
            &mut fetch_github_repository(&preprocessed.name),
            before,
            after,
            "spoon-pom",
            1000,
        );
        preprocessed.processor.purge_caches();
        let c_len = processing_ordered_commits.len();
        let c = (0..c_len - 1)
            .map(|c| &processing_ordered_commits[c..(c + window_size).min(c_len)])
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
        let stores = &preprocessed.processor.main_stores;
        let src = src_tr;
        let dst = dst_tr;
        let mappings = VecStore::default();
        type DS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;
        // type DS<'a> = CompletePostOrder<HashedNodeRef<'a>, u32>;
        let mapper = GreedySubtreeMatcher::<DS<_>, DS<_>, _, _>::matchh::<
            DefaultMultiMappingStore<_>,
        >(stores, src, dst, mappings);
        let SubtreeMatcher {
            src_arena,
            dst_arena,
            mappings,
            ..
        } = mapper.into();
        print_mappings(&dst_arena, &src_arena, &stores, &mappings);
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

        let processing_ordered_commits = preprocessed.pre_process_with_limit(
            &mut fetch_github_repository(&preprocessed.name),
            before,
            after,
            "spoon-pom",
            1000,
        );
        preprocessed.purge_caches();
        let c_len = processing_ordered_commits.len();
        let c = (0..c_len - 1)
            .map(|c| &processing_ordered_commits[c..(c + window_size).min(c_len)])
            .next()
            .unwrap();
        let oid_src = &c[0];
        let oid_dst = &c[1];
        let stores = &preprocessed.processor.main_stores;

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

        let src = src_tr;
        let dst = dst_tr;
        let mappings = VecStore::default();
        type DS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;
        let mapper = GreedySubtreeMatcher::<DS<_>, DS<_>, _, _>::matchh::<
            DefaultMultiMappingStore<_>,
        >(stores, src, dst, mappings);
        let SubtreeMatcher {
            src_arena,
            dst_arena,
            mappings,
            ..
        } = mapper.into();
        print_mappings(&dst_arena, &src_arena, &stores, &mappings);

        let gt_out_format = "JSON";
        let gt_out = other_tools::gumtree::subprocess(
            &preprocessed.processor.main_stores,
            src_tr,
            dst_tr,
            "gumtree-subtree",
            "Chawathe",
            60 * 5,
            gt_out_format,
        )
        .unwrap();

        let pp = SimpleJsonPostProcess::new(&gt_out);
        let gt_timings = pp.performances();
        let counts = pp.counts();
        dbg!(gt_timings, counts.mappings, counts.actions);
        let valid = pp._validity_mappings(
            &preprocessed.processor.main_stores,
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
            "a7f0adb2dd8449af6f9e9b5a25f11b5dcf5868f1",
            "29b9537e00d857c92378648ca7163ba0dc63da39",
        );
        // before 29b9537e00d857c92378648ca7163ba0dc63da39
        // after a7f0adb2dd8449af6f9e9b5a25f11b5dcf5868f1
        assert!(window_size > 1);

        let processing_ordered_commits = preprocessed.pre_process_with_limit(
            &mut fetch_github_repository(&preprocessed.name),
            before,
            after,
            "",
            3,
        );
        preprocessed.purge_caches();
        let c_len = processing_ordered_commits.len();
        assert!(c_len > 0);
        dbg!(&processing_ordered_commits);
        let c = (0..c_len - 1)
            .map(|c| &processing_ordered_commits[c..(c + window_size).min(c_len)])
            .next()
            .unwrap();
        let oid_src = &c[0];
        let oid_dst = &c[1];
        dbg!(oid_src, oid_dst);
        let stores = &preprocessed.processor.main_stores;

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

        let hyperast = as_nospaces(stores);
        let partial_lazy = algorithms::gumtree_partial_lazy::diff(&hyperast, &src_tr, &dst_tr);
        dbg!(
            &partial_lazy.mapping_durations,
            partial_lazy.prepare_gen_t,
            partial_lazy.gen_t
        );
    }
    #[test]
    fn issue_logging_log4j2_pom() {
        // cargo build --release && time target/release/window_combination apache/logging-log4j2 7e745b42bda9bf6f8ea681d38992d18036fc021e ebfc8945a5dd77b617f4667647ed4b740323acc8 "" batch2/validity_logging-log4j2.csv batch2/perfs_logging-log4j2.csv 2 Chawathe &> batch2/logging-log4j2.log
        // thread 'main' panicked at '114 55318 "reporting"', hyperast/src/tree_gen/mod.rs:414:13
        let preprocessed = PreProcessedRepository::new("apache/logging-log4j2");
        let window_size = 2;
        let mut preprocessed = preprocessed;
        let (before, after) = (
            "7e745b42bda9bf6f8ea681d38992d18036fc021e",
            "ebfc8945a5dd77b617f4667647ed4b740323acc8",
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

    #[test]
    fn issue_mappings_pomxml() {
        // INRIA/spoon 76ffd3353a535b0ce6edf0bf961a05236a40d3a1 74ee133f4fe25d8606e0775ade577cd8e8b5cbfd "spoon-pom" "" 2
        // hast, gt evolutions: 517,517,
        // missing, additional mappings: 43,10,
        // 1.089578603,2.667414915,1.76489064,1.59514709,2.984131976,35.289540009
        let preprocessed = PreProcessedRepository::new("INRIA/spoon");
        let window_size = 2;
        let mut preprocessed = preprocessed;
        let (before, after) = (
            "b5806e1f42e105c223e1c6659256a4a3a4538b6c",
            "568b4526d7af83de99c65bc64a55ddcb6b6d3488",
        );
        assert!(window_size > 1);

        let processing_ordered_commits = preprocessed.pre_process_with_limit(
            &mut fetch_github_repository(&preprocessed.name),
            before,
            after,
            "",
            2,
        );
        preprocessed.purge_caches();
        let c_len = processing_ordered_commits.len();
        let c = (0..c_len - 1)
            .map(|c| &processing_ordered_commits[c..(c + window_size).min(c_len)])
            .next()
            .unwrap();
        let oid_src = &c[0];
        let oid_dst = &c[1];
        let stores = &preprocessed.processor.main_stores;

        let commit_src = preprocessed.commits.get_key_value(&oid_src).unwrap();
        let src_tr = commit_src.1.ast_root;
        let src_tr = preprocessed.child_by_name(src_tr, "pom.xml").unwrap();
        // let src_tr = stores.node_store.resolve(src_tr).get_child(&0);
        dbg!(stores.node_store.resolve(src_tr).child_count());

        let commit_dst = preprocessed.commits.get_key_value(&oid_dst).unwrap();
        let dst_tr = commit_dst.1.ast_root;
        let dst_tr = preprocessed.child_by_name(dst_tr, "pom.xml").unwrap();
        // let dst_tr = stores.node_store.resolve(dst_tr).get_child(&0);

        let src = &src_tr;
        let dst = &dst_tr;
        let mappings = VecStore::default();
        use hyperast::types::DecompressedFrom;
        let mut _src_arena = Decompressible::<_, LazyPostOrder<_, u32>>::decompress(stores, src);
        let mut _dst_arena = Decompressible::<_, LazyPostOrder<_, u32>>::decompress(stores, dst);

        let src_arena = _src_arena.as_mut();
        let dst_arena = _dst_arena.as_mut();
        let mut mapper = hyper_diff::matchers::Mapper {
            hyperast: stores,
            mapping: hyper_diff::matchers::Mapping {
                src_arena,
                dst_arena,
                mappings,
            },
        };
        dbg!();
        use hyper_diff::decompressed_tree_store::ShallowDecompressedTreeStore;
        use hyper_diff::matchers::mapping_store::MappingStore;
        mapper.mapping.mappings.topit(
            mapper.mapping.src_arena.len(),
            mapper.mapping.dst_arena.len(),
        );
        dbg!();
        let mm = LazyGreedySubtreeMatcher::<_, _, _, VecStore<_>>::compute_multi_mapping::<
            DefaultMultiMappingStore<_>,
        >(&mut mapper);
        dbg!();
        use hyper_diff::matchers::heuristic::gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher;
        LazyGreedySubtreeMatcher::<_, _, _, VecStore<_>, 10>::filter_mappings(&mut mapper, &mm);
        // TODO do something with the multi mappings
        // modify filter_mappings to extract redundant mappings
        // the store it alongside other mappings
        dbg!();
        use hyper_diff::matchers::heuristic::gt::lazy2_greedy_bottom_up_matcher::GreedyBottomUpMatcher;
        GreedyBottomUpMatcher::<_, _, _, _, VecStore<_>, 1000, 1, 2>::execute(&mut mapper);
        // This one matches everingthing as it should but it is much slower
        // GreedyBottomUpMatcher::<_, _, _, _, VecStore<_>, 10_000, 1, 2>::execute(
        //     &mut mapper,
        //     &stores.label_store,
        // );
        dbg!();
        // type DS<'a> = CompletePostOrder<HashedNodeRef<'a>, u32>;
        // let mapper = GreedySubtreeMatcher::<DS, DS, _, _, _>::matchh::<DefaultMultiMappingStore<_>>(
        //     &stores.node_store,
        //     &src,
        //     &dst,
        //     mappings,
        // );
        let mappings = mapper.mapping.mappings;
        let dst_arena = _dst_arena.decomp.complete(stores);
        let src_arena = _src_arena.decomp.complete(stores);
        let src_arena = Decompressible {
            hyperast: stores,
            decomp: &src_arena,
        };
        let dst_arena = Decompressible {
            hyperast: stores,
            decomp: &dst_arena,
        };
        print_mappings(&dst_arena, &src_arena, &stores, &mappings);

        let gt_out_format = "JSON";
        let gt_out = other_tools::gumtree::subprocess(
            &preprocessed.processor.main_stores,
            src_tr,
            dst_tr,
            "gumtree-subtree",
            "Chawathe",
            60 * 5,
            gt_out_format,
        )
        .unwrap();

        let pp = SimpleJsonPostProcess::new(&gt_out);
        let gt_timings = pp.performances();
        let counts = pp.counts();
        dbg!(gt_timings, counts.mappings, counts.actions);
        let valid = pp._validity_mappings(
            &preprocessed.processor.main_stores,
            &src_arena,
            src_tr,
            &dst_arena,
            dst_tr,
            &mappings,
        );
        dbg!(valid.additional_mappings, valid.missing_mappings);
    }
}
