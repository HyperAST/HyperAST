use std::path::Path;
use std::time::Instant;
use tpe::{TpeOptimizer, range, categorical_range};
use rand::SeedableRng as _;
use crate::run_diff::{run_diff_file, run_diff_folder};

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