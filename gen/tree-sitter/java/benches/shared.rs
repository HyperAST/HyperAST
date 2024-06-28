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
  r#"(method_declaration
    (modifiers
      "public"
      "static"
    )
    type: (void_type)
    name: (_) (#EQ? "main")
)"#,
r#"(method_declaration
    name: (_) (#EQ? "main")
)"#,
    r#"(class_declaration
    body: (_
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
r#"(_ 
  name: (_) (#EQ? "main")
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
