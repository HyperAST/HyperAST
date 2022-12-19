pub mod parser;

// use std::hash::Hash;

use crate::{
    hashed::NodeHashs,
    // hashed::{inner_node_hash, SyntaxNodeHashs},
    nodes::Space,
    // utils::{self, clamp_u64_to_u32},
};
// use crate::nodes::SimpleNode1;

use self::parser::{Node as _, TreeCursor as _};

pub type Spaces = Vec<Space>;

/// Builder of a node for the hyperAST
pub trait Accumulator {
    type Node;
    fn push(&mut self, full_node: Self::Node);
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
}

impl<U: NodeHashs> SubTreeMetrics<U> {
    pub fn acc(&mut self, other: Self) {
        self.height = self.height.max(other.height);
        self.size += other.size;
        self.size_no_spaces += other.size_no_spaces;
        self.hashs.acc(&other.hashs);
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

pub trait TreeGen {
    type Acc: AccIndentation;
    type Global: GlobalData;
    fn make(
        &mut self,
        global: &mut Self::Global,
        acc: <Self as TreeGen>::Acc,
        label: Option<String>,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node;
}

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
    type TreeCursor<'a>: parser::TreeCursor<'a, Self::Node<'a>>;

    fn init_val(&mut self, text: &Self::Text, node: &Self::Node<'_>) -> Self::Acc;

    fn pre(
        &mut self,
        text: &Self::Text,
        node: &Self::Node<'_>,
        stack: &Vec<Self::Acc>,
        global: &mut Self::Global,
    ) -> <Self as TreeGen>::Acc;

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
        stack: &mut Vec<Self::Acc>,
        cursor: &mut Self::TreeCursor<'_>,
        global: &mut Self::Global,
    ) {
        let mut has = Has::Down;
        loop {
            let sbl = cursor.node().start_byte();
            if has != Has::Up && cursor.goto_first_child() {
                global.set_sum_byte_length(sbl);
                has = Has::Down;
                global.down();

                let n = self.pre(text, &cursor.node(), &stack, global);

                stack.push(n);
            } else {
                let acc = stack.pop().unwrap();
                global.up();

                let full_node: Option<_> = if let Some(parent) = stack.last_mut() {
                    Some(self.post(parent, global, text, acc))
                } else {
                    stack.push(acc);
                    None
                };

                let sbl = cursor.node().end_byte();
                if cursor.goto_next_sibling() {
                    global.set_sum_byte_length(sbl);
                    has = Has::Right;
                    let parent = stack.last_mut().unwrap();
                    parent.push(full_node.unwrap());
                    global.down();
                    let n = self.pre(text, &cursor.node(), &stack, global);
                    stack.push(n);
                } else {
                    has = Has::Up;
                    if cursor.goto_parent() {
                        if let Some(full_node) = full_node {
                            global.set_sum_byte_length(sbl);
                            stack.last_mut().unwrap().push(full_node);
                        } else {
                            if has == Has::Down {
                                global.set_sum_byte_length(sbl);
                            }
                            return;
                        }
                    } else {
                        if has == Has::Down {
                            global.set_sum_byte_length(sbl);
                        }
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
    if padding_start != pos {
        let spaces = &text[padding_start..pos];
        // let spaces = Space::format_indentation(spaces);
        spaces.iter().for_each(|x| {
            assert!(
                *x == b' ' || *x == b'\n' || *x == b'\t' || *x == b'\r',
                "{} {} {:?}",
                x,
                padding_start,
                std::str::from_utf8(&spaces).unwrap()
            )
        });
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
