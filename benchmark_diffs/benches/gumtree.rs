use std::time::Duration;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hyper_diff::algorithms;
use hyperast::store::SimpleStores;
use hyperast_benchmark_diffs::preprocess::parse_string_pair;

// Load the content of A1.java and A2.java
const A1_CONTENT: &str = include_str!("../src/A1.java");
const A2_CONTENT: &str = include_str!("../src/A2.java");

fn diff_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Gumtree");
    group
        .significance_level(0.05)
        .sample_size(100)
        .measurement_time(Duration::from_secs(12));
    group.bench_function("hyperdiff_gumtree_stable_bottom_up_A1_A2", |b| {
        // Setup (none)
        b.iter(|| {
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

            // Perform the diff using gumtree stable
            let diff_result = algorithms::gumtree_stable::diff(
                &stores,
                &src_tr.local.compressed_node,
                &dst_tr.local.compressed_node,
            );

            // Ensure the result is used to prevent optimization
            black_box(diff_result);
        })
    });
    //return;
    group.bench_function("hyperdiff_gumtree_bottom_up_A1_A2", |b| {
        // Setup (none)
        b.iter(|| {
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

            // Perform the diff using gumtree
            let diff_result = algorithms::gumtree::diff(
                &stores,
                &src_tr.local.compressed_node,
                &dst_tr.local.compressed_node,
            );

            // Ensure the result is used to prevent optimization
            black_box(diff_result);
        })
    });
}

criterion_group!(benches, diff_benchmark);
criterion_main!(benches);
