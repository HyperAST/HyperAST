use std::fmt::Debug;
use std::marker::PhantomData;
use std::time::Instant;
use hyperast::{types, PrimInt};
use hyperast::full::FullNode;
use hyperast::store::SimpleStores;
use hyperast::test_utils::simple_tree::{vpair_to_stores, TStore, TreeRef};
use hyperast::test_utils::simple_tree::Tree;
use hyperast::tree_gen::StatsGlobalData;
use hyperast::types::{DecompressedFrom, HyperAST, HyperASTShared, NodeId};
use crate::actions::script_generator2::SimpleAction;
use crate::algorithms::{DiffResult, PreparedMappingDurations};
use crate::decompressed_tree_store::{CompletePostOrder, FullyDecompressedTreeStore, PostOrder};
use crate::decompressed_tree_store::complete_post_order::DisplayCompletePostOrder;
use crate::decompressed_tree_store::lazy_post_order::LazyPostOrder;
use crate::decompressed_tree_store::pre_order_wrapper::{DisplaySimplePreOrderMapper, SimplePreOrderMapper};
use crate::matchers::heuristic::gt::bottom_up_matcher::BottomUpMatcher;
use crate::matchers::heuristic::gt::greedy_bottom_up_matcher::GreedyBottomUpMatcher;
use crate::matchers::heuristic::gt::greedy_subtree_matcher::{GreedySubtreeMatcher, SubtreeMatcher};
use crate::matchers::heuristic::gt::simple_bottom_up_matcher3::SimpleBottomUpMatcher3;
use crate::matchers::{mapping_store, Decompressible, Mapper};
use crate::matchers::mapping_store::{DefaultMappingStore, DefaultMultiMappingStore, MappingStore, MonoMappingStore, VecStore};
use crate::tests::examples;
use crate::tree::tree_path::CompressedTreePath;

// From benchmark_diffs crate (todo: move test)
// pub fn print_mappings_no_ranges<
//     'a,
//     IdD: 'a + PrimInt + Debug,
//     M: MonoMappingStore<Src= IdD, Dst= IdD>,
//     HAST: HyperAST + Copy,
//     // IdN: Clone + Eq + Debug,
//     DD: PostOrder<HAST, IdD> ,
//     SD: PostOrder<HAST, IdD>,
// >(
//     dst_arena: &'a DD,
//     src_arena: &'a SD,
//     stores: HAST,
//     mappings: &M,
// )
// where
// // <NS as types::NodeStore<IdN>>::R<'store>: 'store + Tree<TreeId = IdN, Label = LS::I>,
// // <<NS as types::NodeStore<IdN>>::R<'store> as types::Typed>::Type: Debug,
// {
//     let mut mapped = vec![false; dst_arena.len()];
//     let src_arena = SimplePreOrderMapper::from(src_arena);
//     let disp = DisplayCompletePostOrder::<IdD, _, _>::new(stores, dst_arena);
//     let dst_arena = format!("{:?}", disp);
//     let mappings = src_arena
//         .map
//         .iter()
//         .map(|x| {
//             if let Some(dst) = mappings.get_dst(x) {
//                 if mapped[dst.to_usize().unwrap()] {
//                     assert!(false, "GreedySubtreeMatcher {}", dst.to_usize().unwrap())
//                 }
//                 mapped[dst.to_usize().unwrap()] = true;
//                 Some(dst)
//             } else {
//                 None
//             }
//         })
//         .fold("".to_string(), |x, c| {
//             if let Some(c) = c {
//                 let c = c.to_usize().unwrap();
//                 format!("{x}{c}\n")
//             } else {
//                 format!("{x} \n")
//             }
//         });
//     let src_arena = format!(
//         "{:?}",
//         DisplaySimplePreOrderMapper {
//             inner: &src_arena,
//             stores: &stores
//         }
//     );
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
//
// #[test]
// fn gumtree_simple_bottom_up_matcher3_test1() {
//     let (stores, src, dst) = vpair_to_stores(examples::example_action2());
//
//     type DS<HAST: HyperASTShared> = Decompressible<HAST, LazyPostOrder<HAST::IdN, u32>>;
//     let src_arena = DS::decompress(&stores, &src);
//     let dst_arena = DS::decompress(&stores, &dst);
//
//     let mut mappings = DefaultMappingStore::default();
//
//     mappings.topit(src_arena.len(), dst_arena.len());
//     print_mappings_no_ranges(&dst_arena, &src_arena, &stores, &mappings);
//     println!();
//
//     let stores = stores.change_type_store::<TreeRef<Tree>>();
//
//     let mappings = DefaultMappingStore::default();
//     let mapper = GreedySubtreeMatcher::<
//         CompletePostOrder<_, u16>,
//         CompletePostOrder<_, u16>,
//         _,
//         _,
//     >::matchh::<DefaultMultiMappingStore<_>>(
//         &stores, src, dst, mappings
//     );
//     let SubtreeMatcher { mappings, .. } = mapper.into();
//     let mapper = SimpleBottomUpMatcher3::<
//         CompletePostOrder<_, u16>,
//         CompletePostOrder<_, u16>,
//         _,
//         _,
//     >::matchh(&stores, &src, &dst, mappings);
//     let BottomUpMatcher {
//         src_arena,
//         dst_arena,
//         mappings,
//         ..
//     } = mapper.into();
//     print_mappings_no_ranges(&dst_arena, &src_arena, &stores, &mappings);
//     println!();
//
//     // auto(src_arena, dst_arena, mappings);
// }

// #[test]
// fn test_simple_bottom_up_matcher3_1() {
//     let (label_store, node_store, src, dst) = vpair_to_stores(example_bottom_up());
//     let mut ms: DefaultMappingStore<u16> = DefaultMappingStore::default();
//     let src = &src;
//     let dst = &dst;
//
//     let src_arena = CompletePostOrder::<_, u16>::make(&node_store, src);
//     let dst_arena = CompletePostOrder::<_, u16>::make(&node_store, dst);
//     let src = &(src_arena.root());
//     let dst = &(dst_arena.root());
//     let from_src = |path: &[u8]| src_arena.child(&node_store, src, path);
//     let from_dst = |path: &[u8]| dst_arena.child(&node_store, dst, path);
//     println!("rootsrc: {:?}", src);
//     println!("rootdst: {:?}", dst);
//
//     ms.topit(src_arena.len() + 1, dst_arena.len() + 1);
//     ms.link(from_src(&[0, 2, 0]), from_dst(&[0, 2, 0]));
//     ms.link(from_src(&[0, 2, 1]), from_dst(&[0, 2, 1]));
//     ms.link(from_src(&[0, 2, 2]), from_dst(&[0, 2, 2]));
//     ms.link(from_src(&[0, 2, 3]), from_dst(&[0, 2, 3]));
//     for (f, s) in ms.iter() {
//         assert!(ms.has(&f, &s), "{} -x-> {}", f, s);
//     }
//     let ms1 = ms.clone();
//     for (f, s) in ms.iter() {
//         assert!(ms1.has(&f, &s), "{} -x-> {}", f, s);
//     }
//
//     let mut mapper = SimpleBottomUpMatcher3::<_, _, _, VecStore<_>>::new(&src_arena, &dst_arena);
//     GreedyBottomUpMatcher::execute(&mut mapper);
//
//     let BottomUpMatcher::<_, _, _, Tree, _, _> {
//         src_arena,
//         dst_arena,
//         mappings: ms1,
//         ..
//     } = mapper.into();
//     let src = src_arena.root();
//     let dst = dst_arena.root();
//
//     // // assertEquals(5, ms1.size());
//     assert_eq!(5, ms1.src_to_dst.iter().filter(|x| **x != 0).count());
//     assert_eq!(5, ms1.len());
//     for (f, s) in ms.iter() {
//         assert!(ms1.has(&f, &s), "{} -x-> {}", f, s);
//     }
//     assert!(ms1.has(&src, &dst));
//
//     let ms2 = ms.clone();
//     let mut mapper = SimpleBottomUpMatcher3::<_, _, _, _, NS<Tree>, _, _, 0, 1, 2>::new(
//         &node_store,
//         &label_store,
//         src_arena,
//         dst_arena,
//         ms2,
//     );
//     SimpleBottomUpMatcher3::execute(&mut mapper);
//     let BottomUpMatcher::<_, _, _, Tree, _, _> {
//         src_arena,
//         dst_arena,
//         mappings: ms2,
//         ..
//     } = mapper.into();
//     let src = &src_arena.root();
//     let dst = &dst_arena.root();
//     let from_src = |path: &[u8]| src_arena.child(&node_store, src, path);
//     let from_dst = |path: &[u8]| dst_arena.child(&node_store, dst, path);
//     assert!(ms2.has(src, dst));
//     for (f, s) in ms.iter() {
//         assert!(ms2.has(&f, &s));
//     }
//     assert!(ms2.has(&from_src(&[0]), &from_dst(&[0])));
//     assert!(ms2.has(&from_src(&[0, 2]), &from_dst(&[0, 2])));
//     assert_eq!(7, ms2.len());
//
//     let ms3 = ms.clone();
//     let mut mapper = GreedyBottomUpMatcher::<_, _, _, _, NS<Tree>, _, _, 10, 1, 2>::new(
//         &node_store,
//         &label_store,
//         src_arena,
//         dst_arena,
//         ms3,
//     );
//     GreedyBottomUpMatcher::execute(&mut mapper);
//     let BottomUpMatcher::<_, _, _, Tree, _, _> {
//         src_arena,
//         dst_arena,
//         mappings: ms3,
//         ..
//     } = mapper.into();
//     let src = &src_arena.root();
//     let dst = &dst_arena.root();
//     let from_src = |path: &[u8]| src_arena.child(&node_store, src, path);
//     let from_dst = |path: &[u8]| dst_arena.child(&node_store, dst, path);
//     assert_eq!(9, ms3.len());
//     for (f, s) in ms.iter() {
//         assert!(ms3.has(&f, &s));
//     }
//     assert!(ms3.has(src, dst));
//     assert!(ms3.has(&from_src(&[0]), &from_dst(&[0])));
//     assert!(ms3.has(&from_src(&[0, 0]), &from_dst(&[0, 0])));
//     assert!(ms3.has(&from_src(&[0, 1]), &from_dst(&[0, 1])));
//     assert!(ms3.has(&from_src(&[0, 2]), &from_dst(&[0, 2])));
// }
