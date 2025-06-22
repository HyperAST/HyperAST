use std::cmp::max;
use std::io::Write;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::Instant;
use tpe::{TpeOptimizer, range, categorical_range};
use rand::SeedableRng as _;
use hyperast_benchmark_diffs::preprocess_repo::parse_repo;
use hyperast_vcs_git::multi_preprocessed::PreProcessedRepositories;
use crate::run_diff::{run_diff_file, run_diff_folder, run_diff_trees};

fn run_tpe_local(path: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("datasets/defects4j");

    let before = root.join("before").join(&path);
    let after = root.join("after").join(&path);
    
    let algorithms = vec!["greedy", "hybrid", "simple"];
    let mut opt_algorithm = TpeOptimizer::new(
        tpe::parzen_estimator(),
        categorical_range(3).unwrap()
    );
    let mut opt_max_size = TpeOptimizer::new(
        tpe::parzen_estimator(),
        tpe::categorical_range(1000).unwrap()
    );
    let mut opt_sim_threshold = TpeOptimizer::new(
        tpe::parzen_estimator(),
        range(0.1, 1.0).unwrap()
    );

    let mut best_configuration = (Some("simple"), None, None);
    let mut min_score = run_diff_file(&before, &after, "simple", 0, 0.0);
    
    let mut rng = rand::rngs::StdRng::from_seed(Default::default());
    for _ in 0..100 {
        let algorithm = opt_algorithm.ask(&mut rng).unwrap();
        let max_size = opt_max_size.ask(&mut rng).unwrap();
        let sim_threshold = opt_sim_threshold.ask(&mut rng).unwrap();

        println!("Running with algorithm = {:?}, max_size = {:?}, sim_threshold = {:?}", algorithm, max_size, sim_threshold);
        let v = run_diff_file(
            &before, 
            &after, 
            algorithms[algorithm as usize],
            max_size as usize, 
            sim_threshold,
        );
        opt_algorithm.tell(algorithm, v as f64).unwrap();
        opt_max_size.tell(max_size, v as f64).unwrap();
        opt_sim_threshold.tell(sim_threshold, v as f64).unwrap();

        if v < min_score {
            min_score = v;
            best_configuration = (Some(algorithms[algorithm as usize]), Some(max_size), Some(sim_threshold));
        }
    }

    println!(
        "Grid search complete. Best configuration = {:?}, Minimum score = {}",
        best_configuration, min_score
    );
}

fn run_tpe_repo(repo_user: &str, repo_name: &str, before: &str, after: &str, iterations: usize) -> ((Option<&'static str>, Option<f64>, Option<f64>), usize) {
    let mut repositories = PreProcessedRepositories::default();
    let (stores, src_tr, dst_tr) = parse_repo(
        &mut repositories,
        repo_user,
        repo_name,
        hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        before,
        after,
    );

    let algorithms = vec!["greedy", "hybrid", "simple"];
    let mut opt_algorithm = TpeOptimizer::new(
        tpe::parzen_estimator(),
        categorical_range(3).unwrap()
    );
    let mut opt_max_size = TpeOptimizer::new(
        tpe::parzen_estimator(),
        tpe::categorical_range(1000).unwrap()
    );
    let mut opt_sim_threshold = TpeOptimizer::new(
        tpe::parzen_estimator(),
        range(0.1, 1.0).unwrap()
    );

    let mut best_configuration = (Some("simple"), None, None);
    let mut min_score = run_diff_trees(&stores, &src_tr, &dst_tr, "simple", 0, 0.0);

    let mut rng = rand::rngs::StdRng::from_seed(Default::default());
    for _ in 0..iterations {
        let algorithm = opt_algorithm.ask(&mut rng).unwrap();
        let max_size = opt_max_size.ask(&mut rng).unwrap();
        let sim_threshold = opt_sim_threshold.ask(&mut rng).unwrap();

        println!("Running with algorithm = {:?}, max_size = {:?}, sim_threshold = {:?}", algorithm, max_size, sim_threshold);
        let v = run_diff_trees(
            &stores,
            &src_tr,
            &dst_tr,
            algorithms[algorithm as usize],
            max_size as usize,
            sim_threshold,
        );
        opt_algorithm.tell(algorithm, v as f64).unwrap();
        opt_max_size.tell(max_size, v as f64).unwrap();
        opt_sim_threshold.tell(sim_threshold, v as f64).unwrap();

        if v < min_score {
            min_score = v;
            best_configuration = (Some(algorithms[algorithm as usize]), Some(max_size), Some(sim_threshold));
        }
    }

    println!(
        "Grid search complete. Best configuration = {:?}, Minimum score = {}",
        best_configuration, min_score
    );
    
    return (best_configuration, min_score)
}

fn run_tpe_repo_maxsize_only(repo_user: &str, repo_name: &str, before: &str, after: &str, iterations: usize) -> (usize, usize) {
    let mut repositories = PreProcessedRepositories::default();
    let (stores, src_tr, dst_tr) = parse_repo(
        &mut repositories,
        repo_user,
        repo_name,
        hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        before,
        after,
    );

    let mut opt_max_size = TpeOptimizer::new(
        tpe::parzen_estimator(),
        tpe::categorical_range(1000).unwrap()
    );

    let mut best_configuration: usize = 0;
    let mut min_score = run_diff_trees(&stores, &src_tr, &dst_tr, "simple", 0, 0.0);

    let mut rng = rand::rngs::StdRng::from_seed(Default::default());
    for _ in 0..iterations {
        let max_size = opt_max_size.ask(&mut rng).unwrap();

        println!("Running with max_size = {:?}", max_size);
        let v = run_diff_trees(
            &stores,
            &src_tr,
            &dst_tr,
            "hybrid",
            max_size as usize,
            0.0f64,
        );
        opt_max_size.tell(max_size, v as f64).unwrap();

        if v < min_score {
            min_score = v;
            best_configuration = max_size as usize
        }
    }

    println!(
        "Grid search complete. Best configuration = {:?}, Minimum score = {}",
        best_configuration, min_score
    );

    return (best_configuration, min_score)
}

#[test]
fn run_tpe_local_1() {
    let now = Instant::now();
    run_tpe_local("Cli/13/src_java_org_apache_commons_cli2_WriteableCommandLine.java");
    dbg!(now.elapsed().as_secs_f64());
}

#[test]
fn run_tpe_local_2() {
    let now = Instant::now();
    run_tpe_local("Chart/10/source_org_jfree_chart_imagemap_StandardToolTipTagFragmentGenerator.java");
    dbg!(now.elapsed().as_secs_f64());
}

#[test]
fn run_tpe_local_3() {
    let now = Instant::now();
    run_tpe_local("Mockito/32/src_org_mockito_internal_configuration_SpyAnnotationEngine.java");
    dbg!(now.elapsed().as_secs_f64());
}

#[test]
fn run_tpe_local_4() {
    let now = Instant::now();
    run_tpe_local("JxPath/7/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationLessThanOrEqual.java");
    dbg!(now.elapsed().as_secs_f64());
}

#[test]
fn run_tpe_repo_1() {
    let now = Instant::now();
    run_tpe_repo(
        "apache", 
        "maven",
        "a02834611bad3442ad073b10f1dee2322916f1f3", 
        "c3cf29438e3d65d6ee5c5726f8611af99d9a649a",
        100
    );
    dbg!(now.elapsed().as_secs_f64());
}

#[test]
fn run_tpe_repo_2() {
    let now = Instant::now();
    run_tpe_repo(
        "apache",
        "maven",
        "a02834611bad3442ad073b10f1dee2322916f1f3",
        "c3cf29438e3d65d6ee5c5726f8611af99d9a649a",
        10,
    );
    dbg!(now.elapsed().as_secs_f64());
}

#[test]
fn run_tpe_repo_3() {
    let now = Instant::now();
    run_tpe_repo(
        "quarkus",
        "quarkusio",
        "a0659dba3ff3df590088262f42329efa0b4b30e9",
        "be1bda0f121ac24cb789b103e216151b53c0a076",
        10,
    );
    dbg!(now.elapsed().as_secs_f64());
}

#[test]
fn run_tpe_repo_4() {
    let now = Instant::now();
    run_tpe_repo(
        "qos-ch",
        "slf4j",
        "03aa6b915a82a037d2936ca0b166626d32e9a1f6",
        "0def25ebfa0e546525fb90aa8d5946d16f26c561",
        10,
    );
    dbg!(now.elapsed().as_secs_f64());
}

struct BenchmarkItem {
    repositories: PreProcessedRepositories, // todo: ugly workaround to avoid issues with borrowing
    repo_user: &'static str,
    repo_name: &'static str,
    config: hyperast_vcs_git::processing::RepoConfig,
    before: &'static str,
    after: &'static str,
}

fn run_tpe_repo_multiple(dataset: Vec<BenchmarkItem>) {

    
    // let mut dataset = vec![
        // BenchmarkItem {
        //     repositories: PreProcessedRepositories::default(),
        //     repo_user: "google",
        //     repo_name: "gson",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     before: "de140ed74fcf9894709286d6cec5c405034d9234",
        //     after: "f75118e27c65409365ec8c8f32b49dfddfaf4186"
        // },
        // BenchmarkItem {
        //     repositories: PreProcessedRepositories::default(),
        //     repo_user: "google",
        //     repo_name: "gson",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     before: "3c9abdeea9afd3a3d7a7f99658455ceb4a994029",
        //     after: "259c477cecaea8e73cd19e5207ba63edc04157da"
        // },
        // BenchmarkItem {
        //     repositories: PreProcessedRepositories::default(),
        //     repo_user: "qos-ch",
        //     repo_name: "slf4j",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     before: "0def25ebfa0e546525fb90aa8d5946d16f26c561",
        //     after: "69c333de280100f7dc99ee00e302192690fcc761"
        // },
        // BenchmarkItem {
        //     repositories: PreProcessedRepositories::default(),
        //     repo_user: "qos-ch",
        //     repo_name: "slf4j",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     before: "c2c29fec98e05d4410c305edf06965a79a69a4f6",
        //     after: "0597225b5b6326dbd6bf6f0038d8b6cf5c8ca377"
        // },
        // BenchmarkItem {
        //     repositories: PreProcessedRepositories::default(),
        //     repo_user: "alibaba",
        //     repo_name: "arthas",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     before: "b0990518e5428204e133944a3509b9751312757a",
        //     after: "1140fa0d996f95659875d51afba6793c524beb79"
        // },
        // BenchmarkItem {
        //     repositories: PreProcessedRepositories::default(),
        //     repo_user: "alibaba",
        //     repo_name: "arthas",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     before: "0d6c1a63eb308531780ecf85f78e67f18303815c",
        //     after: "63ee8dfb19e94bcf867f55190fd8b01fd399afb2"
        // },
        // BenchmarkItem {
        //     repositories: PreProcessedRepositories::default(),
        //     repo_user: "FasterXML",
        //     repo_name: "jackson-core",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     before: "373f108a16e3f315e9df9eaecb482e43f9953621",
        //     after: "0d9823619c4daa3f6aa9ee0d615f140978bcc51d"
        // },
        // BenchmarkItem {
        //     repositories: PreProcessedRepositories::default(),
        //     repo_user: "FasterXML",
        //     repo_name: "jackson-core",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     before: "2272fcf675c3936568c855d2f8b3da58bb7713af",
        //     after: "6d2236ea9127757cbb85a6b60b42b5a597205d19"
        // },
        // BenchmarkItem {
        //     repositories: PreProcessedRepositories::default(),
        //     repo_user: "apache",
        //     repo_name: "skywalking",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     before: "57746c24d3fd831835a3709ea3078fa26928f54e",
        //     after: "39508f81c8f8e04e86b670fb3877be28eaf5f01f"
        // },
        // BenchmarkItem {
        //     repositories: PreProcessedRepositories::default(),
        //     repo_user: "apache",
        //     repo_name: "skywalking",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     before: "47ce2720b9be6af391138c2b84c4ec63c454a3b3",
        //     after: "43d79d9fec224036cc3cbc7185c9faa7ecd4838c"
        // },
    // ];

    let mut buf_perfs = BufWriter::with_capacity(4 * 8 * 1024, File::create("/tmp/repo_tpe.csv").unwrap());
    writeln!(
        buf_perfs,
        "repo_user,repo_name,before,after,max_size,min_score,runtime",
    )
        .unwrap();

    for iterations in vec![10, 25, 50, 100] {
        for item in &dataset {
            println!("Running for {}/{} {}/{} with {} iterations", item.repo_user, item.repo_name, item.before, item.after, iterations);
            
            let now = Instant::now();
    
            let (max_size, min_score) = run_tpe_repo_maxsize_only(
                item.repo_user,
                item.repo_name,
                item.before,
                item.after,
                iterations
            );
            let elapsed = now.elapsed().as_secs_f64();
    
            writeln!(buf_perfs, "{},{},{},{},{},{},{}",
                     item.repo_user,
                     item.repo_name,
                     item.before,
                     item.after,
                     max_size,
                     min_score,
                     elapsed).unwrap();
            buf_perfs.flush().unwrap();
        }
    }
    
    
}

#[test]
fn run_tpe_random_commits() {
    run_tpe_repo_multiple(vec![
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "google",
            repo_name: "gson",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "de140ed74fcf9894709286d6cec5c405034d9234",
            after: "f75118e27c65409365ec8c8f32b49dfddfaf4186"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "google",
            repo_name: "gson",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "3c9abdeea9afd3a3d7a7f99658455ceb4a994029",
            after: "259c477cecaea8e73cd19e5207ba63edc04157da"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "qos-ch",
            repo_name: "slf4j",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "0def25ebfa0e546525fb90aa8d5946d16f26c561",
            after: "69c333de280100f7dc99ee00e302192690fcc761"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "qos-ch",
            repo_name: "slf4j",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "c2c29fec98e05d4410c305edf06965a79a69a4f6",
            after: "0597225b5b6326dbd6bf6f0038d8b6cf5c8ca377"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "alibaba",
            repo_name: "arthas",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "b0990518e5428204e133944a3509b9751312757a",
            after: "1140fa0d996f95659875d51afba6793c524beb79"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "alibaba",
            repo_name: "arthas",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "0d6c1a63eb308531780ecf85f78e67f18303815c",
            after: "63ee8dfb19e94bcf867f55190fd8b01fd399afb2"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "FasterXML",
            repo_name: "jackson-core",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "373f108a16e3f315e9df9eaecb482e43f9953621",
            after: "0d9823619c4daa3f6aa9ee0d615f140978bcc51d"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "FasterXML",
            repo_name: "jackson-core",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "2272fcf675c3936568c855d2f8b3da58bb7713af",
            after: "6d2236ea9127757cbb85a6b60b42b5a597205d19"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "apache",
            repo_name: "skywalking",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "57746c24d3fd831835a3709ea3078fa26928f54e",
            after: "39508f81c8f8e04e86b670fb3877be28eaf5f01f"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "apache",
            repo_name: "skywalking",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "47ce2720b9be6af391138c2b84c4ec63c454a3b3",
            after: "43d79d9fec224036cc3cbc7185c9faa7ecd4838c"
        },
    ])
}


#[test]
fn run_tpe_gson() {
    run_tpe_repo_multiple(vec![
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "google",
            repo_name: "gson",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "dd2fe59c0d3390b2ad3dd365ed6938a5c15844cb",
            after: "330c613e7596e801df055bb60c1f6e41b22fbdcd"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "google",
            repo_name: "gson",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "330c613e7596e801df055bb60c1f6e41b22fbdcd",
            after: "de140ed74fcf9894709286d6cec5c405034d9234"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "google",
            repo_name: "gson",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "de140ed74fcf9894709286d6cec5c405034d9234",
            after: "f75118e27c65409365ec8c8f32b49dfddfaf4186"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "google",
            repo_name: "gson",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "f75118e27c65409365ec8c8f32b49dfddfaf4186",
            after: "3c9abdeea9afd3a3d7a7f99658455ceb4a994029"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "google",
            repo_name: "gson",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "3c9abdeea9afd3a3d7a7f99658455ceb4a994029",
            after: "259c477cecaea8e73cd19e5207ba63edc04157da"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "google",
            repo_name: "gson",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "259c477cecaea8e73cd19e5207ba63edc04157da",
            after: "5206d803dab1655472d655a0b6710b4622039a22"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "google",
            repo_name: "gson",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "5206d803dab1655472d655a0b6710b4622039a22",
            after: "257bee9eff81889893ca02a6925aa1b620378e9e"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "google",
            repo_name: "gson",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "257bee9eff81889893ca02a6925aa1b620378e9e",
            after: "63d74b39400be6e2a244a227820fa9d984a493e9"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "google",
            repo_name: "gson",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "00ae39775708147e115512be5d4f92bee02e9b89",
            after: "0eec6f35c59f164ee49ef175e38896db20296b44"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "google",
            repo_name: "gson",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "0eec6f35c59f164ee49ef175e38896db20296b44",
            after: "4e65e6ab368d92638b4ca04521958dbf1d3753e7"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "google",
            repo_name: "gson",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "4e65e6ab368d92638b4ca04521958dbf1d3753e7",
            after: "6010131366fa3f72c3f07151bff6d1c4e1a7f6e0"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "google",
            repo_name: "gson",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "6010131366fa3f72c3f07151bff6d1c4e1a7f6e0",
            after: "bfe0fd58e3efc9f54938a87b955bbfc42dfd45e1"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "google",
            repo_name: "gson",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "bfe0fd58e3efc9f54938a87b955bbfc42dfd45e1",
            after: "6ed64ca3a8990c60b758d838e16713b2a1e0f461"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "google",
            repo_name: "gson",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "6ed64ca3a8990c60b758d838e16713b2a1e0f461",
            after: "0074376f7e605700f218ff7362dd45a690926074"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "google",
            repo_name: "gson",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "0074376f7e605700f218ff7362dd45a690926074",
            after: "45e5e141b1a39926e6b43c6eacd3dc0280321f71"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "google",
            repo_name: "gson",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "45e5e141b1a39926e6b43c6eacd3dc0280321f71",
            after: "c6d44259b53a9b2756b5767b843d15e8acacaa31"
        }
        
    ])
}