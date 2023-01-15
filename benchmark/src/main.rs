pub mod write_serializer;

use std::{
    env,
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
    str::FromStr,
    time::Instant,
};

use hyper_ast_cvs_git::{
    allrefs::write_referencial_relations,
    git::{fetch_github_repository, retrieve_commit},
    preprocessed::PreProcessedRepository,
};
use hyper_ast_gen_ts_java::utils::memusage_linux;
use serde::{Deserialize, Serialize};

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
    let dir_path = args.get(4).map_or("", |x| x);
    let out = args.get(5).and_then(|x| {
        if x.is_empty() {
            None
        } else {
            PathBuf::from_str(x).ok()
        }
    });

    // single_commit_ref_ana(repo_name, after, dir_path, out);
    multi_commit_ref_ana::<50>(repo_name, before, after, dir_path, out);
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

fn multi_commit_ref_ana<const SEARCH_SKIP_SIZE: usize>(
    repo_name: &String,
    before: &str,
    after: &str,
    dir_path: &str,
    out: Option<PathBuf>,
) {
    let batch_id = format!("{}:({},{})", repo_name, before, after);
    let mut preprocessed = PreProcessedRepository::new(&repo_name);
    let processing_ordered_commits = preprocessed.pre_process_with_limit(
        &mut fetch_github_repository(&repo_name),
        before,
        after,
        dir_path,
        1000,
    );
    let mu = memusage_linux();
    log::warn!("total memory used {}", mu);
    preprocessed.purge_caches();
    let mu = mu - memusage_linux();
    log::warn!("cache size: {}", mu);
    log::warn!(
        "commits to search ({}): {:?}",
        preprocessed.commits.len(),
        processing_ordered_commits
    );
    let mu = memusage_linux();
    let mut i = 0;
    for c in &processing_ordered_commits {
        log::warn!("search of commit {:?}", c.to_string());
        let c = preprocessed.commits.get_key_value(c).unwrap();
        let root = c.1.ast_root;
        let out = out.as_ref().map(|x| x.join(c.0.to_string()));
        if let Some(out) = out {
            let mut file = File::create(out).unwrap();
            let mut buf = BufWriter::with_capacity(BUFF_WRITER_CAPACITY, &mut file);

            let info = Info {
                repo_name: repo_name.to_string(),
                commit: c.0.to_string(),
                no: i,
                batch_id: batch_id.clone(),
            };

            let construction_time = c.1.processing_time();
            let construction_memory_fooprint = c.1.memory_used().bytes().max(0);
            let construction_perfs = Perfs {
                time: construction_time,
                memory: construction_memory_fooprint.try_into().unwrap(),
            };

            if i % SEARCH_SKIP_SIZE != 0 {
                i += 1;
                let instance = Instance {
                    construction_perfs,
                    search_perfs: None,
                    info,
                };
                instance.serialize(WriteJson::from(&mut buf)).unwrap();

                buf.flush().unwrap();
                continue;
            }
            log::warn!("search refs");

            write!(buf, r#"{{"relations":["#).unwrap();

            let now = Instant::now();

            write_referencial_relations(&preprocessed.processor.main_stores, root, &mut buf);

            let search_time = now.elapsed().as_nanos();

            buf.flush().unwrap();

            let current_search_memory_fooprint: isize = (memusage_linux() - mu.clone()).into();
            let with_search_memory_fooprint =
                construction_memory_fooprint + current_search_memory_fooprint;
            // let ast_memory_fooprint = 0; // would need to flush caches
            write!(buf, "]").unwrap();

            let instance = Instance {
                construction_perfs,
                search_perfs: Some(Perfs {
                    time: search_time,
                    memory: with_search_memory_fooprint.max(0).unsigned_abs(),
                }),
                info,
            };
            instance
                .serialize(WritePartialJson::from(&mut buf))
                .unwrap();

            buf.flush().unwrap();
        } else {
            let mut out = io::stdout();
            write_referencial_relations(&preprocessed.processor.main_stores, root, &mut out);
            out.flush().unwrap();
        }
        log::warn!("done searching refs");
        i += 1;
    }
    let mu = memusage_linux();
    drop(preprocessed);
    log::warn!("hyperAST size: {}", mu - memusage_linux());
}

pub fn single_commit_ref_ana(
    repo_name: &String,
    after: &str,
    dir_path: &str,
    out: Option<PathBuf>,
) {
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
        write_referencial_relations(&preprocessed.processor.main_stores, root, &mut out);
        out.flush().unwrap();
    } else {
        let mut out = io::stdout();
        write_referencial_relations(&preprocessed.processor.main_stores, root, &mut out);
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
