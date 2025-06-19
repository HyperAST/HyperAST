use hyperast::{full::FullNode, store::SimpleStores, tree_gen::StatsGlobalData};
use hyperast_gen_ts_java::{
    legion_with_refs::{self, JavaTreeGen, Local},
    types::TStore,
};

use hyper_diff::algorithms;

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
    let mut md_cache = Default::default(); // [cite: 133, 139]
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

fn test_compare_java(src: &[u8], dst: &[u8]) {
    let (stores, src, dst) = preprocess_diff(src, dst);

    println!("Src:\n {:?}\n\n Dst:\n {:?}", src.local.compressed_node, dst.local.compressed_node);
    let diff_result = algorithms::gumtree::diff(
        &stores,
        &src.local.compressed_node,
        &dst.local.compressed_node,
        1000, 0.5f64
    );
    dbg!(&diff_result.mapper.mappings);
    dbg!(&diff_result.mapper.mappings.src_to_dst.len());
    dbg!(&diff_result.actions);

    todo!()

}

#[test]
fn test_compare_1() {
    if let Some((src, dst)) = PAIRS.get(2) {
        test_compare_java(src, dst);
    }
}

#[test]
fn test_compare_all() {
    PAIRS
        .into_iter()
        .for_each(|(src, dst)| test_compare_java(src, dst));
}