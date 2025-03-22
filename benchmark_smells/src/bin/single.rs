use hyperast::utils::memusage_linux;
use hyperast_benchmark_smells::simple::count_matches;
use hyperast_vcs_git::preprocessed::PreProcessedRepository;

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

/// enables uses of [`hyperast::utils::memusage_linux()`]
#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
    use std::env;
    let args: Vec<String> = env::args().collect();
    log::warn!("args: {:?}", args);
    let repo_name = args
        .get(1)
        .expect("give an argument like openjdk/jdk or INRIA/spoon");
    let commit = args.get(2).map_or("", |x| x);
    let query = args.get(3).map_or("", |x| x);
    single(repo_name, commit, query);
}

const INCREMENTAL_QUERIES: bool = true;

fn single(repo_name: &str, commit: &str, query: &str) {

    let query = if INCREMENTAL_QUERIES {
        hyperast_tsquery::Query::with_precomputed(
            &query,
            hyperast_gen_ts_java::language(),
            hyperast_vcs_git::java_processor::SUB_QUERIES,
        ).unwrap().1
    } else {
        hyperast_tsquery::Query::new(&query, hyperast_gen_ts_java::language()).unwrap()
    };

    let mut preprocessed = PreProcessedRepository::new(&repo_name);
    let oid = preprocessed.pre_process_single(
        &mut hyperast_vcs_git::git::fetch_github_repository(&preprocessed.name),
        commit,
        "",
    );
    eprintln!("computing matches of {oid}");

    let stores = &preprocessed.processor.main_stores;

    let commit = preprocessed.commits.get_key_value(&oid).unwrap();
    let time = commit.1.processing_time();
    let tr = commit.1.ast_root;
    use hyperast::types::WithStats;
    let s = stores.node_store.resolve(tr).size();

   let matches = count_matches(stores, tr, &query);
   let matches = matches.into_iter().map(|x|format!(",{}",x)).collect::<String>();

    let mu = memusage_linux();
    // TODO
    log::warn!("ed+mappings size: {}", memusage_linux() - mu);
    log::warn!("done computing diff");
    println!(
        "{oid},{},{},{}{}",
        s,
        Into::<isize>::into(&commit.1.memory_used()),
        time,
        matches,
    );
}

#[test]
fn aaa() {
    println!("hello!")
}


#[test]
fn bbb() {
    println!("hello!   bbb")
}
