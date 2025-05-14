use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use hyper_diff::{
    decompressed_tree_store::SimpleZsTree,
    matchers::{Decompressible, mapping_store::DefaultMappingStore, optimal::zs::ZsMatcher},
};
use hyperast::test_utils::simple_tree::{SimpleTree, vpair_to_stores};

fn compare_simple_tree_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("SimpleTree");

    const PAIRS: [(&[u8], &[u8]); 5] = [
        ("abaaacdefg".as_bytes(), "abcdefg".as_bytes()),
        (
            "za".as_bytes(),
            "qvvsdflflvjehrgipuerpq".as_bytes(),
        ),
        (
            "abaaeqrogireiuvnlrpgacdefg".as_bytes(),
            "aaaa".as_bytes(),
        ),
        (
            "abaaeqrogireiuvnlrpgacdefgabaaeqrogireiuvnlrpgacdefg".as_bytes(),
            "qvvsdflflvjehrgipuerpqqvvsdflflvjehrgipuerpq".as_bytes(),
        ),
        (
            "abaaeqro64646s468gireiuvnlrpg137zfaèàç-_éèàaç_è'ç(-cdefgrgeedbdsfdg6546465".as_bytes(),
            "qvvsdflflvjehrgegrhdbeoijovirejvoirzejvoerivjeorivjeroivjeroivjerovijrevoierjvoierjoipuerpq".as_bytes(),
        ),
    ];
    type ST<K> = SimpleTree<K>;

    pub(crate) fn example_gt_java_code() -> (ST<u8>, ST<u8>) {
        macro_rules! tree {
            ( $k:expr ) => {
                SimpleTree::new($k, None, vec![])
            };
            ( $k:expr, $l:expr) => {
                SimpleTree::new($k, Some($l), vec![])
            };
            ( $k:expr, $l:expr; [$($x:expr),+ $(,)?]) => {
                SimpleTree::new($k, Some($l), vec![$($x),+])
            };
            ( $k:expr; [$($x:expr),+ $(,)?]) => {
                SimpleTree::new($k, None, vec![$($x),+])
            };
        }
        let src = tree!(
            0, "a"; [
                tree!(0, "b"),
                tree!(0, "c"; [
                    tree!(0, "d"),
                    tree!(0, "e"),
                    tree!(0, "f"),
                    tree!(0, "r1"),
                ]),
        ]);
        let dst = tree!(
            0,"z"; [
                tree!( 0, "a"; [
                    tree!(0, "b"),
                    tree!(0, "c"; [
                        tree!(0, "d"),
                        tree!(1, "y"),
                        tree!(0, "f"),
                        tree!(0, "r2"),
                    ]),
                ]),
        ]);
        (src, dst)
    }
    let (stores, src, dst) = vpair_to_stores(example_gt_java_code());
    // assert_eq!(label_store.resolve(&0).to_owned(), b"");
    let label_store = &stores.label_store;
    let node_store = &stores.node_store;
    // println!(
    //     "src tree:\n{:?}",
    //     DisplayTree::new(label_store, node_store, src)
    // );
    // println!(
    //     "dst tree:\n{:?}",
    //     DisplayTree::new(label_store, node_store, dst)
    // );

    let pairs = PAIRS.into_iter().map(|x| (src, dst));

    for (i, p) in pairs.into_iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("zs", i), &p, |b, p| {
            b.iter(|| {
                ZsMatcher::<DefaultMappingStore<u16>, Decompressible<_, SimpleZsTree<_, u16>>>::matchh(
                    &stores, p.0, p.1,
                )
            })
        });
    }
    group.finish()
}

criterion_group!(simple_tree, compare_simple_tree_group);
criterion_main!(simple_tree);
