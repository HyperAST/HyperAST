#![feature(array_chunks)]
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::Duration, io,
};

use http::StatusCode;
use hyper_ast_cvs_git::multi_preprocessed::PreProcessedRepositories;
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
    tree::tree_path::{CompressedTreePath, TreePath}, algorithms::gumtree_lazy,
};
use tower_http::{services::ServeDir, cors::CorsLayer};

use crate::{
    app::{scripting_app, fetch_git_file, track_code_route, view_code_route, commit_metadata_route},
    examples::{example_app, kv_store_app},
    scripting::ScriptContent,
};
use axum::{body::{Bytes}, Router, routing::get_service, response::IntoResponse};
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
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

mod app;
mod examples;
mod scripting;
mod file;
mod track;
mod view;
mod commit;
#[derive(Default)]
pub struct AppState {
    db: HashMap<String, Bytes>,
    repositories: PreProcessedRepositories,
    mappings: HashMap<(NodeIdentifier,NodeIdentifier), gumtree_lazy::PersistableMappings<NodeIdentifier>>,
}

type SharedState = Arc<RwLock<AppState>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "client=debug,client::file=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let shared_state = SharedState::default();
    let app = Router::new()
        .fallback(fallback)
        .merge(kv_store_app(Arc::clone(&shared_state)))
        .merge(scripting_app(Arc::clone(&shared_state)))
        .merge(fetch_git_file(Arc::clone(&shared_state)))
        .merge(track_code_route(Arc::clone(&shared_state)))
        .merge(view_code_route(Arc::clone(&shared_state)))
        .merge(commit_metadata_route(Arc::clone(&shared_state)))
        .merge(example_app())
        .layer(CorsLayer::permissive())
        .with_state(Arc::clone(&shared_state));
    // TODOs auth admin to list pending constructions,
    // all repositories are blacklised by default
    // give provider per forge
    // to whitelist repositories either for all past commits or also all future commits
    // manage users and quota

    
    let local = [127, 0, 0, 1];
    let global = [0, 0, 0, 0];
    
    let addr = SocketAddr::from((global, 8080));
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

    let script = ScriptContent {
        init: r##"#{depth:0, files: 0, type_decl: 0}"##.to_string(),
        filter: r##"
if is_directory() {
    children().map(|x| {[x, #{depth: s.depth + 1, files: s.files, type_decl: s.type_decl}]})
} else if is_file() {
    children().map(|x| {[x, #{depth: s.depth + 1, type_decl: s.type_decl}]})
} else {
    []
}"##
        .to_string(),
        accumulate: r##"
if is_directory() {
    p.files += s.files;
    p.type_decl += s.type_decl;
} else if is_file() {
    p.files += 1;
    p.type_decl += s.type_decl;
} else if is_type_decl() {
    p.type_decl += 1; 
}"##
        .to_string(),
    };

    let req = req_build
        .timeout(Duration::from_secs(60 * 60))
        .header("content-type", "application/json")
        .body(serde_json::to_string(&script).unwrap())
        .build()?;
    let resp = client.execute(req)?;
    println!("{:#?}", resp.text()?);
    Ok(())
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
