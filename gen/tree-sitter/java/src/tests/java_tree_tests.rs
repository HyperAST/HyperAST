use core::fmt;
use std::io::{stdout, Write};

use tree_sitter::{Language, Parser};

use crate::{
    hashed::{HashedCompressedNode, SyntaxNodeHashs},
    java_tree_gen::{
        print_tree_labels, print_tree_syntax, serialize, spaces_after_lb, Acc, JavaTreeGen,
        LabelStore, NodeStore,
    },
    nodes::CompressedNode,
    store::TypeStore,
};

// use crate::java_tree_gen::{JavaTreeGen, TreeContext, TreeGenerator};

extern "C" {
    fn tree_sitter_java() -> Language;
}

#[test]
fn test_equals() {
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
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(HashedCompressedNode::new(
            SyntaxNodeHashs::default(),
            CompressedNode::Spaces(vec![].into_boxed_slice()),
        )),
    };
    let tree = parser.parse(text, None).unwrap();
    let mut acc_stack = vec![Acc::new(java_tree_gen.type_store.get("file"))];
    let full_node = java_tree_gen.generate(text, tree.walk(), &mut acc_stack);
    println!("{}", tree.root_node().to_sexp());
    // print_tree_structure(&java_tree_gen.node_store, &full_node.id());
    print_tree_labels(
        &java_tree_gen.node_store,
        &java_tree_gen.label_store,
        &full_node.id(),
    );
    println!();
    println!();
    println!();

    let text = {
        let source_code1 = "
        class A {
            
        }";
        source_code1.as_bytes()
    };
    let tree = parser.parse(text, None).unwrap();
    let mut acc_stack = vec![Acc::new(java_tree_gen.type_store.get("file"))];
    let _full_node = java_tree_gen.generate(text, tree.walk(), &mut acc_stack);

    let text = {
        let source_code1 = "
        class A {
            int a = 0xffff;
        }";
        source_code1.as_bytes()
    };
    let tree = parser.parse(text, None).unwrap();
    let mut acc_stack = vec![Acc::new(java_tree_gen.type_store.get("file"))];
    let _full_node = java_tree_gen.generate(text, tree.walk(), &mut acc_stack);

    // let text = {
    //     let source_code1 = "
    //     class A {
    //         int a = 0;
    //         void test() {
    //             a;
    //         }
    //     }";
    //     source_code1.as_bytes()
    // };
    // let tree = parser.parse(text, Some(&tree)).unwrap();
    // let (full_node) = java_tree_gen.generate(text, tree.walk());
}
#[test]
fn test_special() {
    let mut parser = Parser::new();

    {
        let language = unsafe { tree_sitter_java() };
        parser.set_language(language).unwrap();
    }

    // let mut parser: Parser, old_tree: Option<&Tree>
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(HashedCompressedNode::new(
            SyntaxNodeHashs::default(),
            CompressedNode::Spaces(vec![].into_boxed_slice()),
        )),
    };

    let text = {
        let source_code1 = "class A {
    class B {
        int a = 0xffff;
    }
}";
        source_code1.as_bytes()
    };
    let tree = parser.parse(text, None).unwrap();
    println!("{}", tree.root_node().to_sexp());

    let mut acc_stack = vec![Acc::new(java_tree_gen.type_store.get("file"))];
    let full_node = java_tree_gen.generate(text, tree.walk(), &mut acc_stack);

    println!("debug full node: {:?}", &full_node);
    // let mut out = String::new();
    let mut out = IoOut { out: stdout() };
    serialize(
        &java_tree_gen.node_store,
        &java_tree_gen.label_store,
        &full_node.id(),
        &mut out,
        &std::str::from_utf8(&java_tree_gen.line_break).unwrap(),
    );
    println!();
    print_tree_syntax(
        &java_tree_gen.node_store,
        &java_tree_gen.label_store,
        &full_node.id(),
    );
    println!();
    stdout().flush().unwrap();
}

struct IoOut<W: std::io::Write> {
    out: W,
}

impl<W: std::io::Write> std::fmt::Write for IoOut<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.out
            .write_all(s.as_bytes())
            .map_err(|_| std::fmt::Error)
    }
}

#[test]
fn test_2_spaces_after_lb() {
    let r = spaces_after_lb("\n".as_bytes(), "\n  ".as_bytes());
    assert_eq!(
        r.and_then(|x| Some(std::str::from_utf8(x).unwrap())),
        Some("  ")
    )
}

#[test]
fn test_1_space_after_lb() {
    let r = spaces_after_lb("\n".as_bytes(), "\n ".as_bytes());
    assert_eq!(
        r.and_then(|x| Some(std::str::from_utf8(x).unwrap())),
        Some(" ")
    )
}

#[test]
fn test_no_spaces_after_lb() {
    let r = spaces_after_lb("\n".as_bytes(), "\n".as_bytes());
    assert_eq!(
        r.and_then(|x| Some(std::str::from_utf8(x).unwrap())),
        Some("")
    )
}

#[test]
fn test_spaces_after_lb_special() {
    let r = spaces_after_lb("\n\r".as_bytes(), "\n\r\t ".as_bytes());
    assert_eq!(
        r.and_then(|x| Some(std::str::from_utf8(x).unwrap())),
        Some("\t ")
    )
}
