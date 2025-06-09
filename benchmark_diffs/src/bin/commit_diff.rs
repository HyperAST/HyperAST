use clap::{Parser, ValueEnum};
use hyper_diff::OptimizedDiffConfig;
use hyperast_vcs_git::{
    git::Oid, multi_preprocessed::PreProcessedRepositories, processing::RepoConfig,
};
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    time::Instant,
};

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Repository name (e.g., "openjdk/jdk" or "INRIA/spoon")
    #[arg(
        long = "repo",
        short = 'r',
        help = "Repository name in format 'owner/repo'"
    )]
    repo_name: String,

    /// Before commit hash (starting point for traversal, if not provided, will traverse all commits)
    #[arg(long, short = 'b', default_value = "")]
    before: String,

    /// After commit hash (optional - if not provided, will traverse from before)
    #[arg(long, short = 'a', default_value = "")]
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
    #[arg(long, short = 'd', default_value = "gt-lazy")]
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

struct DiffProcessor {
    args: Args,
    preprocessed: PreProcessedRepositories,
    repo: hyperast_vcs_git::processing::ConfiguredRepo2,
}

impl DiffProcessor {
    fn new(args: Args) -> anyhow::Result<Self> {
        let repo_parts: Vec<&str> = args.repo_name.split('/').collect();
        if repo_parts.len() != 2 {
            anyhow::bail!("Repository name must be in format 'owner/repo'");
        }

        let mut preprocessed = PreProcessedRepositories::default();
        let user = repo_parts[0];
        let name = repo_parts[1];
        let repo = hyperast_vcs_git::git::Forge::Github.repo(user, name);
        let repo = preprocessed.register_config(repo, RepoConfig::JavaMaven);
        let repo = repo.fetch();

        Ok(Self {
            args,
            preprocessed,
            repo,
        })
    }

    fn run(&mut self) -> anyhow::Result<()> {
        self.setup_logging();

        match self.args.mode {
            ProcessingMode::Incremental => {
                self.process_incremental()?;
            }
            ProcessingMode::Whole => {
                self.process_whole()?;
            }
        }

        Ok(())
    }

    fn setup_logging(&self) {
        if self.args.verbose {
            env_logger::Builder::from_default_env()
                .filter_level(log::LevelFilter::Info)
                .init();
        } else {
            env_logger::Builder::from_default_env()
                .filter_level(log::LevelFilter::Warn)
                .init();
        }
    }

    fn create_output_writer(&self) -> anyhow::Result<Option<BufWriter<File>>> {
        if let Some(output_path) = &self.args.output {
            let file = File::create(output_path)?;
            let mut buf = BufWriter::with_capacity(4 * 8 * 1024, file);
            writeln!(
                buf,
                "input,src_s,dst_s,src_heap,dst_heap,src_t,dst_t,mappings,diff_t,changes"
            )?;
            buf.flush()?;
            Ok(Some(buf))
        } else {
            Ok(None)
        }
    }

    /// Process commits incrementally (interlace building and diffing)
    fn process_incremental(&mut self) -> anyhow::Result<()> {
        let batch_id = format!(
            "{}:({},{})",
            &self.repo.spec.url(),
            self.args.before,
            self.args.after
        );
        log::info!("Processing batch: {}", batch_id);

        let mut output_writer = self.create_output_writer()?;

        let progress_bar = ProgressBar::new(self.args.max_commits as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} commits ({eta})")
                .expect("Failed to set progress bar template")
                .progress_chars("#>-"),
        );

        let mut curr = self.args.after.clone();

        for i in 0..self.args.max_commits {
            if curr == self.args.before {
                break;
            }

            let processing_ordered_commits = self
                .preprocessed
                .processor
                .pre_process_with_limit(&self.repo, "", &curr, 2)?;

            if processing_ordered_commits.len() < 2 {
                log::warn!("Not enough commits found for diff");
                break;
            }

            let oid_src = processing_ordered_commits[1];
            let oid_dst = processing_ordered_commits[0];

            progress_bar.set_message(format!("Processing {}/{}", oid_src, oid_dst));

            self.process_diff_pair(&oid_src, &oid_dst, &mut output_writer)?;

            curr = oid_src.to_string();
            progress_bar.inc(1);
        }

        progress_bar.finish_with_message("Incremental processing completed");
        self.log_memory_usage();

        Ok(())
    }

    /// Build all commits first, then compute diffs
    fn process_whole(&mut self) -> anyhow::Result<()> {
        let batch_id = format!(
            "{}:({},{})",
            &self.repo.spec.url(),
            self.args.before,
            self.args.after
        );

        let start_time = Instant::now();
        use hyperast_gen_ts_java::utils::memusage_linux;
        let mu = memusage_linux();

        let processing_ordered_commits = self.preprocessed.processor.pre_process_with_limit(
            &self.repo,
            &self.args.before,
            &self.args.after,
            self.args.max_commits,
        )?;

        let hyperast_size = memusage_linux() - mu;
        log::info!(
            "HyperAST built in {:?}, size: {} KB",
            start_time.elapsed(),
            hyperast_size
        );
        log::info!("Processing batch: {}", batch_id);
        log::info!("Found {} commits", processing_ordered_commits.len());

        let mut output_writer = self.create_output_writer()?;

        // Calculate total number of diffs to process
        let total_diffs = processing_ordered_commits.len().saturating_sub(1);
        let progress_bar = ProgressBar::new(total_diffs as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} diffs ({eta})")
                .expect("Failed to set progress bar template")
                .progress_chars("#>-"),
        );

        for i in 0..processing_ordered_commits.len().saturating_sub(1) {
            let oid_src = &processing_ordered_commits[i];
            let oid_dst = &processing_ordered_commits[i + 1];

            progress_bar.set_message(format!("Diffing {}/{}", oid_src, oid_dst));

            self.process_diff_pair(oid_src, oid_dst, &mut output_writer)?;

            progress_bar.inc(1);
        }

        progress_bar.finish_with_message("Whole processing completed");
        self.log_memory_usage();

        Ok(())
    }

    fn process_diff_pair(
        &mut self,
        oid_src: &Oid,
        oid_dst: &Oid,
        output_writer: &mut Option<BufWriter<File>>,
    ) -> anyhow::Result<()> {
        use hyperast::types::WithStats;
        use hyperast_gen_ts_java::utils::memusage_linux;

        let stores = &self.preprocessed.processor.main_stores;

        let commit_src = self
            .preprocessed
            .get_commit(&self.repo.config, oid_src)
            .ok_or_else(|| anyhow::anyhow!("Failed to get commit {}", oid_src))?;
        let time_src = commit_src.processing_time();
        let src_tr = commit_src.ast_root;
        let src_s = stores.node_store.resolve(src_tr).size();

        let commit_dst = self
            .preprocessed
            .get_commit(&self.repo.config, oid_dst)
            .ok_or_else(|| anyhow::anyhow!("Failed to get commit {}", oid_dst))?;
        let time_dst = commit_dst.processing_time();
        let dst_tr = commit_dst.ast_root;
        let dst_s = stores.node_store.resolve(dst_tr).size();

        let hyperast = hyperast_vcs_git::no_space::as_nospaces2(stores);

        let mu = memusage_linux();
        let diff_result = match self.args.algorithm {
            DiffAlgorithm::GTBase => Box::new(hyper_diff::algorithms::gumtree::diff(
                &hyperast, &src_tr, &dst_tr,
            )),
            DiffAlgorithm::GTLazy => Box::new(hyper_diff::algorithms::gumtree_lazy::diff(
                &hyperast, &src_tr, &dst_tr,
            )),
            DiffAlgorithm::CDBaseDeepLabel => Box::new(
                hyper_diff::algorithms::change_distiller_optimized::diff_optimized(
                    &hyperast,
                    &src_tr,
                    &dst_tr,
                    OptimizedDiffConfig::baseline(),
                ),
            ),
            DiffAlgorithm::CDBaseStatement => Box::new(
                hyper_diff::algorithms::change_distiller_optimized::diff_optimized(
                    &hyperast,
                    &src_tr,
                    &dst_tr,
                    OptimizedDiffConfig::baseline().with_statement_level_iteration(),
                ),
            ),
            DiffAlgorithm::CDBaseDeepStatement => Box::new(
                hyper_diff::algorithms::change_distiller_optimized::diff_optimized(
                    &hyperast,
                    &src_tr,
                    &dst_tr,
                    OptimizedDiffConfig::baseline()
                        .with_statement_level_iteration()
                        .with_label_caching()
                        .with_deep_leaves(),
                ),
            ),
            DiffAlgorithm::CDOptDeepLabel => Box::new(
                hyper_diff::algorithms::change_distiller_optimized::diff_optimized(
                    &hyperast,
                    &src_tr,
                    &dst_tr,
                    OptimizedDiffConfig::optimized(),
                ),
            ),
            DiffAlgorithm::CDOptDeepLabelCache => Box::new(
                hyper_diff::algorithms::change_distiller_optimized::diff_optimized(
                    &hyperast,
                    &src_tr,
                    &dst_tr,
                    OptimizedDiffConfig::optimized().with_label_caching(),
                ),
            ),
            DiffAlgorithm::CDOptStatement => Box::new(
                hyper_diff::algorithms::change_distiller_optimized::diff_optimized(
                    &hyperast,
                    &src_tr,
                    &dst_tr,
                    OptimizedDiffConfig::optimized().with_statement_level_iteration(),
                ),
            ),
            DiffAlgorithm::CDOptDeepStatement => Box::new(
                hyper_diff::algorithms::change_distiller_optimized::diff_optimized(
                    &hyperast,
                    &src_tr,
                    &dst_tr,
                    OptimizedDiffConfig::optimized()
                        .with_statement_level_iteration()
                        .with_deep_leaves(),
                ),
            ),
            DiffAlgorithm::CDOptStatementLabelCache => Box::new(
                hyper_diff::algorithms::change_distiller_optimized::diff_optimized(
                    &hyperast,
                    &src_tr,
                    &dst_tr,
                    OptimizedDiffConfig::optimized()
                        .with_statement_level_iteration()
                        .with_label_caching(),
                ),
            ),
            DiffAlgorithm::CDOptDeepStatementLabelCache => Box::new(
                hyper_diff::algorithms::change_distiller_optimized::diff_optimized(
                    &hyperast,
                    &src_tr,
                    &dst_tr,
                    OptimizedDiffConfig::optimized()
                        .with_statement_level_iteration()
                        .with_label_caching()
                        .with_deep_leaves(),
                ),
            ),
        };
        let summarized = diff_result.summarize();

        use hyper_diff::algorithms::ComputeTime;
        let total_diff_time: f64 = summarized.time();
        let diff_memory = memusage_linux() - mu;

        if self.args.verbose {
            log::info!(
                "Diff computed - mappings: {}, time: {:.3}s, memory: {} KB",
                summarized.mappings,
                total_diff_time,
                diff_memory
            );
        }

        if let Some(writer) = output_writer {
            writeln!(
                writer,
                "{}/{},{},{},{},{},{},{},{},{},{}",
                oid_src,
                oid_dst,
                src_s,
                dst_s,
                Into::<isize>::into(&commit_src.memory_used()),
                Into::<isize>::into(&commit_dst.memory_used()),
                time_src,
                time_dst,
                summarized.mappings,
                total_diff_time,
                summarized.actions.map_or(-1, |x| x as isize),
            )?;
            writer.flush()?;
        }

        Ok(())
    }

    fn log_memory_usage(&mut self) {
        use hyperast_gen_ts_java::utils::memusage_linux;
        let mu = memusage_linux();
        drop(&mut self.preprocessed);
        let freed_memory = mu - memusage_linux();
        log::info!("Memory freed: {} KB", freed_memory);
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.verbose {
        println!("Starting diff processing with configuration:");
        println!("  Repository: {}", args.repo_name);
        println!(
            "  Before: {}",
            if args.before.is_empty() {
                "HEAD"
            } else {
                &args.before
            }
        );
        println!(
            "  After: {}",
            if args.after.is_empty() {
                "latest"
            } else {
                &args.after
            }
        );
        println!("  Mode: {:?}", args.mode);
        println!("  Algorithm: {:?}", args.algorithm);
        println!("  Max commits: {}", args.max_commits);
        if let Some(ref output) = args.output {
            println!("  Output: {}", output.display());
        }
        println!();
    }

    let mut processor = DiffProcessor::new(args)?;
    processor.run()?;

    Ok(())
}
