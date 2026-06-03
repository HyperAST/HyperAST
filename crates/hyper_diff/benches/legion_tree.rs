use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hyper_diff::{
    decompressed_tree_store::SimpleZsTree,
    matchers::{Decompressible, mapping_store::DefaultMappingStore, optimal::zs::ZsMatcher},
};
use hyperast_gen_ts_java::{
    legion_with_refs::{self, JavaTreeGen},
    types::TStore,
};

fn compare_simple_tree_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("LegionTree");

    const PAIRS: [(&[u8], &[u8]); 3] = [
        ("class A {}".as_bytes(), "class B {}".as_bytes()),
        (
            "class A {}".as_bytes(),
            "class A { class B {} }".as_bytes(),
        ),
        (
            "class A {}".as_bytes(),
            "class A { class A { class A { class A { class A { class A {} } } } } }".as_bytes(),
        ),
    ];

    let mut stores = hyperast::store::SimpleStores::<TStore>::default();
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen::new(&mut stores, &mut md_cache);

    let pairs = PAIRS.into_iter().map(|(src, dst)| {
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

        (src.local, dst.local)
    }).collect::<Vec<_>>();

    for (i, p) in pairs.into_iter().enumerate() {
        group.throughput(Throughput::Elements((p.0.metrics.size + p.0.metrics.size) as u64));
        group.bench_with_input(BenchmarkId::new("zs", i), &p, |b, p| {
      
            b.iter(|| {
                ZsMatcher::<DefaultMappingStore<u16>, Decompressible<_, SimpleZsTree<_, u16>>>::matchh(
                    &stores, p.0.compressed_node, p.1.compressed_node,
                )
            })
        });
    }
    group.finish()
}

criterion_group!(simple_tree, compare_simple_tree_group);
criterion_main!(simple_tree);
