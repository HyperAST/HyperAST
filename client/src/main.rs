use hyper_gumtree::{
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
        mapping_store::{DefaultMappingStore, VecStore, DefaultMultiMappingStore},
        optimal::zs::ZsMatcher,
    },
    tree::tree_path::{CompressedTreePath, TreePath},
};

// fn main_compress() {
//     // use hyper_ast_gen_ts_java::java_tree_gen_full_compress::{
//     //     JavaTreeGen, LabelStore, NodeStore, SimpleStores,
//     // };
//     //     println!("Hello, world!");

//     //     let mut parser = Parser::new();

//     //     {
//     //         let language = unsafe { tree_sitter_java() };
//     //         parser.set_language(language).unwrap();
//     //     }

//     //     let mut java_tree_gen = JavaTreeGen::new();

//     //     // src
//     //     let text = {
//     //         let source_code1 = "class A {
//     //     class B {
//     //         int a = 0xffff;
//     //     }
//     // }";
//     //         source_code1.as_bytes()
//     //     };
//     //     let tree = parser.parse(text, None).unwrap();
//     //     println!("{}", tree.root_node().to_sexp());

//     //     let full_node_src = java_tree_gen.generate_default(text, tree.walk());

//     //     println!("debug full node 1: {:?}", &full_node_src);

//     //     // dst
//     //     let text = {
//     //         let source_code1 = "class A {
//     //     class C {
//     //         int a = 0xffff;
//     //     }
//     // }";
//     //         source_code1.as_bytes()
//     //     };
//     //     let tree = parser.parse(text, None).unwrap();
//     //     println!("{}", tree.root_node().to_sexp());

//     //     let full_node_dst = java_tree_gen.generate_default(text, tree.walk());

//     //     println!("debug full node 2: {:?}", &full_node_dst);

//     //     let JavaTreeGen {
//     //         line_break:_,
//     //         stores: SimpleStores {
//     //             label_store,
//     //             type_store:_,
//     //             node_store,
//     //         }
//     //     } = java_tree_gen;

//     //     let mapping_store = DefaultMappingStore::default();
//     //     // let a = SimpleBottomUpMatcher::<
//     //     let a = ZsMatcher::<
//     //         CompletePostOrder<u32, u16>,
//     //         HashedCompressedNode<SyntaxNodeHashs<u32>, _, u32>,
//     //         u16,
//     //         NodeStore,
//     //         LabelStore,
//     //     >::matchh(
//     //         &node_store,
//     //         &label_store,
//     //         *full_node_src.id(),
//     //         *full_node_dst.id(),
//     //         mapping_store,
//     //     );
//     //     a.mappings
//     //         .src_to_dst
//     //         .iter()
//     //         .map(|x| if *x == 0 { None } else { Some(*x - 1) })
//     //         .zip(
//     //             a.mappings
//     //                 .dst_to_src
//     //                 .iter()
//     //                 .map(|x| if *x == 0 { None } else { Some(*x - 1) }),
//     //         )
//     //         .enumerate()
//     //         .for_each(|x| println!("{:?}", x));
//     //     // a.src_to_dst.iter().enumerate().for_each(|(i,m)| {
//     //     //     println!("{:?}", (i,m,&a.dst_to_src[*m as usize]));
//     //     // });
//     //     // println!("-----------");
//     //     // a.dst_to_src.iter().enumerate().for_each(|(i,m)| {
//     //     //     println!("{:?}", (i,m,&a.src_to_dst[*m as usize]));
//     //     // });

//     //     // // let mut out = String::new();
//     //     // let mut out = IoOut {
//     //     //     out: stdout()
//     //     // };
//     //     // serialize(
//     //     //     &java_tree_gen.node_store,
//     //     //     &java_tree_gen.label_store,
//     //     //     &full_node.id(),
//     //     //     &mut out,
//     //     //     &std::str::from_utf8(&java_tree_gen.line_break).unwrap(),
//     //     // );
//     //     // println!();
//     //     // print_tree_syntax(
//     //     //     &java_tree_gen.node_store,
//     //     //     &java_tree_gen.label_store,
//     //     //     &full_node.id(),
//     //     // );
//     //     // println!();
//     //     // stdout().flush().unwrap();
// }

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

// static CASE_1: &'static str = "class A{}";
// static CASE_2: &'static str = "class B{}";

// static CASE_1: &'static str = "class A{interface B{}}"; // 0.3.1.3.1 // 0.3.1
// static CASE_2: &'static str = "class A{}";

// static CASE_1: &'static str = "class A{} interface B{}";
// static CASE_2: &'static str = "interface B{} class A{}";

// static CASE_1: &'static str = "class A{enum C{}} interface B{}";
// static CASE_2: &'static str = "class A{} interface B{enum C{}}";
use hyper_gumtree::matchers::heuristic::gt::greedy_subtree_matcher::GreedySubtreeMatcher;

// struct JTG<'stores, 'cache>(JavaTreeGen<'stores, 'cache>);

// impl<'stores, 'cache> ActionApplier<HashedNode> for JTG<'stores, 'cache> {
//     type S = JavaTreeGen<'stores, 'cache>;

//     type R<'d> = HashedNodeRef<'d>;

//     fn store(&mut self) -> &mut JavaTreeGen<'stores, 'cache> {
//         &mut self.0
//     }
// }

fn main() {
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
            
            _,// HashedNodeRef,
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
        .generate().unwrap();

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
