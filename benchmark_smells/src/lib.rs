//! Benchmark of smells finder using the hyperAST.
//!
//! validity: baseline tree-sitter, same number of matches (tree-sitter and our query syntax have slightly diverged, so it can only be done on a subset)
//!
//! performances: baseline tree-sitter, time/memory show perf issues when not using our approach
//!
//! code: repository (reuse known repositories) (but tree-sitter does not work on entire commits) ) / files (reuse tsg dataset)
//!
//! Priorities:
//! The main objective of this benchmark suite is to measure performances (mem, latency, wall time).
//! The validity will first be limitated to the capabilities of the baseline.

pub mod diffing;
pub mod github_ranges;
pub mod positions;
pub mod queries;
pub mod simple;

mod data;
pub use data::DATASET;

use std::{env, fs, io, path, time};

pub fn tempfile() -> io::Result<(path::PathBuf, fs::File)> {
    let mut path = env::temp_dir();
    let file_name = time::SystemTime::UNIX_EPOCH;
    path.push(file_name.elapsed().unwrap().as_nanos().to_string());
    let file = fs::File::create(&path)?;
    Ok((path, file))
}

pub fn with_profiling<F: Fn()>(out: &path::Path, f: F) {
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&[
            // "libc",
            "libgcc", "pthread", "vdso",
        ])
        .build()
        .unwrap();
    f();
    match guard.report().build() {
        Ok(report) => {
            let mut file = fs::File::create(out).unwrap();
            let profile = report.pprof().unwrap();
            use pprof::protos::Message;
            let mut content = Vec::new();
            profile.encode(&mut content).unwrap();
            use io::Write;
            file.write_all(&content).unwrap();
        }
        Err(_) => {}
    };
}
