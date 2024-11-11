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
pub mod tsg;

pub mod cursor_on_unbuild;

pub use tree_sitter::CaptureQuantifier;
pub use tree_sitter::Language;
pub use tree_sitter::Point;
pub use tree_sitter::QueryError;
pub use tree_sitter::QueryErrorKind;
pub(crate) use ts_private_bypass::*;

pub use indexed::CaptureId;
pub use indexed::PatternId;
pub use indexed::Symbol;

use std::{collections::VecDeque, usize};

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

use hyper_ast::store::nodes::legion::RawHAST;

impl<'a, 'b, TS, Acc>
    hyper_ast::tree_gen::More<RawHAST<'a, 'b, TS>, Acc>
    for crate::Query
where
    TS: hyper_ast::types::ETypeStore<Ty2 = Acc::Type>
        + hyper_ast::types::RoleStore<IdF = u16, Role = hyper_ast::types::Role>,
    Acc: hyper_ast::types::Typed
        + hyper_ast::tree_gen::WithRole<hyper_ast::types::Role>
        + hyper_ast::tree_gen::WithChildren<hyper_ast::store::nodes::legion::NodeIdentifier>,
{
    const ENABLED: bool = true;
    fn match_precomp_queries(
        &self,
        stores: &RawHAST<'a, 'b, TS>,
        acc: &Acc,
        label: Option<&str>,
    ) -> hyper_ast::tree_gen::PrecompQueries {
        let pos = hyper_ast::position::StructuralPosition::empty();
        let cursor = crate::cursor_on_unbuild::TreeCursor::new(stores, acc, label, pos);
        let qcursor = self.matches_immediate(cursor); // TODO filter on height (and visibility?)
        let mut r = Default::default();
        for m in qcursor {
            assert!(m.pattern_index.to_usize() < 16);
            r |= 1 << m.pattern_index.to_usize() as u16;
        }
        r
    }
}

impl<'a, 'b, TS, Acc>
    hyper_ast::tree_gen::More<RawHAST<'a, 'b, TS>, Acc>
    for &crate::Query
where
    TS: hyper_ast::types::ETypeStore<Ty2 = Acc::Type>
        + hyper_ast::types::RoleStore<IdF = u16, Role = hyper_ast::types::Role>,
    Acc: hyper_ast::types::Typed
        + hyper_ast::tree_gen::WithRole<hyper_ast::types::Role>
        + hyper_ast::tree_gen::WithChildren<hyper_ast::store::nodes::legion::NodeIdentifier>,
{
    const ENABLED: bool = true;
    fn match_precomp_queries(
        &self,
        stores: &RawHAST<'a, 'b, TS>,
        acc: &Acc,
        label: Option<&str>,
    ) -> hyper_ast::tree_gen::PrecompQueries {
        (*self).match_precomp_queries(stores, acc, label)
    }
}

// arc is messing with orphan rule stuff...
// impl<
//         'a,
//         'b,
//         TS,
//         Acc: hyper_ast::types::Typed
//             + hyper_ast::tree_gen::WithRole<hyper_ast::types::Role>
//             + hyper_ast::tree_gen::WithChildren<hyper_ast::store::nodes::legion::NodeIdentifier>,
//     > hyper_ast::tree_gen::More<RawHAST<'a, 'b, TS>, Acc>
//     for std::sync::Arc<crate::Query>
// where
//     TS: hyper_ast::types::ETypeStore<Ty2 = Acc::Type>
//         + hyper_ast::types::RoleStore<IdF = u16, Role = hyper_ast::types::Role>,
// {
//     const ENABLED: bool = true;
//     fn match_precomp_queries(
//         &self,
//         stores: &RawHAST<'a, 'b, TS>,
//         acc: &Acc,
//         label: Option<&str>,
//     ) -> hyper_ast::tree_gen::PrecompQueries {
//         let cursor = crate::cursor_on_unbuild::TreeCursor::new(
//             stores,
//             acc,
//             label,
//             hyper_ast::position::StructuralPosition::empty(),
//         );
//         let qcursor = self.matches_immediate(cursor); // TODO filter on height (and visibility?)
//         let mut r = Default::default();
//         for m in qcursor {
//             assert!(m.pattern_index.to_usize() < 16);
//             r |= 1 << m.pattern_index.to_usize() as u16;
//         }
//         r
//     }
// }
