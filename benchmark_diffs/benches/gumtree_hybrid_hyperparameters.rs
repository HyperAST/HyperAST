use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use hyper_diff::algorithms;
use hyperast::store::SimpleStores;
use hyperast_benchmark_diffs::preprocess::parse_string_pair;
use std::{
    hint::black_box,
    path::{Path, PathBuf},
};

fn find_java_files(dir: &Path, root: &Path) -> Vec<PathBuf> {
    let mut java_files = Vec::new();

    if dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        java_files.extend(find_java_files(&path, &root));
                    } else if path.extension().and_then(|ext| ext.to_str()) == Some("java") {
                        if let Ok(rel_path) = path.strip_prefix(root) {
                            java_files.push(rel_path.to_path_buf());
                        }
                    }
                }
            }
        }
    }

    java_files
}

fn diff_benchmark(c: &mut Criterion) {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    let mut group = c.benchmark_group("gumtree_hybrid_hyperparameter_minsize");

    group.sample_size(10);

    // Get path to dataset
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("datasets/defects4j");

    let before_dir = root.join("before");
    let test_inputs: Vec<_> = find_java_files(&before_dir, &before_dir)
        .into_iter()
        .map(|path| {
            let buggy_path = root.join("before").join(&path);
            let fixed_path = root.join("after").join(&path);

            // Read file contents
            let buggy_content = std::fs::read_to_string(&buggy_path)
                .expect(&format!("Failed to read buggy file: {:?}", buggy_path));
            let fixed_content = std::fs::read_to_string(&fixed_path)
                .expect(&format!("Failed to read fixed file: {:?}", fixed_path));

            log::info!(
                "Processing test case: {:?} with {} lines",
                path,
                buggy_content.lines().count()
            );

            (path, buggy_content, fixed_content)
        })
        .collect();

    macro_rules! run_diff_for_thresholds {
        ($($threshold:expr),*) => {$({
            const SIZE_THRESHOLD: usize = $threshold;
            group.bench_with_input(
                BenchmarkId::new("hybrid_hyperparameter", SIZE_THRESHOLD),
                &SIZE_THRESHOLD,
                |b, _i| {
                    b.iter(|| {
                        for (_, b, f) in test_inputs.iter() {
                            run_diff::<SIZE_THRESHOLD>(b, f);
                        }
                    })
                }
            );
        })*};
    }

    run_diff_for_thresholds!(0, 50, 100, 200, 300, 500, 700, 1000, 1500);
    group.finish();
}

fn run_diff<const SIZE_THRESHOLD: usize>(src: &str, dst: &str) {
    let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default();

    let (src_tr, dst_tr) =
        parse_string_pair(&mut stores, &mut md_cache, black_box(src), black_box(dst));

    let diff_result = algorithms::gumtree_hybrid::diff_hybrid(
        &stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
        SIZE_THRESHOLD,
    );

    black_box(diff_result);
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10).configure_from_args();
    targets = diff_benchmark
);
criterion_main!(benches);
