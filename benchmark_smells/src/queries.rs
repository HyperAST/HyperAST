fn test_method_body(body: &str) -> String {
  format!(r#"(method_declaration
        (modifiers
          (marker_annotation
            name: (_) (#EQ? "Test")
          )
        )
        name: (_)
        body: (_
        {body}
        )
)"#)
}

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
    )"#
  )
}
