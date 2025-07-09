use hyperast::{
    store::SimpleStores,
    test_utils::simple_tree::{DisplayTree, LS, SimpleTree, TStore},
    types::DecompressedFrom,
};

use crate::{
    decompressed_tree_store::{
        CompletePostOrder, ShallowDecompressedTreeStore, lazy_post_order::LazyPostOrder,
    },
    matchers::{
        Decompressible, Mapper, Mapping,
        heuristic::gt::{
            greedy_bottom_up_matcher::GreedyBottomUpMatcher,
            lazy_marriage_bottom_up_matcher::LazyMarriageBottomUpMatcher,
            lazy2_greedy_bottom_up_matcher::LazyGreedyBottomUpMatcher,
            marriage_bottom_up_matcher::MarriageBottomUpMatcher,
        },
        mapping_store::{DefaultMappingStore, MappingStore, VecStore},
    },
    tests::examples::*,
    tree::simple_tree::{NS, Tree, vpair_to_stores},
};

#[derive(Clone, Copy)]
enum GumtreeVariant {
    Greedy,
    Stable,
    GreedyLazy,
    StableLazy,
}

impl GumtreeVariant {
    fn is_lazy(&self) -> bool {
        return match self {
            Self::Greedy | Self::Stable => false,
            Self::GreedyLazy | Self::StableLazy => true,
        };
    }
}

#[test]
fn test_gumtree_stable() {
    run_stable_examples_with_variant(GumtreeVariant::Stable);
}

#[test]
fn test_gumtree_lazy_stable() {
    run_stable_examples_with_variant(GumtreeVariant::StableLazy);
}

#[test]
fn test_gumtree_greedy_unstable() {
    run_unstable_examples_with_variant(GumtreeVariant::Greedy)
}

#[test]
fn test_gumtree_greedy_lazy_unstable() {
    run_unstable_examples_with_variant(GumtreeVariant::GreedyLazy)
}

fn run_unstable_examples_with_variant(variant: GumtreeVariant) {
    assert!(!is_stable(example_unstable1(), variant));
    assert!(!is_stable(example_unstable2(), variant));
}

fn run_stable_examples_with_variant(variant: GumtreeVariant) {
    //assert!(is_stable(example_stable_test1(), variant));
    assert!(is_stable(example_stable_test2(), variant));
    assert!(is_stable(example_stable1(), variant));
    assert!(is_stable(example_stable2(), variant));
    assert!(is_stable(example_stable3(), variant));
    assert!(is_stable(example_unstable1(), variant));
    assert!(is_stable(example_unstable2(), variant));
}

fn is_stable(
    example: ((SimpleTree<u8>, SimpleTree<u8>), VecStore<u16>),
    variant: GumtreeVariant,
) -> bool {
    let (vpair, mappings) = example;
    let (stores, src, dst) = vpair_to_stores(vpair);
    let result1 = test_with_mappings(&stores, src, dst, mappings.clone(), variant);
    let result2 = test_with_mappings(&stores, dst, src, mappings.mirror(), variant).mirror();

    println!("result1: {:?}\nresult2: {:?}", result1, result2);
    return result1 == result2;
}

fn _calculate_mappings(example: ((SimpleTree<u8>, SimpleTree<u8>), Vec<Vec<u8>>, Vec<Vec<u8>>)) {
    let (vpair, map_src, map_dst) = example;
    let (stores, src, dst) = vpair_to_stores(vpair);

    let src_arena = Decompressible::<_, CompletePostOrder<u16, u16>>::decompress(&stores, &src);
    let dst_arena = Decompressible::<_, CompletePostOrder<u16, u16>>::decompress(&stores, &dst);

    let mut m: DefaultMappingStore<u16> = DefaultMappingStore::default();
    m.topit(src_arena.len(), dst_arena.len());

    let src = src_arena.root();
    let dst = dst_arena.root();

    for (map_src, map_dst) in map_src.iter().zip(map_dst) {
        m.link(
            src_arena.child(&src, &map_src),
            dst_arena.child(&dst, &map_dst),
        );
    }
    println!("{:?}", m);
}

fn test_with_mappings(
    stores: &SimpleStores<TStore, NS<Tree>, LS<u16>>,
    src: u16,
    dst: u16,
    mappings: VecStore<u16>,
    variant: GumtreeVariant,
) -> VecStore<u16> {
    print_tree(stores, src, "src tree");
    print_tree(stores, dst, "dst tree");

    if !variant.is_lazy() {
        let src_arena = Decompressible::<_, CompletePostOrder<u16, u16>>::decompress(stores, &src);
        let dst_arena = Decompressible::<_, CompletePostOrder<u16, u16>>::decompress(stores, &dst);

        let mapping = Mapper {
            hyperast: stores,
            mapping: Mapping {
                src_arena,
                dst_arena,
                mappings,
            },
        };
        let mapping = match variant {
            GumtreeVariant::Greedy => {
                GreedyBottomUpMatcher::<
                    Decompressible<_, CompletePostOrder<_, u16>>,
                    Decompressible<_, CompletePostOrder<_, u16>>,
                    &SimpleStores<TStore, NS<Tree>, LS<u16>>,
                    VecStore<u16>,
                >::match_it(mapping)
                .mapping
            }
            GumtreeVariant::Stable => {
                MarriageBottomUpMatcher::<
                    Decompressible<_, CompletePostOrder<_, u16>>,
                    Decompressible<_, CompletePostOrder<_, u16>>,
                    &SimpleStores<TStore, NS<Tree>, LS<u16>>,
                    VecStore<u16>,
                >::match_it(mapping)
                .mapping
            }
            _ => panic!(),
        };
        println!(
            "{}",
            mapping.mappings.display(
                &|src: u16| mapping.src_arena.original(&src).to_string(),
                &|dst: u16| mapping.dst_arena.original(&dst).to_string(),
            )
        );
        return mapping.mappings;
    } else {
        let mut owned_src_arena =
            Decompressible::<_, LazyPostOrder<u16, u16>>::decompress(stores, &src);
        let mut owned_dst_arena =
            Decompressible::<_, LazyPostOrder<u16, u16>>::decompress(stores, &dst);
        let mut src_arena = owned_src_arena.as_mut();
        let mut dst_arena = owned_dst_arena.as_mut();

        // Attempted to decompress all mapped nodes and their siblings, but this did not fix the panic
        // for src in &mappings.dst_to_src {
        // if *src != 0 {
        // let n = src_arena.decompress_to(src.shallow());
        // if let Some(p) = src_arena.parent(&n) {
        // src_arena.decompress_children(&p);
        // }
        // }
        // }
        // for dst in &mappings.src_to_dst {
        // if *dst != 0 {
        // let n = dst_arena.decompress_to(dst.shallow());
        // if let Some(p) = dst_arena.parent(&n) {
        // dst_arena.decompress_children(&p);
        // }
        // }
        // }

        // Instead, we decompress the whole tree. For testing purposes, this should not matter (only for performance)
        src_arena.complete_subtree(&src_arena.root());
        dst_arena.complete_subtree(&dst_arena.root());

        let mut m: DefaultMappingStore<u16> = DefaultMappingStore::default();
        m.topit(src_arena.len(), dst_arena.len());

        let mapper = Mapper {
            hyperast: stores,
            mapping: Mapping {
                src_arena,
                dst_arena,
                mappings,
            },
        };

        let mapping = match variant {
            GumtreeVariant::GreedyLazy => {
                LazyGreedyBottomUpMatcher::<_, _, _, _, VecStore<_>>::match_it(mapper).mapping
            }
            GumtreeVariant::StableLazy => {
                LazyMarriageBottomUpMatcher::<_, _, _, _, VecStore<_>>::match_it(mapper).mapping
            }
            _ => panic!(),
        };
        // println!(
        // "{}",
        // mapping.mappings.display(
        // &|src: u16| mapping.src_arena.original(&src).to_string(),
        // &|dst: u16| mapping.dst_arena.original(&dst).to_string(),
        // )
        // );
        return mapping.mappings;
    }
}

fn print_tree(stores: &SimpleStores<TStore, NS<Tree>, LS<u16>>, src: u16, caption: &str) {
    let label_store = &stores.label_store;
    let node_store = &stores.node_store;
    println!(
        "{}:\n{:?}",
        caption,
        DisplayTree::new(label_store, node_store, src)
    );
}
