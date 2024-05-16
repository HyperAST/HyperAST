use std::{fmt::Debug, hash::Hasher};

use hyper_ast::{
    position::{StructuralPosition, TreePath, TreePathMut},
    store::{defaults::NodeIdentifier, SimpleStores},
    types::HyperAST,
};
use tree_sitter_graph::GenQuery;
use tree_sitter_graph::{graph::GraphErazing, ExtendingStringQuery};

use crate::{legion_with_refs, types::TStore};

#[test]
fn tsg_vanilla() {
    static DEBUG_ATTR_PREFIX: &'static str = "debug_";
    static ROOT_NODE_VAR: &'static str = "ROOT_NODE";
    /// The name of the file path global variable
    pub const FILE_PATH_VAR: &str = "FILE_PATH";
    static JUMP_TO_SCOPE_NODE_VAR: &'static str = "JUMP_TO_SCOPE_NODE";
    let file_name = "AAA.java";
    let language = tree_sitter_java::language();

    let tsg = tree_sitter::Query::from_str(language.clone(), SOURCE2).unwrap();
    // tree_sitter_graph::ast::File::<tree_sitter::Query>;
    dbg!(&tsg);

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).unwrap();
    let tree = parser.parse(CODE, None).unwrap();

    dbg!(&tree);
    dbg!(tree.root_node().to_sexp());

    let mut globals = tree_sitter_graph::Variables::new();
    let mut graph = tree_sitter_graph::graph::Graph::<tree_sitter_graph::MyTSNode<'_>>::new();
    globals
        .add(
            ROOT_NODE_VAR.into(),
            tree_sitter_graph::graph::Value::GraphNode(graph.add_graph_node()),
        )
        .expect("Failed to set ROOT_NODE");
    globals
        .add(FILE_PATH_VAR.into(), file_name.into())
        .expect("Failed to set FILE_PATH");
    // let jump_to_scope_node = self.inject_node(stack_graphs::graph::NodeID::jump_to());
    let jump_to_scope_node = graph.add_graph_node();
    globals
        .add(JUMP_TO_SCOPE_NODE_VAR.into(), jump_to_scope_node.into())
        .expect("Failed to set JUMP_TO_SCOPE_NODE");

    let functions = tree_sitter_graph::functions::Functions::<
        GraphErazing<tree_sitter_graph::graph::TSNodeErazing>,
    >::stdlib();
    let mut config = tree_sitter_graph::ExecutionConfig::new(&functions, &globals)
        .lazy(true)
        .debug_attributes(
            [DEBUG_ATTR_PREFIX, "tsg_location"].concat().as_str().into(),
            [DEBUG_ATTR_PREFIX, "tsg_variable"].concat().as_str().into(),
            [DEBUG_ATTR_PREFIX, "tsg_match_node"]
                .concat()
                .as_str()
                .into(),
        );
    let cancellation_flag = tree_sitter_graph::NoCancellation;
    tsg.execute_into(&mut graph, &tree, CODE, &mut config, &cancellation_flag)
        .unwrap();

    println!("{}", graph.pretty_print());

    let mut file = tree_sitter_graph::ast::File::<tree_sitter::Query>::new(language);
    let mut aaa = tree_sitter_graph::parser::Parser::<ExtendingStringQuery>::new(SOURCE2);
    aaa.parse_into_file(&mut file).unwrap();

    for (i, s) in aaa.query_source.each.into_iter().enumerate() {
        println!("const A{}: &str r#\"{}\"#;", i, s);
    }
}

#[test]
fn tsg_hyperast() {
    unsafe { crate::legion_with_refs::HIDDEN_NODES = true };
    static DEBUG_ATTR_PREFIX: &'static str = "debug_";
    static ROOT_NODE_VAR: &'static str = "ROOT_NODE";
    /// The name of the file path global variable
    pub const FILE_PATH_VAR: &str = "FILE_PATH";
    static JUMP_TO_SCOPE_NODE_VAR: &'static str = "JUMP_TO_SCOPE_NODE";
    let file_name = "a/b/AAA.java";
    let language = tree_sitter_java::language();

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

    let text = CODE;
    let tree = match legion_with_refs::tree_sitter_parse(text.as_bytes()) {
        Ok(t) => t,
        Err(t) => t,
    };
    // println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text.as_bytes(), tree.walk());

    // dbg!(&full_node);
    let tsg_source = SOURCE2;
    let tsg = impls::PM::from_str(language.clone(), tsg_source).unwrap();
    dbg!(&tsg);

    type Functions = tree_sitter_graph::functions::Functions<GraphErazing<impls::MyNodeErazing>>;
    type Graph<'a> = tree_sitter_graph::graph::Graph<impls::MyNode<'a>>;

    let mut globals = tree_sitter_graph::Variables::new();
    let mut graph = Graph::default();
    globals
        .add(
            ROOT_NODE_VAR.into(),
            tree_sitter_graph::graph::Value::GraphNode(graph.add_graph_node()),
        )
        .expect("Failed to set ROOT_NODE");
    globals
        .add(FILE_PATH_VAR.into(), file_name.into())
        .expect("Failed to set FILE_PATH");
    // let jump_to_scope_node = self.inject_node(stack_graphs::graph::NodeID::jump_to());
    let jump_to_scope_node = graph.add_graph_node();
    globals
        .add(JUMP_TO_SCOPE_NODE_VAR.into(), jump_to_scope_node.into())
        .expect("Failed to set JUMP_TO_SCOPE_NODE");

    let mut functions = Functions::stdlib();
    tree_sitter_stack_graphs::functions::add_path_functions(&mut functions);
    let mut config = tree_sitter_graph::ExecutionConfig::new(&functions, &globals)
        .lazy(true)
        .debug_attributes(
            [DEBUG_ATTR_PREFIX, "tsg_location"].concat().as_str().into(),
            [DEBUG_ATTR_PREFIX, "tsg_variable"].concat().as_str().into(),
            [DEBUG_ATTR_PREFIX, "tsg_match_node"]
                .concat()
                .as_str()
                .into(),
        );
    let cancellation_flag = tree_sitter_graph::NoCancellation;

    use impls::MyNode;
    let tree: MyNode = MyNode {
        stores: &stores,
        pos: hyper_ast::position::StructuralPosition::new(full_node.local.compressed_node),
    };

    // SAFETY: should be sound, just circumventing borrow checker limitations
    // MSG: due to current limitations in the borrow checker, this implies a `'static` lifetime
    let tree: MyNode = unsafe { std::mem::transmute(tree) };

    if let Err(err) = tsg.execute_lazy_into2(&mut graph, tree, &mut config, &cancellation_flag) {
        println!("{}", graph.pretty_print());
        let source_path = std::path::Path::new("AAA.java");
        let tsg_path = std::path::Path::new("java.tsg");
        eprintln!(
            "{}",
            err.display_pretty(&source_path, text, &tsg_path, tsg_source)
        );
    } else {
        println!("{}", graph.pretty_print());
    }
}

mod impls {
    use super::*;

    pub struct MyNode<
        'hast,
        HAST = SimpleStores<crate::types::TStore>,
        P = hyper_ast::position::StructuralPosition,
    > {
        pub stores: &'hast HAST,
        pub pos: P,
    }

    impl<'tree, HAST, P: PartialEq> PartialEq for MyNode<'tree, HAST, P> {
        fn eq(&self, other: &Self) -> bool {
            self.pos == other.pos
        }
    }

    impl<'tree, HAST, P: Clone> Clone for MyNode<'tree, HAST, P> {
        fn clone(&self) -> Self {
            Self {
                stores: self.stores,
                pos: self.pos.clone(),
            }
        }
    }

    impl<'tree, HAST, P> tree_sitter_graph::graph::SyntaxNode for MyNode<'tree, HAST, P>
    where
        HAST::T: hyper_ast::types::WithSerialization,
        HAST: HyperAST<'tree, IdN = NodeIdentifier>,
        P: Clone + TreePath<HAST::IdN, HAST::Idx> + std::hash::Hash,
    {
        fn id(&self) -> usize {
            // let id = self.pos.node().unwrap(); // TODO make an associated type
            // let id: usize = unsafe { std::mem::transmute(id) };
            // id

            let mut hasher = std::hash::DefaultHasher::new();
            self.pos.hash(&mut hasher);
            hasher.finish() as usize
        }

        fn kind(&self) -> &'static str {
            let n = self.pos.node().unwrap();
            let n = self.stores.resolve_type(n);
            use hyper_ast::types::HyperType;
            n.as_static_str()
        }

        fn start_position(&self) -> tree_sitter::Point {
            // use hyper_ast::position::computing_offset_bottom_up::extract_position_it;
            // let p = extract_position_it(self.stores, self.pos.iter());
            tree_sitter::Point {
                row: 0, //p.range().start,
                column: 0,
            }
        }

        fn end_position(&self) -> tree_sitter::Point {
            todo!()
        }

        fn byte_range(&self) -> std::ops::Range<usize> {
            todo!()
        }

        fn range(&self) -> tree_sitter::Range {
            let r = self.byte_range();
            tree_sitter::Range {
                start_byte: r.start,
                end_byte: r.end,
                start_point: self.start_position(),
                end_point: self.end_position(),
            }
        }

        fn text(&self) -> String {
            hyper_ast::nodes::TextSerializer::new(self.stores, *self.pos.node().unwrap())
                .to_string()
        }

        fn named_child_count(&self) -> usize {
            todo!()
        }

        fn parent(&self) -> Option<Self>
        where
            Self: Sized,
        {
            todo!()
        }
    }

    impl<'tree, HAST, P> tree_sitter_graph::graph::SyntaxNodeExt for MyNode<'tree, HAST, P>
    where
        HAST::T: hyper_ast::types::WithSerialization,
        HAST: HyperAST<'tree, IdN = NodeIdentifier>,
        P: Clone + TreePathMut<HAST::IdN, HAST::Idx> + std::hash::Hash,
    {
        type Cursor = Vec<Self>;

        fn walk(&self) -> Self::Cursor {
            todo!()
        }

        fn named_children<'cursor>(
            &self,
            cursor: &'cursor mut Self::Cursor,
        ) -> impl ExactSizeIterator<Item = Self> + 'cursor
        where
            'tree: 'cursor,
        {
            todo!();
            cursor.iter().cloned()
        }

        type QM<'cursor> = MyQMatch<'cursor, 'tree, HAST, P>
    where
        Self: 'cursor;
    }

    pub struct MyNodeErazing;

    impl tree_sitter_graph::graph::Erzd for MyNodeErazing {
        type Original<'tree> = MyNode<'tree>;
    }

    impl<'tree> tree_sitter_graph::graph::LErazng for MyNode<'tree> {
        type LErazing = MyNodeErazing;
    }

    pub struct MyQMatch<'cursor, 'tree, HAST: HyperAST<'tree>, P> {
        // root: StructuralPosition<NodeIdentifier, u16>,
        root: P,
        stores: &'tree HAST,
        b: &'cursor (),
        captures: hyper_ast_gen_ts_tsquery::search::Captured<HAST::IdN, HAST::Idx>,
    }

    impl<'cursor, 'tree, HAST, P> tree_sitter_graph::graph::QMatch for MyQMatch<'cursor, 'tree, HAST, P>
    where
        HAST::IdN: Debug,
        HAST: HyperAST<'tree>,
        P: Clone + TreePathMut<HAST::IdN, HAST::Idx>,
    {
        type I = u32;

        type Item = MyNode<'tree, HAST, P>;

        fn nodes_for_capture_index(&self, index: Self::I) -> impl Iterator<Item = Self::Item> + '_ {
            dbg!(index);
            dbg!(&self.captures);
            self.captures.by_capture_id(index).into_iter().map(|c| {
                let mut p = self.root.clone();
                for i in c.path.iter().rev() {
                    use hyper_ast::types::NodeStore;
                    let nn = self.stores.node_store().resolve(p.node().unwrap());
                    use hyper_ast::types::WithChildren;
                    let node = nn.child(i).unwrap();
                    p.goto(node, *i);
                }
                assert_eq!(p.node(), Some(&c.match_node));
                MyNode {
                    stores: self.stores,
                    pos: p,
                }
            })
        }
        fn pattern_index(&self) -> usize {
            self.captures.pattern_index()
        }
    }

    pub struct MyQMatches<'query, 'cursor: 'query, 'tree: 'cursor> {
        q: &'query PM<crate::types::Type>,
        cursor: &'cursor mut Vec<u16>,
        matchs: hyper_ast_gen_ts_tsquery::IterMatched<
            &'query hyper_ast_gen_ts_tsquery::search::PreparedMatcher<crate::types::Type>,
            &'tree SimpleStores<TStore>,
            crate::iter::IterAll<
                'tree,
                StructuralPosition<NodeIdentifier, u16>,
                SimpleStores<TStore>,
            >,
            crate::types::TIdN<NodeIdentifier>,
        >,
        node: MyNode<
            'tree,
            SimpleStores<crate::types::TStore>,
            hyper_ast::position::StructuralPosition,
        >,
    }

    impl<'query, 'cursor: 'query, 'tree: 'cursor> Iterator for MyQMatches<'query, 'cursor, 'tree> {
        type Item = impls::MyQMatch<
            'cursor,
            'tree,
            SimpleStores<TStore>,
            StructuralPosition<NodeIdentifier, u16>,
        >;

        fn next(&mut self) -> Option<Self::Item> {
            let (root, captures) = self.matchs.next()?;
            let stores = self.matchs.hast;
            dbg!(&self.q.0.captures[..self.q.0.captures.len().min(10)]);
            for c in &captures.0 {
                dbg!(&self.q.0.captures[c.id as usize]);
            }
            Some(impls::MyQMatch {
                stores,
                b: &&(),
                captures,
                root,
            })
        }
    }

    pub struct PM<Ty>(pub hyper_ast_gen_ts_tsquery::search::PreparedMatcher<Ty>);

    impl<Ty: Debug> Debug for PM<Ty> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }

    impl GenQuery for PM<crate::types::Type> {
        type Lang = tree_sitter::Language;

        type Ext = ExtendingStringQuery<crate::types::Type, Self, Self::Lang>;

        fn pattern_count(&self) -> usize {
            self.0.pattern_count()
        }

        fn capture_index_for_name(&self, name: &str) -> Option<u32> {
            self.0.capture_index_for_name(name)
        }

        fn capture_quantifiers(
            &self,
            index: usize,
        ) -> impl std::ops::Index<usize, Output = tree_sitter::CaptureQuantifier> {
            self.0.capture_quantifiers(index)
        }

        fn capture_names(&self) -> &[&str] {
            todo!()
        }

        fn check(
            file: &mut tree_sitter_graph::ast::File<Self>,
        ) -> Result<(), tree_sitter_graph::checker::CheckError>
        where
            Self: Sized,
        {
            file.check2()
        }

        type Node<'tree> = MyNode<
            'tree,
            SimpleStores<crate::types::TStore>,
            hyper_ast::position::StructuralPosition,
        >;

        type Cursor = Vec<u16>;

        fn matches<'query, 'cursor: 'query, 'tree: 'cursor>(
            &'query self,
            cursor: &'cursor mut Self::Cursor,
            node: &Self::Node<'tree>,
        ) -> Self::Matches<'query, 'cursor, 'tree> {
            use crate::iter::IterAll as JavaIter;
            let matchs = self
                .0
                .apply_matcher::<SimpleStores<TStore>, JavaIter<hyper_ast::position::StructuralPosition, _>, crate::types::TIdN<_>>(
                    node.stores,
                    *node.pos.node().unwrap(),
                );
            // let a = matchs.next();
            let node = node.clone();
            impls::MyQMatches {
                q: self,
                cursor,
                matchs,
                node,
            }
        }

        type Match<'cursor, 'tree: 'cursor> = impls::MyQMatch<'cursor, 'tree, SimpleStores<TStore>, StructuralPosition<NodeIdentifier, u16>>
        where
            Self: 'cursor;

        type Matches<'query, 'cursor: 'query, 'tree: 'cursor> =
            impls::MyQMatches<'query, 'cursor, 'tree>
        where
            Self: 'tree,
            Self: 'query,
            Self: 'cursor;

        type I = u32;
    }

    pub struct ExtendingStringQuery<Ty, Q = tree_sitter::Query, L = tree_sitter::Language> {
        pub(crate) query: Option<Q>,
        pub(crate) acc: String,
        pub(crate) _phantom: std::marker::PhantomData<(Ty, L)>,
    }

    impl<Ty, Q, L> Default for ExtendingStringQuery<Ty, Q, L> {
        fn default() -> Self {
            Self {
                query: Default::default(),
                acc: Default::default(),
                _phantom: std::marker::PhantomData,
            }
        }
    }

    impl tree_sitter_graph::ExtendedableQuery
        for ExtendingStringQuery<crate::types::Type, PM<crate::types::Type>, tree_sitter::Language>
    {
        type Query = PM<crate::types::Type>;
        type Lang = tree_sitter::Language;

        fn as_ref(&self) -> Option<&Self::Query> {
            self.query.as_ref()
        }

        fn with_capacity(capacity: usize) -> Self {
            let acc = String::with_capacity(capacity);
            Self {
                acc,
                ..Default::default()
            }
        }

        fn make_query(
            &mut self,
            language: &Self::Lang,
            source: &str,
        ) -> Result<Self::Query, tree_sitter::QueryError> {
            self.acc += source;
            self.acc += "\n";
            dbg!(source);
            let matcher = hyper_ast_gen_ts_tsquery::prepare_matcher::<crate::types::Type>(source);
            Ok(PM(matcher))
        }

        fn make_main_query(&self, language: &Self::Lang) -> Self::Query {
            let matcher =
                hyper_ast_gen_ts_tsquery::prepare_matcher::<crate::types::Type>(&self.acc);
            PM(matcher)
        }
    }
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

const SOURCE2: &str = include_str!("java.tsg");
