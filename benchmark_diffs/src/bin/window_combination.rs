// window of one is just consecutive commits

use hyper_ast_cvs_git::preprocessed::PreProcessedRepository;
use std::{env, io::Write, path::PathBuf, str::FromStr};

use hyper_ast_benchmark_diffs::window_combination::windowed_commits_compare;

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
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
