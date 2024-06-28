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

use self::parser::{Node as _, TreeCursor as _, Visibility};

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
}
impl<T: Debug, Id> Debug for BasicAccumulator<T, Id> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BasicAccumulator")
            .field("kind", &self.kind)
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
pub struct SubTreeMetrics<U: NodeHashs> {
    pub hashs: U,
    /// WIP make it space independent
    pub size: u32,
    /// WIP make it space independent, I believe is already is
    pub height: u32,

    pub size_no_spaces: u32,
    pub line_count: u16, 
}

impl<U: NodeHashs> SubTreeMetrics<U> {
    pub fn acc(&mut self, other: Self) {
        self.height = self.height.max(other.height);
        self.size += other.size;
        self.size_no_spaces += other.size_no_spaces;
        self.hashs.acc(&other.hashs);
        self.line_count += other.line_count;
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
        self.depth -= 0;
    }

    fn right(&mut self) {
        self.position += 1;
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
        assert!(self.sum_byte_length <= sum_byte_length);
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
}

pub enum PreResult<Acc> {
    /// Do not process node and its children
    Skip,
    /// Do not process node (but process children)
    Ignore,
    /// Do not process children
    SkipChildren(Acc),
    Ok(Acc),
}

/// Define a zipped visitor, where you mostly have to implement,
/// [`ZippedTreeGen::pre`] going down,
/// and [`ZippedTreeGen::post`] going up in the traversal.
pub trait ZippedTreeGen: TreeGen
where
    Self::Global: TotalBytesGlobalData,
{
    // # results
    // type Node1;
    type Stores;
    // # source
    type Text: ?Sized;
    type Node<'a>: parser::Node<'a>;
    type TreeCursor<'a>: parser::TreeCursor<'a, Self::Node<'a>> + Debug;

    fn init_val(&mut self, text: &Self::Text, node: &Self::Node<'_>) -> Self::Acc;

    /// Can be implemented if you want to skip certain nodes,
    /// note that skipping only act on the "overlay" tree structure,
    /// meaning that the content of a skipped node is fed to its parents
    ///
    /// The default implementation skips nothing.
    ///
    ///  see also also the following example use:
    /// [`hyper_ast_gen_ts_cpp::legion::CppTreeGen::pre_skippable`](../../hyper_ast_gen_ts_cpp/legion/struct.CppTreeGen.html#method.pre_skippable)
    fn pre_skippable(
        &mut self,
        text: &Self::Text,
        cursor: &Self::TreeCursor<'_>,
        stack: &Parents<Self::Acc>,
        global: &mut Self::Global,
    ) -> PreResult<<Self as TreeGen>::Acc> {
        PreResult::Ok(self.pre(text, &cursor.node(), stack, global))
    }

    /// Called when going up
    fn pre(
        &mut self,
        text: &Self::Text,
        node: &Self::Node<'_>,
        // TODO make a special wrapper for the Vec<Option<_>>
        stack: &Parents<Self::Acc>,
        global: &mut Self::Global,
    ) -> <Self as TreeGen>::Acc;

    /// Called when going up
    fn post(
        &mut self,
        parent: &mut <Self as TreeGen>::Acc,
        global: &mut Self::Global,
        text: &Self::Text,
        acc: <Self as TreeGen>::Acc,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node;

    fn stores(&mut self) -> &mut Self::Stores;

    fn gen(
        &mut self,
        text: &Self::Text,
        stack: &mut Parents<Self::Acc>,
        cursor: &mut Self::TreeCursor<'_>,
        global: &mut Self::Global,
    ) {
        let mut has = Has::Down;
        loop {
            if has != Has::Up
                && let Some(visibility) = cursor.goto_first_child_extended()
            {
                has = Has::Down;
                global.down();
                let n = self.pre_skippable(text, cursor, &stack, global);
                match n {
                    PreResult::Skip => {
                        has = Has::Up;
                        global.up();
                    }
                    PreResult::Ignore => {
                        if let Visibility::Visible = visibility {
                            stack.push(P::ManualyHidden);
                        } else {
                            stack.push(P::BothHidden);
                        }
                    }
                    PreResult::SkipChildren(acc) => {
                        has = Has::Up;
                        if let Visibility::Visible = visibility {
                            stack.push(P::Visible(acc));
                        } else {
                            unimplemented!("Only concrete nodes should be leafs")
                        }
                    }
                    PreResult::Ok(acc) => {
                        global.set_sum_byte_length(acc.begin_byte());
                        if let Visibility::Visible = visibility {
                            stack.push(P::Visible(acc));
                        } else {
                            stack.push(P::Hidden(acc));
                        }
                    }
                }
            } else {
                let is_visible;
                let is_parent_hidden;
                let full_node: Option<_> = match (stack.pop().unwrap(), stack.parent_mut_with_vis())
                {
                    (P::Visible(acc), None) => {
                        global.up();
                        is_visible = true;
                        is_parent_hidden = false;
                        //global.set_sum_byte_length(acc.end_byte());
                        stack.push(P::Visible(acc));
                        None
                    }
                    (_, None) => {
                        panic!();
                    }
                    (P::ManualyHidden, Some((v, _))) => {
                        is_visible = false;
                        is_parent_hidden = v == Visibility::Hidden;
                        None
                    }
                    (P::BothHidden, Some((v, _))) => {
                        is_visible = false;
                        is_parent_hidden = v == Visibility::Hidden;
                        None
                    }
                    (P::Visible(acc), Some((v, parent))) => {
                        is_visible = true;
                        is_parent_hidden = v == Visibility::Hidden;
                        if !acc.has_children() {
                            global.set_sum_byte_length(acc.end_byte());
                        }
                        if is_parent_hidden && parent.end_byte() <= acc.begin_byte() {
                            panic!()
                        }
                        global.up();
                        let full_node = self.post(parent, global, text, acc);
                        Some(full_node)
                    }
                    (P::Hidden(acc), Some((v, parent))) => {
                        is_visible = false;
                        is_parent_hidden = v == Visibility::Hidden;
                        if !acc.has_children() {
                            global.set_sum_byte_length(acc.end_byte());
                        }
                        if is_parent_hidden && parent.end_byte() <= acc.begin_byte() {
                            panic!()
                        }
                        global.up();
                        let full_node = self.post(parent, global, text, acc);
                        Some(full_node)
                    }
                };

                // TODO opt out of using end_byte other than on leafs,
                // it should help with trailing spaces,
                // something like `cursor.node().child_count().ne(0).then(||cursor.node().end_byte())` then just call set_sum_byte_length if some
                if let Some(visibility) = cursor.goto_next_sibling_extended() {
                    has = Has::Right;
                    let parent = stack.parent_mut().unwrap();
                    if let Some(full_node) = full_node {
                        parent.push(full_node);
                    }
                    loop {
                        let parent = stack.parent_mut().unwrap();
                        if parent.end_byte() <= cursor.node().start_byte() {
                            loop {
                                let p = stack.pop().unwrap();
                                match p {
                                    P::ManualyHidden => (),
                                    P::BothHidden => (),
                                    P::Hidden(acc) => {
                                        let parent = stack.parent_mut().unwrap();
                                        let full_node = self.post(parent, global, text, acc);
                                        parent.push(full_node);
                                        break;
                                    }
                                    P::Visible(acc) => {
                                        let parent = stack.parent_mut().unwrap();
                                        let full_node = self.post(parent, global, text, acc);
                                        parent.push(full_node);
                                        break;
                                    }
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    global.down();
                    let n = self.pre_skippable(text, cursor, &stack, global);
                    match n {
                        PreResult::Skip => {
                            has = Has::Up;
                            global.up();
                        }
                        PreResult::Ignore => {
                            if let Visibility::Visible = visibility {
                                stack.push(P::ManualyHidden);
                            } else {
                                stack.push(P::BothHidden);
                            }
                        }
                        PreResult::SkipChildren(acc) => {
                            has = Has::Up;
                            if let Visibility::Visible = visibility {
                                stack.push(P::Visible(acc));
                            } else {
                                unimplemented!("Only concrete nodes should be leafs")
                            }
                        }
                        PreResult::Ok(acc) => {
                            global.set_sum_byte_length(acc.begin_byte());
                            if let Visibility::Visible = visibility {
                                stack.push(P::Visible(acc));
                            } else {
                                stack.push(P::Hidden(acc));
                            }
                        }
                    }
                } else {
                    has = Has::Up;
                    if is_parent_hidden || stack.0.last().map_or(false, P::is_both_hidden) {
                        if let Some(full_node) = full_node {
                            let parent = stack.parent_mut().unwrap();
                            parent.push(full_node);
                        }
                    } else if cursor.goto_parent() {
                        if let Some(full_node) = full_node {
                            let parent = stack.parent_mut().unwrap();
                            parent.push(full_node);
                        } else if is_visible {
                            if has == Has::Down {}
                            return;
                        }
                    } else {
                        assert!(full_node.is_none());
                        if has == Has::Down {}
                        return;
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Eq)]
enum Has {
    Down,
    Up,
    Right,
}

pub(crate) fn things_after_last_lb<'b>(lb: &[u8], spaces: &'b [u8]) -> Option<&'b [u8]> {
    spaces
        .windows(lb.len())
        .rev()
        .position(|window| window == lb)
        .and_then(|i| Some(&spaces[spaces.len() - i - 1..]))
}

// pub fn hash_for_node<T: Hash, U>(
//     hashs: &SyntaxNodeHashs<u32>,
//     size: u32,
//     node: &SimpleNode1<U, T>,
// ) -> SyntaxNodeHashs<u32> {
//     let hashed_kind = clamp_u64_to_u32(&utils::hash(&node.kind));
//     let hashed_label = clamp_u64_to_u32(&utils::hash(&node.label));
//     SyntaxNodeHashs {
//         structt: inner_node_hash(hashed_kind, 0, size, hashs.structt),
//         label: inner_node_hash(hashed_kind, hashed_label, size, hashs.label),
//         syntax: inner_node_hash(hashed_kind, hashed_label, size, hashs.syntax),
//     }
// }

pub fn compute_indentation<'a>(
    line_break: &Vec<u8>,
    text: &'a [u8],
    pos: usize,
    padding_start: usize,
    parent_indentation: &'a [Space],
) -> Vec<Space> {
    let spaces = { &text[padding_start..pos] };
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

// pub fn handle_spacing<
//     NS: NodeStore<HashedCompressedNode<SyntaxNodeHashs<u32>>>,
//     Acc: AccIndentation<Node = FullNode<Global, Local>>,
// >(
//     padding_start: usize,
//     pos: usize,
//     text: &[u8],
//     node_store: &mut NS,
//     depth: &usize,
//     position: usize,
//     parent: &mut Acc,
// ) {
//     let tmp = get_spacing(padding_start, pos, text, parent.indentation());
//     if let Some(relativized) = tmp {
//         let hashs = SyntaxNodeHashs {
//             structt: 0,
//             label: 0,
//             syntax: clamp_u64_to_u32(&utils::hash(&relativized)),
//         };
//         let node = CompressedNode::Spaces(relativized.into_boxed_slice());
//         let spaces_leaf = HashedCompressedNode::new(hashs, node);
//         let compressed_node = node_store.get_id_or_insert_node(spaces_leaf);
//         let full_spaces_node = FullNode {
//             global: Global {
//                 depth: *depth,
//                 position,
//             },
//             local: Local {
//                 compressed_node,
//                 metrics: SubTreeMetrics {
//                     size: 1,
//                     height: 1,
//                     hashs,
//                 },
//             },
//         };
//         parent.push(full_spaces_node);
//     };
// }

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

// /// end of tree but not end of file,
// /// thus to be a bijection, we need to get the last spaces
// pub fn handle_final_space<
//     NS: NodeStore<HashedCompressedNode<SyntaxNodeHashs<u32>>>,
//     Acc: AccIndentation<Node = FullNode<Global, Local>>,
// >(
//     depth: &usize,
//     sum_byte_length: usize,
//     text: &[u8],
//     node_store: &mut NS,
//     position: usize,
//     parent: &mut Acc,
// ) {
//     if has_final_space(depth, sum_byte_length, text) {
//         handle_spacing(
//             sum_byte_length,
//             text.len(),
//             text,
//             node_store,
//             depth,
//             position,
//             parent,
//         )
//     }
// }
