//! Tree Generators
//!
//! This module contains facilities to help you build an HyperAST.
//! - [`TreeGen::make`] is where a subtree is pushed in the HyperAST
//!   - You should also use [`crate::store::nodes::legion::NodeStore::prepare_insertion`]
//!     to insert subtrees in the HyperAST while deduplicating identical ones
//! - To visit parsers with a zipper/cursor interface you should implement [`ZippedTreeGen`]
//!   - [`crate::parser::TreeCursor`] should be implemented to wrap you parser's interface
//!
//!
//! ## Important Note
//! To make code analysis incremental in the HyperAST,
//! we locally persist locally derived values, we call them metadata.
//! To save memory, we also deduplicate identical nodes using the type, label and children of a subtree.
//! In other word, in the HyperAST, you store Metadata (derived values) along subtrees of the HyperAST,
//! and deduplicate subtree using identifying data.
//! To ensure derived data are unique per subtree,
//! metadata should only be derived from local identifying values.

pub mod parser;

use std::fmt::Debug;

use crate::{hashed::NodeHashs, nodes::Space};

use self::parser::Visibility;

pub type Spaces = Vec<Space>;

/// Builder of a node for the hyperAST
pub trait Accumulator {
    type Node;
    fn push(&mut self, full_node: Self::Node);
}

pub trait WithByteRange {
    fn has_children(&self) -> bool {
        todo!()
    }
    fn begin_byte(&self) -> usize;
    fn end_byte(&self) -> usize;
}

// TODO merge with other node traits?
pub trait WithChildren<Id: Clone> {
    fn children(&self) -> &[Id];
    fn child_count(&self) -> usize {
        let cs = self.children();
        cs.len()
    }
    fn child(&self, idx: usize) -> Option<Id> {
        let cs = self.children();
        cs.get(idx).cloned()
    }
}
// TODO merge with other node traits?
pub trait WithRole<R> {
    fn role_at(&self, idx: usize) -> Option<R>;
}

pub struct BasicAccumulator<T, Id> {
    pub kind: T,
    pub children: Vec<Id>,
}

impl<T, Id> BasicAccumulator<T, Id> {
    pub fn new(kind: T) -> Self {
        Self {
            kind,
            children: vec![],
        }
    }

    #[cfg(feature = "legion")]
    pub fn add_primary<L, K>(
        self,
        dyn_builder: &mut impl crate::store::nodes::EntityBuilder,
        interned_kind: K,
        label_id: Option<L>,
    ) where
        K: 'static + std::marker::Send + std::marker::Sync,
        L: 'static + std::marker::Send + std::marker::Sync,
        Id: 'static + std::marker::Send + std::marker::Sync + Eq,
    {
        // TODO better handle the interneds
        // TODO the "static" interning should be hanled more specifically
        dyn_builder.add(interned_kind);
        if let Some(label_id) = label_id {
            dyn_builder.add(label_id);
        }

        let children = self.children;
        if children.len() == 1 {
            let Ok(cs) = children.try_into() else {
                unreachable!();
            };
            dyn_builder.add(crate::store::nodes::legion::compo::CS0::<_, 1>(cs));
        } else if children.len() == 2 {
            let Ok(cs) = children.try_into() else {
                unreachable!();
            };
            dyn_builder.add(crate::store::nodes::legion::compo::CS0::<_, 2>(cs));
        } else if !children.is_empty() {
            // TODO make global components, at least for primaries.
            dyn_builder.add(crate::store::nodes::legion::compo::CS(
                children.into_boxed_slice(),
            ));
        }
    }
}

impl<T: Debug, Id> Debug for BasicAccumulator<T, Id> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BasicAccumulator")
            .field("kind", &self.kind)
            .field("children", &self.children.len())
            .finish()
    }
}

impl<T, Id> Accumulator for BasicAccumulator<T, Id> {
    type Node = Id;
    fn push(&mut self, node: Self::Node) {
        self.children.push(node);
    }
}

/// Builder of a node aware of its indentation for the hyperAST
pub trait AccIndentation: Accumulator {
    fn indentation<'a>(&'a self) -> &'a Spaces;
}

#[derive(Default, Debug, Clone, Copy)]
pub struct SubTreeMetrics<U> {
    pub hashs: U,
    pub size: u32,
    pub height: u32,
    pub size_no_spaces: u32,
    /// should include lines inside labels
    pub line_count: u16, // TODO u16 is definitely not enough at the directory level e.g. 1.6MLoCs for Hadoop
                         // pub byte_len: u32,
}

impl<U: NodeHashs> SubTreeMetrics<U> {
    pub fn acc(&mut self, other: Self) {
        self.height = self.height.max(other.height);
        self.size += other.size;
        self.size_no_spaces += other.size_no_spaces;
        self.hashs.acc(&other.hashs);
        self.line_count = self.line_count.saturating_add(other.line_count);
    }
}

impl<U> SubTreeMetrics<U> {
    pub fn map_hashs<V>(self, f: impl Fn(U) -> V) -> SubTreeMetrics<V> {
        SubTreeMetrics {
            hashs: f(self.hashs),
            size: self.size,
            height: self.height,
            size_no_spaces: self.size_no_spaces,
            line_count: self.line_count,
        }
    }

    #[must_use]
    #[cfg(feature = "legion")]
    pub fn add_md_metrics(
        self,
        dyn_builder: &mut impl crate::store::nodes::EntityBuilder,
        children_is_empty: bool,
    ) -> U {
        use crate::store::nodes::legion::compo;
        if !children_is_empty {
            dyn_builder.add(compo::Size(self.size));
            dyn_builder.add(compo::SizeNoSpaces(self.size_no_spaces));
            dyn_builder.add(compo::Height(self.height));
        }

        if self.line_count > 0 {
            dyn_builder.add(compo::LineCount(self.line_count));
        }

        self.hashs
    }
}

impl<U: crate::hashed::ComputableNodeHashs> SubTreeMetrics<U> {
    pub fn finalize<K: ?Sized + std::hash::Hash, L: ?Sized + std::hash::Hash>(
        self,
        k: &K,
        l: &L,
        line_count: u16,
    ) -> SubTreeMetrics<crate::hashed::HashesBuilder<U>> {
        let size_no_spaces = self.size_no_spaces + 1;
        use crate::hashed::IndexingHashBuilder;
        let hashs = crate::hashed::HashesBuilder::new(self.hashs, k, l, size_no_spaces);
        SubTreeMetrics {
            hashs,
            size: self.size + 1,
            height: self.height + 1,
            size_no_spaces,
            line_count: self.line_count + line_count,
        }
    }
}

pub trait GlobalData {
    fn up(&mut self);
    fn right(&mut self);
    fn down(&mut self);
}

#[derive(Debug, Clone, Copy)]
pub struct BasicGlobalData {
    depth: usize,
    /// preorder position
    position: usize,
}

impl Default for BasicGlobalData {
    fn default() -> Self {
        Self {
            depth: 1,
            position: 0,
        }
    }
}

impl GlobalData for BasicGlobalData {
    fn up(&mut self) {
        self.depth -= 1;
        // TODO fix, there are issues the depth count is too big, I am probably missing a up somewhere
    }

    fn right(&mut self) {
        self.position += 1;
        // self.depth -= 1;
    }

    /// goto the first children
    fn down(&mut self) {
        self.position += 1;
        self.depth += 1;
    }
}
pub trait TotalBytesGlobalData {
    fn set_sum_byte_length(&mut self, sum_byte_length: usize);
}

#[derive(Debug, Clone, Copy)]
pub struct TextedGlobalData<'a> {
    text: &'a [u8],
    inner: BasicGlobalData,
}

impl<'a> TextedGlobalData<'a> {
    pub fn new(inner: BasicGlobalData, text: &'a [u8]) -> Self {
        Self { text, inner }
    }
    pub fn text(self) -> &'a [u8] {
        self.text
    }
}

impl<'a> GlobalData for TextedGlobalData<'a> {
    fn up(&mut self) {
        self.inner.up();
    }

    fn right(&mut self) {
        self.inner.right();
    }

    /// goto the first children
    fn down(&mut self) {
        self.inner.down();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpacedGlobalData<'a> {
    sum_byte_length: usize,
    inner: TextedGlobalData<'a>,
}
impl<'a> From<TextedGlobalData<'a>> for SpacedGlobalData<'a> {
    fn from(inner: TextedGlobalData<'a>) -> Self {
        Self {
            sum_byte_length: 0,
            inner,
        }
    }
}
impl<'a> From<SpacedGlobalData<'a>> for BasicGlobalData {
    fn from(x: SpacedGlobalData<'a>) -> Self {
        BasicGlobalData::from(x.inner)
    }
}
impl<'a> From<TextedGlobalData<'a>> for BasicGlobalData {
    fn from(x: TextedGlobalData<'a>) -> Self {
        BasicGlobalData::from(x.inner)
    }
}
impl<'a> From<&mut SpacedGlobalData<'a>> for BasicGlobalData {
    fn from(x: &mut SpacedGlobalData<'a>) -> Self {
        BasicGlobalData::from(x.inner)
    }
}
impl<'a> From<&mut TextedGlobalData<'a>> for BasicGlobalData {
    fn from(x: &mut TextedGlobalData<'a>) -> Self {
        BasicGlobalData::from(x.inner)
    }
}
impl<'a> SpacedGlobalData<'a> {
    pub fn sum_byte_length(self) -> usize {
        self.sum_byte_length
    }
}
impl<'a> TotalBytesGlobalData for SpacedGlobalData<'a> {
    fn set_sum_byte_length(&mut self, sum_byte_length: usize) {
        // assert!(self.sum_byte_length <= sum_byte_length);
        assert!(
            self.sum_byte_length <= sum_byte_length,
            "new byte offset is smaller: {} > {}",
            self.sum_byte_length,
            sum_byte_length
        );
        self.sum_byte_length = sum_byte_length;
    }
}

impl<'a> GlobalData for SpacedGlobalData<'a> {
    fn up(&mut self) {
        self.inner.up();
    }

    fn right(&mut self) {
        self.inner.right();
    }

    /// goto the first children
    fn down(&mut self) {
        self.inner.down();
    }
}

/// Primary trait to implement to generate AST.
pub trait TreeGen {
    /// Container holding data waiting to be added to the HyperAST
    /// Note: needs WithByteRange to handle hidden node properly, it allows to go back up without using the cursor. When Treesitter is "fixed" change that
    type Acc: AccIndentation + WithByteRange;
    /// Container holding global data used during generation.
    ///
    /// Useful for transient data needed during generation,
    /// this way you avoid cluttering [TreeGen::Acc].
    ///
    /// WARN make sure it does not leaks contextual data in subtrees.
    type Global: GlobalData;
    fn make(
        &mut self,
        global: &mut Self::Global,
        acc: <Self as TreeGen>::Acc,
        label: Option<String>,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node;
}

#[derive(Debug)]
pub struct Parents<Acc>(Vec<P<Acc>>);
impl<Acc> From<Acc> for Parents<Acc> {
    fn from(value: Acc) -> Self {
        Self::new(P::Visible(value))
    }
}

#[derive(Debug)]
enum P<Acc> {
    ManualyHidden,
    BothHidden,
    Hidden(Acc),
    Visible(Acc),
}

impl<Acc> P<Acc> {
    fn s(&self) -> &str {
        match self {
            P::ManualyHidden => "ManualyHidden",
            P::BothHidden => "BothHidden",
            P::Hidden(_) => "Hidden",
            P::Visible(_) => "Visible",
        }
    }
    fn is_both_hidden(&self) -> bool {
        match self {
            P::BothHidden => true,
            _ => false,
        }
    }
    fn unwrap(self) -> Acc {
        match self {
            P::ManualyHidden => panic!(),
            P::BothHidden => panic!(),
            P::Hidden(p) => p,
            P::Visible(p) => p,
        }
    }
    fn as_ref(&self) -> P<&Acc> {
        match self {
            P::ManualyHidden => P::ManualyHidden,
            P::BothHidden => P::BothHidden,
            P::Hidden(t) => P::Hidden(t),
            P::Visible(t) => P::Visible(t),
        }
    }
    fn as_mut(&mut self) -> P<&mut Acc> {
        match self {
            P::ManualyHidden => P::ManualyHidden,
            P::BothHidden => P::BothHidden,
            P::Hidden(t) => P::Hidden(t),
            P::Visible(t) => P::Visible(t),
        }
    }
}

impl<Acc> P<Acc> {
    fn ok(self) -> Option<Acc> {
        match self {
            P::ManualyHidden => None,
            P::BothHidden => None,
            P::Hidden(p) => Some(p),
            P::Visible(p) => Some(p),
        }
    }
    fn visibility(self) -> Option<(Visibility, Acc)> {
        match self {
            P::ManualyHidden => None,
            P::BothHidden => None,
            P::Hidden(a) => Some((Visibility::Hidden, a)),
            P::Visible(a) => Some((Visibility::Visible, a)),
        }
    }
}

impl<Acc> Parents<Acc> {
    fn new(value: P<Acc>) -> Self {
        Self(vec![value])
    }
    pub fn finalize(mut self) -> Acc {
        assert_eq!(self.0.len(), 1);
        self.0.pop().unwrap().unwrap()
    }
    fn push(&mut self, value: P<Acc>) {
        self.0.push(value)
    }
    fn pop(&mut self) -> Option<P<Acc>> {
        self.0.pop()
    }
    pub fn parent(&self) -> Option<&Acc> {
        self.0.iter().rev().find_map(|x| x.as_ref().ok())
    }
    fn parent_mut(&mut self) -> Option<&mut Acc> {
        self.0.iter_mut().rev().find_map(|x| x.as_mut().ok())
    }
    fn parent_mut_with_vis(&mut self) -> Option<(Visibility, &mut Acc)> {
        self.0
            .iter_mut()
            .rev()
            .find_map(|x| x.as_mut().visibility())
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

pub struct RoleAcc<R> {
    pub current: Option<R>,
    pub roles: Vec<R>,
    pub offsets: Vec<u8>,
}

impl<R> Default for RoleAcc<R> {
    fn default() -> Self {
        Self {
            current: None,
            roles: Default::default(),
            offsets: Default::default(),
        }
    }
}

impl<R> RoleAcc<R> {
    pub fn acc(&mut self, role: R, o: usize) {
        use num::ToPrimitive;
        if let Some(o) = o.to_u8() {
            self.roles.push(role);
            self.offsets.push(o);
        } else {
            log::warn!("overflowed 255 offseted role...");
            debug_assert!(false);
            // TODO could increase to u16,
            // at least on some variants.
            // TODO could also use the repeat nodes to break down nodes with way to many children...
        }
    }

    #[cfg(feature = "legion")]
    pub fn add_md(self, dyn_builder: &mut impl crate::store::nodes::EntityBuilder)
    where
        R: 'static + std::marker::Send + std::marker::Sync,
    {
        debug_assert!(self.current.is_none());
        if self.roles.len() > 0 {
            dyn_builder.add(self.roles.into_boxed_slice());
            use crate::store::nodes::legion::compo;
            dyn_builder.add(compo::RoleOffsets(self.offsets.into_boxed_slice()));
        }
    }
}

#[cfg(feature = "legion")]
pub fn add_md_precomp_queries(
    dyn_builder: &mut impl crate::store::nodes::EntityBuilder,
    precomp_queries: PrecompQueries,
) {
    use crate::store::nodes::legion::compo;
    if precomp_queries > 0 {
        dyn_builder.add(compo::Precomp(precomp_queries));
    } else {
        dyn_builder.add(compo::PrecompFlag);
    }
}

pub mod zipped;
pub use zipped::PreResult;
pub use zipped::ZippedTreeGen;

/// utils for generating code with tree-sitter
#[cfg(feature = "ts")]
pub mod utils_ts {

    pub trait TsEnableTS: crate::types::ETypeStore
    where
        Self::Ty2: TsType,
    {
        fn obtain_type<N: crate::tree_gen::parser::NodeWithU16TypeId>(n: &N) -> Self::Ty2;
    }

    pub trait TsType: crate::types::HyperType + Copy {
        fn spaces() -> Self;
        fn is_repeat(&self) -> bool;
    }

    pub fn tree_sitter_parse(
        text: &[u8],
        language: &tree_sitter::Language,
    ) -> Result<tree_sitter::Tree, tree_sitter::Tree> {
        let mut parser = tree_sitter::Parser::new();
        // TODO see if a timeout of a cancellation flag could be useful
        // const MINUTE: u64 = 60 * 1000 * 1000;
        // parser.set_timeout_micros(timeout_micros);
        // parser.set_cancellation_flag(flag);
        parser.set_language(language).unwrap();
        let tree = parser.parse(text, None).unwrap();
        if tree.root_node().has_error() {
            Err(tree)
        } else {
            Ok(tree)
        }
    }

    use super::parser::Visibility;

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    #[allow(dead_code)] // NOTE: created by tree sitter
    pub(crate) enum TreeCursorStep {
        TreeCursorStepNone,
        TreeCursorStepHidden,
        TreeCursorStepVisible,
    }

    impl TreeCursorStep {
        pub(crate) fn ok(&self) -> Option<Visibility> {
            match self {
                TreeCursorStep::TreeCursorStepNone => None,
                TreeCursorStep::TreeCursorStepHidden => Some(Visibility::Hidden),
                TreeCursorStep::TreeCursorStepVisible => Some(Visibility::Visible),
            }
        }
    }

    extern "C" {
        fn ts_tree_cursor_goto_first_child_internal(
            self_: *mut tree_sitter::ffi::TSTreeCursor,
        ) -> TreeCursorStep;
        fn ts_tree_cursor_goto_next_sibling_internal(
            self_: *mut tree_sitter::ffi::TSTreeCursor,
        ) -> TreeCursorStep;
    }

    #[repr(transparent)]
    pub struct TNode<'a>(pub tree_sitter::Node<'a>);

    impl<'a> crate::tree_gen::parser::Node for TNode<'a> {
        fn kind(&self) -> &str {
            self.0.kind()
        }

        fn start_byte(&self) -> usize {
            self.0.start_byte()
        }

        fn end_byte(&self) -> usize {
            self.0.end_byte()
        }

        fn child_count(&self) -> usize {
            self.0.child_count()
        }

        fn child(&self, i: usize) -> Option<Self> {
            self.0.child(i).map(TNode)
        }

        fn is_named(&self) -> bool {
            self.0.is_named()
        }

        fn is_missing(&self) -> bool {
            self.0.is_missing()
        }

        fn is_error(&self) -> bool {
            self.0.is_error()
        }
    }

    impl<'a> crate::tree_gen::parser::NodeWithU16TypeId for TNode<'a> {
        fn kind_id(&self) -> u16 {
            self.0.kind_id()
        }
    }

    #[repr(transparent)]
    #[derive(Clone)]
    pub struct TTreeCursor<'a, const HIDDEN_NODES: bool = false>(pub tree_sitter::TreeCursor<'a>);

    impl<'a, const HIDDEN_NODES: bool> std::fmt::Debug for TTreeCursor<'a, HIDDEN_NODES> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_tuple("TTreeCursor")
                .field(&self.0.node().kind())
                .finish()
        }
    }

    impl<'a, const HIDDEN_NODES: bool> crate::tree_gen::parser::TreeCursor
        for TTreeCursor<'a, HIDDEN_NODES>
    {
        type N = TNode<'a>;
        fn node(&self) -> TNode<'a> {
            TNode(self.0.node())
        }

        fn role(&self) -> Option<std::num::NonZeroU16> {
            self.0.field_id()
        }

        fn goto_parent(&mut self) -> bool {
            self.0.goto_parent()
        }

        fn goto_first_child(&mut self) -> bool {
            self.goto_first_child_extended().is_some()
        }

        fn goto_next_sibling(&mut self) -> bool {
            self.goto_next_sibling_extended().is_some()
        }

        fn goto_first_child_extended(&mut self) -> Option<Visibility> {
            if HIDDEN_NODES {
                unsafe {
                    let s = &mut self.0;
                    let s: *mut tree_sitter::ffi::TSTreeCursor = std::mem::transmute(s);
                    ts_tree_cursor_goto_first_child_internal(s)
                }
                .ok()
            } else {
                if self.0.goto_first_child() {
                    Some(Visibility::Visible)
                } else {
                    None
                }
            }
        }

        fn goto_next_sibling_extended(&mut self) -> Option<Visibility> {
            if HIDDEN_NODES {
                let r = unsafe {
                    let s = &mut self.0;
                    let s: *mut tree_sitter::ffi::TSTreeCursor = std::mem::transmute(s);
                    ts_tree_cursor_goto_next_sibling_internal(s)
                }
                .ok();
                r
            } else {
                if self.0.goto_next_sibling() {
                    Some(Visibility::Visible)
                } else {
                    None
                }
            }
        }
    }

    /// Guaranteed to work even when considering hidden nodes,
    /// i.e., goto_next_cchildren() skips hidden parents...
    pub struct PrePost<C> {
        has: super::zipped::Has,
        stack: Vec<C>,
        vis: bitvec::vec::BitVec,
    }

    impl<'a, C: super::parser::TreeCursor + Clone> PrePost<C> {
        pub fn new(cursor: &C) -> Self {
            use bitvec::prelude::Lsb0;
            let mut vis = bitvec::bitvec![];
            vis.push(Visibility::Hidden == Visibility::Hidden);
            let pre_post = Self {
                has: super::zipped::Has::Down,
                stack: vec![cursor.clone()],
                vis,
            };
            pre_post
        }

        pub fn current(&mut self) -> Option<(&C, &mut super::zipped::Has)> {
            self.stack.last().map(|c| (c, &mut self.has))
        }

        pub fn next(&mut self) -> Option<Visibility> {
            use super::zipped::Has;
            use crate::tree_gen::parser::Node;
            if self.vis.is_empty() {
                return None;
            };
            let Some(cursor) = self.stack.last_mut() else {
                return None;
            };
            let mut cursor = cursor.clone();
            if self.has != Has::Up
                && let Some(visibility) = cursor.goto_first_child_extended()
            {
                self.stack.push(cursor);
                self.has = Has::Down;
                self.vis.push(visibility == Visibility::Hidden);
                Some(visibility)
            } else {
                use std::ops::Deref;
                if let Some(visibility) = cursor.goto_next_sibling_extended() {
                    let _ = self.stack.pop().unwrap();
                    let c = self.stack.last_mut().unwrap();
                    if c.node().end_byte() <= cursor.node().start_byte() {
                        self.has = Has::Up;
                        let vis = if *self.vis.last().unwrap().deref() {
                            Visibility::Hidden
                        } else {
                            Visibility::Visible
                        };
                        return Some(vis);
                    }
                    self.stack.push(cursor);
                    self.vis.push(visibility == Visibility::Hidden);
                    self.has = Has::Right;
                    Some(visibility)
                } else if let Some(c) = self.stack.pop() {
                    self.has = Has::Up;
                    if self.stack.is_empty() {
                        self.stack.push(c);
                        None
                        // depends on usage
                        // let vis = if self.vis.pop().unwrap() {
                        //     Visibility::Hidden
                        // } else {
                        //     Visibility::Visible
                        // };
                        // Some(vis)
                    } else {
                        let vis = if *self.vis.last().unwrap().deref() {
                            Visibility::Hidden
                        } else {
                            Visibility::Visible
                        };
                        Some(vis)
                    }
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(feature = "ts")]
mod zipped_ts;
#[cfg(feature = "ts")]
mod zipped_ts0;
#[doc(hidden)]
#[cfg(feature = "ts")]
pub mod zipped_ts_no_goto_parent;
#[doc(hidden)]
#[cfg(feature = "ts")]
pub mod zipped_ts_no_goto_parent_a;
#[doc(hidden)]
#[cfg(feature = "ts")]
pub mod zipped_ts_simp;
#[doc(hidden)]
#[cfg(feature = "ts")]
pub mod zipped_ts_simp0;
#[doc(hidden)]
#[cfg(feature = "ts")]
pub mod zipped_ts_simp1;

pub(crate) fn things_after_last_lb<'b>(lb: &[u8], spaces: &'b [u8]) -> Option<&'b [u8]> {
    spaces
        .windows(lb.len())
        .rev()
        .position(|window| window == lb)
        .and_then(|i| Some(&spaces[spaces.len() - i - 1..]))
}

pub fn compute_indentation<'a>(
    line_break: &Vec<u8>,
    text: &'a [u8],
    pos: usize,
    padding_start: usize,
    parent_indentation: &'a [Space],
) -> Vec<Space> {
    let spaces = { &text[padding_start..pos] };
    // let spaces = text.get(padding_start.min(text.len()-1)..pos.min(text.len()));
    // let Some(spaces) = spaces else {
    //     return parent_indentation.to_vec()
    // };
    let spaces_after_lb = things_after_last_lb(&*line_break, spaces);
    match spaces_after_lb {
        Some(s) => Space::format_indentation(s),
        None => parent_indentation.to_vec(),
    }
}

pub fn try_compute_indentation<'a>(
    line_break: &Vec<u8>,
    text: &'a [u8],
    pos: usize,
    padding_start: usize,
    parent_indentation: &'a [Space],
) -> Vec<Space> {
    let spaces = { &text[padding_start..pos] };
    let spaces_after_lb = things_after_last_lb(&*line_break, spaces);
    match spaces_after_lb {
        Some(s) => Space::try_format_indentation(s).unwrap_or(parent_indentation.to_vec()),
        None => parent_indentation.to_vec(),
    }
}

pub fn get_spacing(
    padding_start: usize,
    pos: usize,
    text: &[u8],
    _parent_indentation: &Spaces,
) -> Option<Vec<u8>> {
    // TODO change debug assert to assert if you want to strictly enforce spaces, issues with other char leaking is often caused by "bad" grammar.
    if padding_start != pos {
        let spaces = &text[padding_start..pos];
        // let spaces = Space::format_indentation(spaces);
        let mut bslash = false;
        spaces.iter().for_each(|x| {
            if bslash && (*x == b'\n' || *x == b'\r') {
                bslash = false
            } else if *x == b'\\' {
                debug_assert!(!bslash);
                bslash = true
            } else {
                debug_assert!(
                    *x == b' ' || *x == b'\n' || *x == b'\t' || *x == b'\r',
                    "{} {} {:?}",
                    x,
                    padding_start,
                    std::str::from_utf8(&spaces).unwrap()
                )
            }
        });
        debug_assert!(
            !bslash,
            "{}",
            std::str::from_utf8(&&text[padding_start.saturating_sub(100)..pos + 50]).unwrap()
        );
        let spaces = spaces.to_vec();
        // let spaces = Space::replace_indentation(parent_indentation, &spaces);
        // TODO put back the relativisation later, can pose issues when computing len of a subtree (contextually if we make the optimisation)
        Some(spaces)
    } else {
        None
    }
}

pub fn try_get_spacing(
    padding_start: usize,
    pos: usize,
    text: &[u8],
    _parent_indentation: &Spaces,
) -> Option<Vec<u8>> {
    // ) -> Option<Spaces> {
    if padding_start != pos {
        let spaces = &text[padding_start..pos];
        // println!("{:?}",std::str::from_utf8(spaces).unwrap());
        if spaces
            .iter()
            .find(|&x| *x != b' ' && *x != b'\n' && *x != b'\t' && *x != b'\r')
            .is_some()
        {
            return None;
        }
        let spaces = spaces.to_vec();

        // let spaces = Space::try_format_indentation(spaces)?;
        // let spaces = Space::replace_indentation(parent_indentation, &spaces);
        // TODO put back the relativisation later, can pose issues when computing len of a subtree (contextually if we make the optimisation)
        Some(spaces)
    } else {
        None
    }
}

pub fn has_final_space(depth: &usize, sum_byte_length: usize, text: &[u8]) -> bool {
    // TODO not sure about depth
    *depth == 0 && sum_byte_length < text.len()
}

pub fn hash32<T: ?Sized + std::hash::Hash>(t: &T) -> u32 {
    crate::utils::clamp_u64_to_u32(&crate::utils::hash(t))
}

pub trait Prepro<T> {
    const USING: bool;
    fn preprocessing(&self, ty: T) -> Result<crate::scripting::Acc, String>;
}

impl<T> Prepro<T> for () {
    const USING: bool = false;
    fn preprocessing(&self, _t: T) -> Result<crate::scripting::Acc, String> {
        Ok(todo!())
    }
}

pub type PrecompQueries = u16;

pub trait More<HAST: crate::types::TypeStore, Acc> {
    const ENABLED: bool;
    fn match_precomp_queries(
        &self,
        stores: &HAST,
        acc: &Acc,
        label: Option<&str>,
    ) -> PrecompQueries;
}

impl<HAST: crate::types::TypeStore, Acc> More<HAST, Acc> for () {
    const ENABLED: bool = false;
    fn match_precomp_queries(
        &self,
        _stores: &HAST,
        _acc: &Acc,
        _label: Option<&str>,
    ) -> PrecompQueries {
        Default::default()
    }
}

pub mod metric_definition;
