use std::{fs::File, io::BufWriter, io::Write, path::PathBuf};

use hyperast::{
    types::{LabelStore, WithStats},
    utils::memusage_linux,
};
use hyperast_vcs_git::{
    maven::MavenModuleAcc,
    maven_processor::MavenProcessorHolder,
    multi_preprocessed::PreProcessedRepositories,
    processing::{
        CacheHolding, ConfiguredRepoHandle2, ConfiguredRepoTrait, erased::ParametrizedCommitProc2,
    },
};
use num_traits::ToPrimitive;

use crate::{
    other_tools,
    postprocess::{CompressedBfPostProcess, PathJsonPostProcess},
    window_combination::write_perfs,
};
use hyper_diff::algorithms::{self, ComputeTime};

pub struct CommitCompareParameters<'a> {
    pub configured_repo: ConfiguredRepoHandle2,
    pub before: &'a str,
    pub after: &'a str,
    // pub dir_path: &'a str,
}

// WARN for now only works with Maven and Java
pub fn windowed_commits_compare(
    window_size: usize,
    mut preprocessed: PreProcessedRepositories,
    params: Vec<CommitCompareParameters>,
    diff_algorithm: &str,
    limit: usize,
    out: Option<(PathBuf, PathBuf)>,
) {
    assert!(window_size > 1);

    // let batch_id = format!("{}:({},{})", name, before, after);
    let mu = memusage_linux();
    let mut repo_names = vec![];
    let processing_ordered_commits: Vec<_> = params
        .into_iter()
        .map(|x| {
            println!("{}:({},{})", x.configured_repo.spec(), &x.before, &x.after);
            repo_names.push(x.configured_repo.spec().to_string());
            (
                preprocessed
                    .pre_process_with_limit(
                        &mut x.configured_repo.clone().fetch(),
                        x.before,
                        x.before,
                        // x.dir_path,
                        limit,
                    )
                    .unwrap(),
                x.configured_repo,
            )
        })
        .collect();
    let hyperast_size = memusage_linux() - mu;
    log::warn!("hyperAST size: {}", hyperast_size);
    // log::warn!("batch_id: {batch_id}");
    let mu = memusage_linux();
    log::warn!("total memory used {mu}");
    // preprocessed.purge_caches(); // WARN do not purge object_map
    let mu = mu - memusage_linux();
    log::warn!("cache size: {mu}");
    // log::warn!(
    //     "commits ({}): {:?}",
    //     preprocessed.commits.len(),
    //     processing_ordered_commits
    // );
    let mut loop_count = 0;

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
    let r_len = processing_ordered_commits.len();
    dbg!(&r_len);
    let min_len = processing_ordered_commits
        .iter()
        .map(|x| x.0.len())
        .min()
        .unwrap();
    dbg!(&min_len, 0..=min_len - window_size);
    for c in (0..min_len - window_size).map(|c| {
        processing_ordered_commits
            .iter()
            .map(|x| (&x.0[c..(c + window_size)], &x.1))
            .collect::<Vec<_>>()
    }) {
        // dbg!(&c, 1..min_len - window_size);
        let oid_src: Vec<_> = c.iter().map(|x| (x.0[0], x.1)).collect();
        for oid_dst in (1..window_size).map(|i| c.iter().map(|c| (c.0[i], c.1)).collect::<Vec<_>>())
        {
            log::warn!("diff of {oid_src:?} and {oid_dst:?}");
            assert_eq!(oid_src.len(), oid_dst.len());

            let mut src_acc = MavenModuleAcc::from("".to_string());
            let mut src_mem = hyperast::utils::Bytes::default(); // NOTE it does not consider the size of the root, but it is an implementation detail
            let mut src_s = 0;
            for (i, (oid_src, repo)) in oid_src.iter().enumerate() {
                let commit_src = preprocessed.get_commit(repo.config(), &oid_src).unwrap();
                let node_store = &preprocessed.processor.main_stores.node_store;
                let src_tr = commit_src.ast_root;
                let s = node_store.resolve(src_tr).size();
                src_s += s;
                dbg!(s, node_store.resolve(src_tr).size_no_spaces());
                src_mem += commit_src.memory_used();
                let oid = commit_src.tree_oid;
                let label_store = &mut preprocessed.processor.main_stores.label_store;
                src_acc.push_submodule(
                    label_store.get_or_insert(&*repo_names[i]),
                    preprocessed
                        .processor
                        .processing_systems
                        .get::<MavenProcessorHolder>()
                        .unwrap()
                        .with_parameters(repo.config.1)
                        .get_caches()
                        .object_map
                        .get(&oid)
                        .unwrap()
                        .clone(),
                )
            }

            let mut dst_acc = MavenModuleAcc::from("".to_string());
            let mut dst_mem = hyperast::utils::Bytes::default();
            let mut dst_s = 0;
            for (i, (oid_dst, repo)) in oid_dst.iter().enumerate() {
                let commit_dst = preprocessed.get_commit(repo.config(), &oid_dst).unwrap();
                let node_store = &preprocessed.processor.main_stores.node_store;
                let dst_tr = commit_dst.ast_root;
                let s = node_store.resolve(dst_tr).size();
                dst_s += s;
                dbg!(s, node_store.resolve(dst_tr).size_no_spaces());
                dst_mem += commit_dst.memory_used();
                let oid = commit_dst.tree_oid;
                let label_store = &mut preprocessed.processor.main_stores.label_store;
                dst_acc.push_submodule(
                    label_store.get_or_insert(&*repo_names[i]),
                    preprocessed
                        .processor
                        .processing_systems
                        .get::<MavenProcessorHolder>()
                        .unwrap()
                        .with_parameters(repo.config.1)
                        .get_caches()
                        .object_map
                        // .get::<Caches>().unwrap().object_map//object_map_maven
                        .get(&oid)
                        .unwrap()
                        .clone(),
                )
            }

            let stores = &mut preprocessed.processor.main_stores;
            let src_tr = PreProcessedRepositories::make(src_acc, stores).0;
            let dst_tr = PreProcessedRepositories::make(dst_acc, stores).0;

            let stores = &preprocessed.processor.main_stores;
            let hyperast = hyperast_vcs_git::no_space::as_nospaces2(stores);

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
            if !summarized_lazy.compare_results(&not_lazy)
                || !summarized_lazy.compare_results(&partial_lazy)
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
                (total_lazy_t * 20.).ceil().to_u64().unwrap(),
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
            let oid_src = oid_src
                .iter()
                .map(|x| x.0.to_string())
                .collect::<Vec<String>>()
                .join("+");
            let oid_dst = oid_dst
                .iter()
                .map(|x| x.0.to_string())
                .collect::<Vec<String>>()
                .join("+");

            if let Some((buf_validity, buf_perfs)) = &mut buf {
                dbg!(
                    &src_s,
                    &dst_s,
                    Into::<isize>::into(&src_mem),
                    Into::<isize>::into(&dst_mem),
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
                        Into::<isize>::into(&src_mem),
                        Into::<isize>::into(&dst_mem),
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
                        Into::<isize>::into(&src_mem),
                        Into::<isize>::into(&dst_mem),
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
                    &oid_dst,
                    src_s,
                    dst_s,
                    summarized_lazy,
                )
                .unwrap();
                write_perfs(
                    buf_perfs,
                    "gumtree_not_lazy",
                    &oid_src,
                    &oid_dst,
                    src_s,
                    dst_s,
                    &not_lazy,
                )
                .unwrap();
                write_perfs(
                    buf_perfs,
                    "gumtree_partial_lazy",
                    &oid_src,
                    &oid_dst,
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
                        Into::<isize>::into(&src_mem),
                        Into::<isize>::into(&dst_mem),
                        summarized_lazy.actions.map_or(-1, |x| x as isize),
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
        log::warn!("done computing diff {loop_count}");
        loop_count += 1;
    }
    let mu = memusage_linux();
    drop(preprocessed);
    log::warn!("hyperAST size: {}", mu - memusage_linux());
}

#[test]
fn test() {}
