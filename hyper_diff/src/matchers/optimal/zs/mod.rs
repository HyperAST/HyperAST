// Zhang and Shasha edit distance algorithm for labeled trees, 1989
//
// implementation originally inspired by Gumtree

use std::{fmt::Debug, marker::PhantomData};

use num_traits::{cast, one, zero, PrimInt, ToPrimitive};
use str_distance::DistanceMetric;

use crate::decompressed_tree_store::{DecompressedTreeStore, PostOrderKeyRoots};
use crate::matchers::mapping_store::MonoMappingStore;
use hyper_ast::types::{LabelStore, NodeStore, SlicedLabel, Stored, Tree, DecompressedSubtree};

// TODO use the Mapping struct
pub struct ZsMatcher<M, SD, DD = SD> {
    pub mappings: M,
    pub src_arena: SD,
    pub dst_arena: DD,
}

impl<SD, DD, M: MonoMappingStore> ZsMatcher<M, SD, DD> {
    pub fn matchh<'store: 'b, 'b: 'c, 'c, T, S, LS>(
        node_store: &'store S,
        label_store: &'store LS,
        src: T::TreeId,
        dst: T::TreeId,
    ) -> Self
    where
        T::TreeId: Clone,
        T: Tree<Label = LS::I>,
        M::Src: PrimInt + std::ops::SubAssign + Debug,
        M::Dst: PrimInt + std::ops::SubAssign + Debug,
        SD: 'b + PostOrderKeyRoots<'b, T, M::Src> + DecompressedSubtree<'store, T>,
        DD: 'b + PostOrderKeyRoots<'b, T, M::Dst> + DecompressedSubtree<'store, T>,
        T: 'store + Tree,
        S: 'store + NodeStore<T::TreeId, R<'store> = T>,
        LS: 'store + LabelStore<SlicedLabel>,
    {
        let src_arena = SD::decompress(node_store, &src);
        let dst_arena = DD::decompress(node_store, &dst);
        // let mappings = ZsMatcher::<M, SD, DD>::match_with(node_store, label_store, &src_arena, &dst_arena);
        let mappings = {
            let mut mappings = M::default();
            mappings.topit(
                (&src_arena).len().to_usize().unwrap(),
                (&dst_arena).len().to_usize().unwrap(),
            );
            let base = MatcherImpl::<'store, 'b, '_, SD, DD, S::R<'store>, S, LS, M> {
                node_store,
                label_store,
                src_arena: &src_arena,
                dst_arena: &dst_arena,
                phantom: PhantomData,
            };
            let mut dist = base.compute_dist();
            base.compute_mappings(&mut mappings, &mut dist);
            mappings
        };
        Self {
            src_arena,
            dst_arena,
            mappings,
        }
    }

    pub fn match_with<'store: 'b, 'b, 'c, T, S, LS>(
        node_store: &'store S,
        label_store: &'store LS,
        src_arena: SD,
        dst_arena: DD,
    ) -> M
    where
        T::TreeId: Clone,
        T: Tree<Label = LS::I>,
        M::Src: PrimInt + std::ops::SubAssign + Debug,
        M::Dst: PrimInt + std::ops::SubAssign + Debug,
        SD: 'b + PostOrderKeyRoots<'b, T, M::Src>,
        DD: 'b + PostOrderKeyRoots<'b, T, M::Dst>,
        T: 'store + Tree,
        S: NodeStore<T::TreeId, R<'store> = T>,
        LS: LabelStore<SlicedLabel>,
    {
        let mut mappings = M::default();
        mappings.topit(
            src_arena.len().to_usize().unwrap() + 1,
            dst_arena.len().to_usize().unwrap() + 1,
        );
        let base = MatcherImpl::<'store, 'b, '_, _, _, S::R<'store>, _, _, M> {
            node_store,
            label_store,
            src_arena: &src_arena,
            dst_arena: &dst_arena,
            phantom: PhantomData,
        };
        let mut dist = base.compute_dist();
        base.compute_mappings(&mut mappings, &mut dist);
        mappings
    }
}

// TODO use the Mapper struct
pub struct MatcherImpl<'store, 'b, 'c, SD: 'b, DD: 'b, T: 'store + Stored, S, LS, M>
where
    S: NodeStore<T::TreeId, R<'store> = T>,
{
    node_store: &'store S,
    label_store: &'store LS,
    pub src_arena: &'c SD,
    pub dst_arena: &'c DD,
    pub(super) phantom: PhantomData<*const (T, M, &'b ())>,
}

impl<
        'store: 'b,
        'b: 'c,
        'c,
        SD: 'c + PostOrderKeyRoots<'b, T, M::Src>,
        DD: 'c + PostOrderKeyRoots<'b, T, M::Dst>,
        T: 'store + Tree,
        S: NodeStore<T::TreeId, R<'store> = T>,
        LS: LabelStore<SlicedLabel>,
        M: MonoMappingStore,
    > MatcherImpl<'store, 'b, 'c, SD, DD, T, S, LS, M>
where
    T::TreeId: Clone,
    T: Tree<Label = LS::I>,
    M::Src: PrimInt + std::ops::SubAssign + Debug,
    M::Dst: PrimInt + std::ops::SubAssign + Debug,
{
    fn get_deletion_cost(&self, _di: &T::TreeId) -> f64 {
        1.0
    }

    fn get_insertion_cost(&self, _dj: &T::TreeId) -> f64 {
        1.0
    }

    fn get_update_cost(
        &self, //cache: &mut Cache<LS::I>,
        r1: &T::TreeId,
        r2: &T::TreeId,
    ) -> f64 {
        // if r1 == r2 { // Cannot be used because we return 1 if there is no label in either node
        //     return 0.;
        // }
        let n1 = self.node_store.resolve(r1);
        let t1 = n1.get_type();
        let l1 = n1.try_get_label();
        let n2 = self.node_store.resolve(r2);
        let t2 = n2.get_type();
        if t1 != t2 {
            return f64::MAX;
        }
        let Some(l1) = l1 else {
            return 1.0
        };
        let Some(l2) = n2.try_get_label() else {
            return 1.0
        };
        if l1 == l2 {
            return 0.;
        }
        let s1 = self.label_store.resolve(&l1);
        let s2 = self.label_store.resolve(&l2);
        debug_assert_ne!(s1.len(), 0);
        debug_assert_ne!(s2.len(), 0);
        if s1.len() == 0 || s2.len() == 0 {
            return 1.;
        }
        const S_LEN: usize = 3;
        let s1 = s1.as_bytes();
        let s2 = s2.as_bytes();
        if s1.len() > 30 || s2.len() > 30 {
            debug_assert_eq!(S_LEN, 3);
            qgrams::qgram_distance_hash_opti(s1, s2)
        } else {
            const S: &[u8] = b"##";
            debug_assert_eq!(S_LEN, 3);
            // TODO find a way to repeat at compile time
            //format!("{empty:#>width$}", empty = "", width = 3-1);
            //"#".repeat(3 - 1)

            let s1 = {
                let mut tmp = S.to_vec();
                tmp.extend_from_slice(&s1);
                tmp.extend_from_slice(S);
                tmp
            };
            let s2 = {
                let mut tmp = S.to_vec();
                tmp.extend_from_slice(&s2);
                tmp.extend_from_slice(S);
                tmp
            };
            let d = str_distance_patched::QGram::new(S_LEN).normalized(s1, s2);
            d
        }
    }
}

pub struct ZsMatcherDist {
    pub(crate) tree: Vec<Vec<f64>>,
    forest: Vec<Vec<f64>>,
}

// TODO make a fully typed interface to each dist
impl ZsMatcherDist {
    fn f_dist<IdD1: PrimInt, IdD2: PrimInt>(&self, row: IdD1, col: IdD2) -> f64 {
        self.forest[row.to_usize().unwrap()][col.to_usize().unwrap()]
    }
}

impl<
        'store: 'b,
        'b: 'c,
        'c,
        SD: 'c + DecompressedTreeStore<'b, T, M::Src> + PostOrderKeyRoots<'b, T, M::Src>,
        DD: 'c + DecompressedTreeStore<'b, T, M::Dst> + PostOrderKeyRoots<'b, T, M::Dst>,
        T: 'store + Tree,
        S: NodeStore<T::TreeId, R<'store> = T>,
        LS: LabelStore<SlicedLabel>,
        M: MonoMappingStore,
    > MatcherImpl<'store, 'b, 'c, SD, DD, T, S, LS, M>
where
    T::TreeId: Clone,
    T: Tree<Label = LS::I>,
    M::Src: PrimInt + std::ops::SubAssign + Debug,
    M::Dst: PrimInt + std::ops::SubAssign + Debug,
{
    pub(crate) fn compute_dist(&self) -> ZsMatcherDist {
        let mut dist = ZsMatcherDist {
            tree: vec![vec![0.0; self.dst_arena.len() + 1]; self.src_arena.len() + 1],
            forest: vec![vec![0.0; self.dst_arena.len() + 1]; self.src_arena.len() + 1],
        };
        let mut src_kr: Vec<_> = self.src_arena.iter_kr().collect();
        if src_kr.len() == 0  || src_kr[src_kr.len() - 1] != self.src_arena.root() {
            src_kr.push(self.src_arena.root());
        }
        let mut dst_kr: Vec<_> = self.dst_arena.iter_kr().collect();
        if dst_kr.len() == 0  || dst_kr[dst_kr.len() - 1] != self.dst_arena.root() {
            dst_kr.push(self.dst_arena.root());
        }
        for i in &src_kr {
            for j in &dst_kr {
                self.forest_dist(&mut dist, &i, &j)
            }
        }
        dist
    }

    pub(crate) fn forest_dist(&self, dist: &mut ZsMatcherDist, i: &M::Src, j: &M::Dst) {
        let sa = self.src_arena;
        let da = self.dst_arena;
        // println!("i:{:?} j:{:?}", i, j);
        let lldsrc = sa.lld(&i).to_usize().unwrap();
        let llddst = da.lld(&j).to_usize().unwrap();
        dist.forest[lldsrc][llddst] = 0.0;
        for di in lldsrc..=i.to_usize().unwrap() {
            let odi = cast(di).unwrap();
            let srctree = sa.tree(&odi);
            let lldsrc2 = sa.lld(&odi);
            let cost_del = self.get_deletion_cost(&srctree);
            dist.forest[di + 1][llddst] = dist.forest[di][llddst] + cost_del;
            for dj in llddst..=j.to_usize().unwrap() {
                let odj = cast(dj).unwrap();
                let dsttree = da.tree(&odj);
                let llddst2 = da.lld(&odj);
                let cost_ins = self.get_insertion_cost(&dsttree);
                dist.forest[lldsrc][dj + 1] = dist.forest[lldsrc][dj] + cost_ins;
                if lldsrc2 == sa.lld(&i) && (llddst2 == da.lld(&j)) {
                    let cost_upd = self.get_update_cost(&srctree, &dsttree);
                    dist.forest[di + 1][dj + 1] = f64::min(
                        f64::min(
                            dist.forest[di][dj + 1] + cost_del,
                            dist.forest[di + 1][dj] + cost_ins,
                        ),
                        dist.forest[di][dj] + cost_upd,
                    );
                    dist.tree[di + 1][dj + 1] = dist.forest[di + 1][dj + 1];
                } else {
                    dist.forest[di + 1][dj + 1] = f64::min(
                        f64::min(
                            dist.forest[di][dj + 1] + cost_del,
                            dist.forest[di + 1][dj] + cost_ins,
                        ),
                        dist.f_dist(lldsrc2, llddst2) + dist.tree[di + 1][dj + 1],
                    );
                }
            }
        }
    }

    pub(crate) fn compute_mappings(&self, mappings: &mut M, dist: &mut ZsMatcherDist) {
        let mut root_node_pair = true;
        let mut tree_pairs: Vec<(M::Src, M::Dst)> = Default::default();
        // push the pair of trees (ted1,ted2) to stack
        tree_pairs.push((self.src_arena.root() + one(), self.dst_arena.root() + one()));
        while !tree_pairs.is_empty() {
            let tree_pair = tree_pairs.pop().unwrap();

            let last_row = tree_pair.0;
            let last_col = tree_pair.1;

            // compute forest distance matrix
            if !root_node_pair {
                self.forest_dist(dist, &(last_row - one()), &(last_col - one()));
            }

            root_node_pair = false;

            // compute mapping for current forest distance matrix
            let first_row: M::Src = self.src_arena.lld(&(last_row - one()));
            let first_col: M::Dst = self.dst_arena.lld(&(last_col - one()));

            let mut row: M::Src = last_row;
            let mut col: M::Dst = last_col;

            while (row > first_row) || (col > first_col) {
                if (row > first_row)
                    && (dist.f_dist(row - one(), col) + 1.0 == dist.f_dist(row, col))
                {
                    // node with postorderID row is deleted from ted1
                    row -= one();
                } else if (col > first_col)
                    && (dist.f_dist(row, col - one()) + 1.0 == dist.f_dist(row, col))
                {
                    // node with postorderID col is inserted into ted2
                    col -= one();
                } else {
                    // node with postorderID row in ted1 is renamed to node col
                    // in ted2
                    debug_assert_ne!(
                        last_row,
                        zero(),
                        "{:?} {:?} {:?} {:?} {:?} {:?}",
                        row,
                        col,
                        first_col,
                        first_row,
                        dist.f_dist(row, col - one()) + 1.0,
                        dist.f_dist(row, col)
                    );
                    debug_assert_ne!(
                        last_col,
                        zero(),
                        "{:?} {:?} {:?} {:?}",
                        row,
                        col,
                        first_col,
                        first_row,
                    );
                    debug_assert_ne!(row, zero(), "{:?} {:?} {:?}", col, first_row, first_col);
                    debug_assert_ne!(col, zero(), "{:?} {:?} {:?}", row, first_row, first_col);
                    if (self.src_arena.lld(&(row - one()))
                        == self.src_arena.lld(&(last_row - one())))
                        && (self.dst_arena.lld(&(col - one()))
                            == self.dst_arena.lld(&(last_col - one())))
                    {
                        // if both subforests are trees, map nodes
                        let t_src = self
                            .node_store
                            .resolve(&self.src_arena.tree(&(row - one())))
                            .get_type();
                        let t_dst = self
                            .node_store
                            .resolve(&self.dst_arena.tree(&(col - one())))
                            .get_type();
                        if t_src == t_dst {
                            mappings.link(row - one(), col - one());
                            // assert_eq!(self.mappings.get_dst(&row),col);
                        } else {
                            panic!("Should not map incompatible nodes.");
                        }
                        if row > zero() {
                            row -= one();
                        }
                        if col > zero() {
                            col -= one();
                        }
                    } else {
                        // pop subtree pair
                        tree_pairs.push((row, col));
                        // continue with forest to the left of the popped
                        // subtree pair
                        if row > zero() {
                            row = row - one();
                            row = self.src_arena.lld(&row);
                        } else {
                            row = zero()
                        }
                        if col > zero() {
                            col = col - one();
                            col = self.dst_arena.lld(&col);
                        } else {
                            col = zero()
                        }
                    }
                }
            }
        }
    }
}

/// TODO waiting for the release of fix of wrong variable on line 5 of normalized()
pub mod str_distance_patched {
    #[derive(Debug, Clone)]
    pub struct QGram {
        /// Length of the fragment
        q: usize,
    }

    impl QGram {
        pub fn new(q: usize) -> Self {
            assert_ne!(q, 0);
            Self { q }
        }
    }

    use str_distance::qgram::QGramIter;
    use str_distance::DistanceMetric;

    impl DistanceMetric for QGram {
        type Dist = usize;

        fn distance<S, T>(&self, a: S, b: T) -> Self::Dist
        where
            S: IntoIterator,
            T: IntoIterator,
            <S as IntoIterator>::IntoIter: Clone,
            <T as IntoIterator>::IntoIter: Clone,
            <S as IntoIterator>::Item: PartialEq + PartialEq<<T as IntoIterator>::Item>,
            <T as IntoIterator>::Item: PartialEq,
        {
            let a: Vec<_> = a.into_iter().collect();
            let b: Vec<_> = b.into_iter().collect();

            let iter_a = QGramIter::new(&a, self.q);
            let iter_b = QGramIter::new(&b, self.q);

            eq_map(iter_a, iter_b)
                .into_iter()
                .map(|(n1, n2)| if n1 > n2 { n1 - n2 } else { n2 - n1 })
                .sum()
        }

        fn normalized<S, T>(&self, a: S, b: T) -> f64
        where
            S: IntoIterator,
            T: IntoIterator,
            <S as IntoIterator>::IntoIter: Clone,
            <T as IntoIterator>::IntoIter: Clone,
            <S as IntoIterator>::Item: PartialEq + PartialEq<<T as IntoIterator>::Item>,
            <T as IntoIterator>::Item: PartialEq,
        {
            let a = a.into_iter();
            let b = b.into_iter();

            let len_a = a.clone().count();
            let len_b = b.clone().count();
            if len_a == 0 && len_b == 0 {
                return 0.0;
            }
            if len_a == 0 || len_b == 0 {
                return 1.0;
            }
            if std::cmp::min(len_a, len_b) <= self.q {
                return if a.eq(b) { 0. } else { 1. };
            }
            let d = self.distance(a, b);
            // dbg!(d);
            // dbg!(len_a);
            // dbg!(len_b);
            // dbg!(self.q);
            (d as f32 / (len_a + len_b - 2 * self.q + 2) as f32) as f64
        }
    }
    fn eq_map<'a, S, T>(a: QGramIter<'a, S>, b: QGramIter<'a, T>) -> Vec<(usize, usize)>
    where
        S: PartialEq + PartialEq<T>,
        T: PartialEq,
    {
        // remove duplicates and count how often a qgram occurs
        fn count_distinct<U: PartialEq>(v: &mut Vec<(U, usize)>) {
            'outer: for idx in (0..v.len()).rev() {
                let (qgram, num) = v.swap_remove(idx);
                for (other, num_other) in v.iter_mut() {
                    if *other == qgram {
                        *num_other += num;
                        continue 'outer;
                    }
                }
                v.push((qgram, num));
            }
        }
        let mut distinct_a: Vec<_> = a.map(|s| (s, 1)).collect();
        let mut distinct_b: Vec<_> = b.map(|s| (s, 1)).collect();

        count_distinct(&mut distinct_a);
        count_distinct(&mut distinct_b);

        let mut nums: Vec<_> = distinct_a.iter().map(|(_, n)| (*n, 0)).collect();

        'outer: for (qgram_b, num_b) in distinct_b {
            for (idx, (qgram_a, num_a)) in distinct_a.iter().enumerate() {
                if *qgram_a == qgram_b {
                    nums[idx] = (*num_a, num_b);
                    continue 'outer;
                }
            }
            nums.push((0, num_b));
        }
        nums
    }
}

pub mod qgrams {
    use std::collections::HashMap;

    use hyper_ast::compat::DefaultHashBuilder;

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
        // the type cannot be inferred!
        // a.as_mut().copy_from_slice(slice);
        <A as AsMut<[T]>>::as_mut(&mut a).copy_from_slice(slice);
        a
    }

    pub fn qgram_distance_hash_opti(s: &[u8], t: &[u8]) -> f64 {
        const Q: usize = 3;
        const QM: usize = 2;
        if std::cmp::min(s.len(), t.len()) < Q {
            return if s.eq(t) { 0. } else { 1. };
        }
        // Divide s into q-grams and store them in a hash map
        let mut qgrams =
            HashMap::<[u8; Q], i32, DefaultHashBuilder>::with_hasher(DefaultHashBuilder::new());
        let pad_s = pad::<QM>(s);
        for i in 0..=pad_s.len() - Q {
            // dbg!(i);
            // dbg!(std::str::from_utf8(&pad_s[i..i + Q]).unwrap());
            let qgram = make_array(&pad_s[i..i + Q]);
            *qgrams.entry(qgram).or_insert(0) += 1;
        }
        for i in 0..=s.len() - Q {
            // dbg!(i);
            // dbg!(std::str::from_utf8(&s[i..i + Q]).unwrap());
            let qgram = make_array(&s[i..i + Q]);
            *qgrams.entry(qgram).or_insert(0) += 1;
        }

        // // Divide t into q-grams and store them in a hash map
        let pad_t = pad::<QM>(t);
        // dbg!(pad_t.len() - Q);
        for i in 0..=pad_t.len() - Q {
            // dbg!(i);
            let qgram = make_array(&pad_t[i..i + Q]);
            // dbg!(std::str::from_utf8(&pad_t[i..i + Q]).unwrap());
            *qgrams.entry(qgram).or_insert(0) -= 1;
        }
        for i in 0..=t.len() - Q {
            // dbg!(i);
            let qgram = make_array(&t[i..i + Q]);
            // dbg!(std::str::from_utf8(&t[i..i + Q]).unwrap());
            *qgrams.entry(qgram).or_insert(0) -= 1;
        }

        let qgrams_dist: u32 = qgrams.into_iter().map(|(_, i)| i32::abs(i) as u32).sum();

        // dbg!(&qgrams_dist);
        // dbg!(s.len() + 2 * Q);
        // dbg!(t.len() + 2 * Q);

        // Compute the q-gram distance
        // let distance = qgrams_dist as f64 / (s_qgrams.len() + t_qgrams.len()) as f64;
        // distance
        (qgrams_dist as f32 / ((s.len() + 2 * QM) + (t.len() + 2 * QM) - 2 * (QM + 1) + 2) as f32)
            as f64
    }
}

#[cfg(test)]
pub(super) mod other_qgrams {
    use crate::matchers::optimal::zs::qgrams::qgram_distance_hash_opti;
    use std::collections::{HashMap, HashSet};

    use super::qgrams::pad;

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
        (qgrams_dist as f32 / ((s.len() + 2 * Q) + (t.len() + 2 * Q) - 2 * (Q + 1) + 2) as f32)
            as f64
    }

    /// give Q - 1 as const parameter to avoid using const generic exprs
    fn qgram_distance_hash<const Q: usize>(s: &[u8], t: &[u8]) -> f64 {
        if std::cmp::min(s.len(), t.len()) < Q {
            return if s.eq(t) { 0. } else { 1. };
        }
        // Divide s into q-grams and store them in a hash map
        let mut qgrams =
            HashMap::<&[u8], i32, DefaultHashBuilder>::with_hasher(DefaultHashBuilder::new());
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
        (qgrams_dist as f32 / ((s.len() + 2 * Q) + (t.len() + 2 * Q) - 2 * (Q + 1) + 2) as f32)
            as f64
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
    use hyper_ast::compat::DefaultHashBuilder;
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
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::decompressed_tree_store::{ShallowDecompressedTreeStore, SimpleZsTree as ZsTree};

    use crate::matchers::mapping_store::DefaultMappingStore;
    use crate::{ tests::examples::example_zs_paper,
        tree::simple_tree::vpair_to_stores,
    };

    #[test]
    fn test_zs_paper_for_initial_layout() {
        let (label_store, compressed_node_store, src, dst) = vpair_to_stores(example_zs_paper());
        // assert_eq!(label_store.resolve(&0).to_owned(), b"");
        let src_arena = {
            let a: ZsTree<_, u16> = ZsTree::<_, _>::decompress(&compressed_node_store, &src);
            // // assert_eq!(a.id_compressed, vec![0, 1, 2, 3, 4, 5]);
            // // // assert_eq!(a.id_parent, vec![0, 0, 0, 1, 1, 4]);
            // // // assert_eq!(a.id_first_child, vec![1, 3, 0, 0, 5, 0]);
            // // assert_eq!(a.llds, vec![3, 3, 2, 3, 5, 5]);
            // // assert_eq!(a.kr, vec![0, 2, 4]);
            // assert_eq!(a.id_compressed, vec![3, 5, 4, 1, 2, 0]);
            assert_eq!(&*a.llds, &vec![0, 1, 1, 0, 4, 0]);
            assert_eq!(a.iter_kr().collect::<Vec<_>>(), vec![2, 4, 5]);
            a
        };
        let dst_arena = {
            let a = ZsTree::<_, u16>::decompress(&compressed_node_store, &dst);
            // // assert_eq!(a.id_compressed, vec![6, 7, 2, 8, 3, 5]);
            // // // assert_eq!(a.id_parent, vec![0, 0, 0, 1, 3, 3]);
            // // // assert_eq!(a.id_first_child, vec![1, 3, 0, 4, 0, 0]);
            // // assert_eq!(a.llds, vec![4, 4, 2, 4, 4, 5]);
            // // assert_eq!(a.kr, vec![0, 2, 5]);
            // assert_eq!(&*a.id_compressed, &vec![3, 5, 8, 7, 2, 6]);
            assert_eq!(&*a.llds, &vec![0, 1, 0, 0, 4, 0]);
            assert_eq!(a.iter_kr().collect::<Vec<_>>(), vec![1, 4, 5]);
            a
        };

        let matcher = MatcherImpl::<_, _, _, _, _, DefaultMappingStore<_>> {
            node_store: &compressed_node_store,
            label_store: &label_store,
            src_arena: &src_arena,
            dst_arena: &dst_arena,
            phantom: PhantomData,
        };

        let tree_dist = vec![
            vec![0.0; matcher.dst_arena.len() as usize + 1];
            matcher.src_arena.len() as usize + 1
        ];
        let forest_dist = vec![
            vec![0.0; matcher.dst_arena.len() as usize + 1];
            matcher.src_arena.len() as usize + 1
        ];
        let mut dist = ZsMatcherDist {
            tree: tree_dist,
            forest: forest_dist,
        };
        matcher.forest_dist(&mut dist, &4, &5);
        println!("{:?}", dist.tree);
        dist.tree = vec![
            vec![0.0; matcher.dst_arena.len() as usize + 1];
            matcher.src_arena.len() as usize + 1
        ];
        dist.forest = vec![
            vec![0.0; matcher.dst_arena.len() as usize + 1];
            matcher.src_arena.len() as usize + 1
        ];
        matcher.forest_dist(&mut dist, &4, &5);
    }
}
