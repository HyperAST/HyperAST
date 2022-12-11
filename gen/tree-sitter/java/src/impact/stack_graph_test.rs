use stack_graphs::graph::{NodeID, StackGraph};

mod types {
    // use stack_graphs::arena::Handle;
    // use stack_graphs::graph::{File as _File, Node as _Node, Symbol as _Symbol};

    // pub type File = Handle<_File>;
    // pub type Node = Handle<_Node>;
    // pub type Symbol = Handle<_Symbol>;
}

#[test]
fn test() {
    let mut graph = StackGraph::new();
    let root = graph.root_node();
    let sym_dot = graph.add_symbol(".");
    let sym_main = graph.add_symbol("__main__");
    let sym_a = graph.add_symbol("a");
    let _sym_b = graph.add_symbol("b");
    let sym_foo = graph.add_symbol("foo");

    let main_file = graph.get_or_create_file("main.py");
    let main = graph
        .add_pop_symbol_node(NodeID::new_in_file(main_file, 0), sym_main, true)
        .expect("Duplicate node ID");
    let main_dot_1 = graph
        .add_pop_symbol_node(NodeID::new_in_file(main_file, 1), sym_dot, false)
        .expect("Duplicate node ID");
    let main_bottom_2 = graph
        .add_internal_scope_node(NodeID::new_in_file(main_file, 2))
        .expect("Duplicate node ID");
    let main_3 = graph
        .add_internal_scope_node(NodeID::new_in_file(main_file, 3))
        .expect("Duplicate node ID");
    let main_4 = graph
        .add_internal_scope_node(NodeID::new_in_file(main_file, 4))
        .expect("Duplicate node ID");
    let main_top_5 = graph
        .add_internal_scope_node(NodeID::new_in_file(main_file, 5))
        .expect("Duplicate node ID");
    let main_foo = graph
        .add_push_symbol_node(NodeID::new_in_file(main_file, 6), sym_foo, true)
        .expect("Duplicate node ID");
    let main_dot_7 = graph
        .add_push_symbol_node(NodeID::new_in_file(main_file, 7), sym_dot, false)
        .expect("Duplicate node ID");
    let main_a = graph
        .add_push_symbol_node(NodeID::new_in_file(main_file, 8), sym_a, true)
        .expect("Duplicate node ID");
    graph.add_edge(root, main, 0);
    graph.add_edge(main, main_dot_1, 0);
    graph.add_edge(main_dot_1, main_bottom_2, 0);
    graph.add_edge(main_bottom_2, main_3, 0);
    graph.add_edge(main_foo, main_3, 0);
    graph.add_edge(main_3, main_4, 0);
    graph.add_edge(main_4, main_dot_7, 0);
    graph.add_edge(main_dot_7, main_a, 0);
    graph.add_edge(main_a, root, 0);
    graph.add_edge(main_4, main_top_5, 0);
}

mod t2 {
    use std::collections::BTreeSet;

    use stack_graphs::{
        arena::Handle,
        graph::{File, Node, NodeID, StackGraph, Symbol},
        partial::PartialPaths,
        stitching::{Database, ForwardPartialPathStitcher},
    };

    #[allow(non_snake_case)]
    #[test]
    fn test() {
        let mut graph = StackGraph::new();
        let root = graph.root_node();
        let jump_to = graph.jump_to_node();
        let sym_call = graph.symbol("()");
        let sym_dot = graph.symbol(".");
        let sym_zero = graph.symbol("0");
        let sym_main = graph.symbol("__main__");
        let sym_A = graph.symbol("A");
        let sym_a = graph.symbol("a");
        let sym_b = graph.symbol("b");
        let sym_x = graph.symbol("x");
        let sym_foo = graph.symbol("foo");
        let sym_bar = graph.symbol("bar");
        let sym_print = graph.symbol("print");

        let main_file = graph.file("main.py");
        let main = graph.definition(main_file, 0, sym_main);
        let main_dot_1 = graph.pop_symbol(main_file, 1, sym_dot);
        let main_bottom_2 = graph.internal_scope(main_file, 2);
        let main_3 = graph.internal_scope(main_file, 3);
        let main_4 = graph.internal_scope(main_file, 4);
        let main_5 = graph.internal_scope(main_file, 5);
        let main_top_6 = graph.internal_scope(main_file, 6);
        let main_exported = graph.exported_scope(main_file, 7);
        let main_zero_8 = graph.pop_symbol(main_file, 8, sym_zero);
        let main_A = graph.reference(main_file, 9, sym_A); // foo(#A#)
        let main_bar = graph.reference(main_file, 10, sym_bar); // .#bar#
        let main_dot_11 = graph.push_symbol(main_file, 11, sym_dot); // #.#bar
        let main_call_12 = graph.push_scoped_symbol(main_file, 12, sym_call, main_file, 7);
        let main_foo = graph.reference(main_file, 13, sym_foo); // #foo#(A)
        let main_dot_14 = graph.push_symbol(main_file, 14, sym_dot);
        let main_b = graph.reference(main_file, 15, sym_b);
        let main_dot_16 = graph.push_symbol(main_file, 16, sym_dot);
        let main_a = graph.reference(main_file, 17, sym_a);
        let main_call_18 = graph.push_scoped_symbol(main_file, 18, sym_call, main_file, 7);
        let main_print = graph.reference(main_file, 19, sym_print);
        graph.edge(root, main);
        graph.edge(main, main_dot_1);
        graph.edge(main_dot_1, main_bottom_2);
        graph.edge(main_bottom_2, main_3);
        graph.edge(main_exported, main_zero_8);
        graph.edge(main_zero_8, main_A);
        graph.edge(main_A, main_3);
        graph.edge(main_bar, main_dot_11);
        graph.edge(main_dot_11, main_call_12);
        graph.edge(main_call_12, main_foo);
        graph.edge(main_foo, main_3);
        graph.edge(main_3, main_4);
        graph.edge(main_4, main_dot_14);
        graph.edge(main_dot_14, main_b);
        graph.edge(main_b, root);
        graph.edge(main_4, main_5);
        graph.edge(main_5, main_dot_16);
        graph.edge(main_dot_16, main_a);
        graph.edge(main_a, root);
        graph.edge(main_5, main_top_6);
        // TODO not sure
        graph.edge(main_call_18, main_print);
        graph.edge(main_print, main_3);

        let a_file = graph.file("a.py");
        let a = graph.definition(a_file, 0, sym_a);
        let a_dot_1 = graph.pop_symbol(a_file, 1, sym_dot);
        let a_bottom_2 = graph.internal_scope(a_file, 2);
        let a_3 = graph.internal_scope(a_file, 3);
        let a_top_4 = graph.internal_scope(a_file, 4);
        let a_foo = graph.definition(a_file, 5, sym_foo);
        let a_call_6 = graph.pop_scoped_symbol(a_file, 6, sym_call);
        let a_return_7 = graph.internal_scope(a_file, 7);
        let a_x_ref = graph.reference(a_file, 8, sym_x);
        let a_params_9 = graph.internal_scope(a_file, 9);
        let a_drop_10 = graph.drop_scopes(a_file, 10);
        let a_lexical_11 = graph.internal_scope(a_file, 11);
        let a_formals_12 = graph.internal_scope(a_file, 12);
        let a_drop_13 = graph.drop_scopes(a_file, 13);
        let a_x_def = graph.definition(a_file, 14, sym_x);
        let a_x_15 = graph.pop_symbol(a_file, 15, sym_x);
        let a_zero_16 = graph.push_symbol(a_file, 16, sym_zero);
        let a_x_17 = graph.push_symbol(a_file, 17, sym_x);
        graph.edge(root, a);
        graph.edge(a, a_dot_1);
        graph.edge(a_dot_1, a_bottom_2);
        graph.edge(a_bottom_2, a_3);
        graph.edge(a_3, a_foo);
        graph.edge(a_foo, a_call_6);
        graph.edge(a_call_6, a_return_7);
        graph.edge(a_return_7, a_x_ref);
        graph.edge(a_x_ref, a_params_9);
        graph.edge(a_params_9, a_drop_10);
        graph.edge(a_drop_10, a_lexical_11);
        graph.edge(a_lexical_11, a_bottom_2);
        graph.edge(a_params_9, a_formals_12);
        graph.edge(a_formals_12, a_drop_13);
        graph.edge(a_drop_13, a_x_def);
        graph.edge(a_formals_12, a_x_15);
        graph.edge(a_x_15, a_zero_16);
        graph.edge(a_zero_16, jump_to);
        graph.edge(a_x_15, a_x_17);
        graph.edge(a_x_17, jump_to);
        graph.edge(a_3, a_top_4);

        let b_file = graph.file("b.py");
        let b = graph.definition(b_file, 0, sym_b);
        let b_dot_1 = graph.pop_symbol(b_file, 1, sym_dot);
        let b_bottom_2 = graph.internal_scope(b_file, 2);
        let b_3 = graph.internal_scope(b_file, 3);
        let b_top_4 = graph.internal_scope(b_file, 4);
        let b_A = graph.definition(b_file, 5, sym_A);
        let b_dot_6 = graph.pop_symbol(b_file, 6, sym_dot);
        let b_class_members_7 = graph.internal_scope(b_file, 7);
        let b_bar = graph.definition(b_file, 8, sym_bar);
        let b_call_9 = graph.pop_scoped_symbol(b_file, 9, sym_call);
        let b_self_10 = graph.internal_scope(b_file, 10);
        let b_dot_11 = graph.pop_symbol(b_file, 11, sym_dot);
        let b_instance_members_12 = graph.internal_scope(b_file, 12);
        graph.edge(root, b);
        graph.edge(b, b_dot_1);
        graph.edge(b_dot_1, b_bottom_2);
        graph.edge(b_bottom_2, b_3);
        graph.edge(b_3, b_A);
        graph.edge(b_A, b_dot_6);
        graph.edge(b_dot_6, b_class_members_7);
        graph.edge(b_class_members_7, b_bar);
        graph.edge(b_A, b_call_9);
        graph.edge(b_call_9, b_self_10);
        graph.edge(b_self_10, b_dot_11);
        graph.edge(b_dot_11, b_instance_members_12);
        graph.edge(b_instance_members_12, b_class_members_7);
        graph.edge(b_3, b_top_4);

        check_jump_to_definition(
            &graph,
            &[
                // reference to `a` in import statement
                "<%1> () [main.py(17) reference a] -> [a.py(0) definition a] <%1> ()",
                // reference to `b` in import statement
                "<%1> () [main.py(15) reference b] -> [b.py(0) definition b] <%1> ()",
                // reference to `foo` in function call resolves to function definition
                "<%1> () [main.py(13) reference foo] -> [a.py(5) definition foo] <%1> ()",
                // reference to `A` as function parameter resolves to class definition
                "<%1> () [main.py(9) reference A] -> [b.py(5) definition A] <%1> ()",
                // reference to `bar` on result flows through body of `foo` to find `A.bar`
                "<%1> () [main.py(10) reference bar] -> [b.py(8) definition bar] <%1> ()",
                // reference to `x` in function body resolves to formal parameter
                "<%1> () [a.py(8) reference x] -> [a.py(14) definition x] <%1> ()",
            ],
        );
    }

    fn check_jump_to_definition(graph: &StackGraph, expected_partial_paths: &[&str]) {
        let mut partials = PartialPaths::new();
        let mut db = Database::new();

        // Generate partial paths for everything in the database.
        for file in graph.iter_files() {
            partials.find_all_partial_paths_in_file(graph, file, |graph, partials, path| {
                if !path.is_complete_as_possible(graph) {
                    return;
                }
                if !path.is_productive(partials) {
                    return;
                }
                db.add_partial_path(graph, partials, path);
            });
        }

        let references = graph
            .iter_nodes()
            .filter(|handle| graph[*handle].is_reference());
        let complete_partial_paths = ForwardPartialPathStitcher::find_all_complete_partial_paths(
            graph,
            &mut partials,
            &mut db,
            references,
        );
        let results = complete_partial_paths
            .into_iter()
            .map(|partial_path| partial_path.display(graph, &mut partials).to_string())
            .collect::<BTreeSet<_>>();

        let expected_partial_paths = expected_partial_paths
            .iter()
            .map(|s| s.to_string())
            .collect::<BTreeSet<_>>();
        assert_eq!(expected_partial_paths, results);
    }

    pub trait CreateStackGraph {
        type File: Clone + Copy;
        type Node: Clone + Copy;
        type Symbol: Clone + Copy;

        fn definition(
            &mut self,
            file: Self::File,
            local_id: u32,
            symbol: Self::Symbol,
        ) -> Self::Node;

        fn drop_scopes(&mut self, file: Self::File, local_id: u32) -> Self::Node;

        fn edge(&mut self, source: Self::Node, sink: Self::Node);

        fn exported_scope(&mut self, file: Self::File, local_id: u32) -> Self::Node;

        fn file(&mut self, name: &str) -> Self::File;

        fn internal_scope(&mut self, file: Self::File, local_id: u32) -> Self::Node;

        fn jump_to_node(&mut self) -> Self::Node;

        fn pop_scoped_symbol(
            &mut self,
            file: Self::File,
            local_id: u32,
            symbol: Self::Symbol,
        ) -> Self::Node;

        fn pop_symbol(
            &mut self,
            file: Self::File,
            local_id: u32,
            symbol: Self::Symbol,
        ) -> Self::Node;

        fn push_scoped_symbol(
            &mut self,
            file: Self::File,
            local_id: u32,
            symbol: Self::Symbol,
            scope_file: Self::File,
            scope_id: u32,
        ) -> Self::Node;

        fn push_symbol(
            &mut self,
            file: Self::File,
            local_id: u32,
            symbol: Self::Symbol,
        ) -> Self::Node;

        fn reference(
            &mut self,
            file: Self::File,
            local_id: u32,
            symbol: Self::Symbol,
        ) -> Self::Node;

        fn root_node(&mut self) -> Self::Node;

        fn symbol(&mut self, value: &str) -> Self::Symbol;
    }

    impl CreateStackGraph for StackGraph {
        type File = Handle<File>;
        type Node = Handle<Node>;
        type Symbol = Handle<Symbol>;

        fn definition(
            &mut self,
            file: Handle<File>,
            local_id: u32,
            symbol: Handle<Symbol>,
        ) -> Handle<Node> {
            self.add_pop_symbol_node(NodeID::new_in_file(file, local_id), symbol, true)
                .expect("Duplicate node ID")
        }

        fn drop_scopes(&mut self, file: Handle<File>, local_id: u32) -> Handle<Node> {
            self.add_drop_scopes_node(NodeID::new_in_file(file, local_id))
                .expect("Duplicate node ID")
        }

        fn edge(&mut self, source: Handle<Node>, sink: Handle<Node>) {
            self.add_edge(source, sink, 0);
        }

        fn exported_scope(&mut self, file: Handle<File>, local_id: u32) -> Handle<Node> {
            self.add_exported_scope_node(NodeID::new_in_file(file, local_id))
                .expect("Duplicate node ID")
        }

        fn file(&mut self, name: &str) -> Handle<File> {
            self.get_or_create_file(name)
        }

        fn internal_scope(&mut self, file: Handle<File>, local_id: u32) -> Handle<Node> {
            self.add_internal_scope_node(NodeID::new_in_file(file, local_id))
                .expect("Duplicate node ID")
        }

        fn jump_to_node(&mut self) -> Handle<Node> {
            StackGraph::jump_to_node(self)
        }

        fn pop_scoped_symbol(
            &mut self,
            file: Handle<File>,
            local_id: u32,
            symbol: Handle<Symbol>,
        ) -> Handle<Node> {
            self.add_pop_scoped_symbol_node(NodeID::new_in_file(file, local_id), symbol, false)
                .expect("Duplicate node ID")
        }

        fn pop_symbol(
            &mut self,
            file: Handle<File>,
            local_id: u32,
            symbol: Handle<Symbol>,
        ) -> Handle<Node> {
            self.add_pop_symbol_node(NodeID::new_in_file(file, local_id), symbol, false)
                .expect("Duplicate node ID")
        }

        fn push_scoped_symbol(
            &mut self,
            file: Handle<File>,
            local_id: u32,
            symbol: Handle<Symbol>,
            scope_file: Handle<File>,
            scope_id: u32,
        ) -> Handle<Node> {
            let scope = NodeID::new_in_file(scope_file, scope_id);
            self.add_push_scoped_symbol_node(
                NodeID::new_in_file(file, local_id),
                symbol,
                scope,
                false,
            )
            .expect("Duplicate node ID")
        }

        fn push_symbol(
            &mut self,
            file: Handle<File>,
            local_id: u32,
            symbol: Handle<Symbol>,
        ) -> Handle<Node> {
            self.add_push_symbol_node(NodeID::new_in_file(file, local_id), symbol, false)
                .expect("Duplicate node ID")
        }

        fn reference(
            &mut self,
            file: Handle<File>,
            local_id: u32,
            symbol: Handle<Symbol>,
        ) -> Handle<Node> {
            self.add_push_symbol_node(NodeID::new_in_file(file, local_id), symbol, true)
                .expect("Duplicate node ID")
        }

        fn root_node(&mut self) -> Handle<Node> {
            StackGraph::root_node(self)
        }

        fn symbol(&mut self, value: &str) -> Handle<Symbol> {
            self.add_symbol(value)
        }
    }
}
