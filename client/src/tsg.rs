use crate::SharedState;
use axum::Json;
use hyper_ast_cvs_git::SimpleStores;
use hyper_ast_tsquery::stepped_query::{Node, QueryMatcher};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tree_sitter_graph::GenQuery;

pub type Functions<Node> =
    tree_sitter_graph::functions::Functions<tree_sitter_graph::graph::GraphErazing<Node>>;

#[derive(Deserialize, Clone)]
pub struct Param {
    user: String,
    name: String,
    commit: String,
}

#[derive(Deserialize, Clone)]
pub struct Content {
    pub language: String,
    pub query: String,
    pub commits: usize,
}

#[derive(Debug, Serialize, Clone)]
pub enum QueryingError {
    MissingLanguage(String),
    TsgParsing(String),
}

#[derive(Serialize)]
pub struct ComputeResults {
    pub prepare_time: f64,
    pub results: Vec<Result<ComputeResultIdentified, String>>,
}

#[derive(Serialize)]
pub struct ComputeResultIdentified {
    pub commit: String,
    #[serde(flatten)]
    pub inner: ComputeResult,
}

#[derive(Serialize)]
pub struct ComputeResult {
    pub compute_time: f64,
    pub result: serde_json::Value,
}

pub fn simple(
    query: Content,
    state: SharedState,
    path: Param,
) -> Result<Json<ComputeResults>, QueryingError> {
    let now = Instant::now();
    let Param { user, name, commit } = path.clone();
    let Content {
        language,
        query,
        commits,
    } = query;
    let language: tree_sitter::Language = hyper_ast_cvs_git::resolve_language(&language)
        .ok_or_else(|| QueryingError::MissingLanguage(language))?;
    let repo_spec = hyper_ast_cvs_git::git::Forge::Github.repo(user, name);
    let repo = state
        .repositories
        .write()
        .unwrap()
        .get_config(repo_spec.clone());
    let repo = match repo {
        Some(repo) => repo,
        None => {
            let configs = &mut state.repositories.write().unwrap();
            configs.register_config(
                repo_spec.clone(),
                hyper_ast_cvs_git::processing::RepoConfig::JavaMaven,
            );
            log::error!("missing config for {}", repo_spec);
            configs.get_config(repo_spec.clone()).unwrap()
        }
    };
    // .ok_or_else(|| ScriptingError::Other("missing config for repository".to_string()))?;
    let mut repo = repo.fetch();
    log::warn!("done cloning {}", &repo.spec);
    let commits = state
        .repositories
        .write()
        .unwrap()
        .pre_process_with_limit(&mut repo, "", &commit, commits)
        .unwrap();
    let tsg = QueryMatcher::<SimpleStores>::from_str(language.clone(), &query)
        .map_err(|e| QueryingError::TsgParsing(e.to_string()))?;
    let prepare_time = now.elapsed().as_secs_f64();
    log::info!("done construction of {commits:?} in  {}", repo.spec);
    let mut results = vec![];
    for commit_oid in &commits {
        let result = simple_aux(&state, &repo, commit_oid, &query, &tsg)
            .map(|inner| ComputeResultIdentified {
                commit: commit_oid.to_string(),
                inner,
            })
            .map_err(|err| format!("{:?}", err));
        results.push(result);
    }
    log::info!("done querying of {commits:?} in  {}", repo.spec);
    Ok(Json(ComputeResults {
        prepare_time,
        results,
    }))
}

fn simple_aux(
    state: &crate::AppState,
    repo: &hyper_ast_cvs_git::processing::ConfiguredRepo2,
    commit_oid: &hyper_ast_cvs_git::git::Oid,
    query: &str,
    tsg: &tree_sitter_graph::ast::File<QueryMatcher<SimpleStores>>,
) -> Result<ComputeResult, QueryingError> {
    let now = Instant::now();
    type Graph<'a, HAST> = tree_sitter_graph::graph::Graph<Node<'a, HAST>>;
    // SimpleStores<hyper_ast_gen_ts_java::types::TStore>
    let mut globals = tree_sitter_graph::Variables::new();
    let mut graph = Graph::default();
    init_globals(&mut globals, &mut graph);
    let mut functions = Functions::stdlib();
    // tree_sitter_stack_graphs::functions::add_path_functions(&mut functions);
    let mut config = configure(&globals, &functions);
    let cancellation_flag = tree_sitter_graph::NoCancellation;

    let repositories = state.repositories.read().unwrap();
    let commit = repositories.get_commit(&repo.config, commit_oid).unwrap();
    let code = commit.ast_root;
    let stores = &repositories.processor.main_stores;

    let tree = Node::new(stores, hyper_ast::position::StructuralPosition::new(code));
    // SAFETY: just circumventing a limitation in the borrow checker, ie. all associated lifetimes considered as being 'static
    let tree = unsafe { std::mem::transmute(tree) };

    if let Err(err) = tsg.execute_lazy_into2(&mut graph, tree, &mut config, &cancellation_flag) {
        println!("{}", graph.pretty_print());
        let source_path = std::path::Path::new(&"");
        let tsg_path = std::path::Path::new(&"");
        eprintln!("{}", err.display_pretty(&source_path, "", &tsg_path, query));
    }
    let result = serde_json::to_value(graph).unwrap();
    let compute_time = now.elapsed().as_secs_f64();
    Ok(ComputeResult {
        result,
        compute_time,
    })
}

static DEBUG_ATTR_PREFIX: &'static str = "debug_";
pub static ROOT_NODE_VAR: &'static str = "ROOT_NODE";
/// The name of the file path global variable
pub const FILE_PATH_VAR: &str = "FILE_PATH";
static JUMP_TO_SCOPE_NODE_VAR: &'static str = "JUMP_TO_SCOPE_NODE";
static FILE_NAME: &str = "a/b/AAA.java";

fn configure<'a, 'g, Node>(
    globals: &'a tree_sitter_graph::Variables<'g>,
    functions: &'a Functions<Node>,
) -> tree_sitter_graph::ExecutionConfig<'a, 'g, tree_sitter_graph::graph::GraphErazing<Node>> {
    let config = tree_sitter_graph::ExecutionConfig::new(functions, globals)
        .lazy(true)
        .debug_attributes(
            [DEBUG_ATTR_PREFIX, "tsg_location"].concat().as_str().into(),
            [DEBUG_ATTR_PREFIX, "tsg_variable"].concat().as_str().into(),
            [DEBUG_ATTR_PREFIX, "tsg_match_node"]
                .concat()
                .as_str()
                .into(),
        );
    config
}

fn init_globals<Node: tree_sitter_graph::graph::SyntaxNodeExt>(
    globals: &mut tree_sitter_graph::Variables,
    graph: &mut tree_sitter_graph::graph::Graph<Node>,
) {
    globals
        .add(ROOT_NODE_VAR.into(), graph.add_graph_node().into())
        .expect("Failed to set ROOT_NODE");
    globals
        .add(FILE_PATH_VAR.into(), FILE_NAME.into())
        .expect("Failed to set FILE_PATH");
    globals
        .add(JUMP_TO_SCOPE_NODE_VAR.into(), graph.add_graph_node().into())
        .expect("Failed to set JUMP_TO_SCOPE_NODE");
}
