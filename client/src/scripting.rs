use std::{fmt::Display, rc::Rc};

use hyper_ast::{
    store::defaults::NodeIdentifier,
    types::{Typed, WithChildren, WithStats},
};
use hyper_ast_cvs_git::git::fetch_github_repository;
use rhai::{Array, Dynamic, Engine, Scope, packages::{CorePackage, Package, BasicArrayPackage}};
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

#[derive(Debug)]
pub enum ScriptingError {
    Compiling(String),
    Evaluation(String),
}

impl Display for ScriptingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptingError::Compiling(x) => writeln!(f, "script compile: {}", x),
            ScriptingError::Evaluation(x) => writeln!(f, "script evaluation: {}", x),
        }
    }
}

pub fn simple(
    script: ScriptContent,
    state: SharedState,
    path: ScriptingParam,
) -> Result<String, ScriptingError> {
    let ScriptingParam { user, name, commit } = path.clone();
    let mut engine = Engine::new();
    engine.disable_symbol("/");
    let init_script = engine
        .compile(script.init.clone())
        .map_err(|x| ScriptingError::Compiling(format!("{}, {}", x, script.init.clone())))?;
    let accumulate_script = engine
        .compile(script.accumulate.clone())
        .map_err(|x| ScriptingError::Compiling(format!("{}, {}", x, script.accumulate.clone())))?;
    let filter_script = engine
        .compile(script.filter.clone())
        .map_err(|x| ScriptingError::Compiling(format!("{}, {}", x, script.filter.clone())))?;
    let mut repo = fetch_github_repository(&format!("{}/{}", user, name));
    log::info!("done cloning {user}/{name}");
    let mut get_mut = state.write().unwrap();
    let commits = get_mut
        .repositories
        .pre_process_with_limit(&mut repo, "", &commit, "", 2);
    log::info!("done construction of {commits:?} in {user}/{name}");
    let commit_src = get_mut
        .repositories
        .commits
        .get_key_value(&commits[0])
        .unwrap();
    let src_tr = commit_src.1.ast_root;
    let node_store = &get_mut.repositories.processor.main_stores.node_store;
    let size = node_store.resolve(src_tr).size();
    drop(get_mut);
    macro_rules! ns {
        ($s:expr) => {
            $s.read()
                .unwrap()
                .repositories
                .processor
                .main_stores
                .node_store
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
        .map_err(|x| ScriptingError::Evaluation(x.to_string()))?;
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
            filter_engine.register_fn("is_directory", move || {
                let node_store = &ns!(s);
                node_store.resolve(current).get_type().is_directory()
            });
            let s = state.clone();
            filter_engine.register_fn("is_type_decl", move || {
                let node_store = &&ns!(s);
                node_store.resolve(current).get_type().is_type_declaration()
            });
            let s = state.clone();
            filter_engine.register_fn("is_file", move || {
                let node_store = &&ns!(s);
                node_store.resolve(current).get_type().is_file()
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
                .map_err(|x| ScriptingError::Evaluation(x.to_string()))?;
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
            let node_store = &ns!(s);
            node_store.resolve(current).get_type().to_string()
        });
        let s = state.clone();
        acc_engine.register_fn("is_type_decl", move || {
            let node_store = &&ns!(s);
            node_store.resolve(current).get_type().is_type_declaration()
        });
        let s = state.clone();
        acc_engine.register_fn("is_directory", move || {
            let node_store = &&ns!(s);
            node_store.resolve(current).get_type().is_directory()
        });
        let s = state.clone();
        acc_engine.register_fn("is_file", move || {
            let node_store = &&ns!(s);
            node_store.resolve(current).get_type().is_file()
        });
        acc_engine
            .eval_ast_with_scope(&mut scope, &accumulate_script)
            .map_err(|x| ScriptingError::Evaluation(x.to_string()))?;
        stack[acc.parent].value = Some(scope.get_value("p").unwrap());
    };
    let r = format!(
        "Computed {result} in commit {} of size {size} at github.com/{user}/{name}",
        &commit[..8.min(commit.len())]
    );
    Ok(r)
}
