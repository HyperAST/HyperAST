use std::{env, path::Path};

use hyper_ast_benchmark_diffs::{buggy_fixed::run_dir, with_profiling};
#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;
use std::io::Write;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

pub(crate) fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format(|buf, record| {
            if record.level().to_level_filter() > log::LevelFilter::Debug {
                writeln!(buf, "{}", record.args())
            } else {
                writeln!(
                    buf,
                    "[{} {}] {}",
                    buf.timestamp_millis(),
                    record.level(),
                    record.args()
                )
            }
        })
        .init();
    with_profiling(Path::new("profile.pb"), || {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let data_root = root.parent().unwrap().join("gt_datasets/defects4j");
        let data_root = data_root.as_path();
        std::fs::read_dir(data_root).expect("should be a dir");
        let root_buggy = data_root.join("buggy/Jsoup"); // /Jsoup/92
        let root_fixed = data_root.join("fixed/Jsoup"); // /Jsoup/92
        let res = run_dir(&root_buggy, &root_fixed).unwrap();
        dbg!(res);
        println!("success");
    });
}
