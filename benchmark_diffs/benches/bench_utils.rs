use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

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
#[derive(Debug, Clone, Copy)]
pub enum DataSet {
    GhJava(Option<&'static str>),
    GhPython(Option<&'static str>),
    Defects4J(Option<&'static str>),
    BugsInPy(Option<&'static str>),
}

impl Display for DataSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (dataset_name, project_opt) = self.parts();
        let project_name = project_opt.unwrap_or_default();
        write!(f, "{}/{}", dataset_name, project_name)
    }
}

impl DataSet {
    /// Returns a tuple of (dataset_name, project_name_option)
    fn parts(&self) -> (&'static str, Option<&'static str>) {
        match self {
            DataSet::GhJava(project) => ("gh-java", *project),
            DataSet::GhPython(project) => ("gh-python", *project),
            DataSet::Defects4J(project) => ("defects4j", *project),
            DataSet::BugsInPy(project) => ("bugsinpy", *project),
        }
    }

    pub fn get_path_dataset_project(&self) -> (PathBuf, PathBuf) {
        let dataset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("datasets");

        let (dataset_name, opt_project_name) = self.parts();
        let project_name = opt_project_name.unwrap_or_default();

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
}
