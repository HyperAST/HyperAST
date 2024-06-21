use hyper_ast::store::SimpleStores;
use tree_sitter_graph::GenQuery;

use crate::legion_with_refs;
use crate::tsg::{configure, init_globals, Functions, ROOT_NODE_VAR};

const CODE0: &str = include_str!("AAA.java");
const SOURCE0: &str = include_str!("java.tsg");

#[test]
fn tsg_vanilla() {
    let language = tree_sitter_java::language();

    let text = CODE0;
    // let text =
    //     &std::fs::read_to_string("/Users/quentin/spoon/src/main/java/spoon/MavenLauncher.java")
    //         .unwrap();
    let tsg_source = SOURCE0;

    let tsg = tree_sitter::Query::from_str(language.clone(), tsg_source).unwrap();
    dbg!(&tsg);

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).unwrap();
    let tree = parser.parse(text, None).unwrap();

    dbg!(&tree);
    dbg!(tree.root_node().to_sexp());

    let mut globals = tree_sitter_graph::Variables::new();
    let mut graph = tree_sitter_graph::graph::Graph::<tree_sitter_graph::MyTSNode<'_>>::new();
    let functions = Functions::stdlib();
    init_globals(&mut globals, &mut graph);
    let mut config = configure(&globals, &functions);
    let cancellation_flag = tree_sitter_graph::NoCancellation;
    tsg.execute_into(&mut graph, &tree, text, &mut config, &cancellation_flag)
        .unwrap();

    println!("{}", graph.pretty_print());
    assert_eq!(50, graph.node_count());
    let root = &graph[globals
        .get(&ROOT_NODE_VAR.into())
        .unwrap()
        .as_graph_node_ref()
        .unwrap()];
    assert_eq!(1, root.iter_edges().count());

    // let mut file = tree_sitter_graph::ast::File::<tree_sitter::Query>::new(language);
    // let mut aaa = tree_sitter_graph::parser::Parser::<ExtendingStringQuery>::new(SOURCE2);
    // aaa.parse_into_file(&mut file).unwrap();

    // for (i, s) in aaa.query_source.each.into_iter().enumerate() {
    //     println!("const A{}: &str r#\"{}\"#;", i, s);
    // }
}

#[test]
fn tsg_hyperast_stepped_query() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let language = tree_sitter_java::language();

    let text = CODE0;
    // NOTE you can use a real world java file
    // let text =
    //     &std::fs::read_to_string("/Users/quentin/spoon/src/main/java/spoon/MavenLauncher.java")
    //         .unwrap();
    let tsg_source = SOURCE0;
    // choose the stepped query implementation (like the treesitter one)
    use crate::tsg::stepped_query as impls;

    // parsing code into hyperast
    let mut stores = hyper_ast::store::SimpleStores {
        label_store: hyper_ast::store::labels::LabelStore::new(),
        type_store: crate::types::TStore::default(),
        node_store: hyper_ast::store::nodes::legion::NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = legion_with_refs::JavaTreeGen::new(&mut stores, &mut md_cache);
    let tree = match legion_with_refs::tree_sitter_parse(text.as_bytes()) {
        Ok(t) => t,
        Err(t) => t,
    };
    // println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text.as_bytes(), tree.walk());

    // parsing tsg query
    let tsg = impls::QueryMatcher::<SimpleStores<crate::types::TStore>>::from_str(
        language.clone(),
        tsg_source,
    )
    .unwrap();
    type Graph<'a> =
        tree_sitter_graph::graph::Graph<impls::Node<'a, SimpleStores<crate::types::TStore>>>;

    let mut globals = tree_sitter_graph::Variables::new();
    let mut graph = Graph::default();
    init_globals(&mut globals, &mut graph);
    let mut functions = Functions::stdlib();
    tree_sitter_stack_graphs::functions::add_path_functions(&mut functions);
    let mut config = configure(&globals, &functions);
    let cancellation_flag = tree_sitter_graph::NoCancellation;

    let pos = hyper_ast::position::StructuralPosition::new(full_node.local.compressed_node);
    let tree: impls::Node<_> = impls::Node::new(&stores, pos);

    // SAFETY: just circumventing a limitation in the borrow checker
    let tree = unsafe { std::mem::transmute(tree) };
    if let Err(err) = tsg.execute_lazy_into2(&mut graph, tree, &mut config, &cancellation_flag) {
        println!("{}", graph.pretty_print());
        let source_path = std::path::Path::new("./src/tests/AAA.java");
        let tsg_path = std::path::Path::new("./src/tests/java.tsg");
        eprintln!(
            "{}",
            err.display_pretty(&source_path, text, &tsg_path, tsg_source)
        );
    } else {
        println!("{}", graph.pretty_print());
    }
    assert_eq!(50, graph.node_count());
    let root = &graph[globals
        .get(&ROOT_NODE_VAR.into())
        .unwrap()
        .as_graph_node_ref()
        .unwrap()];
    assert_eq!(1, root.iter_edges().count());
}

#[test]
fn tsg_hyperast_recursive_query() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    let language = tree_sitter_java::language();

    let text = CODE0;
    // let text =
    //     &std::fs::read_to_string("/Users/quentin/spoon/src/main/java/spoon/MavenLauncher.java")
    //         .unwrap();
    let tsg_source = SOURCE0;
    // choose query implementation
    use crate::tsg::recursive_query as impls;

    // parsing code into hyperast
    let mut stores = hyper_ast::store::SimpleStores {
        label_store: hyper_ast::store::labels::LabelStore::new(),
        type_store: crate::types::TStore::default(),
        node_store: hyper_ast::store::nodes::legion::NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = legion_with_refs::JavaTreeGen::new(&mut stores, &mut md_cache);
    let tree = match legion_with_refs::tree_sitter_parse(text.as_bytes()) {
        Ok(t) => t,
        Err(t) => t,
    };
    // println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text.as_bytes(), tree.walk());

    // parsing tsg query
    let tsg = impls::QueryMatcher::from_str(language.clone(), tsg_source).unwrap();
    type Graph<'a> = tree_sitter_graph::graph::Graph<impls::Node<'a>>;

    let mut globals = tree_sitter_graph::Variables::new();
    let mut graph = Graph::default();
    init_globals(&mut globals, &mut graph);
    let mut functions = Functions::stdlib();
    tree_sitter_stack_graphs::functions::add_path_functions(&mut functions);
    let mut config = configure(&globals, &functions);
    let cancellation_flag = tree_sitter_graph::NoCancellation;

    let tree = impls::Node {
        stores: &stores,
        pos: hyper_ast::position::StructuralPosition::new(full_node.local.compressed_node),
    };

    if let Err(err) = tsg.execute_lazy_into2(&mut graph, tree, &mut config, &cancellation_flag) {
        println!("{}", graph.pretty_print());
        let source_path = std::path::Path::new("./src/tests/AAA.java");
        let tsg_path = std::path::Path::new("./src/tests/java.tsg");
        eprintln!(
            "{}",
            err.display_pretty(&source_path, text, &tsg_path, tsg_source)
        );
    } else {
        println!("{}", graph.pretty_print());
    }
    assert_eq!(50, graph.node_count());
}

const CODE: &str = r#"
package a.b;

class AAA {}

"#;

const SOURCE1: &str = r#"
(identifier) @name {
  node @name.ref
  attr (@name.ref) aa = @name
}
"#;

const SOURCE: &str = r#";;;;;;;;;;;;;;;;;;;
;; Global Variables

global ROOT_NODE

;;;;;;;;;;;;;;;;;;;;;;;
;; Attribute Shorthands

attribute node_definition = node        => type = "pop_symbol", node_symbol = node, is_definition
attribute node_reference = node         => type = "push_symbol", node_symbol = node, is_reference
attribute pop_node = node               => type = "pop_symbol", node_symbol = node
attribute pop_scoped_node = node        => type = "pop_scoped_symbol", node_symbol = node
attribute pop_scoped_symbol = symbol    => type = "pop_scoped_symbol", symbol = symbol
attribute pop_symbol = symbol           => type = "pop_symbol", symbol = symbol
attribute push_node = node              => type = "push_symbol", node_symbol = node
attribute push_scoped_node = node       => type = "push_scoped_symbol", node_symbol = node
attribute push_scoped_symbol = symbol   => type = "push_scoped_symbol", symbol = symbol
attribute push_symbol = symbol          => type = "push_symbol", symbol = symbol
attribute scoped_node_definition = node => type = "pop_scoped_symbol", node_symbol = node, is_definition
attribute scoped_node_reference = node  => type = "push_scoped_symbol", node_symbol = node, is_reference
attribute symbol_definition = symbol    => type = "pop_symbol", symbol = symbol, is_definition
attribute symbol_reference = symbol     => type = "push_symbol", symbol = symbol, is_reference

attribute node_symbol = node            => symbol = (source-text node), source_node = node


(program)@prog {
  node @prog.defs
  node @prog.lexical_scope
  edge @prog.lexical_scope -> ROOT_NODE
  edge @prog.lexical_scope -> @prog.defs
}

[
  (module_declaration)
  (package_declaration)
  (import_declaration)
] @decl
{
  node @decl.defs
  node @decl.lexical_scope
}

[
  (class_declaration)
  (enum_declaration)
  (field_declaration)
  (interface_declaration)
  (method_declaration)
  (constructor_declaration)
] @decl
{
  node @decl.defs
  node @decl.lexical_scope
  node @decl.static_defs
}



(program (_)@declaration)@prog {
  edge @prog.defs -> @declaration.defs
  edge @declaration.lexical_scope -> @prog.lexical_scope
}


(import_declaration (_) @ref) @import {
  edge @ref.lexical_scope -> @import.lexical_scope
}

;; X
(identifier) @name {
  node @name.lexical_scope
  node @name.type

  node @name.ref
  attr (@name.ref) node_reference = @name
  edge @name.ref -> @name.lexical_scope
}

"#;
