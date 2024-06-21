use std::fmt::Display;

use hyper_ast::store::SimpleStores;
use hyper_ast_gen_ts_java::{
    legion_with_refs,
    tsg::{configure, init_globals, Functions, It},
};

fn main() {
    use std::path::Path;
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
    let mut args = std::env::args();
    args.next().unwrap();
    let Some(codes) = args.next() else {
        let codes = hyper_ast_gen_ts_java::tsg::CODES.iter().enumerate();
        let queries: Vec<_> = [include_str!("../src/tests/java.tsg")]
            .iter()
            .enumerate()
            .collect();
        tsg_hyperast_stepped_loop(codes, &queries);
        return;
    };
    // "../stack-graphs/languages/tree-sitter-stack-graphs-java/test"
    let codes = hyper_ast_gen_ts_java::tsg::It::new(Path::new(&codes).to_owned()).map(|x| {
        let text = std::fs::read_to_string(&x).expect(&format!(
            "{:?} is not a java file or a dir containing java files: ",
            x
        ));
        (x.to_str().unwrap().to_string(), text)
    });
    let Some(queries) = args.next() else {
        let queries: Vec<_> = [include_str!("../src/tests/java.tsg")]
            .iter()
            .enumerate()
            .collect();
        tsg_hyperast_stepped_loop(codes, &queries);
        return;
    };
    let queries: Vec<_> = It::new(Path::new(&queries).to_owned())
        .map(|x| {
            let text = std::fs::read_to_string(&x).expect(&format!(
                "{:?} in not a file of treesitter queries of a dir containing such files",
                x
            ));
            (x.to_str().unwrap().to_string(), text)
        })
        .collect();
    tsg_hyperast_stepped_loop(codes, &queries);
}

fn tsg_hyperast_stepped_loop(
    codes: impl Iterator<Item = (impl Display, impl AsRef<str>)>,
    queries: &[(impl Display, impl AsRef<str>)],
) {
    let mut stores = hyper_ast::store::SimpleStores {
        label_store: hyper_ast::store::labels::LabelStore::new(),
        type_store: hyper_ast_gen_ts_java::types::TStore::default(),
        node_store: hyper_ast::store::nodes::legion::NodeStore::new(),
    };
    let mut md_cache = Default::default();
    for code in codes {
        for query in queries {
            tsg_hyperast_stepped(&code.0, code.1.as_ref(), &query, &mut stores, &mut md_cache);
        }
    }
}

fn tsg_hyperast_stepped<'a, 'c, 'd>(
    code_name: impl Display,
    code_text: &'a str,
    query: &(impl Display, impl AsRef<str>),
    stores: &'a mut SimpleStores<hyper_ast_gen_ts_java::types::TStore>,
    md_cache: &'c mut std::collections::HashMap<legion::Entity, legion_with_refs::MD>,
) -> tree_sitter_graph::graph::Graph<
    hyper_ast_gen_ts_java::tsg::stepped_query::Node<
        'a,
        SimpleStores<hyper_ast_gen_ts_java::types::TStore>,
    >,
> {
    unsafe { legion_with_refs::HIDDEN_NODES = true };
    let language = tree_sitter_java::language();

    let code_name = &code_name.to_string();
    let text = code_text;
    // NOTE you can use a real world java file
    // let text =
    //     &std::fs::read_to_string("/Users/quentin/spoon/src/main/java/spoon/MavenLauncher.java")
    //         .unwrap();
    let tsg_path = query.0.to_string();
    let tsg_source = query.1.as_ref();
    // choose the stepped query implementation (like the treesitter one)
    use hyper_ast_gen_ts_java::tsg::stepped_query as impls;

    let tree = match legion_with_refs::tree_sitter_parse(text.as_bytes()) {
        Ok(t) => t,
        Err(t) => t,
    };

    let mut java_tree_gen = legion_with_refs::JavaTreeGen::new(stores, md_cache);

    // println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(code_name.as_bytes(), text.as_bytes(), tree.walk());

    // parsing tsg query
    use tree_sitter_graph::GenQuery;
    let tsg = impls::QueryMatcher::from_str(language.clone(), tsg_source).unwrap();
    type Graph<'a> = tree_sitter_graph::graph::Graph<
        impls::Node<'a, SimpleStores<hyper_ast_gen_ts_java::types::TStore>>,
    >;

    let mut globals = tree_sitter_graph::Variables::new();
    let mut graph = Graph::default();
    init_globals(&mut globals, &mut graph);
    let mut functions = Functions::stdlib();
    tree_sitter_stack_graphs::functions::add_path_functions(&mut functions);
    let mut config = configure(&globals, &functions);
    let cancellation_flag = tree_sitter_graph::NoCancellation;

    let tree = impls::Node::new(
        &*java_tree_gen.stores,
        hyper_ast::position::StructuralPosition::new(full_node.local.compressed_node),
    );

    if let Err(err) = tsg.execute_lazy_into2(&mut graph, tree, &mut config, &cancellation_flag) {
        println!("{}", graph.pretty_print());
        let source_path = std::path::Path::new(&code_name);
        let tsg_path = std::path::Path::new(&tsg_path);
        eprintln!(
            "{}",
            err.display_pretty(&source_path, text, &tsg_path, tsg_source)
        );
    }
    graph
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
