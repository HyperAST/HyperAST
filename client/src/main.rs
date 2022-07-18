#![feature(generic_associated_types)]

use std::{fmt::Debug, sync::Mutex};

use hyper_gumtree::{
    actions::{
        action_vec::{apply_action, ActionsVec},
        bfs_wrapper,
        script_generator2::{Act, ScriptGenerator, SimpleAction},
        Actions,
    },
    matchers::{
        decompressed_tree_store::{
            BreathFirst, BreathFirstIterable, CompletePostOrder, Initializable, PostOrderIterable,
            ShallowDecompressedTreeStore, SimpleZsTree,
        },
        heuristic::gt::{
            bottom_up_matcher::BottomUpMatcher, greedy_bottom_up_matcher::GreedyBottomUpMatcher,
            greedy_subtree_matcher::SubtreeMatcher,
        },
        mapping_store::{DefaultMappingStore, VecStore},
        optimal::zs::ZsMatcher,
    },
    tree::tree_path::TreePath,
};

fn main_compress() {
    // use rusted_gumtree_gen_ts_java::java_tree_gen_full_compress::{
    //     JavaTreeGen, LabelStore, NodeStore, SimpleStores,
    // };
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
    //         line_break:_,
    //         stores: SimpleStores {
    //             label_store,
    //             type_store:_,
    //             node_store,
    //         }
    //     } = java_tree_gen;

    //     let mapping_store = DefaultMappingStore::new();
    //     // let a = SimpleBottomUpMatcher::<
    //     let a = ZsMatcher::<
    //         CompletePostOrder<u32, u16>,
    //         HashedCompressedNode<SyntaxNodeHashs<u32>, _, u32>,
    //         u16,
    //         NodeStore,
    //         LabelStore,
    //     >::matchh(
    //         &node_store,
    //         &label_store,
    //         *full_node_src.id(),
    //         *full_node_dst.id(),
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

use hyper_ast::{
    filter::BloomResult,
    hashed::HashedNode,
    nodes::RefContainer,
    position::{
        ExploreStructuralPositions, Scout, StructuralPosition, StructuralPositionStore,
        TreePath as _,
    },
    store::{
        defaults::LabelIdentifier,
        labels::LabelStore,
        nodes::{
            legion::{HashedNodeRef, NodeIdentifier},
            DefaultNodeStore as NodeStore,
        },
        SimpleStores, TypeStore,
    },
    tree_gen::ZippedTreeGen,
    types::{Labeled, NodeStoreExt, Stored, Tree, Typed, WithChildren},
    utils::memusage_linux,
};
use rusted_gumtree_gen_ts_java::legion_with_refs::{
    print_tree_ids, print_tree_syntax, JavaTreeGen,
};

// static CASE_1: &'static str = "class A{}";
// static CASE_2: &'static str = "class B{}";

static CASE_1: &'static str = "class A{interface B{}}"; // 0.3.1.3.1 // 0.3.1
static CASE_2: &'static str = "class A{}";

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
    let src = full_node1.local.compressed_node;
    let dst = full_node2.local.compressed_node;

    let actions = {
        let mappings: VecStore<u16> = DefaultMappingStore::new();
        // GreedySubtreeMatcher.MIN_HEIGHT = 0;
        // GreedyBottomUpMatcher
        {
            let mappings: VecStore<u16> = DefaultMappingStore::new();
            let mapper = ZsMatcher::<SimpleZsTree<_, _>, _, _, _, _>::matchh(
                &java_tree_gen.stores.node_store,
                &java_tree_gen.stores.label_store,
                src,
                dst,
                mappings,
            );
            let ZsMatcher {
                src_arena,
                dst_arena,
                mappings: ms,
                ..
            } = mapper;

            dbg!(ms);
        }
        let mapper = GreedyBottomUpMatcher::<
            CompletePostOrder<_, u16>,
            CompletePostOrder<_, u16>,
            _,
            HashedNodeRef,
            _,
            _,
            1000,
            1,
            1,
        >::matchh(
            &java_tree_gen.stores.node_store,
            &java_tree_gen.stores.label_store,
            &src,
            &dst,
            mappings,
        );
        let BottomUpMatcher {
            src_arena,
            dst_arena,
            mappings: ms,
            ..
        } = mapper.into();
        println!("ms={:?}", ms);
        println!("{:?} {:?}", dst_arena.root(), dst);
        println!("{:?}", dst_arena);
        println!(
            "{:?}",
            dst_arena
                .iter_df_post()
                .map(|id: u16| dst_arena.original(&id))
                .collect::<Vec<_>>()
        );
        let dst_arena = bfs_wrapper::SD::from(&java_tree_gen.stores.node_store, &dst_arena);
        println!("{:?} {:?}", dst_arena.root(), dst);
        println!("{:?}", dst_arena);
        println!(
            "{:?}",
            dst_arena
                .iter_bf()
                .map(|id| dst_arena.original(&id))
                .collect::<Vec<_>>()
        );
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
            &ms,
        )
        .generate();

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
    {
        // let mut java_tree_gen = JavaTreeGen {
        //     line_break: "\n".as_bytes().to_vec(),
        //     stores: &mut stores,
        //     md_cache: &mut md_cache,
        // };

        // println!("{:?}", actions.len());
        let mut root = vec![src];
        for x in actions.iter() {
            use hyper_ast::types::LabelStore;
            let SimpleAction { path, action } = x;
            match action {
                Act::Delete {} => {
                    println!("del {:?}", path);
                }
                Act::Update { new } => println!(
                    "upd {:?} {:?}",
                    java_tree_gen.stores.label_store.resolve(new),
                    path
                ),
                Act::Move { from } => println!("mov {:?} {:?}", from, path),
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
            // java_tree_gen2.apply_action(x, &mut root);
            apply_action::<HashedNode, JavaTreeGen<'_, '_>>(x, &mut root, &mut java_tree_gen);
            // java_tree_gen2.build_then_insert(todo!(), todo!(), todo!());
        }
        let then = root; //ActionsVec::apply_actions(actions.iter(), *src, &mut node_store);

        print_tree_syntax(
            &java_tree_gen.stores.node_store,
            &java_tree_gen.stores.label_store,
            &dst,
        );
        println!();
        print_tree_syntax(
            &java_tree_gen.stores.node_store,
            &java_tree_gen.stores.label_store,
            then.last().unwrap(),
        );
        println!();
        assert_eq!(*then.last().unwrap(), dst);
    }

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

    //     use rusted_gumtree_gen_ts_java::java_tree_gen_no_compress_arena::{JavaTreeGen, LabelStore, NodeStore,SimpleStores,HashedNode};
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

    //     let mapping_store = DefaultMappingStore::new();
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

static CASE_BIG1: &'static str = r#"/***/

package fr.inria.controlflow;

import org.junit.Test;
import spoon.processing.AbstractProcessor;
import spoon.processing.ProcessingManager;
import spoon.reflect.code.CtIf;
import spoon.reflect.factory.Factory;

import static junit.framework.TestCase.assertFalse;
import static org.junit.Assert.assertTrue;

class A{class B{}}"#;

static CASE_BIG2: &'static str = r#"/***/

package fr.inria.controlflow;

import org.junit.Test;
import spoon.processing.AbstractProcessor;
import spoon.processing.ProcessingManager;
import spoon.reflect.code.CtIf;
import bbbbbbbbbbbbbbbbbb.CtMethod;
import spoon.reflect.factory.Factory;
import aaaaaaaaaaaaaaaaaa.QueueProcessingManager;

import static junit.framework.TestCase.assertFalse;
import static org.junit.Assert.assertTrue;

class A{}class B{}"#;
