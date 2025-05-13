use core::fmt;
use std::io::{Write, stdout};

use hyperast::{
    filter::BloomResult,
    nodes::RefContainer,
    position::{
        PositionConverter, Scout, StructuralPosition, StructuralPositionStore, TypedScout,
        TypedTreePath,
    },
    store::SimpleStores,
    types::{NodeId, Typed, WithChildren},
    utils::memusage,
};
use pretty_assertions::assert_eq;

use crate::impact::element::{IdentifierFormat, LabelPtr, RefsEnum};
use crate::{
    legion_with_refs::{self, JavaTreeGen, NodeIdentifier},
    types::{TIdN, TStore},
};

fn run(text: &[u8]) {
    let mut stores = SimpleStores::<TStore>::default();
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen::new(&mut stores, &mut md_cache);

    let tree = match legion_with_refs::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text, tree.walk());

    println!();
    println!(
        "{}",
        hyperast::nodes::SyntaxSerializer::new(
            &*java_tree_gen.stores,
            full_node.local.compressed_node
        )
    );
    println!(
        "{}",
        hyperast::nodes::SexpSerializer::new(
            &*java_tree_gen.stores,
            full_node.local.compressed_node
        )
    );
    stdout().flush().unwrap();
    println!(
        "{}",
        hyperast::nodes::TextSerializer::new(
            &*java_tree_gen.stores,
            full_node.local.compressed_node
        )
    )
}
#[test]
fn test_cases() {
    let cases = [
        CASE_1,
        CASE_1_1,
        CASE_1_2,
        CASE_1_3,
        CASE_1_4,
        CASE_1_5,
        CASE_1_5,
        CASE_1_6,
        CASE_1_7,
        CASE_1_8,
        CASE_1_9,
        CASE_1_10,
        CASE_2,
        CASE_3,
        CASE_4,
        CASE_5,
        CASE_6,
        CASE_7,
        CASE_8,
        CASE_8_1,
        CASE_9,
        CASE_10,
        CASE_11,
        CASE_11_BIS,
        CASE_12,
        CASE_13,
        CASE_14,
        CASE_15,
        CASE_15_1,
        CASE_15_2,
        CASE_16,
        CASE_17,
        CASE_18,
        CASE_19,
        CASE_20,
        CASE_21,
        CASE_22,
        CASE_23,
        CASE_24,
        CASE_25,
        CASE_26,
        CASE_27,
        CASE_28,
        CASE_29,
        CASE_30,
        CASE_31,
        CASE_32,
        CASE_33,
        A,
    ];
    for case in cases {
        run(case.as_bytes())
    }
}

#[test]
fn test_equals() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    let text = CASE_33.as_bytes();
    let mut stores = SimpleStores::<TStore>::default();
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen::new(&mut stores, &mut md_cache);
    let tree = match legion_with_refs::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text, tree.walk());

    println!();
    println!(
        "{}",
        hyperast::nodes::SyntaxSerializer::new(
            &*java_tree_gen.stores,
            full_node.local.compressed_node
        )
    );
    println!();
    stdout().flush().unwrap();

    println!(
        "{}",
        hyperast::nodes::TextSerializer::new(
            &*java_tree_gen.stores,
            full_node.local.compressed_node
        )
    );

    {
        let stores = java_tree_gen.stores;
        // playing with refs
        let a = &full_node.local.compressed_node;
        let Some(mut ana) = full_node.local.ana else {
            panic!("None");
        };
        println!("refs:",);
        ana.print_refs(&stores.label_store);

        let b = stores.node_store.resolve(*a);
        use hyperast::types::LabelStore;
        macro_rules! scoped_ref {
            ( $o:expr, $i:expr ) => {{
                let o = $o;
                let i = $i;
                let f = IdentifierFormat::from(i);
                let i = stores.label_store.get_or_insert(i);
                let i = LabelPtr::new(i, f);
                ana.solver.intern_ref(RefsEnum::ScopedIdentifier(o, i))
            }};
        }
        let root = ana.solver.intern(RefsEnum::Root);
        let i = scoped_ref!(root, "B");
        let d = ana.solver.nodes.with(i);
        let c = b.check(d);
        match c {
            BloomResult::MaybeContain => println!("Maybe contains B"),
            BloomResult::DoNotContain => println!("Do not contains B"),
        }
    }
    //     use hyperast::position::extract_position;
    //     let mut position = extract_position(&java_tree_gen.stores, d_it.parents(), d_it.offsets());
    //     position.set_len(b.get_bytes_len() as usize);
    //     println!("position: {:?}", position);
}

#[test]
fn test_special() {
    // let mut parser: Parser, old_tree: Option<&Tree>
    let mut stores = SimpleStores::<TStore>::default();

    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen::new(&mut stores, &mut md_cache);

    let text = {
        let source_code1 = "package p.y;
public class A {
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
            a.b;
            a.f();
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
    let tree = match legion_with_refs::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    println!("{}", tree.root_node().to_sexp());

    let _full_node = java_tree_gen.generate_file(b"", text, tree.walk());

    let text = {
        let source_code1 = "class A {
    class A0 {
        int a = 0xffff;
    }
    class B { int a = 0xfffa;
              int b = 0xfffa;

              Object test0() {
                    get();
                    aaa.getM();
                    aaa.getM(AAA::ff);
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
     public void testBasicSingle() {

        // contract: NaiveExceptionControlFlowStrategy should result in every statement parented by a try
        //           block having a path to the corresponding catch block.

        CtMethod<?> method = Launcher.parseClass(\"class A {\\n\" +
                                                 \"  void m() {\\n\" +
                                                 \"    try {\\n\" +
                                                 \"      a();\\n\" +
                                                 \"      b();\\n\" +
                                                 \"      c();\\n\" +
                                                 \"    } catch (Exception e) {\\n\" +
                                                 \"      bang();\\n\" +
                                                 \"    }\\n\" +
                                                 \"    x();\\n\" +
                                                 \"  }\\n\" +
                                                 \"}\\n\").getMethods().iterator().next();

        ControlFlowBuilder builder = new ControlFlowBuilder();
        builder.setExceptionControlFlowStrategy(new NaiveExceptionControlFlowStrategy());
        builder.build(method);
        ControlFlowGraph cfg = builder.getResult();
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
    let tree = match legion_with_refs::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text, tree.walk());

    println!("debug full node: {:?}", &full_node);
    // let mut out = String::new();

    println!(
        "{}",
        hyperast::nodes::SyntaxSerializer::new(
            &*java_tree_gen.stores,
            full_node.local.compressed_node
        )
    );
    stdout().flush().unwrap();

    let mut out = BuffOut {
        buff: "".to_owned(),
    };
    use std::fmt::Write;
    write!(
        out,
        "{}",
        hyperast::nodes::TextSerializer::new(
            &*java_tree_gen.stores,
            full_node.local.compressed_node
        )
    )
    .unwrap();
    assert_eq!(std::str::from_utf8(text).unwrap(), out.buff);

    println!("{:?}", java_tree_gen.stores.node_store);
    println!("{}", java_tree_gen.stores.label_store);

    let mu = memusage();
    drop(java_tree_gen);
    let mu = mu - memusage();
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
fn test_offset_computation() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    let text = CASE_29.as_bytes();
    let mut stores = SimpleStores::<TStore>::default();
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen::new(&mut stores, &mut md_cache);
    let tree = match legion_with_refs::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text, tree.walk());
    let mut s = StructuralPositionStore::new(full_node.local.compressed_node);
    let mut scout = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let mut scout: TypedScout<TIdN<NodeIdentifier>, u16> = s.type_scout(&mut scout, unsafe {
        &TIdN::from_id(full_node.local.compressed_node)
    });
    {
        let mut f = |x, i: u16| {
            let b = stores.node_store.resolve(x);
            let x = b.child(&i).unwrap();
            use hyperast::types::TypedNodeStore;
            let x: crate::types::TIdN<_> = stores.node_store.try_typed(&x).unwrap();
            dbg!(stores.node_store.resolve_typed(&x).get_type());
            scout.goto_typed(x, i);
            // scout.up(&s);
            // scout.goto_typed(x, i);
            x
        };
        let x = full_node.local.compressed_node;
        let x = f(x, 30);
        let x = f(*x.as_id(), 6);
        let x = f(*x.as_id(), 2);
        let x = f(*x.as_id(), 7);
        let x = f(*x.as_id(), 24);
        let _ = x;
    }
    s.check(&stores).unwrap();
    let x = s.push_typed(&mut scout);
    let z = s.get(x);
    hyperast::position::position_accessors::assert_invariants_post_full(&z, &stores);
    let position_converter = &PositionConverter::new(&z).with_stores(&stores);

    let p = if true {
        use hyperast::position;
        // use position::offsets_and_nodes;
        // let src = offsets::OffsetsRef::from(path_to_target.as_slice());
        // let src = src.with_root(src_tr);
        // let src = src.with_store(stores);
        // // let no_spaces_path_to_target: offsets::Offsets<_, position::tags::TopDownNoSpace> =
        // //     src.compute_no_spaces::<_, offsets::Offsets<_, _>>();
        let src = position_converter;
        let pos: position::Position = src.compute_pos_post_order::<_, position::Position, _>();
        println!("|{}|", std::str::from_utf8(&text[pos.range()]).unwrap());
        pos
    } else {
        panic!("removed")
        // position_converter.make_file_and_offset()
    };
    dbg!(&p);
    println!("{:?}", p);
    println!("|{}|", std::str::from_utf8(&text[p.range()]).unwrap());
    assert_eq!(
        std::str::from_utf8(&text[p.range()]).unwrap(),
        r#"ModelUtils.canBeBuilt(new File("./target/spooned/spoon/test/template/ReturnReplaceResult.java"), 8);"#
    );
}

#[test]
fn test_offset_computation2() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    let text = CASE_30.as_bytes();
    let mut stores = SimpleStores::<TStore>::default();
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen::new(&mut stores, &mut md_cache);
    let tree = match legion_with_refs::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text, tree.walk());
    let mut s = StructuralPositionStore::new(full_node.local.compressed_node);
    let mut scout = Scout::from((StructuralPosition::from((vec![], vec![])), 0));
    let mut scout: TypedScout<TIdN<NodeIdentifier>, u16> = s.type_scout(&mut scout, unsafe {
        &TIdN::from_id(full_node.local.compressed_node)
    });

    println!(
        "{}",
        hyperast::nodes::SyntaxSerializer::new(
            &*java_tree_gen.stores,
            full_node.local.compressed_node
        )
    );
    {
        let mut f = |x, i: u16| {
            let b = stores.node_store.resolve(x);
            let x = b.child(&i).unwrap();
            use hyperast::types::TypedNodeStore;
            let x = stores.node_store.try_typed(&x).unwrap();
            scout.goto_typed(x, i);
            scout.up(&s);
            scout.goto_typed(x, i);
            x
        };
        let x = full_node.local.compressed_node;
        let x = f(x, 5);
        let _ = x;
    }
    s.check(&stores).unwrap();
    let x = s.push_typed(&mut scout);
    let z = s.get(x);
    hyperast::position::position_accessors::assert_invariants_post(&z, &stores);
    let position_converter = &PositionConverter::new(&z).with_stores(&stores);
    let p = if true {
        use hyperast::position;
        // use position::offsets_and_nodes;
        // let src = offsets::OffsetsRef::from(path_to_target.as_slice());
        // let src = src.with_root(src_tr);
        // let src = src.with_store(stores);
        // // let no_spaces_path_to_target: offsets::Offsets<_, position::tags::TopDownNoSpace> =
        // //     src.compute_no_spaces::<_, offsets::Offsets<_, _>>();
        let src = position_converter;
        let pos: position::Position = src.compute_pos_post_order::<_, position::Position, _>();
        println!("|{}|", std::str::from_utf8(&text[pos.range()]).unwrap());
        pos
    } else {
        panic!("removed")
        // position_converter.make_file_and_offset()
    };
    println!("{:?}", p);
    println!("|{}|", std::str::from_utf8(&text[p.range()]).unwrap());
    assert_eq!(
        std::str::from_utf8(&text[p.range()]).unwrap(),
        r#"public class InnerTypeOk {
  private void test() {
    Entry<String, String> test;
  }
}"#
    );
}
