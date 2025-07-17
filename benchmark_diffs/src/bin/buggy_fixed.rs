use std::{env, path::Path};

use hyperast_benchmark_diffs::{
    buggy_fixed::{buggy_fixed_dataset_roots, run_dir},
    setup_env_logger, with_profiling,
};
#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

pub(crate) fn main() {
    setup_env_logger();
    with_profiling(Path::new("profile.pb"), || {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let src_dst = buggy_fixed_dataset_roots(root, "defects4j");
        let [src, dst] = src_dst.map(|x| x.join("Jsoup"));
        let res = run_dir(&src, &dst).unwrap();
        dbg!(res);
        println!("success");
    });
}
