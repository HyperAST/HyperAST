use criterion::{
    BenchmarkId, Criterion, SamplingMode, Throughput, black_box, criterion_group, criterion_main,
    measurement::{Measurement, ValueFormatter},
};
use hyperast_benchmark_diffs::common;
use regex::Regex;
use std::fmt;
use std::marker::PhantomData;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Configuration for different optimization combinations to benchmark
struct AlgorithmConfig {
    name: &'static str,
    arg: &'static str,
}

fn create_algorithm_configs() -> Vec<AlgorithmConfig> {
    vec![
        AlgorithmConfig {
            name: "change_distiller_lazy_2",
            arg: "change_distiller_lazy_2",
        },
        AlgorithmConfig {
            name: "change_distiller_lazy",
            arg: "change_distiller_lazy",
        },
        AlgorithmConfig {
            name: "change_distiller",
            arg: "change_distiller",
        },
        AlgorithmConfig {
            name: "gumtree_lazy",
            arg: "gumtree_lazy",
        },
    ]
}

#[derive(Clone)]
struct PeakMemoryInput {
    algorithm: String,
    buggy: String,
    fixed: String,
}

#[derive(Clone, Copy, Debug)]
struct PeakMemoryValue(u64); // in bytes

struct PeakMemory;

impl Measurement for PeakMemory {
    type Intermediate = PeakMemoryInput;
    type Value = PeakMemoryValue;

    fn start(&self) -> Self::Intermediate {
        panic!("Use iter_custom for custom measurement input");
    }

    fn end(&self, i: Self::Intermediate) -> Self::Value {
        let output = Command::new("/usr/bin/time")
            .args([
                "-l",
                "./target/release/algorithm_runner",
                &i.algorithm,
                &i.buggy,
                &i.fixed,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute time command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        let peak = extract_memory_stat(&stderr, "peak memory footprint");
        PeakMemoryValue(peak)
    }

    fn add(&self, v1: &Self::Value, v2: &Self::Value) -> Self::Value {
        PeakMemoryValue(v1.0 + v2.0)
    }
    fn zero(&self) -> Self::Value {
        PeakMemoryValue(0)
    }
    fn to_f64(&self, val: &Self::Value) -> f64 {
        val.0 as f64 / 1_048_576.0 // MB
    }
    fn formatter(&self) -> &dyn ValueFormatter {
        &PeakMemoryFormatter
    }
}

struct PeakMemoryFormatter;
impl ValueFormatter for PeakMemoryFormatter {
    fn format_value(&self, value: f64) -> String {
        format!("{:.2} MB", value)
    }
    fn format_throughput(&self, throughput: &Throughput, value: f64) -> String {
        match *throughput {
            Throughput::Bytes(bytes) => format!("{:.2} MB/s", (bytes as f64) / value / 1_048_576.0),
            Throughput::Elements(elems) => format!("{:.2} MB/elem", value / elems as f64),
            Throughput::BytesDecimal(bytes) => {
                format!("{:.2} MB/s", (bytes as f64) / value / 1_048_576.0)
            }
        }
    }
    fn scale_values(&self, typical: f64, values: &mut [f64]) -> &'static str {
        // Already in MB
        "MB"
    }
    fn scale_throughputs(
        &self,
        _typical: f64,
        _throughput: &Throughput,
        _values: &mut [f64],
    ) -> &'static str {
        "MB"
    }
    fn scale_for_machines(&self, _values: &mut [f64]) -> &'static str {
        "MB"
    }
}

fn extract_memory_stat(time_output: &str, stat_name: &str) -> u64 {
    for line in time_output.lines() {
        if line.contains(stat_name) {
            let re = Regex::new(r"(\d+)").unwrap();
            if let Some(captures) = re.captures(line) {
                if let Some(value_str) = captures.get(1) {
                    if let Ok(value) = value_str.as_str().parse::<u64>() {
                        return value;
                    }
                }
            }
        }
    }
    0
}

fn benchmark_memory_criterion(c: &mut Criterion<PeakMemory>) {
    // Build the runner binary first
    let build_status = Command::new("cargo")
        .args(["build", "--release", "--bin", "algorithm_runner"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to build algorithm_runner");
    assert!(build_status.success(), "Build failed");

    let test_inputs = common::get_all_case_paths();
    let algorithms = &["change_distiller", "change_distiller_lazy"];

    let mut group = c.benchmark_group("cd_memory_time");
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);

    for (input_idx, (buggy, fixed)) in test_inputs.iter().enumerate() {
        let parsed_input = common::preprocess(&(buggy, fixed));
        let file_name = buggy.split('/').last().unwrap().to_string();
        for alg in algorithms.iter() {
            let bench_name = format!(
                "Mem CD Single - {} - {} loc {} nodes {}",
                alg, parsed_input.loc, parsed_input.node_count, file_name
            );
            let input = PeakMemoryInput {
                algorithm: alg.to_string(),
                buggy: buggy.clone(),
                fixed: fixed.clone(),
            };
            group.bench_with_input(bench_name, &input, |b, input| {
                b.iter_custom(|iters| {
                    let mut total = PeakMemoryValue(0);
                    for _ in 0..iters {
                        let val = PeakMemory.end(input.clone());
                        total = PeakMemory.add(&total, &val);
                    }
                    total
                });
            });
        }
    }

    group.finish();
}

fn custom_criterion() -> Criterion<PeakMemory> {
    Criterion::default().with_measurement(PeakMemory)
}

criterion_group! {
    name = benches;
    config = custom_criterion();
    targets = benchmark_memory_criterion,
}

criterion_main!(benches);
