use crate::{
    decompressed_tree_store::{CompletePostOrder, ShallowDecompressedTreeStore},
    matchers::{
        Decompressible, Mapper,
        heuristic::gt::greedy_bottom_up_matcher::GreedyBottomUpMatcher,
        mapping_store::{MappingStore, VecStore},
    },
    tests::examples::example_unstable,
};

use hyperast::{
    test_utils::simple_tree,
    types::{LabelStore, Labeled},
};
use hyperast::{test_utils::simple_tree::vpair_to_stores, types::DecompressedFrom as _};

#[test]
fn test_unstable_greedy_src() {
    let (stores, src, dst) = vpair_to_stores(example_unstable());
    // assert_eq!(label_store.resolve(&0).to_owned(), b"");
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
    let src_a =  
        child_at_path(hyperast, src_arena, src, &[0, 0])
    ;
    let src_x = 
        child_at_path(hyperast, src_arena, src, &[0])
    ;
    let src_b = child_at_path(hyperast, src_arena, src, &[1, 0]);
    let src_c = child_at_path(hyperast, src_arena, src, &[1, 1]);
    let src_y = child_at_path(hyperast, src_arena, src, &[1]);
    let dst_x = child_at_path(hyperast, dst_arena, dst, &[0]);
    let dst_a = child_at_path(hyperast, dst_arena, dst, &[1, 0]);
    let dst_b = child_at_path(hyperast, dst_arena, dst, &[1, 1]);
    let dst_c = child_at_path(hyperast, dst_arena, dst, &[1, 2]);
    let dst_y = child_at_path(hyperast, dst_arena, dst, &[1]);
    
    let mappings = &mut mapper.mappings;
    mappings.link(src_a, dst_a);
    mappings.link(src_b, dst_b);
    mappings.link(src_c, dst_c);

    GreedyBottomUpMatcher::<_, _, _, _, 1>::execute(&mut mapper);
    let Mapper {
        hyperast,
        mapping:
            crate::matchers::Mapping {
                src_arena,
                dst_arena,
                mappings,
            },
    } = mapper.into();
    let src = &src_arena.root();
    let dst = &dst_arena.root();
    dbg!(&mappings, src, dst);
    assert!(mappings.has(&src_x, &dst_y));
    assert!(!mappings.has(&src_y, &dst_x));
}
#[test]
fn test_unstable_greedy_dst() {
    let (stores, dst, src) = vpair_to_stores(example_unstable());
    // assert_eq!(label_store.resolve(&0).to_owned(), b"");
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
    let src_arena = &mapper.dst_arena;
    let dst_arena = &mapper.src_arena;
    let src = &src_arena.root();
    let dst = &dst_arena.root();
    dbg!(src_arena.child(src, &[0]));
    let src_a = child_at_path(hyperast, src_arena, src, &[0, 0]);
    let src_x = child_at_path(hyperast, src_arena, src, &[0]);
    let src_b = child_at_path(hyperast, src_arena, src, &[1, 0]);
    let src_c = child_at_path(hyperast, src_arena, src, &[1, 1]);
    let src_y = child_at_path(hyperast, src_arena, src, &[1]);
    let dst_x = child_at_path(hyperast, dst_arena, dst, &[0]);
    let dst_a = child_at_path(hyperast, dst_arena, dst, &[1, 0]);
    let dst_b = child_at_path(hyperast, dst_arena, dst, &[1, 1]);
    let dst_c = child_at_path(hyperast, dst_arena, dst, &[1, 2]);
    let dst_y = child_at_path(hyperast, dst_arena, dst, &[1]);
    
    let mappings = &mut mapper.mappings;
    mappings.link(src_a, dst_a);
    mappings.link(src_b, dst_b);
    mappings.link(src_c, dst_c);

    GreedyBottomUpMatcher::<_, _, _, _, 1>::execute(&mut mapper);
    let Mapper {
        hyperast,
        mapping:
            crate::matchers::Mapping {
                src_arena,
                dst_arena,
                mappings,
            },
    } = mapper.into();
    let src = &src_arena.root();
    let dst = &dst_arena.root();
    dbg!(&mappings, src, dst);
    assert!(mappings.has(&src_y, &dst_y));
    assert!(!mappings.has(&src_x, &dst_x));
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
