mod predicate;

// static const TSQueryError PARENT_DONE = -1;

const PATTERN_DONE_MARKER: u16 = u16::MAX;

// #define MAX_STEP_CAPTURE_COUNT 3
const MAX_STEP_CAPTURE_COUNT: usize = 3;
// #define MAX_NEGATED_FIELD_COUNT 8
// #define MAX_STATE_PREDECESSOR_COUNT 256

mod indexed;
mod query;
mod query_cursor;
mod utils;

mod ffi;
mod ffi_extra;
mod ts_private_bypass;

pub mod default_impls;
#[cfg(feature = "hyper_ast")]
pub mod hyperast;
#[cfg(feature = "hyper_ast")]
pub mod hyperast_opt;
pub mod stepped_query;
pub mod stepped_query_imm;
pub mod tsg;

pub mod cursor_on_unbuild;

use hyper_ast::types::HyperAST;
pub use tree_sitter::CaptureQuantifier;
pub use tree_sitter::Language;
pub use tree_sitter::Point;
pub use tree_sitter::QueryError;
pub use tree_sitter::QueryErrorKind;
pub(crate) use ts_private_bypass::*;

pub use indexed::CaptureId;
pub use indexed::PatternId;
pub use indexed::Symbol;

use std::fmt::Debug;
use std::ops::Deref;
use std::{collections::VecDeque, usize};

pub use stepped_query_imm::ExtendingStringQuery;
pub use stepped_query_imm::MyNodeErazing;
pub use stepped_query_imm::QueryMatcher;

type Depth = u32;
type Precomps = u16;
// type Precomps = u16;

#[derive(Clone)]
pub struct Query {
    // captures: SymbolTable,
    // predicate_values: SymbolTable,
    // capture_quantifiers: utils::Array<CaptureQuantifiers>,
    steps: indexed::Steps,
    pattern_map: Vec<query::PatternEntry>,
    pattern_map2: Vec<query::PatternEntry>, // Note: unused for now, planed to use it for preprocessed queries but not sur in the end
    // predicate_steps: utils::Array<ffi::TSQueryPredicateStep>,
    patterns: indexed::Patterns,
    step_offsets: Vec<query::StepOffset>,
    negated_fields: indexed::NegatedFields,
    // string_buffer: utils::Array<char>,
    // repeat_symbols_with_rootless_patterns: utils::Array<ffi::TSSymbol>,
    language: *const ffi::TSLanguage,
    wildcard_root_pattern_count: u16,

    capture_names: Vec<&'static str>,
    capture_quantifiers_vec: Vec<Vec<CaptureQuantifier>>,
    text_predicates: predicate::TextPredicateCaptures,
    property_predicates: predicate::PropertyPredicates,
    property_settings: predicate::PropertySettings,
    general_predicates: predicate::GeneralPredicates,
    immediate_predicates: Vec<predicate::ImmediateTextPredicate>,
    precomputed_patterns: Option<query::PrecomputedPatterns>,
    used_precomputed: Precomps,
    enabled_pattern_map: Vec<u16>,
    enabled_pattern_count: u16,
}

unsafe impl Send for Query {}
unsafe impl Sync for Query {}

#[derive(Clone, Debug)]
struct Slice<I> {
    offset: I,
    length: I,
}

impl<I> Slice<I> {
    fn new(offset: I, length: I) -> Self {
        Self { offset, length }
    }
}

impl Query {
    pub fn matches<'query, Cursor: self::Cursor>(
        &'query self,
        cursor: Cursor,
    ) -> QueryCursor<'query, Cursor, Cursor::Node> {
        QueryCursor::<Cursor, _> {
            halted: false,
            ascending: false,
            states: vec![],
            capture_list_pool: Default::default(),
            finished_states: Default::default(),
            max_start_depth: u32::MAX,
            did_exceed_match_limit: false,
            depth: 0,
            on_visible_node: cursor.is_visible_at_root(),
            query: self,
            cursor,
            next_state_id: indexed::StateId::ZERO,
        }
    }

    /// Match all patterns that starts on cursor current node
    pub fn matches_immediate<'query, Cursor: self::Cursor>(
        &'query self,
        cursor: Cursor,
    ) -> QueryCursor<'query, Cursor, Cursor::Node> {
        let mut qcursor = self.matches(cursor);
        // can only match patterns starting on provided node
        qcursor.set_max_start_depth(0);
        qcursor
    }

    pub fn _check_preprocessed(&self, pattern_id: usize, precomp: usize) {
        if pattern_id == 0 && self.pattern_map.len() == 1 {
            assert_eq!(
                self.used_precomputed.count_ones() as usize,
                precomp,
                "{:b}",
                self.used_precomputed
            );
        } else {
            for p in &self.pattern_map {
                if p.pattern_index == PatternId::new(pattern_id) {
                    assert_eq!(p.precomputed.count_ones() as usize, precomp);
                }
            }
        }
    }

    pub fn capture_count(&self) -> usize {
        self.capture_names.len()
    }
}

pub struct QueryCursor<'query, Cursor, Node> {
    halted: bool,
    ascending: bool,
    on_visible_node: bool,
    cursor: Cursor,
    query: &'query Query,
    states: Vec<query_cursor::State>,
    depth: Depth,
    max_start_depth: Depth,
    capture_list_pool: indexed::CaptureListPool<Node>,
    finished_states: VecDeque<query_cursor::State>,
    next_state_id: indexed::StateId,
    // only triggers when there is no more capture list available
    // not triggered by reaching max_start_depth
    did_exceed_match_limit: bool,
}

#[derive(Clone)]
pub struct Capture<Node> {
    pub node: Node,
    pub index: CaptureId,
}

impl<Node> Debug for Capture<Node> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.index.to_usize())
    }
}

pub struct QueryMatch<Node> {
    pub pattern_index: PatternId,
    pub captures: indexed::Captures<Node>,
    // id of state
    id: indexed::StateId,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum TreeCursorStep {
    TreeCursorStepNone,
    TreeCursorStepHidden,
    TreeCursorStepVisible,
}

pub trait Cursor {
    type Node: Node + Clone;
    type NodeRef<'a>: Node<IdF = <Self::Node as Node>::IdF, TP<'a> = <Self::Node as Node>::TP<'a>>
    where
        Self: 'a;

    fn goto_next_sibling_internal(&mut self) -> TreeCursorStep;

    fn goto_first_child_internal(&mut self) -> TreeCursorStep;

    fn goto_parent(&mut self) -> bool;
    fn current_node(&self) -> Self::NodeRef<'_>;

    fn parent_is_error(&self) -> bool;

    type Status: Status<IdF = <Self::Node as Node>::IdF>;

    fn current_status(&self) -> Self::Status;

    fn text_provider(&self) -> <Self::Node as Node>::TP<'_>;

    fn wont_match(&self, _needed: Precomps) -> bool {
        false
    }
    fn is_visible_at_root(&self) -> bool {
        true
    }
    fn has_parent(&self) -> bool;
    fn persist(&mut self) -> Self::Node;
    fn persist_parent(&mut self) -> Option<Self::Node>;
}

pub trait Status {
    type IdF;
    fn has_later_siblings(&self) -> bool;
    fn has_later_named_siblings(&self) -> bool;
    fn can_have_later_siblings_with_this_field(&self) -> bool;
    fn field_id(&self) -> Self::IdF;
    fn has_supertypes(&self) -> bool;
    fn contains_supertype(&self, sym: Symbol) -> bool;
}

pub trait Node {
    type IdF;

    fn symbol(&self) -> Symbol;

    fn is_named(&self) -> bool;
    fn str_symbol(&self) -> &str;

    fn start_point(&self) -> crate::Point;

    fn has_child_with_field_id(&self, field_id: Self::IdF) -> bool;

    fn equal(&self, other: &Self) -> bool;
    fn compare(&self, other: &Self) -> std::cmp::Ordering;
    type TP<'a>: Copy;
    fn text(&self, text_provider: Self::TP<'_>) -> std::borrow::Cow<str>;
    fn text_equal(&self, text_provider: Self::TP<'_>, other: impl Iterator<Item = u8>) -> bool {
        self.text(text_provider)
            .as_bytes()
            .iter()
            .copied()
            .eq(other)
    }
}

impl<T> Node for &T
where
    T: Node,
{
    type IdF = T::IdF;

    fn symbol(&self) -> Symbol {
        (*self).symbol()
    }

    fn is_named(&self) -> bool {
        (*self).is_named()
    }

    fn str_symbol(&self) -> &str {
        (*self).str_symbol()
    }

    fn start_point(&self) -> crate::Point {
        (*self).start_point()
    }

    fn has_child_with_field_id(&self, field_id: Self::IdF) -> bool {
        (*self).has_child_with_field_id(field_id)
    }

    fn equal(&self, other: &Self) -> bool {
        (*self).equal(other)
    }

    fn compare(&self, other: &Self) -> std::cmp::Ordering {
        (*self).compare(other)
    }

    type TP<'a> = T::TP<'a>;

    fn text(&self, text_provider: Self::TP<'_>) -> std::borrow::Cow<str> {
        (*self).text(text_provider)
    }
}

impl<'query, Cursor: self::Cursor> Iterator for QueryCursor<'query, Cursor, Cursor::Node>
where
    <Cursor::Status as Status>::IdF: Into<u16> + From<u16>,
{
    type Item = QueryMatch<Cursor::Node>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let result = self.next_match()?;
            if result.satisfies_text_predicates(
                self.cursor.text_provider(),
                self.query
                    .text_predicates_for_pattern_id(result.pattern_index),
            ) {
                return Some(result);
            }
        }
    }
}
impl<Node: self::Node> QueryMatch<Node> {
    pub(crate) fn satisfies_text_predicates<'a, 'b, T: 'a + AsRef<str>>(
        &self,
        text_provider: Node::TP<'b>,
        mut text_predicates: impl Iterator<Item = &'a TextPredicateCapture<T>>,
    ) -> bool {
        text_predicates.all(|predicate| match predicate {
            TextPredicateCapture::EqCapture(left, right, is_positive, match_all_nodes) => {
                // WARN sligntly different sem as we compare nodes structurally and not textually
                // bad for comparing the name of a type ref with the name of a variable ref
                let mut nodes_1 = self.nodes_for_capture_index(*left);
                let mut nodes_2 = self.nodes_for_capture_index(*right);
                while let (Some(node1), Some(node2)) = (nodes_1.next(), nodes_2.next()) {
                    let comp = node1.equal(node2);
                    if comp != *is_positive && *match_all_nodes {
                        return false;
                    }
                    if comp == *is_positive && !*match_all_nodes {
                        return true;
                    }
                }
                nodes_1.next().is_none() && nodes_2.next().is_none()
            }
            TextPredicateCapture::EqString(left, right, is_positive, match_all_nodes) => {
                let nodes = self.nodes_for_capture_index(*left);
                let s = right.as_ref().as_bytes();
                for node in nodes {
                    let comp = node.text_equal(text_provider, s.iter().copied());
                    if comp != *is_positive && *match_all_nodes {
                        return false;
                    }
                    if comp == *is_positive && !*match_all_nodes {
                        return true;
                    }
                }
                true
            }
            TextPredicateCapture::MatchString(i, r, is_positive, match_all_nodes) => {
                let nodes = self.nodes_for_capture_index(*i);
                for node in nodes {
                    let text = node.text(text_provider);
                    let text = text.as_bytes();
                    let is_positive_match = r.is_match(text);
                    if is_positive_match != *is_positive && *match_all_nodes {
                        return false;
                    }
                    if is_positive_match == *is_positive && !*match_all_nodes {
                        return true;
                    }
                }
                true
            }
            TextPredicateCapture::AnyString(i, v, is_positive) => {
                let nodes = self.nodes_for_capture_index(*i);
                for node in nodes {
                    let text = node.text(text_provider);
                    let text = text.as_bytes();
                    if (v.iter().any(|s| text == s.as_ref().as_bytes())) != *is_positive {
                        return false;
                    }
                }
                true
            }
        })
    }

    pub fn nodes_for_capture_index<'a>(
        &'a self,
        index: CaptureId,
    ) -> impl Iterator<Item = &'a Node> {
        self.captures.nodes_for_capture_index(index)
    }
}

#[derive(Default)]
pub struct PreparedQuerying<Q, HAST, Acc>(Q, std::marker::PhantomData<(HAST, Acc)>);

impl<'a, HAST, Acc> From<&'a crate::Query> for PreparedQuerying<&'a crate::Query, HAST, Acc> {
    fn from(value: &'a crate::Query) -> Self {
        Self(value, Default::default())
    }
}

impl<Q, HAST, Acc> Deref for PreparedQuerying<Q, HAST, &Acc> {
    type Target = Q;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, HAST, Acc> hyper_ast::tree_gen::Prepro<T> for PreparedQuerying<&crate::Query, HAST, Acc> {
    const USING: bool = false;

    fn preprocessing(&self, ty: T) -> Result<hyper_ast::scripting::Acc, String> {
        unimplemented!()
    }
}

impl<'s, TS, T, Acc> hyper_ast::tree_gen::More for PreparedQuerying<&crate::Query, (TS, T), Acc>
where
    TS: 'static
        + Clone
        + hyper_ast::types::ETypeStore<Ty2 = Acc::Type>
        + hyper_ast::types::RoleStore<IdF = u16, Role = hyper_ast::types::Role>,
    T: hyper_ast::types::WithRoles,
    T: hyper_ast::types::Tree,
    T::TreeId: Copy,
    Acc: hyper_ast::types::Typed
        + hyper_ast::tree_gen::WithRole<hyper_ast::types::Role>
        + hyper_ast::tree_gen::WithChildren<T::TreeId>,
    for<'c> &'c Acc: hyper_ast::tree_gen::WithLabel<L = &'c str>,
{
    type Acc = Acc;
    type T = T;
    type TS = TS;
    const ENABLED: bool = true;
    fn match_precomp_queries<
        'a,
        HAST: HyperAST<
                'a,
                IdN = <Self::T as hyper_ast::types::Stored>::TreeId,
                TS = Self::TS,
                T = Self::T,
            > + std::clone::Clone,
    >(
        &self,
        stores: HAST,
        acc: &Acc,
        label: Option<&str>,
    ) -> hyper_ast::tree_gen::PrecompQueries {
        if self.0.enabled_pattern_count() == 0 {
            return Default::default();
        }
        let pos = hyper_ast::position::StructuralPosition::empty();

        let cursor = crate::cursor_on_unbuild::TreeCursor::new(stores, acc, label, pos);
        let qcursor = self.0.matches_immediate(cursor); // TODO filter on height (and visibility?)
        let mut r = Default::default();
        for m in qcursor {
            assert!(m.pattern_index.to_usize() < 16);
            r |= 1 << m.pattern_index.to_usize() as u16;
        }
        r
    }
}

impl<'s, TS, T, Acc> hyper_ast::tree_gen::PreproTSG<'s>
    for PreparedQuerying<&crate::Query, (TS, T), Acc>
where
    TS: 'static
        + Clone
        + hyper_ast::types::ETypeStore<Ty2 = Acc::Type>
        + hyper_ast::types::RoleStore<IdF = u16, Role = hyper_ast::types::Role>,
    T: hyper_ast::types::WithRoles,
    T: hyper_ast::types::Tree,
    T::TreeId: Copy,
    Acc: hyper_ast::types::Typed
        + hyper_ast::tree_gen::WithRole<hyper_ast::types::Role>
        + hyper_ast::tree_gen::WithChildren<T::TreeId>,
    for<'c> &'c Acc: hyper_ast::tree_gen::WithLabel<L = &'c str>,
{
    const GRAPHING: bool = false;
    fn compute_tsg<
        HAST: HyperAST<
                's,
                IdN = <Self::T as hyper_ast::types::Stored>::TreeId,
                TS = Self::TS,
                T = Self::T,
            > + std::clone::Clone,
    >(
        &self,
        _stores: HAST,
        _acc: &Acc,
        _label: Option<&str>,
    ) -> Result<(), String> {
        Ok(())
    }
}

pub struct PreparedOverlay<Q, O> {
    pub query: Q,
    pub overlayer: O,
    pub functions: std::sync::Arc<dyn std::any::Any>,
}

#[cfg(feature = "tsg")]
impl<'aaa, 'hast, 'g, 'q, 'm, HAST, Acc> hyper_ast::tree_gen::More
    for PreparedOverlay<
        &'q crate::Query,
        &'m tree_sitter_graph::ast::File<stepped_query_imm::QueryMatcher<'hast, HAST, &Acc>>,
    >
where
    HAST: 'hast + HyperAST<'hast> + Clone,
    HAST::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: std::hash::Hash,
    HAST::T: hyper_ast::types::WithSerialization
        + hyper_ast::types::WithStats
        + hyper_ast::types::WithRoles,
    HAST::TS: 'static
        + Clone
        + hyper_ast::types::ETypeStore<Ty2 = Acc::Type>
        + hyper_ast::types::RoleStore<IdF = u16, Role = hyper_ast::types::Role>,
    Acc: hyper_ast::tree_gen::WithRole<hyper_ast::types::Role>
        + hyper_ast::tree_gen::WithChildren<HAST::IdN>
        + hyper_ast::types::Typed,
    for<'acc> &'acc Acc: hyper_ast::tree_gen::WithLabel<L = &'acc str>,
{
    type TS = HAST::TS;
    type Acc = Acc;
    type T = HAST::T;

    const ENABLED: bool = true;

    fn match_precomp_queries<
        'a,
        HAST2: HyperAST<
                'a,
                IdN = <Self::T as hyper_ast::types::Stored>::TreeId,
                TS = Self::TS,
                T = Self::T,
            > + std::clone::Clone,
    >(
        &self,
        stores: HAST2,
        acc: &Acc,
        label: Option<&str>,
    ) -> hyper_ast::tree_gen::PrecompQueries {
        // self.query.match_precomp_queries(stores, acc, label)
        if self.query.enabled_pattern_count() == 0 {
            return Default::default();
        }
        let pos = hyper_ast::position::StructuralPosition::empty();
        let cursor = crate::cursor_on_unbuild::TreeCursor::new(stores, acc, label, pos);
        let qcursor = self.query.matches_immediate(cursor); // TODO filter on height (and visibility?)
        let mut r = Default::default();
        for m in qcursor {
            assert!(m.pattern_index.to_usize() < 16);
            r |= 1 << m.pattern_index.to_usize() as u16;
        }
        r
    }
}

#[cfg(feature = "tsg")]
impl<'aaa, 'g, 'q, 'm, 'hast, HAST, Acc>
    hyper_ast::tree_gen::Prepro<<HAST::TS as hyper_ast::types::ETypeStore>::Ty2>
    for PreparedOverlay<
        &'q crate::Query,
        &'m tree_sitter_graph::ast::File<stepped_query_imm::QueryMatcher<'hast, HAST, &Acc>>,
    >
where
    HAST: 'hast + HyperAST<'hast> + Clone,
    HAST::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: std::hash::Hash,
    HAST::T: hyper_ast::types::WithSerialization
        + hyper_ast::types::WithStats
        + hyper_ast::types::WithRoles,
    HAST::TS: 'static
        + Clone
        + hyper_ast::types::ETypeStore<Ty2 = Acc::Type>
        + hyper_ast::types::RoleStore<IdF = u16, Role = hyper_ast::types::Role>
        + hyper_ast::types::TypeStore,
    Acc: hyper_ast::types::Typed
        + hyper_ast::tree_gen::WithRole<hyper_ast::types::Role>
        + hyper_ast::tree_gen::WithChildren<HAST::IdN>
        + hyper_ast::types::Typed,
    for<'acc> &'acc Acc: hyper_ast::tree_gen::WithLabel<L = &'acc str>,
{
    const USING: bool = false;

    fn preprocessing(
        &self,
        _ty: <HAST::TS as hyper_ast::types::ETypeStore>::Ty2,
    ) -> Result<hyper_ast::scripting::Acc, String> {
        unimplemented!()
    }
}

#[cfg(feature = "tsg")]
impl<'aaa, 'g, 'q, 'm, 'hast, HAST, Acc> hyper_ast::tree_gen::PreproTSG<'hast>
    for PreparedOverlay<
        &'q crate::Query,
        &'m tree_sitter_graph::ast::File<stepped_query_imm::QueryMatcher<'hast, HAST, &Acc>>,
    >
where
    HAST: 'hast + HyperAST<'hast> + Clone,
    HAST::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: std::hash::Hash,
    HAST::T: hyper_ast::types::WithSerialization
        + hyper_ast::types::WithStats
        + hyper_ast::types::WithRoles,
    HAST::TS: 'static
        + Clone
        + hyper_ast::types::ETypeStore<Ty2 = Acc::Type>
        + hyper_ast::types::RoleStore<IdF = u16, Role = hyper_ast::types::Role>
        + hyper_ast::types::TypeStore,
    Acc: hyper_ast::types::Typed
        + 'static
        + hyper_ast::tree_gen::WithRole<hyper_ast::types::Role>
        + hyper_ast::tree_gen::WithChildren<HAST::IdN>
        + hyper_ast::types::Typed,
    for<'acc> &'acc Acc: hyper_ast::tree_gen::WithLabel<L = &'acc str>,
{
    const GRAPHING: bool = true;
    fn compute_tsg<
        HAST2: 'static
            + HyperAST<
                'hast,
                IdN = <Self::T as hyper_ast::types::Stored>::TreeId,
                Idx = <Self::T as hyper_ast::types::WithChildren>::ChildIdx,
                TS = Self::TS,
                T = Self::T,
            >
            + std::clone::Clone,
    >(
        &self,
        stores: HAST2,
        acc: &Acc,
        label: Option<&str>,
    ) -> Result<(), String> {
        // NOTE I had to do a lot of unsafe magic :/
        // mostly exending lifetime and converting HAST to HAST2 on compatible structures

        use tree_sitter_graph::graph::Graph;
        let cancellation_flag = tree_sitter_graph::NoCancellation;
        let mut globals = tree_sitter_graph::Variables::new();
        let mut graph: Graph<stepped_query_imm::Node<'_, HAST2, &Acc>> =
            tree_sitter_graph::graph::Graph::default();
        init_globals(&mut globals, &mut graph);

        // retreive the custom functions
        // NOTE need the concrete type of the stores to instanciate
        // WARN cast will fail if the original instance type was not identical
        type Fcts<HAST, Acc> = tree_sitter_graph::functions::Functions<
            tree_sitter_graph::graph::GraphErazing<stepped_query_imm::MyNodeErazing<HAST, Acc>>,
        >;
        let functions: &Fcts<HAST2, &Acc> = self
            .functions
            .deref()
            .downcast_ref()
            .expect("identical instance type");

        // tree_sitter_stack_graphs::functions::add_path_functions(&mut functions);

        let mut config = configure(&globals, functions);

        let pos = hyper_ast::position::StructuralPosition::empty();
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
            stepped_query_imm::MyQMatches::<_, HAST2, _> {
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
                    c: &(),
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
                    .nodes_for_capture_index(stanza.full_match_file_capture_index)
                    .next()
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
                    if let Err(err) = ctx.exec(
                        graph,
                        inherited_variables,
                        cancellation_flag,
                        stanza.full_match_file_capture_index,
                        shorthands,
                        mat,
                        config,
                        &current_regex_captures,
                        &statement,
                        error_context,
                    ) {
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
            log::error!("curr kind {}", hyper_ast::types::Typed::get_type(acc));
            let s = graph.to_json().unwrap();
            log::error!("graph: {}", s);
        }
        Ok(())
    }
}

pub use tree_sitter_stack_graphs::functions::add_path_functions;

static DEBUG_ATTR_PREFIX: &'static str = "debug_";
pub static ROOT_NODE_VAR: &'static str = "ROOT_NODE";
/// The name of the file path global variable
pub const FILE_PATH_VAR: &str = "FILE_PATH";
static JUMP_TO_SCOPE_NODE_VAR: &'static str = "JUMP_TO_SCOPE_NODE";
static FILE_NAME: &str = "a/b/AAA.java";

#[cfg(feature = "tsg")]
fn configure<'a, 'g, Node>(
    globals: &'a tree_sitter_graph::Variables<'g>,
    functions: &'a tree_sitter_graph::functions::Functions<
        tree_sitter_graph::graph::GraphErazing<Node>,
    >,
) -> tree_sitter_graph::ExecutionConfig<'a, 'g, tree_sitter_graph::graph::GraphErazing<Node>> {
    let config = tree_sitter_graph::ExecutionConfig::new(functions, globals)
        .lazy(true);
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
fn init_globals<Node: tree_sitter_graph::graph::SyntaxNodeExt>(
    _globals: &mut tree_sitter_graph::Variables,
    _graph: &mut tree_sitter_graph::graph::Graph<Node>,
) {
    // globals
    //     .add(ROOT_NODE_VAR.into(), graph.add_graph_node().into())
    //     .expect("Failed to set ROOT_NODE");
    // globals
    //     .add(FILE_PATH_VAR.into(), FILE_NAME.into())
    //     .expect("Failed to set FILE_PATH");
    // globals
    //     .add(JUMP_TO_SCOPE_NODE_VAR.into(), graph.add_graph_node().into())
    //     .expect("Failed to set JUMP_TO_SCOPE_NODE");
}
