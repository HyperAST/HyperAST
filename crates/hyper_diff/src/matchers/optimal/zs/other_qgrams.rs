use super::qgrams::pad;
use crate::matchers::optimal::zs::qgrams::qgram_distance_hash_opti;
use std::collections::{HashMap, HashSet};

#[test]
fn aaa() {
    dbg!(std::str::from_utf8(&pad::<2>(b"abcdefg")).unwrap());
}
#[test]
fn bbb() {
    const Q: usize = 2;
    let s = b"abcdefg";
    let pad_s = pad::<{ Q }>(s);
    pad_s.windows(Q + 1).for_each(|qgram| {
        dbg!(std::str::from_utf8(qgram).unwrap());
    });
    for qgram in s.windows(Q + 1) {
        dbg!(std::str::from_utf8(qgram).unwrap());
    }
}

/// just check fo absence of presence of ngram, not distance
/// give Q - 1 as const parameter to avoid using const generic exprs
fn qgram_metric_hash<const Q: usize>(s: &[u8], t: &[u8]) -> f64 {
    if std::cmp::min(s.len(), t.len()) < Q {
        return if s.eq(t) { 0. } else { 1. };
    }
    // Divide s into q-grams and store them in a hash map
    let mut s_qgrams = HashSet::new();
    let pad_s = pad::<Q>(s);
    pad_s.windows(Q + 1).for_each(|qgram| {
        // dbg!(std::str::from_utf8(qgram).unwrap());
        s_qgrams.insert(qgram);
    });
    for qgram in s.windows(Q + 1) {
        // dbg!(std::str::from_utf8(qgram).unwrap());
        s_qgrams.insert(qgram);
    }

    // Count the number of common q-grams
    let mut qgrams_dist = 0;
    let mut t_qgrams = HashSet::new();
    let pad_t = pad::<Q>(t);
    pad_t.windows(Q + 1).for_each(|qgram| {
        if s_qgrams.contains(qgram) {
            if !t_qgrams.contains(qgram) {
                qgrams_dist += 1;
                t_qgrams.insert(qgram);
            }
        } else if !t_qgrams.contains(qgram) {
            qgrams_dist += 1;
            t_qgrams.insert(qgram);
        }
    });
    for qgram in t.windows(Q + 1) {
        if s_qgrams.contains(qgram) && !t_qgrams.contains(qgram) {
            t_qgrams.insert(qgram);
        } else {
            qgrams_dist += 1;
        }
    }

    // dbg!(&qgrams_dist);
    // dbg!(s.len() + 2 * Q);
    // dbg!(t.len() + 2 * Q);

    // Compute the q-gram distance
    // let distance = qgrams_dist as f64 / (s_qgrams.len() + t_qgrams.len()) as f64;
    // distance
    (qgrams_dist as f32 / ((s.len() + 2 * Q) + (t.len() + 2 * Q) - 2 * (Q + 1) + 2) as f32) as f64
}

/// give Q - 1 as const parameter to avoid using const generic exprs
fn qgram_distance_hash<const Q: usize>(s: &[u8], t: &[u8]) -> f64 {
    if std::cmp::min(s.len(), t.len()) < Q {
        return if s.eq(t) { 0. } else { 1. };
    }
    // Divide s into q-grams and store them in a hash map
    let mut qgrams =
        HashMap::<&[u8], i32, DefaultHashBuilder>::with_hasher(DefaultHashBuilder::default());
    let pad_s = pad::<Q>(s);
    pad_s.windows(Q + 1).for_each(|qgram| {
        // dbg!(std::str::from_utf8(qgram).unwrap());
        *qgrams.entry(qgram).or_insert(0) += 1;
    });
    for qgram in s.windows(Q + 1) {
        // dbg!(std::str::from_utf8(qgram).unwrap());
        *qgrams.entry(qgram).or_insert(0) += 1;
    }

    // Divide t into q-grams and store them in a hash map
    let pad_t = pad::<Q>(t);
    pad_t.windows(Q + 1).for_each(|qgram| {
        // dbg!(std::str::from_utf8(qgram).unwrap());
        *qgrams.entry(qgram).or_insert(0) -= 1;
    });
    for qgram in t.windows(Q + 1) {
        // dbg!(std::str::from_utf8(qgram).unwrap());
        *qgrams.entry(qgram).or_insert(0) -= 1;
    }

    // use specs::prelude::ParallelIterator;
    // let qgrams_dist: u32 = qgrams
    //     .into_par_iter()
    //     .map(|(_, i)| i32::abs(i) as u32)
    //     .sum();
    let qgrams_dist: u32 = qgrams.into_iter().map(|(_, i)| i32::abs(i) as u32).sum();

    // dbg!(&qgrams_dist);
    // dbg!(s.len() + 2 * Q);
    // dbg!(t.len() + 2 * Q);

    // Compute the q-gram distance
    // let distance = qgrams_dist as f64 / (s_qgrams.len() + t_qgrams.len()) as f64;
    // distance
    (qgrams_dist as f32 / ((s.len() + 2 * Q) + (t.len() + 2 * Q) - 2 * (Q + 1) + 2) as f32) as f64
}

/// give Q - 1 as const parameter to avoid using const generic exprs
/// considering bench_hash and bench_single_hash, this is worst than qgram_distance_hash
fn qgram_distance_single_hash<const Q: usize>(s: &[u8], t: &[u8]) -> f64 {
    // Divide s into q-grams and store them in a hash map
    let mut s_qgrams = HashSet::new();
    let pad_s = pad::<Q>(s);
    pad_s.windows(Q + 1).for_each(|qgram| {
        s_qgrams.insert(qgram);
    });
    for qgram in s.windows(Q + 1) {
        s_qgrams.insert(qgram);
    }

    // Count the number of common q-grams
    let mut common_qgrams = 0;
    let pad_t = pad::<Q>(t);
    pad_t.windows(Q + 1).for_each(|qgram| {
        if s_qgrams.contains(qgram) {
            s_qgrams.remove(qgram);
            common_qgrams += 1;
        }
    });
    for qgram in t.windows(Q + 1) {
        if s_qgrams.contains(qgram) {
            s_qgrams.remove(qgram);
            common_qgrams += 1;
        }
    }

    // Compute the q-gram distance
    let distance = common_qgrams as f64 / (s_qgrams.len() + t.len() - Q + 1) as f64;
    distance
}

#[test]
fn validity_qgram_distance_hash() {
    dbg!(qgram_metric_hash::<2>(
        "abaaacdef".as_bytes(),
        "abcdefg".as_bytes()
    ));
    dbg!(qgram_distance_hash_opti(
        "abaaacdef".as_bytes(),
        "abcdefg".as_bytes()
    ));
    dbg!(qgram_distance_hash::<2>(
        "abaaacdef".as_bytes(),
        "abcdefg".as_bytes()
    ));
    use str_distance::DistanceMetric;
    dbg!(super::str_distance_patched::QGram::new(3)
        .normalized("##abaaacdef##".as_bytes(), "##abcdefg##".as_bytes()));
}

extern crate test;
use hyperast::compat::DefaultHashBuilder;
use test::Bencher;

const PAIR1: (&[u8], &[u8]) = ("abaaacdefg".as_bytes(), "abcdefg".as_bytes());
const PAIR2: (&[u8], &[u8]) = (
    "abaaeqrogireiuvnlrpgacdefg".as_bytes(),
    "qvvsdflflvjehrgipuerpq".as_bytes(),
);

#[allow(soft_unstable)]
#[bench]
fn bench_hash(b: &mut Bencher) {
    b.iter(|| qgram_distance_hash::<2>(PAIR1.0, PAIR1.1))
}

#[allow(soft_unstable)]
#[bench]
fn bench_hash_opti(b: &mut Bencher) {
    b.iter(|| qgram_distance_hash_opti(PAIR1.0, PAIR1.1))
}

#[allow(soft_unstable)]
#[bench]
fn bench_hash_opti2(b: &mut Bencher) {
    b.iter(|| qgram_distance_hash_opti(PAIR2.0, PAIR2.1))
}

#[allow(soft_unstable)]
#[bench]
fn bench_single_hash(b: &mut Bencher) {
    b.iter(|| qgram_distance_single_hash::<2>("abcdefg".as_bytes(), "abcdefg".as_bytes()))
}

#[allow(soft_unstable)]
#[bench]
fn bench_str_distance(b: &mut Bencher) {
    use str_distance::DistanceMetric;
    b.iter(|| super::str_distance_patched::QGram::new(3).normalized(PAIR1.0, PAIR1.1))
}

#[allow(soft_unstable)]
#[bench]
fn bench_str_distance2(b: &mut Bencher) {
    use str_distance::DistanceMetric;
    b.iter(|| super::str_distance_patched::QGram::new(3).normalized(PAIR2.0, PAIR2.1))
}
