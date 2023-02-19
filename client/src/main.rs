#![feature(array_chunks)]
use std::{
    collections::HashMap,
    fmt::Display,
    net::SocketAddr,
    sync::{atomic::AtomicI64, Arc, RwLock},
    thread,
    time::Duration,
};

use hyper_ast_cvs_git::{
    git::fetch_github_repository, multi_preprocessed::PreProcessedRepositories,
};
use hyper_diff::{
    actions::{
        action_vec::apply_action,
        script_generator2::{Act, ScriptGenerator, SimpleAction},
    },
    decompressed_tree_store::{bfs_wrapper, CompletePostOrder, SimpleZsTree},
    matchers::{
        heuristic::gt::{
            bottom_up_matcher::BottomUpMatcher, greedy_bottom_up_matcher::GreedyBottomUpMatcher,
            greedy_subtree_matcher::SubtreeMatcher,
        },
        mapping_store::{DefaultMappingStore, DefaultMultiMappingStore, VecStore},
        optimal::zs::ZsMatcher,
    },
    tree::tree_path::{CompressedTreePath, TreePath},
};

use hyper_ast::{
    cyclomatic::{Mcc, MetaData},
    hashed::HashedNode,
    store::{
        labels::LabelStore,
        nodes::{
            legion::{HashedNodeRef, NodeIdentifier},
            DefaultNodeStore as NodeStore,
        },
        SimpleStores, TypeStore,
    },
    types::{Type, Typed, WithChildren},
};
use hyper_ast_gen_ts_java::legion_with_refs::{
    print_tree_ids, print_tree_syntax, print_tree_syntax_with_ids, JavaTreeGen,
};
use hyper_diff::matchers::heuristic::gt::greedy_subtree_matcher::GreedySubtreeMatcher;
use rhai::{Array, Dynamic, Engine, Map, Scope};
use serde::Deserialize;

#[derive(Debug)]
enum ScriptingError {
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

impl IntoResponse for ScriptingError {
    fn into_response(self) -> Response {
        self.to_string().into_response()
    }
}

#[derive(Default)]
struct AppState {
    db: HashMap<String, Bytes>,
    repositories: PreProcessedRepositories,
}

#[derive(Deserialize, Clone)]
struct ScriptingParam {
    user: String,
    name: String,
    commit: String,
}

#[derive(Deserialize)]
struct ScriptContent {
    init: String,
    accumulate: String,
    filter: String,
}

// #[axum_macros::debug_handler]
async fn scripting(
    axum::extract::Path(path): axum::extract::Path<ScriptingParam>,
    axum::extract::State(state): axum::extract::State<SharedState>,
    axum::extract::Json(script): axum::extract::Json<ScriptContent>,
) -> axum::response::Result<String> {
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
    use hyper_ast::types::WithStats;
    let node_store = &get_mut.repositories.processor.main_stores.node_store;
    let size = node_store.resolve(src_tr).size();

    drop(get_mut);

    macro_rules! ns {
        ($s:expr) => {
            $s.write()
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
    let result: Dynamic = loop {
        let Some(mut acc) = stack.pop() else {
            unreachable!()
        };
        let stack_len = stack.len();
        // dbg!(&acc);
        if acc.pending_cs < 0 {
            let mut engine = Engine::new();
            let mut scope = Scope::new();
            scope.push(
                "s",
                acc.value.clone().unwrap(), //_or(Default::default()),
            );
            engine.disable_symbol("/");
            let current = acc.sid;
            let s = state.clone();
            engine.register_fn("is_directory", move || {
                let node_store = &&ns!(s);
                node_store.resolve(current).get_type().is_directory()
            });
            let s = state.clone();
            engine.register_fn("is_type_decl", move || {
                let node_store = &&ns!(s);
                node_store.resolve(current).get_type().is_type_declaration()
            });
            let s = state.clone();
            engine.register_fn("is_file", move || {
                let node_store = &&ns!(s);
                node_store.resolve(current).get_type().is_file()
            });
            let s = state.clone();
            engine.register_fn("children", move || {
                let node_store = &ns!(s);
                node_store
                    .resolve(current)
                    .children()
                    .map_or(Default::default(), |v| {
                        v.0.iter().map(|x| Dynamic::from(*x)).collect::<Array>()
                    })
            });
            let prepared: Dynamic = engine
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
        let mut engine = Engine::new();
        let mut scope = Scope::new();
        scope.push("s", acc.value.take().unwrap()); //_or(Default::default()));
        scope.push(
            "p",
            stack[acc.parent].value.take().unwrap(), //_or(Default::default()),
        );
        engine.disable_symbol("/");
        let current = acc.sid;
        let s = state.clone();
        engine.register_fn("size", move || {
            let node_store = &ns!(s);
            node_store.resolve(current).size() as i64
        });
        let s = state.clone();
        engine.register_fn("type", move || {
            let node_store = &ns!(s);
            node_store.resolve(current).get_type().to_string()
        });
        let s = state.clone();
        engine.register_fn("is_type_decl", move || {
            let node_store = &&ns!(s);
            node_store.resolve(current).get_type().is_type_declaration()
        });
        let s = state.clone();
        engine.register_fn("is_directory", move || {
            let node_store = &&ns!(s);
            node_store.resolve(current).get_type().is_directory()
        });
        let s = state.clone();
        engine.register_fn("is_file", move || {
            let node_store = &&ns!(s);
            node_store.resolve(current).get_type().is_file()
        });
        engine
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

use axum::{
    body::Bytes,
    error_handling::HandleErrorLayer,
    extract::DefaultBodyLimit,
    handler::Handler,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use tower::{limit::ConcurrencyLimitLayer, BoxError, ServiceBuilder};
use tower_http::{
    compression::CompressionLayer, limit::RequestBodyLimitLayer, trace::TraceLayer,
    ServiceBuilderExt,
};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

/// axum handler for "GET /" which returns a string and causes axum to
/// immediately respond with status code `200 OK` and with the string.
pub async fn hello() -> String {
    "Hello, World!".into()
}

/// axum handler for any request that fails to match the router routes.
/// This implementation returns HTTP status code Not Found (404).
pub async fn fallback(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::NOT_FOUND,
        format!("No route {}", uri),
    )
}

/// Tokio signal handler that will wait for a user to press CTRL+C.
/// We use this in our hyper `Server` method `with_graceful_shutdown`.
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("expect tokio signal ctrl-c");
    println!("signal shutdown");
}

type SharedState = Arc<RwLock<AppState>>;

async fn kv_get(
    axum::extract::Path(key): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> Result<Bytes, hyper::StatusCode> {
    let db = &state.read().unwrap().db;

    if let Some(value) = db.get(&key) {
        Ok(value.clone())
    } else {
        Err(hyper::StatusCode::NOT_FOUND)
    }
}

async fn kv_set(
    axum::extract::Path(key): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<SharedState>,
    bytes: Bytes,
) {
    state.write().unwrap().db.insert(key, bytes);
}

async fn list_keys(axum::extract::State(state): axum::extract::State<SharedState>) -> String {
    let db = &state.read().unwrap().db;

    db.keys()
        .map(|key| key.to_string())
        .collect::<Vec<String>>()
        .join("\n")
}

fn example_app() -> Router<SharedState> {
    Router::new().route("/", get(hello))
    // .route("/demo.html", get(get_demo_html))
    // .route("/hello.html", get(hello_html))
    // .route("/demo-status", get(demo_status))
    // .route("/demo-uri", get(demo_uri))
    // .route("/demo.png", get(get_demo_png))
    // .route(
    //     "/foo",
    //     get(get_foo)
    //         .put(put_foo)
    //         .patch(patch_foo)
    //         .post(post_foo)
    //         .delete(delete_foo),
    // )
    // .route("/items/:id", get(get_items_id))
    // .route("/items", get(get_items))
    // .route("/demo.json", get(get_demo_json).put(put_demo_json))
}

fn kv_store_app(st: SharedState) -> Router<SharedState> {
    Router::new()
        .route(
            "/:key",
            // Add compression to `kv_get`
            get(kv_get.layer(CompressionLayer::new()))
                // But don't compress `kv_set`
                .post_service(
                    kv_set
                        .layer((
                            DefaultBodyLimit::disable(),
                            RequestBodyLimitLayer::new(1024 * 5_000 /* ~5mb */),
                            ConcurrencyLimitLayer::new(1),
                        ))
                        .with_state(st),
                ),
        )
        .route("/keys", get(list_keys))
}

fn scripting_app(_st: SharedState) -> Router<SharedState> {
    let scripting_service_config = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            dbg!(e);
        }))
        .load_shed()
        .concurrency_limit(16)
        .buffer(200)
        .rate_limit(10, Duration::from_secs(5))
        .request_body_limit(1024 * 5_000 /* ~5mb */)
        .timeout(Duration::from_secs(10))
        .layer(TraceLayer::new_for_http());
    Router::new()
        .route(
            "/script/github/:user/:name/:commit",
            post(scripting).layer(scripting_service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/script/gitlab/:user/:name/:commit",
            post(scripting).layer(scripting_service_config), // .with_state(Arc::clone(&shared_state)),
        )
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "client=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let shared_state = SharedState::default();
    let app = Router::new()
        .fallback(fallback)
        .merge(kv_store_app(Arc::clone(&shared_state)))
        .merge(scripting_app(Arc::clone(&shared_state)))
        .merge(example_app())
        .with_state(Arc::clone(&shared_state));
    // TODOs auth admin to list pending constructions,
    // all repositories are blacklised by default
    // give provider per forge
    // to whitelist repositories either for all past commits or also all future commits
    // manage users and quota

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

#[test]
fn test_scripting() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::default();
    let req_build = client.post(
        "http://localhost:8080/script/github/INRIA/spoon/4acedc53a13a727be3640fe234f7e261d2609d58",
    );
    let req = req_build
    .timeout(Duration::from_secs(60*60))
    .header("content-type", "application/json")
    .body(r##"{
"init": "#{depth:0, files: 0, type_decl: 0}",
"filter": "if is_directory() { children().map(|x| {[x, #{depth: s.depth + 1, files: s.files, type_decl: s.type_decl}]}) } else if is_file() { children().map(|x| {[x, #{depth: s.depth + 1, type_decl: s.type_decl}]}) } else {[]}",
"accumulate":"if is_directory() { p.files += s.files; p.type_decl += s.type_decl; } else if is_file() { p.files += 1; p.type_decl += s.type_decl; } else if is_type_decl() { p.type_decl += 1; }"
}"##).build()?;
    let resp = client.execute(req)?;
    println!("{:#?}", resp.text()?);
    Ok(())
}

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     HttpServer::new(move || App::new().service(scripting))
//         .bind(("127.0.0.1", 8080))
//         .unwrap()
//         .run()
//         .await
// }

// mod identity;

// async fn main3() -> std::io::Result<()> {
//     // let secret_key = cookie::Key::generate();
//     // let redis_store = actix_session::storage::RedisSessionStore::new("redis://127.0.0.1:6379")
//     //     .await
//     //     .unwrap();
//     HttpServer::new(move || {
//         // let auth = HttpAuthentication::basic(|req, _credentials| async { dbg!();Ok(req) });
//         App::new()
//             // .wrap(middleware::Logger::default())
//             // .wrap(auth)
//             // .wrap(
//             //     IdentityMiddleware::builder()
//             //         .login_deadline(Some(Duration::from_secs(60 * 60)))
//             //         .visit_deadline(Some(Duration::from_secs(10 * 60)))
//             //         .build(),
//             // )
//             // .wrap(SessionMiddleware::new(
//             //     redis_store.clone(),
//             //     secret_key.clone(),
//             // ))
//             // .service(identity::index)
//             // .service(identity::login)
//             // .service(identity::logout)
//             .service(scripting)
//     })
//     .bind(("127.0.0.1", 8080))
//     .unwrap()
//     .run()
//     .await
// }

fn main2() {
    // TODO fix stores and cache should not be leaked to make them static
    // It is requested by the type checker.
    // It seems caused by apply_actions in combination with implementation of NodeStoreExt2 for JavaTreeGen
    // They use HTBRs and JavaTreeGen has lifetimes for stores and md_cache (not owned).
    // I believe the borrow checker is wrong, and fail reduce the lifetime.
    let stores = Box::new(SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    });
    let md_cache = Box::new(Default::default());
    let mut java_tree_gen = JavaTreeGen::<'static, '_> {
        line_break: "\n".as_bytes().to_vec(),
        stores: Box::leak(stores),
        md_cache: Box::leak(md_cache),
    };
    // let case1 = CASE_1;
    // let case2 = CASE_1;

    let case1 = CASE_BIG1;
    let case2 = CASE_BIG2;

    let tree = match JavaTreeGen::tree_sitter_parse(case1.as_bytes()) {
        Ok(t) => t,
        Err(t) => t,
    };
    println!("{}", tree.root_node().to_sexp());
    let full_node1 = java_tree_gen.generate_file(b"", case1.as_bytes(), tree.walk());

    let tree = match JavaTreeGen::tree_sitter_parse(case2.as_bytes()) {
        Ok(t) => t,
        Err(t) => t,
    };
    println!("{}", tree.root_node().to_sexp());
    let full_node2 = java_tree_gen.generate_file(b"", case2.as_bytes(), tree.walk());
    // let JavaTreeGen {
    //     mut stores,
    //     mut md_cache,
    //     ..
    // } = java_tree_gen;
    print_tree_syntax(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &full_node1.local.compressed_node,
    );
    println!();
    print_tree_syntax(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &full_node2.local.compressed_node,
    );
    println!();
    print_tree_ids(
        &java_tree_gen.stores.node_store,
        &full_node1.local.compressed_node,
    );
    println!();
    print_tree_ids(
        &java_tree_gen.stores.node_store,
        &full_node2.local.compressed_node,
    );
    println!();

    dbg!(java_tree_gen
        .stores
        .node_store
        .resolve(full_node1.local.compressed_node)
        .get_type());
    dbg!(&Mcc::retrieve(
        &java_tree_gen
            .stores
            .node_store
            .resolve(full_node1.local.compressed_node)
    ));

    let src = full_node1.local.compressed_node;
    let dst = full_node2.local.compressed_node;

    let actions = {
        // GreedySubtreeMatcher.MIN_HEIGHT = 0;
        // GreedyBottomUpMatcher
        {
            let mapper = ZsMatcher::<DefaultMappingStore<u16>, SimpleZsTree<_, _>>::matchh(
                &java_tree_gen.stores.node_store,
                &java_tree_gen.stores.label_store,
                src,
                dst,
            );
            let ZsMatcher {
                src_arena: _,
                dst_arena: _,
                mappings: ms,
                ..
            } = mapper;

            dbg!(ms);
        }
        let mappings: VecStore<u16> = DefaultMappingStore::default();
        let mapper = GreedySubtreeMatcher::<
            CompletePostOrder<_, u16>,
            CompletePostOrder<_, u16>,
            _,
            _, // HashedNodeRef,
            _,
            // 2,
        >::matchh::<DefaultMultiMappingStore<_>>(
            &java_tree_gen.stores.node_store,
            &src,
            &dst,
            mappings,
        );
        let SubtreeMatcher {
            src_arena,
            dst_arena,
            mappings,
            ..
        } = mapper.into();
        // let mapper = GreedyBottomUpMatcher::<
        //     CompletePostOrder<_, u16>,
        //     CompletePostOrder<_, u16>,
        //     _,
        //     HashedNodeRef,
        //     _,
        //     _,
        //     1000,
        //     1,
        //     2,
        // >::matchh(
        //     &java_tree_gen.stores.node_store,
        //     &java_tree_gen.stores.label_store,
        //     &src,
        //     &dst,
        //     mappings,
        // );
        let mut mapper = GreedyBottomUpMatcher::<
            CompletePostOrder<_, u16>,
            CompletePostOrder<_, u16>,
            _,
            _,
            _,
            _,
            // 1000,
            // 1,
            // 2,
        >::new(
            &java_tree_gen.stores.node_store,
            &java_tree_gen.stores.label_store,
            src_arena,
            dst_arena,
            mappings,
        );
        mapper.execute();
        let BottomUpMatcher {
            src_arena,
            dst_arena,
            mappings,
            ..
        } = mapper.into();
        println!("ms={:?}", mappings);
        // println!("{:?} {:?}", dst_arena.root(), dst);
        // println!("{:?}", dst_arena);
        // println!(
        //     "{:?}",
        //     dst_arena
        //         .iter_df_post()
        //         .map(|id: u16| dst_arena.original(&id))
        //         .collect::<Vec<_>>()
        // );
        let dst_arena =
            bfs_wrapper::SimpleBfsMapper::from(&java_tree_gen.stores.node_store, dst_arena);
        // println!("{:?} {:?}", dst_arena.root(), dst);
        // println!("{:?}", dst_arena);
        // println!(
        //     "{:?}",
        //     dst_arena
        //         .iter_bf()
        //         .map(|id| dst_arena.original(&id))
        //         .collect::<Vec<_>>()
        // );
        let script_gen = ScriptGenerator::<
            _,
            HashedNodeRef,
            _,
            _, // bfs_wrapper::SD<_, _, CompletePostOrder<_, u16>>,
            NodeStore,
            _,
            _,
        >::precompute_actions(
            &java_tree_gen.stores.node_store,
            &src_arena,
            &dst_arena,
            &mappings,
        )
        .generate()
        .unwrap();

        let ScriptGenerator {
            store: _, actions, ..
        } = script_gen;
        actions
        // ActionsVec(vec![])
    };

    // /// TODO try to not store intermediate nodes permanently.
    // let mut stores = stores;
    // let mut md_cache = md_cache;

    // let mut stores = SimpleStores {
    //     label_store: LabelStore::new(),
    //     type_store: TypeStore {},
    //     node_store: NodeStore::new(),
    // };
    // let mut md_cache = Default::default();
    // let mut java_tree_gen = JavaTreeGen {
    //     line_break: "\n".as_bytes().to_vec(),
    //     stores: &mut stores,
    //     md_cache: &mut md_cache,
    // };

    fn access(store: &NodeStore, r: NodeIdentifier, p: &CompressedTreePath<u16>) -> NodeIdentifier {
        let mut x = r;
        for p in p.iter() {
            x = store.resolve(x).child(&p).unwrap();
        }
        x
    }

    // println!("{:?}", actions.len());
    let mut root = vec![src];
    for x in actions.iter() {
        use hyper_ast::types::LabelStore;
        let SimpleAction { path, action } = x;
        let id = access(
            &java_tree_gen.stores.node_store,
            if let Act::Delete {} = action {
                src
            } else {
                dst
            },
            &path.ori,
        );
        if java_tree_gen.stores.node_store.resolve(id).get_type() != Type::Spaces {
            match action {
                Act::Delete {} => {
                    print!("del {:?} ", path);
                    let id = access(&java_tree_gen.stores.node_store, src, &path.ori);
                    print_tree_syntax(
                        &java_tree_gen.stores.node_store,
                        &java_tree_gen.stores.label_store,
                        &id,
                    );
                    println!();
                }
                Act::Update { new } => println!(
                    "upd {:?} {:?}",
                    java_tree_gen.stores.label_store.resolve(new),
                    path
                ),
                Act::Move { from } => {
                    print!("mov {:?} {:?}", from, path);
                    let id = access(&java_tree_gen.stores.node_store, src, &from.ori);
                    print_tree_syntax(
                        &java_tree_gen.stores.node_store,
                        &java_tree_gen.stores.label_store,
                        &id,
                    );
                    println!();
                }
                Act::MovUpd { from, new } => {
                    println!(
                        "mou {:?} {:?} {:?}",
                        java_tree_gen.stores.label_store.resolve(new),
                        from,
                        path
                    )
                }
                Act::Insert { sub } => {
                    print!("ins {:?} ", path);
                    print_tree_syntax(
                        &java_tree_gen.stores.node_store,
                        &java_tree_gen.stores.label_store,
                        sub,
                    );
                    println!();
                }
            }
        }
        // java_tree_gen2.apply_action(x, &mut root);
        apply_action::<HashedNode, JavaTreeGen<'_, '_>, _>(x, &mut root, &mut java_tree_gen);
        // java_tree_gen2.build_then_insert(todo!(), todo!(), todo!());
    }
    let then = root; //ActionsVec::apply_actions(actions.iter(), *src, &mut node_store);

    print_tree_syntax_with_ids(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &dst,
    );
    println!();
    print_tree_syntax_with_ids(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        then.last().unwrap(),
    );
    println!();
    // print_tree_ids(
    //     &java_tree_gen.stores.node_store,
    //     &full_node1.local.compressed_node,
    // );
    // println!();
    // print_tree_ids(
    //     &java_tree_gen.stores.node_store,
    //     &full_node2.local.compressed_node,
    // );
    // println!();
    assert_eq!(*then.last().unwrap(), dst);

    // println!();
    // print_tree_syntax(
    //     &java_tree_gen.stores.node_store,
    //     &java_tree_gen.stores.label_store,
    //     &full_node.local.compressed_node,
    // );
    // println!();
    // stdout().flush().unwrap();

    // let mut out = IoOut { stream: stdout() };
    // serialize(
    //     &java_tree_gen.stores.node_store,
    //     &java_tree_gen.stores.label_store,
    //     &full_node.local.compressed_node,
    //     &mut out,
    //     &std::str::from_utf8(&java_tree_gen.line_break).unwrap(),
    // );

    //     use hyper_ast_gen_ts_java::java_tree_gen_no_compress_arena::{JavaTreeGen, LabelStore, NodeStore,SimpleStores,HashedNode};
    //     // tree_sitter_cli::generate::parse_grammar;

    //     println!("Hello, world!");

    //     let mut parser = Parser::new();

    //     {
    //         let language = unsafe { tree_sitter_java() };
    //         parser.set_language(language).unwrap();
    //     }

    //     let mut java_tree_gen = JavaTreeGen::new();

    //     // src
    //     let text = {
    //         let source_code1 = "class A {
    //     class B {
    //         int a = 0xffff;
    //     }
    // }";
    //         source_code1.as_bytes()
    //     };
    //     let tree = parser.parse(text, None).unwrap();
    //     println!("{}", tree.root_node().to_sexp());

    //     let full_node_src = java_tree_gen.generate_default(text, tree.walk());

    //     println!("debug full node 1: {:?}", &full_node_src);

    //     // dst
    //     let text = {
    //         let source_code1 = "class A {
    //     class C {
    //         int a = 0xffff;
    //     }
    // }";
    //         source_code1.as_bytes()
    //     };
    //     let tree = parser.parse(text, None).unwrap();
    //     println!("{}", tree.root_node().to_sexp());

    //     let full_node_dst = java_tree_gen.generate_default(text, tree.walk());

    //     println!("debug full node 2: {:?}", &full_node_dst);

    //     let JavaTreeGen {
    //         line_break: _,
    //         stores : SimpleStores {
    //             node_store,
    //             label_store,
    //             type_store: _,
    //         } } = java_tree_gen;

    //     let mapping_store = DefaultMappingStore::default();
    //     // let a = SimpleBottomUpMatcher::<
    //     let a = ZsMatcher::<
    //         CompletePostOrder<_, u16>,
    //         HashedNode,
    //         u16,
    //         NodeStore,
    //         LabelStore,
    //     >::matchh(
    //         &node_store,
    //         &label_store,
    //         *full_node_src.local().id(),
    //         *full_node_dst.local().id(),
    //         mapping_store,
    //     );
    //     a.mappings
    //         .src_to_dst
    //         .iter()
    //         .map(|x| if *x == 0 { None } else { Some(*x - 1) })
    //         .zip(
    //             a.mappings
    //                 .dst_to_src
    //                 .iter()
    //                 .map(|x| if *x == 0 { None } else { Some(*x - 1) }),
    //         )
    //         .enumerate()
    //         .for_each(|x| println!("{:?}", x));
    //     // a.src_to_dst.iter().enumerate().for_each(|(i,m)| {
    //     //     println!("{:?}", (i,m,&a.dst_to_src[*m as usize]));
    //     // });
    //     // println!("-----------");
    //     // a.dst_to_src.iter().enumerate().for_each(|(i,m)| {
    //     //     println!("{:?}", (i,m,&a.src_to_dst[*m as usize]));
    //     // });

    //     // // let mut out = String::new();
    //     // let mut out = IoOut {
    //     //     out: stdout()
    //     // };
    //     // serialize(
    //     //     &java_tree_gen.node_store,
    //     //     &java_tree_gen.label_store,
    //     //     &full_node.id(),
    //     //     &mut out,
    //     //     &std::str::from_utf8(&java_tree_gen.line_break).unwrap(),
    //     // );
    //     // println!();
    //     // print_tree_syntax(
    //     //     &java_tree_gen.node_store,
    //     //     &java_tree_gen.label_store,
    //     //     &full_node.id(),
    //     // );
    //     // println!();
    //     // stdout().flush().unwrap();
}

static CASE_BIG1: &'static str = r#"class A{class C{}class B{{while(1){if(1){}else{}};}}}class D{class E{}class F{{while(2){if(2){}else{}};}}}"#;

static CASE_BIG2: &'static str = r#"class A{class C{}}class B{{while(1){if(1){}else{}};}}class D{class E{}}class F{{while(2){if(2){}else{}};}}"#;
