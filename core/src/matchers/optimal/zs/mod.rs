use std::{collections::VecDeque, fmt::Debug, marker::PhantomData};

use num_traits::{cast, one, zero, PrimInt};
use str_distance::DistanceMetric;

use crate::{
    matchers::{
        decompressed_tree_store::{
            DecompressedTreeStore, Initializable as _, PostOrderKeyRoots,
            ShallowDecompressedTreeStore, SimpleZsTree as ZsTree,
        },
        mapping_store::{DefaultMappingStore, MappingStore},
        matcher::Matcher,
    },
    tree::tree::{LabelStore, NodeStore, OwnedLabel, Tree, WithHashs},
};

pub struct ZsMatcher<
    'a,
    D: 'a + DecompressedTreeStore<T::TreeId, IdD>,
    T: Tree + WithHashs,
    IdD: PrimInt + Into<usize>,
    S: for<'b> NodeStore<'b, T>,
    LS: LabelStore<OwnedLabel, I = T::Label>,
> {
    compressed_node_store: &'a S,
    label_store: &'a LS,
    pub(crate) src_arena: D,
    pub(crate) dst_arena: D,
    pub mappings: DefaultMappingStore<IdD>,
    phantom: PhantomData<*const T>,

    tree_dist: Vec<Vec<f64>>,
    forest_dist: Vec<Vec<f64>>,
}

impl<
        'a,
        D: 'a + DecompressedTreeStore<T::TreeId, IdD> + PostOrderKeyRoots<T::TreeId, IdD>,
        T: Tree<TreeId = IdC> + WithHashs,
        IdC: PrimInt,
        IdD: 'a + PrimInt + Into<usize> + std::ops::SubAssign + Debug,
        S: for<'b> NodeStore<'b, T>,
        LS: LabelStore<OwnedLabel, I = T::Label>,
    > ZsMatcher<'a, D, T, IdD, S, LS>
{
    fn f_dist(&self, row: IdD, col: IdD) -> f64 {
        self.forest_dist[row.into()][col.into()]
    }

    pub fn matchh(
        compressed_node_store: &'a S,
        label_store: &'a LS,
        src: T::TreeId,
        dst: T::TreeId,
        mappings: DefaultMappingStore<IdD>,
    ) -> ZsMatcher<'a, ZsTree<T::TreeId, IdD>, T, IdD, S, LS> {
        let mut matcher = ZsMatcher::<'a, ZsTree<T::TreeId, IdD>, T, IdD, S, LS> {
            compressed_node_store,
            src_arena: ZsTree::new(compressed_node_store, &src),
            dst_arena: ZsTree::new(compressed_node_store, &dst),
            mappings,
            phantom: PhantomData,
            tree_dist: vec![],
            forest_dist: vec![],
            label_store: label_store,
        };
        matcher.mappings.topit(
            cast::<_, usize>(matcher.src_arena.len()).unwrap() + 1,
            cast::<_, usize>(matcher.dst_arena.len()).unwrap() + 1,
        );
        ZsMatcher::execute(&mut matcher);
        matcher
    }

    fn execute(&mut self) {
        self.compute_tree_dist();

        let mut root_node_pair = true;

        let mut tree_pairs: VecDeque<(IdD, IdD)> = Default::default(); //ArrayDeque<int[]>

        // push the pair of trees (ted1,ted2) to stack
        tree_pairs.push_front((
            cast::<_, IdD>(self.src_arena.len()).unwrap(),
            cast::<_, IdD>(self.dst_arena.len()).unwrap(),
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
        self.tree_dist = vec![
            vec![0.0; cast::<_, usize>(self.dst_arena.len()).unwrap() + 1];
            cast::<_, usize>(self.src_arena.len()).unwrap() + 1
        ];
        self.forest_dist = vec![
            vec![0.0; cast::<_, usize>(self.dst_arena.len()).unwrap() + 1];
            cast::<_, usize>(self.src_arena.len()).unwrap() + 1
        ];

        for i in 1..self.src_arena.leaf_count().into() {
            for j in 1..self.dst_arena.leaf_count().into() {
                self.forest_dist(
                    self.src_arena.kr(cast(i).unwrap()),
                    self.dst_arena.kr(cast(j).unwrap()),
                );
            }
        }
    }

    pub(crate) fn forest_dist(&mut self, i: IdD, j: IdD) {
        let sa = &self.src_arena;
        let da = &self.dst_arena;
        println!("i:{:?} j:{:?}", i, j);
        let lldsrc = sa.lld(&i).into();
        let llddst = da.lld(&j).into();
        self.forest_dist[lldsrc - 1][llddst - 1] = 0.0;
        for di in lldsrc..i.into() + 1 {
            let odi = cast(di).unwrap();
            let srctree = sa.tree(&odi);
            let lldsrc2 = sa.lld(&odi);
            let cost_del = self.get_deletion_cost(srctree);
            self.forest_dist[di][llddst - 1] = self.forest_dist[di - 1][llddst - 1] + cost_del;
            for dj in llddst..j.into() + 1 {
                let odj = cast(dj).unwrap();
                let dsttree = da.tree(&odj);
                let llddst2 = da.lld(&odj);
                let cost_ins = self.get_insertion_cost(dsttree);
                self.forest_dist[lldsrc - 1][dj] = self.forest_dist[lldsrc - 1][dj - 1] + cost_ins;
                if lldsrc2 == sa.lld(&i) && (llddst2 == da.lld(&j)) {
                    let cost_upd = self.get_update_cost(srctree, dsttree);
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
                        self.forest_dist[lldsrc2.into() - 1][llddst2.into() - 1]
                            + self.tree_dist[di][dj],
                    );
                }
            }
        }
    }

    fn get_deletion_cost(&self, _di: T::TreeId) -> f64 {
        1.0
    }

    fn get_insertion_cost(&self, _dj: T::TreeId) -> f64 {
        1.0
    }

    fn get_update_cost(&self, n1: T::TreeId, n2: T::TreeId) -> f64 {
        let t1 = self.compressed_node_store.resolve(&n1).get_type();
        let t2 = self.compressed_node_store.resolve(&n2).get_type();
        if t1 == t2 {
            // todo relax comparison on types ?
            let l1 = {
                let r = self.compressed_node_store.resolve(&n1);
                if !r.has_label() {
                    return 1.0;
                };
                self.label_store.resolve(&r.get_label()).to_owned()
            };
            let l2 = {
                let r = self.compressed_node_store.resolve(&n2);
                if !r.has_label() {
                    return 1.0;
                };
                self.label_store.resolve(&r.get_label()).to_owned()
            };
            const S: usize = 3;
            let l1 = {
                let mut tmp = vec![];
                tmp.extend_from_slice(&[b'#'; S - 1]);
                tmp.extend_from_slice(&l1);
                tmp.extend_from_slice(&[b'#'; S - 1]);
                tmp
            };
            let l2 = {
                let mut tmp = vec![];
                tmp.extend_from_slice(&[b'#'; S - 1]);
                tmp.extend_from_slice(&l2);
                tmp.extend_from_slice(&[b'#'; S - 1]);
                tmp
            };
            str_distance::qgram::QGram::new(S).normalized(&l1, &l2)
        } else {
            f64::MAX
        }
    }
}

impl<
        'a,
        T: Tree<TreeId = IdC> + WithHashs,
        IdD: 'a + PrimInt + Into<usize> + std::ops::SubAssign + Debug,
        IdC: 'a + PrimInt,
        S: for<'b> NodeStore<'b, T>,
        LS: LabelStore<OwnedLabel, I = T::Label>,
    > Matcher<'a, ZsTree<T::TreeId, IdD>, T, S>
    for ZsMatcher<'a, ZsTree<T::TreeId, IdD>, T, IdD, S, LS>
{
    type Store = DefaultMappingStore<IdD>;

    type Ele = IdD;

    fn matchh(
        compressed_node_store: &'a S,
        src: &'a T::TreeId,
        dst: &'a T::TreeId,
        mappings: Self::Store,
    ) -> Self::Store {
        todo!();
        // let mut matcher = ZsMatcher::<'a, ZsTree<T,HK,HP,IdC,IdD>, T, HK, HP, IdC,IdD, S,LS> {
        //     compressed_node_store,
        //     src_arena: ZsTree::new(compressed_node_store, src),
        //     dst_arena: ZsTree::new(compressed_node_store, dst),
        //     mappings,
        //     phantom: PhantomData,
        //     tree_dist: vec![],
        //     forest_dist: vec![],
        //     label_store: todo!(),
        // };
        // matcher
        //     .mappings
        //     .topit(matcher.src_arena.len(), matcher.dst_arena.len());
        // println!("aaa");
        // ZsMatcher::execute(&mut matcher);
        // matcher.mappings
    }
}
// pub trait ZsStore<IdC: PrimInt, IdD: PrimInt + Into<usize>>: PostOrder<IdC, IdD> {
//     // fn lld(&self, last_row: IdD) -> IdD;
//     // fn tree(&self, row: IdD) -> IdC;
//     // fn get_node_count(&self) -> IdD;
//     // fn get_leaf_count(&self) -> IdD;
//     fn kr(&self, x: IdD) -> IdD;
// }

// pub struct ZsTree<IdC: PrimInt, IdD: PrimInt + Into<usize>> {
//     id_compressed: Vec<IdC>,
//     pub(crate) llds: Vec<IdD>,
//     /// LR_keyroots(T) = {k | there exists no k’> k such that l(k)= l(k’)}.
//     kr: Vec<IdD>,
// }

// impl<IdC: PrimInt, IdD: PrimInt + Into<usize>> PostOrder<IdC, IdD> for ZsTree<IdC, IdD> {
//     fn lld(&self, i: IdD) -> IdD {
//         self.llds[i.into() - 1] + num_traits::one()
//     }

//     // fn tree(&self, i: IdD) -> IdC {
//     //     self.id_compressed[i.into() - 1]
//     // }

//     // fn get_node_count(&self) -> IdD {
//     //     cast(self.id_compressed.len()).unwrap()
//     // }
// }
// impl<IdC: PrimInt, IdD: PrimInt + Into<usize>> ZsStore<IdC, IdD> for ZsTree<IdC, IdD> {
//     fn kr(&self, x: IdD) -> IdD {
//         self.kr[x.into()]
//     }
// }

// impl<IdC: PrimInt, IdD: PrimInt + Into<usize>> DecompressedTreeStore<IdC, IdD>
//     for ZsTree<IdC, IdD>
// {
//     fn new<
//         T: Tree<TreeId = IdC> + WithHashs<HK = HK, HP = HP>,
//         HK: HashKind,
//         HP: PrimInt,
//         S: for<'b> NodeStore<'b,T>,
//     >(
//         store: &S,
//         root: &IdC,
//     ) -> Self {
//         struct R<IdC, Idx, IdD> {
//             curr: IdC,
//             idx: Idx,
//             lld: IdD,
//         }

//         let mut leaf_count = 0;
//         let mut stack = vec![R {
//             curr: *root,
//             idx: zero(),
//             lld: zero(),
//         }];
//         let mut llds: Vec<IdD> = vec![];
//         let mut id_compressed = vec![];
//         loop {
//             if let Some(R { curr, idx, lld }) = stack.pop() {
//                 let x = store.get_node_at_id(&curr);
//                 let l = x.child_count();

//                 if l == zero() {
//                     // leaf
//                     let lld = cast(id_compressed.len()).unwrap();
//                     if let Some(tmp) = stack.last_mut() {
//                         if tmp.idx == one() {
//                             tmp.lld = lld;
//                         }
//                     }
//                     llds.push(lld);
//                     id_compressed.push(curr);
//                     leaf_count += 1;
//                 } else if idx < l {
//                     //
//                     let child = x.get_child(&idx);
//                     stack.push(R {
//                         curr,
//                         idx: idx + one(),
//                         lld: lld,
//                     });
//                     stack.push(R {
//                         curr: child,
//                         idx: zero(),
//                         lld: zero(),
//                     });
//                 } else {
//                     if let Some(tmp) = stack.last_mut() {
//                         if tmp.idx == one() {
//                             tmp.lld = lld;
//                         }
//                     }
//                     id_compressed.push(curr);
//                     llds.push(lld);
//                 }
//             } else {
//                 break;
//             }
//         }

//         let node_count = id_compressed.len();
//         let mut kr = vec![num_traits::zero(); leaf_count + 1];
//         let mut visited = vec![false; node_count];
//         let mut k = kr.len() - 1;
//         for i in (1..node_count).rev() {
//             if !visited[llds[i].into()] {
//                 kr[k] = cast(i + 1).unwrap();
//                 visited[llds[i].into()] = true;
//                 if k > 0 {
//                     k -= 1;
//                 }
//             }
//         }
//         id_compressed.shrink_to_fit();
//         llds.shrink_to_fit();
//         kr.shrink_to_fit();
//         Self {
//             id_compressed,
//             llds,
//             kr,
//         }
//     }

//     fn len(&self) -> usize {
//         self.id_compressed.len()
//     }

//     fn original(&self, id: IdD) -> IdC {
//         self.id_compressed[id.into() - 1]
//     }

//     fn leaf_count(&self) -> IdD {
//         cast(self.kr.len()).unwrap()
//     }

//     fn root(&self) -> IdD {
//         todo!()
//     }

//     fn child<T: Tree<TreeId = IdC>, S: for<'b> NodeStore<'b,T>>(
//         &self,
//         store: &S,
//         x: IdD,
//         p: &[T::ChildIdx],
//     ) -> IdD {
//         todo!()
//     }

//     fn descendants<T: Tree<TreeId = IdC>, S: for<'b> NodeStore<'b,T>>(&self, store: &S, x: IdD) -> Vec<IdD> {
//         todo!()
//     }

//     fn children<
//         T: Tree<TreeId = IdC>,
//         S: for<'b> NodeStore<'b,T>,
//     >(
//         &self,
//         store: &S,
//         x: IdD,
//     ) -> Vec<IdD> {
//         todo!()
//     }

//     // fn has_parent(&self, id: IdD) -> bool {
//     //     todo!() //self.parent(id) != None
//     // }

//     // fn parent(&self, id: IdD) -> Option<IdD> {
//     //     todo!() //let r = self.id_parent[id.to_usize().unwrap()];
//     //             // if r == num_traits::zero() {
//     //             //     None
//     //             // } else {
//     //             //     Some(r)
//     //             // }
//     // }

//     // fn has_children(&self, id: IdD) -> bool {
//     //     todo!() //self.first_child(id) != None
//     // }

//     // fn first_child(&self, id: IdD) -> Option<IdD> {
//     //     todo!() //let r = self.id_first_child[id.to_usize().unwrap()];
//     //             // if r == num_traits::zero() {
//     //             //     None
//     //             // } else {
//     //             //     Some(r)
//     //             // }
//     // }
// }

#[cfg(test)]
mod tests {

    use super::*;

    use crate::{
        matchers::decompressed_tree_store::Initializable,
        tests::{
            examples::example_zs_paper,
            simple_tree::{vpair_to_stores, Tree, LS, NS},
        },
    };

    #[test]
    fn test_zs_paper_for_initial_layout() {
        let (label_store, compressed_node_store, src, dst) = vpair_to_stores(example_zs_paper());
        assert_eq!(label_store.resolve(&0).to_owned(), b"");
        let src_arena = {
            let a = ZsTree::<u16, u16>::new(&compressed_node_store, &src);
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
            let a = ZsTree::<u16, u16>::new(&compressed_node_store, &dst);
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

        let mappings = DefaultMappingStore::new();
        let mut matcher = ZsMatcher::<ZsTree<u16, u16>, Tree, u16, NS<Tree>, LS<u16>> {
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
