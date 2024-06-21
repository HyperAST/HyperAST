use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

pub const QUERY_MAIN_METH: (&str, &str) = (
    r#"(program
(class_declaration 
name: (_) @name
body: (_
  (method_declaration
    (modifiers
      "public"
      "static"
    )
    type: (void_type)
    name: (_) (#EQ? "main")
  )
)
)
)"#,
    r#"(program
(class_declaration 
name: (_) @name
body: (_
  (method_declaration
    (modifiers
      "public"
      "static"
    )
    type: (void_type)
    name: (_) @main (#eq? @main "main")
  )
)
)
)"#,
);

pub const QUERY_MAIN_METH_SUBS: &[&str] = &[
    r#"
    (marker_annotation
        name: (_) (#EQ? "Override")
    )"#,
    r#"
(method_declaration
  (modifiers
    "public"
    "static"
  )
  type: (void_type)
  name: (_) (#EQ? "main")
)"#,
    r#"
(method_declaration
    (modifiers
        (marker_annotation)
    )
)"#,
    r#"
(class_declaration
    name: (_) @name
    body: (_
        (method_declaration)
    )
)"#,
];
pub const QUERY_OVERRIDES: (&str, &str) = (
    r#"(program
(class_declaration
  name: (_) @name
  body: (_
    (method_declaration
      (modifiers
        (marker_annotation
          name: (_) (#EQ? "Override")
        )
      )
      name: (_)@meth_name
    )
  )
)
  )"#,
    r#"(program
    (class_declaration
      name: (_) @name
      body: (_
        (method_declaration
          (modifiers
            (marker_annotation
              name: (_)@anot (#eq? @anot "Override")
            )
          )
          name: (_)@meth_name
        )
      )
    )
)"#,
);

pub const QUERY_OVERRIDES_SUBS: &[&str] = &[
    r#"
            (marker_annotation
                name: (_) (#EQ? "Override")
            )"#,
    r#"
    (method_declaration
        (modifiers
            (marker_annotation
                name: (_) (#EQ? "Override")
            )
        )
    )"#,
    r#"
            (method_declaration
                (modifiers
                    (marker_annotation)
                )
            )"#,
    r#"
    (class_declaration
        name: (_) @name
        body: (_
            (method_declaration)
        )
    )"#,
];

pub const QUERIES: &[(&[&str], &str, &str, &str, usize)] = &[
    (
        &[QUERY_OVERRIDES_SUBS[1]],
        QUERY_OVERRIDES.0,
        QUERY_OVERRIDES.1,
        "overrides",
        1,
    ),
    (
        &[QUERY_OVERRIDES_SUBS[1]],
        QUERY_OVERRIDES.0,
        QUERY_OVERRIDES.1,
        "overrides",
        2,
    ),
    (
        &[QUERY_MAIN_METH_SUBS[1]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth",
        1,
    ),
    (
        &[QUERY_MAIN_METH_SUBS[1]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth",
        2,
    ),
    (
        &[QUERY_MAIN_METH_SUBS[1]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth",
        3,
    ),
    // (
    //     &[QUERY_MAIN_METH_SUBS[1]],
    //     QUERY_MAIN_METH.0,
    //     QUERY_MAIN_METH.1,
    //     "main_meth",
    //     4,
    // ),
];


fn prep_baseline<'query, 'tree>(
    query: &'query str,
    name: &str,
    text: &'tree [u8],
) -> (tree_sitter::Query, tree_sitter::Tree) {
    let language = tree_sitter_java::language();

    let query = tree_sitter::Query::new(&language, query).unwrap();

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).unwrap();
    let tree = parser.parse(text, None).unwrap();

    (query, tree)
}

fn compare_querying_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("QueryingRepeatSpoon");
    group.sample_size(10);

    let codes = "../../../../stack-graphs/languages/tree-sitter-stack-graphs-java/test";
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

    for p in QUERIES.into_iter().map(|x| (x, codes.as_ref())) {
        group.throughput(Throughput::Elements(p.0 .4 as u64));

        group.bench_with_input(
            BenchmarkId::new(format!("baseline-{}", p.0 .3), p.0 .4),
            &p,
            |b, (q, f)| {
                b.iter(|| {
                    for _ in 0..p.0 .4 {
                        for (name, text) in f.into_iter() {
                            let (q, t) =
                                prep_baseline(q.2, name.to_str().unwrap(), text.as_bytes());
                            let mut cursor = tree_sitter::QueryCursor::default();
                            black_box(cursor.matches(&q, t.root_node(), text.as_bytes()).count());
                        }
                    }
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new(format!("sharing_default-{}", p.0 .3), p.0 .4),
            &p,
            |b, (q, f)| {
                b.iter(|| {
                    let query =
                        hyper_ast_tsquery::Query::new(q.1, tree_sitter_java::language()).unwrap();
                    let mut stores = hyper_ast::store::SimpleStores {
                        label_store: hyper_ast::store::labels::LabelStore::new(),
                        type_store: hyper_ast_gen_ts_java::types::TStore::default(),
                        node_store: hyper_ast::store::nodes::legion::NodeStore::new(),
                    };
                    let mut md_cache = Default::default();
                    let mut java_tree_gen =
                        hyper_ast_gen_ts_java::legion_with_refs::JavaTreeGen::new(
                            &mut stores,
                            &mut md_cache,
                        );
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
                    for _ in 0..p.0 .4 {
                        for &n in &roots {
                            let pos = hyper_ast::position::StructuralPosition::new(n);
                            let cursor = hyper_ast_tsquery::hyperast::TreeCursor::new(&stores, pos);
                            let matches = query.matches(cursor);
                            black_box(matches.count());
                        }
                    }
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new(format!("sharing_precomputed-{}", p.0 .3), p.0 .4),
            &p,
            |b, (q, f)| {
                b.iter(|| {
                    let (precomp, query) = hyper_ast_tsquery::Query::with_precomputed(
                        q.1,
                        tree_sitter_java::language(),
                        q.0,
                    )
                    .unwrap();
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
                    for _ in 0..p.0 .4 {
                        for &n in &roots {
                            let pos = hyper_ast::position::StructuralPosition::new(n);
                            let cursor = hyper_ast_tsquery::hyperast::TreeCursor::new(&stores, pos);
                            let matches = query.matches(cursor);
                            black_box(matches.count());
                        }
                    }
                })
            },
        );
    }
    group.finish()
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
