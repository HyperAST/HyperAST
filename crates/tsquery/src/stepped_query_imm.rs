//! Attempt to integrate another query matchers compatible with hyperast.
//! The query matcher used here is largely inspired by tree_sitter (query.c).
//! Trying to make this one applicable directly on subtrees, ie. immediated/shallow

use hyper_ast::{
    tree_gen,
    types::{
        self, ETypeStore, HyperAST, HyperASTShared, Role, RoleStore, WithRoles, WithSerialization,
        WithStats,
    },
};
use std::{fmt::Debug, vec};

use crate::CaptureId;

#[repr(transparent)]
pub struct Node<
    'hast,
    HAST: HyperASTShared,
    Acc: hyper_ast::tree_gen::WithLabel,
    Idx = <HAST as HyperASTShared>::Idx,
    P = hyper_ast::position::StructuralPosition<<HAST as HyperASTShared>::IdN, Idx>,
    L = <Acc as hyper_ast::tree_gen::WithLabel>::L,
>(
    // NOTE actually a bad idea to directly wrap cursor_on_unbuild::Node,
    // the nodes go in tsg Graphs and by holding a reference to HAST it locks down everything
    // TODO find a way to extract the essentials from Node (to free Graph), the rest could be then part of the execution context.
    // Doing so will probably contribute to facilitating the staged storage of graph nodes and edges.
    pub crate::cursor_on_unbuild::Node<HAST, Acc, Idx, P, L>,
    /// issue with lifetime bound when associated with trait impl returns
    /// https://play.rust-lang.org/?version=nightly&mode=debug&edition=2024&gist=562bb768901a847e263090e7557e1d93
    std::marker::PhantomData<&'hast ()>,
);
// pub use crate::cursor_on_unbuild::Node;

impl<'hast, 'acc, HAST: HyperASTShared + Clone, Acc> Clone for Node<'hast, HAST, &'acc Acc>
where
    &'acc Acc: hyper_ast::tree_gen::WithLabel,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), std::marker::PhantomData)
    }
}

impl<'hast, HAST: HyperASTShared, Acc: hyper_ast::tree_gen::WithLabel> PartialEq
    for Node<'hast, HAST, Acc>
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

type IdF = u16;

impl<'hast, 'acc, 'l, HAST, Acc> crate::Node for self::Node<'hast, HAST, &'acc Acc>
where
    HAST: HyperAST<'hast> + Clone,
    HAST::TS: ETypeStore<Ty2 = Acc::Type>,
    HAST::TS: hyper_ast::types::RoleStore<IdF = IdF, Role = Role>,
    HAST::T: hyper_ast::types::WithRoles,
    HAST::IdN: Copy,
    Acc: tree_gen::WithChildren<HAST::IdN> + tree_gen::WithRole<Role> + types::Typed,
    &'acc Acc: hyper_ast::tree_gen::WithLabel,
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

    type IdF = <HAST::TS as hyper_ast::types::RoleStore>::IdF;

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

impl<'hast, 'acc, HAST, Acc> crate::Cursor for Node<'hast, HAST, &'acc Acc>
where
    HAST: HyperAST<'hast> + Clone,
    HAST::TS: ETypeStore<Ty2 = Acc::Type> + RoleStore<IdF = IdF, Role = Role>,
    HAST::IdN: Copy,
    HAST::T: WithRoles,
    Acc: hyper_ast::tree_gen::WithRole<Role>,
    Acc: hyper_ast::tree_gen::WithChildren<HAST::IdN>,
    Acc: hyper_ast::types::Typed,
    &'acc Acc: hyper_ast::tree_gen::WithLabel,
{
    type Node = Self;
    type NodeRef<'a>
        = &'a Self
    where
        Self: 'a;

    fn goto_next_sibling_internal(&mut self) -> crate::TreeCursorStep {
        self.0.goto_next_sibling_internal()
    }

    fn goto_first_child_internal(&mut self) -> crate::TreeCursorStep {
        self.0.goto_first_child_internal()
    }

    fn goto_parent(&mut self) -> bool {
        self.0.goto_parent()
    }

    fn current_node(&self) -> &Self {
        self
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

    fn text_provider(&self) -> <Self::Node as crate::Node>::TP<'_> {
        self.0.text_provider()
    }

    fn is_visible_at_root(&self) -> bool {
        self.0.is_visible_at_root()
    }
}

impl<'hast, HAST: HyperASTShared, Acc: hyper_ast::tree_gen::WithLabel> Node<'hast, HAST, Acc> {
    pub fn new(
        stores: HAST,
        acc: Acc,
        label: Option<Acc::L>,
        pos: hyper_ast::position::StructuralPosition<HAST::IdN, HAST::Idx>,
    ) -> Self {
        Self(
            crate::cursor_on_unbuild::Node::new(stores, acc, label, pos),
            Default::default(),
        )
    }
}

pub struct MyNodeErazing<HAST, Acc>(std::marker::PhantomData<(HAST, Acc)>);
impl<'acc, HAST, Acc> Default for MyNodeErazing<HAST, &'acc Acc> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[cfg(feature = "tsg")]
impl<'acc, HAST: HyperASTShared + 'static, Acc: 'static> tree_sitter_graph::graph::Erzd
    for MyNodeErazing<HAST, &'acc Acc>
where
    &'acc Acc: hyper_ast::tree_gen::WithLabel,
{
    type Original<'tree> = Node<'tree, HAST, &'acc Acc>;
}

#[cfg(feature = "tsg")]
impl<'acc, HAST: HyperASTShared + 'static, Acc: 'static> tree_sitter_graph::graph::LErazng
    for Node<'_, HAST, &'acc Acc>
where
    &'acc Acc: hyper_ast::tree_gen::WithLabel,
{
    type LErazing = MyNodeErazing<HAST, &'acc Acc>;
}

pub struct QueryMatcher<'hast, HAST, Acc> {
    pub query: crate::Query,
    _phantom: std::marker::PhantomData<(&'hast (), HAST, Acc)>,
}

impl<HAST, Acc> QueryMatcher<'_, HAST, Acc> {
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

impl<HAST, Acc> Debug for QueryMatcher<'_, HAST, Acc> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.query)
    }
}

#[cfg(feature = "tsg")]
impl<'acc, 'hast, HAST, Acc> tree_sitter_graph::GenQuery for QueryMatcher<'hast, HAST, &'acc Acc>
where
    HAST: 'hast + HyperAST<'hast> + Clone,
    HAST::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: Copy + std::hash::Hash,
    HAST::T: WithSerialization + WithStats + WithRoles,
    HAST::TS: ETypeStore<Ty2 = Acc::Type>
        + hyper_ast::types::RoleStore<IdF = IdF, Role = Role>
        + hyper_ast::types::RoleStore,
    Acc: hyper_ast::tree_gen::WithRole<Role>,
    Acc: hyper_ast::tree_gen::WithChildren<HAST::IdN>,
    Acc: hyper_ast::types::Typed,
    &'acc Acc: hyper_ast::tree_gen::WithLabel,
{
    type Lang = tree_sitter::Language;

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

    type Node<'tree> = Node<'hast, HAST, &'acc Acc>;

    type Cursor = Vec<u16>;

    type Match<'cursor, 'tree: 'cursor>
        = self::MyQMatch<'cursor, 'hast, HAST, &'acc Acc>
    where
        Self: 'cursor;

    type Matches<'query, 'cursor: 'query, 'tree: 'cursor>
        = self::MyQMatches<
        'query,
        'cursor,
        'hast,
        crate::QueryCursor<'query, Self::Node<'tree>, Self::Node<'tree>>,
        HAST,
        &'acc Acc,
    >
    where
        Self: 'query,
        Self: 'cursor;

    type I = u32;

    fn matches<'query, 'cursor: 'query, 'tree: 'cursor>(
        &'query self,
        cursor: &'cursor mut Self::Cursor,
        node: &Node<HAST, &'acc Acc>,
    ) -> self::MyQMatches<
        'query,
        'cursor,
        'hast,
        crate::QueryCursor<'query, Self::Node<'tree>, Self::Node<'tree>>,
        HAST,
        &'acc Acc,
    > {
        let _ = cursor;
        let _ = node;
        unimplemented!("try resolve the issue of lifetime of `node` that may not live long enough")
        // let matchs = self.query.matches_immediate(node.clone());
        // let node = node.clone();
        // self::MyQMatches {
        //     q: self,
        //     cursor,
        //     matchs,
        //     node,
        // }
    }

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
    for ExtendingStringQuery<QueryMatcher<'hast, HAST, &'acc Acc>, tree_sitter::Language>
where
    HAST: 'hast + HyperAST<'hast> + Clone,
    HAST::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: Copy + std::hash::Hash,
    HAST::T: WithSerialization + WithStats + WithRoles,
    HAST::TS: ETypeStore<Ty2 = Acc::Type>
        + hyper_ast::types::RoleStore<IdF = IdF, Role = Role>
        + hyper_ast::types::RoleStore,
    Acc: hyper_ast::tree_gen::WithChildren<HAST::IdN>
        + hyper_ast::tree_gen::WithRole<Role>
        + hyper_ast::types::Typed,
    &'acc Acc: hyper_ast::tree_gen::WithLabel,
{
    type Query = QueryMatcher<'hast, HAST, &'acc Acc>;
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
impl<'hast, 'acc, 'l, HAST, Acc> tree_sitter_graph::graph::SyntaxNode
    for Node<'hast, HAST, &'acc Acc>
where
    HAST: 'hast + HyperAST<'hast> + Clone,
    HAST::IdN: std::hash::Hash + Copy + Debug,
    HAST::Idx: std::hash::Hash,
    HAST::T: WithSerialization + types::WithChildren + WithStats + WithRoles,
    HAST::TS: ETypeStore<Ty2 = Acc::Type>
        + hyper_ast::types::RoleStore<IdF = IdF, Role = Role>
        + hyper_ast::types::RoleStore,
    Acc: hyper_ast::tree_gen::WithChildren<HAST::IdN>
        + hyper_ast::tree_gen::WithRole<Role>
        + hyper_ast::types::Typed,
    &'acc Acc: hyper_ast::tree_gen::WithLabel,
{
    fn id(&self) -> usize {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::hash::DefaultHasher::new();
        self.0.pos.hash(&mut hasher);
        hasher.finish() as usize
    }

    fn kind(&self) -> &'static str {
        use hyper_ast::types::HyperType;
        self.0.kind().as_static_str()
        // use hyper_ast::position::position_accessors::SolvedPosition;
        // let n = self.0.pos.node();
        // let n = self.0.stores.node_store.resolve(&n);
        // // TS::
        // let n = self.0.stores.type_store.resolve_type(&n);
        // n.as_static_str()
    }

    fn start_position(&self) -> tree_sitter::Point {
        // TODO compute the position
        // let conv =
        //     hyper_ast::position::PositionConverter::new(&self.0.pos).with_stores(&self.0.stores);

        // let conv: &hyper_ast::position::WithHyperAstPositionConverter<
        //     hyper_ast::position::StructuralPosition<_, _>,
        //     HAST,
        // > = unsafe { std::mem::transmute(&conv) };
        // let pos: hyper_ast::position::row_col::RowCol<usize> =
        //     conv.compute_pos_post_order::<_, hyper_ast::position::row_col::RowCol<usize>, HAST::IdN>();
        // // use hyper_ast::position::computing_offset_bottom_up::extract_position_it;
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
        use hyper_ast::position::TreePath;
        let stores: &HAST = unsafe { std::mem::transmute(&self.0.stores) };
        if let Some(root) = self.0.pos.node() {
            hyper_ast::nodes::TextSerializer::new(stores, *root).to_string()
        } else {
            // log::error!("{}", self.kind());
            // use crate::Node;
            // self.0.text(())
            self.0
                .label
                .as_ref()
                .map_or("aaa", |x| x.as_ref())
                .to_string()
            // "".to_string()
        }
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

#[cfg(feature = "tsg")]
impl<'acc, 'hast, HAST, Acc> tree_sitter_graph::graph::SyntaxNodeExt
    for Node<'hast, HAST, &'acc Acc>
where
    HAST: 'hast + HyperAST<'hast> + Clone,
    HAST::IdN: Copy + std::hash::Hash + Debug,
    HAST::Idx: Copy + std::hash::Hash,
    HAST::T: WithSerialization + WithStats + WithRoles,
    HAST::TS: ETypeStore<Ty2 = Acc::Type>
        + hyper_ast::types::RoleStore<IdF = IdF, Role = Role>
        + hyper_ast::types::RoleStore,
    Acc: hyper_ast::tree_gen::WithRole<Role>,
    Acc: hyper_ast::tree_gen::WithChildren<HAST::IdN>,
    Acc: hyper_ast::types::Typed,
    &'acc Acc: hyper_ast::tree_gen::WithLabel,
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
        vec![todo!()].iter().cloned()
    }

    type QM<'cursor>
        = MyQMatch<'cursor, 'hast, HAST, &'acc Acc>
    where
        Self: 'cursor;
}

pub struct MyQMatch<
    'cursor,
    'hast,
    HAST: hyper_ast::types::HyperASTShared,
    Acc: hyper_ast::tree_gen::WithLabel,
    Idx = <HAST as HyperASTShared>::Idx,
    P = hyper_ast::position::StructuralPosition<<HAST as HyperASTShared>::IdN, Idx>,
    L = <Acc as hyper_ast::tree_gen::WithLabel>::L,
> {
    pub stores: HAST,
    pub b: &'cursor (),
    pub c: &'hast (),
    pub qm: crate::QueryMatch<Node<'hast, HAST, Acc, Idx, P, L>>,
    pub i: u16,
}

#[cfg(feature = "tsg")]
impl<'cursor, 'hast, 'acc, HAST, Acc> tree_sitter_graph::graph::QMatch
    for MyQMatch<'cursor, 'hast, HAST, &'acc Acc>
where
    HAST: HyperAST<'hast> + Clone,
    HAST::TS: ETypeStore<Ty2 = Acc::Type> + hyper_ast::types::RoleStore<IdF = IdF, Role = Role>,
    HAST::T: hyper_ast::types::WithRoles,
    HAST::IdN: Copy,
    Acc: tree_gen::WithChildren<HAST::IdN> + tree_gen::WithRole<Role> + types::Typed,
    &'acc Acc: hyper_ast::tree_gen::WithLabel,
{
    type I = u32;

    type Item = Node<'hast, HAST, &'acc Acc>;

    fn nodes_for_capture_index(&self, index: Self::I) -> impl Iterator<Item = Self::Item> {
        // log::error!("{}", index);
        self.qm
            .nodes_for_capture_index(CaptureId::new(index))
            .cloned()
    }

    fn pattern_index(&self) -> usize {
        // self.qm.pattern_index.to_usize()
        self.i as usize
    }
}

pub struct MyQMatches<
    'query,
    'cursor,
    'hast,
    It,
    HAST: hyper_ast::types::HyperASTShared,
    Acc: hyper_ast::tree_gen::WithLabel,
    Idx = <HAST as HyperASTShared>::Idx,
    P = hyper_ast::position::StructuralPosition<<HAST as HyperASTShared>::IdN, Idx>,
    L = <Acc as hyper_ast::tree_gen::WithLabel>::L,
> {
    pub(crate) q: &'query QueryMatcher<'hast, HAST, Acc>,
    pub(crate) cursor: &'cursor mut Vec<u16>,
    pub(crate) matchs: It,
    pub(crate) node: Node<'hast, HAST, Acc, Idx, P, L>,
}

impl<'query, 'cursor, 'hast, 'acc, It, HAST, Acc> Iterator
    for MyQMatches<'query, 'cursor, 'hast, It, HAST, &'acc Acc>
where
    HAST: HyperAST<'hast> + Clone,
    It: Iterator<Item = crate::QueryMatch<Node<'hast, HAST, &'acc Acc>>>,
    &'acc Acc: hyper_ast::tree_gen::WithLabel,
{
    type Item = self::MyQMatch<'cursor, 'hast, HAST, &'acc Acc>;

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
            c: &&(),
            qm,
            i,
        })
    }
}
