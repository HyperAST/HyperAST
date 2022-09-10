use std::{
    io::{stdout, Write},
    path::Path,
    time::Instant,
};

use hyper_ast::store::{
    labels::LabelStore,
    nodes::legion::{HashedNodeRef, NodeStore},
    SimpleStores, TypeStore,
};
use hyper_ast_gen_ts_java::legion_with_refs::{
    print_tree_ids, print_tree_syntax, JavaTreeGen, TreeJsonSerializer,
};
use hyper_gumtree::{
    actions::{script_generator2::ScriptGenerator, Actions},
    decompressed_tree_store::{
        bfs_wrapper,
        bfs_wrapper::SimpleBfsMapper,
        complete_post_order::DisplayCompletePostOrder,
        pre_order_wrapper::{DisplaySimplePreOrderMapper, SimplePreOrderMapper},
        ShallowDecompressedTreeStore,
    },
    matchers::{
        heuristic::gt::bottom_up_matcher::BottomUpMatcher,
        mapping_store::{MappingStore, MonoMappingStore},
    },
};

#[test]
fn test_simple_1() {
    let buggy = r#"class A{class C{}class B{{while(1){if(1){}else{}};}}}class D{class E{}class F{{while(2){if(2){}else{}};}}}"#;
    let fixed = r#"class A{class C{}}class B{{while(1){if(1){}else{}};}}class D{class E{}}class F{{while(2){if(2){}else{}};}}"#;
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    println!("{}", simple(&mut java_tree_gen, buggy, fixed))
}

#[test]
fn test_crash1() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let buggy_path = Path::new(
        "../../gt_datasets/defects4j/buggy/Cli/22/src_java_org_apache_commons_cli_PosixParser.java",
    );
    let fixed_path = Path::new(
        "../../gt_datasets/defects4j/fixed/Cli/22/src_java_org_apache_commons_cli_PosixParser.java",
    );
    let buggy = std::fs::read_to_string(buggy_path).expect("the buggy code");
    let fixed = std::fs::read_to_string(fixed_path).expect("the fixed code");
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    print!("{:?} len={}: ", buggy_path, buggy.len());
    let len = simple(&mut java_tree_gen, &buggy, &fixed);
    println!("{}", len);
}

#[test]
fn test_crash1_1() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let buggy = CASE1;
    let fixed = CASE2;
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    print!("len={}: ", buggy.len());
    let len = simple(&mut java_tree_gen, &buggy, &fixed);
    println!("{}", len);
}

static CASE1: &'static str = r#"class A {
    {
        if (1) {
        } else if (2) {
            h(42);
        } else if (3) {
            g(42);
        } else {
            h(42);
        }
    }
}"#;

static CASE2: &'static str = r#"class A {
    {
        } else {
            h(42, stopAtNonOption);
        }
    }
}"#;

#[test]
fn test_crash1_2() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let buggy = CASE3;
    let fixed = CASE4;
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    print!("len={}: ", buggy.len());
    let len = simple(&mut java_tree_gen, &buggy, &fixed);
    println!("{}", len);
}

static CASE3: &'static str = r#"class A {
    {
        if (1) {
        } else if (2) {
            g(t);
        } else if (3) {
            if (4) {
                p(t, s);
            } else {
                b(t, s);
            }
        } else if (s) {
            h(t);
        } else {
            g(t);
        }
    }
}"#;

static CASE4: &'static str = r#"class A {
    {
        if (1) {
        } else if (2) {
            g(t);
        } else if (3) {
            if (4) {
                p(t, s);
            } else {
                b(t, s);
            }
        } else {dst_c
            h(t, s);
        }
    }
}"#;

#[test]
fn test_all() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let root = Path::new("../../gt_datasets/defects4j");
    std::fs::read_dir(root).expect("should be a dir");
    let root_buggy = root.join("buggy");
    let root_fixed = root.join("fixed");
    for buggy_project in iter_dirs(&root_buggy) {
        for buggy_case in iter_dirs(&buggy_project.path()) {
            let buggy_path = std::fs::read_dir(buggy_case.path())
                .expect("should be a dir")
                .into_iter()
                .filter_map(|x| x.ok())
                .filter(|x| x.file_type().unwrap().is_file())
                .next()
                .unwrap()
                .path();
            let fixed_path = root_fixed.join(buggy_path.strip_prefix(&root_buggy).unwrap());
            let buggy = std::fs::read_to_string(&buggy_path).expect("the buggy code");
            let fixed = std::fs::read_to_string(fixed_path).expect("the fixed code");
            let mut stores = SimpleStores {
                label_store: LabelStore::new(),
                type_store: TypeStore {},
                node_store: NodeStore::new(),
            };
            let mut md_cache = Default::default();
            let mut java_tree_gen = JavaTreeGen {
                line_break: "\n".as_bytes().to_vec(),
                stores: &mut stores,
                md_cache: &mut md_cache,
            };
            let now = Instant::now();

            println!("{:?} len={}", buggy_path, buggy.len());
            let len = simple(&mut java_tree_gen, &buggy, &fixed);
            let processing_time = now.elapsed().as_nanos();
            println!("tt={} evos={}", processing_time, len);
        }
    }
}

fn iter_dirs(root_buggy: &std::path::Path) -> impl Iterator<Item = std::fs::DirEntry> {
    std::fs::read_dir(root_buggy)
        .expect(&format!("{:?} should be a dir", root_buggy))
        .into_iter()
        .filter_map(|x| x.ok())
        .filter(|x| x.file_type().unwrap().is_dir())
}

fn simple<'a>(java_tree_gen: &mut JavaTreeGen<'a, '_>, buggy: &'a str, fixed: &'a str) -> usize {
    let now = Instant::now();
    let tree = match JavaTreeGen::tree_sitter_parse(buggy.as_bytes()) {
        Ok(t) => t,
        Err(t) => t,
    };
    // println!("{}", tree.root_node().to_sexp());
    let processing_time = now.elapsed().as_nanos();
    println!();
    let full_node1 = java_tree_gen.generate_file(b"", buggy.as_bytes(), tree.walk());

    let processing_time = now.elapsed().as_nanos();
    
    let tree = match JavaTreeGen::tree_sitter_parse(fixed.as_bytes()) {
        Ok(t) => t,
        Err(t) => t,
    };
    // println!("{}", tree.root_node().to_sexp());
    let full_node2 = java_tree_gen.generate_file(b"", fixed.as_bytes(), tree.walk());
    // print_tree_syntax(
    //     &java_tree_gen.stores.node_store,
    //     &java_tree_gen.stores.label_store,
    //     &full_node1.local.compressed_node,
    // );
    // println!();
    // print_tree_syntax(
    //     &java_tree_gen.stores.node_store,
    //     &java_tree_gen.stores.label_store,
    //     &full_node2.local.compressed_node,
    // );
    // println!();

    // println!(
    //     "{}",
    //     TreeJsonSerializer::new(
    //         &java_tree_gen.stores.node_store,
    //         &java_tree_gen.stores.label_store,
    //         full_node1.local.compressed_node,
    //     )
    // );
    // println!(
    //     "{}",
    //     TreeJsonSerializer::new(
    //         &java_tree_gen.stores.node_store,
    //         &java_tree_gen.stores.label_store,
    //         full_node2.local.compressed_node,
    //     )
    // );
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

    // dbg!(java_tree_gen
    //     .stores
    //     .node_store
    //     .resolve(full_node1.local.compressed_node)
    //     .get_type());

    let src = full_node1.local.compressed_node;
    let dst = full_node2.local.compressed_node;

    let actions = {
        let mappings: VecStore<u16> = DefaultMappingStore::new();
        // {
        //     use hyper_gumtree::matchers::{
        //         decompressed_tree_store::SimpleZsTree,
        //         optimal::zs::ZsMatcher,
        //     };
        //     let mappings: VecStore<u16> = DefaultMappingStore::new();
        //     let mapper = ZsMatcher::<SimpleZsTree<_, _>, _, _, _, _>::matchh(
        //         &java_tree_gen.stores.node_store,
        //         &java_tree_gen.stores.label_store,
        //         src,
        //         dst,
        //         mappings,
        //     );
        //     let ZsMatcher {
        //         src_arena,
        //         dst_arena,
        //         mappings: ms,
        //         ..
        //     } = mapper;

        //     dbg!(ms);
        // }

        use hyper_gumtree::decompressed_tree_store::CompletePostOrder;
        use hyper_gumtree::matchers::{
            heuristic::gt::{
                greedy_bottom_up_matcher::GreedyBottomUpMatcher,
                greedy_subtree_matcher::{GreedySubtreeMatcher, SubtreeMatcher},
            },
            mapping_store::{DefaultMappingStore, VecStore},
        };

        let mapper = GreedySubtreeMatcher::<
            CompletePostOrder<_, u16>,
            CompletePostOrder<_, u16>,
            _,
            HashedNodeRef,
            _,
            // 2,
        >::matchh(&java_tree_gen.stores.node_store, &src, &dst, mappings);
        let SubtreeMatcher {
            src_arena,
            dst_arena,
            mappings,
            ..
        } = mapper.into();
        // {
        //     let mut mapped = vec![false; dst_arena.len()];
        //     let src_arena = SimplePreOrderMapper::from(&src_arena);
        //     let dst_arena = DisplayCompletePostOrder {
        //         inner: &dst_arena,
        //         node_store: &java_tree_gen.stores.node_store,
        //         label_store: &java_tree_gen.stores.label_store,
        //     }
        //     .to_string();
        //     let mappings = src_arena
        //         .map
        //         .iter()
        //         .map(|x| {
        //             if mappings.is_src(x) {
        //                 let dst = mappings.get_dst(x);
        //                 if mapped[dst as usize] {
        //                     assert!(false, "GreedySubtreeMatcher {}", dst)
        //                 }
        //                 mapped[dst as usize] = true;
        //                 Some(dst)
        //             } else {
        //                 None
        //             }
        //         })
        //         .fold("".to_string(), |x, c| {
        //             if let Some(c) = c {
        //                 format!("{x}{c}\n")
        //             } else {
        //                 format!("{x} \n")
        //             }
        //         });
        //     // let mappings = format!(
        //     //     "\n{}",
        //     //     mappings.display(&|src: u16| src.to_string(), &|dst: u16| dst.to_string(),)
        //     // );

        //     let src_arena = DisplaySimplePreOrderMapper {
        //         inner: &src_arena,
        //         node_store: &java_tree_gen.stores.node_store,
        //     }
        //     .to_string();
        //     let cols = vec![src_arena, mappings, dst_arena];
        //     let sizes: Vec<_> = cols
        //         .iter()
        //         .map(|x| x.lines().map(|x| x.len()).max().unwrap_or(0))
        //         .collect();
        //     let mut cols: Vec<_> = cols.iter().map(|x| x.lines()).collect();
        //     loop {
        //         let mut b = false;
        //         print!("|");
        //         for i in 0..cols.len() {
        //             if let Some(l) = cols[i].next() {
        //                 print!(" {}{} |", l, " ".repeat(sizes[i] - l.len()));
        //                 b = true;
        //             } else {
        //                 print!(" {} |", " ".repeat(sizes[i]));
        //             }
        //         }
        //         println!();
        //         if !b {
        //             break;
        //         }
        //     }
        // }
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
            HashedNodeRef,
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

        // {
        //     let mut mapped = vec![false; dst_arena.len()];
        //     let src_arena = SimplePreOrderMapper::from(&src_arena);
        //     let dst_arena = DisplayCompletePostOrder {
        //         inner: &dst_arena,
        //         node_store: &java_tree_gen.stores.node_store,
        //         label_store: &java_tree_gen.stores.label_store,
        //     }
        //     .to_string();
        //     let mappings = src_arena
        //         .map
        //         .iter()
        //         .map(|x| {
        //             if mappings.is_src(x) {
        //                 let dst = mappings.get_dst(x);
        //                 if mapped[dst as usize] {
        //                     assert!(false, "GreedyBottomUpMatcher {}", dst)
        //                 }
        //                 mapped[dst as usize] = true;
        //                 Some(dst)
        //             } else {
        //                 None
        //             }
        //         })
        //         .fold("".to_string(), |x, c| {
        //             if let Some(c) = c {
        //                 format!("{x}{c}\n")
        //             } else {
        //                 format!("{x} \n")
        //             }
        //         });
        //     // let mappings = format!(
        //     //     "\n{}",
        //     //     mappings.display(&|src: u16| src.to_string(), &|dst: u16| dst.to_string(),)
        //     // );

        //     let src_arena = DisplaySimplePreOrderMapper {
        //         inner: &src_arena,
        //         node_store: &java_tree_gen.stores.node_store,
        //     }
        //     .to_string();
        //     let cols = vec![src_arena, mappings, dst_arena];
        //     let sizes: Vec<_> = cols
        //         .iter()
        //         .map(|x| x.lines().map(|x| x.len()).max().unwrap_or(0))
        //         .collect();
        //     let mut cols: Vec<_> = cols.iter().map(|x| x.lines()).collect();
        //     loop {
        //         let mut b = false;
        //         print!("|");
        //         for i in 0..cols.len() {
        //             if let Some(l) = cols[i].next() {
        //                 print!(" {}{} |", l, " ".repeat(sizes[i] - l.len()));
        //                 b = true;
        //             } else {
        //                 print!(" {} |", " ".repeat(sizes[i]));
        //             }
        //         }
        //         println!();
        //         if !b {
        //             break;
        //         }
        //     }
        // }
        let dst_arena = SimpleBfsMapper::from(&java_tree_gen.stores.node_store, &dst_arena);
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
        >::precompute_actions(
            &java_tree_gen.stores.node_store,
            &src_arena,
            &dst_arena,
            &mappings,
        )
        .generate();

        let ScriptGenerator {
            store: _, actions, ..
        } = script_gen;
        actions
        // ActionsVec(vec![])
    };
    return actions.len();
}
