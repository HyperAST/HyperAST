use hyperast::test_utils::simple_tree::SimpleTree;

use crate::{matchers::mapping_store::VecStore, tests::tree};

type ST<K> = SimpleTree<K>;

#[allow(unused)]
pub(crate) fn example_stable_test1() -> ((ST<u8>, ST<u8>), VecStore<u16>) {
    let src = tree!(
        0, "t"; [
            tree!(0, "a"; [
                tree!(0, "1"),
                tree!(0, "2"),
            ]),
            tree!(0, "b"; [
                tree!(0, "3"),
                tree!(0, "4"),
            ])
    ]);
    let dst = tree!(
        0, "t"; [
            tree!(0, "c"; [
                tree!(0, "3"),
                tree!(0, "2"),
            ]),
            tree!(0, "d"; [
                tree!(0, "4"),
                tree!(0, "1"),
            ])
    ]);
    //let map_src = vec![vec![0, 0], vec![0, 1], vec![1, 0], vec![1, 1]];
    //let map_dst = vec![vec![1, 1], vec![0, 1], vec![0, 0], vec![1, 0]];
    let mappings = VecStore {
        src_to_dst: vec![5, 2, 0, 1, 4, 0, 0, 0],
        dst_to_src: vec![4, 2, 0, 5, 1, 0, 0, 0],
    };
    ((src, dst), mappings)
}

// This is a variation of the previous test: gumtree stable succeeds, but it doesn't map
// anything because there is a cycle of 'best' mappings (but all have the same weight)
pub(crate) fn example_stable_test2() -> ((ST<u8>, ST<u8>), VecStore<u16>) {
    let src = tree!(
        0, "t"; [
            tree!(0, "a"; [
                tree!(0, "1"),
                tree!(0, "2"),
            ]),
            tree!(0, "b"; [
                tree!(0, "3"),
                tree!(0, "4"),
            ])
    ]);
    let dst = tree!(
        0, "t"; [
            tree!(0, "c"; [
                tree!(0, "2"),
                tree!(0, "3"),
            ]),
            tree!(0, "d"; [
                tree!(0, "4"),
                tree!(0, "1"),
            ])
    ]);
    //let map_src = vec![vec![0, 0], vec![0, 1], vec![1, 0], vec![1, 1]];
    //let map_dst = vec![vec![1, 1], vec![0, 0], vec![0, 1], vec![1, 0]];
    let mappings = VecStore {
        src_to_dst: vec![5, 1, 0, 2, 4, 0, 0, 0],
        dst_to_src: vec![2, 4, 0, 5, 1, 0, 0, 0],
    };
    ((src, dst), mappings)
}

pub(crate) fn example_stable1() -> ((ST<u8>, ST<u8>), VecStore<u16>) {
    let src = tree!(
        0, "t"; [
            tree!(0, "a"; [
                tree!(0, "1"),
                tree!(0, "2"),
            ]),
            tree!(0, "b"; [
                tree!(0, "3"),
                tree!(0, "4"),
            ])
    ]);
    let dst = tree!(
        0, "t"; [
            tree!(0, "c"; [
                tree!(0, "4"),
                tree!(0, "1"),
            ]),
            tree!(0, "d"; [
                tree!(0, "3"),
                tree!(0, "2"),
            ])
    ]);
    //let map_src = vec![vec![0, 0], vec![0, 1], vec![1, 0], vec![1, 1]];
    //let map_dst = vec![vec![0, 1], vec![1, 1], vec![1, 0], vec![0, 0]];
    let mappings = VecStore {
        src_to_dst: vec![2, 5, 0, 4, 1, 0, 0, 0],
        dst_to_src: vec![5, 1, 0, 4, 2, 0, 0, 0],
    };
    ((src, dst), mappings)
}

pub(crate) fn example_stable2() -> ((ST<u8>, ST<u8>), VecStore<u16>) {
    let src = tree!(
        0, "t"; [
            tree!(0, "a"; [
                tree!(0, "1"),
                tree!(0, "2"),
                tree!(0, "3"),
            ]),
            tree!(0, "b"; [
                tree!(0, "4"),
                tree!(0, "5"),
                tree!(0, "6"),
            ]),
            tree!(0, "c"; [
                tree!(0, "7"),
                tree!(0, "8"),
                tree!(0, "9"),
            ])
    ]);
    let dst = tree!(
        0, "t"; [
            tree!(0, "d"; [
                tree!(0, "1"),
                tree!(0, "4"),
                tree!(0, "7"),
            ]),
            tree!(0, "e"; [
                tree!(0, "2"),
                tree!(0, "5"),
                tree!(0, "8"),
            ]),
            tree!(0, "f"; [
                tree!(0, "3"),
                tree!(0, "6"),
                tree!(0, "9"),
            ])
    ]);
    // let map_src = vec![
    // vec![0, 0],
    // vec![0, 1],
    // vec![0, 2],
    // vec![1, 0],
    // vec![1, 1],
    // vec![1, 2],
    // vec![2, 0],
    // vec![2, 1],
    // vec![2, 2],
    // ];
    // let map_dst = vec![
    // vec![0, 0],
    // vec![1, 0],
    // vec![2, 0],
    // vec![0, 1],
    // vec![1, 1],
    // vec![2, 1],
    // vec![0, 2],
    // vec![1, 2],
    // vec![2, 2],
    // ];
    let mappings = VecStore {
        src_to_dst: vec![1, 5, 9, 0, 2, 6, 10, 0, 3, 7, 11, 0, 0, 0],
        dst_to_src: vec![1, 5, 9, 0, 2, 6, 10, 0, 3, 7, 11, 0, 0, 0],
    };
    ((src, dst), mappings)
}

pub(crate) fn example_stable3() -> ((ST<u8>, ST<u8>), VecStore<u16>) {
    let src = tree!(
        0, "t"; [
            tree!(0, "a"; [
                tree!(0, "1"),
                tree!(0, "2"),
                tree!(0, "3"),
                tree!(0, "4"),
            ]),
            tree!(0, "b"; [
                tree!(0, "b2"; [
                    tree!(0, "5"),
                    tree!(0, "6"),
                    tree!(0, "7"),
                    tree!(0, "8"),
                ])
            ]),
            tree!(0, "c"; [
                tree!(0, "9"),
                tree!(0, "10"),
                tree!(0, "11"),
                tree!(0, "12"),
            ])
    ]);
    let dst = tree!(
        0, "t"; [
            tree!(0, "d"; [
                tree!(0, "5"),
                tree!(0, "6"),
                tree!(0, "11"),
                tree!(0, "12"),
                tree!(0, "14")
            ]),
            tree!(0, "e"; [
                tree!(0, "1"),
                tree!(0, "2"),
                tree!(0, "7"),
                tree!(0, "8"),
                tree!(0, "13")
            ]),
            tree!(0, "f"; [
                tree!(0, "f2"; [
                    tree!(0, "3"),
                    tree!(0, "4"),
                    tree!(0, "9"),
                    tree!(0, "10"),
                ])
            ]),
    ]);
    // let map_src = vec![
    // vec![0, 0],
    // vec![0, 1],
    // vec![0, 2],
    // vec![0, 3],
    // vec![1, 0, 0],
    // vec![1, 0, 1],
    // vec![1, 0, 2],
    // vec![1, 0, 3],
    // vec![2, 0],
    // vec![2, 1],
    // vec![2, 2],
    // vec![2, 3],
    // ];
    // let map_dst = vec![
    // vec![1, 0],
    // vec![1, 1],
    // vec![2, 0, 0],
    // vec![2, 0, 1],
    // vec![0, 0],
    // vec![0, 1],
    // vec![1, 2],
    // vec![1, 3],
    // vec![2, 0, 2],
    // vec![2, 0, 3],
    // vec![0, 2],
    // vec![0, 3],
    // ];
    let mappings = VecStore {
        src_to_dst: vec![7, 8, 13, 14, 0, 1, 2, 9, 10, 0, 0, 15, 16, 3, 4, 0, 0, 0],
        dst_to_src: vec![
            6, 7, 14, 15, 0, 0, 1, 2, 8, 9, 0, 0, 3, 4, 12, 13, 0, 0, 0, 0,
        ],
    };
    ((src, dst), mappings)
}

pub(crate) fn example_unstable1() -> ((ST<u8>, ST<u8>), VecStore<u16>) {
    let src = tree!(
        0, "t"; [
            tree!(0, "a"; [
                tree!(0, "1"),
                tree!(0, "3"),
            ]),
            tree!(0, "b"; [
                tree!(0, "2"),
            ])
    ]);
    let dst = tree!(
        0, "t"; [
            tree!(0, "c"; [
                tree!(0, "1"),
                tree!(0, "2"),
            ])
    ]);
    //let map_src = vec![vec![0, 0], vec![1, 0]];
    //let map_dst = vec![vec![0, 0], vec![0, 1]];
    let mappings = VecStore {
        src_to_dst: vec![1, 0, 0, 2, 0, 0, 0],
        dst_to_src: vec![1, 4, 0, 0, 0],
    };
    ((src, dst), mappings)
}

pub(crate) fn example_unstable2() -> ((ST<u8>, ST<u8>), VecStore<u16>) {
    let src = tree!(
        0, "r"; [
            tree!(
                0, "x"; [
                    tree!(0, "a"),
            ]),
            tree!(
                0, "y"; [
                    tree!(0, "b"),
                    tree!(0, "c"),
            ]),
    ]);
    let dst = tree!(
        0, "r"; [
            tree!(0, "x"),
            tree!(
                0, "y"; [
                    tree!(0, "a"),
                    tree!(0, "b"),
                    tree!(0, "c"),
            ]),
    ]);
    //let map_src = vec![vec![0, 0], vec![1, 0], vec![1, 1]];
    //let map_dst = vec![vec![1, 0], vec![1, 1], vec![1, 2]];
    let mappings = VecStore {
        src_to_dst: vec![2, 0, 3, 4, 0, 0, 0],
        dst_to_src: vec![0, 1, 3, 4, 0, 0, 0],
    };
    ((src, dst), mappings)
}

#[allow(unused)] // TODO make a test with this example
pub(crate) fn example_single() -> (ST<u8>, ST<u8>) {
    let src = tree!(0, "f");
    let dst = tree!(0, "f");
    (src, dst)
}

#[allow(unused)] // TODO make a test with this example
pub(crate) fn example_simple() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        0, "f"; [
            tree!(0, "d"),
            tree!(0, "e"),
    ]);
    let dst = tree!(
        0, "f"; [
            tree!(0, "c"),
            tree!(0, "e"),
    ]);
    (src, dst)
}

#[allow(unused)] // TODO make a test with this example
pub(crate) fn example_simple1() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        0, "f"; [
            tree!(0, "g"; [
                tree!(0, "d"),
                tree!(0, "e"),
            ])
    ]);
    let dst = tree!(
        0, "f"; [
            tree!(0, "g"; [
                tree!(0, "c"),
                tree!(0, "e"),
            ])
    ]);
    (src, dst)
}

#[allow(unused)] // TODO make a test with this example
pub(crate) fn example_move() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        0, "f"; [
            tree!(0, "g"; [
                tree!(0, "d"),
                tree!(0, "e"),
            ]),
            tree!(0, "h")
    ]);
    let dst = tree!(
        0, "f"; [
            tree!(0, "g"),
            tree!(0, "h"; [
                tree!(0, "d"),
                tree!(0, "e"),
            ]),
    ]);
    (src, dst)
}

#[allow(unused)] // TODO make a test with this example
pub(crate) fn example_move1() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        0, "f"; [
            tree!(0, "g"; [
                tree!(0, "i"; [
                    tree!(0, "d"),
                    tree!(0, "e"),
                ]),
            ]),
            tree!(0, "h")
    ]);
    let dst = tree!(
        0, "f"; [
            tree!(0, "g"),
            tree!(0, "h"; [
                tree!(0, "i"; [
                    tree!(0, "d"),
                    tree!(0, "e"),
                ]),
            ]),
    ]);
    (src, dst)
}

#[allow(unused)] // TODO make a test with this example
pub(crate) fn example_move2() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        0, "f"; [
            tree!(0, "g"; [
                tree!(0, "c"),
                tree!(0, "d"),
                tree!(0, "e"),
            ]),
            tree!(0, "h")
    ]);
    let dst = tree!(
        0, "f"; [
            tree!(0, "g"),
            tree!(0, "h"; [
                tree!(0, "i"; [
                    tree!(0, "d"),
                    tree!(0, "e"),
                ]),
            ]),
    ]);
    (src, dst)
}

#[allow(unused)] // TODO make a test with this example
pub(crate) fn example_move3() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        0, "f"; [
            tree!(0, "x"),
            tree!(0, "g"; [
                tree!(0, "c"),
                tree!(0, "d"),
                tree!(0, "e"),
            ]),
            tree!(0, "h")
    ]);
    let dst = tree!(
        0, "f"; [
            tree!(0, "x"),
            tree!(0, "g"),
            tree!(0, "h"; [
                tree!(0, "i"; [
                    tree!(0, "d"),
                    tree!(0, "e"),
                ]),
            ]),
    ]);
    (src, dst)
}

pub(crate) fn example_zs_paper() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        0, "f"; [
            tree!(0, "d"; [
                tree!(0, "q"),
                tree!(0, "c"; [
                    tree!(0, "b")
                ]),
            ]),
            tree!(0, "e"),
    ]);
    let dst = tree!(
        0, "f"; [
            tree!(0, "c"; [
                tree!(0, "d"; [
                    tree!(0, "a"),
                    tree!(1 , "b")
                ])
            ]),
            tree!(0, "e"),
    ]);
    (src, dst)
}

pub(crate) fn example_gt_java_code() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        0, "a"; [
            tree!(0, "b"),
            tree!(0, "c"; [
                tree!(0, "d"),
                tree!(0, "e"),
                tree!(0, "f"),
                tree!(0, "r1"),
            ]),
    ]);
    let dst = tree!(
        0,"z"; [
            tree!( 0, "a"; [
                tree!(0, "b"),
                tree!(0, "c"; [
                    tree!(0, "d"),
                    tree!(1, "y"),
                    tree!(0, "f"),
                    tree!(0, "r2"),
                ]),
            ]),
    ]);
    (src, dst)
}

pub(crate) fn example_gt_slides() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        0,"6"; [
            tree!(0, "5"; [
                tree!(0, "2"; [
                    tree!(0, "1"),
                ]),
                tree!(0, "3"),
                tree!(0, "4"),
            ]),
    ]);
    let dst = tree!(
        0,"6"; [
            tree!(0, "2"; [
                tree!(0, "1"),
            ]),
            tree!(0, "4"; [
                tree!(0, "3"),
            ]),
            tree!(0, "5"),
    ]);
    (src, dst)
}

#[allow(unused)]
pub(crate) fn example_gumtree() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        0,"a"; [
            tree!(0, "e"; [
                tree!(0, "f"),
            ]),
            tree!(0, "b"; [
                tree!(0, "c"),
                tree!(0, "d"),
            ]),
            tree!(0, "g"),
    ]);
    let dst = tree!(
        0,"z"; [
            tree!(0, "b"; [
                tree!(0, "c"),
                tree!(0, "d"),
            ]),
            tree!(1, "h"; [
                tree!(0, "e"; [
                    tree!(0, "y"),
                ]),
            ]),
            tree!(0, "g"),
    ]);
    (src, dst)
}

#[allow(unused)]
pub fn example_gumtree_ambiguous() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        0,"a"; [
            tree!(0, "e"; [
                tree!(0, "f")
            ]),
            tree!(0, "b"; [
                tree!(0, "c"),
                tree!(0, "d"),
            ]),
            tree!(0, "g"),
    ]);
    let dst = tree!(
        0,"z"; [
            tree!(0, "b"; [
                tree!(0, "c"),
                tree!(0, "d"),
            ]),
            tree!(1, "h"; [
                tree!(0, "e"; [
                    tree!(0, "y")
                ])
            ]),
            tree!(0, "g"),
            tree!(0, "b"; [
                tree!(0, "c"),
                tree!(0, "d"),
            ]),
    ]);
    (src, dst)
}

#[allow(unused)]
pub(crate) fn example_bottom_up() -> (ST<u8>, ST<u8>) {
    // types : ["td";"md";"vis";"name";"block";"s"]
    let src = tree!(
        0; [
            tree!( 1; [
                tree!(2, "public"),
                tree!(3, "foo"),
                tree!(4; [
                    tree!(5, "s1"),
                    tree!(5, "s2"),
                    tree!(5, "s3"),
                    tree!(5, "s4"),
                ]),
            ])
    ]);
    let dst = tree!(
        0; [tree!(1; [
                tree!(2, "private"),
                tree!(3, "bar"),
                tree!(4; [
                    tree!(5, "s1"),
                    tree!(5, "s2"),
                    tree!(5, "s3"),
                    tree!(5, "s4"),
                    tree!(5, "s5"),
                ]),
            ])
    ]);
    (src, dst)
}

pub(crate) fn example_action() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        0,"a"; [
            tree!(0, "e"; [
                tree!(0, "f")]),
            tree!(0, "b"; [
                tree!(0, "c"),
                tree!(0, "d")]),
            tree!(0, "g"; [
                tree!(0, "h")]),
            tree!(0, "i"),
            tree!(0, "j"; [
                tree!(0, "k")]),
    ]);
    let dst = tree!(
        0,"Z"; [
            tree!(0, "b"; [
                tree!(0, "c"),
                tree!(0, "d")]),
            tree!(0, "h"; [
                tree!(0, "e"; [
                    tree!(0, "y")])]),
            tree!(0, "x"; [
                tree!(0, "w")]),
            tree!(0, "j"; [
                tree!( 0, "u"; [
                    tree!(0, "v"; [
                    tree!(0, "k")])]
            )]),
    ]);
    (src, dst)
}

pub(crate) fn example_action2() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        0,"a"; [
            tree!(0, "e"; [
                tree!(0, "f")]),
            tree!(0, "b"; [
                tree!(0, "c"),
                tree!(0, "d")]),
            tree!(0, "g"; [
                tree!(0, "h")]),
            tree!(0, "i"),
            tree!(0, "ii"),
            tree!(0, "j"; [
                tree!(0, "k")]),
    ]);
    let dst = tree!(
        0,"Z"; [
            tree!(0, "b"; [
                tree!(0, "c"),
                tree!(0, "d")]),
            tree!(0, "h"; [
                tree!(0, "e"; [
                    tree!(0, "y")])]),
            tree!(0, "x"; [
                tree!(0, "w")]),
            tree!(0, "j"; [
                tree!( 0, "u"; [
                    tree!(0, "v"; [
                        tree!(0, "k")])]
            )]),
    ]);
    (src, dst)
}

/// class A {} renamed to B
#[allow(unused)]
pub(crate) fn example_eq_simple_class_rename() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        0, "program"; [
            tree!(1, "class_decl"; [
                tree!(2, "class"),
                tree!(3, "A"),
                tree!(4, " "),
                tree!(5, "class body"; [
                    tree!(6, "{"),
                    tree!(7, "}")
                ]),
            ]),
    ]);
    let dst = tree!(
        0, "program"; [
            tree!(1, "class_decl"; [
                tree!(2, "class"),
                tree!(3, "B"),
                tree!(4, " "),
                tree!(5, "class body"; [
                    tree!(6, "{"),
                    tree!(7, "}")
                ]),
            ]),
    ]);
    (src, dst)
}

#[allow(unused)]
pub(crate) fn example_very_simple_post_order() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        6, "6"; [
            tree!(2, "2"; [
                tree!(0, "0"),
                tree!(1, "1"),
            ]),
            tree!(5, "5"; [
                tree!(3, "3"),
                tree!(4, "4"),
            ]),
    ]);
    let dst = tree!(
        6, "6"; [
            tree!(2, "2"; [
                tree!(0, "0"),
                tree!(1, "1"),
            ]),
            tree!(5, "5"; [
                tree!(3, "3"),
                tree!(4, "4"),
            ]),
    ]);
    (src, dst)
}

pub(crate) fn example_unstable() -> (ST<u8>, ST<u8>) {
    let src = tree!(
        0, "r"; [
            tree!(
                0, "x"; [
                    tree!(0, "a"),
            ]),
            tree!(
                0, "y"; [
                    tree!(0, "b"),
                    tree!(0, "c"),
            ]),
    ]);
    let dst = tree!(
        0, "r"; [
            tree!(0, "x"),
            tree!(
                0, "y"; [
                    tree!(0, "a"),
                    tree!(0, "b"),
                    tree!(0, "c"),
            ]),
    ]);
    (src, dst)
}

pub(crate) fn example_change_distiller() -> (ST<u8>, ST<u8>) {
    let src = tree!(0, "a"; [
        tree!(0, "b"),
        tree!(42; [ // let's say 42 is for a statement
            tree!(0, "c"),
            tree!(0, "d"),
        ]),
        tree!(52; [ // let's say 52 is when it contains a statement and is one too
            tree!(42, "e"),
        ]),
    ]);
    let dst = tree!(0, "a"; [
        tree!(42; [
            tree!(0, "c"),
            tree!(0, "d"),
        ]),
        tree!(52; [
            tree!(42, "e"),
        ]),
        tree!(0, "b"),
    ]);
    (src, dst)
}
