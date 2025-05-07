use hyper_diff::{
    algorithms, decompressed_tree_store::ShallowDecompressedTreeStore,
    matchers::mapping_store::MappingStore,
};
use hyperast::{
    full::FullNode,
    store::SimpleStores,
    tree_gen::StatsGlobalData,
    types::{HyperAST, LabelStore, Labeled},
};
use hyperast_gen_ts_java::{
    legion_with_refs::{self, JavaTreeGen, Local},
    types::TStore,
};
use num_traits::{ToPrimitive, Zero};

const PAIRS: [(&[u8], &[u8]); 12] = [
    ("class A {}".as_bytes(), "class B {}".as_bytes()),
    ("class A {}".as_bytes(), "class A { class B {} }".as_bytes()),
    (
        "class A {}".as_bytes(),
        "class A { class A { class A { class A {} } } }".as_bytes(),
    ),
    (
        "void process() {
        int a = 1;
        int b = 2;
        int c = 3;
    }"
        .as_bytes(),
        "void process() {
        int c = 3;
        int x = 1;
        int b = 2;
    }"
        .as_bytes(),
    ),
    (
        "void calculate() {
        int result = 0;
        int temp1 = computeA();
        int temp2 = computeB();
        int finalValue = resulte + temp1 + temp2;
    }"
        .as_bytes(),
        "void calculate() {
        int temp2 = computeB();
        int temp3 = computeA();
        int result = 0;
        int finalValue = result + temp3 + temp2;
    }"
        .as_bytes(),
    ),
    (
        "void f() {
        int a = 0;
        int b = 1;
        int c = 2;
    }"
        .as_bytes(),
        "void f() {
        int c = 2;
        int b = 1;
        int a = 0;
    }"
        .as_bytes(),
    ),
    (
        "class A {
        void foo() {
            int x = 1;
        }
        void bar() {
            int x = 1;
        }
    }"
        .as_bytes(),
        "class A {
        void bar() {
            int x = 1;
        }
        void foo() {
            int x = 1;
        }
    }"
        .as_bytes(),
    ),
    (
        "{ int x = 1; }
    { int y = 2; }"
            .as_bytes(),
        "{ int y = 2; }
        { int x = 1; }"
            .as_bytes(),
    ),
    (
        b"{ int a = 1; } { int b = 2; }",
        b"{ int b = 2; } { int a = 1; }",
    ),
    (
        b"{ int x = 1; } { int x = 1; }",
        b"{ int x = 1; } { int x = 1; }",
    ),
    (
        b"class X {
        void a() { int x = 1; }
        void b() { int x = 1; }
    }",
        b"class X {
        void b() { int x = 1; }
        void a() { int x = 1; }
    }",
    ),
    (
        b"class X {
        void a() { int x = 1; }
    }",
        b"class X {
        void a() { int x = 1; }
        void b() { int x = 1; }
    }",
    ),
];

fn preprocess_diff(
    src: &[u8],
    dst: &[u8],
) -> (
    SimpleStores<TStore>,
    FullNode<StatsGlobalData, Local>,
    FullNode<StatsGlobalData, Local>,
) {
    let mut stores = SimpleStores::<TStore>::default();
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen::new(&mut stores, &mut md_cache);
    let tree = match legion_with_refs::tree_sitter_parse(src) {
        Ok(t) => t,
        Err(t) => t,
    };
    let src = java_tree_gen.generate_file(b"", src, tree.walk());
    let tree = match legion_with_refs::tree_sitter_parse(dst) {
        Ok(t) => t,
        Err(t) => t,
    };
    let dst = java_tree_gen.generate_file(b"", dst, tree.walk());
    return (stores, src, dst);
}

fn test(src: &[u8], dst: &[u8]) {
    println!(
        "\n-----------\nSrc:\n{}\nDst:\n{}",
        String::from_utf8_lossy(src),
        String::from_utf8_lossy(dst)
    );
    let (stores, src, dst) = preprocess_diff(src, dst);

    let result = algorithms::change_distiller::diff(
        &stores,
        &src.local.compressed_node,
        &dst.local.compressed_node,
    );

    let src_fmt = |src: u32| {
        let oid = result.mapper.src_arena.original(&src);
        let node = result.mapper.hyperast.node_store().resolve(oid);
        let t = result.mapper.hyperast.resolve_type(&oid);
        let label_id = node.try_get_label();
        if let Some(label_id) = label_id {
            format!(
                "{}: \"{}\"",
                t,
                result.mapper.hyperast.label_store().resolve(label_id)
            )
        } else {
            t.to_string()
        }
    };

    let dst_fmt = |dst: u32| {
        let oid = result.mapper.dst_arena.original(&dst);
        let node = result.mapper.hyperast.node_store().resolve(oid);
        let t = result.mapper.hyperast.resolve_type(&oid);
        let label_id = node.try_get_label();
        if let Some(label_id) = label_id {
            format!(
                "{}: \"{}\"",
                t,
                result.mapper.hyperast.label_store().resolve(label_id)
            )
        } else {
            t.to_string()
        }
    };

    println!("Mappings: {}", result.mapper.mapping.mappings.len());
    // copied and updated from crates/hyper_diff/src/matchers/mapping_store.rs
    for (i, x) in result.mapper.mapping.mappings.src_to_dst.iter().enumerate() {
        if !x.is_zero() {
            let src_idx: u32 = i.try_into().unwrap();
            let dst_idx: u32 = (x.to_usize().unwrap() - 1).try_into().unwrap(); // subtract 1 since all indexes are shifted by 1 to enable 0 to mean no mappings
            println!("SD {} -> {}", src_fmt(src_idx), dst_fmt(dst_idx));
        } else if i < result.mapper.mapping.mappings.src_to_dst.len() - 1 {
            // skip the last one since it gives index out of bounds errors
            let src_idx: u32 = i.try_into().unwrap();
            println!("S  {} -> no dst", src_fmt(src_idx));
        }
    }
}

#[test]
fn small_examples_cd() {
    PAIRS.into_iter().for_each(|(src, dst)| test(src, dst));
    assert!(false)
}
