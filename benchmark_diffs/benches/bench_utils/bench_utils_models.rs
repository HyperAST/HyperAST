use hyper_diff::{
    decompressed_tree_store::{CompletePostOrder, lazy_post_order::LazyPostOrder},
    matchers::{
        Decompressible, Mapper,
        heuristic::gt::{
            greedy_bottom_up_matcher::GreedyBottomUpMatcher,
            lazy_simple_bottom_up_matcher::LazySimpleBottomUpMatcher,
            lazy2_greedy_bottom_up_matcher::GreedyBottomUpMatcher as LazyGreedyBottomUpMatcher,
            simple_bottom_up_matcher3::SimpleBottomUpMatcher,
        },
        mapping_store::{MappingStore, VecStore},
    },
};
use hyperast::types::{self, HyperAST, HyperASTShared, NodeId};
use hyperast_gen_ts_java::legion_with_refs::Local;
use std::{
    fmt::{Debug, Display},
    fs,
    fs::{File, create_dir_all},
    io::Write,
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::bench_utils::bench_utils_methods::lazy_top_down;

#[allow(type_alias_bounds)]
type CDS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;
#[allow(type_alias_bounds)]
type DS<HAST: HyperASTShared> = Decompressible<HAST, LazyPostOrder<HAST::IdN, u32>>;

pub enum HeuristicType {
    Lazy,
    Greedy,
}

#[derive(Clone, Copy)]
pub enum Heuristic {
    Greedy,
    Simple,
    LazyGreedy,
    LazySimple,
}

impl Display for Heuristic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Heuristic::Greedy => write!(f, "Gumtree_Greedy"),
            Heuristic::Simple => write!(f, "Gumtree_Simple"),
            Heuristic::LazyGreedy => write!(f, "Lazy_Gumtree_Greedy"),
            Heuristic::LazySimple => write!(f, "Lazy_Gumtree_Simple"),
        }
    }
}

impl Heuristic {
    pub fn get_heuristic_type(&self) -> HeuristicType {
        match self {
            Heuristic::Greedy => HeuristicType::Greedy,
            Heuristic::Simple => HeuristicType::Greedy,
            Heuristic::LazyGreedy => HeuristicType::Lazy,
            Heuristic::LazySimple => HeuristicType::Lazy,
        }
    }
}

/// Four subsets of the total dataset. The optional string is a specific project in the subset, None if we want all.
#[derive(Debug, Clone, Serialize)]
pub enum DataSet {
    GhJava(Option<String>),
    GhPython(Option<String>),
    Defects4J(Option<String>),
    BugsInPy(Option<String>),
}

impl Display for DataSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (dataset_name, project_opt) = self.parts();
        let project_name = project_opt.as_deref().unwrap_or_default();
        write!(f, "{}/{}", dataset_name, project_name)
    }
}

impl DataSet {
    /// Returns a tuple of (dataset_name, project_name_option)
    fn parts(&self) -> (&'static str, &Option<String>) {
        match self {
            DataSet::GhJava(project) => ("gh-java", project),
            DataSet::GhPython(project) => ("gh-python", project),
            DataSet::Defects4J(project) => ("defects4j", project),
            DataSet::BugsInPy(project) => ("bugsinpy", project),
        }
    }

    fn get_base_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("datasets")
    }

    fn get_directory_names(path: &PathBuf) -> Vec<String> {
        assert!(path.exists(), "Path does not exist: {:?}", path);
        assert!(path.is_dir(), "Path is not a directory: {:?}", path);
        let mut dir_names = Vec::new();

        for entry_result in fs::read_dir(path).expect("Could not read dir") {
            let entry = entry_result.expect("Error when reading file entry");
            let path = entry.path();

            if path.is_dir() {
                if let Some(os_str_name) = path.file_name() {
                    if let Some(name_str) = os_str_name.to_str() {
                        dir_names.push(name_str.to_string());
                    } else {
                        eprintln!("Skipping non-UTF-8 directory name: {:?}", os_str_name);
                    }
                }
            }
        }

        dir_names
    }

    pub fn get_all_projects_of_dataset(&self) -> Vec<String> {
        let base_path = DataSet::get_base_path();
        let (dataset_name, _) = self.parts();
        let path = base_path.join(dataset_name).join("before");
        DataSet::get_directory_names(&path)
    }

    pub fn get_path_dataset_project(&self) -> (PathBuf, PathBuf) {
        let dataset_root = DataSet::get_base_path();

        let (dataset_name, opt_project_name) = self.parts();
        let project_name = opt_project_name.as_deref().unwrap_or_default();

        let full_path_before = dataset_root
            .join(dataset_name)
            .join("before")
            .join(project_name);
        let full_path_after = dataset_root
            .join(dataset_name)
            .join("after")
            .join(project_name);

        assert!(
            dataset_root.exists(),
            "Path to dataset did not exist, path was: {:?}",
            dataset_root.display()
        );
        assert!(
            full_path_before.exists(),
            "Path to dataset before subset did not exist, path was: {:?}",
            full_path_before.display()
        );
        assert!(
            full_path_after.exists(),
            "Path to dataset before subset did not exist, path was: {:?}",
            full_path_after.display()
        );
        (full_path_before, full_path_after)
    }

    pub fn generate_bench_group_name(&self) -> String {
        let (dataset_name, opt_project_name) = self.parts();
        let project_name = opt_project_name.as_deref().unwrap_or_default();
        format!("{}_{}", dataset_name, project_name)
    }
}

#[derive(Serialize)]
pub struct BenchInfo {
    pub(crate) dataset: DataSet,
    pub(crate) metrics_src: u32,
    pub(crate) metrics_dst: u32,
    pub(crate) num_matches_greedy_top_down: usize,
    pub(crate) num_matches_lazy_top_down: usize,
    pub(crate) num_matches_greedy_bottom_up: usize,
    pub(crate) num_matches_simple_bottom_up: usize,
    pub(crate) num_matches_lazy_greedy_bottom_up: usize,
    pub(crate) num_matches_lazy_simple_bottom_up: usize,
}

impl BenchInfo {
    pub fn compute_new<HAST: HyperAST + Copy>(
        dataset: DataSet,
        src: &Local,
        dst: &Local,
        greedy_mapper: Mapper<HAST, CDS<HAST>, CDS<HAST>, VecStore<u32>>,
        lazy_mapper: (DS<HAST>, DS<HAST>),
    ) -> Self
    where
        HAST::IdN: Clone + Debug + Eq,
        HAST::IdN: NodeId<IdN = HAST::IdN>,
        HAST::Idx: hyperast::PrimInt,
        HAST::Label: Debug + Clone + Copy + Eq,
        for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
    {
        let num_matches_greedy_bottom_up =
            GreedyBottomUpMatcher::<_, _, _, _>::match_it(greedy_mapper.clone())
                .mappings
                .len();
        let num_matches_simple_bottom_up =
            SimpleBottomUpMatcher::<_, _, _, _>::match_it(greedy_mapper.clone())
                .mappings
                .len();
        let num_matches_lazy_greedy_bottom_up = LazyGreedyBottomUpMatcher::<_, _, _, _>::match_it(
            lazy_top_down(&mut lazy_mapper.clone()),
        )
        .mappings
        .len();
        let num_matches_lazy_simple_bottom_up = LazySimpleBottomUpMatcher::<_, _, _, _>::match_it(
            lazy_top_down(&mut lazy_mapper.clone()),
        )
        .mappings
        .len();

        BenchInfo {
            dataset,
            metrics_src: src.metrics.size,
            metrics_dst: dst.metrics.size,
            num_matches_greedy_top_down: greedy_mapper.mappings.len(),
            num_matches_lazy_top_down: lazy_top_down(&mut lazy_mapper.clone()).mappings.len(),
            num_matches_greedy_bottom_up: num_matches_greedy_bottom_up,
            num_matches_simple_bottom_up: num_matches_simple_bottom_up,
            num_matches_lazy_greedy_bottom_up: num_matches_lazy_greedy_bottom_up,
            num_matches_lazy_simple_bottom_up: num_matches_lazy_simple_bottom_up,
        }
    }

    pub fn write_to_file(&self, dataset: DataSet) {
        // Assemble path to file location, add timestamp for uniqueness
        let timestamp = chrono::Local::now().format("%Y-%m-%dT%H-%M").to_string();
        let file_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("benches")
            .join("bench_stats")
            .join(format!("{}_{}.json", dataset, timestamp));

        // Create the path to the file if it does not exist yet
        if let Some(parent) = file_path.parent() {
            create_dir_all(parent).expect("Failed to create parent dirs");
        }

        // Serialize the bench info
        let json_string =
            serde_json::to_string_pretty(self).expect("couldnt serialize BenchInfo to json string");

        // Write to file
        let mut file = File::create(&file_path).expect("Unable to create file");
        file.write_all(json_string.as_bytes())
            .expect("Failed to write JSON to file");
        println!("Wrote meta data to: {:?}", file_path);
    }
}
