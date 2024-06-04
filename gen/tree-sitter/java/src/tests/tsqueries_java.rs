use std::path::{Path, PathBuf};

use hyper_ast::store::{defaults::NodeIdentifier, SimpleStores};

use crate::tsg::{CODE, CODE1, CODE3, CODES, QUERIES};

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

/// WARN the path need to be set to a directory containing some Java files
/// NOTE I use the dataset in the stack-graphs repo
fn tsg_test(p: &str) -> String {
    let r = "../../../../stack-graphs/languages/tree-sitter-stack-graphs-java/test/";
    std::fs::read_to_string(Path::new(r).join(p)).unwrap()
}

fn run_recursive(query: &str, text: &[u8]) -> usize {
    let (matcher, stores, code) = prep_recursive(query, text);

    use crate::iter::IterAll as JavaIter;
    use crate::types::TStore;
    type It<'a, HAST> = JavaIter<'a, hyper_ast::position::StructuralPosition, HAST>;
    let matchs =
        matcher.apply_matcher::<SimpleStores<TStore>, It<_>, crate::types::TIdN<_>>(&stores, code);
    dbg!();
    let mut count = 0;
    for m in matchs {
        count += 1;
        for c in &m.1 .0 {
            dbg!(&matcher.captures[c.id as usize]);
        }
        dbg!(m);
    }
    count
}

fn prep_recursive<'store>(
    query: &str,
    text: &[u8],
) -> (
    hyper_ast_gen_ts_tsquery::search::PreparedMatcher<crate::types::Type>,
    SimpleStores<crate::types::TStore>,
    NodeIdentifier,
) {
    use crate::legion_with_refs;
    let matcher = hyper_ast_gen_ts_tsquery::prepare_matcher::<crate::types::Type>(query);

    let mut stores = hyper_ast::store::SimpleStores {
        label_store: hyper_ast::store::labels::LabelStore::new(),
        type_store: crate::types::TStore::default(),
        node_store: hyper_ast::store::nodes::legion::NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = legion_with_refs::JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };

    let tree = match legion_with_refs::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text, tree.walk());
    eprintln!(
        "{}",
        hyper_ast::nodes::SyntaxSerializer::new(&stores, full_node.local.compressed_node)
    );
    (matcher, stores, full_node.local.compressed_node)
}
fn run_baseline(query: &str, text: &[u8]) -> usize {
    let mut cursor = tree_sitter::QueryCursor::default();
    let (query, tree) = prep_baseline(query, text);
    dbg!(&tree);
    dbg!(tree.root_node().to_sexp());
    let matches = cursor.matches(&query, tree.root_node(), text);
    let mut count = 0;
    for m in matches {
        count += 1;
        dbg!(&m);
        dbg!(m.pattern_index);
        for capt in m.captures {
            let index = capt.index;
            let name = query.capture_names()[index as usize];
            let _i = query.capture_index_for_name(name).unwrap();
            let n = capt.node;
            let k = n.kind();
            let r = n.byte_range();
            dbg!(name);
            dbg!(k);
            dbg!(r);
        }
    }
    count
}

fn prep_baseline<'query, 'tree>(
    query: &'query str,
    text: &'tree [u8],
) -> (tree_sitter::Query, tree_sitter::Tree) {
    let language = tree_sitter_java::language();

    let query = tree_sitter::Query::new(&language, query).unwrap();

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).unwrap();
    let tree = parser.parse(text, None).unwrap();

    (query, tree)
}

#[cfg(test)]
fn run_stepped(query: &str, text: &[u8]) -> usize {
    let (query, stores, code) = prep_stepped(query, text);
    let qcursor = query.matches(
        hyper_ast_gen_ts_tsquery::search::steped::hyperast::TreeCursor::new(
            &stores,
            hyper_ast::position::StructuralPosition::new(code),
        ),
    );

    let mut count = 0;
    for m in qcursor {
        count += 1;
        dbg!(m.pattern_index);
        dbg!(m.captures.len());
        for c in &m.captures {
            let i = c.index;
            dbg!(i);
            let name = query.capture_name(i);
            dbg!(name);
            use hyper_ast::position::TreePath;
            let n = c.node.pos.node().unwrap();
            let n = hyper_ast::nodes::SyntaxSerializer::new(c.node.stores, *n);
            dbg!(n.to_string());
        }
    }
    count
}

fn prep_stepped<'store>(
    query: &str,
    text: &[u8],
) -> (
    hyper_ast_gen_ts_tsquery::search::steped::Query,
    SimpleStores<crate::types::TStore>,
    NodeIdentifier,
) {
    use crate::legion_with_refs;
    use hyper_ast_gen_ts_tsquery::search::steped;
    let query = steped::Query::new(query, tree_sitter_java::language()).unwrap();

    let mut stores = hyper_ast::store::SimpleStores {
        label_store: hyper_ast::store::labels::LabelStore::new(),
        type_store: crate::types::TStore::default(),
        node_store: hyper_ast::store::nodes::legion::NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = legion_with_refs::JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };

    let tree = match legion_with_refs::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text, tree.walk());
    eprintln!(
        "{}",
        hyper_ast::nodes::SyntaxSerializer::new(&stores, full_node.local.compressed_node)
    );

    (query, stores, full_node.local.compressed_node)
}

#[cfg(test)]
fn run_stepped2(query: &str, text: &[u8]) -> usize {
    let (query, tree) = prep_stepped2(query, text);

    let qcursor = query.matches(tree.root_node().walk());

    let mut count = 0;
    for m in qcursor {
        count += 1;
        dbg!(m.pattern_index);
        dbg!(m.captures.len());
        for c in &m.captures {
            let i = c.index;
            dbg!(i);
            let name = query.capture_name(i);
            dbg!(name);
            let n = c.node;
            dbg!(n.utf8_text(text).unwrap());
        }
    }
    count
}

fn prep_stepped2<'store>(
    query: &str,
    text: &[u8],
) -> (
    hyper_ast_gen_ts_tsquery::search::steped::Query,
    tree_sitter::Tree,
) {
    use hyper_ast_gen_ts_tsquery::search::steped;
    let query = steped::Query::new(query, tree_sitter_java::language()).unwrap();

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_java::language()).unwrap();
    let tree = parser.parse(text, None).unwrap();

    (query, tree)
}

#[test]
fn all_run_recursive() {
    for (i, text) in CODES.iter().enumerate() {
        for (j, query) in QUERIES.iter().enumerate() {
            dbg!(i, j);
            run_recursive(query, text.as_bytes());
        }
    }
}

#[test]
fn all_run_baseline() {
    for (i, text) in CODES.iter().enumerate() {
        for (j, query) in QUERIES.iter().enumerate() {
            dbg!(i, j);
            run_baseline(query, text.as_bytes());
        }
    }
}

#[test]
fn compare_all_test() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
    let codes = CODES.iter().enumerate();
    // NOTE Uncomment and set the path to a directory containing java files you want to test querying on.
    // let codes = It::new(
    //     Path::new("../../../../stack-graphs/languages/tree-sitter-stack-graphs-java/test")
    //         .to_owned(),
    // )
    // .map(|x| {
    //     let text = std::fs::read_to_string(&x).expect("Find a dir containing java files");
    //     (x, text)
    // });

    compare_all(codes)
}

fn compare_all(
    codes: impl Iterator<Item = (impl std::fmt::Debug + Clone, impl AsRef<str>)>,
) {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let mut good = vec![];
    let mut bad = vec![];
    let mut codes_count = 0;
    let mut used = std::collections::HashSet::<usize>::new();
    for (i, text) in codes {
        codes_count += 1;
        for (j, query) in QUERIES.iter().enumerate() {
            dbg!(&i, &j);
            let text = text.as_ref().as_bytes();
            let mut cursor = tree_sitter::QueryCursor::default();
            let g_res = prep_baseline(query, text);
            let g_matches = { cursor.matches(&g_res.0, g_res.1.root_node(), text) };
            // let f_res = f_aux(query, text);
            // let f_matches = {
            //     type It<'a, HAST> =
            //         crate::iter::IterAll<'a, hyper_ast::position::StructuralPosition, HAST>;
            //     f_res.0
            //     .apply_matcher::<SimpleStores<crate::types::TStore>, It<_>, crate::types::TIdN<_>>(
            //         &f_res.1, f_res.2,
            //     )
            // };
            let h_res = prep_stepped(query, text);
            let h_matches = h_res.0.matches(
                hyper_ast_gen_ts_tsquery::search::steped::hyperast::TreeCursor::new(
                    &h_res.1,
                    hyper_ast::position::StructuralPosition::new(h_res.2),
                ),
            );
            let g_c = g_matches.into_iter().count();
            let f_c = 0;
            // let f_c = f_matches.into_iter().count();
            let h_c = h_matches.into_iter().count();
            if g_c > 0 {
                used.insert(j);
            }
            if g_c != 0 || f_c != 0 || h_c != 0 {
                // if g_c != f_c {
                //     bad.push(((i.clone(), j), (g_c, f_c)));
                //     dbg!(g_res.1.root_node().to_sexp());
                //     dbg!(g_c, f_c);
                // }
                if g_c != h_c {
                    bad.push(((i.clone(), j), (g_c, h_c)));
                    if g_c == f_c {
                        dbg!(g_res.1.root_node().to_sexp());
                    }
                    dbg!(g_c, h_c);
                }
                // g_c == f_c &&
                if g_c == h_c {
                    good.push(((i.clone(), j), g_c));
                }
            }
        }
    }
    println!("good:");
    for good in &good {
        println!("{:?}", good);
    }
    println!("bads:");
    for bad in &bad {
        println!("{:?}", bad);
    }
    eprintln!("bad    : {}", bad.len()); // should be zero
    eprintln!("good   : {}", good.len());
    eprintln!(
        "ratio  : {:.2}%",
        bad.len() as f64 / good.len() as f64 * 100.
    );
    let total = QUERIES.len() * codes_count;
    eprintln!("total  : {}", total);
    let active = good.len() + bad.len();
    eprintln!("activ  : {:.2}%", active as f64 / total as f64 * 100.); // should reach 0 for matching coverage
    eprintln!("queries: {}", QUERIES.len()); // should reach 0 for matching coverage
    eprintln!("used   : {}", used.len()); // should reach 0 for matching coverage
    eprintln!(
        "used%  : {:.2}%",
        used.len() as f64 / QUERIES.len() as f64 * 100.
    ); // should reach 0 for matching coverage
    assert_eq!(bad.len(), 0)
}

#[test]
fn sg_dataset() {
    let path = Path::new("../../../../stack-graphs/languages/tree-sitter-stack-graphs-java/test");
    dbg!(path.exists());

    let it = crate::tsg::It::new(path.to_owned());
    for p in it {
        dbg!(p);
    }
}

const A0: &str = r#"(program)@prog @__tsg__full_match"#;

#[test]
fn it_0() {
    let query = A0;
    let text = CODE.as_bytes();
    run_recursive(query, text);
}

#[test]
fn bl_0() {
    let query = A0;
    let text = CODE.as_bytes();
    run_baseline(query, text);
}

#[test]
fn st_0_h() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = A0;
    let text = CODE.as_bytes();
    assert_eq!(1, run_stepped(query, text));
}

const A1: &str = r#"(program (_)@declaration)@prog @__tsg__full_match"#;
#[test]
fn it_1() {
    let query = A1;
    let text = CODE.as_bytes();
    run_recursive(query, text);
    // TODO should match 2 times
    // Not sure how to handle that
    // Would be safer to add another code example
    // CODE1 matches 3 times with tsqueries.
}
#[test]
fn bl_1() {
    let query = A1;
    let text = CODE1.as_bytes();
    dbg!(run_baseline(query, text));
}
#[test]
fn it_1_h() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = A1;
    let text = CODE.as_bytes();
    run_recursive(query, text);
    // TODO should match 2 times
    // Not sure how to handle that
    // Would be safer to add another code example
    // CODE1 matches 3 times with tsqueries.
}

#[test]
fn st_1_h() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = A1;
    let text = CODE.as_bytes();
    assert_eq!(2, run_stepped(query, text));
}

#[test]
fn st_1_h2() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = A1;
    let text = CODE1.as_bytes();
    assert_eq!(3, run_stepped(query, text));
}

#[test]
fn st_1_h3() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = A1;
    let text = tsg_test("variable_declaration.java");
    let text = text.as_bytes();
    assert_eq!(2, run_stepped(query, text));
}

const A2: &str = r#"[
  (module_declaration)
  (package_declaration)
  (import_declaration)
] @decl
@__tsg__full_match"#;
#[test]
fn it_2() {
    let query = A2;
    let text = CODE.as_bytes();
    run_recursive(query, text);
}
#[test]
fn it_2_h() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = A2;
    let text = CODE.as_bytes();
    run_recursive(query, text);
}
const A3: &str = r#"(program
  (package_declaration
    (identifier)@pkg_name)? @package) @prog @__tsg__full_match"#;

#[test]
fn it_3() {
    let query = A3;
    let text = CODE.as_bytes();
    run_recursive(query, text);
}

#[test]
fn bl_3() {
    let query = A3;
    let text = CODE.as_bytes();
    run_baseline(query, text);
}
const A7: &str = r#"(scoped_absolute_identifier scope: (_) @scope name: (_) @name) @scoped_name @__tsg__full_match"#;
#[test]
fn it_7() {
    let query = A7;
    let text = CODE.as_bytes();
    run_recursive(query, text);
}
const A38: &str = r#"(element_value_pair value: (_) @value) @this @__tsg__full_match"#;
#[test]
fn st_38() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
    let query = A38;
    let text = tsg_test("decl/field_modifiers.java");
    let text = text.as_bytes();
    assert_eq!(1, run_stepped(query, text));
}
const A39: &str = r#"(field_declaration (modifiers) @modifiers) @decl @__tsg__full_match"#;
#[test]
fn it_39() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = A39;
    let text = CODE3.as_bytes();
    run_recursive(query, text);
}
const A45: &str = r#"(method_declaration
  (modifiers "static"?@is_static)?
  type: (_) @type
  name: (identifier) @name
  body: (block) @_block) @method
@__tsg__full_match"#;
#[test]
fn it_45() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = A45;
    let text = std::fs::read_to_string("../../../../stack-graphs/languages/tree-sitter-stack-graphs-java/test/decl/annotation_type_body.java").unwrap();
    let text = text.as_bytes();
    assert_eq!(1, run_recursive(query, text));
}
#[test]
fn bl_45() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = A45;
    let text = std::fs::read_to_string("../../../../stack-graphs/languages/tree-sitter-stack-graphs-java/test/decl/annotation_type_body.java").unwrap();
    let text = text.as_bytes();
    let c = run_baseline(query, text);
    dbg!(c);
}
const A53: &str = r#"[
  (assert_statement)
  (block)
  (break_statement)
  (continue_statement)
  (declaration)
  (do_statement)
  (expression_statement)
  (enhanced_for_statement)
  (for_statement)
  (if_statement)
  (labeled_statement)
  (local_variable_declaration)
  (return_statement)
  (switch_expression)
  (synchronized_statement)
  (throw_statement)
  (try_statement)
  (try_with_resources_statement)
  (while_statement)
  (yield_statement)
] @stmt
@__tsg__full_match"#;
#[test]
fn it_53() {
    let query = A53;
    let text = CODE.as_bytes();
    run_recursive(query, text);
    // TODO missing matches using supertypes
    // NOTE adding (class_declaration) and (package_declaration) to the list makes it match the nodes like tsqueries
}
#[test]
fn it_53_declaration() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = "(declaration)";
    let text = CODE.as_bytes();
    run_recursive(query, text);
    // TODO missing matches using supertypes
    // NOTE adding (class_declaration) and (package_declaration) to the list makes it match the nodes like tsqueries
}
#[test]
fn bl_53() {
    let query = A53;
    let text = CODE.as_bytes();
    run_baseline(query, text);
}
const A56: &str = r#"(block
  (_) @left
  .
  (_) @right
)
@__tsg__full_match"#;
#[test]
fn it_56() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = A56;
    let text = tsg_test("decl/annotation_type_body.java");
    let text = text.as_bytes();
    assert_eq!(1, run_recursive(query, text));
}
#[test]
fn bl_56() {
    let query = A56;
    let text = tsg_test("decl/annotation_type_body.java");
    let text = text.as_bytes();
    let c = run_baseline(query, text);
    dbg!(c);
}
const A57: &str = r#"(block
  .
  (_) @first) @block @__tsg__full_match"#;
#[test]
fn it_57() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = A57;
    let text = tsg_test("decl/annotation_type_body.java");
    let text = text.as_bytes();
    assert_eq!(1, run_recursive(query, text));
}
const A58: &str = r#"(block
  (_) @last
  . ) @block @__tsg__full_match"#;
#[test]
fn it_58() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = A58;
    let text = tsg_test("decl/type_identifier.java");
    let text = text.as_bytes();
    assert_eq!(1, run_recursive(query, text));
    let text = tsg_test("decl/record.java");
    let text = text.as_bytes();
    assert_eq!(1, run_recursive(query, text));
    let text = tsg_test("decl/collection_import.java");
    let text = text.as_bytes();
    assert_eq!(1, run_recursive(query, text));
    let text = tsg_test("decl/annotation_type_body.java");
    let text = text.as_bytes();
    assert_eq!(1, run_recursive(query, text));
}
#[test]
fn bl_58() {
    let query = A58;
    let text = tsg_test("decl/type_identifier.java");
    let text = text.as_bytes();
    let c = run_baseline(query, text);
    dbg!(c);
}
const A63: &str = r#"(declaration) @_decl @__tsg__full_match"#;

#[test]
fn st_63() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
    let query = A63;
    let text = tsg_test("variable_declaration.java");
    let text = text.as_bytes();
    assert_eq!(2, run_stepped(query, text));
}
const A68: &str =
    r#"(for_statement !init !condition !update body: (_) @body) @this @__tsg__full_match"#;
#[test]
fn st_68() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
    let query = A68;
    let text = tsg_test("statement/continue.java");
    let text = text.as_bytes();
    assert_eq!(2, run_stepped(query, text));
}
const A69: &str = r#"(for_statement init: (expression) @init condition: (_) @condition update: (_) @update body: (_) @body) @stmt @__tsg__full_match"#;
#[test]
fn st_69() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
    let query = A69;
    let text = tsg_test("statement/for_statement.java");
    let text = text.as_bytes();
    assert_eq!(1, run_stepped(query, text));
}
const A80: &str = r#"(variable_declarator
  name: (_) @name) @var_decl @__tsg__full_match"#;
#[test]
fn it_80_h() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = A80;
    let text = CODE3.as_bytes();
    run_recursive(query, text);
}
const A86: &str = r#"(switch_block (switch_block_statement_group (switch_label)+ . (statement) @first)) @this @__tsg__full_match"#;
#[test]
fn it_86_h() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
    let query = A86;
    let text = tsg_test("statement/switch_expression.java");
    let text = text.as_bytes();
    assert_eq!(1, run_stepped(query, text));
}

const A114: &str = r#"(primary_expression/identifier) @name
@__tsg__full_match"#;
#[test]
fn it_114_h() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = A114;
    let text = CODE.as_bytes();
    run_recursive(query, text);
}
