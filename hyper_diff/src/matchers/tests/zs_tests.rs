use crate::{
    decompressed_tree_store::{ShallowDecompressedTreeStore, SimpleZsTree},
    matchers::{
        mapping_store::{DefaultMappingStore, MappingStore},
        optimal::zs::ZsMatcher,
    },
    tests::examples::{example_gt_java_code, example_gt_slides, example_zs_paper},
    tree::simple_tree::{vpair_to_stores, DisplayTree},
};
use hyper_ast::types::LabelStore;

#[test]
fn test_zs_paper_for_initial_layout() {
    let (label_store, ..) = vpair_to_stores(example_zs_paper());
    println!("{}", label_store.resolve(&0).to_owned());
    // assert_eq!(label_store.resolve(&0).to_owned(), b"");
}

#[test]
fn test_with_custom_example() {
    let (label_store, node_store, src, dst) = vpair_to_stores(example_gt_java_code());
    // assert_eq!(label_store.resolve(&0).to_owned(), b"");
    println!(
        "src tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, src)
    );
    println!(
        "dst tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, dst)
    );
    let mapper = ZsMatcher::<DefaultMappingStore<u16>, SimpleZsTree<_, u16>>::matchh(
        &node_store,
        &label_store,
        src,
        dst,
    );
    let ZsMatcher {
        src_arena,
        dst_arena,
        mappings,
        ..
    } = mapper;
    let src = &src_arena.root();
    let dst = &dst_arena.root();
    assert_eq!(6, mappings.src_to_dst.iter().filter(|x| **x != 0).count());
    println!(
        "{}",
        mappings.display(
            &|src: u16| src_arena.original(&(src - 1)).to_string(),
            &|dst: u16| dst_arena.original(&(dst - 1)).to_string()
        )
    );
    println!(
        "src:{} dst:{}",
        src_arena.original(src),
        dst_arena.original(dst),
    );
    println!(
        "[0]:{} [0]:{}",
        src_arena.original(&src_arena.child(&node_store, src, &[0])),
        dst_arena.original(&dst_arena.child(&node_store, dst, &[0])),
    );
    assert!(mappings.has(
        &(src_arena.child(&node_store, src, &[0]) - 2),
        &(dst_arena.child(&node_store, dst, &[0, 0]) - 1)
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
    // assert_eq!(label_store.resolve(&0).to_owned(), b"");
    let mapper = ZsMatcher::<DefaultMappingStore<u16>, SimpleZsTree<_, u16>>::matchh(
        &node_store,
        &label_store,
        src,
        dst,
    );
    let ZsMatcher {
        src_arena,
        dst_arena,
        mappings,
    } = mapper;
    let src = &src_arena.root();
    let dst = &dst_arena.root();
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
    // assert_eq!(label_store.resolve(&0).to_owned(), b"");
    let mapper = ZsMatcher::<DefaultMappingStore<u16>, SimpleZsTree<_, u16>>::matchh(
        &node_store,
        &label_store,
        src,
        dst,
    );
    let ZsMatcher {
        src_arena,
        dst_arena,
        mappings,
        ..
    } = mapper;
    let src = &(src_arena.root()); // todo try call root()
    let dst = &(dst_arena.root());
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
    // assert_eq!(label_store.resolve(&0).to_owned(), b"");
    let mapper = ZsMatcher::<DefaultMappingStore<u16>, SimpleZsTree<_, u16>>::matchh(
        &node_store,
        &label_store,
        src,
        dst,
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
