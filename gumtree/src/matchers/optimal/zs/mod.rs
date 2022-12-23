use std::{collections::VecDeque, fmt::Debug, marker::PhantomData};

use num_traits::{cast, one, zero, PrimInt, ToPrimitive};
use str_distance::DistanceMetric;

use crate::decompressed_tree_store::{DecompressedTreeStore, Initializable, PostOrderKeyRoots};
use crate::matchers::mapping_store::MonoMappingStore;
use hyper_ast::types::{LabelStore, NodeStore, SlicedLabel, Stored, Tree};
use logging_timer::time;

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
        SD: 'b + PostOrderKeyRoots<'b, T, M::Src> + Initializable<'store, T>,
        DD: 'b + PostOrderKeyRoots<'b, T, M::Dst> + Initializable<'store, T>,
        T: 'store + Tree,
        S: 'store + NodeStore<T::TreeId, R<'store> = T>,
        LS: 'store + LabelStore<SlicedLabel>,
    {
        let src_arena = SD::new(node_store, &src);
        let dst_arena = DD::new(node_store, &dst);
        // let mappings = ZsMatcher::<M, SD, DD>::match_with(node_store, label_store, &src_arena, &dst_arena);
        let mappings = {
            let mut mappings = M::default();
            mappings.topit(
                (&src_arena).len().to_usize().unwrap() + 1,
                (&dst_arena).len().to_usize().unwrap() + 1,
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

    #[time("warn")]
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

    fn get_update_cost(&self, n1: &T::TreeId, n2: &T::TreeId) -> f64 {
        let t1 = self.node_store.resolve(n1).get_type();
        let t2 = self.node_store.resolve(n2).get_type();
        if t1 == t2 {
            // todo relax comparison on types ?
            let l1 = {
                let r = self.node_store.resolve(n1);
                if !r.has_label() {
                    return 1.0;
                };
                self.label_store.resolve(&r.get_label()).to_owned()
            };
            let l2 = {
                let r = self.node_store.resolve(n2);
                if !r.has_label() {
                    return 1.0;
                };
                self.label_store.resolve(&r.get_label()).to_owned()
            };
            if l1.len() == 0 || l2.len() == 0 {
                return 1.;
            }
            const S_LEN: usize = 3;
            const S: &str = "##";
            // TODO find a way to repeat at compile time
            //format!("{empty:#>width$}", empty = "", width = 3-1);
            //"#".repeat(3 - 1)

            let l1 = {
                let mut tmp = "".to_string();
                tmp.push_str(S);
                tmp.push_str(&l1);
                tmp.push_str(S);
                tmp
            };
            let l2 = {
                let mut tmp = "".to_string();
                tmp.push_str(S);
                tmp.push_str(&l2);
                tmp.push_str(S);
                tmp
            };
            // str_distance::qgram::QGram::new(S).normalized(l1.as_bytes(), l2.as_bytes())
            str_distance_patched::QGram::new(S_LEN).normalized(l1.as_bytes(), l2.as_bytes())
        } else {
            f64::MAX
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
    #[time("warn")]
    pub(crate) fn compute_dist(&self) -> ZsMatcherDist {
        let mut dist = ZsMatcherDist {
            tree: vec![vec![0.0; self.dst_arena.len() + 1]; self.src_arena.len() + 1],
            forest: vec![vec![0.0; self.dst_arena.len() + 1]; self.src_arena.len() + 1],
        };
        let mut src_kr: Vec<_> = self.src_arena.iter_kr().collect();
        if src_kr[src_kr.len() - 1] != self.src_arena.root() {
            src_kr.push(self.src_arena.root());
        }
        let mut dst_kr: Vec<_> = self.dst_arena.iter_kr().collect();
        if dst_kr[dst_kr.len() - 1] != self.dst_arena.root() {
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

    #[time("warn")]
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
mod str_distance_patched {
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
            (self.distance(a, b) as f32 / (len_a + len_b - 2 * self.q + 2) as f32) as f64
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::decompressed_tree_store::{ShallowDecompressedTreeStore, SimpleZsTree as ZsTree};

    use crate::matchers::mapping_store::DefaultMappingStore;
    use crate::{
        decompressed_tree_store::Initializable, tests::examples::example_zs_paper,
        tree::simple_tree::vpair_to_stores,
    };

    #[test]
    fn test_zs_paper_for_initial_layout() {
        let (label_store, compressed_node_store, src, dst) = vpair_to_stores(example_zs_paper());
        // assert_eq!(label_store.resolve(&0).to_owned(), b"");
        let src_arena = {
            let a: ZsTree<_, u16> = ZsTree::<_, _>::new(&compressed_node_store, &src);
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
            let a = ZsTree::<_, u16>::new(&compressed_node_store, &dst);
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
