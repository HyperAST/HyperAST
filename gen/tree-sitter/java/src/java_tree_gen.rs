use std::{io::BufReader, rc::Rc};

use tree_sitter::{Parser, Language};
pub struct Type {}
pub trait BasicNode {
    fn typ(&self) -> Type;
}
pub trait LabeledNode : BasicNode {
    fn label(&self) -> String;
}
pub trait BasicTree : BasicNode {
    fn getChildren(&self) -> Vec<Rc<&dyn BasicNode>>;
}
pub trait GtTree: LabeledNode {
}

#[derive(PartialEq, Eq, Debug)]
pub enum CompressedTree {

}

#[derive(PartialEq, Eq, Debug)]
pub struct DecompressedTree {
    parent: Box<DecompressedTree>,
    compressed: CompressedTree
}

impl BasicNode for DecompressedTree {
    fn typ(&self) -> Type {
        Type {}
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct TreeContext {
    root: Option<DecompressedTree>
}

pub struct JavaTreeGen<'a> {
    parser: Parser,
    s: &'a [u8],
    pub treeContext:TreeContext // TODO
}

pub trait TreeGenerator<'a> {
    fn generate(&mut self, text: &'a [u8]);
}

extern "C" { fn tree_sitter_java() -> Language; }

impl JavaTreeGen<'_> { 
    pub fn new() -> JavaTreeGen<'static> {
        let mut parser = Parser::new();

        let language = unsafe { tree_sitter_java() };
        parser.set_language(language).unwrap();

        JavaTreeGen{parser, s: &mut [], treeContext:TreeContext{ root: Option::None }}
    }
}

impl<'a> TreeGenerator<'a> for JavaTreeGen<'a> {
    fn generate(&mut self, text: &'a [u8]) {
        
        let tree = self.parser.parse(text, None).unwrap();
        let root_node = tree.root_node();
        self.s = text.clone();

        assert_eq!(root_node.kind(), "program");

    }
}