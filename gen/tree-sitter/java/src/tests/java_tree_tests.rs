use crate::java_tree_gen::{JavaTreeGen, TreeGenerator};

#[test]
fn test_equals() {
    let source_code1 = "void test() {}";
    let mut tc1 = &mut JavaTreeGen::new();
    tc1.generate(source_code1.as_bytes());

    let source_code2 = "void test() {}";
    let mut tc2 = &mut JavaTreeGen::new();
    tc2.generate(source_code2.as_bytes());

    assert_eq!(&tc1.treeContext, &tc2.treeContext);
}

#[test]
fn test_to_string() {
}