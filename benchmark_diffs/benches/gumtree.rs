use criterion::measurement::Measurement;
use criterion::{
    BenchmarkGroup, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main,
};
use hyper_diff::algorithms;
use hyperast::store::SimpleStores;
use hyperast::store::defaults::NodeIdentifier;
use hyperast::types::WithStats;
use hyperast::types::{self, HyperAST, NodeId};
use hyperast_benchmark_diffs::preprocess::JavaPreprocessFileSys;
use hyperast_gen_ts_java::legion_with_refs::{self, JavaTreeGen, Local};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum GumtreeVariant {
    Greedy,
    Stable,
    GreedyLazy,
    StableLazy,
}

impl std::fmt::Display for GumtreeVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl std::fmt::Display for DataSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataSet::Defects4j => write!(f, "defects4j"),
            DataSet::GhJava => write!(f, "defects4j"),
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
    let data_root = root.join(dataset.to_string());
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
) -> Vec<(PathBuf, Local, Local)> {
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
                    let path = src_entry.path();
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
            _ => panic!("no file type"),
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
    let mut group = c.benchmark_group("bench_gumtree");
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
        diff_benchmark_project(&mut group, project);
    }
    group.finish();
}

fn diff_benchmark_project(group: &mut BenchmarkGroup<impl Measurement>, project: &'static str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    dbg!(root);
    let src_dst =
        hyperast_benchmark_diffs::buggy_fixed::buggy_fixed_dataset_roots(root, DataSet::Defects4j);
    let [src_path, dst_path] = src_dst.clone().map(|x| x.join(project));

    let stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let md_cache = Default::default();
    let mut java_gen = JavaPreprocessFileSys {
        main_stores: stores,
        java_md_cache: md_cache,
    };
    let pairs = parse_dir_per_file(&mut java_gen, src_path, dst_path);

    let stores = hyperast_vcs_git::no_space::as_nospaces2(&java_gen.main_stores);
    for (_i, (file_name, src, dst)) in pairs.iter().enumerate() {
        group.throughput(Throughput::Elements(
            (stores.node_store().resolve(src.compressed_node).size()
                + stores.node_store().resolve(dst.compressed_node).size())
            .div_ceil(2) as u64,
        ));
        for variant in GumtreeVariant::variants() {
            let file_name = file_name
                .components()
                .skip(src_dst[0].components().count())
                .map(|x| x.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("_");
            group.bench_with_input(
                BenchmarkId::new(variant.to_string(), file_name),
                &(src, dst),
                |b, (src, dst)| {
                    b.iter(|| {
                        run(&stores, src, dst, variant);
                    });
                },
            );
        }
    }
}

pub fn run<HAST: HyperAST<IdN = NodeIdentifier> + Copy>(
    stores: HAST,
    src_tr: &Local,
    dst_tr: &Local,
    variant: GumtreeVariant,
) where
    HAST::IdN: Clone + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Idx: hyperast::PrimInt,
    HAST::Label: Debug + Clone + Copy + Eq,
    for<'t> <HAST as types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
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
    config = Criterion::default().configure_from_args()
        .measurement_time(Duration::from_secs(10))
        .sample_size(10);
    targets = diff_benchmark
);
criterion_main!(benches);
