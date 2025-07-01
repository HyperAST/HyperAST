use std::{fs::File, io::BufWriter, io::Write, path::PathBuf};

use crate::window_combination::write_perfs;
use hyper_diff::algorithms::{self};
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
    limit: usize,
    out: Option<PathBuf>,
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

    let mut buf_perfs = BufWriter::with_capacity(4 * 8 * 1024, File::create(out.unwrap()).unwrap());
    writeln!(
        buf_perfs,
        "input,kind,src_s,dst_s,mappings,actions,prepare_topdown_t,topdown_t,prepare_bottomup_t,bottomup_t,prepare_gen_t,gen_t,topdown_m,bottomup_m",
    )
        .unwrap();

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

            let mu = memusage_linux();

            let max_sizes = [50, 100, 200, 500, 1000];

            for &max_size in &max_sizes {
                // Greedy
                let greedy =
                    algorithms::gumtree::diff(&hyperast, &src_tr, &dst_tr, max_size, 0.5f64);
                let summarized_greedy = greedy.summarize();
                dbg!(max_size, src_s, &summarized_greedy);

                // Hybrid
                let hybrid =
                    algorithms::gumtree_hybrid::diff_hybrid(&hyperast, &src_tr, &dst_tr, max_size);
                let summarized_hybrid = &hybrid.summarize();
                dbg!(max_size, src_s, &summarized_hybrid);

                // Greedy lazy
                let greedy_lazy =
                    algorithms::gumtree_lazy::diff(&hyperast, &src_tr, &dst_tr, max_size, 0.5f64);
                let summarized_greedy_lazy = greedy_lazy.summarize();
                dbg!(max_size, src_s, &summarized_greedy_lazy);

                // Hybrid lazy
                let hybrid_lazy = algorithms::gumtree_hybrid_lazy::diff_hybrid_lazy(
                    &hyperast, &src_tr, &dst_tr, max_size,
                );
                let summarized_hybrid_lazy = &hybrid_lazy.summarize();
                dbg!(max_size, src_s, &summarized_hybrid_lazy);

                // Check if lazy always gives same result for hybrid
                // assert_eq!(summarized_hybrid.actions.map_or(-1, |x| x as isize), summarized_hybrid_lazy.actions.map_or(-1, |x| x as isize));
                // assert_eq!(summarized_hybrid.mappings, summarized_hybrid_lazy.mappings);

                // dbg!(
                //     max_size,
                //     &src_s,
                //     &dst_s,
                //     Into::<isize>::into(&src_mem),
                //     Into::<isize>::into(&dst_mem),
                // );

                write_perfs(
                    &mut buf_perfs,
                    &format!("greedy_{}", max_size),
                    &oid_src,
                    &oid_dst,
                    src_s,
                    dst_s,
                    &summarized_greedy,
                )
                .unwrap();

                write_perfs(
                    &mut buf_perfs,
                    &format!("hybrid_{}", max_size),
                    &oid_src,
                    &oid_dst,
                    src_s,
                    dst_s,
                    &summarized_hybrid,
                )
                .unwrap();

                write_perfs(
                    &mut buf_perfs,
                    &format!("greedy_lazy_{}", max_size),
                    &oid_src,
                    &oid_dst,
                    src_s,
                    dst_s,
                    &summarized_greedy_lazy,
                )
                .unwrap();

                write_perfs(
                    &mut buf_perfs,
                    &format!("hybrid_lazy_{}", max_size),
                    &oid_src,
                    &oid_dst,
                    src_s,
                    dst_s,
                    &summarized_hybrid_lazy,
                )
                .unwrap();
            }

            // Simple (max_size = 0)
            let simple = algorithms::gumtree_hybrid::diff_hybrid(&hyperast, &src_tr, &dst_tr, 0);
            let summarized_simple = &simple.summarize();
            dbg!("simple", src_s, summarized_simple);

            log::warn!("ed+mappings size: {}", memusage_linux() - mu);

            // dbg!(
            //     &src_s,
            //     &dst_s,
            //     Into::<isize>::into(&src_mem),
            //     Into::<isize>::into(&dst_mem),
            //     &summarized_simple
            // );

            write_perfs(
                &mut buf_perfs,
                "simple",
                &oid_src,
                &oid_dst,
                src_s,
                dst_s,
                &summarized_simple,
            )
            .unwrap();
            buf_perfs.flush().unwrap();
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
