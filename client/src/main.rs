use rusted_gumtree_core::{
    matchers::{
        decompressed_tree_store::{BreathFirst, CompletePostOrder},
        heuristic::gt::simple_bottom_up_matcher::SimpleBottomUpMatcher,
        mapping_store::DefaultMappingStore,
        matcher::Matcher,
        optimal::zs::ZsMatcher,
    },
    tree::tree::{DeCompressedTree, Type},
};
use rusted_gumtree_gen_ts_java::{
    java_tree_gen::{
        HashedCompressedNode, JavaTreeGen, LabelStore, NodeStore, SyntaxNodeHashs,
        SyntaxNodeHashsKinds,
    },
    vec_map_store::VecMapStore,
};
use tree_sitter::{Language, Parser};

extern "C" {
    fn tree_sitter_java() -> Language;
}

fn main() {
    println!("Hello, world!");

    let mut parser = Parser::new();

    {
        let language = unsafe { tree_sitter_java() };
        parser.set_language(language).unwrap();
    }

    let mut java_tree_gen = JavaTreeGen::new();

    // src
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

    let full_node_src = java_tree_gen.generate_default(text, tree.walk());

    println!("debug full node 1: {:?}", &full_node_src);

    // dst
    let text = {
        let source_code1 = "class A {
    class C {
        int a = 0xffff;
    }
}";
        source_code1.as_bytes()
    };
    let tree = parser.parse(text, None).unwrap();
    println!("{}", tree.root_node().to_sexp());

    let full_node_dst = java_tree_gen.generate_default(text, tree.walk());

    println!("debug full node 2: {:?}", &full_node_dst);

    let JavaTreeGen {
        line_break,
        label_store,
        type_store,
        mut node_store,
    } = java_tree_gen;

    let mapping_store = DefaultMappingStore::new();
    // let a = SimpleBottomUpMatcher::<
    let a = ZsMatcher::<
        CompletePostOrder<u32, u16>,
        HashedCompressedNode<u32, SyntaxNodeHashs<u32>>,
        u16,
        NodeStore,
        LabelStore,
    >::matchh(
        &node_store,
        &label_store,
        full_node_src.id(),
        full_node_dst.id(),
        mapping_store,
    );
    a.mappings
        .src_to_dst
        .iter()
        .map(|x| if *x == 0 { None } else { Some(*x - 1) })
        .zip(
            a.mappings
                .dst_to_src
                .iter()
                .map(|x| if *x == 0 { None } else { Some(*x - 1) }),
        )
        .enumerate()
        .for_each(|x| println!("{:?}", x));
    // a.src_to_dst.iter().enumerate().for_each(|(i,m)| {
    //     println!("{:?}", (i,m,&a.dst_to_src[*m as usize]));
    // });
    // println!("-----------");
    // a.dst_to_src.iter().enumerate().for_each(|(i,m)| {
    //     println!("{:?}", (i,m,&a.src_to_dst[*m as usize]));
    // });

    // // let mut out = String::new();
    // let mut out = IoOut {
    //     out: stdout()
    // };
    // serialize(
    //     &java_tree_gen.node_store,
    //     &java_tree_gen.label_store,
    //     &full_node.id(),
    //     &mut out,
    //     &std::str::from_utf8(&java_tree_gen.line_break).unwrap(),
    // );
    // println!();
    // print_tree_syntax(
    //     &java_tree_gen.node_store,
    //     &java_tree_gen.label_store,
    //     &full_node.id(),
    // );
    // println!();
    // stdout().flush().unwrap();
}
