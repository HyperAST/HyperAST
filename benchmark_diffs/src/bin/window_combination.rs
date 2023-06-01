// window of one is just consecutive commits

use hyper_ast_cvs_git::preprocessed::PreProcessedRepository;
use std::{env, io::Write, path::PathBuf, str::FromStr};

use hyper_ast_benchmark_diffs::window_combination::windowed_commits_compare;

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn _main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format(|buf, record| {
            if record.level().to_level_filter() > log::LevelFilter::Debug {
                writeln!(buf, "{}", record.args())
            } else {
                writeln!(
                    buf,
                    "[{} {}] {}",
                    buf.timestamp_millis(),
                    record.level(),
                    record.args()
                )
            }
        })
        .init();
    let args: Vec<String> = env::args().collect();
    log::warn!("args: {:?}", args);
    let repo_name = args
        .get(1)
        .expect("give an argument like openjdk/jdk or INRIA/spoon"); //"openjdk/jdk";//"INRIA/spoon";
    let before = args.get(2).map_or("", |x| x);
    let after = args.get(3).map_or("", |x| x);
    let dir_path = args.get(4).map_or("", |x| x);
    let out_validity = args.get(5).and_then(|x| {
        if x.is_empty() {
            None
        } else {
            Some(PathBuf::from_str(x).unwrap())
        }
    });
    let out_perfs = args.get(6).and_then(|x| {
        if x.is_empty() {
            None
        } else {
            Some(PathBuf::from_str(x).unwrap())
        }
    });
    let out = out_validity.zip(out_perfs);
    let window_size = args.get(7).map_or(2, |x| usize::from_str(x).unwrap());
    let diff_algorithm = "Chawathe".to_string();
    let diff_algorithm = args.get(8).unwrap_or(&diff_algorithm);
    // concecutive_commits
    let preprocessed = PreProcessedRepository::new(&repo_name);
    windowed_commits_compare(window_size, preprocessed, (before, after), dir_path, diff_algorithm, out);
}

#[test]
fn concecutive_commits() {
    let preprocessed = PreProcessedRepository::new("repo_name");
    windowed_commits_compare(2, preprocessed, ("before", "after"), "", "Chawathe", None);
}

#[test]
fn issue_mappings_pomxml_spoon_pom() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format(|buf, record| {
            if record.level().to_level_filter() > log::LevelFilter::Debug {
                writeln!(buf, "{}", record.args())
            } else {
                writeln!(
                    buf,
                    "[{} {}] {}",
                    buf.timestamp_millis(),
                    record.level(),
                    record.args()
                )
            }
        })
        .init();
    // INRIA/spoon 7c7f094bb22a350fa64289a94880cc3e7231468f 78d88752a9f4b5bc490f5e6fb0e31dc9c2cf4bcd "spoon-pom" "" 2
    let preprocessed = PreProcessedRepository::new("INRIA/spoon");
    windowed_commits_compare(
        2,
        preprocessed,
        (
            "7c7f094bb22a350fa64289a94880cc3e7231468f",
            "78d88752a9f4b5bc490f5e6fb0e31dc9c2cf4bcd",
        ),
        "spoon-pom",
        "Chawathe",
        None,
    );
}

#[test]
fn issue_mappings_pomxml_spoon_pom_2() {
    // INRIA/spoon 76ffd3353a535b0ce6edf0bf961a05236a40d3a1 74ee133f4fe25d8606e0775ade577cd8e8b5cbfd "spoon-pom" "" 2
    // hast, gt evolutions: 517,517,
    // missing, additional mappings: 43,10,
    // 1.089578603,2.667414915,1.76489064,1.59514709,2.984131976,35.289540009
    let preprocessed = PreProcessedRepository::new("INRIA/spoon");
    windowed_commits_compare(
        2,
        preprocessed,
        (
            "76ffd3353a535b0ce6edf0bf961a05236a40d3a1",
            "74ee133f4fe25d8606e0775ade577cd8e8b5cbfd",
        ),
        "spoon-pom",
        "Chawathe",
        None,
    );
}

fn main() {
    let window_size = 2;
    let mut preprocessed = PreProcessedRepository::new("INRIA/spoon");
    let (before, after) = (
            // "e01840e86db739374c9c4eb84a948b24ca017d8e", // parent
            "cf626243f627cca3d52ff073ebc53eca9595d4b5", // git log --pretty=format:"%H" | head
            // # classic gumtree
            // only mappings // java -cp  gumtree.spoon.AstComparator /tmp/spoon  187.53s user 3.72s system 174% cpu 1:49.78 total
            // diff // java -cp  gumtree.spoon.AstComparator /tmp/spoon  198.09s user 2.94s system 183% cpu 1:49.42 total
            // # lazy
            // cargo run --bin=window_combination --release  59.67s user 0.34s system 99% cpu 1:00.45 total
            // cargo run --bin=window_combination --release  61.64s user 0.35s system 99% cpu 1:02.55 total


            "00dc4b0b13622dfeccb8d67757422c5bd1bf1e38",
        );
    assert!(window_size > 1);

    let batch_id = format!("{}:({},{})", &preprocessed.name, before, after);
    let mu = memusage_linux();
    let processing_ordered_commits = preprocessed.pre_process_with_limit(
        &mut hyper_ast_cvs_git::git::fetch_github_repository(&preprocessed.name),
        before,
        after,
        "",
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

    use hyper_ast_gen_ts_java::utils::memusage_linux;
    for c in (0..c_len - 1)
        .map(|c| &processing_ordered_commits[c..(c + window_size).min(c_len)])
    {
        let oid_src = c[0];
        for oid_dst in &c[1..] {
            log::warn!("diff of {oid_src} and {oid_dst}");

            let stores = &preprocessed.processor.main_stores;

            let commit_src = preprocessed.commits.get_key_value(&oid_src).unwrap();
            let src_tr = commit_src.1.ast_root;
            // let src_s = stores.node_store.resolve(src_tr).size();
            // dbg!(src_s, stores.node_store.resolve(src_tr).size_no_spaces());

            let commit_dst = preprocessed.commits.get_key_value(&oid_dst).unwrap();
            let dst_tr = commit_dst.1.ast_root;
            // let dst_s = stores.node_store.resolve(dst_tr).size();
            // dbg!(dst_s, stores.node_store.resolve(dst_tr).size_no_spaces());

            let hyperast = hyper_ast_cvs_git::no_space::as_nospaces(stores);

            let mu = memusage_linux();
            // let not_lazy = hyper_diff::algorithms::gumtree::diff(&hyperast, &src_tr, &dst_tr);
            // let not_lazy = not_lazy.summarize();
            // dbg!(&not_lazy);
            // let partial_lazy = hyper_diff::algorithms::gumtree_partial_lazy::diff(&hyperast, &src_tr, &dst_tr);
            // let partial_lazy = partial_lazy.summarize();
            // dbg!(&partial_lazy);
            let lazy = hyper_diff::algorithms::gumtree_lazy::diff(&hyperast, &src_tr, &dst_tr);
            let summarized_lazy = &lazy.summarize();
            dbg!(summarized_lazy);
            // if summarized_lazy.compare_results(&not_lazy) || summarized_lazy.compare_results(&partial_lazy) {
            //     log::error!("there is an difference between the optimisations");
            // }
            log::warn!("ed+mappings size: {}", memusage_linux() - mu);
        }
        log::warn!("done computing diff {i}");
        i += 1;
    }
    let mu = memusage_linux();
    drop(preprocessed);
    log::warn!("hyperAST size: {}", mu - memusage_linux());;
}
