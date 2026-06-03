use std::collections::HashMap;

use hyperast::compat::DefaultHashBuilder;

const PAD: [u8; 10] = *b"##########";

pub(super) fn pad<const Q: usize>(s: &[u8]) -> Vec<u8> {
    [&s[s.len() - Q..], &PAD[..Q], &s[..Q]].concat()
}

fn make_array<A, T>(slice: &[T]) -> A
where
    A: Sized + Default + AsMut<[T]>,
    T: Copy,
{
    let mut a = Default::default();
    <A as AsMut<[T]>>::as_mut(&mut a).copy_from_slice(slice);
    a
}

pub fn qgram_distance_hash_opti(s: &[u8], t: &[u8]) -> f64 {
    const Q: usize = 3;
    const QM: usize = 2;
    if std::cmp::min(s.len(), t.len()) < Q {
        return if s.eq(t) { 0. } else { 1. };
    }
    let hb = DefaultHashBuilder::default();
    // Divide s into q-grams and store them in a hash map
    let mut qgrams = HashMap::<[u8; Q], i32, DefaultHashBuilder>::with_hasher(hb);
    let pad_s = pad::<QM>(s);
    for i in 0..=pad_s.len() - Q {
        let qgram = make_array(&pad_s[i..i + Q]);
        *qgrams.entry(qgram).or_insert(0) += 1;
    }
    for i in 0..=s.len() - Q {
        let qgram = make_array(&s[i..i + Q]);
        *qgrams.entry(qgram).or_insert(0) += 1;
    }

    // // Divide t into q-grams and store them in a hash map
    let pad_t = pad::<QM>(t);
    for i in 0..=pad_t.len() - Q {
        let qgram = make_array(&pad_t[i..i + Q]);
        *qgrams.entry(qgram).or_insert(0) -= 1;
    }
    for i in 0..=t.len() - Q {
        let qgram = make_array(&t[i..i + Q]);
        *qgrams.entry(qgram).or_insert(0) -= 1;
    }

    let qgrams_dist: u32 = qgrams.into_iter().map(|(_, i)| i32::abs(i) as u32).sum();

    // Compute the q-gram distance
    (qgrams_dist as f32 / ((s.len() + 2 * QM) + (t.len() + 2 * QM) - 2 * (QM + 1) + 2) as f32)
        as f64
}
