mod estimate;
mod finalize;
mod fs_container;
mod max;
mod mean;
mod min;
mod named_container;
mod quantile;
#[cfg(feature = "impact")]
mod refs;
mod stats;

use crate::SharedState;
use average::Merge;
use axum::Json;
use hyperast::store::nodes::compo::Flags;
use hyperast::types::HyperAST;
use hyperast::{
    store::defaults::NodeIdentifier,
    types::{HyperType, LabelStore, Labeled, WithChildren, WithStats},
};
use rhai::{
    Array, Dynamic, Engine, Instant, Scope,
    packages::{BasicArrayPackage, CorePackage, Package},
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Deserialize, Clone)]
pub struct ScriptingParam {
    user: String,
    name: String,
    commit: String,
}

#[derive(Deserialize, Clone)]
pub struct ScriptContentDepth {
    #[serde(flatten)]
    inner: ScriptContent,
    commits: usize,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ScriptContent {
    pub init: String,
    pub accumulate: String,
    pub filter: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ScriptingError {
    AtCompilation(String),
    AtEvaluation(String),
    Other(String),
}

#[derive(Deserialize, Serialize)]
pub struct ComputeResult {
    pub compute_time: f64,
    pub result: Dynamic,
}

#[derive(Deserialize, Serialize)]
pub struct ComputeResultIdentified {
    pub commit: String,
    #[serde(flatten)]
    pub inner: ComputeResult,
}

impl Display for ComputeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Deserialize, Serialize)]
pub struct ComputeResults {
    pub prepare_time: f64,
    pub results: Vec<Result<ComputeResultIdentified, String>>,
}

impl Display for ComputeResults {
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
    let (commit, engine, init_script, accumulate_script, filter_script, mut repo) =
        simple_prepare(path, script, &state)?;
    let commits = state
        .repositories
        .write()
        .unwrap()
        .pre_process_with_limit(&mut repo, "", &commit, 2)
        .unwrap();
    log::info!("done construction of {commits:?} in  {}", repo.spec);

    let commit_oid = &commits[0];
    simple_aux(
        state,
        &repo,
        commit_oid,
        &engine,
        &init_script,
        &filter_script,
        &accumulate_script,
        now,
    )
    .map(|r| Json(r))
}

pub fn simple_depth(
    script: ScriptContentDepth,
    state: SharedState,
    path: ScriptingParam,
) -> Result<Json<ComputeResults>, ScriptingError> {
    let ScriptContentDepth {
        inner: script,
        commits,
    } = script;
    let now = Instant::now();
    let ScriptingParam { user, name, commit } = path.clone();
    let mut engine = Engine::new();
    engine.disable_symbol("/");
    add_utils(&mut engine);
    let init_script = engine.compile(script.init.clone()).map_err(|x| {
        ScriptingError::AtCompilation(format!("Init: {}, {}", x, script.init.clone()))
    })?;
    let filter_script = engine.compile(script.filter.clone()).map_err(|x| {
        ScriptingError::AtCompilation(format!("Filter: {}, {}", x, script.filter.clone()))
    })?;
    let accumulate_script = engine.compile(script.accumulate.clone()).map_err(|x| {
        ScriptingError::AtCompilation(format!("Acc: {}, {}", x, script.accumulate.clone()))
    })?;
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
    let prepare_time = now.elapsed().as_secs_f64();
    let mut results = vec![];
    for commit_oid in &commits {
        let now = Instant::now();
        let r = simple_aux(
            state.clone(),
            &repo,
            commit_oid,
            &engine,
            &init_script,
            &filter_script,
            &accumulate_script,
            now,
        );
        match r {
            Ok(r) => results.push(Ok(ComputeResultIdentified {
                commit: commit_oid.to_string(),
                inner: r,
            })),
            Err(ScriptingError::AtEvaluation(e)) => results.push(Err(e)),
            Err(e) => return Err(e),
        }
    }
    let r = ComputeResults {
        prepare_time,
        results,
    };
    Ok(Json(r))
}

fn simple_prepare(
    path: ScriptingParam,
    script: ScriptContent,
    state: &rhai::Shared<crate::AppState>,
) -> Result<
    (
        String,
        Engine,
        rhai::AST,
        rhai::AST,
        rhai::AST,
        hyperast_vcs_git::processing::ConfiguredRepo2,
    ),
    ScriptingError,
> {
    let ScriptingParam { user, name, commit } = path.clone();
    let mut engine = Engine::new();
    engine.disable_symbol("/");
    add_utils(&mut engine);
    let init_script = engine.compile(script.init.clone()).map_err(|x| {
        ScriptingError::AtCompilation(format!("Init: {}, {}", x, script.init.clone()))
    })?;
    let filter_script = engine.compile(script.filter.clone()).map_err(|x| {
        ScriptingError::AtCompilation(format!("Filter: {}, {}", x, script.filter.clone()))
    })?;
    let accumulate_script = engine.compile(script.accumulate.clone()).map_err(|x| {
        ScriptingError::AtCompilation(format!("Acc: {}, {}", x, script.accumulate.clone()))
    })?;
    let repo_spec = hyperast_vcs_git::git::Forge::Github.repo(user, name);
    let repo = state
        .repositories
        .write()
        .unwrap()
        .get_config(repo_spec)
        .ok_or_else(|| ScriptingError::Other("missing config for repository".to_string()))?;
    let repo = repo.fetch();
    log::warn!("done cloning {}", &repo.spec);
    Ok((
        commit,
        engine,
        init_script,
        accumulate_script,
        filter_script,
        repo,
    ))
}

fn simple_aux(
    state: rhai::Shared<crate::AppState>,
    repo: &hyperast_vcs_git::processing::ConfiguredRepo2,
    commit_oid: &hyperast_vcs_git::git::Oid,
    engine: &Engine,
    init_script: &rhai::AST,
    filter_script: &rhai::AST,
    accumulate_script: &rhai::AST,
    now: Instant,
) -> Result<ComputeResult, ScriptingError> {
    let repositories = state.repositories.read().unwrap();
    let commit_src = repositories.get_commit(&repo.config, commit_oid).unwrap();
    let src_tr = commit_src.ast_root;
    let node_store = &repositories.processor.main_stores.node_store;
    // let size = node_store.resolve(src_tr).size();
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

        let stack_len = stack.len();

        if acc.pending_cs < 0 {
            let mut scope = Scope::new();
            scope.push("s", acc.value.clone().unwrap());
            filter_engine.disable_symbol("/");
            let current = acc.sid;
            let s = state.clone();
            filter_engine.register_fn("type", move || {
                let stores = &stores!(s);
                let t = stores.resolve_type(&current);
                t.to_string()
            });
            let s = state.clone();
            filter_engine.register_fn("is_directory", move || {
                let stores = &stores!(s);
                let t = stores.resolve_type(&current);
                t.is_directory()
            });
            let s = state.clone();
            filter_engine.register_fn("is_type_decl", move || {
                let stores = &stores!(s);
                let t = stores.resolve_type(&current);
                let s = t.as_shared();
                s == hyperast::types::Shared::TypeDeclaration
            });
            let s = state.clone();
            filter_engine.register_fn("is_file", move || {
                let stores = &stores!(s);
                let t = stores.resolve_type(&current);
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
            let s = state.clone();
            filter_engine.register_fn("is_java_file", move || {
                let stores = &stores!(s);
                let node_store = &stores.node_store;
                let n = node_store.resolve(current);
                let t = stores.resolve_type(&current);
                t.is_file()
                    && stores
                        .label_store
                        .resolve(n.get_label_unchecked())
                        .ends_with(".java")
            });
            let s = state.clone();
            filter_engine.register_fn("file_name", move || {
                let stores = &stores!(s);
                let node_store = &stores.node_store;
                let n = node_store.resolve(current);
                let t = stores.resolve_type(&current);
                if t.is_file() || t.is_directory() {
                    Ok(stores
                        .label_store
                        .resolve(n.get_label_unchecked())
                        .to_string())
                } else {
                    Err(Box::<rhai::EvalAltResult>::from(
                        "file_name() should be called on a file or a directory",
                    ))
                }
            });
            let s = state.clone();
            filter_engine.register_fn("is_maven_module", move || {
                let stores = &stores!(s);
                let node_store = &stores.node_store;
                let n = node_store.resolve(current);
                use enumset::EnumSet;
                use hyperast_vcs_git::maven::SemFlag;
                n.get_component::<Flags<EnumSet<SemFlag>>>()
                    .map_or(false, |x| x.contains(SemFlag::IsMavenModule))
            });
            let s = state.clone();
            filter_engine.register_fn("hold_maven_submodule", move || {
                let stores = &stores!(s);
                let node_store = &stores.node_store;
                let n = node_store.resolve(current);
                use enumset::EnumSet;
                use hyperast_vcs_git::maven::SemFlag;
                n.get_component::<Flags<EnumSet<SemFlag>>>()
                    .map_or(false, |x| x.contains(SemFlag::HoldMavenSubModule))
            });
            let s = state.clone();
            filter_engine.register_fn("hold_java_folder", move || {
                let stores = &stores!(s);
                let node_store = &stores.node_store;
                let n = node_store.resolve(current);
                use enumset::EnumSet;
                use hyperast_vcs_git::maven::SemFlag;
                n.get_component::<Flags<EnumSet<SemFlag>>>()
                    .map_or(false, |x| {
                        x.contains(SemFlag::HoldMainFolder) || x.contains(SemFlag::HoldTestFolder)
                    })
            });
            add_utils(&mut filter_engine);
            let prepared: Dynamic = filter_engine
                .eval_ast_with_scope(&mut scope, &filter_script)
                .map_err(|x| ScriptingError::AtEvaluation(x.to_string()))?;
            acc.value = Some(scope.get_value("s").unwrap());
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
            let t = stores.resolve_type(&current);
            t.to_string()
        });
        let s = state.clone();
        acc_engine.register_fn("is_type_decl", move || {
            let stores = &stores!(s);
            let t = stores.resolve_type(&current);
            let s = t.as_shared();
            s == hyperast::types::Shared::TypeDeclaration
        });
        let s = state.clone();
        acc_engine.register_fn("is_directory", move || {
            let stores = &stores!(s);
            let t = stores.resolve_type(&current);
            t.is_directory()
        });
        let s = state.clone();
        acc_engine.register_fn("is_file", move || {
            let stores = &stores!(s);
            let t = stores.resolve_type(&current);
            t.is_file()
        });
        let s = state.clone();
        acc_engine.register_fn("is_java_file", move || {
            let stores = &stores!(s);
            let node_store = &stores.node_store;
            let n = node_store.resolve(current);
            let t = stores.resolve_type(&current);
            t.is_file()
                && stores
                    .label_store
                    .resolve(n.get_label_unchecked())
                    .ends_with(".java")
        });
        let s = state.clone();
        acc_engine.register_fn("file_name", move || {
            let stores = &stores!(s);
            let node_store = &stores.node_store;
            let n = node_store.resolve(current);
            let t = stores.resolve_type(&current);
            if t.is_file() || t.is_directory() {
                Ok(stores
                    .label_store
                    .resolve(n.get_label_unchecked())
                    .to_string())
            } else {
                Err(Box::<rhai::EvalAltResult>::from(
                    "file_name() should be called on a file or a directory",
                ))
            }
        });
        let s = state.clone();
        acc_engine.register_fn("is_maven_module", move || {
            let stores = &stores!(s);
            let node_store = &stores.node_store;
            let n = node_store.resolve(current);
            use enumset::EnumSet;
            use hyperast_vcs_git::maven::SemFlag;
            n.get_component::<Flags<EnumSet<SemFlag>>>()
                .map_or(false, |x| x.contains(SemFlag::IsMavenModule))
        });
        let s = state.clone();
        acc_engine.register_fn("hold_maven_submodule", move || {
            let stores = &stores!(s);
            let node_store = &stores.node_store;
            let n = node_store.resolve(current);
            use enumset::EnumSet;
            use hyperast_vcs_git::maven::SemFlag;
            n.get_component::<Flags<EnumSet<SemFlag>>>()
                .map_or(false, |x| x.contains(SemFlag::HoldMavenSubModule))
        });
        let s = state.clone();
        acc_engine.register_fn("hold_java_folder", move || {
            let stores = &stores!(s);
            let node_store = &stores.node_store;
            let n = node_store.resolve(current);
            use enumset::EnumSet;
            use hyperast_vcs_git::maven::SemFlag;
            n.get_component::<Flags<EnumSet<SemFlag>>>()
                .map_or(false, |x| {
                    x.contains(SemFlag::HoldMainFolder) || x.contains(SemFlag::HoldTestFolder)
                })
        });
        #[cfg(feature = "impact")]
        {
            let s = state.clone();
            acc_engine.register_fn("references", move |sig: String, p_ref: String| {
                let stores = &stores!(s);
                refs::find_refs(stores, current, &p_ref, &sig).map_or(0, |x| x as i64)
            });

            let s = state.clone();
            acc_engine.register_fn(
                "pp",
                |s: refs::QPath,
                 node: NodeIdentifier,
                 i: i64|
                 -> Result<refs::Pos, Box<rhai::EvalAltResult>> {
                    // let stores = &stores!(s);
                    // let it = s.0;
                    // let position_converter =
                    //     &hyperast::position::PositionConverter::new(&it).with_stores(stores);
                    // let p = position_converter
                    //     .compute_pos_post_order::<_, hyperast::position::Position, _>();
                    // Ok(refs::Pos::from(p))
                    todo!("need to choose a convenient path, try to exploit param overloading")
                },
            );
        }
        add_utils(&mut acc_engine);
        acc_engine
            .eval_ast_with_scope(&mut scope, &accumulate_script)
            .map_err(|x| ScriptingError::AtEvaluation(x.to_string()))?;
        stack[acc.parent].value = Some(scope.get_value("p").unwrap());
    };
    let compute_time = now.elapsed().as_secs_f64();
    let result = result.finalize();
    let r = ComputeResult {
        compute_time,
        result,
    };
    Ok(r)
}

use self::{max::Max, mean::Mean, min::Min, quantile::Quantile, stats::Stats};
use finalize::Finalize;

fn add_utils(engine: &mut Engine) {
    engine
        .register_type_with_name::<Mean>("Mean")
        .register_fn("Mean", Mean::default)
        .register_fn("+=", |x: &mut Mean, y: Mean| {
            x.merge(&y);
        })
        .register_fn("+=", |m: &mut Mean, x: i64| m.add_i64(x));

    engine
        .register_type_with_name::<Max>("Max")
        .register_fn("Max", Max::default)
        .register_fn("+=", |x: &mut Max, y: Max| {
            x.merge(&y);
        })
        .register_fn("+=", |m: &mut Max, x: i64| m.add_i64(x));

    engine
        .register_type_with_name::<Min>("Min")
        .register_fn("Min", Min::default)
        .register_fn("+=", |x: &mut Min, y: Min| {
            x.merge(&y);
        })
        .register_fn("+=", |m: &mut Min, x: i64| m.add_i64(x));

    engine
        .register_type_with_name::<Quantile>("Quantile")
        .register_fn("Quantile", Quantile::new)
        .register_fn("Median", || Quantile::new(0.5))
        .register_fn("+=", |x: &mut Quantile, y: Quantile| {
            x.merge(&y);
        })
        .register_fn("+=", |m: &mut Quantile, x: i64| m.add_i64(x));

    engine
        .register_type_with_name::<Stats>("Stats")
        .register_fn("Stats", Stats::new)
        .register_fn("+=", |x: &mut Stats, y: Stats| {
            x.merge(&y);
        })
        .register_fn("+=", |m: &mut Stats, x: i64| m.add_i64(x));

    use named_container::NamedContainer;
    engine
        .register_type_with_name::<NamedContainer<Dynamic>>("NamedCont")
        .register_fn("NamedCont", NamedContainer::<Dynamic>::new)
        .register_fn(
            "+=",
            |context: rhai::NativeCallContext,
             x: &mut NamedContainer<Dynamic>,
             y: Dynamic|
             -> Result<(), Box<rhai::EvalAltResult>> {
                let cont = std::mem::replace(&mut x.content, Dynamic::UNIT);
                context.call_native_fn("+", (cont, y)).map(|y| {
                    x.content = y;
                })
            },
        )
        .register_fn(
            "is_empty",
            |context: rhai::NativeCallContext,
             x: &mut NamedContainer<Dynamic>|
             -> Result<bool, Box<rhai::EvalAltResult>> {
                context
                    .call_native_fn("is_empty", (x.content.clone(),))
                    .map(|r| r)
            },
        );
    use fs_container::FsContainer;
    engine
        .register_type_with_name::<FsContainer<Dynamic>>("FsCont")
        .register_fn("FsCont", FsContainer::<Dynamic>::new)
        .register_fn(
            "+=",
            |context: rhai::NativeCallContext,
             x: &mut FsContainer<Dynamic>,
             mut y: Dynamic|
             -> Result<(), Box<rhai::EvalAltResult>> {
                // let mut cont = std::mem::replace(&mut x.content, Dynamic::UNIT);
                // dbg!(&cont, &y);
                // context.call_native_fn("+=", (cont, y))
                // .map(|y|{
                //     x.content = y;
                // })
                let cont = &mut x.content;
                context
                    .call_native_fn_raw("+=", true, &mut [cont, &mut y])
                    .and_then(|y| y.as_unit().map_err(|e| e.into()))
            },
        )
        .register_fn(
            "is_empty",
            |context: rhai::NativeCallContext,
             x: &mut FsContainer<Dynamic>|
             -> Result<bool, Box<rhai::EvalAltResult>> {
                context
                    .call_native_fn("is_empty", (x.content.clone(),))
                    .map(|r| r)
            },
        );
    #[cfg(feature = "impact")]
    engine
        .register_type_with_name::<refs::QPath>("Path")
        .register_fn("Path", refs::QPath::new)
        .register_fn(
            "goto",
            |s: &mut refs::QPath,
             node: NodeIdentifier,
             i: i64|
             -> Result<(), Box<rhai::EvalAltResult>> {
                let i = i.to_u16().ok_or(concat!(
                    "given child offset is too big,",
                    "you most likely made an error,",
                    "otherwise change the configured offset size"
                ))?;
                Ok(s.goto(node, i))
            },
        );
}
