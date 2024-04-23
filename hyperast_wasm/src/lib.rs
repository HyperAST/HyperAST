mod store;
mod network;
mod types;

use wasm_bindgen::prelude::*;

// #[wasm_bindgen]
// extern "C" {
//     pub fn alert(s: &str);
// }

#[wasm_bindgen]
pub fn greet(name: &str) {
    log(&format!("Hello, {}!", name));
}

// lifted from the `console_log` example
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
fn run() {
    log(&format!("Hello {}!", "mister"));
}

use crate::store::FetchedHyperAST;
use std::sync::Arc;
use std::collections::HashMap;
use crate::store::NodeIdentifier;


// #[wasm_bindgen]
// pub struct HyperAstDb2 {
//     ast: Arc<FetchedHyperAST>,
// }


#[wasm_bindgen]
pub struct HyperAstDb {
    api_addr: String,
    ast: Arc<FetchedHyperAST>,
}

#[wasm_bindgen]
impl HyperAstDb {
    #[wasm_bindgen(constructor)]
    pub fn new(api_addr: &str) -> Self {
        let api_addr = api_addr.to_string();
        let ast = Default::default();
        Self { api_addr,ast }
    }

    pub fn git(&mut self, ) -> GitSession {
        // user: &str, name: &str, commit: &str
        // let commit = types::Commit {
        //     repo: types::Repo {
        //         user: user.to_string(),
        //         name: name.to_string(),
        //     },
        //     id: commit.to_string(),
        // };
        GitSession::new(self.api_addr.clone(), self.ast.clone())
    }

    pub fn scratch_pad(&mut self) -> ScratchPadSession {
        ScratchPadSession::new(self.api_addr.clone(), self.ast.clone())
    }
}

#[wasm_bindgen]
pub struct ScratchPadSession {
    api_addr: String,
    ast: Arc<FetchedHyperAST>,
    snap: HashMap<usize, InternalNodeId>,
}

impl ScratchPadSession {
    fn new(api_addr: String, ast: Arc<FetchedHyperAST>) -> Self {
        let api_addr = api_addr.to_string();
        Self {
            api_addr,
            ast,
            snap: Default::default(),
        }
    }
}
type LocalId = std::num::NonZeroU32;
enum InternalNodeId{
    Remote(NodeIdentifier),
    Local(LocalId),
}

#[wasm_bindgen]
pub struct NodeId(
    InternalNodeId
);

#[wasm_bindgen]
impl ScratchPadSession {
    pub fn snap_build(&mut self, _prev: f32, _path: &[f32], _typ: String) -> () {
        todo!("compute parents from prev snap and path to edit")
    }
    pub fn snap_build_with_label(&mut self, _prev: f32, _path: &[f32], _typ: String, _label: String) -> () {
        todo!("compute parents from prev snap and path to edit")
    }
    pub fn snap_build_empty(&mut self, _prev: f32, _path: &[f32]) -> () {
        todo!("compute parents from prev snap and path to edit")
    }
    // pub async fn push_type(&mut self, typ: &str) -> NodeId {
    //     todo!()
    // }
    // pub async fn push_labeled(&mut self, typ: &str, label: &str ) -> NodeId {
    //     todo!()
    // }
    // pub async fn push_with_children(&mut self, typ: &str, children: Vec<NodeId>, ) -> NodeId {
    //     todo!()
    // }
    pub async fn fetch_node(&mut self, _node: &mut NodeId) -> String {
        todo!()
    }
}

#[wasm_bindgen]
pub struct GitSession {
    api_addr: String,
    ast: Arc<FetchedHyperAST>,
    commits: HashMap<types::Commit, NodeIdentifier>
}

impl GitSession {
    fn new(api_addr: String, ast: Arc<FetchedHyperAST>) -> Self {
        let api_addr = api_addr.to_string();
        Self {
            api_addr,
            ast,
            commits: Default::default(),
        }
    }
}

#[wasm_bindgen]
impl GitSession {
    pub async fn fetch_node(&mut self, _node: NodeId) -> String {
        todo!()
    }
}

