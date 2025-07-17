//! further benchmarks query matching,
//! here focuses on impact of using different precomputed queries
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hyperast_gen_ts_java::legion_with_refs::{self, JavaTreeGen};
use std::hint::black_box;
use std::path::{Path, PathBuf};

mod shared;
use shared::*;

pub const QUERIES: &[BenchQuery] = &[
    (
        &[QUERY_RET_NULL_SUBS[0]],
        QUERY_RET_NULL.0,
        QUERY_RET_NULL.1,
        "ret_null",
        295, // matches on spoon
    ),
    (
        &[QUERY_RET_NULL_SUBS[1], QUERY_RET_NULL_SUBS[2]],
        QUERY_RET_NULL.0,
        QUERY_RET_NULL.1,
        "ret_null_1+2",
        295, // matches on spoon
    ),
    (
        &[QUERY_RET_NULL_SUBS[1]],
        QUERY_RET_NULL.0,
        QUERY_RET_NULL.1,
        "ret_null_1",
        295, // matches on spoon
    ),
    (
        &[QUERY_RET_NULL_SUBS[2]],
        QUERY_RET_NULL.0,
        QUERY_RET_NULL.1,
        "ret_null_2",
        295, // matches on spoon
    ),
    (
        &[QUERY_RET_NULL_SUBS[0]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth",
        1, // matches on spoon
    ),
    (
        &[QUERY_MAIN_METH_SUBS[0]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth_0",
        1, // matches on spoon
    ),
    (
        &[QUERY_MAIN_METH_SUBS[1]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth_1",
        1, // matches on spoon
    ),
    (
        &[QUERY_MAIN_METH_SUBS[2]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth_2",
        1, // matches on spoon
    ),
    (
        &[QUERY_MAIN_METH_SUBS[3]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth_3",
        1, // matches on spoon
    ),
    (
        &[QUERY_MAIN_METH_SUBS[1], QUERY_MAIN_METH_SUBS[2]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth_1+2",
        1, // matches on spoon
    ),
    (
        &[QUERY_MAIN_METH_SUBS[6]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth_6",
        1, // matches on spoon
    ),
    (
        &[
            QUERY_MAIN_METH_SUBS[4],
            QUERY_MAIN_METH_SUBS[5],
            QUERY_MAIN_METH_SUBS[6],
        ],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth_4+5+6",
        1, // matches on spoon
    ),
];

fn preps_default(
    p: &(&BenchQuery, &[(std::path::PathBuf, String)]),
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
            let tree = match legion_with_refs::tree_sitter_parse(text.as_bytes()) {
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
    (bench_param, f): &(&BenchQuery, &[(std::path::PathBuf, String)]),
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
    assert_eq!(precomp.enabled_pattern_count(), bench_param.0.len());
    let mut stores =
        hyperast::store::SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default();
    let more =
        hyperast_tsquery::PreparedQuerying::<_, hyperast_gen_ts_java::types::TStore, _>::from(
            &precomp,
        );
    let mut java_tree_gen = JavaTreeGen::with_preprocessing(&mut stores, &mut md_cache, more);
    let roots: Vec<_> = f
        .into_iter()
        .map(|(name, text)| {
            let name = &name.to_str().unwrap();
            let tree = match legion_with_refs::tree_sitter_parse(text.as_bytes()) {
                Ok(t) => t,
                Err(t) => t,
            };
            log::trace!("preprocess file: {}", name);
            let full_node =
                java_tree_gen.generate_file(name.as_bytes(), text.as_bytes(), tree.walk());
            full_node.local.compressed_node
        })
        .collect();
    log::trace!("finished preprocessing");
    (query, stores, roots)
}

fn compare_querying_group(c: &mut Criterion) {
    // log::set_logger(&LOGGER)
    //     .map(|()| log::set_max_level(log::LevelFilter::Trace))
    //     .unwrap();
    let mut group = c.benchmark_group("QueryingRepeat3Spoon");
    // group.sample_size(10);
    // let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    // group.plot_config(plot_config);

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let ws_path = root.parent().unwrap().parent().unwrap();
    let project_path = ws_path.parent().unwrap().join("spoon");
    assert!(
        project_path.is_dir(),
        "You must provide a Java project to analyze, such as Spoon"
    );
    // TODO clone repo in a tmp dir

    let oid = std::process::Command::new("git")
        .current_dir(&project_path)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(oid.status.success());
    assert_eq!(
        std::str::from_utf8(&oid.stdout),
        Ok("56e12a0c0e0e69ea70863011b4f4ca3305e0542b\n"),
        "expect to analyse Spoon's commit 56e12a0"
    );

    // spoon dataset (only source code to avoid including resources), could add tests if necessary
    let codes = project_path.join("src/main/java");
    let codes = It::new(codes).map(|x| {
        let text = std::fs::read_to_string(&x).expect(&format!(
            "{:?} is not a java file or a dir containing java files: ",
            x
        ));
        (x, text)
    });
    let codes: Box<[_]> = codes.collect();

    for parameter in QUERIES.into_iter().map(|x| (x, codes.as_ref())) {
        group.throughput(Throughput::Elements(parameter.0.4 as u64));
        bench_baseline(&mut group, parameter);
        bench_rust_baseline(&mut group, parameter);

        let mut pp = None;
        group.bench_with_input(
            BenchmarkId::new("default", parameter.0.3),
            &parameter,
            |b, parameter| {
                let (query, stores, roots) = &pp.get_or_insert(preps_default(parameter));
                b.iter(|| {
                    let mut count = 0;
                    for &n in roots {
                        let pos = hyperast::position::StructuralPosition::new(n);
                        let cursor =
                            hyperast_tsquery::hyperast_cursor::TreeCursor::new(stores, pos);
                        let matches = query.matches(cursor);
                        count += black_box(matches.count());
                    }
                    assert_eq!(count as u64, parameter.0.4);
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("default_opt", parameter.0.3),
            &parameter,
            |b, parameter| {
                let (query, stores, roots) = &pp.get_or_insert(preps_default(parameter));
                b.iter(|| {
                    let mut count = 0;
                    for &n in roots {
                        let pos = hyperast::position::structural_pos::CursorWithPersistance::new(n);
                        let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(stores, pos);
                        let matches = query.matches(cursor);
                        count += black_box(matches.count());
                    }
                    assert_eq!(count as u64, parameter.0.4);
                })
            },
        );
        drop(pp);

        let mut pp = None;
        group.bench_with_input(
            BenchmarkId::new("precomputed", parameter.0.3),
            &parameter,
            |b, parameter| {
                let (query, stores, roots) = &pp.get_or_insert(preps_precomputed(parameter));
                b.iter(|| {
                    let mut count = 0;
                    for &n in roots {
                        let pos = hyperast::position::StructuralPosition::new(n);
                        let cursor =
                            hyperast_tsquery::hyperast_cursor::TreeCursor::new(stores, pos);
                        let matches = query.matches(cursor);
                        count += black_box(matches.count());
                    }
                    assert_eq!(count as u64, parameter.0.4);
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("precomputed_opt", parameter.0.3),
            &parameter,
            |b, parameter| {
                let (query, stores, roots) = &pp.get_or_insert(preps_precomputed(parameter));
                b.iter(|| {
                    let mut count = 0;
                    for &n in roots {
                        let pos = hyperast::position::structural_pos::CursorWithPersistance::new(n);
                        let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(stores, pos);
                        let matches = query.matches(cursor);
                        count += black_box(matches.count());
                    }
                    assert_eq!(count as u64, parameter.0.4);
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
    let id = BenchmarkId::new("baseline", parameter.0.3);
    group.bench_with_input(id, &parameter, |b, parameter| {
        let f: Box<[_]> = parameter
            .1
            .into_iter()
            .map(prep_baseline(parameter.0.2))
            .collect();
        b.iter(|| {
            let mut count = 0;
            for (q, t, text) in f.iter() {
                let mut cursor = tree_sitter::QueryCursor::default();
                count += black_box(cursor.matches(&q, t.root_node(), text.as_bytes()).count());
            }
            assert_eq!(count as u64, parameter.0.4, "{}", parameter.0.3);
        })
    });
}

fn bench_rust_baseline(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
    parameter: (&BenchQuery, &[(PathBuf, String)]),
) {
    group.bench_with_input(
        BenchmarkId::new("baseline_query_cursor", parameter.0.3),
        &parameter,
        |b, parameter| {
            let p: Box<[_]> = parameter
                .1
                .into_iter()
                .map(prep_baseline_query_cursor(parameter.0.2))
                .collect();
            b.iter(|| {
                let mut count = 0;
                for (q, t, text) in p.iter() {
                    let cursor =
                        hyperast_tsquery::default_impls::TreeCursor::new(text.as_bytes(), t.walk());
                    count += black_box(q.matches(cursor).count());
                }
                assert_eq!(count as u64, parameter.0.4)
            })
        },
    );
}

criterion_group!(
    name = querying;
    config = Criterion::default().configure_from_args();
    targets = compare_querying_group
);
criterion_main!(querying);
