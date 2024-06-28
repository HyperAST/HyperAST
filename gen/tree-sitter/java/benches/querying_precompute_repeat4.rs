//! further benchmarks query matching,
//! here focuses on impact of using different precomputed queries
//! including analyzing tests from spoon
use std::path::{Path, PathBuf};

use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration,
};

mod shared;
use shared::*;

pub const QUERIES: &[BenchQuery] = &[
    (
        &[QUERY_TESTS_SUBS[0]],
        QUERY_TESTS.0,
        QUERY_TESTS.1,
        "tests",
        1576, // matches on spoon
    ),
    (
        &[QUERY_TESTS_SUBS[1]],
        QUERY_TESTS.0,
        QUERY_TESTS.1,
        "tests_1",
        1576, // matches on spoon
    ),
    (
        &[QUERY_TESTS_SUBS[2]],
        QUERY_TESTS.0,
        QUERY_TESTS.1,
        "tests_2",
        1576, // matches on spoon
    ),
    (
        &[QUERY_TESTS_SUBS[3]],
        QUERY_TESTS.0,
        QUERY_TESTS.1,
        "tests_3",
        1576, // matches on spoon
    ),
    (
        &[QUERY_TESTS_SUBS[4]],
        QUERY_TESTS.0,
        QUERY_TESTS.1,
        "tests_4",
        1576, // matches on spoon
    ),
    (
        &[QUERY_TESTS_SUBS[3], QUERY_TESTS_SUBS[4]],
        QUERY_TESTS.0,
        QUERY_TESTS.1,
        "tests_3+4",
        1576, // matches on spoon
    ),
    (
        &[QUERY_OVERRIDES_SUBS[0]],
        QUERY_OVERRIDES.0,
        QUERY_OVERRIDES.1,
        "overrides",
        3856, // matches on spoon
    ),
    (
        &[QUERY_OVERRIDES_SUBS[1]],
        QUERY_OVERRIDES.0,
        QUERY_OVERRIDES.1,
        "overrides_1",
        3856, // matches on spoon
    ),
    (
        &[QUERY_OVERRIDES_SUBS[2]],
        QUERY_OVERRIDES.0,
        QUERY_OVERRIDES.1,
        "overrides_2",
        3856, // matches on spoon
    ),
    (
        &[QUERY_OVERRIDES_SUBS[3]],
        QUERY_OVERRIDES.0,
        QUERY_OVERRIDES.1,
        "overrides_3",
        3856, // matches on spoon
    ),
    (
        &[QUERY_OVERRIDES_SUBS[4]],
        QUERY_OVERRIDES.0,
        QUERY_OVERRIDES.1,
        "overrides_4",
        3856, // matches on spoon
    ),
    (
        &[QUERY_OVERRIDES_SUBS[3], QUERY_OVERRIDES_SUBS[4]],
        QUERY_OVERRIDES.0,
        QUERY_OVERRIDES.1,
        "overrides_3+4",
        3856, // matches on spoon
    ),
    (
        &[QUERY_RET_NULL_SUBS[0]],
        QUERY_RET_NULL.0,
        QUERY_RET_NULL.1,
        "ret_null",
        417, // matches on spoon
    ),
    (
        &[QUERY_RET_NULL_SUBS[1], QUERY_RET_NULL_SUBS[2]],
        QUERY_RET_NULL.0,
        QUERY_RET_NULL.1,
        "ret_null_1+2",
        417, // matches on spoon
    ),
    (
        &[QUERY_RET_NULL_SUBS[1]],
        QUERY_RET_NULL.0,
        QUERY_RET_NULL.1,
        "ret_null_1",
        417, // matches on spoon
    ),
    (
        &[QUERY_RET_NULL_SUBS[2]],
        QUERY_RET_NULL.0,
        QUERY_RET_NULL.1,
        "ret_null_2",
        417, // matches on spoon
    ),
    (
        &[QUERY_MAIN_METH_SUBS[0]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth",
        46, // matches on spoon
    ),
    (
        &[QUERY_MAIN_METH_SUBS[1]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth_1",
        46, // matches on spoon
    ),
    (
        &[QUERY_MAIN_METH_SUBS[2]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth_2",
        46, // matches on spoon
    ),
    (
        &[QUERY_MAIN_METH_SUBS[3]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth_3",
        46, // matches on spoon
    ),
    (
        &[QUERY_MAIN_METH_SUBS[1], QUERY_MAIN_METH_SUBS[2]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth_1+2",
        46, // matches on spoon
    ),
    (
        &[QUERY_MAIN_METH_SUBS[6]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth_6",
        46, // matches on spoon
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
        46, // matches on spoon
    ),
];

fn prep_baseline<'query, 'tree>(
    query: &'query str,
) -> impl Fn(&'tree (PathBuf, String)) -> (tree_sitter::Query, tree_sitter::Tree, &'tree str) + 'query
{
    |(_, text)| {
        let language = tree_sitter_java::language();
        let query = tree_sitter::Query::new(&language, query).unwrap();
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).unwrap();
        let tree = parser.parse(text, None).unwrap();
        (query, tree, text)
    }
}

fn prep_baseline_query_cursor(
    query: &str,
) -> impl Fn(&(PathBuf, String)) -> (hyper_ast_tsquery::Query, tree_sitter::Tree, &str) + '_ {
    |(_, text)| {
        let language = tree_sitter_java::language();
        let query = hyper_ast_tsquery::Query::new(query, tree_sitter_java::language()).unwrap();
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).unwrap();
        let tree = parser.parse(text, None).unwrap();
        (query, tree, text)
    }
}

fn preps_default(
    p: (&BenchQuery, &[(std::path::PathBuf, String)]),
) -> (
    hyper_ast_tsquery::Query,
    hyper_ast::store::SimpleStores<hyper_ast_gen_ts_java::types::TStore>,
    Vec<legion::Entity>,
) {
    let (q, f) = p;
    let query = hyper_ast_tsquery::Query::new(q.1, tree_sitter_java::language()).unwrap();
    let mut stores = hyper_ast::store::SimpleStores {
        label_store: hyper_ast::store::labels::LabelStore::new(),
        type_store: hyper_ast_gen_ts_java::types::TStore::default(),
        node_store: hyper_ast::store::nodes::legion::NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen =
        hyper_ast_gen_ts_java::legion_with_refs::JavaTreeGen::new(&mut stores, &mut md_cache);
    let roots: Vec<_> = f
        .into_iter()
        .map(|(name, text)| {
            let tree =
                match hyper_ast_gen_ts_java::legion_with_refs::tree_sitter_parse(text.as_bytes()) {
                    Ok(t) => t,
                    Err(t) => t,
                };
            log::trace!("preprocess file: {}", name.to_str().unwrap());
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
    hyper_ast_tsquery::Query,
    hyper_ast::store::SimpleStores<hyper_ast_gen_ts_java::types::TStore>,
    Vec<legion::Entity>,
) {
    dbg!(bench_param);
    let (precomp, query) = hyper_ast_tsquery::Query::with_precomputed(
        bench_param.1,
        tree_sitter_java::language(),
        bench_param.0,
    )
    .unwrap();
    query._check_preprocessed(0, bench_param.0.len());
    let mut stores = hyper_ast::store::SimpleStores {
        label_store: hyper_ast::store::labels::LabelStore::new(),
        type_store: hyper_ast_gen_ts_java::types::TStore::default(),
        node_store: hyper_ast::store::nodes::legion::NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = hyper_ast_gen_ts_java::legion_with_refs::JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
        more: precomp,
    };
    // let roots: Vec<_> = f
    //     .into_iter()
    //     .map(|(name, text)| {
    //         let name = &name.to_str().unwrap();
    //         let tree =
    //             match hyper_ast_gen_ts_java::legion_with_refs::tree_sitter_parse(text.as_bytes()) {
    //                 Ok(t) => t,
    //                 Err(t) => t,
    //             };
    //         log::trace!("preprocess file: {}", name);
    //         let full_node =
    //             java_tree_gen.generate_file(name.as_bytes(), text.as_bytes(), tree.walk());
    //         full_node.local.compressed_node
    //     })
    //     .collect();
    log::trace!("finished preprocessing");
    (query, stores, vec![])
}

fn compare_querying_group(c: &mut Criterion) {
    // log::set_logger(&LOGGER)
    //     .map(|()| log::set_max_level(log::LevelFilter::Trace))
    //     .unwrap();
    let mut group = c.benchmark_group("TO_RM");
    group.sample_size(10);
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    group.plot_config(plot_config);

    let codes = "../../../../stack-graphs/languages/tree-sitter-stack-graphs-java/test";
    let codes = "../../../../spoon/src/main/java"; // spoon dataset (only source code to avoid including resources), could add tests if necessary
    let codes = Path::new(&codes).to_owned();
    // let tests = "../../../../spoon/src/test/java/spoon/test/ctType/testclasses";
    let tests = "../../../../spoon/src/test/";
    let tests = Path::new(&tests).to_owned();
    let codes = It::new(codes).map(|x| {
        let text = std::fs::read_to_string(&x).expect(&format!(
            "{:?} is not a java file or a dir containing java files: ",
            x
        ));
        (x, text)
    });
    let tests = It::new(tests).filter_map(|x| {
        match std::fs::read_to_string(&x) {
            Ok(text) => {
                if let Some(ext) = x.extension() {
                    if ext.to_str() == Some("java") {
                        return Some((x, text));
                    }
                    log::trace!("wrong ext: {:?}", x);
                } else {
                    log::trace!("not ext: {:?}", x);
                }
            }
            Err(err) => {
                log::trace!("{:?} {}", x, err);
            }
        }
        None
        // .expect(&format!(
        //     "{:?} is not a java file or a dir containing java files: ",
        //     x
        // ));
    });
    let codes: Box<[_]> = codes.chain(tests).collect();
    for parameter in QUERIES.into_iter().map(|x| (x, codes.as_ref())) {
        // bench_baseline(&mut group, parameter);
        // bench_rust_baseline(&mut group, parameter);

        // let pp = preps_default(parameter);
        // group.bench_with_input(
        //     BenchmarkId::new("default", parameter.0 .3),
        //     &pp,
        //     |b, (query, stores, roots)| {
        //         b.iter(|| {
        //             let mut count = 0;
        //             for &n in roots {
        //                 let pos = hyper_ast::position::StructuralPosition::new(n);
        //                 let cursor = hyper_ast_tsquery::hyperast::TreeCursor::new(stores, pos);
        //                 let matches = query.matches(cursor);
        //                 count += black_box(matches.count());
        //             }
        //             assert_eq!(count as u64, parameter.0 .4);
        //         })
        //     },
        // );
        // group.bench_with_input(
        //     BenchmarkId::new("default_opt", parameter.0 .3),
        //     &pp,
        //     |b, (query, stores, roots)| {
        //         b.iter(|| {
        //             let mut count = 0;
        //             for &n in roots {
        //                 let pos =
        //                     hyper_ast::position::structural_pos::CursorWithPersistance::new(n);
        //                 let cursor = hyper_ast_tsquery::hyperast_opt::TreeCursor::new(stores, pos);
        //                 let matches = query.matches(cursor);
        //                 count += black_box(matches.count());
        //             }
        //             assert_eq!(count as u64, parameter.0 .4);
        //         })
        //     },
        // );

        let pp = preps_precomputed(parameter);
        // group.bench_with_input(
        //     BenchmarkId::new("precomputed", parameter.0 .3),
        //     &pp,
        //     |b, (query, stores, roots)| {
        //         b.iter(|| {
        //             let mut count = 0;
        //             for &n in roots {
        //                 let pos = hyper_ast::position::StructuralPosition::new(n);
        //                 let cursor = hyper_ast_tsquery::hyperast::TreeCursor::new(stores, pos);
        //                 let matches = query.matches(cursor);
        //                 count += black_box(matches.count());
        //             }
        //             assert_eq!(count as u64, parameter.0 .4);
        //         })
        //     },
        // );
        // group.bench_with_input(
        //     BenchmarkId::new("precomputed_opt", parameter.0 .3),
        //     &pp,
        //     |b, (query, stores, roots)| {
        //         b.iter(|| {
        //             let mut count = 0;
        //             for &n in roots {
        //                 // dbg!(stores
        //                 //     .label_store
        //                 //     .resolve(stores.node_store.resolve(n).try_get_label().unwrap()));
        //                 // eprintln!("{}", hyper_ast::nodes::SyntaxSerializer::new(stores, n));
        //                 let pos =
        //                     hyper_ast::position::structural_pos::CursorWithPersistance::new(n);
        //                 let cursor = hyper_ast_tsquery::hyperast_opt::TreeCursor::new(stores, pos);
        //                 let matches = query.matches(cursor);
        //                 count += black_box(matches.count());
        //             }
        //             assert_eq!(count as u64, parameter.0 .4);
        //         })
        //     },
        // );
    }
    group.finish()
}

fn bench_baseline(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
    parameter: (&BenchQuery, &[(PathBuf, String)]),
) {
    let p: Box<[_]> = parameter
        .1
        .into_iter()
        .map(prep_baseline(parameter.0 .2))
        .collect();
    let id = BenchmarkId::new("baseline", parameter.0 .3);
    group.bench_with_input(id, &p, |b, f| {
        b.iter(|| {
            let mut count = 0;
            for (q, t, text) in f.into_iter() {
                let mut cursor = tree_sitter::QueryCursor::default();
                count += black_box(cursor.matches(&q, t.root_node(), text.as_bytes()).count());
            }
            assert_eq!(count as u64, parameter.0 .4);
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
                    let cursor = hyper_ast_tsquery::default_impls::TreeCursor::new(
                        text.as_bytes(),
                        t.walk(),
                    );
                    count += black_box(q.matches(cursor).count());
                }
                assert_eq!(count as u64, parameter.0 .4)
            })
        },
    );
}

criterion_group!(querying, compare_querying_group);
criterion_main!(querying);

/// Iterates al files in provided directory
pub struct It {
    inner: Option<Box<It>>,
    outer: Option<std::fs::ReadDir>,
    p: Option<std::path::PathBuf>,
}

impl It {
    pub fn new(p: std::path::PathBuf) -> Self {
        Self {
            inner: None,
            outer: None,
            p: Some(p),
        }
    }
}

impl Iterator for It {
    type Item = std::path::PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(p) = &mut self.inner else {
            let Some(d) = &mut self.outer else {
                if let Ok(d) = self.p.as_mut()?.read_dir() {
                    self.outer = Some(d);
                    return self.next();
                } else {
                    return Some(self.p.take()?);
                }
            };
            let p = d.next()?.ok()?.path();
            self.inner = Some(Box::new(It::new(p)));
            return self.next();
        };
        let Some(p) = p.next() else {
            let p = self.outer.as_mut().unwrap().next()?.ok()?.path();
            self.inner = Some(Box::new(It::new(p)));
            return self.next();
        };
        Some(p)
    }
}

static LOGGER: SimpleLogger = SimpleLogger;

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Trace
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            if let (Some(file), Some(line)) = (record.file(), record.line()) {
                eprintln!("{}:{} {} - {}", file, line, record.level(), record.args());
            } else {
                eprintln!("{} - {}", record.level(), record.args());
            }
        }
    }

    fn flush(&self) {}
}
