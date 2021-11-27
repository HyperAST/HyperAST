use super::simple_tree::ST;

pub(crate) fn example_zs_paper() -> (ST<u8>, ST<u8>) {
    let src = ST::new_l_c(
        0,
        "f",
        vec![
            ST::new_l_c(
                0,
                "d",
                vec![
                    ST::new_l(0, "q"),
                    ST::new_l_c(0, "c", vec![ST::new_l(0, "b")]),
                ],
            ),
            ST::new_l(0, "e"),
        ],
    );
    let dst = ST::new_l_c(
        0,
        "f",
        vec![
            ST::new_l_c(
                0,
                "c",
                vec![ST::new_l_c(
                    0,
                    "d",
                    vec![ST::new_l(0, "a"), ST::new_l(1, "b")],
                )],
            ),
            ST::new_l(0, "e"),
        ],
    );
    (src, dst)
}

pub(crate) fn example_gt_java_code() -> (ST<u8>, ST<u8>) {
    let src = ST::new_l_c(
        0,
        "a",
        vec![
            ST::new_l(0, "b"),
            ST::new_l_c(
                0,
                "c",
                vec![
                    ST::new_l(0, "d"),
                    ST::new_l(0, "e"),
                    ST::new_l(0, "f"),
                    ST::new_l(0, "r1"),
                ],
            ),
        ],
    );
    let dst = ST::new_l_c(
        0,
        "z",
        vec![ST::new_l_c(
            0,
            "a",
            vec![
                ST::new_l(0, "b"),
                ST::new_l_c(
                    0,
                    "c",
                    vec![
                        ST::new_l(0, "d"),
                        ST::new_l(1, "y"),
                        ST::new_l(0, "f"),
                        ST::new_l(0, "r2"),
                    ],
                ),
            ],
        )],
    );
    (src, dst)
}

pub(crate) fn example_gt_slides() -> (ST<u8>, ST<u8>) {
    let src = ST::new_l_c(
        0,
        "6",
        vec![ST::new_l_c(
            0,
            "5",
            vec![
                ST::new_l_c(0, "2", vec![ST::new_l(0, "1")]),
                ST::new_l(0, "3"),
                ST::new_l(0, "4"),
            ],
        )],
    );
    let dst = ST::new_l_c(
        0,
        "6",
        vec![
            ST::new_l_c(0, "2", vec![ST::new_l(0, "1")]),
            ST::new_l_c(0, "4", vec![ST::new_l(0, "3")]),
            ST::new_l(0, "5"),
        ],
    );

    (src, dst)
}

pub(crate) fn example_gumtree() -> (ST<u8>, ST<u8>) {
    let src = ST::new_l_c(
        0,
        "a",
        vec![
            ST::new_l_c(0, "e", vec![ST::new_l(0, "f")]),
            ST::new_l_c(0, "b", vec![ST::new_l(0, "c"), ST::new_l(0, "d")]),
            ST::new_l(0, "g"),
        ],
    );
    let dst = ST::new_l_c(
        0,
        "z",
        vec![
            ST::new_l_c(0, "b", vec![ST::new_l(0, "c"), ST::new_l(0, "d")]),
            ST::new_l_c(1, "h", vec![ST::new_l_c(0, "e", vec![ST::new_l(0, "y")])]),
            ST::new_l(0, "g"),
        ],
    );
    (src, dst)
}
pub(crate) fn example_gumtree_ambiguous() -> (ST<u8>, ST<u8>) {
    let src = ST::new_l_c(
        0,
        "a",
        vec![
            ST::new_l_c(0, "e", vec![ST::new_l(0, "f")]),
            ST::new_l_c(0, "b", vec![ST::new_l(0, "c"), ST::new_l(0, "d")]),
            ST::new_l(0, "g"),
        ],
    );
    let dst = ST::new_l_c(
        0,
        "z",
        vec![
            ST::new_l_c(0, "b", vec![ST::new_l(0, "c"), ST::new_l(0, "d")]),
            ST::new_l_c(1, "h", vec![ST::new_l_c(0, "e", vec![ST::new_l(0, "y")])]),
            ST::new_l(0, "g"),
            ST::new_l_c(0, "b", vec![ST::new_l(0, "c"), ST::new_l(0, "d")]),
        ],
    );
    (src, dst)
}

pub(crate) fn example_bottom_up() -> (ST<u8>, ST<u8>) {
    // types : ["td","md","vis","name","block","s"]
    let src = ST::new_c(
        0,
        vec![ST::new_c(
            1,
            vec![
                ST::new_l(2, "public"),
                ST::new_l(3, "foo"),
                ST::new_c(
                    4,
                    vec![
                        ST::new_l(5, "s1"),
                        ST::new_l(5, "s2"),
                        ST::new_l(5, "s3"),
                        ST::new_l(5, "s4"),
                    ],
                ),
            ],
        )],
    );
    let dst = ST::new_c(
        0,
        vec![ST::new_c(
            1,
            vec![
                ST::new_l(2, "private"),
                ST::new_l(3, "bar"),
                ST::new_c(
                    4,
                    vec![
                        ST::new_l(5, "s1"),
                        ST::new_l(5, "s2"),
                        ST::new_l(5, "s3"),
                        ST::new_l(5, "s4"),
                        ST::new_l(5, "s5"),
                    ],
                ),
            ],
        )],
    );

    (src, dst)
}

pub(crate) fn example_action() -> (ST<u8>, ST<u8>) {
    let src = ST::new_l_c(
        0,
        "a",
        vec![
            ST::new_l_c(0, "e", vec![ST::new_l(0, "f")]),
            ST::new_l_c(0, "b", vec![ST::new_l(0, "c"), ST::new_l(0, "d")]),
            ST::new_l_c(0, "g", vec![ST::new_l(0, "h")]),
            ST::new_l(0, "i"),
            ST::new_l_c(0, "j", vec![ST::new_l(0, "k")]),
        ],
    );
    let dst = ST::new_l_c(
        0,
        "Z",
        vec![
            ST::new_l_c(0, "b", vec![ST::new_l(0, "c"), ST::new_l(0, "d")]),
            ST::new_l_c(0, "h", vec![ST::new_l_c(0, "e", vec![ST::new_l(0, "y")])]),
            ST::new_l_c(0, "x", vec![ST::new_l(0, "w")]),
            ST::new_l_c(
                0,
                "j",
                vec![ST::new_l_c(
                    0,
                    "u",
                    vec![ST::new_l_c(0, "v", vec![ST::new_l(0, "k")])],
                )],
            ),
        ],
    );

    (src, dst)
}
