//! Compare query matching performances
//!

use std::path::{Path, PathBuf};

use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration,
};

mod shared;
use hyperast_gen_ts_java::legion_with_refs::{tree_sitter_parse, JavaTreeGen};
use shared::*;

pub const QUERIES: &[BenchQuery] = &[
    (
        &[QUERY_OVERRIDES_SUBS[1]],
        QUERY_OVERRIDES.0,
        QUERY_OVERRIDES.1,
        "overrides",
        3229, // matches on spoon
    ),
    (
        &[QUERY_MAIN_METH_SUBS[0]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth",
        1, // matches on spoon
    ),
    (
        &[QUERY_RET_NULL_SUBS[0]],
        QUERY_RET_NULL.0,
        QUERY_RET_NULL.1,
        "ret_null",
        297, // matches on spoon
    ),
];

fn preps_default(
    p: (&BenchQuery, &[(std::path::PathBuf, String)]),
) -> (
    hyperast_tsquery::Query,
    hyperast::store::SimpleStores<hyperast_gen_ts_java::types::TStore>,
    Vec<legion::Entity>,
) {
    let (q, f) = p;
    let query = hyperast_tsquery::Query::new(q.1, hyperast_gen_ts_java::language()).unwrap();
    let mut stores =
        hyperast::store::SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen::new(&mut stores, &mut md_cache);
    let roots: Vec<_> = f
        .into_iter()
        .map(|(name, text)| {
            let tree = match tree_sitter_parse(text.as_bytes()) {
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
    (query, stores, roots)
}

fn preps_precomputed(
    (bench_param, f): (&BenchQuery, &[(std::path::PathBuf, String)]),
) -> (
    hyperast_tsquery::Query,
    hyperast::store::SimpleStores<hyperast_gen_ts_java::types::TStore>,
    Vec<legion::Entity>,
) {
    let (precomp, query) = hyperast_tsquery::Query::with_precomputed(
        bench_param.1,
        hyperast_gen_ts_java::language(),
        bench_param.0,
    )
    .unwrap();
    let mut stores =
        hyperast::store::SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default();
    let more = hyperast_tsquery::PreparedQuerying::<
    _,
    hyperast_gen_ts_java::types::TStore,
    _,
>::from(&precomp);
    let mut java_tree_gen = JavaTreeGen::with_preprocessing(&mut stores, &mut md_cache,more);
    let roots: Vec<_> = f
        .into_iter()
        .map(|(name, text)| {
            let tree = match tree_sitter_parse(text.as_bytes()) {
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
    (query, stores, roots)
}

fn compare_querying_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("QueryingRepeat2.2Spoon");
    group.sample_size(10);
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    group.plot_config(plot_config);

    let codes = "../../../../spoon/src/main/java"; // spoon dataset (only source code to avoid including resources), could add tests if necessary
    let codes = Path::new(&codes).to_owned();
    let codes = It::new(codes).map(|x| {
        let text = std::fs::read_to_string(&x).expect(&format!(
            "{:?} is not a java file or a dir containing java files: ",
            x
        ));
        (x, text)
    });
    let codes: Box<[_]> = codes.collect();
    for parameter in QUERIES.into_iter().map(|x| (x, codes.as_ref())) {
        group.throughput(criterion::Throughput::Elements(parameter.0 .4));

        bench_baseline(&mut group, parameter);
        bench_rust_baseline(&mut group, parameter);

        let pp = preps_default(parameter);
        group.bench_with_input(
            BenchmarkId::new("default", parameter.0 .3),
            &pp,
            |b, (query, stores, roots)| {
                b.iter(|| {
                    let mut count = 0;
                    for &n in roots {
                        let pos = hyperast::position::StructuralPosition::new(n);
                        let cursor = hyperast_tsquery::hyperast_cursor::TreeCursor::new(stores, pos);
                        let matches = query.matches(cursor);
                        count += black_box(matches.count());
                    }
                    debug_assert_eq!(count as u64, parameter.0 .4);
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("default_opt", parameter.0 .3),
            &pp,
            |b, (query, stores, roots)| {
                b.iter(|| {
                    let mut count = 0;
                    for &n in roots {
                        let pos =
                            hyperast::position::structural_pos::CursorWithPersistance::new(n);
                        let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(stores, pos);
                        let matches = query.matches(cursor);
                        count += black_box(matches.count());
                    }
                    debug_assert_eq!(count as u64, parameter.0 .4);
                })
            },
        );

        let pp = preps_precomputed(parameter);
        group.bench_with_input(
            BenchmarkId::new("sharing_precomputed", parameter.0 .3),
            &pp,
            |b, (query, stores, roots)| {
                b.iter(|| {
                    let mut count = 0;
                    for &n in roots {
                        let pos = hyperast::position::StructuralPosition::new(n);
                        let cursor = hyperast_tsquery::hyperast_cursor::TreeCursor::new(stores, pos);
                        let matches = query.matches(cursor);
                        count += black_box(matches.count());
                    }
                    debug_assert_eq!(count as u64, parameter.0 .4);
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("sharing_precomputed_opt", parameter.0 .3),
            &pp,
            |b, (query, stores, roots)| {
                b.iter(|| {
                    let mut count = 0;
                    for &n in roots {
                        let pos =
                            hyperast::position::structural_pos::CursorWithPersistance::new(n);
                        let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(stores, pos);
                        let matches = query.matches(cursor);
                        count += black_box(matches.count());
                    }
                    debug_assert_eq!(count as u64, parameter.0 .4);
                })
            },
        );
    }
    group.finish()
}

fn bench_baseline(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
    parameter: (&BenchQuery, &[(PathBuf, String)]),
) {
    let id = BenchmarkId::new("baseline", parameter.0 .3);
    group.bench_with_input(id, &parameter, |b, parameter| {
        let f: Box<[_]> = parameter
            .1
            .into_iter()
            .map(prep_baseline(parameter.0 .2))
            .collect();
        b.iter(|| {
            let mut count = 0;
            for (q, t, text) in f.into_iter() {
                let mut cursor = tree_sitter::QueryCursor::default();
                count += black_box(cursor.matches(&q, t.root_node(), text.as_bytes()).count());
            }
            debug_assert_eq!(count as u64, parameter.0 .4);
        })
    });
}

fn bench_rust_baseline(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
    parameter: (&BenchQuery, &[(PathBuf, String)]),
) {
    let pp: Box<[_]> = parameter
        .1
        .into_iter()
        .map(prep_baseline_query_cursor(parameter.0 .2))
        .collect();
    group.bench_with_input(
        BenchmarkId::new("baseline_query_cursor", parameter.0 .3),
        &pp,
        |b, p| {
            b.iter(|| {
                let mut count = 0;
                for (q, t, text) in p.into_iter() {
                    let cursor = hyperast_tsquery::default_impls::TreeCursor::new(
                        text.as_bytes(),
                        t.walk(),
                    );
                    count += black_box(q.matches(cursor).count());
                }
                debug_assert_eq!(count as u64, parameter.0 .4)
            })
        },
    );
}

criterion_group!(querying, compare_querying_group);
criterion_main!(querying);
