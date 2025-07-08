use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use hyperast_gen_ts_tsquery::search::utils;
use std::hint::black_box;

fn compare_capture_names_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("Capture Names");

    const INPUTS: [&[&str]; 5] = [
        &["name"],
        &["name", "body"],
        &["name", "body", "field", "a"],
        &["name", "body", "field", "a", "b", "a"],
        &["name", "body", "field", "a", "b", "a", "c", "d", "d", "d"],
    ];

    for (i, p) in INPUTS.into_iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("simple", i), &p, |b, p| {
            b.iter(|| {
                let mut capture_names = utils::CaptureNames::default();
                for i in p.iter() {
                    black_box(capture_names.intern(i));
                }
                let capture_names = capture_names.into_arc();
                for i in p.iter() {
                    black_box(capture_names.resolve(i));
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("opt", i), &p, |b, p| {
            b.iter(|| {
                let mut capture_names = utils::opt::CaptureNames::default();
                for i in p.iter() {
                    black_box(capture_names.intern(i));
                }
                let capture_names = capture_names.into_arc();
                for i in p.iter() {
                    black_box(capture_names.resolve(i));
                }
            })
        });
    }
    group.finish()
}

criterion_group!(capture_names, compare_capture_names_group);
criterion_main!(capture_names);
