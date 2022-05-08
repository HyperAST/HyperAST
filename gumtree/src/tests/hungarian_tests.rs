use crate::utils::sequence_algorithms::{longest_common_sequence, longest_common_subsequence_str};

#[test]
fn test() {
    let r = hungarian::minimize(&[1, 1, 1, 1, 1, 1], 2, 3);
    println!("{:?}", r);
}

#[test]
fn test_lcss() {
    // Exemple coming from:
    // http://www.geeksforgeeks.org/dynamic-programming-set-4-longest-common-subsequence/
    let indexes = longest_common_subsequence_str("ABCDGH", "AEDFHR");
    assert_eq!(indexes.len(), 3);
    assert!(indexes.contains(&(0, 0)));
    assert!(indexes.contains(&(3, 2)));
    assert!(indexes.contains(&(5, 4)));
}

#[test]
fn test_lcs() {
    let lcs = longest_common_sequence("FUTUR", "CHUTE");
    assert_eq!(lcs, "UT");
}
#[test]
fn test_for_mut() {
    let v = vec![0, 0, 0];
    for mut i in &v {
        i = &5;
        assert_eq!(i, &5)
    }
    assert_eq!(v, vec![0, 0, 0]);
}

#[test]
fn test_hungarian_algorithm() {
    // Exemple coming from https://en.wikipedia.org/wiki/Hungarian_algorithm
    let cost_matrix = [2, 3, 3, 3, 2, 3, 3, 3, 2];
    let result = hungarian::minimize(&cost_matrix, 3, 3);
    assert_eq!(result[0], Some(0));
    assert_eq!(result[1], Some(1));
    assert_eq!(result[2], Some(2));
}
