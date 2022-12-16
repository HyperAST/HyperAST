use std::{collections::VecDeque, fmt::Debug, marker::PhantomData};

use num_traits::{cast, one, zero, PrimInt, ToPrimitive};
use str_distance::DistanceMetric;

use crate::decompressed_tree_store::{
    DecompressedTreeStore, Initializable as _, PostOrderKeyRoots, ShallowDecompressedTreeStore,
    SimpleZsTree as ZsTree,
};
use crate::matchers::mapping_store::{DefaultMappingStore, MappingStore};
use hyper_ast::types::{LabelStore, NodeStore, SlicedLabel, Tree};

pub struct ZsMatcher<
    'a,
    D: 'a,// + DecompressedTreeStore<'a, T, IdD>,
    // IdD: PrimInt + Into<usize>,
    // S: NodeStore<'a, T::TreeId, T>,
    // LS: LabelStore<SlicedLabel, I = T::Label>,
    // T: 'a + Tree + WithHashs,
    IdD,
    T: 'a + Tree,
    // IdC,
    S, //: NodeStore2<T::TreeId, R<'a> = T>, //NodeStore<'a, T::TreeId, T>,
    LS,
> {
    compressed_node_store: &'a S,
    label_store: &'a LS,
    pub src_arena: D,
    pub dst_arena: D,
    pub mappings: DefaultMappingStore<IdD>,
    // phantom: PhantomData<(*const IdC, &'a D)>,
    pub(crate) tree_dist: Vec<Vec<f64>>,
    forest_dist: Vec<Vec<f64>>,
    pub(super) phantom: PhantomData<*const T>,
}

impl<
        'a,
        D: 'a + DecompressedTreeStore<'a, T, IdD> + PostOrderKeyRoots<'a, T, IdD>,
        T: 'a + Tree,
        IdD: PrimInt + std::ops::SubAssign + Debug,
        S: NodeStore<T::TreeId,R<'a>=T>,
        LS: LabelStore<SlicedLabel>,
    > ZsMatcher<'a, D, IdD, T, S, LS>
where
    T::TreeId: Clone,
    T: Tree<Label = LS::I>,
{
    fn f_dist(&self, row: IdD, col: IdD) -> f64 {
        self.forest_dist[row.to_usize().unwrap()][col.to_usize().unwrap()]
    }

    pub(crate) fn make(
        compressed_node_store: &'a S,
        label_store: &'a LS,
        src: T::TreeId,
        dst: T::TreeId,
        mappings: DefaultMappingStore<IdD>,
    ) -> ZsMatcher<'a, ZsTree<T, IdD>, IdD, T, S, LS> {
        let mut matcher = ZsMatcher::<'a, ZsTree<T, IdD>, IdD, T, S, LS> {
            compressed_node_store,
            src_arena: ZsTree::new(compressed_node_store, &src),
            dst_arena: ZsTree::new(compressed_node_store, &dst),
            mappings,
            tree_dist: vec![],
            phantom: PhantomData,
            forest_dist: vec![],
            label_store,
        };
        matcher.mappings.topit(
            matcher.src_arena.len().to_usize().unwrap() + 1,
            matcher.dst_arena.len().to_usize().unwrap() + 1,
        );
        matcher
    }

    pub fn matchh(
        compressed_node_store: &'a S,
        label_store: &'a LS,
        src: T::TreeId,
        dst: T::TreeId,
        mappings: DefaultMappingStore<IdD>,
    ) -> ZsMatcher<'a, ZsTree<T, IdD>, IdD, T, S, LS> {
        let mut matcher = ZsMatcher::<'a, ZsTree<T, IdD>, IdD, T, S, LS>::make(
            compressed_node_store,
            label_store,
            src,
            dst,
            mappings,
        );
        ZsMatcher::execute(&mut matcher);
        matcher
    }

    pub(crate) fn execute(&mut self) {
        self.compute_tree_dist();
        self.compute_mappings();
    }

    pub(crate) fn compute_mappings(&mut self) {
        let mut root_node_pair = true;
        let mut tree_pairs: VecDeque<(IdD, IdD)> = Default::default();
        // push the pair of trees (ted1,ted2) to stack
        tree_pairs.push_front((
            cast(self.src_arena.len()).unwrap(),
            cast(self.dst_arena.len()).unwrap(),
        ));
        while !tree_pairs.is_empty() {
            let tree_pair = tree_pairs.pop_front().unwrap();

            let last_row = tree_pair.0;
            let last_col = tree_pair.1;

            // compute forest distance matrix
            if !root_node_pair {
                self.forest_dist(last_row, last_col);
            }

            root_node_pair = false;

            // compute mapping for current forest distance matrix
            let first_row: IdD = self.src_arena.lld(&last_row) - one();
            let first_col: IdD = self.dst_arena.lld(&last_col) - one();

            let mut row: IdD = cast(last_row).unwrap();
            let mut col: IdD = cast(last_col).unwrap();

            while (row > first_row) || (col > first_col) {
                if (row > first_row)
                    && (self.f_dist(row - one(), col) + 1.0 == self.f_dist(row, col))
                {
                    // node with postorderID row is deleted from ted1
                    row -= one();
                } else if (col > first_col)
                    && (self.f_dist(row, col - one()) + 1.0 == self.f_dist(row, col))
                {
                    // node with postorderID col is inserted into ted2
                    col -= one();
                } else {
                    // node with postorderID row in ted1 is renamed to node col
                    // in ted2
                    if (self.src_arena.lld(&row) == self.src_arena.lld(&last_row))
                        && (self.dst_arena.lld(&col) == self.dst_arena.lld(&last_col))
                    {
                        // if both subforests are trees, map nodes
                        let t_src = self
                            .compressed_node_store
                            .resolve(&self.src_arena.tree(&row))
                            .get_type();
                        let t_dst = self
                            .compressed_node_store
                            .resolve(&self.dst_arena.tree(&col))
                            .get_type();
                        if t_src == t_dst {
                            self.mappings.link(row, col);
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
                        tree_pairs.push_front((row, col));
                        // continue with forest to the left of the popped
                        // subtree pair

                        if row > zero() {
                            row = self.src_arena.lld(&row) - one();
                        } else {
                            row = zero()
                        }
                        if col > zero() {
                            col = self.dst_arena.lld(&col) - one();
                        } else {
                            col = zero()
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn compute_tree_dist(&mut self) {
        self.tree_dist = vec![vec![0.0; self.dst_arena.len() + 1]; self.src_arena.len() + 1];
        self.forest_dist = vec![vec![0.0; self.dst_arena.len() + 1]; self.src_arena.len() + 1];

        for i in 1..self.src_arena.leaf_count().to_usize().unwrap() {
            for j in 1..self.dst_arena.leaf_count().to_usize().unwrap() {
                self.forest_dist(
                    self.src_arena.kr(cast(i).unwrap()),
                    self.dst_arena.kr(cast(j).unwrap()),
                );
            }
        }
        // println!("{:?}",&self.tree_dist);
        // dbg!(&self.forest_dist);
    }

    pub(crate) fn forest_dist(&mut self, i: IdD, j: IdD) {
        let sa = &self.src_arena;
        let da = &self.dst_arena;
        // println!("i:{:?} j:{:?}", i, j);
        let lldsrc = sa.lld(&i).to_usize().unwrap();
        let llddst = da.lld(&j).to_usize().unwrap();
        self.forest_dist[lldsrc - 1][llddst - 1] = 0.0;
        for di in lldsrc..i.to_usize().unwrap() + 1 {
            let odi = cast(di).unwrap();
            let srctree = sa.tree(&odi);
            let lldsrc2 = sa.lld(&odi);
            let cost_del = self.get_deletion_cost(&srctree);
            self.forest_dist[di][llddst - 1] = self.forest_dist[di - 1][llddst - 1] + cost_del;
            for dj in llddst..j.to_usize().unwrap() + 1 {
                let odj = cast(dj).unwrap();
                let dsttree = da.tree(&odj);
                let llddst2 = da.lld(&odj);
                let cost_ins = self.get_insertion_cost(&dsttree);
                self.forest_dist[lldsrc - 1][dj] = self.forest_dist[lldsrc - 1][dj - 1] + cost_ins;
                if lldsrc2 == sa.lld(&i) && (llddst2 == da.lld(&j)) {
                    let cost_upd = self.get_update_cost(&srctree, &dsttree);
                    self.forest_dist[di][dj] = f64::min(
                        f64::min(
                            self.forest_dist[di - 1][dj] + cost_del,
                            self.forest_dist[di][dj - 1] + cost_ins,
                        ),
                        self.forest_dist[di - 1][dj - 1] + cost_upd,
                    );
                    self.tree_dist[di][dj] = self.forest_dist[di][dj];
                } else {
                    self.forest_dist[di][dj] = f64::min(
                        f64::min(
                            self.forest_dist[di - 1][dj] + cost_del,
                            self.forest_dist[di][dj - 1] + cost_ins,
                        ),
                        self.forest_dist[lldsrc2.to_usize().unwrap() - 1]
                            [llddst2.to_usize().unwrap() - 1]
                            + self.tree_dist[di][dj],
                    );
                }
            }
        }
    }

    fn get_deletion_cost(&self, _di: &T::TreeId) -> f64 {
        1.0
    }

    fn get_insertion_cost(&self, _dj: &T::TreeId) -> f64 {
        1.0
    }

    fn get_update_cost(&self, n1: &T::TreeId, n2: &T::TreeId) -> f64 {
        let t1 = self.compressed_node_store.resolve(n1).get_type();
        let t2 = self.compressed_node_store.resolve(n2).get_type();
        if t1 == t2 {
            // todo relax comparison on types ?
            let l1 = {
                let r = self.compressed_node_store.resolve(n1);
                if !r.has_label() {
                    return 1.0;
                };
                self.label_store.resolve(&r.get_label()).to_owned()
            };
            let l2 = {
                let r = self.compressed_node_store.resolve(n2);
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
    use crate::decompressed_tree_store::SimpleZsTree;

    use crate::matchers::mapping_store::VecStore;
    use crate::{
        decompressed_tree_store::Initializable, tests::examples::example_zs_paper,
        tree::simple_tree::vpair_to_stores,
    };

    #[test]
    fn test_zs_paper_for_initial_layout() {
        let (label_store, compressed_node_store, src, dst) = vpair_to_stores(example_zs_paper());
        // assert_eq!(label_store.resolve(&0).to_owned(), b"");
        let src_arena = {
            let a: SimpleZsTree<_, u16> = SimpleZsTree::<_, _>::new(&compressed_node_store, &src);
            // // assert_eq!(a.id_compressed, vec![0, 1, 2, 3, 4, 5]);
            // // // assert_eq!(a.id_parent, vec![0, 0, 0, 1, 1, 4]);
            // // // assert_eq!(a.id_first_child, vec![1, 3, 0, 0, 5, 0]);
            // // assert_eq!(a.llds, vec![3, 3, 2, 3, 5, 5]);
            // // assert_eq!(a.kr, vec![0, 2, 4]);
            // assert_eq!(a.id_compressed, vec![3, 5, 4, 1, 2, 0]);
            // assert_eq!(a.llds, vec![0, 1, 1, 0, 4, 0]);
            // assert_eq!(a.kr, vec![2, 4, 5]);
            a
        };
        let dst_arena = {
            let a = ZsTree::<_, _>::new(&compressed_node_store, &dst);
            // // assert_eq!(a.id_compressed, vec![6, 7, 2, 8, 3, 5]);
            // // // assert_eq!(a.id_parent, vec![0, 0, 0, 1, 3, 3]);
            // // // assert_eq!(a.id_first_child, vec![1, 3, 0, 4, 0, 0]);
            // // assert_eq!(a.llds, vec![4, 4, 2, 4, 4, 5]);
            // // assert_eq!(a.kr, vec![0, 2, 5]);
            // assert_eq!(a.id_compressed, vec![3, 5, 8, 7, 2, 6]);
            // assert_eq!(a.llds, vec![0, 1, 0, 0, 4, 0]);
            // assert_eq!(a.kr, vec![1, 4, 5]);
            a
        };

        let mappings: VecStore<u16> = Default::default();
        let mut matcher = ZsMatcher::<_, u16, _, _, _> {
            compressed_node_store: &compressed_node_store,
            src_arena,
            dst_arena,
            mappings,
            phantom: PhantomData,
            tree_dist: vec![],
            forest_dist: vec![],
            label_store: &label_store,
        };
        // matcher
        //     .mappings
        //     .topit(matcher.src_arena.len(), matcher.dst_arena.len());

        matcher.tree_dist = vec![
            vec![0.0; matcher.dst_arena.len() as usize + 1];
            matcher.src_arena.len() as usize + 1
        ];
        matcher.forest_dist = vec![
            vec![0.0; matcher.dst_arena.len() as usize + 1];
            matcher.src_arena.len() as usize + 1
        ];
        matcher.forest_dist(4, 5);
        println!("{:?}", matcher.tree_dist);
        matcher.tree_dist = vec![
            vec![0.0; matcher.dst_arena.len() as usize + 1];
            matcher.src_arena.len() as usize + 1
        ];
        matcher.forest_dist = vec![
            vec![0.0; matcher.dst_arena.len() as usize + 1];
            matcher.src_arena.len() as usize + 1
        ];
        matcher.forest_dist(4, 5);
    }
}

// impl<IdC: PrimInt, IdD: PrimInt + Into<usize>> ZsTree<IdC, IdD> {
//     fn rec_postorder<
//         T: Tree<TreeId = IdC> + WithHashs<HK = HK, HP = HP>,
//         HK: HashKind,
//         HP: PrimInt,
//         S: for<'b> NodeStore<'b,T>,
//     >(
//         store: &S,
//         curr: &IdC,
//     ) -> Vec<IdC> {
//         let mut i: T::ChildIdx = zero();
//         let mut r = vec![];
//         loop {
//             let x = store.get_node_at_id(&curr);
//             let l = x.child_count();

//             if i < l {
//                 let curr = x.get_child(&i);
//                 let tmp = Self::rec_postorder(store, &curr);
//                 r.extend_from_slice(&tmp);
//                 i = i + one();
//             } else {
//                 break;
//             }
//         }
//         r
//     }

//     fn postorder<
//         T: Tree<TreeId = IdC> + WithHashs<HK = HK, HP = HP>,
//         HK: HashKind,
//         HP: PrimInt,
//         S: for<'b> NodeStore<'b,T>,
//     >(
//         store: &S,
//         root: &IdC,
//     ) -> Vec<IdC> {
//         let mut stack = vec![(*root, zero())];
//         let mut r = vec![];
//         loop {
//             if let Some((curr, idx)) = stack.pop() {
//                 let x = store.get_node_at_id(&curr);
//                 let l = x.child_count();

//                 if idx < l {
//                     let child = x.get_child(&idx);
//                     stack.push((curr, idx + one()));
//                     stack.push((child, zero()));
//                 } else {
//                     r.push(curr);
//                 }
//             } else {
//                 break;
//             }
//         }
//         r
//     }
// }
