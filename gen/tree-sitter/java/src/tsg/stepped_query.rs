//! Attempt to integrate another query matchers compatible with hyperast.
//! The query matcher used here is largely inspired by tree_sitter (query.c).

use hyper_ast::{
    store::SimpleStores,
    types::{HyperAST, RoleStore, WithRoles},
};
use hyper_ast_gen_ts_tsquery::search::steped;
use std::fmt::Debug;
use tree_sitter_graph::GenQuery;

type TStore = crate::types::TStore;
type Type = crate::types::Type;

type HAST = SimpleStores<TStore>;

#[repr(transparent)]
pub struct Node<'tree, HAST: HyperAST<'tree>>(steped::hyperast::Node<'tree, HAST>);

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
    HAST::TS: RoleStore<IdF = steped::FieldId>,
    HAST::T: WithRoles,
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

    fn child_by_field_id(&self, negated_field_id: steped::FieldId) -> Option<Self> {
        self.0.child_by_field_id(negated_field_id).map(|x| Self(x))
    }

    fn equal(&self, other: &Self) -> bool {
        self.0.equal(&other.0)
    }

    fn compare(&self, other: &Self) -> std::cmp::Ordering {
        self.0.compare(&other.0)
    }
}

impl<'tree, HAST: HyperAST<'tree>> steped::Cursor for Node<'tree, HAST>
where
    HAST::IdN: std::fmt::Debug + Copy,
    HAST::TS: RoleStore<IdF = steped::FieldId>,
    HAST::T: WithRoles,
{
    type Node = Self;

    fn goto_next_sibling_internal(
        &mut self,
    ) -> hyper_ast_gen_ts_tsquery::search::steped::TreeCursorStep {
        self.0.goto_next_sibling_internal()
    }

    fn goto_first_child_internal(
        &mut self,
    ) -> hyper_ast_gen_ts_tsquery::search::steped::TreeCursorStep {
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

    fn current_status(&self) -> hyper_ast_gen_ts_tsquery::search::steped::Status {
        self.0.current_status()
    }
}

impl<'tree, HAST: HyperAST<'tree>> Node<'tree, HAST> {
    pub fn new(
        stores: &'tree HAST,
        pos: hyper_ast::position::StructuralPosition<HAST::IdN, HAST::Idx>,
    ) -> Self {
        Self(steped::hyperast::Node { stores, pos })
    }
}

pub struct MyNodeErazing;

impl tree_sitter_graph::graph::Erzd for MyNodeErazing {
    type Original<'tree> = Node<'tree, HAST>;
}

impl<'tree, HAST: HyperAST<'tree>> tree_sitter_graph::graph::LErazng for Node<'tree, HAST> {
    type LErazing = MyNodeErazing;
}

pub struct QueryMatcher<Ty> {
    pub query: hyper_ast_gen_ts_tsquery::search::steped::Query,
    capture_names: Vec<&'static str>,
    capture_quantifiers_vec: Vec<Vec<tree_sitter::CaptureQuantifier>>,
    _phantom: std::marker::PhantomData<Ty>,
}

impl<Ty> QueryMatcher<Ty> {
    fn new(
        source: &str,
        language: &tree_sitter::Language,
    ) -> Result<Self, tree_sitter::QueryError> {
        let query = hyper_ast_gen_ts_tsquery::search::steped::Query::new(source, language.clone())?;

        // let string_count = unsafe { ffi::ts_query_string_count(ptr.0) };
        let capture_count = query.capture_count();
        let pattern_count = query.pattern_count();

        let mut capture_names = Vec::with_capacity(capture_count as usize);
        let mut capture_quantifiers_vec = Vec::with_capacity(pattern_count as usize);
        // let mut text_predicates_vec = Vec::with_capacity(pattern_count);
        // let mut property_predicates_vec = Vec::with_capacity(pattern_count);
        // let mut property_settings_vec = Vec::with_capacity(pattern_count);
        // let mut general_predicates_vec = Vec::with_capacity(pattern_count);

        // Build a vector of strings to store the capture names.
        for i in 0..capture_count {
            let name = query.capture_name(i as u32);
            capture_names.push(name);
        }
        // Build a vector to store capture quantifiers.
        for i in 0..pattern_count {
            let quantifiers = query.quantifiers_at_pattern(i);
            capture_quantifiers_vec.push(quantifiers);
        }

        Ok(Self {
            query,
            capture_names,
            capture_quantifiers_vec,
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<Ty: Debug> Debug for QueryMatcher<Ty> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.query.fmt(f)
    }
}

impl GenQuery for QueryMatcher<Type> {
    type Lang = tree_sitter::Language;

    type Ext = ExtendingStringQuery<Type, Self, Self::Lang>;

    fn pattern_count(&self) -> usize {
        self.query.pattern_count()
    }

    fn capture_index_for_name(&self, name: &str) -> Option<u32> {
        // dbg!(&self.capture_names);
        self.capture_names
            .iter()
            .position(|x| *x == name)
            .map(|i| i as u32)
    }

    fn capture_quantifiers(
        &self,
        index: usize,
    ) -> impl std::ops::Index<usize, Output = tree_sitter::CaptureQuantifier> {
        self.capture_quantifiers_vec[index].clone()
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

    type Match<'cursor, 'tree: 'cursor> = self::MyQMatch<'cursor, 'tree, HAST>
    where
        Self: 'cursor;

    type Matches<'query, 'cursor: 'query, 'tree: 'cursor> =
    self::MyQMatches<'query, 'cursor, 'tree, steped::MatchIt<'query,Self::Node<'tree>,Self::Node<'tree>>>
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
    for ExtendingStringQuery<Type, QueryMatcher<Type>, tree_sitter::Language>
{
    type Query = QueryMatcher<Type>;
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

impl<'tree> tree_sitter_graph::graph::SyntaxNode for Node<'tree, HAST> {
    fn id(&self) -> usize {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::hash::DefaultHasher::new();
        self.0.pos.hash(&mut hasher);
        hasher.finish() as usize
    }

    fn kind(&self) -> &'static str {
        use hyper_ast::position::position_accessors::SolvedPosition;
        let n = self.0.pos.node();
        let n = self.0.stores.resolve_type(&n);
        use hyper_ast::types::HyperType;
        n.as_static_str()
    }

    fn start_position(&self) -> tree_sitter::Point {
        let conv =
            hyper_ast::position::PositionConverter::new(&self.0.pos).with_stores(self.0.stores);
        let pos: hyper_ast::position::row_col::RowCol<usize> =
            conv.compute_pos_post_order::<_, hyper_ast::position::row_col::RowCol<usize>, _>();
        // use hyper_ast::position::computing_offset_bottom_up::extract_position_it;
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
        use hyper_ast::position::position_accessors::SolvedPosition;
        hyper_ast::nodes::TextSerializer::new(self.0.stores, self.0.pos.node()).to_string()
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

impl<'tree> tree_sitter_graph::graph::SyntaxNodeExt for Node<'tree, HAST> {
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

    type QM<'cursor> = MyQMatch<'cursor, 'tree, HAST>
where
    Self: 'cursor;
}

pub struct MyQMatch<'cursor, 'tree, HAST: HyperAST<'tree>> {
    stores: &'tree HAST,
    b: &'cursor (),
    qm: hyper_ast_gen_ts_tsquery::search::steped::query_cursor::QueryMatch<Node<'tree, HAST>>,
}

impl<'cursor, 'tree> tree_sitter_graph::graph::QMatch for MyQMatch<'cursor, 'tree, HAST> {
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

pub struct MyQMatches<'query, 'cursor: 'query, 'tree: 'cursor, It> {
    q: &'query QueryMatcher<Type>,
    cursor: &'cursor mut Vec<u16>,
    matchs: It,
    node: Node<'tree, HAST>,
}

impl<'query, 'cursor: 'query, 'tree: 'cursor, It> Iterator
    for MyQMatches<'query, 'cursor, 'tree, It>
where
    It: Iterator<
        Item = hyper_ast_gen_ts_tsquery::search::steped::query_cursor::QueryMatch<
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
