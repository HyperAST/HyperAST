use std::{
    io::{Write, stdout},
    str::from_utf8,
};

use hyperast::tree_gen::utils_ts::tree_sitter_parse;

use super::test_cases::*;
use crate::bevy::JavaTreeGen;
use crate::bevy::NodeIdentifier;
use crate::bevy::SimpleStores;
use crate::types::TStore;

fn run(text: &[u8]) {
    let (stores, id) = prepare(text);

    println!();
    println!("{}", hyperast::nodes::SyntaxSerializer::new(&stores, id));
    println!("{}", hyperast::nodes::SexpSerializer::new(&stores, id));
    stdout().flush().unwrap();
    let res = hyperast::nodes::TextSerializer::new(&stores, id).to_string();
    println!("{}", res);

    assert_eq!(res, from_utf8(text).unwrap());
}

fn prepare(text: &[u8]) -> (SimpleStores<TStore>, NodeIdentifier) {
    let mut stores = SimpleStores::<TStore>::default();
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen::new(&mut stores, &mut md_cache);

    let tree = match tree_sitter_parse(text, &crate::language()) {
        Ok(t) => t,
        Err(t) => t,
    };
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text, tree.walk());
    (stores, full_node.local.compressed_node)
}

macro_rules! parametrized_test {
    ($m:ident, $in:ident => $t:block, $($case:ident),* $(,)?) => {
        #[allow(non_snake_case)]
        mod $m {$(
            mod $case {
                #[test]
                fn $m() {
                    use super::super::*;
                    match $case {
                        $in => { $t },
                    }
                }
            }
        )*}
    };
}

#[test]
fn aaa() {}

parametrized_test! { test_bij, text => {
        let (stores, id) = prepare(text.as_bytes());
        let res = hyperast::nodes::TextSerializer::new(&stores, id).to_string();
        // println!("{}", res);
        assert_eq!(res, text);
    },
    CASE_1,
    CASE_1_1,
    CASE_1_2,
    CASE_1_3,
    CASE_1_4,
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
}
