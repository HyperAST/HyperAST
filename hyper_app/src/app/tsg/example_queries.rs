use crate::app::types::Config;

#[derive(Clone)]
pub(super) struct Query {
    pub(crate) description: &'static str,
    pub(crate) query: &'static str,
}
#[derive(Clone)]
pub(super) struct Example {
    pub(crate) name: &'static str,
    pub(crate) commit: Commit,
    pub(crate) config: Config,
    pub(crate) commits: usize,
    pub(crate) query: Query,
    pub(crate) path: &'static str,
}

#[derive(Clone)]
pub(crate) enum Forge {
    GitHub,
    GitLab,
}

#[derive(Clone)]
pub(crate) struct Repo {
    pub(crate) forge: Forge,
    pub(crate) user: &'static str,
    pub(crate) name: &'static str,
}
#[derive(Clone)]
pub(crate) struct Commit {
    pub(crate) repo: Repo,
    pub(crate) id: &'static str,
}

impl From<&Repo> for super::super::types::Repo {
    fn from(value: &Repo) -> Self {
        Self {
            user: value.user.into(),
            name: value.name.into(),
        }
    }
}
impl From<&Commit> for super::super::types::Commit {
    fn from(value: &Commit) -> Self {
        Self {
            repo: (&value.repo).into(),
            id: value.id.into(),
        }
    }
}

const BASE_SPOON_EX: Example = Example {
    name: "",
    commit: Commit {
        repo: Repo {
            forge: Forge::GitHub,
            user: "INRIA",
            name: "spoon",
        },
        id: "56e12a0c0e0e69ea70863011b4f4ca3305e0542b",
    },
    config: Config::MavenJava,
    commits: 1,
    query: Query {
        description: "",
        query: "",
    },
    path: "",
};

pub(super) const EXAMPLES: &[Example] = &[
    Example {
        name: "default example (Java)",
        query: Query {
            description: "Count the number of class declarations.",
            query: "(program)@prog {
    node @prog.defs
    node @prog.lexical_scope
}",
        },
        path: "src/main/java/spoon/JLSViolation.java",
        ..BASE_SPOON_EX
    },
    Example {
        name: "top level decls (Java)",
        query: Query {
            description: "Find top level public delcarations, capturing its name and package.",
            query: r#"(program
  (package_declaration (_)@pkg)
  (declaration 
    (modifiers "public")
    name: (_) @name
  )
) {
    node prog
    attr (prog) package = (source-text @pkg)
    attr (prog) decl_name = (source-text @name)
}"#,
        },
        ..BASE_SPOON_EX
    },
    Example {
        name: "test methods (Java)",
        query: Query {
            description: "Find test methods",
            query: r#"(program
  (package_declaration (_)@pkg)
  (declaration 
    (modifiers "public")
    name: (_) @name
    body: (_
      (method_declaration
        (modifiers . (marker_annotation 
          name: (_)@_anot (#eq? @_anot "Test")
        ))
        name: (_)@meth
      )
    )
  )
) {
    node prog
    attr (prog) package = (source-text @pkg)
    attr (prog) decl_name = (source-text @name)
    attr (prog) meth_name = @meth
}"#,
        },
        ..BASE_SPOON_EX
    },
    Example {
        name: "methods per declaration (Java)",
        query: Query {
            description: "Find methods for each declaration.
The pattern '[(meth)@m, (_)]*' allows to match '@m' even when interleaved with other siblings not captured in '@m', 
otherwise, this query would return each 'declaration' as many time as '@m' is interleaved.",
            query: r#"(program
  (package_declaration (_)@pkg)
  (declaration
    name: (_) @name
    body: (_
        [
          (method_declaration
            name: (_)@meth
          )
          (_)
        ]*
    )
  )
) {
    node prog
    attr (prog) package = (source-text @pkg)
    attr (prog) decl_name = (source-text @name)
    attr (prog) meth_names = [(source-text x) for x in @meth]
}"#,
        },
        ..BASE_SPOON_EX
    },
    Example {
        name: "test methods per test class (Java)",
        query: Query {
            description: "Find test methods for each test class.
Due to possible interleaved annotations and late predicates,
it is very important to match annotations immediately i.e '@Test'.
The quantifier on 'marker_annotation' is important to match '@Test' after other annotations.
Without those, on large clases with many anotated methods,
the query engine has to branch quadratically on each child of 'modifiers'.
Indeed, the semantic of the query is actually to produce a different match for each individual annotation
(when there are multiple annotation marker).",
            query: r#"(program
  (package_declaration (_)@pkg)
  (class_declaration 
    (modifiers "public")
    name: (_) @name
    body: (_
        [
          (method_declaration
            (modifiers
              . ; this is very important otherwise the complexity explodes
              [
                (marker_annotation 
                  name: (_)@_anot (#any-eq? @_anot "Test")
                )
                (_)
              ]+
            )
            name: (_)@meth
          )
          (_)
        ]+
    )
  )
) {
    if (not (is-empty @meth)) {
        node prog
        attr (prog) package = (source-text @pkg)
        attr (prog) decl_name = (source-text @name)
        attr (prog) meth_names = [(source-text x) for x in @meth]
    }
}"#,
        },
        ..BASE_SPOON_EX
    },
    Example {
        name: "bad query (Java)",
        query: Query {
            description: "Version of previous example 
that hangs because it executes quadraticaly. 
It also splits lists of methods when interleaved by other members",
            query: r#"(program
  (package_declaration (_)@pkg)
  (class_declaration 
    (modifiers "public")
    name: (_) @name
    body: (_
      (method_declaration
        (modifiers
          (marker_annotation 
            name: (_)@_anot (#eq? @_anot "Test")
          )
        )
        name: (_)@meth
      )*
    )
  )
) {
    if (not (is-empty @meth)) {
        node prog
        attr (prog) package = (source-text @pkg)
        attr (prog) decl_name = (source-text @name)
        attr (prog) meth_names = [(source-text x) for x in @meth]
    }
}"#,
        },
        ..BASE_SPOON_EX
    },
    Example {
        name: "imports named Test (Java)",
        query: Query {
            description: "",
            query: r#"(program
  (package_declaration (_)@pkg)
  (import_declaration 
    (scoped_absolute_identifier 
      name: (_)@name (#eq? @name "Test")
    )
  )@imp
) {
    node prog
    attr (prog) package = (source-text @pkg)
    attr (prog) imp = (source-text @imp)
}"#,
        },
        ..BASE_SPOON_EX
    },
    Example {
        name: "Signature of all declarations (Java)",
        query: Query {
            description: "TODO Signature of all declarations
TODO implement low-hanging parts of signatures, do not fully qualify parameters and type
TODO implement using the scope graph",
            query: r#"(declaration)@decl {
    node n
    attr (n) signature = (signature @decl)
}"#,
        },
        ..BASE_SPOON_EX
    },
];

// TODO example about attributes on edges
// (program)@prog {
//     node @prog.defs
//     attr (@prog.defs) node = @prog
//     edge @prog.defs -> @prog.defs
//     attr (@prog.defs -> @prog.defs) precedence = 1
// }
