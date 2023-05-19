use std::fmt::Display;

use axum::Json;
use hyper_ast::{
    store::defaults::NodeIdentifier,
    types::{HyperType, TypeStore, WithChildren, WithStats},
};
use rhai::{
    packages::{BasicArrayPackage, CorePackage, Package},
    Array, Dynamic, Engine, Instant, Scope,
};
use serde::{Deserialize, Serialize};

use crate::SharedState;

#[derive(Deserialize, Clone)]
pub struct ScriptingParam {
    user: String,
    name: String,
    commit: String,
}

#[derive(Deserialize, Serialize)]
pub struct ScriptContent {
    pub init: String,
    pub accumulate: String,
    pub filter: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ScriptingError {
    AtCompilation(String),
    AtEvaluation(String),
    Other(String),
}

// impl Display for ScriptingError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             ScriptingError::Compiling(x) => writeln!(f, "script is ill-formed: {}", x),
//             ScriptingError::Evaluation(x) => writeln!(f, "script execution failed: {}", x),
//         }
//     }
// }

#[derive(Deserialize, Serialize)]
pub struct ComputeResult {
    pub compute_time: f64,
    pub result: Dynamic,
}

impl Display for ComputeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub fn simple(
    script: ScriptContent,
    state: SharedState,
    path: ScriptingParam,
) -> Result<Json<ComputeResult>, ScriptingError> {
    let now = Instant::now();
    let ScriptingParam { user, name, commit } = path.clone();
    let mut engine = Engine::new();
    engine.disable_symbol("/");
    let init_script = engine
        .compile(script.init.clone())
        .map_err(|x| ScriptingError::AtCompilation(format!("{}, {}", x, script.init.clone())))?;
    let accumulate_script = engine.compile(script.accumulate.clone()).map_err(|x| {
        ScriptingError::AtCompilation(format!("{}, {}", x, script.accumulate.clone()))
    })?;
    let filter_script = engine
        .compile(script.filter.clone())
        .map_err(|x| ScriptingError::AtCompilation(format!("{}, {}", x, script.filter.clone())))?;
    let repo_spec = hyper_ast_cvs_git::git::Forge::Github.repo(user, name);
    let configs = state.configs.read().unwrap();
    let config = configs
        .get(&repo_spec)
        .ok_or_else(|| ScriptingError::Other("missing config for repository".to_string()))?;
    let mut repo = repo_spec.fetch();
    log::warn!("done cloning {}/{}", repo_spec.user, repo_spec.name);
    let commits = state
        .repositories
        .write()
        .unwrap()
        .pre_process_with_config(&mut repo, "", &commit, config.into())
        .unwrap();
    log::info!(
        "done construction of {commits:?} in  {}/{}",
        repo_spec.user,
        repo_spec.name
    );

    let repositories = state.repositories.read().unwrap();
    let commit_src = repositories.commits.get_key_value(&commits[0]).unwrap();
    let src_tr = commit_src.1.ast_root;
    let node_store = &repositories.processor.main_stores.node_store;
    let size = node_store.resolve(src_tr).size();
    drop(repositories);
    macro_rules! ns {
        ($s:expr) => {
            $s.repositories
                .read()
                .unwrap()
                .processor
                .main_stores
                .node_store
        };
    }
    macro_rules! stores {
        ($s:expr) => {
            $s.repositories.read().unwrap().processor.main_stores
        };
    }
    #[derive(Debug)]
    struct Acc {
        sid: NodeIdentifier,
        value: Option<Dynamic>,
        parent: usize,
        pending_cs: isize,
    }
    let init: Dynamic = engine
        .eval_ast(&init_script)
        .map_err(|x| ScriptingError::AtEvaluation(x.to_string()))?;
    let mut stack: Vec<Acc> = vec![];
    stack.push(Acc {
        sid: src_tr,
        value: Some(init),
        parent: 0,
        pending_cs: -1,
    });
    let mut acc_engine = Engine::new_raw();
    acc_engine.on_print(|text| println!("{text}"));
    let package = CorePackage::new();
    package.register_into_engine(&mut acc_engine);
    let package = BasicArrayPackage::new();
    package.register_into_engine(&mut acc_engine);
    let mut filter_engine = Engine::new_raw();
    filter_engine.on_print(|text| println!("{text}"));
    let package = CorePackage::new();
    package.register_into_engine(&mut filter_engine);
    let package = BasicArrayPackage::new();
    package.register_into_engine(&mut filter_engine);
    // let s = state.clone().read().unwrap();
    let result: Dynamic = loop {
        let Some(mut acc) = stack.pop() else {
        unreachable!()
    };
        // let s = Rc::new(s);
        let stack_len = stack.len();
        // dbg!(&acc);
        if acc.pending_cs < 0 {
            // let mut engine = Engine::new();
            let mut scope = Scope::new();
            scope.push("s", acc.value.clone().unwrap());
            filter_engine.disable_symbol("/");
            let current = acc.sid;
            let s = state.clone();
            filter_engine.register_fn("type", move || {
                let stores = &stores!(s);
                let node_store = &stores.node_store;
                let type_store = &stores.type_store;
                let n = node_store.resolve(current);
                let t = type_store.resolve_type(&n);
                t.to_string()
            });
            let s = state.clone();
            filter_engine.register_fn("is_directory", move || {
                let stores = &stores!(s);
                let node_store = &stores.node_store;
                let type_store = &stores.type_store;
                let n = node_store.resolve(current);
                let t = type_store.resolve_type(&n);
                t.is_directory()
            });
            let s = state.clone();
            filter_engine.register_fn("is_type_decl", move || {
                let stores = &stores!(s);
                let node_store = &stores.node_store;
                let type_store = &stores.type_store;
                let n = node_store.resolve(current);
                let t = type_store.resolve_type(&n);
                let s = t.as_shared();
                s == hyper_ast::types::Shared::TypeDeclaration
                // node_store.resolve(current).get_type().is_type_declaration()
            });
            let s = state.clone();
            filter_engine.register_fn("is_file", move || {
                let stores = &stores!(s);
                let node_store = &stores.node_store;
                let type_store = &stores.type_store;
                let n = node_store.resolve(current);
                let t = type_store.resolve_type(&n);
                t.is_file()
            });
            let s = state.clone();
            filter_engine.register_fn("children", move || {
                let node_store = &ns!(s);
                node_store
                    .resolve(current)
                    .children()
                    .map_or(Default::default(), |v| {
                        v.0.iter().map(|x| Dynamic::from(*x)).collect::<Array>()
                    })
            });
            let prepared: Dynamic = filter_engine
                .eval_ast_with_scope(&mut scope, &filter_script)
                .map_err(|x| ScriptingError::AtEvaluation(x.to_string()))?;
            if let Some(prepared) = prepared.try_cast::<Vec<Dynamic>>() {
                stack.push(Acc {
                    pending_cs: prepared.len() as isize,
                    ..acc
                });
                stack.extend(prepared.into_iter().map(|x| x.cast()).map(|x: Array| {
                    let mut it = x.into_iter();
                    Acc {
                        sid: it.next().unwrap().cast(),
                        value: Some(it.next().unwrap()),
                        parent: stack_len,
                        pending_cs: -1,
                    }
                }));
            }
            continue;
        }
        if stack.is_empty() {
            assert_eq!(acc.parent, 0);
            break acc.value.unwrap();
        }
        // let mut engine = Engine::new();
        let mut scope = Scope::new();
        scope.push("s", acc.value.take().unwrap());
        scope.push("p", stack[acc.parent].value.take().unwrap());
        acc_engine.disable_symbol("/");
        let current = acc.sid;
        let s = state.clone();
        acc_engine.register_fn("size", move || {
            let node_store = &ns!(s);
            node_store.resolve(current).size() as i64
        });
        let s = state.clone();
        acc_engine.register_fn("type", move || {
            let stores = &stores!(s);
            let node_store = &stores.node_store;
            let type_store = &stores.type_store;
            let n = node_store.resolve(current);
            let t = type_store.resolve_type(&n);
            t.to_string()
        });
        let s = state.clone();
        acc_engine.register_fn("is_type_decl", move || {
            let stores = &stores!(s);
            let node_store = &stores.node_store;
            let type_store = &stores.type_store;
            let n = node_store.resolve(current);
            let t = type_store.resolve_type(&n);
            let s = t.as_shared();
            s == hyper_ast::types::Shared::TypeDeclaration
            // node_store.resolve(current).get_type().is_type_declaration()
        });
        let s = state.clone();
        acc_engine.register_fn("is_directory", move || {
            let stores = &stores!(s);
            let node_store = &stores.node_store;
            let type_store = &stores.type_store;
            let n = node_store.resolve(current);
            let t = type_store.resolve_type(&n);
            t.is_directory()
        });
        let s = state.clone();
        acc_engine.register_fn("is_file", move || {
            let stores = &stores!(s);
            let node_store = &stores.node_store;
            let type_store = &stores.type_store;
            let n = node_store.resolve(current);
            let t = type_store.resolve_type(&n);
            t.is_file()
        });
        acc_engine
            .eval_ast_with_scope(&mut scope, &accumulate_script)
            .map_err(|x| ScriptingError::AtEvaluation(x.to_string()))?;
        stack[acc.parent].value = Some(scope.get_value("p").unwrap());
    };
    // let r = format!(
    //     "Computed {result} in commit {} of size {size} at github.com/{user}/{name}",
    //     &commit[..8.min(commit.len())]
    // );
    let compute_time = now.elapsed().as_secs_f64();
    let r = ComputeResult {
        compute_time,
        result,
    };
    Ok(Json(r))
}
