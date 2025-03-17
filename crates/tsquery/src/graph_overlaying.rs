use super::stepped_query_imm;
use hyperast::{
    store::SimpleStores,
    tree_gen,
    types::{self, AstLending, HyperAST, HyperASTShared, StoreLending, StoreLending2},
};
use std::{fmt::Debug, hash::Hash};

pub struct PreparedOverlay<Q, O> {
    pub query: Option<Q>,
    pub overlayer: O,
    pub functions: std::sync::Arc<dyn std::any::Any>,
}

#[cfg(feature = "tsg")]
impl<'aaa, 'hast, 'g, 'q, 'm, HAST, Acc> tree_gen::More<HAST>
    for PreparedOverlay<
        &'q crate::Query,
        &'m tree_sitter_graph::ast::File<
            stepped_query_imm::QueryMatcher<<HAST as hyperast::types::StoreLending2>::S<'_>, &Acc>,
        >,
    >
where
    // HAST: 'static + HyperAST + for<'a> types::StoreLending2,
    HAST: types::StoreLending2,
    HAST::IdN: Copy + Hash + Debug,
    HAST::Idx: Hash,
    // for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
    //     types::WithSerialization + types::WithStats + types::WithRoles,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
        types::WithSerialization + types::WithStats + types::WithRoles,
    HAST::TS: 'static
        + Clone
        + types::ETypeStore<Ty2 = Acc::Type>
        + types::RoleStore<IdF = u16, Role = types::Role>,
    Acc: tree_gen::WithRole<types::Role> + tree_gen::WithChildren<HAST::IdN> + types::Typed,
    for<'acc> &'acc Acc: tree_gen::WithLabel<L = &'acc str>,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    type Acc = Acc;

    const ENABLED: bool = true;

    fn match_precomp_queries(
        &self,
        stores: <HAST as hyperast::types::StoreLending2>::S<'_>,
        acc: &Acc,
        label: Option<&str>,
    ) -> tree_gen::PrecompQueries {
        let Some(query) = self.query else {
            return Default::default();
        };
        // self.query.match_precomp_queries(stores, acc, label)
        if query.enabled_pattern_count() == 0 {
            return Default::default();
        }
        let pos = hyperast::position::StructuralPosition::empty();
        let cursor = crate::cursor_on_unbuild::TreeCursor::new(stores, acc, label, pos);
        // let cursor: crate::cursor_on_unbuild::Node<
        //     <HAST as types::StoreLending<'_,>>::S,
        //     &Acc,
        //     <HAST as HyperASTShared>::Idx,
        //     hyperast::position::structural_pos::StructuralPosition<
        //         <HAST as HyperASTShared>::IdN,
        //         <HAST as HyperASTShared>::Idx,
        //     >,
        //     &str,
        // > = crate::cursor_on_unbuild::Node::<_, _, _, _, _> {
        //     stores: stores,
        //     acc: acc,
        //     label: label,
        //     offset: num::zero(),
        //     pos: pos,
        // };
        // let qcursor = query.matches_immediate(cursor); // TODO filter on height (and visibility?)
        // let cursor = unsafe { std::mem::transmute(cursor) };
        let mut qcursor: crate::QueryCursor<
            '_,
            _,
            // crate::cursor_on_unbuild::Node<
            //     <HAST as types::StoreLending<'_>>::S,
            //     &Acc,
            //     <HAST as HyperASTShared>::Idx,
            //     hyperast::position::structural_pos::StructuralPosition<
            //         <HAST as HyperASTShared>::IdN,
            //         <HAST as HyperASTShared>::Idx,
            //     >,
            //     &str,
            // >,
            crate::cursor_on_unbuild::Node<
                <HAST as types::StoreLending2>::S<'_>,
                &Acc,
                <HAST as HyperASTShared>::Idx,
                hyperast::position::structural_pos::StructuralPosition<
                    <HAST as HyperASTShared>::IdN,
                    <HAST as HyperASTShared>::Idx,
                >,
                &str,
            >,
        > = query.matches_immediate(cursor); // TODO filter on height (and visibility?)
        let mut r = Default::default();
        loop {
            let Some(m) = qcursor._next_match() else {
                break;
            };
            assert!(m.pattern_index.to_usize() < 16);
            r |= 1 << m.pattern_index.to_usize() as u16;
        }
        r
    }
}

#[cfg(feature = "tsg")]
impl<'aaa, 'g, 'q, 'm, 'hast, HAST, Acc> tree_gen::Prepro<HAST>
    for PreparedOverlay<
        &'q crate::Query,
        &'m tree_sitter_graph::ast::File<stepped_query_imm::QueryMatcher<<HAST as hyperast::types::StoreLending2>::S<'_>, &Acc>>,
    >
where
    HAST::TS: types::ETypeStore,
    HAST: types::StoreLending2,
    HAST::IdN: Copy + Hash + Debug,
    HAST::Idx: Hash,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
        types::WithSerialization + types::WithStats + types::WithRoles,
    HAST::TS: 'static
        + Clone
        + types::ETypeStore<Ty2 = Acc::Type>
        + types::RoleStore<IdF = u16, Role = types::Role>,
    Acc: tree_gen::WithRole<types::Role> + tree_gen::WithChildren<HAST::IdN> + types::Typed,
    for<'acc> &'acc Acc: tree_gen::WithLabel<L = &'acc str>,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    const USING: bool = false;

    fn preprocessing(
        &self,
        _ty: <HAST::TS as types::ETypeStore>::Ty2,
    ) -> Result<hyperast::scripting::Acc, String> {
        unimplemented!()
    }
}

#[cfg(feature = "tsg")]
impl<'aaa, 'g, 'q, 'm, 'hast, HAST, Acc> tree_gen::PreproTSG<HAST>
    for PreparedOverlay<
        &'q crate::Query,
        &'m tree_sitter_graph::ast::File<
            stepped_query_imm::QueryMatcher<<HAST as hyperast::types::StoreLending2>::S<'_>, &Acc>,
        >,
    >
where
    // HAST: 'static + HyperAST + for<'a> types::StoreLending<'a>,
    HAST: types::StoreLending2,
    HAST::IdN: 'static + Copy + Hash + Debug,
    HAST::Idx: 'static + Hash,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
        types::WithSerialization + types::WithStats + types::WithRoles,
    HAST::TS: 'static
        + Clone
        + types::ETypeStore<Ty2 = Acc::Type>
        + types::RoleStore<IdF = u16, Role = types::Role>
        + types::TypeStore,
    Acc: types::Typed
        + 'static
        + tree_gen::WithRole<types::Role>
        + tree_gen::WithChildren<HAST::IdN>
        + types::Typed,
    for<'acc> &'acc Acc: tree_gen::WithLabel<L = &'acc str>,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    const GRAPHING: bool = true;
    // TODO remove the 'static and other contraints, they add unnecessary unsafes
    // there is probably something to do with spliting GenQuery and the different execs to avoid both
    // - holding graph as mutable to often
    // - bubling the mutability invariant from graph to HAST... (very bad)
    fn compute_tsg(
        &self,
        stores: <HAST as hyperast::types::StoreLending2>::S<'_>,
        acc: &Acc,
        label: Option<&str>,
    ) -> Result<usize, String> {
        // NOTE I had to do a lot of unsafe magic :/
        // mostly exending lifetime and converting HAST to HAST2 on compatible structures

        use tree_sitter_graph::graph::Graph;
        let cancellation_flag = tree_sitter_graph::NoCancellation;
        let mut globals = tree_sitter_graph::Variables::new();
        let mut graph: Graph<
            crate::hyperast_cursor::NodeR<
                hyperast::position::StructuralPosition<HAST::IdN, HAST::Idx>,
            >,
        > = tree_sitter_graph::graph::Graph::default();
        init_globals(&mut globals, &mut graph);

        // retreive the custom functions
        // NOTE need the concrete type of the stores to instanciate
        // WARN cast will fail if the original instance type was not identical
        type Fcts<T> = tree_sitter_graph::functions::Functions<
            T, // tree_sitter_graph::graph::GraphErazing<stepped_query_imm::MyNodeErazing<HAST, Acc>>,
        >;
        let functions: &Fcts<_> = std::ops::Deref::deref(&self.functions)
            .downcast_ref()
            .expect("identical instance type");

        // tree_sitter_stack_graphs::functions::add_path_functions(&mut functions);

        let mut config = configure(&globals, functions);

        let pos = hyperast::position::StructuralPosition::empty();
        let tree = stepped_query_imm::Node::new(stores, acc, label, pos);

        // NOTE could not use the execute_lazy_into due to limitations with type checks (needed some transmutes)
        // ORI: self.overlayer.execute_lazy_into2(&mut graph, tree, &config, &cancellation_flag).unwrap();
        // {
        let mut ctx = tree_sitter_graph::execution::Ctx::new();

        let mut cursor = vec![];
        // NOTE could not find a way to make it type check without inlining
        // ORI: let mut matches = this.query.matches(&mut cursor, tree);
        let mut matches = {
            let q: &stepped_query_imm::QueryMatcher<_, &Acc> =
                unsafe { std::mem::transmute(self.overlayer.query.as_ref().unwrap()) };
            // log::error!("{:?}",this.query.as_ref().unwrap().query.capture_names);
            let node = &tree;
            let cursor = &mut cursor;
            // log::error!("{:?}",this.query.as_ref().unwrap().query);
            // log::error!("{}",this.query.as_ref().unwrap().query);
            let qm = self.overlayer.query.as_ref().unwrap();
            let matchs = qm.query.matches_immediate(node.clone());
            let node = node.clone();
            stepped_query_imm::MyQMatches::<_, _, _> {
                q,
                cursor,
                matchs,
                node,
            }
        };
        let graph = &mut graph;
        loop {
            // NOTE needed to make a transmute to type check
            // ORI: ... matches.next() ...
            let mat: stepped_query_imm::MyQMatch<_, &Acc> = {
                let Some(mat) = matches.next() else { break };
                let mat = stepped_query_imm::MyQMatch {
                    stores: tree.0.stores.clone(),
                    b: mat.b,
                    qm: unsafe { std::mem::transmute(mat.qm) },
                    i: mat.i,
                };
                mat
            };
            use tree_sitter_graph::graph::QMatch;
            let stanza = &self.overlayer.stanzas[mat.pattern_index()];
            // NOTE could not type check it either
            // ORI: stanza.execute_lazy2(
            {
                let inherited_variables = &self.overlayer.inherited_variables;
                let shorthands = &self.overlayer.shorthands;
                let mat = &mat;
                let config = &mut config;
                let cancellation_flag = &cancellation_flag;
                let current_regex_captures = vec![];
                ctx.clear();
                let node = mat
                    .nodes_for_capture_indexi(stanza.full_match_file_capture_index.into())
                    .expect("missing capture for full match");
                log::error!("{:?}", node.0.pos);
                // debug!("match {:?} at {}", node, self.range.start);
                // trace!("{{");
                for statement in &stanza.statements {
                    // NOTE could not properly get the source location, just use a zeroed location
                    // ORI: let error_context = StatementContext::new(...
                    let error_context = {
                        let stmt: &tree_sitter_graph::ast::Statement = &statement;
                        let stanza = &stanza;
                        let source_node = &node;
                        // use crate::graph::SyntaxNode;
                        // let source_location: Location::from(source_node.start_position()), // TODO make a better location for hyperast;
                        let source_location = tree_sitter_graph::Location { row: 0, column: 0 };
                        tree_sitter_graph::execution::error::StatementContext::raw(
                            stmt,
                            stanza.range.start,
                            source_location,
                            source_node.0.kind().to_string(),
                        )
                    };
                    let mat: &QM<_, &Acc, HAST::IdN, HAST::Idx> = mat;
                    type G<IdN, Idx> = Graph<
                        crate::hyperast_cursor::NodeR<
                            hyperast::position::StructuralPosition<IdN, Idx>,
                        >,
                    >;
                    type P<IdN, Idx> = hyperast::position::StructuralPosition<IdN, Idx>;
                    type QM<'c, HAST, Acc, IdN, Idx> =
                        stepped_query_imm::MyQMatch<'c, HAST, Acc, Idx, P<IdN, Idx>>;
                    let full_match_file_capture_index = stanza.full_match_file_capture_index.into();
                    if let Err(err) = ctx
                        .exec::<G<HAST::IdN, HAST::Idx>, QM<HAST::S<'_>, &Acc, HAST::IdN, HAST::Idx>, _>(
                            graph,
                            inherited_variables,
                            cancellation_flag,
                            full_match_file_capture_index,
                            shorthands,
                            mat,
                            config,
                            &current_regex_captures,
                            &statement,
                            error_context,
                        )
                    {
                        log::trace!("{}", graph.pretty_print());
                        let source_path = std::path::Path::new(&"");
                        let tsg_path = std::path::Path::new(&"");
                        log::error!("{}", err.display_pretty(&source_path, "", &tsg_path, ""));
                    }
                    // .with_context(|| exec.error_context.into())?
                }
            };
        }

        if let Err(err) = ctx.eval(
            graph,
            functions,
            &self.overlayer.inherited_variables,
            &cancellation_flag,
        ) {
            log::trace!("{}", graph.pretty_print());
            let source_path = std::path::Path::new(&"");
            let tsg_path = std::path::Path::new(&"");
            log::error!("{}", err.display_pretty(&source_path, "", &tsg_path, ""));
        }
        // }

        // TODO properly return and use the graph (also handle the error propagation)
        if graph.node_count() > 2 {
            log::error!("curr kind {}", hyperast::types::Typed::get_type(acc));
            let s = graph.to_json().unwrap();
            log::error!("graph: {}", s);
        }
        Ok(graph.node_count())
    }
}

// pub use tree_sitter_stack_graphs::functions::add_path_functions;

static DEBUG_ATTR_PREFIX: &'static str = "debug_";
pub static ROOT_NODE_VAR: &'static str = "ROOT_NODE";
/// The name of the file path global variable
pub const FILE_PATH_VAR: &str = "FILE_PATH";
static JUMP_TO_SCOPE_NODE_VAR: &'static str = "JUMP_TO_SCOPE_NODE";
static FILE_NAME: &str = "a/b/AAA.java";

#[cfg(feature = "tsg")]
pub fn configure<'a, 'g, 'b, G>(
    globals: &'b tree_sitter_graph::Variables<'g>,
    functions: &'a tree_sitter_graph::functions::Functions<G>,
) -> tree_sitter_graph::ExecutionConfig<'a, 'g, 'b, G> {
    let config = tree_sitter_graph::ExecutionConfig::new(functions, globals).lazy(true);
    if !cfg!(debug_assertions) {
        config.debug_attributes(
            [DEBUG_ATTR_PREFIX, "tsg_location"].concat().as_str().into(),
            [DEBUG_ATTR_PREFIX, "tsg_variable"].concat().as_str().into(),
            [DEBUG_ATTR_PREFIX, "tsg_match_node"]
                .concat()
                .as_str()
                .into(),
        )
    } else {
        config
    }
}

#[cfg(feature = "tsg")]
pub fn init_globals<Node>(
    globals: &mut tree_sitter_graph::Variables,
    graph: &mut tree_sitter_graph::graph::Graph<Node>,
) {
    // globals
    //     .add(ROOT_NODE_VAR.into(), graph.add_graph_node().into())
    //     .expect("Failed to set ROOT_NODE");
    // // globals
    // //     .add(FILE_PATH_VAR.into(), FILE_NAME.into())
    // //     .expect("Failed to set FILE_PATH");
    // globals
    //     .add(JUMP_TO_SCOPE_NODE_VAR.into(), graph.add_graph_node().into())
    //     .expect("Failed to set JUMP_TO_SCOPE_NODE");
}
