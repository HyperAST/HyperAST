use hyper_ast::utils::memusage_linux;
use hyper_ast_benchmark_smells::simple::count_matches;
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

    // WARN not to be mutated is some places, here is fine, change it at will
    // NOTE there is a upper limit for the number of usable subqueries
    unsafe {
        hyper_ast_cvs_git::java_processor::SUB_QUERIES = &[
            r#"(method_invocation
        (identifier) (#EQ? "fail")
    )"#,
            r#"(try_statement
        (block)
        (catch_clause)
    )"#,
            "(class_declaration)",
            "(method_declaration)",
            r#"(marker_annotation 
        name: (identifier) (#EQ? "Test")
    )"#,
            "(constructor_declaration)",
        ]
    };

    many(repo_name, commit, limit, query);
}

const INCREMENTAL_QUERIES: bool = true;

fn many(repo_name: &str, commit: &str, limit: usize, query: &str) {
    let query = if INCREMENTAL_QUERIES {
        hyper_ast_tsquery::Query::with_precomputed(
            &query,
            hyper_ast_gen_ts_java::language(),
            unsafe { hyper_ast_cvs_git::java_processor::SUB_QUERIES },
        )
        .unwrap()
        .1
    } else {
        hyper_ast_tsquery::Query::new(&query, hyper_ast_gen_ts_java::language()).unwrap()
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

        let matches = count_matches(stores, tr, &query);
        let matches = matches
            .into_iter()
            .map(|x| format!(",{}", x))
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
