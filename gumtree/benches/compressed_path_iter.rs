use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput, black_box};
use hyper_gumtree::tree::tree_path::{self, CompressedTreePath};

fn compare_compressed_path_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("CompressedPathIter");

    let pairs: Vec<CompressedTreePath<u16>> = vec![
        CompressedTreePath::<u16>::from(vec![127, 3, 0, 5]),
        CompressedTreePath::<u16>::from(vec![12, 34, 0, 5, 254, 3, 55, 0, 0, 3, 0, 3, 0, 3, 0]),
        CompressedTreePath::<u16>::from(vec![12, 3, 0, 3, 0, 3, 0, 5, 45]),
        CompressedTreePath::<u16>::from(vec![12, 3, 0, 5, 0, 1]),
        CompressedTreePath::<u16>::from(vec![12, 3, 0, 5]),
        CompressedTreePath::<u16>::from(vec![12, 460, 5, 0, 19, 5]),
        CompressedTreePath::<u16>::from(vec![12, 3, 55, 0, 0, 3, 0, 3, 0, 3, 0, 3, 55, 0, 0, 3, 0, 3, 0, 3, 0]),
    ];

    let pairs: Vec<_> = pairs.iter().map(|x| x.as_bytes()).collect();

    for (i, p) in pairs.into_iter().enumerate() {
        group.throughput(Throughput::Bytes((p.len()) as u64));
        let p: Box<[u8]> = p.into();
        group.bench_with_input(BenchmarkId::new("slicing", i), &p, |b, p| {
            b.iter(|| tree_path::slicing::IntoIter::<u16>::new(p.clone()).map(|x|black_box(x)).collect::<Vec<_>>())
        });
        group.bench_with_input(BenchmarkId::new("indexed", i), &p, |b, p| {
            b.iter(|| tree_path::indexed::IntoIter::<u16>::new(p.clone()).map(|x|black_box(x)).collect::<Vec<_>>())
        });
    }
    group.finish()
}

criterion_group!(paths, compare_compressed_path_iter);
criterion_main!(paths);
