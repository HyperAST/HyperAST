use crate::{
    decompressed_tree_store::{
        CompletePostOrder, DecompressedTreeStore, ShallowDecompressedTreeStore,
    },
    matchers::{
        Decompressible, Mapper,
        heuristic::gt::greedy_bottom_up_matcher::GreedyBottomUpMatcher,
        mapping_store::{MappingStore, VecStore},
    },
    tests::examples::example_unstable,
};

use hyperast::types::{LabelStore, Labeled};
use hyperast::{test_utils::simple_tree::vpair_to_stores, types::DecompressedFrom as _};

#[test]
fn test_unstable_greedy() {
    let (stores, src, dst) = vpair_to_stores(example_unstable());
    let hyperast = &stores;
    let mappings = VecStore::<u32>::default();

    let src_arena = Decompressible::<_, CompletePostOrder<_, u32>>::decompress(hyperast, &src);
    let dst_arena = Decompressible::<_, CompletePostOrder<_, u32>>::decompress(hyperast, &dst);
    let mut mapper = Mapper {
        hyperast,
        mapping: crate::matchers::Mapping {
            src_arena,
            dst_arena,
            mappings,
        },
    };
    mapper.mapping.mappings.topit(
        mapper.mapping.src_arena.len(),
        mapper.mapping.dst_arena.len(),
    );
    let src_arena = &mapper.src_arena;
    let dst_arena = &mapper.dst_arena;
    let src = &src_arena.root();
    let dst = &dst_arena.root();
    dbg!(src_arena.child(src, &[0]));
    let src_a = child_at_path(hyperast, src_arena, src, &[0, 0]);
    assert_eq!(src_a, child_by_label(hyperast, src_arena, src, "a"));
    let src_x = child_at_path(hyperast, src_arena, src, &[0]);
    assert_eq!(src_x, child_by_label(hyperast, src_arena, src, "x"));
    let src_b = child_at_path(hyperast, src_arena, src, &[1, 0]);
    assert_eq!(src_b, child_by_label(hyperast, src_arena, src, "b"));
    let src_c = child_at_path(hyperast, src_arena, src, &[1, 1]);
    assert_eq!(src_c, child_by_label(hyperast, src_arena, src, "c"));
    let src_y = child_at_path(hyperast, src_arena, src, &[1]);
    assert_eq!(src_y, child_by_label(hyperast, src_arena, src, "y"));
    let dst_x = child_at_path(hyperast, dst_arena, dst, &[0]);
    assert_eq!(dst_x, child_by_label(hyperast, dst_arena, dst, "x"));
    let dst_a = child_at_path(hyperast, dst_arena, dst, &[1, 0]);
    assert_eq!(dst_a, child_by_label(hyperast, dst_arena, dst, "a"));
    let dst_b = child_at_path(hyperast, dst_arena, dst, &[1, 1]);
    assert_eq!(dst_b, child_by_label(hyperast, dst_arena, dst, "b"));
    let dst_c = child_at_path(hyperast, dst_arena, dst, &[1, 2]);
    assert_eq!(dst_c, child_by_label(hyperast, dst_arena, dst, "c"));
    let dst_y = child_at_path(hyperast, dst_arena, dst, &[1]);
    assert_eq!(dst_y, child_by_label(hyperast, dst_arena, dst, "y"));

    let mappings = &mut mapper.mapping.mappings;
    mappings.link(src_a, dst_a);
    mappings.link(src_b, dst_b);
    mappings.link(src_c, dst_c);

    let mirrored = mapper.mapping.mappings.clone().mirror();

    GreedyBottomUpMatcher::<_, _, _, _, 1>::execute(&mut mapper);

    let src = &mapper.src_arena.root();
    let dst = &mapper.dst_arena.root();
    dbg!(&mapper.mappings, src, dst);
    assert!(mapper.mappings.has(&src_x, &dst_y));
    assert!(!mapper.mappings.has(&src_y, &dst_x));

    let src_dst_mappings = mapper.mapping.mappings;

    let src_arena = mapper.mapping.dst_arena;
    let dst_arena = mapper.mapping.src_arena;
    let mut mapper = Mapper {
        hyperast,
        mapping: crate::matchers::Mapping {
            src_arena,
            dst_arena,
            mappings: mirrored,
        },
    };
    GreedyBottomUpMatcher::<_, _, _, _, 1>::execute(&mut mapper);

    let src = &mapper.src_arena.root();
    let dst = &mapper.dst_arena.root();
    dbg!(&mapper.mappings, src, dst);
    // CAUTION: must also be mirrored
    assert!(mapper.mappings.has(&dst_y, &src_y));
    assert!(!mapper.mappings.has(&dst_x, &src_x));

    // NOTE invariant for unstability, the stability can be verified by eq
    assert_ne!(src_dst_mappings.dst_to_src, mapper.mappings.src_to_dst);
    assert_ne!(src_dst_mappings.src_to_dst, mapper.mappings.dst_to_src);
}

fn child_at_path<HAST: hyperast::types::HyperAST + Copy, IdD>(
    hyperast: HAST,
    arena: &impl ShallowDecompressedTreeStore<HAST, IdD>,
    root: &IdD,
    p: &[HAST::Idx],
) -> IdD
where
    HAST::IdN: std::fmt::Debug,
{
    let c = arena.child(root, p);
    let i = arena.original(&c);
    dbg!(&i);
    print_label(&hyperast, i);
    c
}

fn print_label<HAST: hyperast::types::HyperAST>(hyperast: &HAST, i: HAST::IdN) {
    use hyperast::types::NodeStore;
    let n = hyperast.node_store().resolve(&i);
    let l = n.try_get_label().unwrap();
    let l = hyperast.label_store().resolve(l);
    dbg!(l);
}

fn child_by_label<HAST: hyperast::types::HyperAST + Copy, IdD>(
    hyperast: HAST,
    arena: &impl DecompressedTreeStore<HAST, IdD>,
    root: &IdD,
    label: &str,
) -> IdD
where
    HAST::IdN: std::fmt::Debug,
    IdD: std::fmt::Debug,
{
    use hyperast::types::NodeStore;
    for c in arena.descendants(root) {
        let i = arena.original(&c);
        let n = hyperast.node_store().resolve(&i);
        let l = n.try_get_label().unwrap();
        let l = hyperast.label_store().resolve(l);
        if l == label {
            return c;
        }
    }
    unreachable!("provide a label present in the tree")
}
