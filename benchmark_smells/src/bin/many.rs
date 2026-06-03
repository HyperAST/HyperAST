// #![allow(unused)]
use std::collections::HashSet;

use hyperast_benchmark_smells::github_ranges::{
    PositionWithContext, format_pos_as_github_diff_url, format_pos_as_github_url,
};
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
    let limit = args.get(3).map_or(2, |x| x.parse().expect("a number"));
    let query = args.get(4).map_or("", |x| x);

    let subs = [
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
    .as_slice();

    many(repo_name, commit, limit, query, subs);
}

const INCREMENTAL_QUERIES: bool = true;
const CSV_FORMATING: bool = false;

fn many(
    repo_name: &str,
    commit: &str,
    limit: usize,
    query: &str,
    subs: impl hyperast_tsquery::ArrayStr,
) {
    let query = if INCREMENTAL_QUERIES {
        hyperast_tsquery::Query::with_precomputed(&query, hyperast_gen_ts_java::language(), subs)
            .map_err(|x| x.to_string())
            .unwrap()
            .1
    } else {
        hyperast_tsquery::Query::new(&query, hyperast_gen_ts_java::language()).unwrap()
    };

    assert!(query.enabled_pattern_count() > 0);

    let mut preprocessed = PreProcessedRepository::new(&repo_name);
    let oids = preprocessed.pre_process_first_parents_with_limit(
        &mut hyperast_vcs_git::git::fetch_github_repository(&preprocessed.name),
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
    let mut old_matches_positions: Vec<HashSet<_>> =
        vec![Default::default(); query.enabled_pattern_count()];
    let mut no_change_commits = vec![];
    for oid in oids {
        let commit = preprocessed.commits.get_key_value(&oid).unwrap();
        let time = commit.1.processing_time();
        let tr = commit.1.ast_root;
        use hyperast::types::WithStats;
        let s = stores.node_store.resolve(tr).size();

        let matches = hyperast_benchmark_smells::github_ranges::compute_postions_with_context(
            stores, tr, &query,
        );
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
                            format_pos_as_github_url(
                                repo_name,
                                &oid.to_string(),
                                &(position.file, position.start, position.end)
                            )
                        )
                    })
                    .collect();
                x
                // format!(",[{:?}]", x)
            })
            .collect();
        eprintln!("{:?}", matches_links);

        // TODO
        if old_matches_count != matches_count {
            eprintln!(
                "following commits did not show a change: {:?}",
                no_change_commits
            );
            old_matches_count = matches_count.clone();

            let (removed, added): (Vec<_>, Vec<_>) = old_matches_positions
                .into_iter()
                .zip(matches_positions.clone().into_iter())
                .map(|(old, new)| {
                    let (old, new) = track_heuristic1_bis(old, new);
                    let (old, new) = track_heuristic2(old, new);
                    (old, new)
                })
                .unzip();

            let print_removed = removed.into_iter().map(|x| {
                x.into_iter()
                    .map(|x| format_pos_as_github_diff_url(repo_name, &oid.to_string(), &x))
            });

            let print_added = added.into_iter().map(|x| {
                x.into_iter()
                    .map(|x| format_pos_as_github_diff_url(repo_name, &oid.to_string(), &x))
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

                for (i, ((added, removed), _count)) in print_added
                    .zip(print_removed)
                    .zip(matches_count.iter())
                    .enumerate()
                {
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
    eprintln!(
        "\nFollowing commits did not show a change: {:?}",
        no_change_commits
    );

    eprintln!("TODO summary")
}

fn track_heuristic1_bis<T: std::hash::Hash + Eq>(
    mut old_a: HashSet<T>,
    old_b: HashSet<T>,
) -> (HashSet<T>, Vec<T>) {
    // (old.difference(new).collect(), new.difference(old).collect()) // NOTE same
    let mut new_b = Vec::new();
    for x in old_b.into_iter() {
        if old_a.contains(&x) {
            old_a.remove(&x);
            new_b.push(x); // no copying here
        }
    }
    (old_a, new_b)
}

fn track_heuristic2(
    mut old_a: HashSet<PositionWithContext>,
    old_b: Vec<PositionWithContext>,
) -> (HashSet<PositionWithContext>, Vec<PositionWithContext>) {
    // (old.difference(new).collect(), new.difference(old).collect()) // NOTE same
    let mut new_b = Vec::new();
    for b in old_b.into_iter() {
        if let Some(i) = old_a.iter().find(|a| a.id == b.id && &a.file == &b.file) {
            old_a.remove(&i.clone());
            new_b.push(b); // no copying here
        }
    }
    (old_a, new_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyperast_benchmark_smells::DATASET;
    macro_rules! select_data {
        (name = $name:expr) => {
            (DATASET.iter().find(|x| x.0 == $name))
                .expect("the entry corresponding to provided name")
        };
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
        many(repo_name, commit, limit, query, [].as_slice());
        eprintln!("conditional_test_logic done!")
    }

    #[test]
    fn assertion_roulette_dubbo() {
        let data = select_data!(name = "dubbo");
        let repo_name = data.1;
        eprintln!("{}:", repo_name);
        let commit = data.2;
        let limit = 2000;
        let query = hyperast_benchmark_smells::queries::assertion_roulette();
        eprint!("{}", query);
        let subs = [
            r#"(method_invocation
                name: (identifier) (#EQ? "assertThat")
            )"#,
            "(class_declaration)",
            "(method_declaration)",
            r#"(marker_annotation
        name: (identifier) (#EQ? "Test")
    )"#,
        ]
        .as_slice();
        many(repo_name, commit, limit, &query, subs);
    }

    #[test]
    fn exception_handling() {
        let repo_name = "dubbo/dubbo";
        let commit = "7c7f094bb22a350fa64289a94880cc3e7231468f";
        let limit = 2000;
        let query = hyperast_benchmark_smells::queries::exception_handling();
        let query = format!("{} @root\n{} @root", query[0], query[1]);
        println!("{}:", repo_name);
        println!("{}", query);
        many(repo_name, commit, limit, &query, [].as_slice());
    }

    #[test]
    fn exception_handling_graphhopper() {
        let data = select_data!(name = "graphhopper");
        // DATASET.iter().find(|x| x.0 == "graphhopper").unwrap();
        let repo_name = data.1;
        eprintln!("{}:", repo_name);
        let commit = data.2;
        let limit = 1000;
        let query = hyperast_benchmark_smells::queries::exception_handling();
        let query = format!("{} @root", query[0]);
        println!("{}:", repo_name);
        println!("{}", query);
        many(repo_name, commit, limit, &query, [].as_slice());
    }
}
