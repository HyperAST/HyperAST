#![allow(unused)]

pub const QUERY_MAIN_METH: (&str, &str) = (
    r#"(program
      (class_declaration
        name: (identifier) @name
        body: (class_body
          (method_declaration
            (modifiers
              "public"
              "static"
            )
            type: (void_type)
            name: (identifier) (#EQ? "main")
          )
        )
      )
    )"#,
    r#"(program
  (class_declaration
    name: (identifier) @name
    body: (class_body
      (method_declaration
        (modifiers
          "public"
          "static"
        )
        type: (void_type)
        name: (identifier) @main (#eq? @main "main")
      )
    )
  )
)"#,
);

pub const QUERY_MAIN_METH_SUBS: &[&str] = &[
    r#"(method_declaration
    (modifiers
      "public"
      "static"
    )
    (void_type)
    (identifier) (#EQ? "main")
)"#,
    r#"(method_declaration
    name: (identifier) (#EQ? "main")
)"#,
    r#"(class_declaration
    body: (class_body
        (method_declaration)
    )
)"#,
    r#"(method_declaration)"#,
    r#"(class_declaration)"#,
    r#"(method_declaration
    (modifiers
      "static"
    )
)"#,
    r#"(method_declaration
  name: (identifier) (#EQ? "main")
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
    r#"
(method_declaration
    (modifiers
        (marker_annotation
            name: (_) (#EQ? "Override")
        )
    )
)"#,
    r#"
(marker_annotation
    name: (_) (#EQ? "Override")
)"#,
    r#"
(_
    name: (_) (#EQ? "Override")
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
    r#"
(class_declaration
    body: (_
        (method_declaration)
    )
)"#,
    r#"
(class_declaration
    body: (_
        (method_declaration
            (modifiers
                (marker_annotation)
            )
        )
    )
)"#,
];

pub const QUERY_RET_NULL: (&str, &str) = (
    r#"(return_statement (null_literal))"#,
    r#"(return_statement (null_literal))"#,
);

pub const QUERY_RET_NULL_SUBS: &[&str] = &[
    r#"(return_statement (null_literal))"#,
    r#"(return_statement)"#,
    r#"(null_literal)"#,
];

pub const QUERY_TESTS: (&str, &str) = (
    r#"(program
  (class_declaration
    name: (_) @name
    body: (_
      (method_declaration
        (modifiers
          (marker_annotation
            name: (_) (#EQ? "Test")
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
              name: (_)@anot (#eq? @anot "Test")
            )
          )
          name: (_)@meth_name
        )
      )
    )
)"#,
);

pub const QUERY_TESTS_SUBS: &[&str] = &[
    r#"(program
  (class_declaration
    name: (_) @name
    body: (_
      (method_declaration
        (modifiers
          (marker_annotation
            name: (_) (#EQ? "Test")
          )
        )
        name: (_)@meth_name
      )
    )
  )
)"#,
    r#"
(method_declaration
    (modifiers
        (marker_annotation
            name: (_) (#EQ? "Test")
        )
    )
)"#,
    r#"
(marker_annotation
    name: (_) (#EQ? "Test")
)"#,
    r#"
(_
    name: (_) (#EQ? "Test")
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
    r#"
(class_declaration
    body: (_
        (method_declaration)
    )
)"#,
    r#"
(class_declaration
    body: (_
        (method_declaration
            (modifiers
                (marker_annotation)
            )
        )
    )
)"#,
];

pub(crate) type BenchQuery<'a> = (&'a [&'a str], &'a str, &'a str, &'a str, u64);

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

pub(crate) fn prep_baseline<'query, 'tree, P>(
    query: &'query str,
) -> impl Fn(&'tree (P, String)) -> (tree_sitter::Query, tree_sitter::Tree, &'tree str) + 'query {
    |(_, text)| {
        let language = hyperast_gen_ts_java::language();
        let query = tree_sitter::Query::new(&language, query).unwrap();
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).unwrap();
        let tree = parser.parse(text, None).unwrap();
        // let size = tree.root_node().descendant_count();
        (query, tree, text)
    }
}

pub(crate) fn prep_baseline_query_cursor<P>(
    query: &str,
) -> impl Fn(&(P, String)) -> (hyperast_tsquery::Query, tree_sitter::Tree, &str) + '_ {
    |(_, text)| {
        let language = hyperast_gen_ts_java::language();
        let query = hyperast_tsquery::Query::new(query, hyperast_gen_ts_java::language()).unwrap();
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).unwrap();
        let tree = parser.parse(text, None).unwrap();
        (query, tree, text)
    }
}
