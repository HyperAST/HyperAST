use crate::utils::pair::Pair;

#[test]
fn test_equals() {
    let p1 = Pair { 0: "a", 1: "b" };
    let p2: Pair<&str, &str> = ("a", "b").into();
    let p3 = Pair::from(("b", "a"));
    assert_eq!(p1, p1);
    assert_eq!(p1, p2);
    assert_ne!(p1, p3);
    let p4 = Pair::from(("a", "c"));
    assert_ne!(p1, p4);
    let Pair(a, b) = p1;
    assert_eq!(a, "a");
    assert_eq!(b, "b");
    // assert_ne!(p1, null);
    // assert_ne!(p1, "foo");
}

#[test]
fn test_to_string() {
    let p1: Pair<&str, &str> = Pair::from(("a", "b"));
    let p3: Pair<&str, &str> = ("b", "a").into();
    assert_eq!("(a, b)", p1.to_string());
    assert_eq!("(b, a)", p3.to_string());
}
