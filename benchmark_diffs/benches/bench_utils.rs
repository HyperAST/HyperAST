use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

pub enum HeuristicType {
    Lazy,
    Greedy,
}
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
        match self {
            DataSet::GhJava(project_opt) => {
                let project_name = project_opt.as_deref().unwrap_or_default();
                write!(f, "gh-java/{}", project_name)
            }
            DataSet::GhPython(project_opt) => {
                let project_name = project_opt.as_deref().unwrap_or_default();
                write!(f, "gh-python/{}", project_name)
            }
            DataSet::Defects4J(project_opt) => {
                let project_name = project_opt.as_deref().unwrap_or_default();
                write!(f, "defects4j/{}", project_name)
            }
            DataSet::BugsInPy(project_opt) => {
                let project_name = project_opt.as_deref().unwrap_or_default();
                write!(f, "bugsinpy/{}", project_name)
            }
        }
    }
}

impl DataSet {
    pub fn get_path_dataset_project(&self) -> PathBuf {
        let dataset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("datasets");

        let project_subdir = match self {
            DataSet::GhJava(project_opt) => {
                let project_name = project_opt.as_deref().unwrap_or_default();
                format!("gh-java/{}", project_name)
            }
            DataSet::GhPython(project_opt) => {
                let project_name = project_opt.as_deref().unwrap_or_default();
                format!("gh-python/{}", project_name)
            }
            DataSet::Defects4J(project_opt) => {
                let project_name = project_opt.as_deref().unwrap_or_default();
                format!("defects4j/{}", project_name)
            }
            DataSet::BugsInPy(project_opt) => {
                let project_name = project_opt.as_deref().unwrap_or_default();
                format!("bugsinpy/{}", project_name)
            }
        };

        let full_path = dataset_root.join(project_subdir);
        assert!(
            dataset_root.exists(),
            "Path to dataset did not exist, path was: {:?}",
            dataset_root.display()
        );
        assert!(
            full_path.exists(),
            "Path to dataset subset did not exist, path was: {:?}",
            full_path.display()
        );
        full_path
    }
}
