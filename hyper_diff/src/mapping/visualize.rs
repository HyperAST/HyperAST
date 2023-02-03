//! pretty-print trees with mappings
//! eg.
//! |   3: 0; f     | 3 |   3: 0; f     |
//! |   2:   0; g   |   |   2:   0; g   |
//! |   0:     0; d | 0 |   0:     0; c |
//! |   1:     0; e | 1 |   1:     0; e |
//!
use std::fmt::Debug;

use hyper_ast::types::{self, LabelStore, NodeStore, Tree};
use num_traits::PrimInt;

use crate::{
    decompressed_tree_store::{
        complete_post_order::DisplayCompletePostOrder,
        pre_order_wrapper::{DisplaySimplePreOrderMapper, SimplePreOrderMapper},
        PostOrder,
    },
    matchers::mapping_store::MonoMappingStore,
};

pub fn print_mappings_no_ranges<
    'store: 'a,
    'a,
    IdD: 'a + PrimInt + Debug,
    M: MonoMappingStore<Src = IdD, Dst = IdD>,
    IdN: Clone + Eq + Debug,
    NS: NodeStore<IdN>,
    LS: LabelStore<str>,
    DD: PostOrder<'a, NS::R<'store>, IdD>,
    SD: PostOrder<'a, NS::R<'store>, IdD>,
>(
    dst_arena: &'a DD,
    src_arena: &'a SD,
    node_store: &'store NS,
    label_store: &'store LS,
    mappings: &M,
) where
    <NS as types::NodeStore<IdN>>::R<'store>: 'store + Tree<TreeId = IdN, Label = LS::I>,
    <<NS as types::NodeStore<IdN>>::R<'store> as types::Typed>::Type: Debug,
{
    let mut mapped = vec![false; dst_arena.len()];
    let src_arena = SimplePreOrderMapper::from(src_arena);
    let disp = DisplayCompletePostOrder::new(node_store, label_store, dst_arena);
    let dst_arena = format!("{:#?}", disp);
    let mappings = src_arena
        .map
        .iter()
        .map(|x| {
            if mappings.is_src(x) {
                let dst = mappings.get_dst_unchecked(x);
                if mapped[dst.to_usize().unwrap()] {
                    assert!(false, "GreedySubtreeMatcher {}", dst.to_usize().unwrap())
                }
                mapped[dst.to_usize().unwrap()] = true;
                Some(dst)
            } else {
                None
            }
        })
        .fold("".to_string(), |x, c| {
            if let Some(c) = c {
                let c = c.to_usize().unwrap();
                format!("{x}{c}\n")
            } else {
                format!("{x} \n")
            }
        });
    let src_arena = format!(
        "{:#?}",
        DisplaySimplePreOrderMapper {
            inner: &src_arena,
            node_store,
            label_store,
        }
    );
    let cols = vec![src_arena, mappings, dst_arena];
    let sizes: Vec<_> = cols
        .iter()
        .map(|x| x.lines().map(|x| x.len()).max().unwrap_or(0))
        .collect();
    let mut cols: Vec<_> = cols.iter().map(|x| x.lines()).collect();
    loop {
        let mut b = false;
        print!("|");
        for i in 0..cols.len() {
            if let Some(l) = cols[i].next() {
                print!(" {}{} |", l, " ".repeat(sizes[i] - l.len()));
                b = true;
            } else {
                print!(" {} |", " ".repeat(sizes[i]));
            }
        }
        println!();
        if !b {
            break;
        }
    }
}

pub fn print_mappings_no_ranges_label<
    'store: 'a,
    'a,
    IdD: 'a + PrimInt + Debug,
    M: MonoMappingStore<Src = IdD, Dst = IdD>,
    IdN: Clone + Eq + Debug,
    NS: NodeStore<IdN>,
    LS: LabelStore<str>,
    DD: PostOrder<'a, NS::R<'store>, IdD>,
    SD: PostOrder<'a, NS::R<'store>, IdD>,
>(
    dst_arena: &'a DD,
    src_arena: &'a SD,
    node_store: &'store NS,
    label_store: &'store LS,
    mappings: &M,
) where
    <NS as types::NodeStore<IdN>>::R<'store>: 'store + Tree<TreeId = IdN, Label = LS::I>,
    <<NS as types::NodeStore<IdN>>::R<'store> as types::Typed>::Type: Debug,
{
    let mut mapped = vec![false; dst_arena.len()];
    let src_arena = SimplePreOrderMapper::from(src_arena);
    let disp = DisplayCompletePostOrder::new(node_store, label_store, dst_arena);
    let dst_arena = format!("{:?}", disp);
    let mappings = src_arena
        .map
        .iter()
        .map(|x| {
            if mappings.is_src(x) {
                let dst = mappings.get_dst_unchecked(x);
                if mapped[dst.to_usize().unwrap()] {
                    assert!(false, "GreedySubtreeMatcher {}", dst.to_usize().unwrap())
                }
                mapped[dst.to_usize().unwrap()] = true;
                Some(dst)
            } else {
                None
            }
        })
        .fold("".to_string(), |x, c| {
            if let Some(c) = c {
                let c = c.to_usize().unwrap();
                format!("{x}{c}\n")
            } else {
                format!("{x} \n")
            }
        });
    let src_arena = format!(
        "{:?}",
        DisplaySimplePreOrderMapper {
            inner: &src_arena,
            node_store,
            label_store,
        }
    );
    let cols = vec![src_arena, mappings, dst_arena];
    let sizes: Vec<_> = cols
        .iter()
        .map(|x| x.lines().map(|x| x.len()).max().unwrap_or(0))
        .collect();
    let mut cols: Vec<_> = cols.iter().map(|x| x.lines()).collect();
    loop {
        let mut b = false;
        print!("|");
        for i in 0..cols.len() {
            if let Some(l) = cols[i].next() {
                print!(" {}{} |", l, " ".repeat(sizes[i] - l.len()));
                b = true;
            } else {
                print!(" {} |", " ".repeat(sizes[i]));
            }
        }
        println!();
        if !b {
            break;
        }
    }
}
