use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hyper_diff::algorithms;
use hyperast::store::SimpleStores;
use hyperast_benchmark_diffs::preprocess::JavaPreprocessFileSys;
use hyperast_gen_ts_java::{
    legion_with_refs::{self, JavaTreeGen, Local},
    types::TStore,
};
use hyperast_vcs_git::no_space::NoSpaceNodeStoreWrapper;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum GumtreeVariant {
    Greedy,
    Stable,
    GreedyLazy,
    StableLazy,
}

impl fmt::Display for GumtreeVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GumtreeVariant::Greedy => write!(f, "Greedy"),
            GumtreeVariant::Stable => write!(f, "Stable"),
            GumtreeVariant::GreedyLazy => write!(f, "Lazy Greedy"),
            GumtreeVariant::StableLazy => write!(f, "Lazy Stable"),
        }
    }
}

impl GumtreeVariant {
    pub fn variants() -> Vec<Self> {
        vec![
            Self::Greedy,
            Self::Stable,
            Self::GreedyLazy,
            Self::StableLazy,
        ]
    }
}

#[derive(Clone, Copy)]
pub enum DataSet {
    Defects4j,
    GhJava,
}

impl DataSet {
    pub fn name(&self) -> &str {
        match self {
            DataSet::Defects4j => "defects4j",
            DataSet::GhJava => "gh-java",
        }
    }
}

pub fn dataset_roots(
    root: &Path,
    dataset: DataSet,
    project: Option<&'static str>,
) -> [std::path::PathBuf; 2] {
    dbg!(&root);
    assert!(
        root.exists(),
        "you should clone the gumtree dataset:\n`cd ..; git clone git@github.com:GumTreeDiff/datasets.git gt_datasets; cd gt_datasets; git checkout 33024da8de4c519bb1c1146b19d91d6cb4c81ea6`"
    );
    let data_root = root.join(dataset.name());
    assert!(
        data_root.exists(),
        "this dataset does not exist or was renamed"
    );
    let data_root = data_root.as_path();
    std::fs::read_dir(data_root).expect("should be a dir");
    let src = data_root.join("before");
    let dst = data_root.join("after");
    assert!(src.exists(), "probably using the wrong format");
    assert!(dst.exists(), "probably using the wrong format");
    if let Some(project_path) = project {
        return [src, dst].map(|path| path.join(project_path));
    }
    [src, dst]
}

fn parse_dir_per_file(
    java_gen: &mut JavaPreprocessFileSys,
    src_root: PathBuf,
    dst_root: PathBuf,
) -> Vec<(String, Local, Local)> {
    let src_dir = std::fs::read_dir(&src_root)
        .expect(&format!("{:?} should be a dir", src_root))
        .into_iter()
        .filter_map(|x| x.ok());
    let dst_dir = std::fs::read_dir(&dst_root)
        .expect(&format!("{:?} should be a dir", dst_root))
        .into_iter()
        .filter_map(|x| x.ok());

    let dir = src_dir.zip(dst_dir);

    dir.flat_map(|(src_entry, dst_entry)| {
        match (src_entry.file_type(), dst_entry.file_type()) {
            (Ok(src), Ok(dst)) => {
                if src.is_file() && dst.is_file() {
                    let path = src_entry.path().to_string_lossy().into_owned();
                    let name = src_entry.file_name().into_string().expect("file name");
                    dbg!(&name);
                    assert_eq!(src_entry.file_name(), dst_entry.file_name());

                    return vec![(
                        path,
                        parse_file(java_gen, src_entry.path()),
                        parse_file(java_gen, dst_entry.path()),
                    )];
                } else if src.is_dir() && dst.is_dir() {
                    // TODO: do something with this
                    return parse_dir_per_file(java_gen, src_entry.path(), dst_entry.path());
                } else {
                    dbg!(src, dst);
                    panic!("Directory structure mismatch between src and dst!");
                }
            }
            (Err(_), _) => panic!("no file type"),
            (_, Err(_)) => panic!("no file type"),
        }
    })
    .collect()
}

fn parse_file(java_gen: &mut JavaPreprocessFileSys, path: PathBuf) -> Local {
    let bytes = std::fs::read(&path).expect("the code");

    let tree = match legion_with_refs::tree_sitter_parse(&bytes) {
        Ok(t) => t,
        Err(t) => t,
    };

    let line_break = if bytes.contains(&b'\r') {
        "\r\n".as_bytes().to_vec()
    } else {
        "\n".as_bytes().to_vec()
    };
    let mut java_tree_gen =
        JavaTreeGen::new(&mut java_gen.main_stores, &mut java_gen.java_md_cache)
            .with_line_break(line_break);
    let full_node = java_tree_gen.generate_file(
        path.file_name()
            .expect("a file name")
            .to_string_lossy()
            .as_bytes(),
        &bytes,
        tree.walk(),
    );
    full_node.local
}

fn diff_benchmark(c: &mut Criterion) {
    let mut timings: HashMap<String, HashMap<GumtreeVariant, Vec<f64>>> = HashMap::new();
    let bench_start = Instant::now();
    for project in [
        // "Chart",
        // "Cli",
        // "Closure",
        // "Codec",
        "Collections",
        // "Compress",
        // "Csv",
        // "Gson",
        // "JacksonCore",
        // "JacksonDatabind",
        // "JacksonXml",
        // "Jsoup",
        // "JxPath",
        // "Lang",
        // "Math",
        // "Mockito",
        // "Time",
    ] {
        diff_benchmark_project(c, project, &mut timings);
    }
    let elapsed = bench_start.elapsed();
    println!("Finished benchmarking in {:.1}s", elapsed.as_secs_f64());

    // Write results to csv
    let mut file = File::create("benchmark_result.csv").unwrap();
    writeln!(file, "file,variant,run,runtime").unwrap();

    for (file_name, variant_map) in timings {
        for (variant, runtimes) in variant_map {
            for (run_index, runtime) in runtimes.iter().enumerate() {
                writeln!(
                    file,
                    "{},{},{},{}",
                    file_name,
                    variant,
                    run_index + 1,
                    runtime
                )
                .unwrap();
            }
        }
    }
}

fn diff_benchmark_project(
    c: &mut Criterion,
    project: &'static str,
    timings: &mut HashMap<String, HashMap<GumtreeVariant, Vec<f64>>>,
) {
    let gumtree_dataset_dir = std::env::var("GUMTREE_DATASET_DIR").unwrap();
    let root = Path::new(&gumtree_dataset_dir);
    let [src_path, dst_path] = dataset_roots(root, DataSet::Defects4j, Some(project));

    let stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let md_cache = Default::default();
    let mut java_gen = JavaPreprocessFileSys {
        main_stores: stores,
        java_md_cache: md_cache,
    };
    let pairs = parse_dir_per_file(&mut java_gen, src_path, dst_path);

    let stores = hyperast_vcs_git::no_space::as_nospaces2(&java_gen.main_stores);

    let files = pairs.len();
    let mut group = c.benchmark_group(project);
    for (i, (file_name, src, dst)) in pairs.iter().enumerate() {
        for variant in GumtreeVariant::variants() {
            print!("Benching file {} out of {} - [", i + 1, files);
            let progress = (i * 150) / files;
            for _ in 0..progress {
                print!("#");
            }
            for _ in progress..150 {
                print!("-")
            }
            println!("]");
            group.bench_function(format!("{}:{}", &variant, &file_name), |b| {
                b.iter_custom(|_iters| {
                    let start = Instant::now();
                    black_box(run(&stores, &src, &dst, variant));
                    let elapsed = start.elapsed();
                    println!("{}: {}", variant, elapsed.as_secs_f64());

                    timings
                        .entry(file_name.to_string())
                        .or_insert_with(HashMap::new)
                        .entry(variant)
                        .or_insert_with(Vec::new)
                        .push(elapsed.as_secs_f64());

                    elapsed
                });
            });
        }
    }
}

pub fn run(
    stores: &SimpleStores<TStore, NoSpaceNodeStoreWrapper, &hyperast::store::labels::LabelStore>,
    src_tr: &Local,
    dst_tr: &Local,
    variant: GumtreeVariant,
) {
    let diff = match variant {
        GumtreeVariant::Greedy => algorithms::gumtree::diff,
        GumtreeVariant::Stable => algorithms::gumtree_stable::diff,
        GumtreeVariant::GreedyLazy => algorithms::gumtree_lazy::diff,
        GumtreeVariant::StableLazy => algorithms::gumtree_stable_lazy::diff,
    };

    diff(stores, &src_tr.compressed_node, &dst_tr.compressed_node);
}

criterion_group!(
    name = benches;
    config = Criterion::default().without_plots().configure_from_args()
        .nresamples(1)
        .measurement_time(Duration::from_nanos(1))
        .sample_size(20)
        .warm_up_time(Duration::from_nanos(1));
    targets = diff_benchmark
);
criterion_main!(benches);
