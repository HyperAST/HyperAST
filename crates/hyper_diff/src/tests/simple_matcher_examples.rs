use hyperast::test_utils::simple_tree::SimpleTree;

/// Example using the datasets/custom/{}/simple_class.java with the tree given by gumtree
pub(crate) fn example_from_gumtree_java_simple() -> (SimpleTree<u8>, SimpleTree<u8>) {
    // Parse the two Java files
    let src_tr = tree!(
        0; [ // program
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "A"), // identifier
                tree!(4), // type_body
            ]),
    ]);
    let dst_tr = tree!(
        0; [ // program
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "B"), // identifier - CHANGED
                tree!(4), // type_body
            ]),
    ]);
    (src_tr, dst_tr)
}

pub(crate) fn example_from_gumtree_java_method() -> (SimpleTree<u8>, SimpleTree<u8>) {
    let src_tr = tree!(
        0; [ // program
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "A"), // identifier
                tree!(4; [ // type_body
                    tree!(5; [ // method_declaration
                        tree!(6, "int"), // type
                        tree!(3, "test"), // identifier
                        tree!(7; [ // formal_parameters
                            tree!(8; [ // formal_parameter
                                tree!(6, "int"), // type
                                tree!(3, "a"), // identifier
                            ]),
                            tree!(8; [ // formal_parameter
                                tree!(6, "String"), // type
                                tree!(3, "b"), // identifier
                            ]),
                        ]),
                        tree!(10; [ // block
                            tree!(11; [ // return_statement
                                tree!(12; [ // binary_expression
                                    tree!(3, "a"), // identifier
                                    tree!(13, "+"), // arithmetic_operator
                                    tree!(3, "b"), // identifier
                                ]),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ]
    );
    let dst_tr = tree!(
        0; [ // program
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "B"), // identifier
                tree!(4; [ // type_body
                    tree!(5; [ // method_declaration
                        tree!(6, "int"), // type
                        tree!(3, "test2"), // identifier
                        tree!(7; [ // formal_parameters
                            tree!(8; [ // formal_parameter
                                tree!(6, "int"), // type
                                tree!(3, "a"), // identifier
                            ]),
                            tree!(8; [ // formal_parameter
                                tree!(6, "String"), // type
                                tree!(3, "c"), // identifier
                            ]),
                        ]),
                        tree!(10; [ // block
                            tree!(14; [ // local_variable_declaration
                                tree!(6, "int"), // type
                                tree!(15; [ // variable_declarator
                                    tree!(3, "b"), // identifier
                                    tree!(16, "="), // affectation_operator
                                    tree!(12; [ // binary_expression
                                        tree!(3, "c"), // identifier
                                        tree!(13, "*"), // arithmetic_operator
                                        tree!(17, "2"), // decimal_integer_literal
                                    ]),
                                ]),
                            ]),
                            tree!(11; [ // return_statement
                                tree!(12; [ // binary_expression
                                    tree!(3, "a"), // identifier
                                    tree!(13, "+"), // arithmetic_operator
                                    tree!(3, "b"), // identifier
                                ]),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ]
    );
    (src_tr, dst_tr)
}

pub(crate) fn example_reorder_children() -> (SimpleTree<u8>, SimpleTree<u8>) {
    // Parse the two Java files
    let src_tr = tree!(
        0; [ // program
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "A"), // identifier
                tree!(4; [ // type_body
                    tree!(5; [ // method_declaration
                        tree!(6, "int"), // type
                        tree!(3, "test2"), // identifier
                        tree!(7; [ // formal_parameters
                            tree!(8; [ // formal_parameter
                                tree!(6, "int"), // type
                                tree!(3, "a"), // identifier
                            ]),
                            tree!(8; [ // formal_parameter
                                tree!(6, "String"), // type
                                tree!(3, "c"), // identifier
                            ]),
                        ]),
                        tree!(10; [ // block
                            tree!(11; [ // return_statement
                                tree!(12; [ // binary_expression
                                    tree!(3, "a"), // identifier
                                    tree!(13, "+"), // arithmetic_operator
                                    tree!(3, "b"), // identifier
                                ]),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "B"), // identifier - CHANGED
                tree!(4), // type_body
            ]),
        ]
    );
    let dst_tr = tree!(
        0; [ // program
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "B"), // identifier - CHANGED
                tree!(4), // type_body
            ]),
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "A"), // identifier
                tree!(4; [ // type_body
                    tree!(5; [ // method_declaration
                        tree!(6, "int"), // type
                        tree!(3, "test2"), // identifier
                        tree!(7; [ // formal_parameters
                            tree!(8; [ // formal_parameter
                                tree!(6, "int"), // type
                                tree!(3, "a"), // identifier
                            ]),
                            tree!(8; [ // formal_parameter
                                tree!(6, "String"), // type
                                tree!(3, "c"), // identifier
                            ]),
                        ]),
                        tree!(10; [ // block
                            tree!(11; [ // return_statement
                                tree!(12; [ // binary_expression
                                    tree!(3, "a"), // identifier
                                    tree!(13, "+"), // arithmetic_operator
                                    tree!(3, "b"), // identifier
                                ]),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ]
    );
    (src_tr, dst_tr)
}

pub(crate) fn example_move_method() -> (SimpleTree<u8>, SimpleTree<u8>) {
    // Parse the two Java files
    let src_tr = tree!(
        0; [ // program
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "A"), // identifier
                tree!(4; [ // type_body
                    tree!(5; [ // method_declaration
                        tree!(6, "int"), // type
                        tree!(3, "test"), // identifier
                        tree!(7; [ // formal_parameters
                            tree!(8; [ // formal_parameter
                                tree!(6, "int"), // type
                                tree!(3, "a"), // identifier
                            ]),
                            tree!(8; [ // formal_parameter
                                tree!(6, "String"), // type
                                tree!(3, "b"), // identifier
                            ]),
                        ]),
                        tree!(10; [ // block
                            tree!(11; [ // return_statement
                                tree!(12; [ // binary_expression
                                    tree!(3, "a"), // identifier
                                    tree!(13, "+"), // arithmetic_operator
                                    tree!(3, "b"), // identifier
                                ]),
                            ]),
                        ]),
                    ]),
                    tree!(5; [ // method_declaration
                        tree!(6, "String"), // type
                        tree!(3, "stuff"), // identifier
                        tree!(7; [ // formal_parameters
                            tree!(8; [ // formal_parameter
                                tree!(6, "String"), // type
                                tree!(3, "b"), // identifier
                            ]),
                        ]),
                        tree!(10; [ // block
                            tree!(11; [ // return_statement
                                tree!(3, "b"), // identifier
                            ]),
                        ]),
                    ]),
                ]),
            ]),
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "B"), // identifier - CHANGED
                tree!(4), // type_body
            ]),
        ]
    );
    let dst_tr = tree!(
        0; [ // program
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "A"), // identifier - CHANGED
                tree!(4; [ // type_body
                     tree!(5; [ // method_declaration
                        tree!(6, "String"), // type
                        tree!(3, "stuff"), // identifier
                        tree!(7; [ // formal_parameters
                            tree!(8; [ // formal_parameter
                                tree!(6, "String"), // type
                                tree!(3, "b"), // identifier
                            ]),
                        ]),
                        tree!(10; [ // block
                            tree!(11; [ // return_statement
                                tree!(3, "b"), // identifier
                            ]),
                        ]),
                    ]),
                ]), // type_body
            ]),
            tree!(1; [ // type_declaration
                tree!(2, "class"), // type_keyword
                tree!(3, "B"), // identifier
                tree!(4; [ // type_body
                    tree!(5; [ // method_declaration
                        tree!(6, "int"), // type
                        tree!(3, "test"), // identifier
                        tree!(7; [ // formal_parameters
                            tree!(8; [ // formal_parameter
                                tree!(6, "int"), // type
                                tree!(3, "a"), // identifier
                            ]),
                            tree!(8; [ // formal_parameter
                                tree!(6, "String"), // type
                                tree!(3, "b"), // identifier
                            ]),
                        ]),
                        tree!(10; [ // block
                            tree!(11; [ // return_statement
                                tree!(12; [ // binary_expression
                                    tree!(3, "a"), // identifier
                                    tree!(13, "+"), // arithmetic_operator
                                    tree!(3, "b"), // identifier
                                ]),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ]
    );
    (src_tr, dst_tr)
}
