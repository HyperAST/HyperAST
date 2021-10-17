use std::any::TypeId;
use std::process::Command;
use std::{io::BufReader, rc::Rc, str::FromStr};

use atomic_counter::RelaxedCounter;
use num::PrimInt;
use rusted_gumtree_core::tree::static_analysis::{Declaration, QualifiedName};
use rusted_gumtree_core::tree::tree::{Label, Type};
use tree_sitter::{Language, Node, Parser, Tree, TreeCursor};

pub trait BasicNode {
    fn typ(&self) -> Type;
}
pub trait LabeledNode: BasicNode {
    fn label(&self) -> String;
}
pub trait BasicTree: BasicNode {
    fn getChildren(&self) -> Vec<Rc<&dyn BasicNode>>;
}
pub trait GtTree: LabeledNode {}

#[derive(PartialEq, Eq, Debug)]
pub enum CompressibleTree {}

#[derive(PartialEq, Eq, Debug)]
pub struct DecompressedTree {
    parent: Box<DecompressedTree>,
    compressed: CompressibleTree,
}

impl BasicNode for DecompressedTree {
    fn typ(&self) -> Type {
        todo!()
    }
}
struct Storage {}
// #[derive(PartialEq, Eq, Debug)]
pub struct TreeContext<'a> {
    storage: &'a mut Storage
    // get_root_acc: &dyn FnOnce(String) -> ChildrenAcc,
    // root: Option<DecompressedTree>,
    // acc: ChildrenAcc<'a>,
}

pub struct JavaTreeGen {
    parser: Parser,
    old_tree: Option<Tree>,
    // pub tree_context: TreeContext, // TODO
}

pub trait TreeGenerator {
    fn generate<'a>(&mut self, text: &'a [u8], tc: TreeContext, init_acc: ChildrenAcc) -> FullNode;
    // fn generate_utf16<'a>(&mut self, text: &'a [u16], tc: TreeContext) -> FullNode;
}

extern "C" {
    fn tree_sitter_java() -> Language;
}

impl JavaTreeGen {
    pub fn new(// tree_context: TreeContext
    ) -> Self {
        let mut parser = Parser::new();

        let language = unsafe { tree_sitter_java() };
        parser.set_language(language).unwrap();

        Self {
            parser,
            old_tree: None,
            // tree_context,
        }
    }
}

impl Default for JavaTreeGen {
    fn default() -> Self {
        JavaTreeGen::new(
        // TreeContext {
        //     acc: ChildrenAcc::new(Type::new("program")),
        // }
        )
    }
}

pub(crate) struct ChildrenAcc<'a> {
    kind: &'a Type,
    label: Option<&'a Label>,
}

impl<'a> ChildrenAcc<'a> {
    fn new(kind: &'a Type) -> Self {
        Self { 
            kind,
            label: None,
        }
    }

    fn to(&self, kind: &'a Type) -> ChildrenAcc {
        ChildrenAcc::new(kind)
    }

    fn add(&mut self, full_node: FullNode) {
        todo!()
    }

    fn create_full_node(
        &self,
        tree_context: &mut TreeContext,
        depth: usize,
        position: usize,
    ) -> FullNode {
        FullNode {
            node: tree_context.shared(self),
            metrics: self.metrics(),
            declared: self.declared(),
            referenced: self.referenced(),
            own_reference: self.own_reference(),
            depth,
            position,
        }
    }

    fn kind(&self) -> &Type {
        &self.kind
    }

    fn metrics(&self) -> Metrics {
        todo!()
    }

    fn declared(&self) -> Vec<Declaration> {
        todo!()
    }

    fn referenced(&self) -> Vec<QualifiedName> {
        todo!()
    }

    fn own_reference(&self) -> QualifiedName {
        todo!()
    }

    fn label(&self, label: Label) -> () {
        todo!()
    }
}

#[derive(PartialEq, Eq)]
enum Has {
    Down,
    Up,
    Right,
}

struct Metrics {}

pub struct FullNode {
    node: CompressibleTree,
    metrics: Metrics,
    declared: Vec<Declaration>,
    referenced: Vec<QualifiedName>,
    own_reference: QualifiedName,
    depth: usize,
    position: usize,
}

type TsString<'a, T> = &'a [T];

struct CompressedTreeBuilder<'a, T: Eq> {
    tree_context: TreeContext<'a>,
    acc_stack: Vec<ChildrenAcc<'a>>,
    indentation_stack: Vec<TsString<'a, T>>,
    position: usize,
    depth: usize,
    sum_byte_length: usize,
    s: TsString<'a, T>,
}

impl<'a, T: Eq> CompressedTreeBuilder<'a, T> {
    fn new(s: &'a [T], init_indent: &'a [T], init_acc: ChildrenAcc<'a>, tc: TreeContext<'a>) -> Self {
        Self {
            acc_stack: vec![init_acc],
            indentation_stack: vec![init_indent],
            tree_context: tc,
            position: 0,
            depth: 0,
            sum_byte_length: 0,
            s,
        }
    }

    fn inc(&'a mut self, kind: &'a Type) {
        self.position += 1;
        let a = self.acc_stack.last().unwrap();
        self.inc_set(a.to(kind));
    }

    fn kind(&mut self, kind: &str) -> &'a Type {
        self.tree_context.kind(kind)
    }

    fn inc_set(&mut self, acc: ChildrenAcc<'a>) {
        self.depth += 1;
        self.acc_stack.push(acc);
    }

    pub fn pop(&mut self) -> ChildrenAcc {
        self.depth -= 1;
        let tmp = self.acc_stack.pop().unwrap();
        return tmp;
    }

    fn acc(&mut self, full_node: FullNode) {
        self.acc_stack.last_mut().unwrap().add(full_node);
    }

    fn getSpaces(&self, cursor: &TreeCursor) -> &[T] {
        let node = cursor.node();
        let pos = node.start_byte() / 2;
        let padding = node.end_byte() / 2;

        &self.s[pos..padding] //substring(pos - padding, pos);
    }

    fn get_current_acc(&self) -> &'a ChildrenAcc {
        self.acc_stack.last().unwrap()
    }

    fn build_from<'b>(& mut self, root: &mut Tree) -> FullNode {
        let cursor = &mut root.walk();
        assert_eq!(cursor.node().kind(), "program"); // for now
        let mut has = Has::Down;
        // self.acc_stack
        //     .push(self.get_current_acc().to(cursor.node().kind().to_string()));
        loop {
            let parent_indentation: &[T] = self.indentation_stack.last().unwrap();
            let a = cursor.goto_first_child();
            if has != Has::Up && a {
                let k:&'a Type = self.kind(cursor.node().kind());
                self.inc(k);

                let spaces = {
                    let node = cursor.node();
                    let pos = node.start_byte() / 2;
                    let padding = node.end_byte() / 2;
                    &self.s[pos..padding]
                };
                // let spaces = self.getSpaces(&cursor);

                self.fun_name(spaces, parent_indentation);
                has = Has::Down;
            } else {
                self.sum_byte_length = cursor.node().end_byte() / 2;
                let fullNode = self.create(cursor);
                if cursor.goto_next_sibling() {
                    self.acc(fullNode);
                    self.inc(self.kind(cursor.node().kind()));

                    let node = cursor.node();
                    let pos = node.start_byte() / 2;
                    let padding = node.end_byte() / 2;
                    let spaces = &self.s[pos..padding];

                    self.fun_name(spaces, parent_indentation);
                    has = Has::Right;
                } else {
                    // postOrder
                    if cursor.goto_parent() {
                        return fullNode;
                    } else {
                        self.acc(fullNode);
                    }
                    has = Has::Up;
                }
            }
        }
    }

    fn fun_name(&mut self, spaces: &'a [T], parentIndentation: &'a [T]) {
        let needle = self.indentation_stack[0];
        let lastNL = spaces
            .windows(needle.len())
            .rev()
            .position(|window| window == needle);
        self.indentation_stack.push(match lastNL {
            Some(i) => &spaces[i..],
            None => parentIndentation,
        });
    }

    fn create(&mut self, cursor: &TreeCursor) -> FullNode {
        let node = cursor.node();
        //         if (depth == 0) {
        //             if (sumByteLength < s.length()) {
        //                 String spaces = s.substring(sumByteLength);
        //                 ChildrenAcc a = treeContext.parentAcc().to(JSitterJavaTreeGenerator2.GT_S.name()).label(spaces);
        //                 CompressingTreeContext.FullNode spacesLeaf = a.createFullNode(treeContext).locate(depth, position);
        // //                    CompressingTreeContext.FullNode spacesLeaf = treeContext.createCompressedTree(
        // //                            spaces, treeContext.parentAcc().to(JSitterJavaTreeGenerator2.GT_S),//new AccC(ct, JSitterJavaTreeGenerator2.GT_S),
        // //                            Collections.emptyList(), spaces.length()
        // //                    ).locate(depth, position);
        //                 acc(spacesLeaf);
        //             }
        //         }
        let pos = node.start_byte() / 2;
        let end = node.end_byte() / 2;
        let length = end - pos;
        let padding = self.sum_byte_length - pos; // TODO not sure
        let label: TsString<'a,T> = {
            if node.is_named() {
                &[]
            } else {
                &self.s[pos..end]
                // if TypeId::of::<T>() == TypeId::of::<u8>() {
                //     // node.utf8_text(self.s).unwrap()
                //     &self.s[pos..end]
                // }
                // else if TypeId::of::<T>() == TypeId::of::<u8>() {
                //     // &self.s[pos..end]
                //     todo!()
                // } else { panic!() }
            }
            // if (self.get_current_acc().children.size() == 0) {
            //     self.extractLabel(node, pos, length);
            // } else {
            //     ""
            // };
        };
        let popedAcc = self.pop();
        let typ = popedAcc.kind();
        //         if (currentAcc.parent != null && padding > 0) {
        //             String spaces = s.substring(pos - padding, pos);
        //             int lastNL = spaces.lastIndexOf("\n");
        //             String r = null;
        //             if (lastNL != -1) {
        //                 String newIndentation = spaces.substring(lastNL);
        //                 String parentIndentation = currentAcc.indentation;
        //                 if (parentIndentation != null
        //                         && newIndentation.length() > parentIndentation.length()
        //                         && newIndentation.startsWith(parentIndentation)
        //                 ) {
        //                     String prefix = spaces.substring(0, lastNL);
        //                     String suffix = newIndentation.substring(parentIndentation.length());
        //                     r = prefix + "0" + suffix;
        //                 } else if (parentIndentation != null
        //                         && newIndentation.length() >= parentIndentation.length()
        //                         && newIndentation.startsWith(parentIndentation)
        //                         && currentAcc.children.size() > 0
        //                 ) {
        //                     String prefix = spaces.substring(0, lastNL);
        //                     r = prefix + "0";
        //                 } else if (parentIndentation != null && !newIndentation.startsWith(parentIndentation)) {
        //                     r = spaces;
        // //                    } else if (currentAcc.children.size() > 0) {
        // //                        r = spaces;
        // //                    } else if (depth == 0 && currentAcc.children.size()==0) {
        // //                        r = spaces;
        //                 } else if (parentIndentation == null) {
        //                     r = spaces;
        //                 }
        //             } else if (currentAcc.children.size() > 0) {
        //                 r = spaces;
        //             }
        //             if (r != null) {
        //                 ChildrenAcc a = currentAcc.to(JSitterJavaTreeGenerator2.GT_S.name()).label(r);
        //                 CompressingTreeContext.FullNode spacesLeaf = a.createFullNode(treeContext).locate(depth, position);
        //                 acc(spacesLeaf);
        //                 inc(currentAcc.to(JSitterJavaTreeGenerator2.GT_S.name()));
        //                 pop();
        //             }
        //             if (SPACES_DEBUG) {
        //                 String parentIndentation = currentAcc.indentation;
        //                 if (parentIndentation != null) {
        //                     String s = Utils.debugSpaces(parentIndentation);
        //                     label = label + " (" + s + ")";
        //                 } else {
        //                     label = label + " (null)";
        //                 }
        //             }
        //         } else if (currentAcc.parent != null) {
        //             if (SPACES_DEBUG) {
        //                 String parentIndentation = currentAcc.indentation;
        //                 if (parentIndentation != null) {
        //                     String s = Utils.debugSpaces(parentIndentation);
        //                     label = label + " (" + s + ")";
        //                 } else {
        //                     label = label + " (null)";
        //                 }
        //             }
        //         }
        // //            if (!aStack.isEmpty() && padding > 0 && currentAcc.children.size() > 0) {
        // //                String spaces = s.substring(pos - padding, pos);
        // //                TreeContextCompressing.FullNode spacesLeaf = treeContext.createCompressedTree(JSitterJavaTreeGenerator2.GT_S, spaces, popedAcc, Collections.emptyList(), spaces.length()).locate(depth, position);
        // //                acc(spacesLeaf);
        // //                inc();
        // //                pop();
        // //            }
        todo!();// popedAcc.label(label); // TODO try to put it sooner
        popedAcc.create_full_node(&mut self.tree_context, self.depth, self.position)
    }
}

trait PrimStr {}

impl PrimStr for u8 {}
impl PrimStr for u16 {}

impl JavaTreeGen {
    fn build_compressed<'a>(&self, text: &'a [u8], root: &mut Tree, tc: TreeContext<'a>, init_acc:ChildrenAcc<'a>) -> FullNode {
        // atomic_counter::RelaxedCounter::default();
        // let a = &root.root_node().kind().to_string();
        // CompressedTreeBuilder::new(
        //     text,
        //     &"\n".as_bytes(),
        //     tc,
        //     a,
        // )
        // .build_from(&mut r)
        CompressedTreeBuilder::new(text, &"\n".as_bytes(), init_acc,tc).build_from(root)
    }

    // fn build_compressed_utf16<'a>(
    //     &self,
    //     text: &'a [u16],
    //     root: &mut Tree,
    //     tc: TreeContext,
    // ) -> FullNode {
    //     // atomic_counter::RelaxedCounter::default();
    //     // CompressedTreeBuilder::new(
    //     //     text,
    //     //     "\n".encode_utf16().collect::<Vec<u16>>().as_ref(),
    //     //     tc,
    //     //     &root.root_node().kind().to_string(),
    //     // )
    //     // .build_from(&mut root)
    //     todo!()
    // }
}

impl TreeGenerator for JavaTreeGen {
    fn generate<'a>(&mut self, text: &'a [u8], tc: TreeContext, init_acc:ChildrenAcc<'a>) -> FullNode {
        let mut tree = self.parser.parse(text, self.old_tree.as_ref()).unwrap();
        println!("{}", tree.root_node().to_sexp());
        let full_node = self.build_compressed(text, &mut tree, tc, init_acc);
        self.old_tree = Option::Some(tree);
        full_node
    }

    // fn generate_utf16<'a>(&mut self, text: &'a [u16], tc: TreeContext) -> FullNode {
    //     todo!();
    //     let mut tree = self
    //         .parser
    //         .parse_utf16(text, self.old_tree.as_ref())
    //         .unwrap();
    //     println!("{}", tree.root_node().to_sexp());
    //     let full_node = self.build_compressed_utf16(text, &mut tree, tc);
    //     self.old_tree = Option::Some(tree);
    //     full_node
    // }
}

impl<'a> TreeContext<'a> {
    fn get_acc(&self, kind: &'a Type) -> ChildrenAcc {
        ChildrenAcc::new(kind)
    }

    pub fn new( storage:&'a mut Storage) -> Self {
        Self {
            storage,
            // acc: ChildrenAcc::new(&TreeContext::internal_kind("program")),
        }
    }

    fn shared(&mut self, acc: &ChildrenAcc) -> CompressibleTree {
        todo!()
    }

    fn internal_kind(kind: &str) -> Type {
        Type::new(kind)
    }

    pub(crate) fn kind(&'a mut self, kind: &str) -> &'a Type {
        &TreeContext::internal_kind(kind)
    }
}
