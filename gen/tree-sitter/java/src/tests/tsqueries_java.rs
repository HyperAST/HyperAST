use hyperast::store::{SimpleStores, defaults::NodeIdentifier};

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

#[test]
// provoke an infinite loop or is very slow
//
fn test_immediate_pred2() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = r#"(program
  (class_declaration
    name: (identifier) @name
    (class_body
      (method_declaration
        (modifiers
           "public"
           "static"
        )
        (void_type)
        (identifier) (#EQ? "main")
      )
    )
  )
)"#; // ;(#MATCH? "^test")
    let text = r#"
class A {
    public static void main() {}
}
    "#;
    // let text = std::fs::read_to_string("../../../../spoon/src/main/java/spoon/support/reflect/declaration/CtAnonymousExecutableImpl.java").unwrap();
    let text = text.as_bytes();
    // let (query, tree) = prep_stepped2(query, text);
    assert_eq!(1, run_stepped(query, text));
}

#[test]
fn test_return_null_with_prepro() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let query = r#"(return_statement (null_literal))"#;
    let prepro = [r#"(return_statement)"#, "(null_literal)"];
    let text = r#"
class A {
    public static void main() {
        return null;
    }
}
    "#;
    let text = text.as_bytes();
    assert_eq!(1, run_prepro(query, &prepro, text));
    // insta::assert_snapshot!(run_prepro(query, &prepro, text), @"1");
}

#[allow(unused)]
fn run_stepped2(query: &str, text: &[u8]) -> usize {
    let (query, tree) = prep_stepped2(query, text);
    let cursor = hyperast_tsquery::default_impls::TreeCursor::new(text, tree.root_node().walk());
    let qcursor = query.matches(cursor);

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

fn prep_stepped2<'store>(query: &str, text: &[u8]) -> (hyperast_tsquery::Query, tree_sitter::Tree) {
    let query = hyperast_tsquery::Query::new(query, crate::language()).unwrap();

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&crate::language()).unwrap();
    let tree = parser.parse(text, None).unwrap();

    (query, tree)
}

#[cfg(test)]
fn run_stepped(query: &str, text: &[u8]) -> usize {
    let (query, stores, code) = prep_stepped(query, text);
    let pos = hyperast::position::StructuralPosition::new(code);
    let cursor = hyperast_tsquery::hyperast_cursor::TreeCursor::new(&stores, pos);
    let matches = query.matches(cursor);

    let mut count = 0;
    for m in matches {
        count += 1;
        dbg!(m.pattern_index);
        dbg!(m.captures.len());
        for c in &m.captures {
            let i = c.index;
            dbg!(i);
            let name = query.capture_name(i);
            dbg!(name);
            use hyperast::position::TreePath;
            let n = c.node.pos.node().unwrap();
            let n = hyperast::nodes::SyntaxSerializer::new(c.node.stores, *n);
            dbg!(n.to_string());
        }
    }
    eprintln!(
        "{:?}",
        hyperast::nodes::SimpleSerializer::<_, _, true, true, true, true, true>::new(&stores, code)
            .to_string()
    );
    count
}

#[cfg(test)]
fn run_prepro(query: &str, subqueries: &[&str], text: &[u8]) -> usize {
    let (query, stores, code) = prep_prepro(query, subqueries, text);
    let pos = hyperast::position::StructuralPosition::new(code);
    let cursor = hyperast_tsquery::hyperast_cursor::TreeCursor::new(&stores, pos);
    let matches = query.matches(cursor);

    let mut count = 0;
    for m in matches {
        count += 1;
        dbg!(m.pattern_index);
        dbg!(m.captures.len());
        for c in &m.captures {
            let i = c.index;
            dbg!(i);
            let name = query.capture_name(i);
            dbg!(name);
            use hyperast::position::TreePath;
            let n = c.node.pos.node().unwrap();
            let n = hyperast::nodes::SyntaxSerializer::new(c.node.stores, *n);
            dbg!(n.to_string());
        }
    }
    eprintln!(
        "{:?}",
        hyperast::nodes::SimpleSerializer::<_, _, true, true, true, true, true>::new(&stores, code)
            .to_string()
    );
    count
}

fn prep_stepped<'store>(
    query: &str,
    text: &[u8],
) -> (
    hyperast_tsquery::Query,
    SimpleStores<crate::types::TStore>,
    NodeIdentifier,
) {
    use crate::legion_with_refs;
    let query = hyperast_tsquery::Query::new(query, crate::language()).unwrap();

    let mut stores = hyperast::store::SimpleStores::default();
    let mut md_cache = Default::default();
    let mut java_tree_gen = legion_with_refs::JavaTreeGen::new(&mut stores, &mut md_cache);

    let tree = match legion_with_refs::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text, tree.walk());
    eprintln!(
        "{}",
        hyperast::nodes::SyntaxSerializer::new(&stores, full_node.local.compressed_node)
    );

    (query, stores, full_node.local.compressed_node)
}

fn prep_prepro<'store>(
    query: &str,
    subqueries: &[&str],
    text: &[u8],
) -> (
    hyperast_tsquery::Query,
    SimpleStores<crate::types::TStore>,
    NodeIdentifier,
) {
    use crate::legion_with_refs;
    use crate::types::TStore;
    let (precomp, query) =
        hyperast_tsquery::Query::with_precomputed(query, crate::language(), subqueries).unwrap();
    assert_eq!(precomp.enabled_pattern_count(), subqueries.len());
    let mut stores = hyperast::store::SimpleStores::<TStore>::default();
    let mut md_cache = Default::default();
    let more = hyperast_tsquery::PreparedQuerying::<_, TStore, _>::from(&precomp);
    let mut java_tree_gen =
        legion_with_refs::JavaTreeGen::with_preprocessing(&mut stores, &mut md_cache, more);

    let tree = match legion_with_refs::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    let full_node = java_tree_gen.generate_file(b"", text, tree.walk());
    (query, stores, full_node.local.compressed_node)
}

#[cfg(feature = "tsg")]
mod test_tsg_queries {
    use super::*;
    use crate::tsg::{CODE, CODE1, CODE3, CODES, QUERIES};

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
        type It<'a, HAST> = JavaIter<'a, hyperast::position::StructuralPosition, HAST>;
        let matchs = matcher
            .apply_matcher::<SimpleStores<TStore>, It<_>, crate::types::TIdN<_>>(&stores, code);
        dbg!();
        let mut count = 0;
        for m in matchs {
            count += 1;
            for c in &m.1.0 {
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
        hyperast_gen_ts_tsquery::search::PreparedMatcher<crate::types::Type>,
        SimpleStores<crate::types::TStore>,
        NodeIdentifier,
    ) {
        use crate::legion_with_refs;
        let matcher = hyperast_gen_ts_tsquery::prepare_matcher::<crate::types::Type>(query);

        let mut stores = hyperast::store::SimpleStores::default();
        let mut md_cache = Default::default();
        let mut java_tree_gen = legion_with_refs::JavaTreeGen::new(&mut stores, &mut md_cache);

        let tree = match legion_with_refs::tree_sitter_parse(text) {
            Ok(t) => t,
            Err(t) => t,
        };
        println!("{}", tree.root_node().to_sexp());
        let full_node = java_tree_gen.generate_file(b"", text, tree.walk());
        eprintln!(
            "{}",
            hyperast::nodes::SyntaxSerializer::new(&stores, full_node.local.compressed_node)
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
                dbg!(name);
                if k == "modifiers" {
                    dbg!(n.utf8_text(text).unwrap());
                }
                let r = n.byte_range();
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
        let language = crate::language();

        let query = tree_sitter::Query::new(&language, query).unwrap();

        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).unwrap();
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

        compare_all(QUERIES, codes)
    }

    fn compare_all(
        queries: &[&str],
        codes: impl Iterator<Item = (impl std::fmt::Debug + Clone, impl AsRef<str>)>,
    ) {
        _compare_all(queries, codes)
    }

    fn _compare_all<'a>(
        queries: impl IntoIterator<Item = &'a &'a str> + Clone,
        codes: impl Iterator<Item = (impl std::fmt::Debug + Clone, impl AsRef<str>)>,
    ) {
        unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
        let mut good = vec![];
        let mut bad = vec![];
        let mut codes_count = 0;
        let mut used = std::collections::HashSet::<usize>::new();
        for (i, text) in codes {
            codes_count += 1;
            for (j, query) in queries.clone().into_iter().enumerate() {
                dbg!(&i, &j);
                let text = text.as_ref().as_bytes();
                let mut cursor = tree_sitter::QueryCursor::default();
                let g_res = prep_baseline(query, text);
                let g_matches = { cursor.matches(&g_res.0, g_res.1.root_node(), text) };
                // let f_res = f_aux(query, text);
                // let f_matches = {
                //     type It<'a, HAST> =
                //         crate::iter::IterAll<'a, hyperast::position::StructuralPosition, HAST>;
                //     f_res.0
                //     .apply_matcher::<SimpleStores<crate::types::TStore>, It<_>, crate::types::TIdN<_>>(
                //         &f_res.1, f_res.2,
                //     )
                // };
                let h_res = prep_stepped(query, text);
                let h_matches = h_res.0.matches(hyperast_tsquery::hyperast::TreeCursor::new(
                    &h_res.1,
                    hyperast::position::StructuralPosition::new(h_res.2),
                ));
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
    fn compare_prepro() {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Trace))
            .unwrap();
        let codes = ["class A {B f(){return null;}}"].iter().enumerate();
        // // NOTE Uncomment and set the path to a directory containing java files you want to test querying on.
        // let codes = crate::tsg::It::new(
        //     Path::new("../../../../stack-graphs/languages/tree-sitter-stack-graphs-java/test")
        //         .to_owned(),
        // )
        // .map(|x| {
        //     let text = std::fs::read_to_string(&x).expect("Find a dir containing java files");
        //     (x, text)
        // });
        let q = &[
            (
                r#"(return_statement (null_literal))"#,
                r#"(return_statement (null_literal))"#,
            ),
            (
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
            ),
        ];
        let _s: &[&[&str]] = &[];
        let s: &[&[&str]] = &[
            &[r#"(return_statement (null_literal))"#],
            &[r#"
        (method_declaration
          (modifiers
            "public"
            "static"
          )
          type: (void_type)
          name: (_) (#EQ? "main")
        )"#],
        ];

        _compare_all_prepro(q, s, codes)
        // _compare_all(q, codes)
    }

    fn _compare_all_prepro<'a>(
        queries: impl IntoIterator<Item = &'a (&'a str, &'a str)> + Clone,
        subqueries: impl IntoIterator<Item = &'a &'a [&'a str]> + Clone,
        codes: impl Iterator<Item = (impl std::fmt::Debug + Clone, impl AsRef<str>)>,
    ) {
        unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
        let mut good = vec![];
        let mut bad = vec![];
        let mut codes_count = 0;
        let mut used = std::collections::HashSet::<usize>::new();
        for (i, text) in codes {
            codes_count += 1;
            let mut subqueries = subqueries.clone().into_iter();
            for (j, query) in queries.clone().into_iter().enumerate() {
                dbg!(&i, &j);
                let text = text.as_ref().as_bytes();
                let mut cursor = tree_sitter::QueryCursor::default();
                let g_res = prep_baseline(query.1, text);
                let g_matches = { cursor.matches(&g_res.0, g_res.1.root_node(), text) };
                // let f_res = f_aux(query, text);
                // let f_matches = {
                //     type It<'a, HAST> =
                //         crate::iter::IterAll<'a, hyperast::position::StructuralPosition, HAST>;
                //     f_res.0
                //     .apply_matcher::<SimpleStores<crate::types::TStore>, It<_>, crate::types::TIdN<_>>(
                //         &f_res.1, f_res.2,
                //     )
                // };
                let h_res = prep_prepro(query.0, subqueries.next().unwrap(), text);
                let pos = hyperast::position::structural_pos::CursorWithPersistance::new(h_res.2);
                let tree_cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(&h_res.1, pos);
                let h_matches = h_res.0.matches(tree_cursor);
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
        let path =
            Path::new("../../../../stack-graphs/languages/tree-sitter-stack-graphs-java/test");
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
        (identifier)@pkg_name)?
        @package) @prog @__tsg__full_match"#;
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
    const A63: &str = r#"(d
    eclaration) @_decl @__tsg__full_match"#;
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

    #[test]
    fn st_issue_infinit() {
        // log::set_logger(&LOGGER)
        //     .map(|()| log::set_max_level(log::LevelFilter::Trace))
        //     .unwrap();
        let codes = CODES.iter().enumerate();
        // // NOTE Uncomment and set the path to a directory containing java files you want to test querying on.
        // let codes = crate::tsg::It::new(
        //     // Path::new("../../../../stack-graphs/languages/tree-sitter-stack-graphs-java/test")
        //     //     .to_owned(),
        //     Path::new("../../../../spoon").to_owned(),
        // )
        // .filter(|x| x.is_dir() || x.extension().map_or(false, |e| e.eq("java")))
        // .filter(|x| !x.starts_with("../../../../spoon/src/test/resources"))
        // .filter_map(|x| {
        //     let text = match std::fs::read_to_string(&x) {
        //         Ok(x) => x,
        //         Err(e) => {
        //             match e.kind() {
        //                 // std::io::ErrorKind::NotFound => todo!(),
        //                 std::io::ErrorKind::InvalidData => return None,
        //                 _ => panic!("{}", e),
        //             }
        //         }
        //     };
        //     Some((x, text))
        // });

        compare_all(&[crate::tsg::A155], codes)
    }

    #[test]
    // provoke an infinite loop or is very slow
    //
    fn st_155_spoon() {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Trace))
            .unwrap();
        unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
        let query = r#"
    (program
      (package_declaration (_)@pkg)
      (declaration
        .
        (modifiers "public")
        name: (_) @name
        body: (_
            [
              (method_declaration
                .
                (modifiers
                  . ; this is very important otherwise the complexity explodes
                  (marker_annotation
                    name: (_)@anot_name
                  )@mod +
                )
                name: (_)@meth_name
              )@meth
              (_)
            ]*
        )
      )
    )"#;
        let text = std::fs::read_to_string("../../../../spoon/src/main/java/spoon/support/reflect/declaration/CtAnonymousExecutableImpl.java").unwrap();
        let text = text.as_bytes();
        assert_eq!(1, run_stepped2(query, text));
    }

    #[test]
    // provoke an infinite loop or is very slow.
    // aparently just very slow ie. the baseline is as slow.
    // no idea how to fix that
    // NOTE immediate predicates would probably be beneficial there ie. shortcut
    // Might be an issue of type of used collection.
    fn bl_155_spoon() {
        unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
        let _query = crate::tsg::A155;
        let query = r#"
    (program
      (package_declaration (_)@pkg)
      (declaration
        (modifiers "public")
        name: (_) @name
        body: (_
            [
              (method_declaration
                (modifiers
                  . ; this is very important otherwise the complexity explodes, due to possible interleaved annotations
                  (marker_annotation
                    name: (_)@anot
                  )+
                )
                name: (_)@meth_name
              )
              (_)
            ]*
        )
      )
    )"#;
        let text = std::fs::read_to_string("../../../../spoon/src/main/java/spoon/support/reflect/declaration/CtAnonymousExecutableImpl.java").unwrap();
        let text = text.as_bytes();
        run_baseline(query, text);
    }

    #[test]
    // provoke an infinite loop or is very slow
    //
    fn test_immediate_pred() {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Trace))
            .unwrap();
        unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
        let query = r#"
    (program
        (declaration
          name: (_) @name
          body: (_
            [
            (method_declaration
                (modifiers
                  (marker_annotation
                    name: (_) (#EQ? "Override")
                  )
                )
                name: (_)@meth_name
            )
            (_)
            ]+
          )
        )
    )"#; // ;(#MATCH? "^test")
        let text = r#"
    class C {
        @Test
        @A
        @B
        @Override
        void t() {

        }
        @Test
        void t2() {

        }
        @AA
        @Test
        @Override
        void t3() {

        }
        @Override
        void f() {

        }
    }
        "#;
        // let text = std::fs::read_to_string("../../../../spoon/src/main/java/spoon/support/reflect/declaration/CtAnonymousExecutableImpl.java").unwrap();
        let text = text.as_bytes();
        // let (query, tree) = prep_stepped2(query, text);
        dbg!(run_stepped2(query, text));
    }

    #[test]
    fn test_precomputed() {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Trace))
            .unwrap();
        unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
        let _precomp = [
            //         //         r#"
            //         // (marker_annotation
            //         //     name: (_) (#EQ? "Override")
            //         // )"#,
            r#"
    (method_declaration
        (modifiers
            (marker_annotation
                name: (_) (#EQ? "Override")
            )
        )
    )"#,
            //         //         r#"
            //         // (method_declaration
            //         //     (modifiers
            //         //         (marker_annotation)
            //         //     )
            //         // )"#,
            //         r#"
            // (class_declaration
            //     name: (_) @name
            //     body: (_
            //         (method_declaration)
            //     )
            // )"#,
        ];
        let _query = r#"
    (program
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
    )"#;
        let precomp = [r#"(null_literal)"#];
        let query = r#"(return_statement (null_literal))"#;
        let _text = r#" class C {
        @Override
        void f() {

        }
        @Test
        @A
        @B
        @Override
        void t() {

        }
        @Test
        void t2() {

        }
        @AA
        @Test
        @Override
        void t3() {

        }
        @Override
        void g() {
            return;
            return;
            return;
            return;
            return;
        }
    }
        "#;
        let text = "class A {B f(){return null;}}";
        // let text = std::fs::read_to_string("../../../../spoon/src/main/java/spoon/support/reflect/declaration/CtAnonymousExecutableImpl.java").unwrap();
        let text = text.as_bytes();
        // let query =
        //     hyperast_tsquery::Query::with_precomputed(query, crate::language(), &precomp)
        //         .unwrap();
        use crate::legion_with_refs;
        let now = Instant::now();
        let (precomp, query) =
            hyperast_tsquery::Query::with_precomputed(query, crate::language(), &precomp).unwrap();
        let mut stores = hyperast::store::SimpleStores::<crate::types::TStore>::default();
        let mut md_cache = Default::default();
        let mut java_tree_gen = legion_with_refs::JavaTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: &mut stores,
            md_cache: &mut md_cache,
            more: precomp,
            // more: (),
        };
        let tree = match legion_with_refs::tree_sitter_parse(text) {
            Ok(t) => t,
            Err(t) => t,
        };
        log::trace!("sexp:\n{}", tree.root_node().to_sexp());
        let full_node = java_tree_gen.generate_file(b"", text, tree.walk());
        log::trace!(
            "syntax ser:\n{}",
            hyperast::nodes::SyntaxSerializer::new(&stores, full_node.local.compressed_node)
        );
        let pre_processing = now.elapsed();
        let now = Instant::now();
        let (query, stores, code) = (query, stores, full_node.local.compressed_node);
        let pos = hyperast::position::structural_pos::CursorWithPersistance::new(code);
        let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(&stores, pos);
        // let pos = hyperast::position::StructuralPosition::new(code);
        // let cursor = hyperast_tsquery::hyperast::TreeCursor::new(&stores, pos);
        let qcursor = query.matches(cursor);
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
                use hyperast::position::TreePath;
                use hyperast::position::structural_pos::AAA;
                let n = c.node.pos.node();
                let n = hyperast::nodes::SyntaxSerializer::new(c.node.stores, n);
                // let n = c.node.pos.node().unwrap();
                // let n = hyperast::nodes::SyntaxSerializer::new(c.node.stores, *n);
                dbg!(n.to_string());
            }
        }
        dbg!(count);
        let post_processing = now.elapsed();
        dbg!(pre_processing, post_processing);
    }

    #[test]
    fn test_precomputed2() {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Trace))
            .unwrap();
        unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
        let _precomp = [
            //         r#"
            // (marker_annotation
            //     name: (_) (#EQ? "Override")
            // )"#,
            r#"
    (method_declaration
        (modifiers
            (marker_annotation
                name: (_) (#EQ? "Override")
            )
        )
    )"#,
            //         r#"
            // (method_declaration
            //     (modifiers
            //         (marker_annotation)
            //     )
            // )"#,
            r#"
    (class_declaration
        name: (_)
        body: (_
            (method_declaration)
        )
    )"#,
        ];
        let _query = r#"
    (program
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
    )"#;
        let precomp = [r#"(return_statement (null_literal))"#];
        let query = r#"(return_statement (null_literal))"#;
        let _text = r#"
    class C {
        @Test
        @A
        @B
        @Override
        void t() {

        }
        @Test
        void t2() {

        }
        @AA
        @Test
        @Override
        void t3() {

        }
        @Override
        void f() {
            return null;
        }
    }
        "#;
        let _text = "class A {B f(){return null;}}";

        let text = std::fs::read_to_string("../../../../spoon/src/main/java/spoon/support/reflect/declaration/CtAnonymousExecutableImpl.java").unwrap();
        let text = text.as_bytes();
        // let query =
        //     hyperast_tsquery::Query::with_precomputed(query, crate::language(), &precomp)
        //         .unwrap();
        let (query, tree) = {
            let query: &str = query;
            let (_precomp, query) =
                hyperast_tsquery::Query::with_precomputed(query, crate::language(), &precomp)
                    .unwrap();

            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&crate::language()).unwrap();
            let tree = parser.parse(text, None).unwrap();

            (query, tree)
        };
        let cursor =
            hyperast_tsquery::default_impls::TreeCursor::new(text, tree.root_node().walk());
        let qcursor = query.matches(cursor);
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
        dbg!(count);
    }

    fn f(q: &str, p: &[&str], _f: &str) {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Trace))
            .unwrap();
        let (_precomp, query) =
            hyperast_tsquery::Query::with_precomputed(q, crate::language(), p).unwrap();
        query._check_preprocessed(0, p.len());
    }

    #[test]
    fn test_subquery_detection() {
        f(
            r#"(method_declaration
        type: (void_type)
        name: (_) (#EQ? "main")
    )"#,
            &[r#"(method_declaration
        name: (_) (#EQ? "main")
    )"#],
            "",
        );
    }

    #[test]
    fn test_subquery_detection2() {
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
      name: (identifier) (#EQ? "main")
    )"#,
        ];

        let text = std::fs::read_to_string("../../../../spoon/src/main/java/spoon/support/reflect/declaration/CtAnonymousExecutableImpl.java").unwrap();
        g(QUERY_MAIN_METH.0, &[QUERY_MAIN_METH_SUBS[2]], &text);
    }

    #[test]
    fn test_subquery_detection3() {
        f(
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
            &[r#"(_
        name: (_) (#EQ? "main")
    )"#],
            "",
        );
    }

    #[test]
    fn test_subquery_detection4() {
        f(
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
            &[r#"(method_declaration
        name: (_) (#EQ? "main")
    )"#],
            "",
        );
    }

    #[test]
    fn test_subquery_detection5() {
        f(
            r#"(try_statement
      (block
        (expression_statement)
        (expression_statement
          (method_invocation
            (identifier) (#EQ? "fail")
            (argument_list
            )
          )
        )
      )
      (catch_clause)
    )"#,
            &[
                r#"(method_invocation
                (identifier) (#EQ? "fail")
            )"#,
                r#"(try_statement
        (block)
        (catch_clause)
    )"#,
            ],
            "",
        );
    }

    #[test]
    fn test_prepro_main_meth() {
        let text = std::fs::read_to_string("../../../../spoon/src/main/java/spoon/support/reflect/declaration/CtAnonymousExecutableImpl.java").unwrap();
        g(
            r#"(class_declaration
            body: (_
                (method_declaration)
            )
        )"#,
            &[r#"(class_declaration
            body: (_
                (method_declaration)
            )
        )"#],
            &text,
        );
    }

    #[test]
    fn test_prepro_imp() {
        let text = r#"package com.google.gson;

    import static com.google.common.truth.Truth.assertThat;
    import static org.junit.Assert.fail;

    import com.google.gson.stream.JsonReader;
    import com.google.gson.stream.JsonWriter;
    import java.io.IOException;
    import java.io.StringReader;
    import org.junit.Test;

    public class TypeAdapterTest {
      @Test
      public void testNullSafe() throws IOException {
        TypeAdapter<String> adapter = new TypeAdapter<String>() {
          @Override public void write(JsonWriter out, String value) {
            throw new AssertionError("unexpected call");
          }

          @Override public String read(JsonReader in) {
            throw new AssertionError("unexpected call");
          }
        }.nullSafe();

        assertThat(adapter.toJson(null)).isEqualTo("null");
        assertThat(adapter.fromJson("null")).isNull();
      }

      /**
       * Tests behavior when {@link TypeAdapter#write(JsonWriter, Object)} manually throws
       * {@link IOException} which is not caused by writer usage.
       */
      @Test
      public void testToJson_ThrowingIOException() {
        final IOException exception = new IOException("test");
        TypeAdapter<Integer> adapter = new TypeAdapter<Integer>() {
          @Override public void write(JsonWriter out, Integer value) throws IOException {
            throw exception;
          }

          @Override public Integer read(JsonReader in) {
            throw new AssertionError("not needed by this test");
          }
        };

        try {
          adapter.toJson(1);
          fail();
        } catch (JsonIOException e) {
          assertThat(e.getCause()).isEqualTo(exception);
        }

        try {
          adapter.toJsonTree(1);
          fail();
        } catch (JsonIOException e) {
          assertThat(e.getCause()).isEqualTo(exception);
        }
      }

      private static final TypeAdapter<String> adapter = new TypeAdapter<String>() {
        @Override public void write(JsonWriter out, String value) throws IOException {
          out.value(value);
        }

        @Override public String read(JsonReader in) throws IOException {
          return in.nextString();
        }
      };

      // Note: This test just verifies the current behavior; it is a bit questionable
      // whether that behavior is actually desired
      @Test
      public void testFromJson_Reader_TrailingData() throws IOException {
        assertThat(adapter.fromJson(new StringReader("\"a\"1"))).isEqualTo("a");
      }

      // Note: This test just verifies the current behavior; it is a bit questionable
      // whether that behavior is actually desired
      @Test
      public void testFromJson_String_TrailingData() throws IOException {
        assertThat(adapter.fromJson("\"a\"1")).isEqualTo("a");
      }
    }"#;
        let q = r#"

    (import_declaration
        "static"
        (scoped_absolute_identifier
          (scoped_absolute_identifier
            (scoped_absolute_identifier
              (identifier) (#EQ? "org")
              (identifier) (#EQ? "junit")
            )
            (identifier) (#EQ? "Assert")
          )
          (identifier) (#EQ? "fail")
        )
    ) @a2

    (try_statement
        (block
          (expression_statement
            (method_invocation
              (field_access
                (identifier) (#EQ? "ToNumberPolicy")
                (identifier) (#EQ? "BIG_DECIMAL")
              )
              (identifier) (#EQ? "readNumber")
              (argument_list
                (method_invocation
                  (identifier) (#EQ? "fromString")
                  (argument_list
                    (string_literal)
                  )
                )
              )
            )
          )
          (expression_statement
            (method_invocation
              (identifier) (#EQ? "fail")
              (argument_list
              )
            )
          )
        )
        (catch_clause
          (catch_formal_parameter
            (catch_type
              (type_identifier)
            )
            (identifier) @p0
          )
          (block
            (expression_statement
              (method_invocation
                (method_invocation
                  (method_invocation
                    (identifier) (#EQ? "assertThat")
                    (argument_list
                      (identifier) @p1
                    )
                  )
                  (identifier) (#EQ? "hasMessageThat")
                  (argument_list
                  )
                )
                (identifier) (#EQ? "isEqualTo")
                (argument_list
                  (binary_expression
                    (string_literal)
                    "+"
                    (string_literal)
                  )
                )
              )
            )
          )
        )
    ) @a3 "#;
        let p = &[r#"(method_invocation
            (identifier) (#EQ? "fail")
        )"#];
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Trace))
            .unwrap();
        let (precomp, query) =
            hyperast_tsquery::Query::with_precomputed(q, crate::language(), p).unwrap();
        // query._check_preprocessed(0, 0);
        log::trace!("\n{}", query);
        let mut stores = hyperast::store::SimpleStores::<crate::types::TStore>::default();
        let mut md_cache = Default::default();
        let mut java_tree_gen = crate::legion_with_refs::JavaTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: &mut stores,
            md_cache: &mut md_cache,
            more: precomp,
        };
        let tree = match crate::legion_with_refs::tree_sitter_parse((&text).as_bytes()) {
            Ok(t) => t,
            Err(t) => t,
        };
        println!("{}", tree.root_node().to_sexp());
        todo!("handle type inconsistences")
        // let full_node = java_tree_gen.generate_file(b"", (&text).as_bytes(), tree.walk());
        // let code = full_node.local.compressed_node;
        // let pos = hyperast::position::structural_pos::CursorWithPersistance::new(code);
        // let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(&stores, pos);
        // dbg!();
        // let qcursor = query.matches(cursor);
        // let mut count = 0;
        // for m in qcursor {
        //     count += 1;
        //     dbg!(m.pattern_index);
        //     dbg!(m.captures.len());
        //     for c in &m.captures {
        //         let i = c.index;
        //         dbg!(i);
        //         let name = query.capture_name(i);
        //         dbg!(name);
        //         use hyperast::position::structural_pos::AAA;
        //         let n = c.node.pos.node();
        //         let n = hyperast::nodes::SyntaxSerializer::new(c.node.stores, n);
        //         dbg!(n.to_string());
        //     }
        // }
        // assert_eq!(1, count);
    }
    #[test]
    fn test_prepro_main_meth2() {
        let text =
            std::fs::read_to_string("../../../../spoon/src/main/java/spoon/Launcher.java").unwrap();

        // let text = r#"class A {
        //     /**
        //      */
        //     public static void main(String[] args) {
        //             new Launcher().run(args);
        //     }
        // }
        // "#;
        g(
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
            &[r#"(_
                name: (_) (#EQ? "main")
            )"#],
            &text,
        );
    }
    #[test]
    fn test_prepro_try_catch_in_test() {
        let text = r#"import static org.junit.Assert.fail;
    class A {
      @Test
      public void testDuplicateLabel() {
        RuntimeTypeAdapterFactory<BillingInstrument> rta =
            RuntimeTypeAdapterFactory.of(BillingInstrument.class);
        rta.registerSubtype(CreditCard.class, "CC");
        try {
          rta.registerSubtype(BankTransfer.class, "CC");
          fail();
        } catch (IllegalArgumentException expected) {
        }
      }
    }
    "#;
        g(
            r#"(program
      (import_declaration
        "static"
        (scoped_absolute_identifier
            scope: (scoped_absolute_identifier
                scope: (scoped_absolute_identifier
                    scope: (identifier) (#EQ? "org")
                    name: (identifier) (#EQ? "junit")
                )
                name: (identifier) (#EQ? "Assert")
            )
            name: (identifier) (#EQ? "fail")
        )
      )
      (class_declaration
        name: (_) @name
        body: (_
          (method_declaration
            (modifiers
              (marker_annotation
                name: (_) (#EQ? "Test")
              )
            )
            name: (_) @meth
            body: (_
                (try_statement
                    body: (_
                        (expression_statement
                            (method_invocation
                                !object
                                name: (identifier) (#EQ? "fail")
                            )
                        )
                    )
                )
            )
          )
        )
      )
    )"#,
            &[
                r#"(method_invocation
        !object
        name: (identifier) (#EQ? "fail")
    )"#,
                r#"
    (import_declaration
      "static"
      (scoped_absolute_identifier
          scope: (scoped_absolute_identifier
              scope: (scoped_absolute_identifier
                  scope: (identifier) (#EQ? "org")
                  name: (identifier) (#EQ? "junit")
              )
              name: (identifier) (#EQ? "Assert")
          )
          name: (identifier) (#EQ? "fail")
      )
    )"#,
            ],
            &text,
        );
    }

    #[test]
    fn test_prepro_empty_block() {
        let text = r#"import static org.junit.Assert.fail;
    class A {
      @Test
      public void testDuplicateLabel() {
        RuntimeTypeAdapterFactory<BillingInstrument> rta =
            RuntimeTypeAdapterFactory.of(BillingInstrument.class);
        rta.registerSubtype(CreditCard.class, "CC");
        try {
          rta.registerSubtype(BankTransfer.class, "CC");
          fail();
        } catch (IllegalArgumentException expected) {
        }
      }
    }
    "#;
        let c = g(
            r#"(program
      (import_declaration
        "static"
        (scoped_absolute_identifier
            scope: (scoped_absolute_identifier
                scope: (scoped_absolute_identifier
                    scope: (identifier) (#EQ? "org")
                    name: (identifier) (#EQ? "junit")
                )
                name: (identifier) (#EQ? "Assert")
            )
            name: (identifier) (#EQ? "fail")
        )
      )
      (class_declaration
        name: (_) @name
        body: (_
          (method_declaration
            (modifiers
              (marker_annotation
                name: (_) (#EQ? "Test")
              )
            )
            name: (_) @meth
            body: (_
                (try_statement
                    body: (_
                        (expression_statement
                            (method_invocation
                                !object
                                name: (identifier) (#EQ? "fail")
                            )
                        )
                    )
                    (catch_clause (block "{" . "}" ))
                )
            )
          )
        )
      )
    )"#,
            &[
                r#"(method_invocation
        !object
        name: (identifier) (#EQ? "fail")
    )"#,
                r#"
    (import_declaration
      "static"
      (scoped_absolute_identifier
          scope: (scoped_absolute_identifier
              scope: (scoped_absolute_identifier
                  scope: (identifier) (#EQ? "org")
                  name: (identifier) (#EQ? "junit")
              )
              name: (identifier) (#EQ? "Assert")
          )
          name: (identifier) (#EQ? "fail")
      )
    )"#,
            ],
            &text,
        );
        assert_eq!(c, 0);
    }

    #[test]
    fn test_neg() {
        let _text = r#"
    class A {
      @Test
      public void f() {
        try {
        } catch (E e) {
        }
      }
    }
    "#;
        let text2 = r#"
    class A {
      @Test
      public void f() {
        try {
        } catch (E e) {
            a;
        }
      }
    }
    "#;
        let q = r#"(program
          (class_declaration
        body: (_
          (method_declaration
            body: (_
                (try_statement
                    (catch_clause (block (_) (#NEG?) ))
                )
            )
          )
        )
          )
        )"#;
        let _q2 = r#"(program
          (class_declaration
        body: (_
          (method_declaration
            body: (_
                (try_statement
                    (catch_clause (block) (#MTY?))
                )
            )
          )
        )
          )
        )"#;
        let _q3 = r#"(program
          (class_declaration
        body: (_
          (method_declaration
            body: (_
                (try_statement
                    (catch_clause (block (_)? @a) (#empty? @a))
                )
            )
          )
        )
          )
        )"#;
        // let c = g(q, &[], &text);
        // assert_eq!(c, 1);
        let c = g(q, &[], &text2);
        assert_eq!(c, 0);
        // (block (_) (#CONT? 1))
    }

    #[test]
    fn test_prepro_ret_null_x() {
        let text = std::fs::read_to_string(
            "../../../../spoon/src/test/java/spoon/test/ctType/testclasses/X.java",
        )
        .unwrap();

        // let text = r#"class A {
        //     /**
        //      */
        //     public static void main(String[] args) {
        //             new Launcher().run(args);
        //     }
        // }
        // "#;
        g(
            r#"(return_statement (null_literal))"#,
            &[r#"(return_statement (null_literal))"#],
            &text,
        );
    }

    /// upstream issue in tree-sitter
    #[test]
    fn test_prepro_firsts() {
        let text = r#"class A {
            void main(String[] args) {}
        }
        "#;
        let count = g(r#"(_ . (identifier) @name)"#, &[], &text);
        // let count = g(r#"(_ . (_) @name)"#, &[], &text); // this one works properly
        assert_eq!(count, 1)
    }

    #[test]
    fn test_prepro_try_catch_pat() {
        let text = r#"/*
        * Copyright (C) 2021 Google Inc.
        *
        * Licensed under the Apache License, Version 2.0 (the "License");
        * you may not use this file except in compliance with the License.
        * You may obtain a copy of the License at
        *
        * http://www.apache.org/licenses/LICENSE-2.0
        *
        * Unless required by applicable law or agreed to in writing, software
        * distributed under the License is distributed on an "AS IS" BASIS,
        * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
        * See the License for the specific language governing permissions and
        * limitations under the License.
        */

       package com.google.gson;

       import static com.google.common.truth.Truth.assertThat;
       import static org.junit.Assert.fail;

       import com.google.gson.internal.LazilyParsedNumber;
       import com.google.gson.stream.JsonReader;
       import com.google.gson.stream.MalformedJsonException;
       import java.io.IOException;
       import java.io.StringReader;
       import java.math.BigDecimal;
       import org.junit.Test;

       public class ToNumberPolicyTest {
         @Test
         public void testDouble() throws IOException {
           ToNumberStrategy strategy = ToNumberPolicy.DOUBLE;
           assertThat(strategy.readNumber(fromString("10.1"))).isEqualTo(10.1);
           assertThat(strategy.readNumber(fromString("3.141592653589793238462643383279"))).isEqualTo(3.141592653589793D);
           try {
             strategy.readNumber(fromString("1e400"));
             fail();
           } catch (MalformedJsonException expected) {
             assertThat(expected).hasMessageThat().isEqualTo(
                 "JSON forbids NaN and infinities: Infinity at line 1 column 6 path $"
                 + "\nSee https://github.com/google/gson/blob/main/Troubleshooting.md#malformed-json");
           }
           try {
             strategy.readNumber(fromString("\"not-a-number\""));
             fail();
           } catch (NumberFormatException expected) {
           }
         }

         @Test
         public void testLazilyParsedNumber() throws IOException {
           ToNumberStrategy strategy = ToNumberPolicy.LAZILY_PARSED_NUMBER;
           assertThat(strategy.readNumber(fromString("10.1"))).isEqualTo(new LazilyParsedNumber("10.1"));
           assertThat(strategy.readNumber(fromString("3.141592653589793238462643383279"))).isEqualTo(new LazilyParsedNumber("3.141592653589793238462643383279"));
           assertThat(strategy.readNumber(fromString("1e400"))).isEqualTo(new LazilyParsedNumber("1e400"));
         }

         @Test
         public void testLongOrDouble() throws IOException {
           ToNumberStrategy strategy = ToNumberPolicy.LONG_OR_DOUBLE;
           assertThat(strategy.readNumber(fromString("10"))).isEqualTo(10L);
           assertThat(strategy.readNumber(fromString("10.1"))).isEqualTo(10.1);
           assertThat(strategy.readNumber(fromString("3.141592653589793238462643383279"))).isEqualTo(3.141592653589793D);
           try {
             strategy.readNumber(fromString("1e400"));
             fail();
           } catch (MalformedJsonException expected) {
             assertThat(expected).hasMessageThat().isEqualTo("JSON forbids NaN and infinities: Infinity; at path $");
           }
           try {
             strategy.readNumber(fromString("\"not-a-number\""));
             fail();
           } catch (JsonParseException expected) {
             assertThat(expected).hasMessageThat().isEqualTo("Cannot parse not-a-number; at path $");
           }

           assertThat(strategy.readNumber(fromStringLenient("NaN"))).isEqualTo(Double.NaN);
           assertThat(strategy.readNumber(fromStringLenient("Infinity"))).isEqualTo(Double.POSITIVE_INFINITY);
           assertThat(strategy.readNumber(fromStringLenient("-Infinity"))).isEqualTo(Double.NEGATIVE_INFINITY);
           try {
             strategy.readNumber(fromString("NaN"));
             fail();
           } catch (MalformedJsonException expected) {
             assertThat(expected).hasMessageThat().isEqualTo(
                 "Use JsonReader.setLenient(true) to accept malformed JSON at line 1 column 1 path $"
                 + "\nSee https://github.com/google/gson/blob/main/Troubleshooting.md#malformed-json");
           }
           try {
             strategy.readNumber(fromString("Infinity"));
             fail();
           } catch (MalformedJsonException expected) {
             assertThat(expected).hasMessageThat().isEqualTo(
                 "Use JsonReader.setLenient(true) to accept malformed JSON at line 1 column 1 path $"
                 + "\nSee https://github.com/google/gson/blob/main/Troubleshooting.md#malformed-json");
           }
           try {
             strategy.readNumber(fromString("-Infinity"));
             fail();
           } catch (MalformedJsonException expected) {
             assertThat(expected).hasMessageThat().isEqualTo(
                 "Use JsonReader.setLenient(true) to accept malformed JSON at line 1 column 1 path $"
                 + "\nSee https://github.com/google/gson/blob/main/Troubleshooting.md#malformed-json");
           }
         }

         @Test
         public void testBigDecimal() throws IOException {
           ToNumberStrategy strategy = ToNumberPolicy.BIG_DECIMAL;
           assertThat(strategy.readNumber(fromString("10.1"))).isEqualTo(new BigDecimal("10.1"));
           assertThat(strategy.readNumber(fromString("3.141592653589793238462643383279"))).isEqualTo(new BigDecimal("3.141592653589793238462643383279"));
           assertThat(strategy.readNumber(fromString("1e400"))).isEqualTo(new BigDecimal("1e400"));

           try {
             strategy.readNumber(fromString("\"not-a-number\""));
             fail();
           } catch (JsonParseException expected) {
             assertThat(expected).hasMessageThat().isEqualTo("Cannot parse not-a-number; at path $");
           }
         }

         @Test
         public void testNullsAreNeverExpected() throws IOException {
           try {
             ToNumberPolicy.DOUBLE.readNumber(fromString("null"));
             fail();
           } catch (IllegalStateException expected) {
             assertThat(expected).hasMessageThat().isEqualTo("Expected a double but was NULL at line 1 column 5 path $"
                 + "\nSee https://github.com/google/gson/blob/main/Troubleshooting.md#adapter-not-null-safe");
           }
           try {
             ToNumberPolicy.LAZILY_PARSED_NUMBER.readNumber(fromString("null"));
             fail();
           } catch (IllegalStateException expected) {
             assertThat(expected).hasMessageThat().isEqualTo("Expected a string but was NULL at line 1 column 5 path $"
                 + "\nSee https://github.com/google/gson/blob/main/Troubleshooting.md#adapter-not-null-safe");
           }
           try {
             ToNumberPolicy.LONG_OR_DOUBLE.readNumber(fromString("null"));
             fail();
           } catch (IllegalStateException expected) {
             assertThat(expected).hasMessageThat().isEqualTo("Expected a string but was NULL at line 1 column 5 path $"
                 + "\nSee https://github.com/google/gson/blob/main/Troubleshooting.md#adapter-not-null-safe");
           }
           try {
             ToNumberPolicy.BIG_DECIMAL.readNumber(fromString("null"));
             fail();
           } catch (IllegalStateException expected) {
             assertThat(expected).hasMessageThat().isEqualTo("Expected a string but was NULL at line 1 column 5 path $"
                 + "\nSee https://github.com/google/gson/blob/main/Troubleshooting.md#adapter-not-null-safe");
           }
         }

         private static JsonReader fromString(String json) {
           return new JsonReader(new StringReader(json));
         }

         private static JsonReader fromStringLenient(String json) {
           JsonReader jsonReader = fromString(json);
           jsonReader.setLenient(true);
           return jsonReader;
         }
       }
       "#;
        let count = g(
            r#"(try_statement
            (block
              (expression_statement
                (method_invocation
                  (identifier) (#EQ? "strategy")
                  (identifier) (#EQ? "readNumber")
                  (argument_list
                    (method_invocation
                      (identifier) (#EQ? "fromString")
                      (argument_list
                        (string_literal)
                      )
                    )
                  )
                )
              )
              (expression_statement
                (method_invocation
                  (identifier) (#EQ? "fail")
                  (argument_list
                  )
                )
              )
            )
            (catch_clause
              (catch_formal_parameter
                (catch_type
                  (type_identifier) (#EQ? "MalformedJsonException")
                )
                (identifier) @p0
              )
              (block
                (expression_statement
                  (method_invocation
                    (method_invocation
                      (method_invocation
                        (identifier) (#EQ? "assertThat")
                        (argument_list
                          (identifier) @p1
                        )
                      )
                      (identifier) (#EQ? "hasMessageThat")
                      (argument_list
                      )
                    )
                    (identifier) (#EQ? "isEqualTo")
                    (argument_list
                      (string_literal)
                    )
                  )
                )
              )
            )
          )"#,
            &[],
            &text,
        );
        // let count = g(r#"(_ . (_) @name)"#, &[], &text); // this one works properly
        assert_eq!(count, 1)
    }

    fn g(q: &str, p: &[&str], text: &str) -> usize {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Trace))
            .unwrap();
        let (precomp, query) =
            hyperast_tsquery::Query::with_precomputed(q, crate::language(), p).unwrap();
        query._check_preprocessed(0, p.len());
        log::trace!("\n{}", query);
        let mut stores = hyperast::store::SimpleStores::<crate::types::TStore>::default();
        let mut md_cache = Default::default();
        let mut java_tree_gen = crate::legion_with_refs::JavaTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: &mut stores,
            md_cache: &mut md_cache,
            more: precomp,
        };
        let tree = match crate::legion_with_refs::tree_sitter_parse(text.as_bytes()) {
            Ok(t) => t,
            Err(t) => t,
        };
        println!("{}", tree.root_node().to_sexp());
        todo!("handle type inconsistences")
        // let full_node = java_tree_gen.generate_file(b"", text.as_bytes(), tree.walk());
        // let code = full_node.local.compressed_node;
        // // let n = hyperast::nodes::SyntaxWithFieldsSerializer::new(&stores, code);
        // // dbg!(n.to_string());
        // let pos = hyperast::position::structural_pos::CursorWithPersistance::new(code);
        // let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(&stores, pos);
        // dbg!();
        // let qcursor = query.matches(cursor);
        // let mut count = 0;
        // for m in qcursor {
        //     count += 1;
        //     dbg!(m.pattern_index);
        //     dbg!(m.captures.len());
        //     for c in &m.captures {
        //         let i = c.index;
        //         dbg!(i);
        //         let name = query.capture_name(i);
        //         dbg!(name);
        //         use hyperast::position::structural_pos::AAA;
        //         let n = c.node.pos.node();
        //         let n = hyperast::nodes::SyntaxSerializer::new(c.node.stores, n);
        //         dbg!(n.to_string());
        //     }
        // }
        // dbg!(count)
    }

    /// concat queries
    #[test]
    fn test_concat_queries() {
        let text = r#"class A {
            void main(String[] args) {}
        }
        "#;
        let count = h(
            &[
                r#"(_ . (identifier) @name)"#,
                r#"(return_statement (null_literal))"#,
            ],
            &text,
        );
        // let count = g(r#"(_ . (_) @name)"#, &[], &text); // this one works properly
        assert_eq!(count, 1)
    }

    fn h(q: &[&str], text: &str) -> usize {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Trace))
            .unwrap();
        let query = hyperast_tsquery::Query::big(q, crate::language()).unwrap();
        let tree = match crate::legion_with_refs::tree_sitter_parse(text.as_bytes()) {
            Ok(t) => t,
            Err(t) => t,
        };
        println!("{}", tree.root_node().to_sexp());
        dbg!();
        let cursor = hyperast_tsquery::default_impls::TreeCursor::new(
            text.as_bytes(),
            tree.root_node().walk(),
        );
        let qcursor = query.matches(cursor);
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
            }
        }
        dbg!(count)
    }

    fn f2(q: &str, p: &[&str]) -> hyperast_tsquery::Query {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Trace))
            .unwrap();
        let (_precomp, query) =
            hyperast_tsquery::Query::with_precomputed(q, crate::language(), p).unwrap();

        query
    }

    /// concat queries
    #[test]
    fn test_precomp_pos() {
        let query = f2(
            r#"(method_invocation
        (identifier) (#EQ? "fail")
    )
    (class_declaration
      body: (_
          (method_declaration
              (modifiers
                  (marker_annotation
                      name: (_) (#EQ? "Test")
                  )
              )
          )

      )
    )"#,
            &[
                // r#"(identifier) (#EQ? "Test")"#,
                r#"(method_invocation
        (identifier) (#EQ? "fail")
    )"#,
                r#"(try_statement
        (block)
        (catch_clause)
    )"#,
                r#"(class_declaration)"#,
                r#"(method_declaration)"#,
                r#"(marker_annotation
        name: (identifier) (#EQ? "Test")
    )"#,
            ],
        );
        query._check_preprocessed(0, 1);
        query._check_preprocessed(1, 3);
    }

    #[test]
    fn test_multiple_pred_subs() {
        let around = |s| {
            format!(
                r#"(if_statement
      consequence: (_
          (expression_statement
              (method_invocation
                  name: (identifier) (#EQ? "{s}")
              )
          )
      )
      !alternative
    ) @root"#
            )
        };
        let query = &format!(
            // "{}\n{}\n{}\n{}\n{}",
            // "{}\n{}",
            "{}",
            around("assertThat"),
            // around("assertEquals"),
            // around("assertSame"),
            // around("assertTrue"),
            // around("assertNull"),
        );
        dbg!(query);
        let query = f2(
            &query,
            &[
                r#"(method_invocation
                  name: (identifier) (#EQ? "assertThat"))"#,
                // "(if_statement)",
                // "(if_statement !alternative)",
                r#"(method_invocation
                  name: (identifier) (#EQ? "assertEquals"))"#,
            ],
        );
        query._check_preprocessed(0, 3);
        query._check_preprocessed(1, 3);
    }
}
