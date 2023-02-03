use num_traits::PrimInt;

/// Returns the longest common subsequence between two strings.
///
/// @return a list of size 2 u32 arrays that corresponds
///     to match of index in sequence 1 to index in sequence 2.
pub fn longest_common_subsequence_str(s0: &str, s1: &str) -> Vec<(u32, u32)> {
    let mut lens: Vec<Vec<u32>> = vec![vec![0;s1.len() + 1];s0.len() + 1]; //new[s0.len() + 1][s1.len() + 1];
    for (i, c0) in s0.chars().enumerate() {
        for (j, c1) in s1.chars().enumerate() {
            if c0 == c1 {
                lens[i + 1][j + 1] = lens[i][j] + 1;
            } else {
                lens[i + 1][j + 1] = Ord::max(lens[i + 1][j], lens[i][j + 1]);
            }
        }
    }
    return extract_indexes(lens, s0.len(), s1.len());
}

/// Returns the hunks of the longest common subsequence between s1 and s2.
/// @return the hunks as a list of u32 arrays of size 4 with start index and end index of sequence 1
///     and corresponding start index and end index in sequence 2.
pub fn hunks(s0: &str, s1: &str) -> Vec<Vec<u32>> {
    let lcs: Vec<(u32, u32)> = longest_common_subsequence_str(s0, s1);
    let mut hunks: Vec<Vec<u32>> = vec![];
    let matchh = lcs.get(0).unwrap();
    let mut inf0 = matchh.0;
    let mut inf1 = matchh.1;
    let mut last0 = 0;
    let mut last1 = 0;
    for i in 1..lcs.len() {
        let matchh = lcs.get(i).unwrap();
        if last0 + 1 != matchh.0 || last1 + 1 != matchh.1 {
            hunks.push(vec![inf0, last0 + 1, inf1, last1 + 1]);
            inf0 = matchh.0;
            inf1 = matchh.1;
        } else if i == lcs.len() - 1 {
            hunks.push(vec![inf0, matchh.0 + 1, inf1, matchh.1 + 1]);
            break;
        }
        last0 = matchh.0;
        last1 = matchh.1;
    }
    return hunks;
}

/// Returns the longest common sequence between two strings as a string.
pub fn longest_common_sequence<'a>(s1: &'a str, s2: &str) -> &'a str {
    let mut start = 0;
    let mut max = 0;
    let mut it1 = s1.chars();
    for i in 0..s1.len() {
        let mut it2 = s2.chars();
        for j in 0..s2.len() {
            let mut x = 0;
            let mut c1 = it1.clone();
            let mut c2 = it2.clone();
            while c1.next() == c2.next() {
                x += 1;
                if ((i + x) >= s1.len()) || ((j + x) >= s2.len()) {
                    break;
                };
            }
            if x > max {
                max = x;
                start = i;
            }
            it2.next();
        }
        it1.next();
    }
    return &s1[start..start + max];
}

// /// Returns the longest common subsequence between the two list of nodes. This version use
// ///     type and label to ensure equality.
// ///
// /// @see ITree#hasSameTypeAndLabel(ITree)
// /// @return a list of size 2 u32 arrays that corresponds
// ///     to match of index in sequence 1 to index in sequence 2.
// pub fn longest_common_subsequence_with_type_and_label<T:Tree>(s0: &[T], s1: &[T]) -> Vec<(T,T)> {
//     longest_common_subsequence(s0,s1,T::hasSameTypeAndLabel)
// }

// /// Returns the longest common subsequence between the two list of nodes. This version use
// ///     isomorphism to ensure equality.
// ///
// /// @see T#isIsomorphicTo(T)
// /// @return a list of size 2 u32 arrays that corresponds
// ///     to match of index in sequence 1 to index in sequence 2.
// pub fn longest_common_subsequence_with_isomorphism<T:Tree>(s0: &[T], s1: &[T]) -> Vec<(T,T)> {
//     longest_common_subsequence(s0,s1,T::isIsomorphicTo)
// }

// /// Returns the longest common subsequence between the two list of nodes. This version use
// ///     isomorphism to ensure equality.
// ///
// /// @see ITree#isIsoStructuralTo(ITree)
// /// @return a list of size 2 u32 arrays that corresponds
// ///     to match of index in sequence 1 to index in sequence 2.
// pub fn longest_common_subsequence_with_isostructure<T:Tree>(s0: &[ITree], s1: &[ITree]) -> Vec<(ITree,ITree)> {
//     longest_common_subsequence(s0,s1,ITree::isIsoStructuralTo)
// }

pub fn longest_common_subsequence<T1, T2, U: PrimInt, F: Fn(&T1, &T2) -> bool>(
    s0: &[T1],
    s1: &[T2],
    cmp: F,
) -> Vec<(U, U)> {
    let mut lens: Vec<Vec<u32>> = vec![vec![0u32; s1.len() + 1]; s0.len() + 1]; // u32[s0.len() + 1][s1.len() + 1];
    for i in 0..s0.len() {
        //(s0.len()-1).min(0) {
        for j in 0..s1.len() {
            //(s1.len()-1).min(0) {
            if cmp(s0.get(i).unwrap(), s1.get(j).unwrap()) {
                lens[i + 1][j + 1] = lens[i][j] + 1;
            } else {
                lens[i + 1][j + 1] = Ord::max(lens[i + 1][j], lens[i][j + 1]);
            }
        }
    }
    return extract_indexes(lens, s0.len(), s1.len());
}

pub fn extract_indexes<T: Eq, U: PrimInt>(
    lens: Vec<Vec<T>>,
    len1: usize,
    len2: usize,
) -> Vec<(U, U)> {
    let mut indexes = vec![]; //new ArrayList<>();
    let mut x = len1;
    let mut y = len2;
    while x != 0 && y != 0 {
        if lens[x][y] == lens[x - 1][y] {
            x -= 1;
        } else if lens[x][y] == lens[x][y - 1] {
            y -= 1;
        } else {
            indexes.push((
                num_traits::cast(x - 1).unwrap(),
                num_traits::cast(y - 1).unwrap(),
            ));
            x -= 1;
            y -= 1;
        }
    }
    indexes.into_iter().rev().collect()
}
