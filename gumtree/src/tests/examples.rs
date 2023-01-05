use crate::tree::simple_tree::{tree, SimpleTree};

type ST<K> = SimpleTree<K>;

pub(crate) fn example_single() -> (ST<u8>, ST<u8>) {
    let src = tree!(0, "f");
    let dst = tree!(0, "f");
    (src, dst)
}

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
