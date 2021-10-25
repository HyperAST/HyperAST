use num_traits::cast;

use crate::{matchers::{decompressed_tree_store::{CompletePostOrder, DecompressedTreeStore, ShallowDecompressedTreeStore, SimpleZsTree}, mapping_store::{DefaultMappingStore, MappingStore}, optimal::zs::ZsMatcher}, tests::{
        examples::{example_gt_java_code, example_gt_slides, example_zs_paper},
        simple_tree::{vpair_to_stores, Tree, LS, NS},
    }, tree::tree::{LabelStore, NodeStore}};

#[test]
fn test_zs_paper_for_initial_layout() {
    let (label_store, ..) = vpair_to_stores(example_zs_paper());
    assert_eq!(label_store.get_node_at_id(&0).to_owned(), b"");
}

#[test]
fn test_with_custom_example() {
    let (label_store, node_store, src, dst) = vpair_to_stores(example_gt_java_code());
    assert_eq!(label_store.get_node_at_id(&0).to_owned(), b"");
    let mappings = DefaultMappingStore::new();
    let mapper = ZsMatcher::<SimpleZsTree<u16, u16>, Tree, u16, NS<Tree>, LS<u16>>::matchh(
        &node_store,
        &label_store,
        &src,
        &dst,
        mappings,
    );
    let ZsMatcher {
        src_arena,
        dst_arena,
        mappings,
        ..
    } = mapper;
    let src = &(src_arena.len() as u16);
    let dst = &(dst_arena.len() as u16);
    assert_eq!(6, mappings.src_to_dst.iter().filter(|x| **x != 0).count());
    {
        mappings
            .src_to_dst
            .to_owned()
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                if *t == 0 {
                    None
                } else {
                    Some((
                        {
                            let g = src_arena.original(&cast(i).unwrap());
                            let n = node_store.get_node_at_id(&g).label;
                            std::str::from_utf8(&label_store.get_node_at_id(&n).to_owned())
                                .unwrap()
                                .to_owned()
                        },
                        {
                            let g = dst_arena.original(&(*t - 1));
                            let n = node_store.get_node_at_id(&g).label;
                            let a = label_store.get_node_at_id(&n).to_owned();
                            std::str::from_utf8(&a).unwrap().to_owned()
                        },
                    ))
                }
            })
            .for_each(|x| println!("{:?}", x))
    };
    assert!(mappings.has(
        &src_arena.child(&node_store, src, &[0]),
        &dst_arena.child(&node_store, dst, &[0, 0])
    ));
    assert!(mappings.has(
        &src_arena.child(&node_store, src, &[1]),
        &dst_arena.child(&node_store, dst, &[0, 1])
    ));
    assert!(mappings.has(
        &src_arena.child(&node_store, src, &[1, 0]),
        &dst_arena.child(&node_store, dst, &[0, 1, 0])
    ));
    assert!(mappings.has(
        &src_arena.child(&node_store, src, &[1, 2]),
        &dst_arena.child(&node_store, dst, &[0, 1, 2])
    ));
    assert!(mappings.has(
        &src_arena.child(&node_store, src, &[1, 3]),
        &dst_arena.child(&node_store, dst, &[0, 1, 3])
    ));
}
#[test]
fn test_with_custom_example2() {
    let (label_store, node_store, src, dst) = vpair_to_stores(example_gt_java_code());
    assert_eq!(label_store.get_node_at_id(&0).to_owned(), b"");
    let mappings = DefaultMappingStore::new();
    let mapper = ZsMatcher::<CompletePostOrder<u16, u16>, Tree, u16, NS<Tree>, LS<u16>>::matchh(
        &node_store,
        &label_store,
        &src,
        &dst,
        mappings,
    );
    let ZsMatcher {
        src_arena,
        dst_arena,
        mappings,
        ..
    } = mapper;
    let src = &(src_arena.len() as u16);
    let dst = &(dst_arena.len() as u16);
    assert_eq!(6, mappings.src_to_dst.iter().filter(|x| **x != 0).count());
    assert!(mappings.has(
        &src_arena.child(&node_store, src, &[0]),
        &dst_arena.child(&node_store, dst, &[0, 0])
    ));
    assert!(mappings.has(
        &src_arena.child(&node_store, src, &[1]),
        &dst_arena.child(&node_store, dst, &[0, 1])
    ));
    assert!(mappings.has(
        &src_arena.child(&node_store, src, &[1, 0]),
        &dst_arena.child(&node_store, dst, &[0, 1, 0])
    ));
    assert!(mappings.has(
        &src_arena.child(&node_store, src, &[1, 2]),
        &dst_arena.child(&node_store, dst, &[0, 1, 2])
    ));
    assert!(mappings.has(
        &src_arena.child(&node_store, src, &[1, 3]),
        &dst_arena.child(&node_store, dst, &[0, 1, 3])
    ));
}

#[test]
fn test_with_slide_example() {
    let (label_store, node_store, src, dst) = vpair_to_stores(example_gt_slides());
    assert_eq!(label_store.get_node_at_id(&0).to_owned(), b"");
    let mappings = DefaultMappingStore::new();
    let mapper = ZsMatcher::<SimpleZsTree<u16, u16>, Tree, u16, NS<Tree>, LS<u16>>::matchh(
        &node_store,
        &label_store,
        &src,
        &dst,
        mappings,
    );
    let ZsMatcher {
        src_arena,
        dst_arena,
        mappings,
        ..
    } = mapper;
    let src = &(src_arena.len() as u16); // todo try call root()
    let dst = &(dst_arena.len() as u16);
    assert_eq!(5, mappings.src_to_dst.iter().filter(|x| **x != 0).count());
    assert!(mappings.has(src, dst));
    assert!(mappings.has(
        &src_arena.child(&node_store, src, &[0, 0]),
        &dst_arena.child(&node_store, dst, &[0])
    ));
    assert!(mappings.has(
        &src_arena.child(&node_store, src, &[0, 0, 0]),
        &dst_arena.child(&node_store, dst, &[0, 0])
    ));
    assert!(mappings.has(
        &src_arena.child(&node_store, src, &[0, 1]),
        &dst_arena.child(&node_store, dst, &[1, 0])
    ));
    assert!(mappings.has(
        &src_arena.child(&node_store, src, &[0, 2]),
        &dst_arena.child(&node_store, dst, &[2])
    ));
}

#[test]
fn test_with_slide_example2() {
    let (label_store, node_store, src, dst) = vpair_to_stores(example_gt_slides());
    assert_eq!(label_store.get_node_at_id(&0).to_owned(), b"");
    let mappings = DefaultMappingStore::new();
    let mapper = ZsMatcher::<CompletePostOrder<u16, u16>, Tree, u16, NS<Tree>, LS<u16>>::matchh(
        &node_store,
        &label_store,
        &src,
        &dst,
        mappings,
    );
    let ZsMatcher {
        src_arena,
        dst_arena,
        mappings,
        ..
    } = mapper;
    let src = &(src_arena.len() as u16);
    let dst = &(dst_arena.len() as u16);
    assert_eq!(5, mappings.src_to_dst.iter().filter(|x| **x != 0).count());
    assert!(mappings.has(src, dst));
    assert!(mappings.has(
        &src_arena.child(&node_store, src, &[0, 0]),
        &dst_arena.child(&node_store, dst, &[0])
    ));
    assert!(mappings.has(
        &src_arena.child(&node_store, src, &[0, 0, 0]),
        &dst_arena.child(&node_store, dst, &[0, 0])
    ));
    assert!(mappings.has(
        &src_arena.child(&node_store, src, &[0, 1]),
        &dst_arena.child(&node_store, dst, &[1, 0])
    ));
    assert!(mappings.has(
        &src_arena.child(&node_store, src, &[0, 2]),
        &dst_arena.child(&node_store, dst, &[2])
    ));
}
