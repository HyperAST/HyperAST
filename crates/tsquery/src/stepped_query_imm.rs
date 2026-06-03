//! Attempt to integrate another query matchers compatible with hyperast.
//! The query matcher used here is largely inspired by tree_sitter (query.c).
//! Trying to make this one applicable directly on subtrees, ie. immediated/shallow

use hyperast::{
    position::TreePathMut,
    tree_gen::{self, WithLabel},
    types::{
        self, ETypeStore, HyperAST, HyperASTShared, Role, RoleStore, WithRoles, WithSerialization,
        WithStats,
    },
};
use std::{fmt::Debug, vec};
#[cfg(feature = "tsg")]
use tree_sitter_graph::{
    MatchLender, MatchLending, MatchesLending, QueryWithLang,
    graph::{NNN, NodeLender, NodeLending, NodesLending},
};

use crate::{CaptureId, hyperast_cursor::NodeR};

impl<HAST: HyperASTShared, Acc: WithLabel, Idx, P: Clone> From<Node<HAST, Acc, Idx, P>>
    for NodeR<P>
{
    fn from(value: Node<HAST, Acc, Idx, P>) -> Self {
        let pos = value.0.pos.clone();
        Self { pos }
    }
}

#[cfg(feature = "tsg")]
impl<P: Clone + std::hash::Hash> tree_sitter_graph::graph::SimpleNode for NodeR<P> {
    fn id(&self) -> usize {
        use std::hash::Hasher;
        let mut hasher = std::hash::DefaultHasher::new();
        self.pos.hash(&mut hasher);
        hasher.finish() as usize
    }

    fn parent(&self) -> Option<Self>
    where
        Self: Sized,
    {
        let mut r = self.clone();
        todo!()
    }
}

#[repr(transparent)]
pub struct Node<
    HAST: HyperASTShared,
    Acc: WithLabel,
    Idx = <HAST as HyperASTShared>::Idx,
    P = hyperast::position::StructuralPosition<<HAST as HyperASTShared>::IdN, Idx>,
    L = <Acc as WithLabel>::L,
>(
    // NOTE actually a bad idea to directly wrap cursor_on_unbuild::Node,
    // the nodes go in tsg Graphs and by holding a reference to HAST it locks down everything
    // TODO find a way to extract the essentials from Node (to free Graph), the rest could be then part of the execution context.
    // Doing so will probably contribute to facilitating the staged storage of graph nodes and edges.
    pub crate::cursor_on_unbuild::Node<HAST, Acc, Idx, P, L>,
);
// pub use crate::cursor_on_unbuild::Node;

impl<'hast, 'acc, HAST: HyperASTShared + Clone, Acc> Clone for Node<HAST, &'acc Acc>
where
    &'acc Acc: WithLabel,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'hast, HAST: HyperASTShared, Acc: WithLabel> PartialEq for Node<HAST, Acc> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

type IdF = u16;

impl<'a, 'hast, 'acc, HAST: HyperAST, Acc> super::TextLending<'a> for self::Node<HAST, &'acc Acc>
where
    &'acc Acc: WithLabel,
{
    type TP = ();
}

impl<'hast, 'acc, HAST, Acc> crate::Node for self::Node<HAST, &'acc Acc>
where
    HAST: HyperAST + Copy,
    HAST::TS: ETypeStore<Ty2 = Acc::Type>,
    HAST::TS: hyperast::types::RoleStore<IdF = IdF, Role = Role>,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: hyperast::types::WithRoles,
    HAST::IdN: Copy,
    Acc: tree_gen::WithChildren<HAST::IdN> + tree_gen::WithRole<Role> + types::Typed,
    &'acc Acc: WithLabel,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    fn symbol(&self) -> crate::Symbol {
        self.0.symbol()
    }

    fn is_named(&self) -> bool {
        self.0.is_named()
    }

    fn str_symbol(&self) -> &str {
        self.0.str_symbol()
    }

    fn start_point(&self) -> tree_sitter::Point {
        self.0.start_point()
    }

    type IdF = <HAST::TS as hyperast::types::RoleStore>::IdF;

    // fn child_by_field_id(&self, field_id: Self::IdF) -> Option<Self> {
    //     // self.0.child_by_field_id(field_id).map(|x| Self(x))
    // }

    fn has_child_with_field_id(&self, field_id: Self::IdF) -> bool {
        self.0.has_child_with_field_id(field_id)
    }

    fn equal(&self, other: &Self, text_provider: <Self as super::TextLending<'_>>::TP) -> bool {
        self.0.equal(&other.0, text_provider)
    }

    fn compare(&self, other: &Self) -> std::cmp::Ordering {
        self.0.compare(&other.0)
    }

    fn text<'s, 'l>(
        &'s self,
        text_provider: <Self as super::TextLending<'l>>::TP,
    ) -> super::BiCow<'s, 'l, str> {
        self.0.text(text_provider)
    }
}

impl<'acc, HAST: HyperAST, Acc> crate::WithField for Node<HAST, &'acc Acc>
where
    &'acc Acc: WithLabel,
{
    type IdF = IdF;
}

impl<'a, 'hast, 'acc, HAST, Acc> crate::CNLending<'a> for Node<HAST, &'acc Acc>
where
    HAST: HyperAST + Copy,
    HAST::TS: ETypeStore<Ty2 = Acc::Type> + RoleStore<IdF = IdF, Role = Role>,
    HAST::IdN: Copy,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithRoles,
    Acc: hyperast::tree_gen::WithRole<Role>,
    Acc: hyperast::tree_gen::WithChildren<HAST::IdN>,
    Acc: hyperast::types::Typed,
    &'acc Acc: WithLabel,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    type NR = Self;
}

impl<'hast, 'acc, HAST, Acc> crate::Cursor for Node<HAST, &'acc Acc>
where
    HAST: HyperAST + Copy,
    HAST::TS: ETypeStore<Ty2 = Acc::Type> + RoleStore<IdF = IdF, Role = Role>,
    HAST::IdN: Copy,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithRoles,
    Acc: hyperast::tree_gen::WithRole<Role>,
    Acc: hyperast::tree_gen::WithChildren<HAST::IdN>,
    Acc: hyperast::types::Typed,
    &'acc Acc: WithLabel,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    type Node = Self;
    // type NodeRef<'a>
    //     = &'a Self
    // where
    //     Self: 'a;

    fn goto_next_sibling_internal(&mut self) -> crate::TreeCursorStep {
        self.0.goto_next_sibling_internal()
    }

    fn goto_first_child_internal(&mut self) -> crate::TreeCursorStep {
        self.0.goto_first_child_internal()
    }

    fn goto_parent(&mut self) -> bool {
        self.0.goto_parent()
    }

    fn current_node(&self) -> Self {
        self.clone()
    }

    fn parent_is_error(&self) -> bool {
        self.0.parent_is_error()
    }

    fn has_parent(&self) -> bool {
        let mut node = self.clone();
        node.goto_parent()
    }

    fn persist(&mut self) -> Self::Node {
        self.clone()
    }

    fn persist_parent(&mut self) -> Option<Self::Node> {
        let mut node = self.clone();
        node.goto_parent();
        Some(node)
    }

    type Status = crate::cursor_on_unbuild::CursorStatus<<Self as crate::Node>::IdF>;

    fn current_status(&self) -> Self::Status {
        self.0.current_status()
    }

    fn text_provider(&self) -> <Self::Node as crate::TextLending<'_>>::TP {
        self.0.text_provider()
    }

    fn is_visible_at_root(&self) -> bool {
        self.0.is_visible_at_root()
    }
}

impl<'hast, HAST: HyperASTShared, Acc: WithLabel> Node<HAST, Acc> {
    pub fn new(
        stores: HAST,
        acc: Acc,
        label: Option<Acc::L>,
        pos: hyperast::position::StructuralPosition<HAST::IdN, HAST::Idx>,
    ) -> Self {
        Self(crate::cursor_on_unbuild::Node::new(stores, acc, label, pos))
    }
}

pub struct MyNodeErazing<HAST, Acc>(std::marker::PhantomData<(HAST, Acc)>);
impl<'acc, HAST, Acc> Default for MyNodeErazing<HAST, &'acc Acc> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[cfg(feature = "tsg")]
impl<'acc, HAST: HyperASTShared, Acc: 'static> tree_sitter_graph::graph::Erzd
    for MyNodeErazing<HAST, &'acc Acc>
where
    &'acc Acc: WithLabel,
{
    type Original<'tree> = Node<HAST, &'acc Acc>;
}

#[cfg(feature = "tsg")]
impl<'acc, HAST: HyperASTShared, Acc: 'static> tree_sitter_graph::graph::LErazng
    for Node<HAST, &'acc Acc>
where
    &'acc Acc: WithLabel,
{
    type LErazing = MyNodeErazing<HAST, &'acc Acc>;
}

pub struct QueryMatcher<TS, Acc> {
    pub query: crate::Query,
    _phantom: std::marker::PhantomData<(TS, Acc)>,
}

impl<TS, Acc> QueryMatcher<TS, Acc> {
    fn new(
        source: &str,
        language: &tree_sitter::Language,
    ) -> Result<Self, tree_sitter::QueryError> {
        let query = crate::Query::new(source, language.clone())?;

        Ok(Self {
            query,
            _phantom: std::marker::PhantomData,
        })
    }
    fn with_precomputed(
        source: &str,
        language: &tree_sitter::Language,
        precomputeds: impl crate::utils::ArrayStr,
    ) -> Result<Self, tree_sitter::QueryError> {
        let query = crate::Query::with_precomputed(source, language.clone(), precomputeds)?.1;

        Ok(Self {
            query,
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<TS, Acc> Debug for QueryMatcher<TS, Acc> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.query)
    }
}

#[cfg(feature = "tsg")]
impl<'acc, TS, Acc> tree_sitter_graph::QueryWithLang for QueryMatcher<TS, &'acc Acc> {
    type Lang = tree_sitter::Language;
    type I = CaptureId;
}

#[cfg(feature = "tsg")]
impl<'a, 'acc, HAST, Acc> NodeLending<'a> for QueryMatcher<HAST, &'acc Acc>
where
    // HAST: types::StoreLending<'a, __ImplBound>,
    HAST: types::HyperAST + Copy,
    HAST::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: Copy + std::hash::Hash,
    // <HAST as types::StoreLending<'a, __ImplBound>>::S: types::AstLending<'a, __ImplBound>,
    // <<HAST as types::StoreLending<'a, __ImplBound>>::S as hyperast::types::AstLending<
    //     'a,
    //     __ImplBound,
    // >>::RT: WithSerialization + WithStats + WithRoles,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
        WithSerialization + WithStats + WithRoles,
    HAST::TS: ETypeStore<Ty2 = Acc::Type> + hyperast::types::RoleStore<IdF = IdF, Role = Role>,
    Acc: hyperast::tree_gen::WithRole<Role>,
    Acc: hyperast::tree_gen::WithChildren<HAST::IdN>,
    Acc: hyperast::types::Typed,
    &'acc Acc: WithLabel,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    type Node = Node<HAST, &'acc Acc>;
}

#[cfg(feature = "tsg")]
impl<'a, 'acc, 'hast, HAST, Acc> MatchesLending<'a> for QueryMatcher<HAST, &'acc Acc>
where
    HAST: types::HyperAST + Copy,
    // HAST: for<'t> types::StoreLending<'t, __ImplBound>,
    // HAST: types::StoreLending<'a, __ImplBound>,
    HAST::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: Copy + std::hash::Hash,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
        WithSerialization + WithStats + WithRoles,
    // for<'t> <<HAST as types::StoreLending<'t>>::S as hyperast::types::AstLending<
    //     'a,
    //     __ImplBound,
    // >>::RT: WithSerialization + WithStats + WithRoles,
    HAST::TS: ETypeStore<Ty2 = Acc::Type> + hyperast::types::RoleStore<IdF = IdF, Role = Role>,
    Acc: hyperast::tree_gen::WithRole<Role>,
    Acc: hyperast::tree_gen::WithChildren<HAST::IdN>,
    Acc: hyperast::types::Typed,
    &'acc Acc: WithLabel,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    type Matches = MyQMatches<
        'a,
        'a,
        crate::QueryCursor<'a, <Self as NodeLending<'a>>::Node, <Self as NodeLending<'a>>::Node>,
        HAST,
        &'acc Acc,
    >;
}

#[cfg(feature = "tsg")]
impl<'acc, HAST, Acc> tree_sitter_graph::GenQuery for QueryMatcher<HAST, &'acc Acc>
where
    HAST: types::HyperAST + Copy,
    // HAST: HyperAST + for<'t> types::StoreLending<'t>,
    HAST::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: Copy + std::hash::Hash,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
        WithSerialization + WithStats + WithRoles,
    HAST::TS: ETypeStore<Ty2 = Acc::Type> + hyperast::types::RoleStore<IdF = IdF, Role = Role>,
    Acc: hyperast::tree_gen::WithRole<Role>,
    Acc: hyperast::tree_gen::WithChildren<HAST::IdN>,
    Acc: hyperast::types::Typed,
    &'acc Acc: WithLabel,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    // type Lang = tree_sitter::Language;

    type Ext = ExtendingStringQuery<Self, Self::Lang>;

    fn pattern_count(&self) -> usize {
        self.query.enabled_pattern_count()
    }

    fn capture_index_for_name(&self, name: &str) -> Option<u32> {
        self.query
            .capture_index_for_name(name)
            .map(|x| x.to_usize() as u32)
    }

    fn capture_quantifiers(
        &self,
        index: usize,
    ) -> impl std::ops::Index<usize, Output = tree_sitter::CaptureQuantifier> {
        let index = self
            .query
            .enabled_pattern_map
            .iter()
            .position(|x| *x as usize == index)
            .unwrap();
        self.query.capture_quantifiers(index)
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

    type Cursor = Vec<u16>;

    fn matches<'a>(
        &self,
        cursor: &mut Self::Cursor,
        node: &<Self as NodeLending<'a>>::Node,
    ) -> <Self as tree_sitter_graph::MatchesLending<'a>>::Matches {
        let matchs = self
            .query
            .matches::<_, <Self as NodeLending<'_>>::Node>(node.clone());
        // let matchs = self.query.matches_immediate(node.clone());
        // TODO find a way to avoid transmuting
        let node = node.clone();
        let node = unsafe { std::mem::transmute(node) };
        let matchs = unsafe { std::mem::transmute(matchs) };
        let q = unsafe { std::mem::transmute(self) };
        let cursor = unsafe { std::mem::transmute(cursor) };
        MyQMatches {
            q,
            cursor,
            matchs,
            node,
        }
    }
    // fn matches<'query, 'cursor: 'query, 'tree: 'cursor>(
    //     &'query self,
    //     cursor: &'cursor mut Self::Cursor,
    //     node: &Node<HAST, &'acc Acc>,
    // ) -> self::MyQMatches<
    //     'query,
    //     'cursor,
    //     'hast,
    //     crate::QueryCursor<'query, Self::Node<'tree>, Self::Node<'tree>>,
    //     HAST,
    //     &'acc Acc,
    // > {
    //     let _ = cursor;
    //     let _ = node;
    //     unimplemented!("try resolve the issue of lifetime of `node` that may not live long enough")
    //     // let matchs = self.query.matches_immediate(node.clone());
    //     // let node = node.clone();
    //     // self::MyQMatches {
    //     //     q: self,
    //     //     cursor,
    //     //     matchs,
    //     //     node,
    //     // }
    // }

    fn from_str(
        language: Self::Lang,
        source: &str,
    ) -> Result<tree_sitter_graph::ast::File<Self>, tree_sitter_graph::ParseError>
    where
        Self: Sized,
    {
        let mut file = tree_sitter_graph::ast::File::<Self>::new(language);
        tree_sitter_graph::parser::Parser::<Self::Ext>::new(source).parse_into_file(&mut file)?;
        Self::check(&mut file)?;
        Ok(file)
    }
}

pub struct ExtendingStringQuery<Q = tree_sitter::Query, L = tree_sitter::Language> {
    pub(crate) query: Option<Q>,
    pub(crate) acc: String,
    pub(crate) precomputeds: Option<Box<dyn crate::utils::ArrayStr>>,
    pub(crate) language: Option<L>,
}

impl<Q, L> ExtendingStringQuery<Q, L> {
    fn empty() -> Self {
        Self {
            query: Default::default(),
            acc: Default::default(),
            precomputeds: Default::default(),
            language: None,
        }
    }
    pub fn new(
        language: L,
        precomputeds: Box<dyn crate::utils::ArrayStr>,
        capacity: usize,
    ) -> Self {
        Self {
            acc: String::with_capacity(capacity),
            precomputeds: Some(precomputeds),
            language: Some(language),
            ..Self::empty()
        }
    }
}

#[cfg(feature = "tsg")]
impl<'acc, 'hast, HAST, Acc> tree_sitter_graph::ExtendedableQuery
    for ExtendingStringQuery<QueryMatcher<HAST, &'acc Acc>, tree_sitter::Language>
where
    HAST: HyperAST + Copy,
    HAST::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: Copy + std::hash::Hash,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
        WithSerialization + WithStats + WithRoles,
    HAST::TS: ETypeStore<Ty2 = Acc::Type> + hyperast::types::RoleStore<IdF = IdF, Role = Role>,
    Acc: hyperast::tree_gen::WithChildren<HAST::IdN>
        + hyperast::tree_gen::WithRole<Role>
        + hyperast::types::Typed,
    &'acc Acc: WithLabel,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    type Query = QueryMatcher<HAST, &'acc Acc>;
    type Lang = tree_sitter::Language;

    fn as_ref(&self) -> Option<&Self::Query> {
        self.query.as_ref()
    }

    fn with_capacity(capacity: usize) -> Self {
        let acc = String::with_capacity(capacity);
        Self {
            acc,
            ..Self::empty()
        }
    }

    fn make_query(
        &mut self,
        language: &Self::Lang,
        source: &str,
    ) -> Result<Self::Query, tree_sitter::QueryError> {
        if let Some(l) = &self.language {
            // this impl cannot accept different languages
            assert_eq!(language, l);
        }
        self.acc += source;
        self.acc += "\n";
        dbg!(source);
        // QueryMatcher::new(source, language)
        let precomputeds = self.precomputeds.as_deref().unwrap();
        QueryMatcher::with_precomputed(source, language, precomputeds)
    }

    fn make_main_query(&self, language: &Self::Lang) -> Self::Query {
        if let Some(l) = &self.language {
            // this impl cannot accept different languages
            // Moreover, given the existance of a main query, having multiple languages should be impossible.
            assert_eq!(language, l);
        }
        // QueryMatcher::new(&self.acc, language).unwrap()
        let precomputeds = self.precomputeds.as_deref().unwrap();
        QueryMatcher::with_precomputed(&self.acc, language, precomputeds).unwrap()
    }
}

#[cfg(feature = "tsg")]
impl<'hast, 'acc, 'l, HAST, Acc> tree_sitter_graph::graph::SimpleNode for Node<HAST, &'acc Acc>
where
    HAST: HyperAST + Copy,
    HAST::IdN: std::hash::Hash + Copy + Debug,
    HAST::Idx: std::hash::Hash,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
        WithSerialization + types::WithChildren + WithStats + WithRoles,
    HAST::TS: ETypeStore<Ty2 = Acc::Type> + hyperast::types::RoleStore<IdF = IdF, Role = Role>,
    Acc: hyperast::tree_gen::WithChildren<HAST::IdN>
        + hyperast::tree_gen::WithRole<Role>
        + hyperast::types::Typed,
    &'acc Acc: WithLabel,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    fn id(&self) -> usize {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::hash::DefaultHasher::new();
        self.0.pos.hash(&mut hasher);
        hasher.finish() as usize
    }

    fn parent(&self) -> Option<Self>
    where
        Self: Sized,
    {
        let mut r = self.clone();
        if r.0.pos.pop().is_some() {
            Some(r)
        } else {
            None
        }
    }
}

#[cfg(feature = "tsg")]
impl<'hast, 'acc, 'l, HAST, Acc> tree_sitter_graph::graph::SyntaxNode for Node<HAST, &'acc Acc>
where
    HAST: HyperAST + Copy,
    HAST::IdN: std::hash::Hash + Copy + Debug,
    HAST::Idx: std::hash::Hash,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
        WithSerialization + types::WithChildren + WithStats + WithRoles,
    HAST::TS: ETypeStore<Ty2 = Acc::Type> + hyperast::types::RoleStore<IdF = IdF, Role = Role>,
    Acc: hyperast::tree_gen::WithChildren<HAST::IdN>
        + hyperast::tree_gen::WithRole<Role>
        + hyperast::types::Typed,
    &'acc Acc: WithLabel,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    fn kind(&self) -> &'static str {
        use hyperast::types::HyperType;
        self.0.kind().as_static_str()
        // use hyperast::position::position_accessors::SolvedPosition;
        // let n = self.0.pos.node();
        // let n = self.0.stores.node_store.resolve(&n);
        // // TS::
        // let n = self.0.stores.type_store.resolve_type(&n);
        // n.as_static_str()
    }

    fn start_position(&self) -> tree_sitter::Point {
        // TODO compute the position
        // let conv =
        //     hyperast::position::PositionConverter::new(&self.0.pos).with_stores(&self.0.stores);

        // let conv: &hyperast::position::WithHyperAstPositionConverter<
        //     hyperast::position::StructuralPosition<_, _>,
        //     HAST,
        // > = unsafe { std::mem::transmute(&conv) };
        // let pos: hyperast::position::row_col::RowCol<usize> =
        //     conv.compute_pos_post_order::<_, hyperast::position::row_col::RowCol<usize>, HAST::IdN>();
        // // use hyperast::position::computing_offset_bottom_up::extract_position_it;
        // // let p = extract_position_it(self.stores, self.pos.iter());
        // tree_sitter::Point {
        //     row: pos.row() as usize, //p.range().start,
        //     column: pos.col() as usize,
        // }
        tree_sitter::Point { row: 0, column: 0 }
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
        use hyperast::position::TreePath;
        let stores: &HAST = &self.0.stores;
        if let Some(root) = self.0.pos.node() {
            hyperast::nodes::TextSerializer::new(stores, *root).to_string()
        } else {
            // log::error!("{}", self.kind());
            // use crate::Node;
            // self.0.text(())
            self.0
                .label
                .as_ref()
                .map_or("aaa", |x| x.as_ref())
                .to_string()
        }
    }

    fn named_child_count(&self) -> usize {
        todo!()
    }
}

#[cfg(feature = "tsg")]
impl<'acc, HAST, Acc> tree_sitter_graph::graph::SyntaxNodeExt for Node<HAST, &'acc Acc>
where
    HAST: HyperAST + Copy,
    HAST::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: Copy + std::hash::Hash,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
        WithSerialization + WithStats + WithRoles,
    HAST::TS: ETypeStore<Ty2 = Acc::Type> + hyperast::types::RoleStore<IdF = IdF, Role = Role>,
    Acc: hyperast::tree_gen::WithRole<Role>,
    Acc: hyperast::tree_gen::WithChildren<HAST::IdN>,
    Acc: hyperast::types::Typed,
    &'acc Acc: WithLabel,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    type Cursor = Vec<Self>;

    fn walk(&self) -> Self::Cursor {
        todo!()
    }

    fn named_children<'cursor>(
        &self,
        _cursor: &'cursor mut Self::Cursor,
    ) -> impl ExactSizeIterator<Item = Self>
    where
        Self: 'cursor,
    {
        #[allow(unreachable_code)]
        vec![todo!()].into_iter()
    }

    // type QM<'cursor>
    //     = MyQMatch<'cursor, 'hast, HAST, &'acc Acc>
    // where
    //     Self: 'cursor;
}

pub struct MyQMatch<
    'cursor,
    HAST: HyperASTShared,
    Acc: WithLabel,
    Idx = <HAST as HyperASTShared>::Idx,
    P = hyperast::position::StructuralPosition<<HAST as HyperASTShared>::IdN, Idx>,
    L = <Acc as WithLabel>::L,
> {
    pub stores: HAST,
    pub b: &'cursor (),
    pub qm: crate::QueryMatch<Node<HAST, Acc, Idx, P, L>>,
    pub i: u16,
}

#[cfg(feature = "tsg")]
impl<'cursor, HAST: HyperASTShared, Acc: WithLabel> QueryWithLang for MyQMatch<'cursor, HAST, Acc> {
    type Lang = tree_sitter::Language;
    type I = CaptureId;
}

pub struct CapturedNodesIter<
    'cursor,
    HAST: HyperASTShared,
    Acc: WithLabel,
    Idx = <HAST as HyperASTShared>::Idx,
    P = hyperast::position::StructuralPosition<<HAST as HyperASTShared>::IdN, Idx>,
> {
    stores: HAST,
    index: CaptureId,
    inner: &'cursor [crate::Capture<Node<HAST, Acc, Idx, P>>],
}

#[cfg(feature = "tsg")]
impl<'a, 'cursor, 'acc, HAST: HyperAST, Acc> NodeLending<'a>
    for CapturedNodesIter<'cursor, HAST, &'acc Acc>
where
    HAST: HyperAST + Copy,
    HAST::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: Copy + std::hash::Hash,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
        WithSerialization + WithStats + WithRoles,
    HAST::TS: ETypeStore<Ty2 = Acc::Type> + hyperast::types::RoleStore<IdF = IdF, Role = Role>,
    Acc: hyperast::tree_gen::WithRole<Role>,
    Acc: hyperast::tree_gen::WithChildren<HAST::IdN>,
    Acc: hyperast::types::Typed,
    &'acc Acc: WithLabel,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    type Node = Node<HAST, &'acc Acc>;
}

#[cfg(feature = "tsg")]
impl<'cursor, 'acc, HAST: HyperAST, Acc> NodeLender for CapturedNodesIter<'cursor, HAST, &'acc Acc>
where
    HAST: HyperAST + Copy,
    HAST::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: Copy + std::hash::Hash,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
        WithSerialization + WithStats + WithRoles,
    HAST::TS: ETypeStore<Ty2 = Acc::Type> + hyperast::types::RoleStore<IdF = IdF, Role = Role>,
    Acc: hyperast::tree_gen::WithRole<Role>,
    Acc: hyperast::tree_gen::WithChildren<HAST::IdN>,
    Acc: hyperast::types::Typed,
    &'acc Acc: WithLabel,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    fn next(&mut self) -> Option<<Self as NodeLending<'_>>::Node> {
        loop {
            if self.inner.is_empty() {
                return None;
            }
            let capture = &self.inner[0];
            self.inner = &self.inner[1..];
            if capture.index != self.index {
                continue;
            }
            let node = capture.node.clone();
            return Some(node);
        }
    }
}

#[cfg(feature = "tsg")]
impl<'a, 'cursor, 'acc, HAST, Acc> NodesLending<'a> for MyQMatch<'cursor, HAST, &'acc Acc>
where
    HAST: HyperAST + Copy,
    HAST::TS: ETypeStore<Ty2 = Acc::Type> + hyperast::types::RoleStore<IdF = IdF, Role = Role>,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: hyperast::types::WithRoles,
    HAST::IdN: Copy,
    Acc: tree_gen::WithChildren<HAST::IdN> + tree_gen::WithRole<Role> + types::Typed,
    &'acc Acc: WithLabel,
    // syn_node_ref
    HAST::IdN: std::hash::Hash + Debug,
    HAST::Idx: std::hash::Hash,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
        WithSerialization + types::WithChildren + WithStats,
    ////
    HAST: HyperAST + Copy,
    HAST::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: Copy + std::hash::Hash,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
        WithSerialization + WithStats + WithRoles,
    HAST::TS: ETypeStore<Ty2 = Acc::Type> + hyperast::types::RoleStore<IdF = IdF, Role = Role>,
    Acc: hyperast::tree_gen::WithRole<Role>,
    Acc: hyperast::tree_gen::WithChildren<HAST::IdN>,
    Acc: hyperast::types::Typed,
    &'acc Acc: WithLabel,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    type Nodes = CapturedNodesIter<'a, HAST, &'acc Acc>;
}

#[allow(type_alias_bounds)]
type Pos<HAST: HyperASTShared> = hyperast::position::StructuralPosition<
    <HAST as HyperASTShared>::IdN,
    <HAST as HyperASTShared>::Idx,
>;

#[cfg(feature = "tsg")]
impl<'cursor, 'acc, HAST, Acc> tree_sitter_graph::graph::QMatch
    for MyQMatch<'cursor, HAST, &'acc Acc>
where
    HAST: HyperAST + Copy,
    HAST::TS: ETypeStore<Ty2 = Acc::Type> + hyperast::types::RoleStore<IdF = IdF, Role = Role>,
    Acc: tree_gen::WithChildren<HAST::IdN> + tree_gen::WithRole<Role> + types::Typed,
    &'acc Acc: WithLabel,
    // syn_node_ref
    HAST::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: Copy + std::hash::Hash,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
        WithSerialization + WithStats + WithRoles,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    type Simple = NodeR<Pos<HAST>>;

    fn nodes_for_capture_index(&self, index: Self::I) -> <Self as NodesLending<'_>>::Nodes {
        CapturedNodesIter::<HAST, &'acc Acc> {
            stores: self.stores.clone(),
            index,
            inner: self.qm.captures.captures(),
        }
    }

    fn nodes_for_capture_indexi(&self, index: Self::I) -> Option<NNN<'_, '_, Self>> {
        CapturedNodesIter::<HAST, &'acc Acc> {
            stores: self.stores.clone(),
            index,
            inner: self.qm.captures.captures(),
        }
        .next()
    }

    fn nodes_for_capture_indexii(
        &self,
        index: Self::I,
    ) -> impl tree_sitter_graph::graph::NodeLender
    + tree_sitter_graph::graph::NodeLending<'_, Node = NNN<'_, '_, Self>> {
        CapturedNodesIter::<HAST, &'acc Acc> {
            stores: self.stores.clone(),
            index,
            inner: self.qm.captures.captures(),
        }
    }
    // fn nodes_for_capture_index(&self, index: Self::I) -> impl Iterator<Item = Self::Item> {
    //     self.qm
    //         .nodes_for_capture_index(CaptureId::new(index))
    //         .cloned()
    // }

    fn pattern_index(&self) -> usize {
        self.i as usize
    }

    fn syn_node_ref(&self, node: &NNN<'_, '_, Self>) -> tree_sitter_graph::graph::SyntaxNodeRef {
        todo!()
    }
    // fn syn_node_ref(&self, node: &Self::Item) -> tree_sitter_graph::graph::SyntaxNodeRef {
    //     tree_sitter_graph::graph::SyntaxNodeRef::new(node)
    // }
    fn node(&self, s: Self::Simple) -> NNN<'_, '_, Self> {
        todo!()
    }
}

pub struct MyQMatches<
    'query,
    'cursor,
    It,
    HAST: HyperASTShared,
    Acc: WithLabel,
    Idx = <HAST as HyperASTShared>::Idx,
    P = hyperast::position::StructuralPosition<<HAST as HyperASTShared>::IdN, Idx>,
    L = <Acc as WithLabel>::L,
> {
    pub(crate) q: &'query QueryMatcher<HAST, Acc>,
    pub(crate) cursor: &'cursor mut Vec<u16>,
    pub(crate) matchs: It,
    pub(crate) node: Node<HAST, Acc, Idx, P, L>,
}

#[cfg(feature = "tsg")]
impl<'query, 'cursor, 'acc, It, HAST, Acc> QueryWithLang
    for MyQMatches<'query, 'cursor, It, HAST, &'acc Acc>
where
    HAST: HyperAST + Copy,
    It: Iterator<Item = crate::QueryMatch<Node<HAST, &'acc Acc>>>,
    &'acc Acc: WithLabel,
{
    type Lang = tree_sitter::Language;
    type I = CaptureId;
}

#[cfg(feature = "tsg")]
impl<'a, 'query, 'cursor, 'acc, It, HAST, Acc> MatchLending<'a>
    for MyQMatches<'query, 'cursor, It, HAST, &'acc Acc>
where
    HAST: HyperAST + Copy,
    It: Iterator<Item = crate::QueryMatch<Node<HAST, &'acc Acc>>>,
    &'acc Acc: WithLabel,
    HAST: HyperAST + Copy,
    HAST::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: Copy + std::hash::Hash,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
        WithSerialization + WithStats + WithRoles,
    HAST::TS: ETypeStore<Ty2 = Acc::Type> + hyperast::types::RoleStore<IdF = IdF, Role = Role>,
    Acc: hyperast::tree_gen::WithRole<Role>,
    Acc: hyperast::tree_gen::WithChildren<HAST::IdN>,
    Acc: hyperast::types::Typed,
    &'acc Acc: WithLabel,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    type Match = self::MyQMatch<'cursor, HAST, &'acc Acc>;
}

#[cfg(feature = "tsg")]
impl<'query, 'cursor, 'acc, It, HAST, Acc> MatchLender
    for MyQMatches<'query, 'cursor, It, HAST, &'acc Acc>
where
    HAST: HyperAST + Copy,
    It: Iterator<Item = crate::QueryMatch<Node<HAST, &'acc Acc>>>,
    &'acc Acc: WithLabel,
    HAST: HyperAST + Copy,
    HAST::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: Copy + std::hash::Hash,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT:
        WithSerialization + WithStats + WithRoles,
    HAST::TS: ETypeStore<Ty2 = Acc::Type> + hyperast::types::RoleStore<IdF = IdF, Role = Role>,
    Acc: hyperast::tree_gen::WithRole<Role>,
    Acc: hyperast::tree_gen::WithChildren<HAST::IdN>,
    Acc: hyperast::types::Typed,
    &'acc Acc: WithLabel,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    fn next(&mut self) -> Option<<Self as MatchLending<'_>>::Match> {
        let qm = self.matchs.next()?;
        let stores = self.node.0.stores.clone();
        let i = self
            .q
            .query
            .enabled_pattern_index(qm.pattern_index)
            .unwrap();
        Some(self::MyQMatch {
            stores,
            b: &&(),
            qm,
            i,
        })
    }
}

impl<'query, 'cursor, 'acc, It, HAST, Acc> Iterator
    for MyQMatches<'query, 'cursor, It, HAST, &'acc Acc>
where
    HAST: HyperAST + Copy,
    It: Iterator<Item = crate::QueryMatch<Node<HAST, &'acc Acc>>>,
    &'acc Acc: WithLabel,
{
    type Item = self::MyQMatch<'cursor, HAST, &'acc Acc>;

    fn next(&mut self) -> Option<Self::Item> {
        let qm = self.matchs.next()?;
        let stores = self.node.0.stores.clone();
        let i = self
            .q
            .query
            .enabled_pattern_index(qm.pattern_index)
            .unwrap();
        Some(self::MyQMatch {
            stores,
            b: &&(),
            qm,
            i,
        })
    }
}
