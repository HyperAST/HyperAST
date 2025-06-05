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

    print_pos(repo_name, commit, limit, query, [].as_slice());
}

const INCREMENTAL_QUERIES: bool = true;

fn print_pos(
    repo_name: &str,
    commit: &str,
    limit: usize,
    query: &str,
    precomputeds: impl hyperast_tsquery::ArrayStr,
) {
    let query = if INCREMENTAL_QUERIES {
        hyperast_tsquery::Query::with_precomputed(
            &query,
            hyperast_gen_ts_java::language(),
            precomputeds,
        )
        .map(|x| x.1)
    } else {
        hyperast_tsquery::Query::new(&query, hyperast_gen_ts_java::language())
    };

    let query = match query {
        Ok(query) => query,
        Err(err) => {
            eprintln!("{}", err);
            panic!("there is an error in the query");
        }
    };

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

    for oid in oids {
        let commit = preprocessed.commits.get_key_value(&oid).unwrap();
        let time = commit.1.processing_time();
        let tr = commit.1.ast_root;
        use hyperast::types::WithStats;
        let s = stores.node_store.resolve(tr).size();

        let matches = hyperast_benchmark_smells::github_ranges::compute_formated_ranges(
            stores,
            tr,
            &query,
            repo_name,
            &oid.to_string(),
        );
        let matches = matches
            .into_iter()
            .map(|x| {
                let x: String = x.into_iter().take(10).map(|x| format!("{},", x)).collect();
                format!(",[{:?}]", x)
            })
            .collect::<String>();

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
fn bench_conditional_logic() {
    let repo_name = "INRIA/spoon";
    let commit = "56e12a0c0e0e69ea70863011b4f4ca3305e0542b";
    let limit = 4000;
    let around = |s| {
        format!(
            r#"(if_statement
        consequence: (_
            (expression_statement
                (method_invocation
                    name: (identifier) (#EQ? "{s}")
                )
            )
        )
        !alternative
    ) @root"#
        )
    };
    let query = &format!(
        // "{}\n{}\n{}\n{}\n{}",
        // "{}\n{}",
        "{}",
        around("assertThat"),
        // around("assertEquals"),
        // around("assertSame"),
        // around("assertTrue"),
        // around("assertNull"),
    );

    let subs = [
        "(if_statement)",
        "(if_statement !alternative)",
        r#"(identifier) (#EQ? "assertThat")"#,
        // r#"(identifier) (#EQ? "assertEquals")"#,
        // r#"(identifier) (#EQ? "assertNotEquals")"#,
    ]
    .as_slice();

    print_pos(repo_name, commit, limit, query, subs);
}

#[test]
fn assertion_roulette() {
    let repo_name = "INRIA/spoon";
    let commit = "7c7f094bb22a350fa64289a94880cc3e7231468f";
    let limit = 6;
    let query = hyperast_benchmark_smells::queries::assertion_roulette();
    print!("{}", query);
    print_pos(repo_name, commit, limit, &query, [].as_slice());
}
