use std::collections::HashSet;

use hyper_ast::{position, utils::memusage_linux};
use hyper_ast_benchmark_smells::{github_ranges::format_pos_as_github_url, simple::count_matches};
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
const CSV_FORMATING: bool = false;

fn many(repo_name: &str, commit: &str, limit: usize, query: &str) {
    let query = if INCREMENTAL_QUERIES {
        hyper_ast_tsquery::Query::with_precomputed(
            &query,
            hyper_ast_gen_ts_java::language(),
            unsafe { hyper_ast_cvs_git::java_processor::SUB_QUERIES },
        )
        .map_err(|x| x.to_string())
        .unwrap()
        .1
    } else {
        hyper_ast_tsquery::Query::new(&query, hyper_ast_gen_ts_java::language()).unwrap()
    };

    assert!(query.enabled_pattern_count() > 0);

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

    if CSV_FORMATING {
        println!("commit_sha, ast_size, memory_used, processing_time, matches_count");
    } else {
        println!("matches_links");
    }

    let mut old_matches_count = vec![];
    let mut old_matches_positions: Vec<HashSet<(String, usize, usize)>> = vec![Default::default(); query.enabled_pattern_count()];
    let mut no_change_commits = vec![];
    for oid in oids {
        let commit = preprocessed.commits.get_key_value(&oid).unwrap();
        let time = commit.1.processing_time();
        let tr = commit.1.ast_root;
        use hyper_ast::types::WithStats;
        let s = stores.node_store.resolve(tr).size();

        let matches = hyper_ast_benchmark_smells::github_ranges::compute_ranges(stores, tr, &query);
        let matches_count: Vec<usize> = matches.iter().map(|x| x.len()).collect();

        let matches_count_print = matches_count
            .iter()
            .map(|x| format!(",{}", x))
            .collect::<String>();

        let matches_positions: Vec<HashSet<_>> = matches
            .iter()
            .map(|x| x.into_iter().cloned().collect())
            .collect();
        let matches_links: Vec<String> = matches
            .into_iter()
            .map(|x| {
                let x: String = x
                    .into_iter()
                    .map(|position| {
                        format!(
                            "{},",
                            format_pos_as_github_url(repo_name, &oid.to_string(), &position)
                        )
                    })
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
            eprintln!("following commits did not show a change: {:?}", no_change_commits);
            old_matches_count = matches_count.clone();

            let removed: Vec<HashSet<_>> = old_matches_positions
                .iter()
                .zip(matches_positions.iter())
                .map(|(old, new)| old.difference(new).collect())
                .collect();
            let added: Vec<HashSet<_>> = old_matches_positions
                .iter()
                .zip(matches_positions.iter())
                .map(|(old, new)| new.difference(old).collect())
                .collect();

            let print_removed = removed.into_iter().map(|x| {
                x.into_iter()
                    .map(|x| format_pos_as_github_url(repo_name, &oid.to_string(), &x))
            });

            let print_added = added.into_iter().map(|x| {
                x.into_iter()
                    .map(|x| format_pos_as_github_url(repo_name, &oid.to_string(), &x))
            });

            if CSV_FORMATING {
                let print_removed = print_removed
                    .into_iter()
                    .map(|x| x.map(|x| format!("{x},")).collect::<String>())
                    .map(|x| format!(",[{}]", x))
                    .collect::<String>();
                let print_added = print_added
                    .into_iter()
                    .map(|x| x.map(|x| format!("{x},")).collect::<String>())
                    .map(|x| format!(",[{}]", x))
                    .collect::<String>();

                println!(
                    "{oid},{},{},{}{}{}{}",
                    s,
                    Into::<isize>::into(&commit.1.memory_used()),
                    time,
                    matches_count_print,
                    print_removed,
                    print_added
                );
            } else {
                println!(
                    "{oid},{},{},{}:\n{}\n",
                    s,
                    Into::<isize>::into(&commit.1.memory_used()),
                    time,
                    matches_count_print,
                );

                for (i, ((added, removed), count)) in print_added.zip(print_removed).zip(matches_count.iter()).enumerate() {
                    println!("\tremoved ({i}): {}", removed.len());
                    for dif in removed {
                        println!("\t\t{}", dif)
                    }

                    println!("\tadded ({i}): {}", added.len());
                    for dif in added {
                        println!("\t\t{}", dif)
                    }
                }
            }
            // println!("Removed occurrences: {}", print_difference,);

            old_matches_positions = matches_positions;
            no_change_commits = vec![];
        } else {
            no_change_commits.push(oid.to_string());
        }
    }
    eprintln!("\nFollowing commits did not show a change: {:?}", no_change_commits);

    eprintln!("TODO summary")
}

// !!! query is currently incorrect but it is running :)
#[test]
fn conditional_test_logic() {
    let repo_name = "INRIA/spoon";
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
    let repo_name = "INRIA/spoon";
    let commit = "7c7f094bb22a350fa64289a94880cc3e7231468f";
    let limit = 6;
    let query = hyper_ast_benchmark_smells::queries::assertion_roulette();
    print!("{}", query);
    many(repo_name, commit, limit, &query);
}

#[test]
fn exeption_handling() {
    let repo_name = "INRIA/spoon";
    let commit = "7c7f094bb22a350fa64289a94880cc3e7231468f";
    let limit = 6;
    let query = hyper_ast_benchmark_smells::queries::exception_handling();
    let query = format!("{} @root", query);
    println!("{}", query);
    many(repo_name, commit, limit, &query);
}
