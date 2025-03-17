//! Zhang and Shasha edit distance algorithm for labeled trees, 1989
//!
//! implementation originally inspired by Gumtree

use crate::decompressed_tree_store::{DecompressedTreeStore, PostOrderKeyRoots};
use crate::matchers::mapping_store::MonoMappingStore;
use hyperast::types::{DecompressedFrom, HyperAST, LabelStore, Labeled, NodeStore};
use hyperast::PrimInt;
use num_traits::{cast, one, zero, ToPrimitive};
use str_distance::DistanceMetric;

// TODO use the Mapping struct
pub struct ZsMatcher<M, SD, DD = SD> {
    pub mappings: M,
    pub src_arena: SD,
    pub dst_arena: DD,
}

impl<SD, DD, M: MonoMappingStore + Default> ZsMatcher<M, SD, DD> {
    pub fn matchh<HAST>(stores: HAST, src: HAST::IdN, dst: HAST::IdN) -> Self
    where
        M::Src: PrimInt,
        M::Dst: PrimInt,
        SD: PostOrderKeyRoots<HAST, M::Src> + DecompressedFrom<HAST, Out = SD>,
        DD: PostOrderKeyRoots<HAST, M::Dst> + DecompressedFrom<HAST, Out = DD>,
        HAST: HyperAST + Copy,
        HAST::Label: Eq,
    {
        let src_arena = SD::decompress(stores, &src);
        let dst_arena = DD::decompress(stores, &dst);
        // let mappings = ZsMatcher::<M, SD, DD>::match_with(stores.node_store(), label_store, &src_arena, &dst_arena);
        let mappings = {
            let mut mappings = M::default();
            mappings.topit(
                (&src_arena).len().to_usize().unwrap(),
                (&dst_arena).len().to_usize().unwrap(),
            );
            let base = MatcherImpl::<SD, DD, HAST, M> {
                stores: stores,
                src_arena: &src_arena,
                dst_arena: &dst_arena,
                phantom: std::marker::PhantomData,
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

    pub fn match_with<HAST>(stores: HAST, src_arena: SD, dst_arena: DD) -> M
    where
        M::Src: PrimInt,
        M::Dst: PrimInt,
        SD: PostOrderKeyRoots<HAST, M::Src>,
        DD: PostOrderKeyRoots<HAST, M::Dst>,
        HAST: HyperAST + Copy,
        HAST::Label: Eq,
    {
        let mut mappings = M::default();
        mappings.topit(
            src_arena.len().to_usize().unwrap() + 1,
            dst_arena.len().to_usize().unwrap() + 1,
        );
        let base = MatcherImpl::<_, _, HAST, M> {
            stores,
            src_arena: &src_arena,
            dst_arena: &dst_arena,
            phantom: std::marker::PhantomData,
        };
        let mut dist = base.compute_dist();
        base.compute_mappings(&mut mappings, &mut dist);
        mappings
    }
}

// TODO use the Mapper struct
pub struct MatcherImpl<'b, 'c, SD, DD, HAST, M> {
    stores: HAST,
    pub src_arena: &'c SD,
    pub dst_arena: &'c DD,
    pub(super) phantom: std::marker::PhantomData<*const (M, &'b ())>,
}


mod qgrams;

#[cfg(test)]
mod other_qgrams;

impl<
        'b: 'c,
        'c,
        SD: PostOrderKeyRoots<HAST, M::Src>,
        DD: PostOrderKeyRoots<HAST, M::Dst>,
        HAST: HyperAST + Copy,
        M: MonoMappingStore,
    > MatcherImpl<'b, 'c, SD, DD, HAST, M>
where
    M::Src: PrimInt,
    M::Dst: PrimInt,
    HAST::Label: Eq,
{
    fn get_deletion_cost(&self, _di: &HAST::IdN) -> f64 {
        1.0
    }

    fn get_insertion_cost(&self, _dj: &HAST::IdN) -> f64 {
        1.0
    }

    fn get_update_cost(
        &self, //cache: &mut Cache<LS::I>,
        r1: &HAST::IdN,
        r2: &HAST::IdN,
    ) -> f64 {
        // if r1 == r2 { // Cannot be used because we return 1 if there is no label in either node
        //     return 0.;
        // }
        let n1 = self.stores.node_store().resolve(r1);
        let t1 = self.stores.resolve_type(r1);
        let l1 = n1.try_get_label();
        let n2 = self.stores.node_store().resolve(r2);
        let t2 = self.stores.resolve_type(r2);
        if t1 != t2 {
            return f64::MAX;
        }
        let Some(l1) = l1 else { return 1.0 };
        let Some(l2) = n2.try_get_label() else {
            return 1.0;
        };
        if l1 == l2 {
            return 0.;
        }
        let s1 = self.stores.label_store().resolve(&l1);
        let s2 = self.stores.label_store().resolve(&l2);
        // debug_assert_ne!(s1.len(), 0);
        // debug_assert_ne!(s2.len(), 0);
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
        'store,
        'b,
        'c,
        's,
        SD: DecompressedTreeStore<HAST, M::Src> + PostOrderKeyRoots<HAST, M::Src>,
        DD: DecompressedTreeStore<HAST, M::Dst> + PostOrderKeyRoots<HAST, M::Dst>,
        HAST: HyperAST + Copy,
        M: MonoMappingStore,
    > MatcherImpl<'b, 'c, SD, DD, HAST, M>
where
    M::Src: PrimInt,
    M::Dst: PrimInt,
    HAST::Label: Eq,
{
    pub(crate) fn compute_dist(&self) -> ZsMatcherDist {
        let mut dist = ZsMatcherDist {
            tree: vec![vec![0.0; self.dst_arena.len() + 1]; self.src_arena.len() + 1],
            forest: vec![vec![0.0; self.dst_arena.len() + 1]; self.src_arena.len() + 1],
        };
        let mut src_kr: Vec<_> = self.src_arena.iter_kr().collect();
        if src_kr.len() == 0 || src_kr[src_kr.len() - 1] != self.src_arena.root() {
            src_kr.push(self.src_arena.root());
        }
        let mut dst_kr: Vec<_> = self.dst_arena.iter_kr().collect();
        if dst_kr.len() == 0 || dst_kr[dst_kr.len() - 1] != self.dst_arena.root() {
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
                            .stores
                            .resolve_type(&self.src_arena.tree(&(row - one())));
                        let t_dst = self
                            .stores
                            .resolve_type(&self.dst_arena.tree(&(col - one())));
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::decompressed_tree_store::{ShallowDecompressedTreeStore, SimpleZsTree as ZsTree};

    use crate::matchers::mapping_store::DefaultMappingStore;
    use crate::matchers::Decompressible;
    use crate::tests::examples::example_zs_paper;
    use crate::tree::simple_tree::TStore;
    use hyperast::test_utils::simple_tree::vpair_to_stores;
    use hyperast::types::HyperASTShared;

    #[test]
    fn test_zs_paper_for_initial_layout() {
        let (stores, src, dst) = vpair_to_stores(example_zs_paper());
        // assert_eq!(label_store.resolve(&0).to_owned(), b"");

        let src_arena = {
            let a: ZsTree<_, u16> = ZsTree::<
                <hyperast::store::SimpleStores<
                    TStore,
                    hyperast::test_utils::simple_tree::NS<hyperast::test_utils::simple_tree::Tree>,
                    hyperast::test_utils::simple_tree::LS<u16>,
                > as HyperASTShared>::IdN,
                _,
            >::decompress(&stores, &src);
            let a = Decompressible {
                hyperast: &stores,
                decomp: a,
            };
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
            // let a = HyperAST::decompress(&stores, &dst);
            let a = ZsTree::<
                <hyperast::store::SimpleStores<
                    TStore,
                    hyperast::test_utils::simple_tree::NS<hyperast::test_utils::simple_tree::Tree>,
                    hyperast::test_utils::simple_tree::LS<u16>,
                > as HyperASTShared>::IdN,
                u16,
            >::decompress(&stores, &dst);
            let a = Decompressible {
                hyperast: &stores,
                decomp: a,
            };
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

        let matcher = MatcherImpl::<_, _, _, DefaultMappingStore<u16>> {
            stores: &stores,
            src_arena: &src_arena,
            dst_arena: &dst_arena,
            phantom: std::marker::PhantomData,
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
