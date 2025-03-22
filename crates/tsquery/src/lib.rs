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
#[cfg(feature = "hyperast")]
pub mod hyperast_cursor;
#[cfg(feature = "hyperast")]
pub mod hyperast_opt;
pub mod stepped_query;
pub mod stepped_query_imm;
pub mod tsg;

pub mod cursor_on_unbuild;

use hyperast::types::HyperAST;
use hyperast::types::NLending;
pub use tree_sitter::CaptureQuantifier;
pub use tree_sitter::Language;
pub use tree_sitter::Point;
pub use tree_sitter::QueryError;
pub use tree_sitter::QueryErrorKind;
pub(crate) use ts_private_bypass::*;

pub use indexed::CaptureId;
pub use indexed::PatternId;
pub use indexed::Symbol;

use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::Deref;
use std::{collections::VecDeque, usize};

pub use stepped_query_imm::ExtendingStringQuery;
pub use stepped_query_imm::MyNodeErazing;
pub use stepped_query_imm::QueryMatcher;

pub use utils::ArrayStr;
pub use utils::ZeroSepArrayStr;

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
    pub fn matches<'query, Cursor: self::Cursor, Node: crate::Node + Clone>(
        &'query self,
        cursor: Cursor,
    ) -> QueryCursor<'query, Cursor, Node> {
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
    pub fn matches_immediate<'query, Cursor: self::Cursor, N: crate::Node + Clone>(
        &'query self,
        cursor: Cursor,
    ) -> QueryCursor<'query, Cursor, N> {
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
    pub halted: bool,
    pub ascending: bool,
    pub on_visible_node: bool,
    pub cursor: Cursor,
    pub query: &'query Query,
    pub states: Vec<query_cursor::State>,
    pub depth: Depth,
    pub max_start_depth: Depth,
    pub capture_list_pool: indexed::CaptureListPool<Node>,
    pub finished_states: VecDeque<query_cursor::State>,
    pub next_state_id: indexed::StateId,
    // only triggers when there is no more capture list available
    // not triggered by reaching max_start_depth
    pub did_exceed_match_limit: bool,
}

impl<'query, Cursor, N> QueryCursor<'query, Cursor, N> {
    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }
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

// pub trait CursorLending<'a> {
//     type NodeRef: Node<IdF = <Self::Node as Node>::IdF, TP<'a> = <Self::Node as Node>::TP<'a>>;
// }

pub trait WithField {
    type IdF;
}
pub trait CNLending<'a, __ImplBound = &'a Self>: WithField {
    type NR: Node<IdF = Self::IdF> + Clone;
}

pub trait Cursor: for<'a> CNLending<'a> {
    type Node: Clone
        + Node<IdF = <Self as WithField>::IdF>
        + for<'a> TextLending<'a, TP = <<Self as CNLending<'a>>::NR as TextLending<'a>>::TP>;

    fn goto_next_sibling_internal(&mut self) -> TreeCursorStep;

    fn goto_first_child_internal(&mut self) -> TreeCursorStep;

    fn goto_parent(&mut self) -> bool;
    fn current_node(&self) -> <Self as CNLending<'_>>::NR;

    fn parent_is_error(&self) -> bool;

    type Status: Status<IdF = <Self::Node as Node>::IdF>;

    fn current_status(&self) -> Self::Status;

    fn text_provider(&self) -> <Self::Node as TextLending<'_>>::TP;

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

pub trait TextLending<'a> {
    type TP: Copy;
}
pub trait Node: for<'a> TextLending<'a> {
    type IdF;

    fn symbol(&self) -> Symbol;

    fn is_named(&self) -> bool;
    fn str_symbol(&self) -> &str;

    fn start_point(&self) -> crate::Point;

    fn has_child_with_field_id(&self, field_id: Self::IdF) -> bool;

    fn equal(&self, other: &Self) -> bool;
    fn compare(&self, other: &Self) -> std::cmp::Ordering;
    fn text<'s, 'l>(&'s self, text_provider: <Self as TextLending<'l>>::TP) -> BB<'s, 'l, str>;
    fn text_equal<'s, 'l>(
        &'s self,
        text_provider: <Self as TextLending<'l>>::TP,
        other: impl Iterator<Item = u8>,
    ) -> bool {
        self.text(text_provider)
            .deref()
            .as_bytes()
            .iter()
            .copied()
            .eq(other)
    }
}

pub enum BB<'a, 'b, B: ?Sized + 'a + 'b>
where
    B: ToOwned,
{
    A(&'a B),
    B(&'b B),
    O(<B as ToOwned>::Owned),
}

impl<'a, 'b, B: ?Sized + 'a + 'b> std::ops::Deref for BB<'a, 'b, B>
where
    B: ToOwned,
    B::Owned: Borrow<B>,
{
    type Target = B;
    fn deref(&self) -> &Self::Target {
        match self {
            BB::A(s) => s,
            BB::B(s) => s,
            BB::O(s) => s.borrow(),
        }
    }
}

impl<'a, T> TextLending<'a> for &T
where
    T: TextLending<'a>,
{
    type TP = T::TP;
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

    fn text<'s, 'l>(&'s self, text_provider: <Self as TextLending<'l>>::TP) -> BB<'s, 'l, str> {
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
    pub(crate) fn satisfies_text_predicates<'a, 'b, 's: 'l, 'l, T: 'a + AsRef<str>>(
        &'s self,
        text_provider: <Node as TextLending<'l>>::TP,
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

mod precompute_pattern_predicate;
pub use precompute_pattern_predicate::PreparedQuerying;
mod graph_overlaying;
pub use graph_overlaying::PreparedOverlay;

// mod staged_graph {
//     use std::collections::VecDeque;
//     use std::collections::HashMap;
//     use tree_sitter_graph::graph::Erzd;
//     use tree_sitter_graph::graph::GraphNode;
//     use tree_sitter_graph::graph::LErazng;
//     use tree_sitter_graph::graph::SyntaxNodeRef;
//     use tree_sitter_graph::graph::GraphNodeRef;
//     use tree_sitter_graph::graph::SyntaxNodeExt;
//     use tree_sitter_graph::graph::WithOutGoingEdges;
//     use tree_sitter_graph::graph::WithAttrs;

//     use std::ops::IndexMut;
//     use std::ops::Index;

//     /// The one that exists while preprocessing a subtree
//     pub struct PartialGraph<S, N = GraphNode> {
//         pub(crate) syntax_nodes: HashMap<SyntaxNodeID, S>,
//         graph_nodes: VecDeque<(u32, N)>,
//     }

//     /// The one stored on the subtree, ie. immutable
//     /// TODO look at other variants
//     pub struct ImmGraph<S, N = GraphNode> {
//         pub(crate) syntax_node_ids: Box<[SyntaxNodeID]>,
//         pub(crate) syntax_nodes: Box<[S]>,
//         graph_node_ids: Box<[u32]>,
//         graph_nodes: Box<[N]>,
//         added: u8,
//         /// if added >0 then graph_node_ids[-1] == total
//         total: u32
//     }

//     /// Complete, but kind of append only.
//     /// mutating an attribute only shadows it
//     pub struct StagedGraph<S, N = GraphNode> {
//         pub(crate) syntax_nodes: HashMap<SyntaxNodeID, S>,
//         graph_nodes: Vec<N>,
//         // hast: HAST,
//         // acc: Acc,
//     }

//     impl<S, N> Default for StagedGraph<S, N> {
//         fn default() -> Self {
//             Self {
//                 syntax_nodes: Default::default(),
//                 graph_nodes: Default::default(),
//             }
//         }
//     }

//     /// Should probably correspond to a topological id of some category of node,
//     /// eg. Identifier, TypeIdentifier, ClassDeclaration, Body, File
//     /// but not static, public, curly, paren, space
//     ///
//     /// the topo id is great to search through a recursive tree struct,
//     /// anyway must continue descending until attr is found at a certain subtree depth
//     pub(crate) type SyntaxNodeID = u32;
//     type GraphNodeID = u32;

//     impl<S, N> StagedGraph<S, N> {
//         /// Creates a new, empty graph.
//         pub fn new() -> Self {
//             StagedGraph::default()
//         }
//     }

//     pub trait WithSynNodes:
//         LErazng + Index<GraphNodeRef, Output = Self::Node> + IndexMut<GraphNodeRef, Output = Self::Node>
//     {
//         type Node: WithAttrs + Default + WithOutGoingEdges;
//         type SNode: SyntaxNodeExt + Clone;
//         fn node(&self, r: SyntaxNodeRef) -> Option<&Self::SNode>;

//         /// Adds a new graph node to the graph, returning a graph DSL reference to it.
//         fn add_graph_node(&mut self) -> GraphNodeRef;
//         fn add_syntax_node(&mut self, node: Self::SNode) -> SyntaxNodeRef;
//     }

//     pub struct GraphErazing<S>(std::marker::PhantomData<S>);

//     impl<S> Default for GraphErazing<S> {
//         fn default() -> Self {
//             Self(Default::default())
//         }
//     }

//     impl<S: Erzd> Erzd for GraphErazing<S> {
//         type Original<'a> = StagedGraph<S::Original<'a>>;
//     }

//     impl<S: LErazng, N> LErazng for StagedGraph<S, N> {
//         type LErazing = GraphErazing<S::LErazing>;
//     }

//     impl<S: LErazng + SyntaxNodeExt + Clone, N: WithAttrs + Default + WithOutGoingEdges> WithSynNodes for StagedGraph<S, N> {
//         type Node = N;
//         type SNode = S;

//         fn node(&self, r: SyntaxNodeRef) -> Option<&Self::SNode> {
//             todo!()
//             // self.syntax_nodes.get(&r.index)
//         }

//         fn add_graph_node(&mut self) -> GraphNodeRef {
//             self.add_graph_node()
//         }

//         fn add_syntax_node(&mut self, node: S) -> SyntaxNodeRef {
//             self.add_syntax_node(node)
//         }
//     }

//     impl<S: SyntaxNodeExt, N: Default> StagedGraph<S, N> {
//         /// Adds a syntax node to the graph, returning a graph DSL reference to it.
//         ///
//         /// The graph won't contain _every_ syntax node in the parsed syntax tree; it will only contain
//         /// those nodes that are referenced at some point during the execution of the graph DSL file.
//         pub fn add_syntax_node(&mut self, node: S) -> SyntaxNodeRef {
//             todo!()
//             // let index = node.id() as SyntaxNodeID;
//             // let index = index as SyntaxNodeID;
//             // let node_ref = SyntaxNodeRef {
//             //     index,
//             //     kind: node.kind(),
//             //     position: node.start_position(),
//             // };
//             // self.syntax_nodes.entry(index).or_insert(node);
//             // node_ref
//         }

//         /// Adds a new graph node to the graph, returning a graph DSL reference to it.
//         pub fn add_graph_node(&mut self) -> GraphNodeRef {
//             todo!()
//             // let graph_node = N::default();
//             // let index = self.graph_nodes.len() as GraphNodeID;
//             // self.graph_nodes.push(graph_node);
//             // GraphNodeRef(index)
//         }

//         // Returns an iterator of references to all of the nodes in the graph.
//         pub fn iter_nodes(&self) -> impl Iterator<Item = GraphNodeRef> {
//             vec![todo!()].into_iter()
//             // (0..self.graph_nodes.len() as u32).map(GraphNodeRef)
//         }

//         // Returns the number of nodes in the graph.
//         pub fn node_count(&self) -> usize {
//             self.graph_nodes.len()
//         }
//     }

//     impl<S, N> Index<SyntaxNodeRef> for StagedGraph<S, N> {
//         type Output = S;
//         fn index(&self, node_ref: SyntaxNodeRef) -> &S {
//             todo!()
//             // &self.syntax_nodes[&node_ref.index]
//         }
//     }

//     impl<S, N> Index<GraphNodeRef> for StagedGraph<S, N> {
//         type Output = N;
//         fn index(&self, index: GraphNodeRef) -> &N {
//             todo!()
//             // &self.graph_nodes[index.0 as usize]
//         }
//     }

//     impl<S, N> IndexMut<GraphNodeRef> for StagedGraph<S, N> {
//         fn index_mut(&mut self, index: GraphNodeRef) -> &mut N {
//             todo!()
//             // &mut self.graph_nodes[index.0 as usize]
//         }
//     }
// }
