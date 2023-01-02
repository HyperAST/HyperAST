use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hyper_gumtree::matchers::optimal::zs::{self, str_distance_patched};
use str_distance::{DistanceMetric};

fn compare_qgrams_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("Qgram");

    const PAIRS: [(&[u8], &[u8]); 5] = [
        ("abaaacdefg".as_bytes(), "abcdefg".as_bytes()),
        (
            "za".as_bytes(),
            "qvvsdflflvjehrgipuerpq".as_bytes(),
        ),
        (
            "abaaeqrogireiuvnlrpgacdefg".as_bytes(),
            "aaaa".as_bytes(),
        ),
        (
            "abaaeqrogireiuvnlrpgacdefgabaaeqrogireiuvnlrpgacdefg".as_bytes(),
            "qvvsdflflvjehrgipuerpqqvvsdflflvjehrgipuerpq".as_bytes(),
        ),
        (
            "abaaeqro64646s468gireiuvnlrpg137zfaèàç-_éèàaç_è'ç(-cdefgrgeedbdsfdg6546465".as_bytes(),
            "qvvsdflflvjehrgegrhdbeoijovirejvoirzejvoerivjeorivjeroivjeroivjerovijrevoierjvoierjoipuerpq".as_bytes(),
        ),
    ];

    for (i, p) in PAIRS.into_iter().enumerate() {
        group.throughput(Throughput::Bytes((p.0.len() + p.1.len()) as u64));
        group.bench_with_input(BenchmarkId::new("hash_opti", i), &p, |b, p| {
            b.iter(|| zs::qgrams::qgram_distance_hash_opti(p.0, p.1))
        });
        group.bench_with_input(BenchmarkId::new("str_distance", i), &p, |b, p| {
            b.iter(|| str_distance_patched::QGram::new(3).normalized(p.0, p.1))
        });
    }
    group.finish()
}

criterion_group!(qgrams, compare_qgrams_group);
criterion_main!(qgrams);
