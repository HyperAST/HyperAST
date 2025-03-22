use hyperast_gen_ts_java::tsg::It;

pub const QUERIES: &[(&[&str], &str)] = &[
    // (
    //     &[
    //         //         r#"
    //         // (marker_annotation
    //         //     name: (_) (#EQ? "Override")
    //         // )"#,
    //         r#"
    // (method_declaration
    //     (modifiers
    //         (marker_annotation
    //             name: (_) (#EQ? "Override")
    //         )
    //     )
    // )"#,
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
    //     ],
    //     r#"(program
    // (class_declaration
    //   name: (_) @name
    //   body: (_
    //     (method_declaration
    //       (modifiers
    //         (marker_annotation
    //           name: (_) (#EQ? "Override")
    //         )
    //       )
    //       name: (_)@meth_name
    //     )
    //   )
    // )
    //   )"#,
    // ),
    (
        &[
            //         r#"
            // (marker_annotation
            //     name: (_) (#EQ? "Override")
            // )"#,
            r#"
(method_declaration
    (modifiers
      "public"
      "static"
    )
    type: (void_type)
    name: (_) (#EQ? "main")
)"#,
            //         r#"
            // (method_declaration
            //     (modifiers
            //         (marker_annotation)
            //     )
            // )"#,
            //         r#"
            // (class_declaration
            //     name: (_) @name
            //     body: (_
            //         (method_declaration)
            //     )
            // )"#,
        ],
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
    ),
];

fn main() {
    use std::path::Path;
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
    let mut args = std::env::args();
    args.next().unwrap();
    let Some(codes) = args.next() else {
        let codes = hyperast_gen_ts_java::tsg::CODES.iter().enumerate();
        let queries: Vec<_> = QUERIES.iter().enumerate().collect();
        compare_all(codes, &queries);
        return;
    };
    // "../stack-graphs/languages/tree-sitter-stack-graphs-java/test"
    let codes = It::new(Path::new(&codes).to_owned()).map(|x| {
        let text = std::fs::read_to_string(&x).expect(&format!(
            "{:?} is not a java file or a dir containing java files: ",
            x
        ));
        (x, text)
    });
    let Some(queries) = args.next() else {
        let queries: Vec<_> = QUERIES.iter().enumerate().collect();
        compare_all(codes, &queries);
        return;
    };
    todo!()
    // let queries: Vec<_> = It::new(Path::new(&queries).to_owned())
    //     .map(|x| {
    //         let text = std::fs::read_to_string(&x).expect(&format!(
    //             "{:?} in not a file of treesitter queries of a dir containing such files",
    //             x
    //         ));
    //         (x, text)
    //     })
    //     .collect();
    // compare_all(codes, &queries);
}

fn compare_all(
    codes: impl Iterator<Item = (impl std::fmt::Debug + Clone, impl AsRef<str>)>,
    queries: &[(
        impl std::fmt::Debug + Clone + Eq + std::hash::Hash,
        &(&[&str], impl AsRef<str>),
    )],
) {
    unsafe { hyperast_gen_ts_java::legion_with_refs::HIDDEN_NODES = true };
    let mut good = vec![];
    let mut bad = vec![];
    let mut codes_count = 0;
    let mut used = std::collections::HashSet::<_>::new();
    for (i, text) in codes {
        codes_count += 1;
        for (j, query) in queries.iter() {
            let precomp = query.0;
            let query = query.1.as_ref();
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
            let h_res = prep_stepped(precomp, query, text);
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
    let total = queries.len() * codes_count;
    eprintln!("total  : {}", total);
    let active = good.len() + bad.len();
    eprintln!("activ  : {:.2}%", active as f64 / total as f64 * 100.); // should reach 0 for matching coverage
    eprintln!("queries: {}", queries.len()); // should reach 0 for matching coverage
    eprintln!("used   : {}", used.len()); // should reach 0 for matching coverage
    eprintln!(
        "used%  : {:.2}%",
        used.len() as f64 / queries.len() as f64 * 100.
    ); // should reach 0 for matching coverage
    assert_eq!(bad.len(), 0)
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

fn prep_stepped<'store>(
    precomp: &[&str],
    query: &str,
    text: &[u8],
) -> (
    hyperast_tsquery::Query,
    hyperast::store::SimpleStores<hyperast_gen_ts_java::types::TStore>,
    hyperast::store::defaults::NodeIdentifier,
) {
    use hyperast_gen_ts_java::legion_with_refs;
    let (precomp, query) =
        hyperast_tsquery::Query::with_precomputed(query, tree_sitter_java::language(), precomp)
            .unwrap();
    let more =
        hyperast_tsquery::PreparedQuerying::<_, hyperast_gen_ts_java::types::TStore, _>::from(
            &precomp,
        );

    let mut stores =
        hyperast::store::SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default();
    let mut java_tree_gen = {
        JavaTreeGen::with_preprocessing(&mut stores, &mut md_cache, precomp)
        // legion_with_refs::JavaTreeGen {
        //     line_break: "\n".as_bytes().to_vec(),
        //     stores: &mut stores,
        //     md_cache: &mut md_cache,
        //     more: precomp,
        // }
    };

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
