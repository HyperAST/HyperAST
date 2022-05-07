pub mod write_serializer;

use std::{
    env,
    fs::File,
    io::{self, BufWriter, Seek, SeekFrom, Write},
    ops::Add,
    path::PathBuf,
    str::FromStr,
    time::{Instant, SystemTime},
};

use rusted_gumtree_cvs_git::{
    allrefs::write_referencial_relations,
    git::{fetch_github_repository, retrieve_commit},
    preprocessed::{self, PreProcessedRepository},
};
use rusted_gumtree_gen_ts_java::utils::memusage_linux;
use serde::{Deserialize, Serialize, Serializer};

use crate::write_serializer::{WriteJson, WritePartialJson};

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

// WARN there is a big impact of the buff writer capacity
const BUFF_WRITER_CAPACITY: usize = 4 * 8 * 1024;

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
    let before = args.get(2).map_or("", |x| x);
    let after = args.get(3).map_or("", |x| x);

    println!("(eq, not): {:?}", check_random_files_reserialization(repo_name, before, after));
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
    before: &str,
    after: &str,
) -> (usize, usize) {
    let mut preprocessed = PreProcessedRepository::new(&repo_name);
    preprocessed.check_random_files_reserialization(&mut fetch_github_repository(&repo_name))
}

fn single_commit_ref_ana(repo_name: &String, after: &str, dir_path: &str, out: Option<PathBuf>) {
    let mut preprocessed = PreProcessedRepository::new(&repo_name);
    preprocessed.pre_process_single(&mut fetch_github_repository(&repo_name), after, dir_path);
    let mu = memusage_linux();
    log::warn!("total memory used {}", mu);
    preprocessed.purge_caches();
    log::warn!("cache size: {}", mu - memusage_linux());
    log::warn!("search refs");
    let repository = fetch_github_repository(preprocessed.name());
    // node identifier at after commit
    let root = preprocessed
        .commits
        .get(&retrieve_commit(&repository, after).unwrap().id())
        .unwrap()
        .ast_root;
    if let Some(out) = out {
        let mut out = BufWriter::with_capacity(BUFF_WRITER_CAPACITY, File::create(out).unwrap());
        write_referencial_relations(&preprocessed, root, &mut out);
        out.flush().unwrap();
    } else {
        let mut out = io::stdout();
        write_referencial_relations(&preprocessed, root, &mut out);
        out.flush().unwrap();
    }
    log::warn!("done searching refs");
    let mu = memusage_linux();
    drop(preprocessed);
    log::warn!("hyperAST size: {}", mu - memusage_linux());
}

#[test]
fn all() {
    use std::fs::read_to_string;
    use std::path::Path;
    use std::path::PathBuf;

    use pommes::Project;

    let path: PathBuf = Path::new("pom.xml").to_path_buf();
    println!("path: {}", &path.display());

    let contents = read_to_string(path).unwrap();
    let _parsed: Project = serde_xml_rs::from_str(&contents).unwrap();

    println!("{:#?}", _parsed);
}
