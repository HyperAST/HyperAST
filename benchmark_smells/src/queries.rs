fn test_method_body(body: &str) -> String {
    format!(
        r#"(method_declaration
    (modifiers
      (marker_annotation
        name: (_) (#EQ? "Test")
      )
    )
    name: (_)
    body: (_
      {body}
    )
) @root"#
    )
}

const ASSERT_THAT: &str = r#"(expression_statement
  (method_invocation
      name: (identifier) (#EQ? "assertThat")
  )
)"#;

const ASSERT_SAME: &str = r#"(expression_statement
  (method_invocation
      name: (identifier) (#EQ? "assertSame")
  )
)"#;

const ASSERT_EQUALS: &str = r#"(expression_statement
  (method_invocation
      name: (identifier) (#EQ? "assertEquals")
  )
)"#;

// query for a method with two assertThat invocations
pub fn assertion_roulette() -> String {
    test_method_body(
        r#"(expression_statement
        (method_invocation
            name: (identifier) (#EQ? "assertThat")
        )
    ) .
    (expression_statement
        (method_invocation
            name: (identifier) (#EQ? "assertThat")
        )
    )"#,
    )
}

pub fn exception_handling() -> String {
    let tc = |s| {
        format!(
            r#"
      (try_statement
        (block
          {s}
    )
    (catch_clause)
)"#
        )
    };
    format!(
        "{}\n{}",
        tc(r#"(expression_statement 
      (method_invocation
        (identifier) (#EQ? "fail")
      )
    )"#),
        tc(r#"(expression_statement 
      (method_invocation
        (identifier) (#EQ? "fail")
      )
    )
    ."#)
    )
}

pub fn conditional_logic() -> String {
    // let no_alt = |s| format!(r#"(if_statement
    //   concequence: (_
    //     {s}
    //   )
    //   !alternative
    // )"#);
    format!(
        r#"(if_statement
      concequence: (_
        {ASSERT_THAT}
      )
      !alternative
    )
    (if_statement
      concequence: (_
        {ASSERT_SAME}
      )
      !alternative
    )
    (if_statement
      concequence: (_
        {ASSERT_EQUALS}
      )
      !alternative
    )
    "#
    )
}

pub fn empty_test() -> String {
    test_method_body(
        r#"
      "{"
      .
      "}"
    "#,
    )
}

pub fn constructor_initialization() -> String {
    let test = test_method_body("");

    format!(
        r#"(class_declaration
    
    (constructor_declaration)

    {test}
    
    )"#
    )
}

pub fn default_test() -> String {
    let test = test_method_body("");
    format!(
        r#"(class_declaration
    name: (_) (#EQ? "ExampleUnitTest")
    body: (_
      {test}
    )
)
(class_declaration
    name: (_) (#EQ? "ExampleInstrumentedTest")
    body: (_
      {test}
    )
)"#
    )
}

pub fn duplicated_assert() -> String {
    test_method_body(&format!(
        r#"{ASSERT_THAT}@first
  {ASSERT_THAT}@second
  (#eq? @first @second)"#
    ))
}

fn ignored_test() -> String {
    format!(
        r#"(method_declaration
  (modifiers
    (marker_annotation
      name: (_) (#EQ? "Ignored")
    )
    (marker_annotation
      name: (_) (#EQ? "Test")
    )
  )
  name: (_)
  body: (_)
) @root"#
    )
}

pub fn magic_number_test() -> String {
    let assertion_p1 = |s| {
        format!(
            r#"(expression_statement
    (method_invocation
        name: (identifier) (#EQ? "assertThat")
        (argument_list ({s}))
    )
  )"#
        )
    };
    let assertion_p2 = |s| {
        format!(
            r#"(expression_statement
    (method_invocation
        name: (identifier) (#EQ? "assertThat")
        (argument_list (_) . ({s}))
    )
  )"#
        )
    };
    format!(
        "{}\n{}",
        test_method_body(&assertion_p1("number_literal")),
        test_method_body(&assertion_p2("number_literal"))
    )
}

pub fn mistery_guest() -> String {
  test_method_body(r#"(expression_statement (assignment_expression
    right: (method_invocation
        object: (identifier) (#EQ? "File")
        name:   (identifier) (#EQ? "createTempFile")
    )
  ))"#)
}

pub fn redundant_print() -> String {
  test_method_body(r#"(expression_statement
    right: (method_invocation
        name:   (identifier) (#EQ? "println")
    )
  )"#)
}

pub fn redundant_assertion() -> String {
  test_method_body(
      r#"(expression_statement
      (method_invocation
          name: (identifier) (#EQ? "assertThat")
          (argument_list (_)@a (_)@b) (#eq? @a @b)
      )
  )"#,
  )
}

pub fn sensitive_equality() -> String {
  test_method_body(
      r#"(expression_statement
      (method_invocation
          name: (identifier) (#EQ? "assertEquals")
          (argument_list (method_invocation 
            name: (identifier) (#EQ? "toString")
          )
      )
  )"#,
  )
}

pub fn sleepy_test() -> String {
  test_method_body(
      r#"(expression_statement
      (method_invocation
        object: (identifier) (#EQ? "Thread")
        name: (identifier) (#EQ? "sleep")
      )
  )"#,
  )
}