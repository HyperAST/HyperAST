//! First attempt to integrate a query matchers compatible with hyperast.
//! The protoype query matcher is implemented totally differently from the original treesitter one.

use std::fmt::Debug;

use hyperast::{
    position::{StructuralPosition, TreePathMut},
    store::{defaults::NodeIdentifier, SimpleStores},
    types::HyperAST,
};
use hyperast_gen_ts_tsquery::auto::tsq_ser_meta::Conv;
use tree_sitter_graph::GenQuery;

use crate::types::TStore;

pub struct Node<
    'hast,
    HAST = SimpleStores<crate::types::TStore>,
    P = hyperast::position::StructuralPosition,
> {
    pub stores: &'hast HAST,
    pub pos: P,
}

impl<'tree, HAST, P: PartialEq> PartialEq for Node<'tree, HAST, P> {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl<'tree, HAST, P: Clone> Clone for Node<'tree, HAST, P> {
    fn clone(&self) -> Self {
        Self {
            stores: self.stores,
            pos: self.pos.clone(),
        }
    }
}

impl<'tree, HAST, P> tree_sitter_graph::graph::SyntaxNode for Node<'tree, HAST, P>
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: hyperast::types::WithSerialization + hyperast::types::WithStats,
    HAST: HyperAST<'tree, IdN = NodeIdentifier>,
    P: Clone
        + Debug
        + std::hash::Hash
        + hyperast::position::node_filter_traits::Full
        + hyperast::position::position_accessors::WithFullPostOrderPath<HAST::IdN, Idx = HAST::Idx>
        + hyperast::position::position_accessors::SolvedPosition<HAST::IdN>,
    HAST::IdN: Copy,
{
    fn id(&self) -> usize {
        // let id = self.pos.node().unwrap(); // TODO make an associated type
        // let id: usize = unsafe { std::mem::transmute(id) };
        // id

        let mut hasher = std::hash::DefaultHasher::new();
        self.pos.hash(&mut hasher);
        use std::hash::Hasher;
        hasher.finish() as usize
    }

    fn kind(&self) -> &'static str {
        let n = self.pos.node();
        let n = self.stores.resolve_type(&n);
        use hyperast::types::HyperType;
        n.as_static_str()
    }

    fn start_position(&self) -> tree_sitter::Point {
        dbg!(&self.pos);
        let conv = hyperast::position::PositionConverter::new(&self.pos).with_stores(self.stores);
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
        hyperast::nodes::TextSerializer::new(self.stores, self.pos.node()).to_string()
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

impl<'tree, HAST, P> tree_sitter_graph::graph::SyntaxNodeExt for Node<'tree, HAST, P>
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: hyperast::types::WithSerialization + hyperast::types::WithStats,
    HAST: HyperAST<'tree, IdN = NodeIdentifier>,
    P: Clone
        + Debug
        + TreePathMut<HAST::IdN, HAST::Idx>
        + std::hash::Hash
        + hyperast::position::node_filter_traits::Full
        + hyperast::position::position_accessors::WithFullPostOrderPath<HAST::IdN, Idx = HAST::Idx>
        + hyperast::position::position_accessors::SolvedPosition<HAST::IdN>,
{
    type Cursor = Self;

    fn walk(&self) -> Self::Cursor {
        self.clone()
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

    type QM<'cursor> = MyQMatch<'cursor, 'tree, HAST, P>
where
    Self: 'cursor;
}

pub struct MyNodeErazing;

impl tree_sitter_graph::graph::Erzd for MyNodeErazing {
    type Original<'tree> = Node<'tree>;
}

impl<'tree> tree_sitter_graph::graph::LErazng for Node<'tree> {
    type LErazing = MyNodeErazing;
}

pub struct MyQMatch<'cursor, 'tree, HAST: HyperAST<'tree>, P> {
    // root: StructuralPosition<NodeIdentifier, u16>,
    root: P,
    stores: &'tree HAST,
    b: &'cursor (),
    captures: hyperast_gen_ts_tsquery::search::Captured<HAST::IdN, HAST::Idx>,
}

impl<'cursor, 'tree, HAST, P> tree_sitter_graph::graph::QMatch for MyQMatch<'cursor, 'tree, HAST, P>
where
    HAST::IdN: Debug,
    HAST: HyperAST<'tree>,
    P: Clone + TreePathMut<HAST::IdN, HAST::Idx>,
{
    type I = u32;

    type Item = Node<'tree, HAST, P>;

    fn nodes_for_capture_index(&self, index: Self::I) -> impl Iterator<Item = Self::Item> + '_ {
        dbg!(index);
        dbg!(&self.captures);
        self.captures.by_capture_id(index).into_iter().map(|c| {
            dbg!(c);
            let mut p = self.root.clone();
            dbg!(p.node());
            for i in c.path.iter().rev() {
                use hyperast::types::NodeStore;
                let nn = self.stores.node_store().resolve(p.node().unwrap());
                use hyperast::types::IterableChildren;
                use hyperast::types::WithChildren;
                dbg!(nn.children().unwrap().iter_children().collect::<Vec<_>>());
                let node = nn.child(i).unwrap();
                p.goto(node, *i);
            }
            assert_eq!(p.node(), Some(&c.match_node));
            Node {
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
    q: &'query QueryMatcher<crate::types::Type>,
    cursor: &'cursor mut Vec<u16>,
    matchs: hyperast_gen_ts_tsquery::IterMatched2<
        hyperast_gen_ts_tsquery::search::recursive2::MatchingIter<
            'tree,
            SimpleStores<TStore>,
            crate::types::TIdN<NodeIdentifier>,
            &'query hyperast_gen_ts_tsquery::search::PreparedMatcher<crate::types::Type>,
        >,
        &'tree SimpleStores<TStore>,
        crate::iter::IterAll<'tree, StructuralPosition<NodeIdentifier, u16>, SimpleStores<TStore>>,
        crate::types::TIdN<NodeIdentifier>,
    >,
    node: Node<'tree, SimpleStores<crate::types::TStore>, hyperast::position::StructuralPosition>,
}

impl<'query, 'cursor: 'query, 'tree: 'cursor> Iterator for MyQMatches<'query, 'cursor, 'tree> {
    type Item = self::MyQMatch<
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
        Some(self::MyQMatch {
            stores,
            b: &&(),
            captures,
            root,
        })
    }
}

pub struct QueryMatcher<Ty, C = Conv<Ty>>(
    pub hyperast_gen_ts_tsquery::search::PreparedMatcher<Ty, C>,
);

impl<Ty: Debug> Debug for QueryMatcher<Ty> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl GenQuery for QueryMatcher<crate::types::Type> {
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

    type Node<'tree> =
        Node<'tree, SimpleStores<crate::types::TStore>, hyperast::position::StructuralPosition>;

    type Cursor = Vec<u16>;

    fn matches<'query, 'cursor: 'query, 'tree: 'cursor>(
        &'query self,
        cursor: &'cursor mut Self::Cursor,
        node: &Self::Node<'tree>,
    ) -> Self::Matches<'query, 'cursor, 'tree> {
        use crate::iter::IterAll as JavaIter;
        use hyperast::position::TreePath;
        let matchs = self
            .0
            .apply_matcher::<SimpleStores<TStore>, JavaIter<hyperast::position::StructuralPosition, _>, crate::types::TIdN<_>>(
                node.stores,
                *node.pos.node().unwrap(),
            );
        // let a = matchs.next();
        let node = node.clone();
        self::MyQMatches {
            q: self,
            cursor,
            matchs,
            node,
        }
    }

    type Match<'cursor, 'tree: 'cursor> = self::MyQMatch<'cursor, 'tree, SimpleStores<TStore>, hyperast::position::StructuralPosition>
    where
        Self: 'cursor;

    type Matches<'query, 'cursor: 'query, 'tree: 'cursor> =
    self::MyQMatches<'query, 'cursor, 'tree>
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
    for ExtendingStringQuery<
        crate::types::Type,
        QueryMatcher<crate::types::Type>,
        tree_sitter::Language,
    >
{
    type Query = QueryMatcher<crate::types::Type>;
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
        _language: &Self::Lang,
        source: &str,
    ) -> Result<Self::Query, tree_sitter::QueryError> {
        self.acc += source;
        self.acc += "\n";
        dbg!(source);
        let matcher = hyperast_gen_ts_tsquery::prepare_matcher::<crate::types::Type>(source);
        Ok(QueryMatcher(matcher))
    }

    fn make_main_query(&self, _language: &Self::Lang) -> Self::Query {
        let matcher = hyperast_gen_ts_tsquery::prepare_matcher::<crate::types::Type>(&self.acc);
        QueryMatcher(matcher)
    }
}
