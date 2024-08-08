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
            // invocation of the method "fail", surrounding string with r# makes that we don't have to escape the '"' in the string
            r#"(method_invocation
        (identifier) (#EQ? "fail")
    )"#,
    // a try block with a catch clause (does not match if there is no catch clause present)
            r#"(try_statement
        (block)
        (catch_clause)
    )"#,
            "(class_declaration)",
            "(method_declaration)",
    // an "@Test" annotation without parameters
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
        ).map_err(|x | x.to_string())
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

    println!(
        "commit_sha, ast_size, memory_used, processing_time, matches_count"
    );
    println!(
        "matches_links"
    );

    let mut old_matches_count = vec![];
    let mut old_matches_links = vec![];
    let mut first_round = true;

    for oid in oids {
        let commit = preprocessed.commits.get_key_value(&oid).unwrap();
        let time = commit.1.processing_time();
        let tr = commit.1.ast_root;
        use hyper_ast::types::WithStats;
        let s = stores.node_store.resolve(tr).size();

        let matches = hyper_ast_benchmark_smells::github_ranges::compute_ranges(stores, tr, &query);
        let matches_count: Vec<usize> = matches.clone()
            .into_iter()
            .map(|x| x.0)
            .collect();

        let matches_count_print = matches.clone()
            .into_iter()
            .map(|x| format!(",{}", x.0))
            .collect::<String>();

        let matches_links: Vec<String> = matches
            .into_iter()
            .map(|x| {
                let x: String = x.1
                    .into_iter()
                    .map(|x| format!("https://github.com/{repo_name}/blob/{oid}/{},", x))
                    .collect();
                x
                // format!(",[{:?}]", x)
            })
            .collect();

        // let matches_links_print = matches
        //     .into_iter()
        //     .map(|x| {
        //         let x: String = x.1
        //             .into_iter()
        //             .map(|x| format!("https://github.com/{repo_name}/blob/{oid}/{},", x))
        //             .collect();
        //         format!(",[{:?}]", x)
        //     })
        //     .collect::<String>();


        let mu = memusage_linux();
        // TODO
        if old_matches_count != matches_count {
            old_matches_count = matches_count;

            // FIXME: this does not actually work because the links contain the oid and are therefore never the same :) also the lines are different :)
            // TODO: ask Quentin how to identify which query matches disappeared over the versions
            let mut difference = vec![];
            if !first_round {
                    for i in old_matches_links {
                        if !matches_links.contains(&i) {
                            difference.push(i);
                        }
                    }
            }
            else {
                difference = matches_links.clone();
                first_round = false;
            }
            old_matches_links = matches_links;

            let print_difference = difference
                .into_iter()
                .map(|x| format!(",{}", x))
                .collect::<String>();

            println!(
                "{oid},{},{},{}{}",
                s,
                Into::<isize>::into(&commit.1.memory_used()),
                time,
                matches_count_print,
            );
            println!(
                "Removed occurrences: {}",
                print_difference,
            );
        }
    }
}

// !!! query is currently incorrect but it is running :)
#[test]
fn conditional_test_logic() {
    let repo_name =  "INRIA/spoon";
    let commit = "7c7f094bb22a350fa64289a94880cc3e7231468f";
    let limit = 2;
    let query = r#"(if_statement consequence: (_ 
    (_ (method_invocation 
         name: (identifier) (#EQ? "assertEquals") 
  ))
    ))"#;
    many(repo_name, commit, limit, query);
    eprintln!("conditional_test_logic done!")
}

#[test]
fn assertion_roulette() {
    let repo_name =  "INRIA/spoon";
    let commit = "7c7f094bb22a350fa64289a94880cc3e7231468f";
    let limit = 600;
    let query = hyper_ast_benchmark_smells::queries::assertion_roulette();
    print!("{}", query);
    many(repo_name, commit, limit, &query);
}
