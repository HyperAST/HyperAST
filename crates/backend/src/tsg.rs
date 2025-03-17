use crate::SharedState;
use axum::Json;
use hyperast_tsquery::stepped_query::{MyQMatch, Node, QueryMatcher};
use hyperast_vcs_git::SimpleStores;
use serde::{Deserialize, Serialize};
use std::time::Instant;

// pub type Functions<Node> =
//     tree_sitter_graph::functions::Functions<tree_sitter_graph::graph::GraphErazing<Node>>;

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
    pub path: Option<String>,
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
        language: lang_name,
        query,
        commits,
        path,
    } = query;
    let language: tree_sitter::Language = hyperast_vcs_git::resolve_language(&lang_name)
        .ok_or_else(|| QueryingError::MissingLanguage(lang_name.clone()))?;
    let repo_spec = hyperast_vcs_git::git::Forge::Github.repo(user, name);
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
                hyperast_vcs_git::processing::RepoConfig::JavaMaven,
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
    let path = &path.unwrap_or_default();
    let tsg = {
        type M = QueryMatcher<SimpleStores>;
        type ExtQ = hyperast_tsquery::stepped_query::ExtendingStringQuery<M, tree_sitter::Language>;

        let source: &str = &query;

        let mut file = tree_sitter_graph::ast::File::<M>::new(language.clone());

        let precomputeds: Box<dyn hyperast_tsquery::ArrayStr> = state
            .repositories
            .read()
            .unwrap()
            .get_precomp_query(repo.config, &lang_name)
            .map_or(Box::new([].as_slice()), |x| Box::new(x));
        let query_source = ExtQ::new(language.clone(), precomputeds, source.len());
        tree_sitter_graph::parser::Parser::<ExtQ>::with_ext(query_source, source)
            .parse_into_file(&mut file)
            .map_err(|e| QueryingError::TsgParsing(e.to_string()))?;
        use tree_sitter_graph::GenQuery;
        QueryMatcher::<SimpleStores>::check(&mut file)
            .map_err(|e| QueryingError::TsgParsing(e.to_string()))?;
        file
    };
    let prepare_time = now.elapsed().as_secs_f64();
    log::info!("done construction of {commits:?} in  {}", repo.spec);
    let mut results = vec![];
    for commit_oid in &commits {
        let result = simple_aux(&state, &repo, commit_oid, &query, &tsg, path)
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
    repo: &hyperast_vcs_git::processing::ConfiguredRepo2,
    commit_oid: &hyperast_vcs_git::git::Oid,
    query: &str,
    tsg: &tree_sitter_graph::ast::File<QueryMatcher<SimpleStores>>,
    // QueryMatcher<hyperast::store::SimpleStores<TStore, &NodeStoreInner, &LabelStore>>,
    path: &str,
) -> Result<ComputeResult, QueryingError> {
    let now = Instant::now();
    // type Graph<'a, HAST> = tree_sitter_graph::graph::Graph<Node<'a, HAST>>;
    // SimpleStores<hyperast_gen_ts_java::types::TStore>
    let mut globals = tree_sitter_graph::Variables::new();
    let mut graph: tree_sitter_graph::graph::Graph<
        // hyperast_tsquery::stepped_query::Node<
        //     hyperast::store::SimpleStores<
        //         hyperast_gen_ts_java::types::TStore,
        //         &hyperast::store::nodes::legion::NodeStoreInner,
        //         &hyperast::store::labels::LabelStore,
        //     >,
        // >,
        hyperast_tsquery::hyperast_cursor::NodeR<
            hyperast::position::StructuralPosition<hyperast::store::defaults::NodeIdentifier, u16>,
        >,
        // Node<
        //     '_,
        //     hyperast_tsquery::hyperast_cursor::NodeR<_>,
        //     hyperast::position::structural_pos::StructuralPosition<_, _>,
        // >,
    > = tree_sitter_graph::graph::Graph::default();
    init_globals(&mut globals, &mut graph);
    let mut functions = tree_sitter_graph::functions::Functions::essentials();

    // TODO add it back
    // let mut functions = tree_sitter_graph::functions::Functions::stdlib();
    // tree_sitter_stack_graphs::functions::add_path_functions(&mut functions);
    let mut config = configure(&globals, &functions);
    let cancellation_flag = tree_sitter_graph::NoCancellation;

    let repositories = state.repositories.read().unwrap();
    let commit = repositories.get_commit(&repo.config, commit_oid).unwrap();
    let code = commit.ast_root;
    let stores = &repositories.processor.main_stores;

    let code =
        hyperast_vcs_git::preprocessed::child_at_path(stores, code, path.split('/')).unwrap();
    dbg!();
    let tree: Node<_> = Node::new(stores, hyperast::position::StructuralPosition::new(code));
    // let tree: Node<_> = hyperast_tsquery::hyperast_cursor::NodeR {
    //     pos: hyperast::position::StructuralPosition::new(code),
    // };
    // SAFETY: just circumventing a limitation in the borrow checker, ie. all associated lifetimes considered as being 'static
    let tree = unsafe { std::mem::transmute(tree) };
    if let Err(err) = tsg.execute_lazy_into2::<_, MyQMatch<SimpleStores>>(
        &mut graph,
        tree,
        &mut config,
        &cancellation_flag,
    ) {
        println!("{}", graph.pretty_print());
        let source_path = std::path::Path::new(&"");
        let tsg_path = std::path::Path::new(&"");
        eprintln!("{}", err.display_pretty(&source_path, "", &tsg_path, query));
    }
    dbg!();
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

fn configure<'a, 'b, 'g, Node>(
    globals: &'b tree_sitter_graph::Variables<'g>,
    functions: &'a tree_sitter_graph::functions::Functions<tree_sitter_graph::graph::Graph<Node>>,
) -> tree_sitter_graph::ExecutionConfig<'a, 'g, 'b, tree_sitter_graph::graph::Graph<Node>> {
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

fn init_globals<Node>(
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
