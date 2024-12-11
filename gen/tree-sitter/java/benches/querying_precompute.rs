use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

mod shared;
use hyper_ast_gen_ts_java::legion_with_refs::JavaTreeGen;
use shared::*;

pub const QUERIES: &[(&[&str], &str, &str, &str)] = &[
    (
        &[QUERY_OVERRIDES_SUBS[1]],
        QUERY_OVERRIDES.0,
        QUERY_OVERRIDES.1,
        "overrides",
    ),
    (
        &[QUERY_MAIN_METH_SUBS[1]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth",
    ),
];

fn prep_default<'store>(
    query: &str,
    name: &str,
    text: &[u8],
) -> (
    hyper_ast_tsquery::Query,
    hyper_ast::store::SimpleStores<hyper_ast_gen_ts_java::types::TStore>,
    hyper_ast::store::defaults::NodeIdentifier,
) {
    use hyper_ast_gen_ts_java::legion_with_refs;
    let query = hyper_ast_tsquery::Query::new(query, hyper_ast_gen_ts_java::language()).unwrap();

    let mut stores =
        hyper_ast::store::SimpleStores::<hyper_ast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default();
    let mut java_tree_gen = legion_with_refs::JavaTreeGen::new(&mut stores, &mut md_cache);

    let tree = match legion_with_refs::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    let full_node = java_tree_gen.generate_file(name.as_bytes(), text, tree.walk());

    (query, stores, full_node.local.compressed_node)
}

fn prep_precomputed<'store>(
    precomp: &[&str],
    query: &str,
    name: &str,
    text: &[u8],
) -> (
    hyper_ast_tsquery::Query,
    hyper_ast::store::SimpleStores<hyper_ast_gen_ts_java::types::TStore>,
    hyper_ast::store::defaults::NodeIdentifier,
) {
    use hyper_ast_gen_ts_java::legion_with_refs;
    let (precomp, query) = hyper_ast_tsquery::Query::with_precomputed(
        query,
        hyper_ast_gen_ts_java::language(),
        precomp,
    )
    .unwrap();

    let mut stores =
        hyper_ast::store::SimpleStores::<hyper_ast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen::new(&mut stores, &mut md_cache).with_more(precomp);

    let tree = match legion_with_refs::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    let full_node = java_tree_gen.generate_file(name.as_bytes(), text, tree.walk());

    (query, stores, full_node.local.compressed_node)
}

fn compare_querying_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("QueryingSpoon");

    // let codes = "../../../../stack-graphs/languages/tree-sitter-stack-graphs-java/test";
    let codes = "../../../../spoon/src/main/java";
    let codes = Path::new(&codes).to_owned();
    let codes = It::new(codes).map(|x| {
        let text = std::fs::read_to_string(&x).expect(&format!(
            "{:?} is not a java file or a dir containing java files: ",
            x
        ));
        (x, text)
    });
    let codes: Box<[_]> = codes.collect();
    // let queries: Vec<_> = QUERIES.iter().enumerate().collect();

    for (_i, p) in QUERIES.into_iter().map(|x| (x, codes.as_ref())).enumerate() {
        let i = p.0 .3;
        // group.throughput(Throughput::Bytes((p.0.len() + p.1.len()) as u64));
        group.bench_with_input(BenchmarkId::new("baseline", i), &p, |b, (q, f)| {
            b.iter(|| {
                for p in f.into_iter() {
                    let (q, t, text) = prep_baseline(q.2)(p);
                    let mut cursor = tree_sitter::QueryCursor::default();
                    black_box(cursor.matches(&q, t.root_node(), text.as_bytes()).count());
                }
            })
        });
        group.bench_with_input(
            BenchmarkId::new("baseline_query_cursor", i),
            &p,
            |b, (q, f)| {
                b.iter(|| {
                    for p in f.into_iter() {
                        let (q, t, text) = prep_baseline_query_cursor(q.2)(p);
                        let cursor = hyper_ast_tsquery::default_impls::TreeCursor::new(
                            text.as_bytes(),
                            t.walk(),
                        );
                        black_box(q.matches(cursor).count());
                    }
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("baseline_query_cursor_immediate", i),
            &p,
            |b, (q, f)| {
                b.iter(|| {
                    for p in f.into_iter() {
                        let (q, t, text) = prep_baseline_query_cursor(q.2)(p);
                        let cursor = hyper_ast_tsquery::default_impls::TreeCursor::new(
                            text.as_bytes(),
                            t.walk(),
                        );
                        black_box(q.matches(cursor).count());
                    }
                })
            },
        );
        group.bench_with_input(BenchmarkId::new("default", i), &p, |b, (q, f)| {
            b.iter(|| {
                for (name, text) in f.into_iter() {
                    let (q, stores, n) = prep_default(q.1, name.to_str().unwrap(), text.as_bytes());
                    let pos = hyper_ast::position::StructuralPosition::new(n);
                    let cursor = hyper_ast_tsquery::hyperast::TreeCursor::new(&stores, pos);
                    let matches = q.matches(cursor);
                    black_box(matches.count());
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("precomputed", i), &p, |b, (q, f)| {
            b.iter(|| {
                for (name, text) in f.into_iter() {
                    let (q, stores, n) =
                        prep_precomputed(q.0, q.1, name.to_str().unwrap(), text.as_bytes());
                    let pos = hyper_ast::position::StructuralPosition::new(n);
                    let cursor = hyper_ast_tsquery::hyperast::TreeCursor::new(&stores, pos);
                    let matches = q.matches(cursor);
                    black_box(matches.count());
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("sharing_default", i), &p, |b, (q, f)| {
            b.iter(|| {
                let query =
                    hyper_ast_tsquery::Query::new(q.1, hyper_ast_gen_ts_java::language()).unwrap();
                let mut stores = hyper_ast::store::SimpleStores::<
                    hyper_ast_gen_ts_java::types::TStore,
                >::default();
                let mut md_cache = Default::default();
                let mut java_tree_gen = hyper_ast_gen_ts_java::legion_with_refs::JavaTreeGen::new(
                    &mut stores,
                    &mut md_cache,
                );
                let roots: Vec<_> = f
                    .into_iter()
                    .map(|(name, text)| {
                        let tree = match hyper_ast_gen_ts_java::legion_with_refs::tree_sitter_parse(
                            text.as_bytes(),
                        ) {
                            Ok(t) => t,
                            Err(t) => t,
                        };
                        let full_node = java_tree_gen.generate_file(
                            name.to_str().unwrap().as_bytes(),
                            text.as_bytes(),
                            tree.walk(),
                        );
                        full_node.local.compressed_node
                    })
                    .collect();
                for n in roots {
                    let pos = hyper_ast::position::StructuralPosition::new(n);
                    let cursor = hyper_ast_tsquery::hyperast::TreeCursor::new(&stores, pos);
                    let matches = query.matches(cursor);
                    black_box(matches.count());
                }
            })
        });
        group.bench_with_input(
            BenchmarkId::new("sharing_precomputed", i),
            &p,
            |b, (q, f)| {
                b.iter(|| {
                    let (precomp, query) = hyper_ast_tsquery::Query::with_precomputed(
                        q.1,
                        hyper_ast_gen_ts_java::language(),
                        q.0,
                    )
                    .unwrap();
                    let mut stores = hyper_ast::store::SimpleStores::<
                        hyper_ast_gen_ts_java::types::TStore,
                    >::default();
                    let mut md_cache = Default::default();
                    let mut java_tree_gen =
                        JavaTreeGen::new(&mut stores, &mut md_cache).with_more(precomp);
                    let roots: Vec<_> = f
                        .into_iter()
                        .map(|(name, text)| {
                            let tree =
                                match hyper_ast_gen_ts_java::legion_with_refs::tree_sitter_parse(
                                    text.as_bytes(),
                                ) {
                                    Ok(t) => t,
                                    Err(t) => t,
                                };
                            let full_node = java_tree_gen.generate_file(
                                name.to_str().unwrap().as_bytes(),
                                text.as_bytes(),
                                tree.walk(),
                            );
                            full_node.local.compressed_node
                        })
                        .collect();
                    for n in roots {
                        let pos = hyper_ast::position::StructuralPosition::new(n);
                        let cursor = hyper_ast_tsquery::hyperast::TreeCursor::new(&stores, pos);
                        let matches = query.matches(cursor);
                        black_box(matches.count());
                    }
                })
            },
        );
    }
    group.finish()
}

criterion_group!(querying, compare_querying_group);
criterion_main!(querying);
