use std::{
    fmt::Display,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use clap::{Parser, ValueEnum};
use hyper_diff::OptimizedDiffConfig;
use hyperast_vcs_git::git::Oid;
use hyperast_vcs_git::preprocessed::PreProcessedRepository;
use indicatif::{ProgressBar, ProgressStyle};

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Parser, Debug)]
#[command(name = "simple_usecase")]
#[command(about = "A simple diff benchmark tool for Git repositories")]
#[command(version, long_about = None)]
struct Args {
    /// Repository name (e.g., "openjdk/jdk" or "INRIA/spoon")
    #[arg(
        long = "repo",
        short = 'r',
        help = "Repository name in format 'owner/repo'"
    )]
    repo_name: String,

    /// Before commit hash (optional - if not provided, will traverse from after)
    #[arg(long, short = 'b')]
    before: Option<String>,

    /// After commit hash (starting point for traversal)
    #[arg(long, short = 'a')]
    after: String,

    /// Output CSV file path (optional)
    #[arg(long, short = 'o')]
    output: Option<PathBuf>,

    /// Maximum number of commits to process
    #[arg(long, short = 'n', default_value = "100")]
    max_commits: usize,

    /// Processing mode
    #[arg(long, short = 'm', default_value = "incremental")]
    mode: ProcessingMode,

    /// Diff algorithm to use
    #[arg(long, short = 'd', default_value = "gumtree-lazy")]
    algorithm: DiffAlgorithm,

    /// Show detailed progress information
    #[arg(long, short = 'v')]
    verbose: bool,
}

#[derive(ValueEnum, Clone, Debug)]
enum ProcessingMode {
    /// Process commits incrementally (interlace building and diffing)
    Incremental,
    /// Build all commits first, then compute diffs
    Whole,
}

#[derive(ValueEnum, Clone, Debug)]
enum DiffAlgorithm {
    #[value(name = "gt-base")]
    GTBase,
    #[value(name = "gt-lazy")]
    GTLazy,
    #[value(name = "cd-base-deep-label")]
    CDBaseDeepLabel,
    #[value(name = "cd-base-statement")]
    CDBaseStatement,
    #[value(name = "cd-base-deep-statement")]
    CDBaseDeepStatement,
    #[value(name = "cd-opt-deep-label")]
    CDOptDeepLabel,
    #[value(name = "cd-opt-deep-label-cache")]
    CDOptDeepLabelCache,
    #[value(name = "cd-opt-statement")]
    CDOptStatement,
    #[value(name = "cd-opt-deep-statement")]
    CDOptDeepStatement,
    #[value(name = "cd-opt-statement-label-cache")]
    CDOptStatementLabelCache,
    #[value(name = "cd-opt-deep-statement-label-cache")]
    CDOptDeepStatementLabelCache,
}

impl std::fmt::Display for ProcessingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessingMode::Incremental => write!(f, "incremental"),
            ProcessingMode::Whole => write!(f, "whole"),
        }
    }
}

impl std::fmt::Display for DiffAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiffAlgorithm::GTBase => write!(f, "gt-base"),
            DiffAlgorithm::GTLazy => write!(f, "gt-lazy"),
            DiffAlgorithm::CDBaseDeepLabel => write!(f, "cd-base-deep-label"),
            DiffAlgorithm::CDBaseStatement => write!(f, "cd-base-statement"),
            DiffAlgorithm::CDBaseDeepStatement => write!(f, "cd-base-deep-statement"),
            DiffAlgorithm::CDOptDeepLabel => write!(f, "cd-opt-deep-label"),
            DiffAlgorithm::CDOptDeepLabelCache => write!(f, "cd-opt-deep-label-cache"),
            DiffAlgorithm::CDOptStatement => write!(f, "cd-opt-statement"),
            DiffAlgorithm::CDOptDeepStatement => write!(f, "cd-opt-deep-statement"),
            DiffAlgorithm::CDOptStatementLabelCache => write!(f, "cd-opt-statement-label-cache"),
            DiffAlgorithm::CDOptDeepStatementLabelCache => {
                write!(f, "cd-opt-deep-statement-label-cache")
            }
        }
    }
}

struct DiffRunner {
    args: Args,
    preprocessed: PreProcessedRepository,
    output_writer: Option<BufWriter<File>>,
}

impl DiffRunner {
    fn new(args: Args) -> Self {
        let preprocessed = PreProcessedRepository::new(&args.repo_name);
        let output_writer = args.output.as_ref().map(|path| {
            let file = File::create(path).expect("Failed to create output file");
            BufWriter::with_capacity(4 * 8 * 1024, file)
        });

        Self {
            args,
            preprocessed,
            output_writer,
        }
    }

    fn run(mut self) {
        log::info!(
            "Starting benchmark with repository: {}",
            self.args.repo_name
        );
        log::info!("Processing mode: {}", self.args.mode);
        log::info!("Algorithm: {}", self.args.algorithm);

        self.write_csv_header();

        match self.args.mode {
            ProcessingMode::Incremental => self.run_incremental(),
            ProcessingMode::Whole => self.run_whole(),
        }

        if let Some(ref mut writer) = self.output_writer {
            writer.flush().expect("Failed to flush output");
        }

        log::info!("Benchmark completed");
    }

    fn write_csv_header(&mut self) {
        if let Some(ref mut writer) = self.output_writer {
            writeln!(
                writer,
                "input,src_s,dst_s,src_heap,dst_heap,src_t,dst_t,mappings,diff_t,changes"
            )
            .expect("Failed to write CSV header");
            writer.flush().expect("Failed to flush output");
        }
    }

    fn run_incremental(&mut self) {
        let batch_id = format!(
            "{}:({},{})",
            &self.preprocessed.name,
            self.args.before.as_deref().unwrap_or(""),
            &self.args.after
        );

        log::info!("Batch ID: {}", batch_id);

        let progress = ProgressBar::new(self.args.max_commits as u64);
        progress.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .expect("Failed to set progress style")
                .progress_chars("#>-"),
        );

        let current_commit = self.args.after.clone();
        let before_commit = self.args.before.clone().unwrap_or(String::new());

        for _ in 0..self.args.max_commits {
            if current_commit == before_commit {
                log::info!("Reached before commit, stopping");
                break;
            }

            progress.set_message(format!("Processing commit {}", &current_commit[..8]));

            let repo_name = self.preprocessed.name.clone();
            let processing_ordered_commits = self.preprocessed.pre_process_with_limit(
                &mut hyperast_vcs_git::git::fetch_github_repository(&repo_name),
                "",
                &current_commit,
                "",
                2,
            );

            if processing_ordered_commits.len() < 2 {
                log::warn!("Not enough commits found, stopping");
                break;
            }

            let oid_src = processing_ordered_commits[1];
            let oid_dst = processing_ordered_commits[0];

            assert_eq!(current_commit, oid_dst.to_string());

            if self.args.verbose {
                log::info!("Computing diff between {} and {}", oid_src, oid_dst);
            }

            self.compute_and_record_diff(&oid_src, &oid_dst);
            progress.inc(1);
        }
        progress.finish_with_message("Incremental processing completed");
    }

    fn run_whole(&mut self) {
        use hyperast_gen_ts_java::utils::memusage_linux;

        let before_commit = self.args.before.as_deref().unwrap_or("");
        let batch_id = format!(
            "{}:({},{})",
            &self.preprocessed.name, before_commit, &self.args.after
        );

        log::info!("Batch ID: {}", batch_id);

        // Pre-process all commits
        let memory_before = memusage_linux();
        let processing_ordered_commits = self.preprocessed.pre_process_with_limit(
            &mut hyperast_vcs_git::git::fetch_github_repository(&self.preprocessed.name),
            before_commit,
            &self.args.after,
            "",
            self.args.max_commits.min(10), // Limit to prevent excessive memory usage
        );
        let hyperast_memory = memusage_linux() - memory_before;

        log::info!("HyperAST memory usage: {} bytes", hyperast_memory);
        log::info!(
            "Total commits processed: {}",
            processing_ordered_commits.len()
        );

        // Purge caches to measure their size
        let memory_before_purge = memusage_linux();
        self.preprocessed.purge_caches();
        let cache_memory = memory_before_purge - memusage_linux();
        log::info!("Cache memory freed: {} bytes", cache_memory);

        // Compute diffs with progress tracking
        let total_diffs = processing_ordered_commits.len().saturating_sub(1);
        let progress = ProgressBar::new(total_diffs as u64);
        progress.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .expect("Failed to set progress style")
                .progress_chars("#>-"),
        );

        let commits_to_process: Vec<_> = processing_ordered_commits.windows(2).collect();
        for (i, window) in commits_to_process.iter().enumerate() {
            let oid_src = window[0];
            let oid_dst = window[1];

            progress.set_message(format!(
                "Diff {}: {}..{}",
                i + 1,
                &oid_src.to_string()[..8],
                &oid_dst.to_string()[..8]
            ));

            if self.args.verbose {
                log::info!("Computing diff between {} and {}", oid_src, oid_dst);
            }

            self.compute_and_record_diff(&oid_src, &oid_dst);
            progress.inc(1);
        }

        progress.finish_with_message("Whole processing completed");
    }

    fn compute_and_record_diff(&mut self, oid_src: &Oid, oid_dst: &Oid) {
        use hyper_diff::algorithms::ComputeTime;
        use hyperast::types::WithStats;
        use hyperast_gen_ts_java::utils::memusage_linux;

        let stores = &self.preprocessed.processor.main_stores;

        // Get source commit info
        let commit_src = self
            .preprocessed
            .commits
            .get_key_value(oid_src)
            .expect("Source commit not found");
        let time_src = commit_src.1.processing_time();
        let src_tr = commit_src.1.ast_root;
        let src_size = stores.node_store.resolve(src_tr).size();

        // Get destination commit info
        let commit_dst = self
            .preprocessed
            .commits
            .get_key_value(oid_dst)
            .expect("Destination commit not found");
        let time_dst = commit_dst.1.processing_time();
        let dst_tr = commit_dst.1.ast_root;
        let dst_size = stores.node_store.resolve(dst_tr).size();

        let hyperast = hyperast_vcs_git::no_space::as_nospaces2(stores);

        // Compute diff based on selected algorithm
        let memory_before = memusage_linux();
        let diff_result = match self.args.algorithm {
            DiffAlgorithm::GTBase => {
                hyper_diff::algorithms::gumtree::diff(&hyperast, &src_tr, &dst_tr)
            }
            DiffAlgorithm::GTLazy => {
                hyper_diff::algorithms::gumtree_lazy::diff(&hyperast, &src_tr, &dst_tr)
            }
            DiffAlgorithm::CDBaseDeepLabel => {
                hyper_diff::algorithms::change_distiller_optimized::diff_optimized(
                    &hyperast,
                    &src_tr,
                    &dst_tr,
                    OptimizedDiffConfig::baseline(),
                )
            }
            DiffAlgorithm::CDBaseStatement => {
                hyper_diff::algorithms::change_distiller_optimized::diff_optimized(
                    &hyperast,
                    &src_tr,
                    &dst_tr,
                    OptimizedDiffConfig::baseline().with_statement_level_iteration(true),
                )
            }
            DiffAlgorithm::CDBaseDeepStatement => {
                hyper_diff::algorithms::change_distiller_optimized::diff_optimized(
                    &hyperast,
                    &src_tr,
                    &dst_tr,
                    OptimizedDiffConfig::baseline()
                        .with_statement_level_iteration(true)
                        .with_label_caching(true)
                        .with_deep_leaves(true),
                )
            }
            DiffAlgorithm::CDOptDeepLabel => {
                hyper_diff::algorithms::change_distiller_optimized::diff_optimized(
                    &hyperast,
                    &src_tr,
                    &dst_tr,
                    OptimizedDiffConfig::optimized(),
                )
            }
            DiffAlgorithm::CDOptDeepLabelCache => {
                hyper_diff::algorithms::change_distiller_optimized::diff_optimized(
                    &hyperast,
                    &src_tr,
                    &dst_tr,
                    OptimizedDiffConfig::optimized().with_label_caching(true),
                )
            }
            DiffAlgorithm::CDOptStatement => {
                hyper_diff::algorithms::change_distiller_optimized::diff_optimized(
                    &hyperast,
                    &src_tr,
                    &dst_tr,
                    OptimizedDiffConfig::optimized().with_statement_level_iteration(true),
                )
            }
            DiffAlgorithm::CDOptDeepStatement => {
                hyper_diff::algorithms::change_distiller_optimized::diff_optimized(
                    &hyperast,
                    &src_tr,
                    &dst_tr,
                    OptimizedDiffConfig::optimized()
                        .with_statement_level_iteration(true)
                        .with_deep_leaves(true),
                )
            }
            DiffAlgorithm::CDOptStatementLabelCache => {
                hyper_diff::algorithms::change_distiller_optimized::diff_optimized(
                    &hyperast,
                    &src_tr,
                    &dst_tr,
                    OptimizedDiffConfig::optimized()
                        .with_statement_level_iteration(true)
                        .with_label_caching(true),
                )
            }
            DiffAlgorithm::CDOptDeepStatementLabelCache => {
                hyper_diff::algorithms::change_distiller_optimized::diff_optimized(
                    &hyperast,
                    &src_tr,
                    &dst_tr,
                    OptimizedDiffConfig::optimized()
                        .with_statement_level_iteration(true)
                        .with_label_caching(true)
                        .with_deep_leaves(true),
                )
            }
        };
        let diff_memory = memusage_linux() - memory_before;

        let summarized_result = diff_result.summarize();
        let total_diff_time = summarized_result.time();

        if self.args.verbose {
            log::info!("Diff memory usage: {} bytes", diff_memory);
            log::debug!("Diff summary: {:?}", summarized_result);
        }

        // Write results to CSV
        if let Some(ref mut writer) = self.output_writer {
            writeln!(
                writer,
                "{}/{},{},{},{},{},{},{},{},{},{}",
                oid_src,
                oid_dst,
                src_size,
                dst_size,
                Into::<isize>::into(&commit_src.1.memory_used()),
                Into::<isize>::into(&commit_dst.1.memory_used()),
                time_src,
                time_dst,
                summarized_result.mappings,
                total_diff_time,
                summarized_result.actions.map_or(-1, |x| x as isize),
            )
            .expect("Failed to write CSV row");
            writer.flush().expect("Failed to flush output");
        }
    }
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    if args.verbose {
        log::info!("Running with arguments: {:#?}", args);
    }

    let runner = DiffRunner::new(args);
    runner.run();
}
