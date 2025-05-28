#[cfg(target_os = "linux")]
mod iai {
    use hyper_diff::matchers::optimal::zs::{self, str_distance_patched};
    use iai_callgrind::{library_benchmark, library_benchmark_group, main};
    use std::hint::black_box;
    use str_distance::DistanceMetric;

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

    #[library_benchmark]
    #[bench::zero(0)]
    #[bench::one(1)]
    #[bench::two(2)]
    #[bench::three(3)]
    #[bench::four(4)]
    fn bench_qgram_distance_hash_opti(i: usize) -> f64 {
        let p = black_box(PAIRS[i]);
        std::hint::black_box(zs::qgrams::qgram_distance_hash_opti(p.0, p.1))
    }

    #[library_benchmark]
    #[bench::zero(0)]
    #[bench::one(1)]
    #[bench::two(2)]
    #[bench::three(3)]
    #[bench::four(4)]
    fn bench_qgram_str_distance_patched(i: usize) -> f64 {
        let p = black_box(PAIRS[i]);
        std::hint::black_box(str_distance_patched::QGram::new(3).normalized(p.0, p.1))
    }

    library_benchmark_group!(name = bench_qgram_group; benchmarks =
        bench_qgram_distance_hash_opti,
        bench_qgram_str_distance_patched
    );
    main!(library_benchmark_groups = bench_qgram_group);

    pub fn call_main() {
        main()
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {}

#[cfg(target_os = "linux")]
fn main() {
    iai::call_main();
}
