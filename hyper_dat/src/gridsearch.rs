use std::path::Path;
use std::time::Instant;
use crate::run_diff::{run_diff_folder, run_diff_file};

fn run_grid_search_defects4j() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("datasets/defects4j");

    let mut best_configuration = (None, None, None);
    let mut min_score = usize::MAX;


    let max_size_values = vec![100, 200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100, 1200, 1300, 1400, 1500, 1600, 1700, 1800, 1900, 2000];
    let sim_threshold_values = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

    // Greedy
    for &max_size in &max_size_values {
        for &sim_threshold in &sim_threshold_values {
            let result = run_diff_folder(&root, "greedy", max_size, sim_threshold);

            println!("greedy, max_size = {}, sim_threshold = {}, score = {}", max_size, sim_threshold, result);

            if result < min_score {
                min_score = result;
                best_configuration = (Some("greedy"), Some(max_size), Some(sim_threshold));
            }
        }
    }

    // Hybrid
    for &max_size in &max_size_values {
        let result = run_diff_folder(&root, "hybrid", max_size, 0.0);

        println!("hybrid, max_size = {}, score = {}", max_size, result);

        if result < min_score {
            min_score = result;
            best_configuration = (Some("hybrid"), Some(max_size), None);
        }
    }

    // Simple
    let result = run_diff_folder(&root, "simple", 0, 0.0);

    println!("simple, score = {}", result);

    if result < min_score {
        min_score = result;
        best_configuration = (Some("simple"), None, None);
    }

    println!(
        "Grid search complete. Best configuration = {:?}, Minimum score = {}",
        best_configuration, min_score
    );
}

fn run_grid_search_local(path: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("datasets/defects4j");
    
    let before = root.join("before").join(&path);
    let after = root.join("after").join(&path);
    
    // Start with simple since it is the most performant
    let mut best_configuration = (Some("simple"), None, None);
    let mut min_score = run_diff_file(&before, &after, "simple", 0, 0.0);
    println!("simple, score = {}", min_score);

    let max_size_values = vec![100, 200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100, 1200, 1300, 1400, 1500, 1600, 1700, 1800, 1900, 2000];
    let sim_threshold_values = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

    
    for &max_size in &max_size_values {
        // Hybrid
        let result = run_diff_file(&before, &after, "hybrid", max_size, 0.0);

        println!("hybrid, max_size = {}, score = {}", max_size, result);

        if result < min_score {
            min_score = result;
            best_configuration = (Some("hybrid"), Some(max_size), None);
        }
        
        // Greedy
        for &sim_threshold in &sim_threshold_values {
            let result = run_diff_file(&before, &after,"greedy", max_size, sim_threshold);

            println!("greedy, max_size = {}, sim_threshold = {}, score = {}", max_size, sim_threshold, result);

            if result < min_score {
                min_score = result;
                best_configuration = (Some("greedy"), Some(max_size), Some(sim_threshold));
            }
        }
    }


    
    dbg!(before, after);

    println!(
        "Grid search complete. Best configuration = {:?}, Minimum score = {}",
        best_configuration, min_score
    );
}

#[test]
fn run_gridsearch_global() {
    let now = Instant::now();
    run_grid_search_defects4j();
    dbg!(now.elapsed().as_secs_f64());
}

#[test]
fn run_gridsearch_local_1() {
    let now = Instant::now();
    run_grid_search_local("Cli/13/src_java_org_apache_commons_cli2_WriteableCommandLine.java");
    dbg!(now.elapsed().as_secs_f64());
}

#[test]
fn run_gridsearch_local_2() {
    let now = Instant::now();
    run_grid_search_local("Chart/10/source_org_jfree_chart_imagemap_StandardToolTipTagFragmentGenerator.java");
    dbg!(now.elapsed().as_secs_f64());
}

#[test]
fn run_gridsearch_local_3() {
    let now = Instant::now();
    run_grid_search_local("Mockito/32/src_org_mockito_internal_configuration_SpyAnnotationEngine.java");
    dbg!(now.elapsed().as_secs_f64());
}

#[test]
fn run_gridsearch_local_4() {
    let now = Instant::now();
    run_grid_search_local("JxPath/7/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationLessThanOrEqual.java");
    dbg!(now.elapsed().as_secs_f64());
}