use core::fmt;
use std::io::{stdout, Write};

use pretty_assertions::assert_eq;

use tree_sitter::{Language, Parser};

use crate::{
    java_tree_gen::spaces_after_lb,
    java_tree_gen_full_compress_ecs::{
        print_tree_labels, print_tree_syntax, serialize, JavaTreeGen, LabelStore, NodeStore,
        SimpleStores,
    },
    store::TypeStore,
    tree_gen::TreeGen,
    utils::memusage_linux,
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
        let source_code1 = "
        class A {void test() {}}
        ";
        source_code1.as_bytes()
    };
    // let mut parser: Parser, old_tree: Option<&Tree>
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: SimpleStores {
            label_store: LabelStore::new(),
            type_store: TypeStore {},
            node_store: NodeStore::new(),
        },
    };
    let tree = parser.parse(text, None).unwrap();
    // let mut acc_stack = vec![Accumulator::new(java_tree_gen.stores.type_store.get("file"))];

    let full_node = java_tree_gen.generate_default(text, tree.walk());
    println!("{}", tree.root_node().to_sexp());
    // print_tree_structure(&java_tree_gen.node_store, &full_node.compressed_node);
    print_tree_labels(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &full_node.local.compressed_node,
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
    let _full_node = java_tree_gen.generate_default(text, tree.walk());

    let text = {
        let source_code1 = "
        class A {
            int a = 0xffff;
        }";
        source_code1.as_bytes()
    };
    let tree = parser.parse(text, None).unwrap();
    let _full_node = java_tree_gen.generate_default(text, tree.walk());

    let text = {
        let source_code1 = "class A {
    int a = 0;
    void test() {
 a;
        a;
    }
}";
        source_code1.as_bytes()
    };
    let tree = parser.parse(text, None).unwrap();
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_default(text, tree.walk());

    println!();
    print_tree_syntax(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &full_node.local.compressed_node,
    );
    println!();
    stdout().flush().unwrap();

    let mut out = IoOut { stream: stdout() };
    serialize(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &full_node.local.compressed_node,
        &mut out,
        &std::str::from_utf8(&java_tree_gen.line_break).unwrap(),
    );
    println!();
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
        stores: SimpleStores {
            label_store: LabelStore::new(),
            type_store: TypeStore {},
            node_store: NodeStore::new(),
        },
    };

    let text = {
        let source_code1 = "class A {
    class A0 {
        int a = 0xffff;
    }
    class B { int a = 0xfffa;
              int b = 0xffff;

              void test() {
                a;
              }
    } class C { int a = 0xffff;
           int b = 0xfffa;

        void test() {
            a;
        } void test2() {
            b;
        }
    }
    class D { 
        int a = 0xffff;
        int b = 0xffff;

     void test() {
         a;
     } void test2() {
         b;
     }
 }
    }";
        // let source_code1 = "class A {
        //     class A0 {
        //         int a = 0xffff;
        // }
        //     }
        // ";
        source_code1.as_bytes()
    };
    let tree = parser.parse(text, None).unwrap();
    println!("{}", tree.root_node().to_sexp());

    let _full_node = java_tree_gen.generate_default(text, tree.walk());

    let text = {
        let source_code1 = "class A {
    class A0 {
        int a = 0xffff;
    }
    class B { int a = 0xfffa;
              int b = 0xfffa;
              
              Object test0() {
                    return factory.getModel().getAllModules().stream()
                    .map(module -> getPackageFromModule(qualifiedName, module))
                    .filter(Objects::nonNull)
                    .findFirst()
                    .orElse(null);
              }
              void test() {
                while (Arrays.asList(SmPLLexer.TokenType.MetavarIdentifier, SmPLLexer.TokenType.WhenMatches).contains(tokens.get(pos).getType())) {
                    switch (tokens.get(pos).getType()) {
                        case MetavarIdentifier:
                            if (genericMetavarTypes.contains(metavarType)) {
                                output.append(metavarType).append(\"(\").append(tokens.get(pos).getText().strip()).append(\");\\n\");
                            } else {
                                output.append(metavarType).append(\" \").append(tokens.get(pos).getText().strip()).append(\";\\n\");
                            }
                            break;
    
                        case WhenMatches:
                            output.append(\"constraint(\\\"regex-match\\\", \" + tokens.get(pos + 1).getText() + \");\\n\");
                            ++pos;
                            break;
    
                        default:
                            throw new IllegalStateException(\"impossible\");
                    }
    
                    ++pos;
                }
              }
    } class C { int a = 0xffff;
           int b = 0xffff;

        void test() {
            a;
        } void test2() {
            b;
        }
    }
    class E { 
        int a = 0xffff;
        int b = 0xffff;

     void test() {
         a;
     } void test2() {
         a;
     }
 }
    }";
        // let source_code1 = "class A {
        //     class A0 {
        //         int a = 0xffff;
        // }
        //     }
        // ";
        source_code1.as_bytes()
    };
    let tree = parser.parse(text, None).unwrap();
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_default(text, tree.walk());

    println!("debug full node: {:?}", &full_node);
    // let mut out = String::new();
    let mut out = IoOut { stream: stdout() };
    serialize(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &full_node.local.compressed_node,
        &mut out,
        &std::str::from_utf8(&java_tree_gen.line_break).unwrap(),
    );
    println!();
    print_tree_syntax(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &full_node.local.compressed_node,
    );
    println!();
    stdout().flush().unwrap();

    let mut out = BuffOut {
        buff: "".to_owned(),
    };
    serialize(
        &java_tree_gen.stores.node_store,
        &java_tree_gen.stores.label_store,
        &full_node.local.compressed_node,
        &mut out,
        &std::str::from_utf8(&java_tree_gen.line_break).unwrap(),
    );
    assert_eq!(std::str::from_utf8(text).unwrap(), out.buff);

    println!("{:?}", java_tree_gen.stores().label_store);
    println!("{:?}", java_tree_gen.stores().node_store);

    let mu = memusage_linux();
    drop(java_tree_gen);
    let mu = mu - memusage_linux();
    println!("mu {}", mu);
}

struct IoOut<W: std::io::Write> {
    stream: W,
}

impl<W: std::io::Write> std::fmt::Write for IoOut<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.stream
            .write_all(s.as_bytes())
            .map_err(|_| std::fmt::Error)
    }
}

struct BuffOut {
    buff: String,
}

impl std::fmt::Write for BuffOut {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Ok(self.buff.extend(s.chars()))
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
