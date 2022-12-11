use std::{
    env,
    path::Path,
};

use hyper_ast_benchmark_diffs::{with_profiling, buggy_fixed::run_dir,
};

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

pub(crate) fn main() {
    with_profiling(Path::new("profile.pb"), || {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let data_root = root.parent().unwrap().join("gt_datasets/defects4j");
        let data_root = data_root.as_path();
        std::fs::read_dir(data_root).expect("should be a dir");
        let root_buggy = data_root.join("buggy/Jsoup"); // /Jsoup/92
        let root_fixed = data_root.join("fixed/Jsoup"); // /Jsoup/92
        run_dir(&root_buggy, &root_fixed).unwrap();
        println!("success");
    });
}