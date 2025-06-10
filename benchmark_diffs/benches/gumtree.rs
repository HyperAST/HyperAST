use std::{
    fmt,
    fs::File,
    path::Path,
    time::{Duration, Instant},
};

use criterion::{Criterion, SamplingMode, black_box, criterion_group, criterion_main};
use hyper_diff::{
    actions::Actions,
    algorithms::{self, DiffResult, MappingDurations},
};
use hyperast::{store::SimpleStores, types::LabelStore};
use hyperast_benchmark_diffs::{
    other_tools,
    postprocess::{CompressedBfPostProcess, PathJsonPostProcess},
    preprocess::{JavaPreprocessFileSys, parse_dir_pair, parse_string_pair},
};
use hyperast_gen_ts_java::{legion_with_refs::Local, types::TStore};
use hyperast_vcs_git::no_space::NoSpaceNodeStoreWrapper;

// Load the content of A1.java and A2.java
const A1_CONTENT: &str = include_str!("../src/A1.java");
const A2_CONTENT: &str = include_str!("../src/A2.java");

#[derive(Clone, Copy)]
enum GumtreeVariant {
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

#[derive(Clone, Copy)]
enum DataSet {
    Defects4j(&'static str),
    BugsInPy,
    GhJava,
    GhPython,
}

impl DataSet {
    pub fn name(&self) -> &str {
        match self {
            DataSet::Defects4j(project) => "defects4j",
            DataSet::BugsInPy => "bugsinpy",
            DataSet::GhJava => "gh-java",
            DataSet::GhPython => "gh-python",
        }
    }
}

fn diff_benchmark(c: &mut Criterion) {
    let gumtree_dataset_dir = std::env::var("GUMTREE_DATASET_DIR").unwrap();
    let root = Path::new(&gumtree_dataset_dir);
    let [src, dst] = dataset_roots(root, DataSet::Defects4j("Closure"));

    let stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let md_cache = Default::default();
    let mut java_gen = JavaPreprocessFileSys {
        main_stores: stores,
        java_md_cache: md_cache,
    };
    let now = Instant::now();
    let (src_tr, dst_tr) = parse_dir_pair(&mut java_gen, &src, &dst);
    let parse_t = now.elapsed().as_secs_f64();

    let stores = hyperast_vcs_git::no_space::as_nospaces2(&java_gen.main_stores);

    let mut group = c.benchmark_group("Gumtree");
    group.sample_size(10);
    //.significance_level(0.1)
    //.sampling_mode(SamplingMode::Flat);
    //.measurement_time(Duration::from_secs(12));
    for variant in [
        GumtreeVariant::Greedy,
        GumtreeVariant::GreedyLazy,
        GumtreeVariant::Stable,
        GumtreeVariant::StableLazy,
    ] {
        group.bench_function(format!("{}", &variant), |b| {
            b.iter(|| {
                black_box(run(&stores, &src_tr, &dst_tr, variant));
            })
        });
    }
}

pub fn dataset_roots(root: &Path, dataset: DataSet) -> [std::path::PathBuf; 2] {
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
    if let DataSet::Defects4j(project) = dataset {
        return [src, dst].map(|path| path.join(project));
    }
    [src, dst]
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

    let DiffResult {
        mapping_durations,
        mapper,
        actions: hast_actions,
        prepare_gen_t,
        gen_t,
    } = diff(stores, &src_tr.compressed_node, &dst_tr.compressed_node);
    let MappingDurations([subtree_matcher_t, bottomup_matcher_t]) = mapping_durations.into();

    let timings = vec![subtree_matcher_t, bottomup_matcher_t, gen_t + prepare_gen_t];
    dbg!(&timings);
}

pub fn benchmark_gumtree_small_file(variant: GumtreeVariant) {
    // Initialize stores for each iteration to avoid side effects
    let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default(); // [cite: 133, 139]

    // Parse the two Java files
    let (src_tr, dst_tr) = parse_string_pair(
        &mut stores,
        &mut md_cache,
        black_box(A1_CONTENT), // Use black_box to prevent optimizations
        black_box(A2_CONTENT),
    );

    let diff = match variant {
        GumtreeVariant::Greedy => algorithms::gumtree::diff,
        GumtreeVariant::Stable => algorithms::gumtree_stable::diff,
        GumtreeVariant::GreedyLazy => algorithms::gumtree_lazy::diff,
        GumtreeVariant::StableLazy => algorithms::gumtree_stable_lazy::diff,
    };

    let diff_result = diff(
        &stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    );

    // Ensure the result is used to prevent optimization
    black_box(diff_result);
}

criterion_group!(
    name = benches;
    config = Criterion::default().configure_from_args();
    targets = diff_benchmark
);
criterion_main!(benches);
