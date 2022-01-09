use std::hash::Hash;

use crate::{
    hashed::{inner_node_hash, SyntaxNodeHashs},
    nodes::{SimpleNode1, Space},
    utils::{self, clamp_u64_to_u32},
};

pub type Spaces = Vec<Space>;

pub trait Accumulator {
    type Node;
    fn push(&mut self, full_node: Self::Node);
}


pub struct BasicAccumulator<T,Id> {
    pub(crate) kind: T,
    pub(crate) children: Vec<Id>,
}

impl<T,Id> BasicAccumulator<T,Id> {
    pub(crate) fn new(kind: T) -> Self {
        Self {
            kind,
            children: vec![],
        }
    }
}

impl<T,Id> Accumulator for BasicAccumulator<T,Id> {
    type Node = Id;
    fn push(&mut self, node: Self::Node) {
        self.children.push(node);
    }
}

pub trait AccIndentation: Accumulator {
    fn indentation<'a>(&'a self) -> &'a Spaces;
}

pub trait TreeGen {
    type Node1;
    type Acc: AccIndentation;
    type Stores;
    type Text: ?Sized;

    fn init_val(&mut self, text: &Self::Text, node: &tree_sitter::Node) -> Self::Acc;

    fn pre(
        &mut self,
        text: &Self::Text,
        node: &tree_sitter::Node,
        stack: &Vec<Self::Acc>,
        sum_byte_length: usize,
    ) -> <Self as TreeGen>::Acc;

    fn post(
        &mut self,
        parent: &mut <Self as TreeGen>::Acc,
        depth: usize,
        position: usize,
        text: &Self::Text,
        node: &tree_sitter::Node,
        acc: <Self as TreeGen>::Acc,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node;

    fn stores(&mut self) -> &mut Self::Stores;

    fn gen(
        &mut self,
        text: &Self::Text,
        stack: &mut Vec<Self::Acc>,
        cursor: &mut tree_sitter::TreeCursor,
    ) -> usize {
        let mut has = Has::Down;
        let mut position = 0;
        let mut depth = 1;
        let mut sum_byte_length;

        loop {
            sum_byte_length = cursor.node().start_byte();
            if has != Has::Up && cursor.goto_first_child() {
                has = Has::Down;
                position += 1;
                depth += 1;

                let n = self.pre(text, &cursor.node(), &stack, sum_byte_length);

                stack.push(n);
            } else {
                let acc = stack.pop().unwrap();
                depth -= 1;

                let full_node: Option<_> = if let Some(parent) = stack.last_mut() {
                    Some(self.post(parent, depth, position, text, &cursor.node(), acc))
                } else {
                    stack.push(acc);
                    None
                };

                sum_byte_length = cursor.node().end_byte();
                if cursor.goto_next_sibling() {
                    has = Has::Right;
                    let parent = stack.last_mut().unwrap();
                    parent.push(full_node.unwrap());
                    position += 1;
                    depth += 1;
                    let n = self.pre(text, &cursor.node(), &stack, sum_byte_length);
                    stack.push(n);
                } else {
                    has = Has::Up;
                    if cursor.goto_parent() {
                        if let Some(full_node) = full_node {
                            stack.last_mut().unwrap().push(full_node);
                        } else {
                            return sum_byte_length;
                        }
                    } else {
                        return sum_byte_length;
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

pub fn label_for_cursor(text: &[u8], node: &tree_sitter::Node) -> Option<Vec<u8>> {
    let pos = node.start_byte();
    let end = node.end_byte();
    let label = {
        if node.child(0).is_some() {
            None
        } else if node.is_named() {
            let t = &text[pos..end];
            Some(t.to_vec())
        } else {
            None
        }
    };
    label
}

pub fn hash_for_node<T: Hash, U>(
    hashs: &SyntaxNodeHashs<u32>,
    size: &u32,
    node: &SimpleNode1<U, T>,
) -> SyntaxNodeHashs<u32> {
    let hashed_kind = &clamp_u64_to_u32(&utils::hash(&node.kind));
    let hashed_label = &clamp_u64_to_u32(&utils::hash(&node.label));
    SyntaxNodeHashs {
        structt: inner_node_hash(hashed_kind, &0, size, &hashs.structt),
        label: inner_node_hash(hashed_kind, hashed_label, size, &hashs.label),
        syntax: inner_node_hash(hashed_kind, hashed_label, size, &hashs.syntax),
    }
}

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
    parent_indentation: &Spaces,
) -> Option<Spaces> {
    if padding_start != pos {
        let spaces = &text[padding_start..pos];
        let spaces = Space::format_indentation(spaces);
        let relativized = Space::replace_indentation(parent_indentation, &spaces);
        Some(relativized)
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
