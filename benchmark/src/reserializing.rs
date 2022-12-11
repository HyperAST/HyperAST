pub mod write_serializer;

use std::{env, io::Write};

use hyper_ast_cvs_git::{git::fetch_github_repository, preprocessed::PreProcessedRepository};
use serde::{Deserialize, Serialize};

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
    benchmark_main()
}

fn benchmark_main() {
    // let f = env_logger::fmt::BufferWriter
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
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

    println!(
        "(eq, not): {:?}",
        check_random_files_reserialization(repo_name)
    );
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Perfs {
    /// time in nano seconds
    time: u128,
    /// memory in bytes
    memory: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Info {
    repo_name: String,
    commit: String,
    no: usize,
    batch_id: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Instance {
    construction_perfs: Perfs,
    search_perfs: Option<Perfs>,
    info: Info,
}

fn check_random_files_reserialization(
    repo_name: &String,
    // before: &str,
    // after: &str,
) -> (usize, usize) {
    let mut preprocessed = PreProcessedRepository::new(&repo_name);
    preprocessed.check_random_files_reserialization(&mut fetch_github_repository(&repo_name))
}
