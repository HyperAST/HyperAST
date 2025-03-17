//! Attempt to integrate another query matchers compatible with hyperast.
//! The query matcher used here is largely inspired by tree_sitter (query.c).

use hyperast::types::{
    HyperAST, HyperASTShared, RoleStore, WithRoles, WithSerialization, WithStats,
};
use hyperast_gen_ts_tsquery::search::steped;
use std::fmt::Debug;
use tree_sitter_graph::GenQuery;

#[repr(transparent)]
pub struct Node<'tree, HAST: HyperASTShared>(steped::hyperast::Node<'tree, HAST>);

impl<'tree, HAST: HyperAST<'tree>> Clone for Node<'tree, HAST> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'tree, HAST: HyperAST<'tree>> PartialEq for Node<'tree, HAST> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'tree, HAST: HyperAST<'tree>> steped::Node for Node<'tree, HAST>
where
    HAST::IdN: std::fmt::Debug + Copy,
    HAST::TS: RoleStore,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithRoles,
{
    fn symbol(&self) -> steped::Symbol {
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

    type IdF = <HAST::TS as RoleStore>::IdF;

    // fn child_by_field_id(&self, field_id: Self::IdF) -> Option<Self> {
    //     // self.0.child_by_field_id(field_id).map(|x| Self(x))
    // }

    fn has_child_with_field_id(&self, field_id: Self::IdF) -> bool {
        self.0.has_child_with_field_id(field_id)
    }

    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0)
    }

    fn compare(&self, other: &Self) -> std::cmp::Ordering {
        self.0.compare(&other.0)
    }

    type TP<'a> = ();
    fn text(&self, text_provider: Self::TP<'_>) -> std::borrow::Cow<str> {
        self.0.text(text_provider)
    }
}

impl<'tree, HAST: HyperAST<'tree>> steped::Cursor for Node<'tree, HAST>
where
    HAST::IdN: std::fmt::Debug + Copy,
    HAST::TS: RoleStore,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithRoles,
{
    type Node = Self;

    fn goto_next_sibling_internal(
        &mut self,
    ) -> hyperast_gen_ts_tsquery::search::steped::TreeCursorStep {
        self.0.goto_next_sibling_internal()
    }

    fn goto_first_child_internal(
        &mut self,
    ) -> hyperast_gen_ts_tsquery::search::steped::TreeCursorStep {
        self.0.goto_first_child_internal()
    }

    fn goto_parent(&mut self) -> bool {
        self.0.goto_parent()
    }

    fn current_node(&self) -> Self::Node {
        Self(self.0.current_node())
    }

    fn parent_node(&self) -> Option<Self::Node> {
        Some(Self(self.0.parent_node()?))
    }

    type Status = steped::hyperast::CursorStatus<<Self as steped::Node>::IdF>;

    fn current_status(&self) -> Self::Status {
        self.0.current_status()
    }

    fn text_provider(&self) -> <Self::Node as steped::Node>::TP<'_> {
        self.0.text_provider()
    }
}

impl<'tree, HAST: HyperAST<'tree>> Node<'tree, HAST> {
    pub fn new(
        stores: &'tree HAST,
        pos: hyperast::position::StructuralPosition<HAST::IdN, HAST::Idx>,
    ) -> Self {
        Self(steped::hyperast::Node { stores, pos })
    }
}

pub struct MyNodeErazing<'hast, HAST>(std::marker::PhantomData<&'hast HAST>);
impl<'hast, HAST> Default for MyNodeErazing<'hast, HAST> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<'hast, HAST: 'static + hyperast::types::HyperAST> tree_sitter_graph::graph::Erzd
    for MyNodeErazing<'hast, HAST>
{
    type Original<'tree> = Node<'tree, HAST>;
}

impl<'tree, HAST: 'static + HyperAST<'tree>> tree_sitter_graph::graph::LErazng
    for Node<'tree, HAST>
{
    type LErazing = MyNodeErazing<'tree, HAST>;
}

pub struct QueryMatcher<HAST> {
    pub query: hyperast_gen_ts_tsquery::search::steped::Query,
    _phantom: std::marker::PhantomData<HAST>,
}

impl<HAST> QueryMatcher<HAST> {
    fn new(
        source: &str,
        language: &tree_sitter::Language,
    ) -> Result<Self, tree_sitter::QueryError> {
        let query = hyperast_gen_ts_tsquery::search::steped::Query::new(source, language.clone())?;

        Ok(Self {
            query,
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<HAST> Debug for QueryMatcher<HAST> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.query.fmt(f)
    }
}

impl<HAST> GenQuery for QueryMatcher<HAST>
where
    HAST: 'static + hyperast::types::HyperASTShared + for<'tree> hyperast::types::HyperAST<'tree>,
    for<'tree> <HAST as HyperAST<'tree>>::TS: hyperast::types::RoleStore,
    for<'tree> <<HAST as HyperAST<'tree>>::TS as hyperast::types::RoleStore>::IdF:
        From<u16> + Into<u16>,
    for<'tree> <HAST as HyperAST<'tree>>::T: WithSerialization + WithStats + WithRoles,
    <HAST as HyperASTShared>::IdN: std::fmt::Debug + Copy + std::hash::Hash,
    <HAST as HyperASTShared>::Idx: Copy + std::hash::Hash,
{
    type Lang = tree_sitter::Language;

    type Ext = ExtendingStringQuery<Self, Self::Lang>;

    fn pattern_count(&self) -> usize {
        self.query.pattern_count()
    }

    fn capture_index_for_name(&self, name: &str) -> Option<u32> {
        // dbg!(&self.capture_names);
        self.query.capture_index_for_name(name)
    }

    fn capture_quantifiers(
        &self,
        index: usize,
    ) -> impl std::ops::Index<usize, Output = tree_sitter::CaptureQuantifier> {
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

    type Node<'tree> = Node<'tree, HAST>;

    type Cursor = Vec<u16>;

    type Match<'cursor, 'tree: 'cursor>
        = self::MyQMatch<'cursor, 'tree, HAST>
    where
        Self: 'cursor;

    type Matches<'query, 'cursor: 'query, 'tree: 'cursor>
        = self::MyQMatches<
        'query,
        'cursor,
        'tree,
        steped::MatchIt<'query, Self::Node<'tree>, Self::Node<'tree>>,
        HAST,
    >
    where
        Self: 'tree,
        Self: 'query,
        Self: 'cursor;

    type I = u32;

    fn matches<'query, 'cursor: 'query, 'tree: 'cursor>(
        &'query self,
        cursor: &'cursor mut Self::Cursor,
        node: &Self::Node<'tree>,
    ) -> Self::Matches<'query, 'cursor, 'tree> {
        let matchs = self.query.matches(node.clone());
        let node = node.clone();
        self::MyQMatches {
            q: self,
            cursor,
            matchs,
            node,
        }
    }
}

pub struct ExtendingStringQuery<Q = tree_sitter::Query, L = tree_sitter::Language> {
    pub(crate) query: Option<Q>,
    pub(crate) acc: String,
    pub(crate) _phantom: std::marker::PhantomData<L>,
}

impl<Q, L> Default for ExtendingStringQuery<Q, L> {
    fn default() -> Self {
        Self {
            query: Default::default(),
            acc: Default::default(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<HAST> tree_sitter_graph::ExtendedableQuery
    for ExtendingStringQuery<QueryMatcher<HAST>, tree_sitter::Language>
where
    HAST: 'static + hyperast::types::HyperASTShared + for<'tree> hyperast::types::HyperAST<'tree>,
    for<'tree> <HAST as HyperAST<'tree>>::T: WithSerialization + WithStats + WithRoles,
    <HAST as HyperASTShared>::IdN: std::fmt::Debug + Copy + std::hash::Hash,
    <HAST as HyperASTShared>::Idx: Copy + std::hash::Hash,
    for<'tree> <HAST as HyperAST<'tree>>::TS: hyperast::types::RoleStore,
    for<'tree> <<HAST as HyperAST<'tree>>::TS as hyperast::types::RoleStore>::IdF:
        From<u16> + Into<u16>,
{
    type Query = QueryMatcher<HAST>;
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
        QueryMatcher::new(source, language)
    }

    fn make_main_query(&self, language: &Self::Lang) -> Self::Query {
        QueryMatcher::new(&self.acc, language).unwrap()
    }
}

impl<'tree, HAST: hyperast::types::HyperAST<'tree>> tree_sitter_graph::graph::SyntaxNode
    for Node<'tree, HAST>
where
    HAST::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: Copy + std::hash::Hash,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithSerialization + WithStats,
{
    fn id(&self) -> usize {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::hash::DefaultHasher::new();
        self.0.pos.hash(&mut hasher);
        hasher.finish() as usize
    }

    fn kind(&self) -> &'static str {
        use hyperast::position::position_accessors::SolvedPosition;
        let n = self.0.pos.node();
        let n = self.0.stores.resolve_type(&n);
        use hyperast::types::HyperType;
        n.as_static_str()
    }

    fn start_position(&self) -> tree_sitter::Point {
        let conv =
            hyperast::position::PositionConverter::new(&self.0.pos).with_stores(self.0.stores);
        let pos: hyperast::position::row_col::RowCol<usize> =
            conv.compute_pos_post_order::<_, hyperast::position::row_col::RowCol<usize>, _>();
        // use hyperast::position::computing_offset_bottom_up::extract_position_it;
        // let p = extract_position_it(self.stores, self.pos.iter());
        tree_sitter::Point {
            row: pos.row() as usize, //p.range().start,
            column: pos.col() as usize,
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
        use hyperast::position::position_accessors::SolvedPosition;
        hyperast::nodes::TextSerializer::new(self.0.stores, self.0.pos.node()).to_string()
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

impl<'tree, HAST: HyperAST<'tree>> tree_sitter_graph::graph::SyntaxNodeExt for Node<'tree, HAST>
where
    <HAST as HyperASTShared>::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: Copy + std::hash::Hash,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithSerialization + WithStats,
{
    type Cursor = Vec<Self>;

    fn walk(&self) -> Self::Cursor {
        todo!()
    }

    fn named_children<'cursor>(
        &self,
        _cursor: &'cursor mut Self::Cursor,
    ) -> impl ExactSizeIterator<Item = Self> + 'cursor
    where
        'tree: 'cursor,
    {
        todo!();
        vec![].iter().cloned()
    }

    type QM<'cursor>
        = MyQMatch<'cursor, 'tree, HAST>
    where
        Self: 'cursor;
}

pub struct MyQMatch<'cursor, 'tree, HAST: HyperAST<'tree>> {
    stores: &'tree HAST,
    b: &'cursor (),
    qm: hyperast_gen_ts_tsquery::search::steped::query_cursor::QueryMatch<Node<'tree, HAST>>,
}

impl<'cursor, 'tree, HAST: hyperast::types::HyperAST<'tree>> tree_sitter_graph::graph::QMatch
    for MyQMatch<'cursor, 'tree, HAST>
{
    type I = u32;

    type Item = Node<'tree, HAST>;

    fn nodes_for_capture_index(&self, index: Self::I) -> impl Iterator<Item = Self::Item> + '_ {
        self.qm
            .captures
            .iter()
            .filter(move |x| x.index == index)
            .map(|x| x.node.clone())
    }

    fn pattern_index(&self) -> usize {
        self.qm.pattern_index
    }
}

pub struct MyQMatches<'query, 'cursor: 'query, 'tree: 'cursor, It, HAST: HyperASTShared> {
    q: &'query QueryMatcher<HAST>,
    cursor: &'cursor mut Vec<u16>,
    matchs: It,
    node: Node<'tree, HAST>,
}

impl<'query, 'cursor: 'query, 'tree: 'cursor, It, HAST: HyperAST<'tree>> Iterator
    for MyQMatches<'query, 'cursor, 'tree, It, HAST>
where
    It: Iterator<
        Item = hyperast_gen_ts_tsquery::search::steped::query_cursor::QueryMatch<
            Node<'tree, HAST>,
        >,
    >,
{
    type Item = self::MyQMatch<'cursor, 'tree, HAST>;

    fn next(&mut self) -> Option<Self::Item> {
        let qm = self.matchs.next()?;
        let stores = self.node.0.stores;
        Some(self::MyQMatch {
            stores,
            b: &&(),
            qm,
        })
    }
}
