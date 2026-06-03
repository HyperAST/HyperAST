pub mod recursive_query;

pub mod stepped_query;

#[cfg(any(test, feature = "all_examples"))]
pub(crate) mod resources;
#[cfg(any(test, feature = "all_examples"))]
pub use resources::*;
#[cfg(any(test, feature = "all_examples"))]
use tree_sitter_graph::graph::GraphErazing;

#[cfg(any(test, feature = "all_examples"))]
pub type Functions<Node> = tree_sitter_graph::functions::Functions<GraphErazing<Node>>;

static DEBUG_ATTR_PREFIX: &'static str = "debug_";
#[cfg(any(test, feature = "all_examples"))]
pub static ROOT_NODE_VAR: &'static str = "ROOT_NODE";
/// The name of the file path global variable
pub const FILE_PATH_VAR: &str = "FILE_PATH";
static JUMP_TO_SCOPE_NODE_VAR: &'static str = "JUMP_TO_SCOPE_NODE";
static FILE_NAME: &str = "a/b/AAA.java";

#[cfg(any(test, feature = "all_examples"))]
pub fn configure<'a, 'g, Node>(
    globals: &'a tree_sitter_graph::Variables<'g>,
    functions: &'a tree_sitter_graph::functions::Functions<GraphErazing<Node>>,
) -> tree_sitter_graph::ExecutionConfig<'a, 'g, GraphErazing<Node>> {
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

#[cfg(any(test, feature = "all_examples"))]
pub fn init_globals<Node: tree_sitter_graph::graph::SyntaxNodeExt>(
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

#[cfg(any(test, feature = "all_examples"))]
/// Iterates al files in provided directory
pub struct It {
    inner: Option<Box<It>>,
    outer: Option<std::fs::ReadDir>,
    p: Option<std::path::PathBuf>,
}

#[cfg(any(test, feature = "all_examples"))]
impl It {
    pub fn new(p: std::path::PathBuf) -> Self {
        Self {
            inner: None,
            outer: None,
            p: Some(p),
        }
    }
}

#[cfg(any(test, feature = "all_examples"))]
impl Iterator for It {
    type Item = std::path::PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        dbg!();
        let Some(p) = &mut self.inner else {
            let Some(d) = &mut self.outer else {
                if let Ok(d) = self.p.as_mut()?.read_dir() {
                    self.outer = Some(d);
                    return self.next();
                } else {
                    return Some(self.p.take()?);
                }
            };
            let p = d.next()?.ok()?.path();
            self.inner = Some(Box::new(It::new(p)));
            return self.next();
        };
        let Some(p) = p.next() else {
            let p = self.outer.as_mut().unwrap().next()?.ok()?.path();
            self.inner = Some(Box::new(It::new(p)));
            return self.next();
        };
        Some(p)
    }
}
