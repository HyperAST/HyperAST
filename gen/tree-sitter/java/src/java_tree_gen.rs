///! first attempt at compressing subtrees
///! trash, just keep it for now beause of some ideas at the end
use std::{
    cell::Ref,
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
    vec, borrow::Borrow,
};

use rusted_gumtree_core::tree::tree::{
    LabelStore as LabelStoreTrait, NodeStore as NodeStoreTrait, NodeStoreMut as NodeStoreMutTrait, OwnedLabel, Type,
};
use tree_sitter::{Language, Parser, TreeCursor};

use crate::{
    hashed::{inner_node_hash, HashedCompressedNode, HashedNode, NodeHashs, SyntaxNodeHashs},
    nodes::{CompressedNode, LabelIdentifier, NodeIdentifier, Space},
    store::TypeStore,
    vec_map_store::VecMapStore,
};

extern "C" {
    fn tree_sitter_java() -> Language;
}

pub struct JavaTreeGen {
    pub line_break: Vec<u8>,

    pub label_store: LabelStore,
    pub type_store: TypeStore,
    pub node_store: NodeStore,
    // pub(crate) space_store: SpacesStoreD,
}

// type SpacesStoreD = SpacesStore<u16, 4>;

pub struct LabelStore {
    // internal: VecMapStore<OwnedLabel, LabelIdentifier>,
}

impl LabelStoreTrait<OwnedLabel> for LabelStore {
    type I = LabelIdentifier;
    fn get_or_insert<T: Borrow<OwnedLabel>>(&mut self, _node: T) -> Self::I {
        // self.internal.get_or_insert(node)
        todo!()
    }

    fn resolve(&self, _id: &Self::I) -> &OwnedLabel {
        // self.internal.resolve(id)
        todo!()
    }
}

pub struct NodeStore {
    internal: VecMapStore<HashedNode, NodeIdentifier>,
}

impl<'a> NodeStoreTrait<'a, NodeIdentifier,Ref<'a, HashedNode>> for NodeStore {

    fn resolve(&'a self, id: &NodeIdentifier) -> Ref<'a, HashedNode> {
        self.internal.resolve(id)
    }
}

impl<'a> NodeStoreMutTrait<'a, HashedNode,Ref<'a, HashedNode>> for NodeStore {
}
impl<'a> NodeStore {
    fn get_or_insert(&mut self, node: HashedNode) -> NodeIdentifier {
        self.internal.get_or_insert(node)
    }
}

#[derive(Debug)]
pub struct FullNode {
    compressible_node: NodeIdentifier,
    depth: usize,
    position: usize,
    height: u32,
    size: u32,
    hashs: SyntaxNodeHashs<u32>,
}

impl FullNode {
    pub fn id(&self) -> &NodeIdentifier {
        &self.compressible_node
    }
}

pub struct Acc {
    kind: Type,
    children: Vec<NodeIdentifier>,
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    padding_start: usize,
}

#[derive(Default)]
struct SubTreeMetrics<U: NodeHashs> {
    hashs: U,
    size: u32,
    height: u32,
}

impl Acc {
    pub(crate) fn new(kind: Type) -> Self {
        Self {
            kind,
            children: vec![],
            metrics: Default::default(),
            padding_start: 0,
        }
    }

    fn push(&mut self, full_node: FullNode) {
        self.children.push(full_node.compressible_node);
        self.metrics.height = self.metrics.height.max(full_node.height);
        self.metrics.size += full_node.size;
        self.metrics.hashs.acc(&full_node.hashs);
    }
}

fn hash<T: Hash>(x: &T) -> u64 {
    let mut state = DefaultHasher::default();
    x.hash(&mut state);
    state.finish()
}

fn clamp_u64_to_u32(x: &u64) -> u32 {
    (((x & 0xffff0000) >> 32) as u32).wrapping_pow((x & 0xffff) as u32)
}

impl JavaTreeGen {
    pub fn new() -> Self {
        Self {
            line_break: "\n".as_bytes().to_vec(),
            label_store: LabelStore::new(),
            type_store: TypeStore {},
            node_store: NodeStore::new(HashedCompressedNode::new(
                SyntaxNodeHashs::default(),
                CompressedNode::Spaces(vec![].into_boxed_slice()),
            )),
        }
    }

    pub fn generate_default(&mut self, text: &[u8], cursor: TreeCursor) -> FullNode {
        let mut acc_stack = vec![Acc::new(self.type_store.get("file"))];
        self.generate(text, cursor, &mut acc_stack)
    }

    pub fn generate(
        &mut self,
        text: &[u8],
        mut cursor: TreeCursor,
        mut acc_stack: &mut Vec<Acc>,
    ) -> FullNode {
        acc_stack.push(Acc {
            kind: self.type_store.get(cursor.node().kind()),
            children: vec![],
            metrics: Default::default(),
            padding_start: 0,
        });

        let mut indentation_stack: Vec<Vec<Space>> = vec![];
        indentation_stack.push(JavaTreeGen::compute_indentation(
            &self.line_break,
            text,
            &cursor,
            0,
            &Space::format_indentation(&self.line_break),
        ));

        let mut has = Has::Down;
        let mut position = 0;
        let mut depth = 1;
        let mut sum_byte_length; // = cursor.node().start_byte();

        loop {
            sum_byte_length = cursor.node().start_byte();
            if has != Has::Up && cursor.goto_first_child() {
                let parent_indentation = indentation_stack.last().unwrap();
                println!("down: {:?}", cursor.node().kind());
                has = Has::Down;
                // // self.inc(k);
                position += 1;
                depth += 1;

                let indent = JavaTreeGen::compute_indentation(
                    &self.line_break,
                    text,
                    &cursor,
                    sum_byte_length,
                    &parent_indentation,
                );
                indentation_stack.push(indent);

                acc_stack.push(Acc {
                    kind: self.type_store.get(cursor.node().kind()),
                    children: vec![],
                    metrics: Default::default(),
                    padding_start: sum_byte_length,
                });
            } else {
                let parent_indentation = indentation_stack.pop().unwrap();
                let full_node = JavaTreeGen::create_full_node(
                    text,
                    &indentation_stack.last().unwrap_or(&vec![Space::LineBreak]),
                    &mut self.node_store,
                    &mut self.label_store,
                    &mut acc_stack,
                    &mut depth,
                    &cursor,
                    sum_byte_length,
                    position,
                );
                sum_byte_length = cursor.node().end_byte();
                if cursor.goto_next_sibling() {
                    println!("right: {:?}", cursor.node().kind());
                    has = Has::Right;
                    // // self.acc(full_node);
                    {
                        let parent = acc_stack.last_mut().unwrap();
                        parent.push(full_node);
                    };
                    // // self.inc(self.kind(cursor.node().kind()));
                    {
                        position += 1;
                        depth += 1;
                        acc_stack.push(Acc {
                            kind: self.type_store.get(cursor.node().kind()),
                            children: vec![],
                            metrics: Default::default(),
                            padding_start: sum_byte_length,
                        });
                    };

                    indentation_stack.push(JavaTreeGen::compute_indentation(
                        &self.line_break,
                        text,
                        &cursor,
                        sum_byte_length,
                        &parent_indentation,
                    ));
                } else {
                    has = Has::Up;
                    if cursor.goto_parent() {
                        println!("up: {:?}", cursor.node().kind());
                        let parent = acc_stack.last_mut().unwrap();
                        parent.push(full_node);
                    } else {
                        return full_node;
                    }
                }
            }
        }
    }

    fn compute_indentation<'a>(
        line_break: &Vec<u8>,
        text: &'a [u8],
        cursor: &TreeCursor,
        padding_start: usize,
        parent_indentation: &'a [Space],
    ) -> Vec<Space> {
        let spaces = {
            let node = cursor.node();
            let pos = node.start_byte();
            &text[padding_start..pos]
        };
        let spaces_after_lb = spaces_after_lb(&*line_break, spaces);
        match spaces_after_lb {
            Some(s) => Space::format_indentation(s),
            None => parent_indentation.to_vec(),
        }
    }

    fn create_full_node(
        text: &[u8],
        old_indentation: &Vec<Space>,
        node_store: &mut NodeStore,
        label_store: &mut LabelStore,
        acc_stack: &mut Vec<Acc>,
        depth: &mut usize,
        cursor: &TreeCursor,
        sum_byte_length: usize,
        position: usize,
    ) -> FullNode {
        let node = cursor.node();
        if *depth == 0 {
            if sum_byte_length < text.len() {
                // end of tree but not end of file,
                // thus to be bijective, we need to get the last spaces
                let spaces = Space::format_indentation(&text[sum_byte_length..]);
                println!("'{:?}'", &spaces);

                let relativized = Space::replace_indentation(&[], &spaces);

                let spaces_leaf = HashedCompressedNode::new(
                    SyntaxNodeHashs {
                        structt: 0,
                        label: 0,
                        syntax: clamp_u64_to_u32(&hash(&relativized)),
                    },
                    CompressedNode::Spaces(relativized.into_boxed_slice()),
                );
                let full_spaces_node = FullNode {
                    hashs: spaces_leaf.hashs.clone(),
                    compressible_node: node_store.get_or_insert(spaces_leaf),
                    depth: *depth,
                    position,
                    size: 1,
                    height: 1,
                };
                acc_stack.last_mut().unwrap().push(full_spaces_node);
            }
        }
        let pos = node.start_byte();
        let end = node.end_byte();
        let Acc {
            children,
            kind,
            metrics:
                SubTreeMetrics {
                    hashs:
                        SyntaxNodeHashs {
                            structt: struct_middle_hash,
                            label: label_middle_hash,
                            syntax: syntax_middle_hash,
                        },
                    size,
                    height,
                },
            padding_start,
        } = acc_stack.pop().unwrap();
        println!(
            "node kind {:?} {} {} {}",
            node.kind(),
            struct_middle_hash,
            label_middle_hash,
            syntax_middle_hash
        );
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
        if padding_start != pos {
            let spaces = Space::format_indentation(&text[padding_start..pos]);
            println!(
                "ps..pos: '{:?}'",
                std::str::from_utf8(&text[padding_start..pos]).unwrap()
            );
            println!("sbl: '{:?}'", sum_byte_length);
            println!(
                "pos..end: '{:?}'",
                std::str::from_utf8(&text[pos..end]).unwrap()
            );
            let relativized = Space::replace_indentation(old_indentation, &spaces);

            let spaces_leaf = HashedCompressedNode::new(
                SyntaxNodeHashs {
                    structt: 0,
                    label: 0,
                    syntax: clamp_u64_to_u32(&hash(&relativized)),
                },
                CompressedNode::Spaces(relativized.into_boxed_slice()),
            );
            let full_spaces_node = FullNode {
                hashs: SyntaxNodeHashs {
                    ..spaces_leaf.hashs
                },
                compressible_node: node_store.get_or_insert(spaces_leaf),
                depth: *depth,
                position,
                size: 1,
                height: 1,
            };
            if acc_stack.is_empty() {
                println!("kind {:?}", kind);
            }
            println!("kind {:?}", kind);
            println!("oi {:?}", old_indentation);
            println!("s {:?}", spaces);
            println!(
                "r {:?}",
                Space::replace_indentation(old_indentation, &spaces)
            );
            println!("kind1 {:?}", acc_stack.last().unwrap().kind);
            acc_stack.last_mut().unwrap().push(full_spaces_node);
        };
        let hashed_label = &clamp_u64_to_u32(&hash(&label));
        let hashed_kind = &clamp_u64_to_u32(&hash(&kind));

        if let Some(t) = &label {
            println!("{:?} label '{:?}'", kind, std::str::from_utf8(&t));
        }
        *depth -= 1;
        let label_id = match label {
            Some(l) => Some(label_store.get_or_insert(&l)),
            None => None,
        };
        let k = *(&kind);
        println!("children {:?} {:?}", k, children.len());
        let compressible_node = HashedCompressedNode::new(
            SyntaxNodeHashs {
                structt: inner_node_hash(hashed_kind, &0, &size, &struct_middle_hash),
                label: inner_node_hash(hashed_kind, hashed_label, &size, &label_middle_hash),
                syntax: inner_node_hash(hashed_kind, hashed_label, &size, &syntax_middle_hash),
            },
            CompressedNode::new(kind, label_id, children),
        );
        println!("hash {:?} {:?}", k, compressible_node.hashs);
        let hashs = SyntaxNodeHashs {
            ..compressible_node.hashs
        };
        let compressible_node_id = node_store.get_or_insert(compressible_node);
        print_tree_syntax(node_store, label_store, &compressible_node_id);
        println!();
        let full_node = FullNode {
            compressible_node: compressible_node_id,
            depth: *depth,
            position,
            size,
            height,
            hashs,
        };
        full_node
    }

    pub fn main() {
        let mut parser = Parser::new();

        {
            let language = unsafe { tree_sitter_java() };
            parser.set_language(language).unwrap();
        }

        let text = {
            let source_code1 = "class A {void test() {}}";
            source_code1.as_bytes()
        };
        // let mut parser: Parser, old_tree: Option<&Tree>
        let tree = parser.parse(text, None).unwrap();
        let mut java_tree_gen = JavaTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            label_store: LabelStore::new(),
            type_store: TypeStore {},
            node_store: NodeStore::new(HashedCompressedNode::new(
                SyntaxNodeHashs {
                    structt: 0,
                    label: 0,
                    syntax: 0,
                },
                CompressedNode::Spaces(vec![].into_boxed_slice()),
            )),
        };
        let mut acc_stack = vec![Acc {
            kind: java_tree_gen.type_store.get("File"),
            children: vec![],
            metrics: Default::default(),
            padding_start: 0,
        }];
        let _full_node = java_tree_gen.generate(text, tree.walk(), &mut acc_stack);

        print_tree_structure(&java_tree_gen.node_store, &_full_node.compressible_node);

        let mut acc_stack = vec![Acc {
            kind: java_tree_gen.type_store.get("File"),
            children: vec![],
            metrics: Default::default(),
            padding_start: 0,
        }];
        let tree = parser.parse(text, Some(&tree)).unwrap();
        let _full_node = java_tree_gen.generate(text, tree.walk(), &mut acc_stack);
    }
    // fn generate<'a>(&mut self, text: &'a [u8], tc: TreeContext, init_acc:ChildrenAcc<'a>) -> FullNode {
    //     let mut tree = self.parser.parse(text, self.old_tree.as_ref()).unwrap();
    //     println!("{}", tree.root_node().to_sexp());
    //     let full_node = self.build_compressed(text, &mut tree, tc, init_acc);
    //     self.old_tree = Option::Some(tree);
    //     full_node
    // }
}

pub fn print_tree_structure(node_store: &NodeStore, id: &NodeIdentifier) {
    let node = node_store.resolve(id);
    // let children: Option<Vec<NodeIdentifier>> =
    match &node.node {
        CompressedNode::Type(kind) => {
            print!("{}", kind.to_string());
            // None
        }
        CompressedNode::Label { kind, label: _ } => {
            print!("({})", kind.to_string());
            // None
        }
        CompressedNode::Children2 { kind, children } => {
            print!("({} ", kind.to_string());
            for id in children {
                print_tree_structure(node_store, &id);
            }
            print!(")");
        }
        CompressedNode::Children { kind, children } => {
            print!("({} ", kind.to_string());
            let children = children.clone();
            for id in children.iter() {
                print_tree_structure(node_store, &id);
            }
            print!(")");
        }
        CompressedNode::Spaces(_) => (),
    };
}

pub fn print_tree_labels(node_store: &NodeStore, label_store: &LabelStore, id: &NodeIdentifier) {
    let node = node_store.resolve(id);
    // let children: Option<Vec<NodeIdentifier>> =
    match &node.node {
        CompressedNode::Type(kind) => {
            print!("{}", kind.to_string());
            // None
        }
        CompressedNode::Label { kind, label } => {
            let s = &label_store.resolve(label);
            if s.len() > 20 {
                print!(
                    "({}='{}...')",
                    kind.to_string(),
                    std::str::from_utf8(&s[..20]).unwrap()
                );
            } else {
                print!(
                    "({}='{}')",
                    kind.to_string(),
                    std::str::from_utf8(s).unwrap()
                );
            }
            // None
        }
        CompressedNode::Children2 { kind, children } => {
            print!("({} ", kind.to_string());
            for id in children {
                print_tree_labels(node_store, label_store, &id);
            }
            print!(")");
        }
        CompressedNode::Children { kind, children } => {
            print!("({} ", kind.to_string());
            let children = children.clone();
            for id in children.iter() {
                print_tree_labels(node_store, label_store, &id);
            }
            print!(")");
        }
        CompressedNode::Spaces(_) => (),
    };
}

pub fn print_tree_syntax(node_store: &NodeStore, label_store: &LabelStore, id: &NodeIdentifier) {
    let node = node_store.resolve(id);
    // let children: Option<Vec<NodeIdentifier>> =
    match &node.node {
        CompressedNode::Type(kind) => {
            print!("{}", kind.to_string());
            // None
        }
        CompressedNode::Label { kind, label } => {
            let s = &label_store.resolve(label);
            if s.len() > 20 {
                print!(
                    "({}='{}...')",
                    kind.to_string(),
                    std::str::from_utf8(&s[..20]).unwrap()
                );
            } else {
                print!(
                    "({}='{}')",
                    kind.to_string(),
                    std::str::from_utf8(s).unwrap()
                );
            }
            // None
        }
        CompressedNode::Children2 { kind, children } => {
            print!("({} ", kind.to_string());
            for id in children {
                print_tree_syntax(node_store, label_store, &id);
            }
            print!(")");
        }
        CompressedNode::Children { kind, children } => {
            print!("({} ", kind.to_string());
            let children = children.clone();
            for id in children.iter() {
                print_tree_syntax(node_store, label_store, &id);
            }
            print!(")");
        }
        CompressedNode::Spaces(s) => {
            print!("(_ ");
            let a = &**s;
            a.iter().for_each(|a| print!("{:?}", a));
            print!(")");
        }
    };
}

pub fn serialize<W: std::fmt::Write>(
    node_store: &NodeStore,
    label_store: &LabelStore,
    id: &NodeIdentifier,
    out: &mut W,
    parent_indent: &str,
) -> Option<String> {
    let node = node_store.resolve(id);
    match &node.node {
        CompressedNode::Type(kind) => {
            out.write_str(&kind.to_string()).unwrap();
            // out.write_fmt(format_args!("{}",kind.to_string())).unwrap();
            None
        }
        CompressedNode::Label { kind: _, label } => {
            let s = &label_store.resolve(label);
            out.write_str(&std::str::from_utf8(s).unwrap()).unwrap();
            // write!(&mut out, "{}", std::str::from_utf8(s).unwrap()).unwrap();
            None
        }
        CompressedNode::Children2 { kind: _, children } => {
            let ind = serialize(node_store, label_store, &children[0], out, parent_indent)
                .unwrap_or(parent_indent.to_owned());
            serialize(node_store, label_store, &children[1], out, &ind);
            None
        }
        CompressedNode::Children { kind: _, children } => {
            let children = &(**children);
            // writeln!(out, "{:?}", children).unwrap();
            // writeln!(out, "{:?}", kind).unwrap();
            let mut it = children.iter();
            let mut ind = serialize(
                node_store,
                label_store,
                &it.next().unwrap(),
                out,
                parent_indent,
            )
            .unwrap_or(parent_indent.to_owned());
            for id in it {
                ind = serialize(node_store, label_store, &id, out, &ind)
                    .unwrap_or(parent_indent.to_owned());
            }
            None
        }
        CompressedNode::Spaces(s) => {
            let a = &**s;
            let mut b = String::new();
            // let mut b = format!("{:#?}", a);
            // fmt::format(args)
            a.iter()
                .for_each(|a| Space::fmt(a, &mut b, parent_indent).unwrap());
            // std::io::Write::write_all(out, "<|".as_bytes()).unwrap();
            // std::io::Write::write_all(out, parent_indent.replace("\n", "n").as_bytes()).unwrap();
            // std::io::Write::write_all(out, "|>".as_bytes()).unwrap();
            out.write_str(&b).unwrap();
            Some(if b.contains("\n") {
                b
            } else {
                parent_indent.to_owned()
            })
        }
    }
}

#[derive(PartialEq, Eq)]
enum Has {
    Down,
    Up,
    Right,
}

pub(crate) fn spaces_after_lb<'b>(lb: &[u8], spaces: &'b [u8]) -> Option<&'b [u8]> {
    spaces
        .windows(lb.len())
        .rev()
        .position(|window| window == lb)
        .and_then(|i| Some(&spaces[spaces.len() - i - 1..]))
}

impl NodeStore {
    pub(crate) fn new(filling_element: HashedNode) -> Self {
        Self {
            internal: VecMapStore::new(filling_element),
        }
    }
}

impl LabelStore {
    pub(crate) fn new() -> Self {
        Self {
            // internal: VecMapStore::new(vec![]),
        }
    }
}

// pub(crate) fn format_indentation_windows(spaces: &[u8]) -> Vec<Spaces> {
//     const line_break:&[u8] = "\r\n".as_bytes();
//     let mut it = spaces.windows(line_break.len());
//     let mut r: Vec<Spaces> = vec![];
//     loop {
//         match it.next() {
//             Some(x) => {
//                 if x == line_break {
//                     r.push(Spaces::LineBreak);
//                     for _ in 0..line_break.len() {
//                         it.next();
//                     }
//                 } else if ' ' as u8 == x[0] {
//                     r.push(Spaces::Space);
//                 } else if '\t' as u8 == x[0] {
//                     r.push(Spaces::Tabulation);
//                 } else {
//                     println!("not a space: {:?}", String::from_utf8(x.to_vec()));
//                     panic!()
//                 }
//             }
//             None => return r,
//         }
//     }
// }

// pub(crate) fn replace_indentation_old<'b>(indentation: &[u8], spaces: &'b [u8]) -> Vec<Spaces> {
//     let mut it = spaces.windows(indentation.len());
//     // .windows(|i| Some(&spaces[spaces.len() - i..]));
//     let mut r: Vec<Spaces> = vec![];
//     // let mut old = 0;
//     loop {
//         match it.next() {
//             Some(x) => {
//                 if x == indentation {
//                     r.push(Spaces::ParentIndentation);
//                     for _ in 0..indentation.len() {
//                         it.next();
//                     }
//                 } else if ' ' as u8 == x[0] {
//                     r.push(Spaces::Space);
//                 // } else if '\n' as u8 == x[0] {
//                 //     r.push(Spaces::NewLine);
//                 // } else if '\r' as u8 == x[0] {
//                 //     r.push(Spaces::CariageReturn);
//                 } else if '\t' as u8 == x[0] {
//                     r.push(Spaces::Tabulation);
//                 } else {
//                     println!("not a space: {:?}", String::from_utf8(x.to_vec()));
//                     panic!()
//                 }
//             }
//             None => return r,
//         }
//     }
// }

// #[derive(Default)]
// struct LabelStore {
//     hash_table: HashSet<String>,
// }

// impl LabelStore {
//     fn get(&mut self, label: &str) -> &str {
//         if self.hash_table.contains(label) {
//             self.hash_table.get(label).unwrap()
//         } else {
//             self.hash_table.insert(label.to_owned());
//             self.hash_table.get(label).unwrap()
//         }
//     }
// }

// pub struct VecHasher<T: Hash> {
//     state: u64,
//     node_table: Rc<Vec<T>>,
//     default: DefaultHasher,
// }

// impl<T: Hash> Hasher for VecHasher<T> {
//     fn write_u16(&mut self, i: u16) {
//         let a = &self.node_table;
//         let b = &a[i as usize];
//         b.hash(&mut self.default);
//         self.state = self.default.finish();
//     }
//     fn write(&mut self, bytes: &[u8]) {
//         // for &byte in bytes {
//         //     self.state = self.state.rotate_left(8) ^ u64::from(byte);
//         // }
//         panic!()
//     }

//     fn finish(&self) -> u64 {
//         self.state
//     }
// }

// impl<T: Hash> VecHasher<T> {
//     fn hash_identifier(&mut self, id: &NodeIdentifier) {}
// }

// pub(crate) struct BuildVecHasher<T> {
//     node_table: Rc<Vec<T>>,
// }

// impl<T: Hash> std::hash::BuildHasher for BuildVecHasher<T> {
//     type Hasher = VecHasher<T>;
//     fn build_hasher(&self) -> VecHasher<T> {
//         VecHasher {
//             state: 0,
//             node_table: self.node_table.clone(),
//             default: DefaultHasher::new(),
//         }
//     }
// }

// struct NodeStore {
//     hash_table: HashSet<NodeStoreEntry, BuildVecHasher<CompressedNode>>,
//     node_table: Rc<Vec<CompressedNode>>,
//     counter: ConsistentCounter,
// }

// impl Default for NodeStore {
//     fn default() -> Self {
//         let node_table: Rc<Vec<CompressedNode>> = Default::default();
//         Self {
//             hash_table: std::collections::HashSet::with_hasher(BuildVecHasher {
//                 node_table: node_table.clone(),
//             }),
//             node_table,
//             counter: Default::default(),
//         }
//     }
// }

// struct NodeStoreEntry {
//     node: NodeIdentifier,
// }

// impl Hash for NodeStoreEntry {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         state.write_u16(self.node);
//         // CustomHasher::hash_identifier(state, &self.node);
//         // self.hash(state);
//     }
//     // fn hash(&self, state: &mut VecHasher<CompressibleNode>) {
//     //     // if TypeId::of::<H>() == TypeId::of::<VecHasher<CompressibleNode>>() {

//     //     // }
//     //     // CustomHasher::hash_identifier(state, &self.node);
//     //     // self.hash(state);
//     // }
// }

// impl PartialEq for NodeStoreEntry {
//     fn eq(&self, other: &Self) -> bool {
//         self.node == other.node
//     }
// }

// impl Eq for NodeStoreEntry {}

// impl NodeStore {
//     fn get_id_or_insert_node(&mut self, node: CompressedNode) -> NodeIdentifier {
//         let entry = NodeStoreEntry { node: 0 };
//         if self.hash_table.contains(&entry) {
//             self.hash_table.get(&entry).unwrap().node
//         } else {
//             let entry_to_insert = NodeStoreEntry {
//                 node: self.counter.get() as NodeIdentifier,
//             };
//             self.counter.inc();
//             self.hash_table.insert(entry_to_insert);
//             self.hash_table.get(&entry).unwrap().node
//         }
//     }

//     fn get_node_at_id(&self, id: &NodeIdentifier) -> &CompressedNode {
//         &self.node_table[*id as usize]
//     }
// }
