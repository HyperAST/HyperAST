use hyper_ast::utils::memusage_linux;
use hyper_ast_cvs_git::preprocessed::PreProcessedRepository;

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

/// enables uses of [`hyper_ast::utils::memusage_linux()`]
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
    let limit = args.get(3).map_or(2, |x| x.parse().expect("a number"));
    let query = args.get(4).map_or("", |x| x);

    print_pos(repo_name, commit, limit, query);
}

const INCREMENTAL_QUERIES: bool = true;

fn print_pos(repo_name: &str, commit: &str, limit: usize, query: &str) {
    let query = if INCREMENTAL_QUERIES {
        hyper_ast_tsquery::Query::with_precomputed(
            &query,
            hyper_ast_gen_ts_java::language(),
            unsafe { hyper_ast_cvs_git::java_processor::SUB_QUERIES },
        )
        .map(|x| x.1)
    } else {
        hyper_ast_tsquery::Query::new(&query, hyper_ast_gen_ts_java::language())
    };

    let query = match query {
        Ok(query) => query,
        Err(err) => {
            eprintln!("{}", err);
            panic!("there is an error in the query");
        },
    };

    let mut preprocessed = PreProcessedRepository::new(&repo_name);
    let oids = preprocessed.pre_process_with_limit(
        &mut hyper_ast_cvs_git::git::fetch_github_repository(&preprocessed.name),
        "",
        commit,
        "",
        limit,
    );
    eprintln!("computing matches of {oids:?}");

    let stores = &preprocessed.processor.main_stores;

    for oid in oids {
        let commit = preprocessed.commits.get_key_value(&oid).unwrap();
        let time = commit.1.processing_time();
        let tr = commit.1.ast_root;
        use hyper_ast::types::WithStats;
        let s = stores.node_store.resolve(tr).size();

        let matches = hyper_ast_benchmark_smells::github_ranges::compute_ranges(stores, tr, &query);
        let matches = matches
            .into_iter()
            .map(|x| {
                let x: String = x
                    .into_iter()
                    .map(|x| format!("https://github.com/{repo_name}/blob/{oid}/{},", x))
                    .collect();
                format!(",[{:?}]", x)
            })
            .collect::<String>();

        let mu = memusage_linux();
        // TODO
        println!(
            "{oid},{},{},{}{}",
            s,
            Into::<isize>::into(&commit.1.memory_used()),
            time,
            matches,
        );
    }
}

#[test]
fn print_positions() {
    let repo_name = "INRIA/spoon";
    let commit = "56e12a0c0e0e69ea70863011b4f4ca3305e0542b";
    let limit = 3;
    let query = r#"(if_statement
        (block)
        (catch_clause)
    ) @root"#;

    print_pos(repo_name, commit, limit, query);
    let s = format!("aaa");
    print!("{}", s);
    dbg!(s);
}


#[test]
fn assertion_roulette() {
    let repo_name =  "INRIA/spoon";
    let commit = "7c7f094bb22a350fa64289a94880cc3e7231468f";
    let limit = 6;
    let query = hyper_ast_benchmark_smells::queries::assertion_roulette();
    print!("{}", query);
    print_pos(repo_name, commit, limit, &query);
}
