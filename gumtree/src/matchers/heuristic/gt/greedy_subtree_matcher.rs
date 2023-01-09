use std::hash::Hash;
use std::{fmt::Debug, marker::PhantomData};

use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent,
};
use crate::matchers::heuristic::gt::height;
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::{mapping_store::MultiMappingStore, similarity_metrics};
use crate::utils::sequence_algorithms::longest_common_subsequence;
use hyper_ast::compat::HashMap;
use hyper_ast::types::{
    DecompressedSubtree, HashKind, HyperAST, IterableChildren, NodeStore, Tree, WithHashs,
};
use num_traits::{one, zero, PrimInt, ToPrimitive};

pub struct GreedySubtreeMatcher<'a, Dsrc, Ddst, T, S, M, const MIN_HEIGHT: usize = 1> {
    internal: SubtreeMatcher<'a, Dsrc, Ddst, T, S, M, MIN_HEIGHT>,
}

impl<
        'a,
        Dsrc: 'a
            + DecompressedTreeStore<'a, T, M::Src>
            + DecompressedWithParent<'a, T, M::Src>
            + DecompressedSubtree<'a, T>
            + ContiguousDescendants<'a, T, M::Src>,
        Ddst: 'a
            + DecompressedTreeStore<'a, T, M::Dst>
            + DecompressedWithParent<'a, T, M::Dst>
            + DecompressedSubtree<'a, T>
            + ContiguousDescendants<'a, T, M::Dst>,
        T: Tree + WithHashs,
        S,
        M: MonoMappingStore,
        const MIN_HEIGHT: usize, // = 2
    > GreedySubtreeMatcher<'a, Dsrc, Ddst, T, S, M, MIN_HEIGHT>
where
    S: 'a + NodeStore<T::TreeId, R<'a> = T>,
    T::TreeId: Clone,
    T::Label: Clone,
    M::Src: 'a + PrimInt + Debug + Hash,
    M::Dst: 'a + PrimInt + Debug + Hash,
{
    pub fn match_it<HAST, MM>(
        mapping: crate::matchers::Mapper<'a, HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<'a, HAST, Dsrc, Ddst, M>
    where
        Self: 'a,
        HAST: HyperAST<'a, NS = S>,
        MM: MultiMappingStore<Src = M::Src, Dst = M::Dst>,
    {
        let mut matcher = Self {
            internal: SubtreeMatcher {
                node_store: mapping.hyperast.node_store(),
                src_arena: mapping.mapping.src_arena,
                dst_arena: mapping.mapping.dst_arena,
                mappings: mapping.mapping.mappings,
                phantom: PhantomData,
            },
        };
        matcher.internal.mappings.topit(
            matcher.internal.src_arena.len(),
            matcher.internal.dst_arena.len(),
        );
        Self::execute::<MM>(&mut matcher);
        crate::matchers::Mapper {
            hyperast: mapping.hyperast,
            mapping: crate::matchers::Mapping {
                src_arena: matcher.internal.src_arena,
                dst_arena: matcher.internal.dst_arena,
                mappings: matcher.internal.mappings,
            },
        }
    }

    pub fn matchh<MM: MultiMappingStore<Src = M::Src, Dst = M::Dst>>(
        node_store: &'a S,
        src: &'a T::TreeId,
        dst: &'a T::TreeId,
        mappings: M,
    ) -> GreedySubtreeMatcher<'a, Dsrc, Ddst, T, S, M, MIN_HEIGHT>
    where
        Self: 'a,
    {
        let mut matcher = GreedySubtreeMatcher::<'a, Dsrc, Ddst, T, S, M, MIN_HEIGHT> {
            internal: SubtreeMatcher {
                node_store,
                src_arena: Dsrc::decompress(node_store, src),
                dst_arena: Ddst::decompress(node_store, dst),
                mappings,
                phantom: PhantomData,
            },
        };
        matcher.internal.mappings.topit(
            matcher.internal.src_arena.len(),
            matcher.internal.dst_arena.len(),
        );
        Self::execute::<MM>(&mut matcher);
        matcher
    }

    pub(crate) fn execute<MM: MultiMappingStore<Src = M::Src, Dst = M::Dst>>(&mut self) {
        let mut mm: MM = Default::default();
        mm.topit(self.internal.src_arena.len(), self.internal.dst_arena.len());
        self.internal.matchh_to_be_filtered(&mut mm);
        self.filter_mappings(&mm);
    }

    fn filter_mappings<MM: MultiMappingStore<Src = M::Src, Dst = M::Dst>>(
        &mut self,
        multi_mappings: &MM,
    ) {
        // Select unique mappings first and extract ambiguous mappings.
        let mut ambiguous_list: Vec<(M::Src, M::Dst)> = vec![];
        let mut ignored = bitvec::bitbox![0;self.internal.src_arena.len()];
        let mut src_ignored = bitvec::bitbox![0;self.internal.src_arena.len()];
        let mut dst_ignored = bitvec::bitbox![0;self.internal.dst_arena.len()];
        for src in multi_mappings.all_mapped_srcs() {
            let mut is_mapping_unique = false;
            if multi_mappings.is_src_unique(&src) {
                let dst = multi_mappings.get_dsts(&src)[0];
                if multi_mappings.is_dst_unique(&dst) {
                    self.internal.add_mapping_recursively(&src, &dst); // TODO subtree opti, do not do explicitly
                    is_mapping_unique = true;
                }
            }

            if !(ignored[src.to_usize().unwrap()] || is_mapping_unique) {
                let adsts = multi_mappings.get_dsts(&src);
                let asrcs = multi_mappings.get_srcs(&multi_mappings.get_dsts(&src)[0]);
                for asrc in asrcs {
                    for adst in adsts {
                        ambiguous_list.push((*asrc, *adst));
                    }
                }
                asrcs
                    .iter()
                    .for_each(|x| ignored.set(x.to_usize().unwrap(), true))
            }
        }

        let mapping_list: Vec<_> = self.sort(ambiguous_list).collect();

        // Select the best ambiguous mappings
        for (src, dst) in mapping_list {
            // println!("mapping={:?},{:?}", src, dst);
            let src_i = src.to_usize().unwrap();
            let dst_i = dst.to_usize().unwrap();
            if !(src_ignored[src_i] || dst_ignored[dst_i]) {
                // println!("selected={:?},{:?}", src, dst);
                self.internal.add_mapping_recursively(&src, &dst);
                src_ignored.set(src_i, true);
                self.internal
                    .src_arena
                    .descendants(self.internal.node_store, &src)
                    .iter()
                    .for_each(|src| src_ignored.set(src.to_usize().unwrap(), true));
                dst_ignored.set(dst_i, true);
                self.internal
                    .dst_arena
                    .descendants(self.internal.node_store, &dst)
                    .iter()
                    .for_each(|dst| dst_ignored.set(dst.to_usize().unwrap(), true));
            }
        }
    }

    fn sort(
        &self,
        mut ambiguous_mappings: Vec<(M::Src, M::Dst)>,
    ) -> impl Iterator<Item = (M::Src, M::Dst)> {
        let mut sib_sim = HashMap::<(M::Src, M::Dst), f64>::default();
        let mut psib_sim = HashMap::<(M::Src, M::Dst), f64>::default();
        let mut p_in_p_sim = HashMap::<(M::Src, M::Dst), f64>::default();
        dbg!(&ambiguous_mappings.len());
        ambiguous_mappings.sort_by(|a, b| {
            let cached_coef_sib = |l: &(M::Src, M::Dst)| {
                sib_sim
                    .entry(*l)
                    .or_insert_with(|| self.coef_sib(&l))
                    .clone()
            };
            let cached_coef_parent = |l: &(M::Src, M::Dst)| {
                psib_sim
                    .entry(*l)
                    .or_insert_with(|| self.coef_parent(&l))
                    .clone()
            };
            let (alink, blink) = (a, b);
            if self.same_parents(alink, blink) {
                std::cmp::Ordering::Equal
            } else {
                self.cached_compare(cached_coef_sib, a, b)
                    .reverse()
                    .then_with(|| self.cached_compare(cached_coef_parent, a, b).reverse())
            }
            .then_with(|| {
                self.cached_compare(
                    |l: &(M::Src, M::Dst)| {
                        p_in_p_sim
                            .entry(*l)
                            .or_insert_with(|| self.coef_pos_in_parent(&l))
                            .clone()
                    },
                    a,
                    b,
                )
            })
            .then_with(|| self.compare_delta_pos(alink, blink))
        });
        ambiguous_mappings.into_iter()
    }

    fn cached_compare<I, F: FnMut(&I) -> O, O: PartialOrd>(
        &self,
        mut cached: F,
        a: &I,
        b: &I,
    ) -> std::cmp::Ordering {
        cached(a)
            .partial_cmp(&cached(b))
            .unwrap_or(std::cmp::Ordering::Equal)
    }

    fn coef_sib(&self, l: &(M::Src, M::Dst)) -> f64 {
        let (p_src, p_dst) = self.parents(l);
        similarity_metrics::SimilarityMeasure::range(
            &self.internal.src_arena.descendants_range(&p_src), //descendants
            &self.internal.dst_arena.descendants_range(&p_dst),
            &self.internal.mappings,
        )
        .dice()
    }

    fn parents(&self, l: &(M::Src, M::Dst)) -> (M::Src, M::Dst) {
        let p_src = self.internal.src_arena.parent(&l.0).unwrap();
        let p_dst = self.internal.dst_arena.parent(&l.1).unwrap();
        (p_src, p_dst)
    }

    fn coef_parent(&self, l: &(M::Src, M::Dst)) -> f64 {
        let s1: Vec<_> = Dsrc::parents(&self.internal.src_arena, l.0).collect();
        let s2: Vec<_> = Ddst::parents(&self.internal.dst_arena, l.1).collect();
        // let s2: Vec<_> = self.internal.dst_arena.parents::<Ddst>(&l.1).collect();
        let common = longest_common_subsequence::<_, _, usize, _>(&s1, &s2, |a, b| {
            let (t, l) = {
                let o = self.internal.src_arena.original(a);
                let n = self.internal.node_store.resolve(&o);
                (n.get_type(), n.try_get_label().cloned())
            };
            let o = self.internal.dst_arena.original(b);
            let n = self.internal.node_store.resolve(&o);
            t == n.get_type() && l.as_ref() == n.try_get_label()
        });
        (2 * common.len()).to_f64().unwrap() / (s1.len() + s2.len()).to_f64().unwrap()
    }

    fn coef_pos_in_parent(&self, l: &(M::Src, M::Dst)) -> f64 {
        // let f = |d: _, s, x| {
        //     vec![x].into_iter().chain(d.parents(&x)).filter_map(|x| {
        //         d.parent(&x).map(|p| {
        //             d.position_in_parent(s, &x).to_f64().unwrap()
        //                 / d.children(s, &p).len().to_f64().unwrap()
        //         })
        //     })
        // };
        // let srcs = f(self.internal.src_arena, self.internal.node_store, l.0);
        // let srcs = norm_path(&self.internal.src_arena, self.internal.node_store, l.0);
        // let dsts = norm_path(&self.internal.dst_arena, self.internal.node_store, l.1);
        let srcs = vec![l.0]
            .into_iter()
            .chain(self.internal.src_arena.parents(l.0))
            .filter_map(|x| {
                self.internal.src_arena.parent(&x).map(|p| {
                    self.internal
                        .src_arena
                        .position_in_parent(&x)
                        .unwrap()
                        .to_f64()
                        .unwrap()
                        / self
                            .internal
                            .src_arena
                            .children(self.internal.node_store, &p)
                            .len()
                            .to_f64()
                            .unwrap()
                })
            });
        let dsts = vec![l.1]
            .into_iter()
            .chain(self.internal.dst_arena.parents(l.1))
            .filter_map(|x| {
                self.internal.dst_arena.parent(&x).map(|p| {
                    self.internal
                        .dst_arena
                        .position_in_parent(&x)
                        .unwrap()
                        .to_f64()
                        .unwrap()
                        / self
                            .internal
                            .dst_arena
                            .children(self.internal.node_store, &p)
                            .len()
                            .to_f64()
                            .unwrap()
                })
            });
        srcs.zip(dsts)
            .map(|(src, dst)| (src - dst) * (src - dst))
            .sum::<f64>()
            .sqrt()
    }

    fn same_parents(&self, alink: &(M::Src, M::Dst), blink: &(M::Src, M::Dst)) -> bool {
        let ap = self.mapping_parents(&alink);
        let bp = self.mapping_parents(&blink);
        ap.0 == bp.0 && ap.1 == bp.1
    }

    fn mapping_parents(&self, l: &(M::Src, M::Dst)) -> (Option<M::Src>, Option<M::Dst>) {
        (
            self.internal.src_arena.parent(&l.0),
            self.internal.dst_arena.parent(&l.1),
        )
    }

    // fn compare_parent_pos(&self, alink: &(M::Src, M::Dst), blink: &(M::Src, M::Dst)) -> std::cmp::Ordering {
    //     let al = self.internal.src_arena.parent(&alink.0);
    //     let bl = self.internal.src_arena.parent(&blink.0);
    //     if al != bl {
    //         return al.cmp(&bl);
    //     }
    //     let al = self.internal.dst_arena.parent(&alink.1);
    //     let bl = self.internal.dst_arena.parent(&blink.1);
    //     al.cmp(&bl)
    // }

    // fn compare_pos(&self, alink: &(M::Src, M::Dst), blink: &(M::Src, M::Dst)) -> std::cmp::Ordering {
    //     if alink.0 != blink.0 {
    //         return alink.0.cmp(&blink.0);
    //     }
    //     return alink.1.cmp(&blink.1);
    // }

    fn compare_delta_pos(
        &self,
        alink: &(M::Src, M::Dst),
        blink: &(M::Src, M::Dst),
    ) -> std::cmp::Ordering {
        return (alink
            .0
            .to_usize()
            .unwrap()
            .abs_diff(alink.1.to_usize().unwrap()))
        .cmp(
            &blink
                .0
                .to_usize()
                .unwrap()
                .abs_diff(blink.1.to_usize().unwrap()),
        );
    }

    // fn sim_sort(
    //     &self,
    //     ambiguous_mappings: Vec<Mapping<IdD>>,
    // ) -> impl Iterator<Item = Mapping<IdD>> {
    //     let mut similarities: Vec<_> = ambiguous_mappings
    //         .into_iter()
    //         .map(|p| (p, self.internal.similarity(&p.0, &p.1)))
    //         .collect();
    //     similarities.sort_by(|(alink, asim), (blink, bsim)| -> std::cmp::Ordering {
    //         if asim != bsim {
    //             // todo caution about exact comparing of floats
    //             if let Some(r) = asim.partial_cmp(&bsim) {
    //                 return r;
    //             }
    //         }
    //         if alink.0 != blink.0 {
    //             return alink.0.cmp(&blink.0);
    //         }
    //         return alink.1.cmp(&blink.1);
    //     });
    //     similarities.into_iter().map(|(x, _)| x)
    // }
}

// TODO remove lifetime from Decompressed traits, anyway we could always add back a wrapper that materializes the relation to the node store
// fn norm_path<'b, 'a: 'b, IdC: 'a + Clone, IdD: 'a + Clone, SS: 'a + NodeStore<IdC>, D>(
//     d: &'b D,
//     s: &'b SS,
//     x: IdD,
// ) -> impl Iterator<Item = f64> + 'b
// where
//     SS::R<'a>: WithChildren<TreeId = IdC>,
//     D: DecompressedWithParent<'a, IdC, IdD> + ShallowDecompressedTreeStore<'a, IdC, IdD>,
// {
//     vec![x.clone()]
//         .into_iter()
//         .chain(D::parents(d, x))
//         .filter_map(|x| {
//             d.parent(&x).map(|p| {
//                 d.position_in_parent(s, &x).to_f64().unwrap()
//                     / d.children(s, &p).len().to_f64().unwrap()
//             })
//         })
// }
impl<'a, Dsrc, Ddst, T: Tree, S, M: MonoMappingStore, const MIN_HEIGHT: usize>
    Into<SubtreeMatcher<'a, Dsrc, Ddst, T, S, M, MIN_HEIGHT>>
    for GreedySubtreeMatcher<'a, Dsrc, Ddst, T, S, M, MIN_HEIGHT>
{
    fn into(self) -> SubtreeMatcher<'a, Dsrc, Ddst, T, S, M, MIN_HEIGHT> {
        self.internal
    }
}

// impl<'a, Dsrc, Ddst, T: 'a + Tree, HAST, M, const MIN_HEIGHT: usize>
//     From<crate::matchers::Mapper<'a, HAST, Dsrc, Ddst, M>>
//     for SubtreeMatcher<'a, Dsrc, Ddst, T, HAST, M, MIN_HEIGHT>
// {
//     fn from(value: crate::matchers::Mapper<'a, HAST, Dsrc, Ddst, M>) -> Self {
//         Self {
//             node_store: value.hyperast.,
//             src_arena: value.mapping.src_arena,
//             dst_arena: value.mapping.dst_arena,
//             mappings: value.mapping.mappings,
//             phantom: PhantomData,
//         }
//     }
// }

// impl<'a, Dsrc, Ddst, T: 'a + Tree, S, M: MonoMappingStore, const MIN_HEIGHT: usize>
//     From<GreedySubtreeMatcher<'a, Dsrc, Ddst, T, S, M, MIN_HEIGHT>>
//     for crate::matchers::Mapper<'a, S, Dsrc, Ddst, M>
// {
//     fn from(value: GreedySubtreeMatcher<'a, Dsrc, Ddst, T, S, M, MIN_HEIGHT>) -> Self {
//         Self { node_store: value.internal.node_store, mapping: Mapping {
//             src_arena: value.internal.src_arena,
//             dst_arena: value.internal.dst_arena,
//             mappings: value.internal.mappings,
//         } }
//     }
// }

pub struct SubtreeMatcher<'a, Dsrc, Ddst, T, S, M, const MIN_HEIGHT: usize> {
    pub(super) node_store: &'a S,
    pub src_arena: Dsrc,
    pub dst_arena: Ddst,
    pub mappings: M,
    pub(super) phantom: PhantomData<*const T>,
}

impl<
        'a,
        Dsrc: DecompressedTreeStore<'a, T, M::Src> + DecompressedWithParent<'a, T, M::Src>,
        Ddst: DecompressedTreeStore<'a, T, M::Dst> + DecompressedWithParent<'a, T, M::Dst>,
        T: Tree + WithHashs,
        S,
        M: MonoMappingStore,
        const MIN_HEIGHT: usize,
    > SubtreeMatcher<'a, Dsrc, Ddst, T, S, M, MIN_HEIGHT>
where
    S: 'a + NodeStore<T::TreeId, R<'a> = T>,
    M::Src: PrimInt + Debug,
    M::Dst: PrimInt + Debug,
    T::TreeId: Clone,
{
    pub(crate) fn add_mapping_recursively(&mut self, src: &M::Src, dst: &M::Dst) {
        self.mappings.link(*src, *dst);
        self.src_arena
            .descendants(self.node_store, src)
            .iter()
            .zip(self.dst_arena.descendants(self.node_store, dst).iter())
            .for_each(|(src, dst)| self.mappings.link(*src, *dst));
    }

    fn pop_larger<'b>(
        &self,
        src_trees: &mut PriorityTreeList<'a, 'b, Dsrc, M::Src, T, S, MIN_HEIGHT>,
        dst_trees: &mut PriorityTreeList<'a, 'b, Ddst, M::Dst, T, S, MIN_HEIGHT>,
    ) {
        if src_trees.peek_height() > dst_trees.peek_height() {
            src_trees.open();
        } else {
            dst_trees.open();
        }
    }

    fn matchh_to_be_filtered<MM: MultiMappingStore<Src = M::Src, Dst = M::Dst>>(
        &self,
        multi_mappings: &mut MM,
    ) {
        let mut src_trees =
            PriorityTreeList::new(self.node_store, &self.src_arena, self.src_arena.root());
        let mut dst_trees =
            PriorityTreeList::new(self.node_store, &self.dst_arena, self.dst_arena.root());
        // let mut aaa = 0;
        while src_trees.peek_height() != -1 && dst_trees.peek_height() != -1 {
            // aaa += 1;
            // println!("multi_mappings={}", multi_mappings.len());
            while src_trees.peek_height() != dst_trees.peek_height() {
                self.pop_larger(&mut src_trees, &mut dst_trees);
                // if src_trees.peek_height() == -1 || dst_trees.peek_height() == -1 {
                //     break;
                // }
            }

            let current_height_src_trees = src_trees.pop().unwrap();
            let current_height_dst_trees = dst_trees.pop().unwrap();

            let mut marks_for_src_trees = bitvec::bitbox![0;current_height_src_trees.len()];
            let mut marks_for_dst_trees = bitvec::bitbox![0;current_height_dst_trees.len()];
            // println!(
            //     "{aaa} marks={},{}",
            //     marks_for_src_trees.len(),
            //     marks_for_dst_trees.len()
            // );

            for i in 0..current_height_src_trees.len() {
                for j in 0..current_height_dst_trees.len() {
                    let src = current_height_src_trees[i];
                    let dst = current_height_dst_trees[j];
                    if self.isomorphic(&src, &dst) {
                        // println!("isomorphic={},{}", i, j);
                        // println!(
                        //     "children={},{}",
                        //     self.node_store
                        //         .resolve(&self.src_arena.original(&src))
                        //         .try_get_children()
                        //         .map_or(0, |x| x.len()),
                        //     self.node_store
                        //         .resolve(&self.dst_arena.original(&dst))
                        //         .try_get_children()
                        //         .map_or(0, |x| x.len())
                        // );
                        // println!("id={:?},{:?}", src, dst);
                        multi_mappings.link(src, dst);
                        marks_for_src_trees.set(i, true);
                        marks_for_dst_trees.set(j, true);
                    }
                }
            }
            // println!("multi_mappings'={}", multi_mappings.len());
            for i in 0..marks_for_src_trees.len() {
                if marks_for_src_trees[i] == false {
                    src_trees.open_tree(&current_height_src_trees[i]);
                }
            }
            // println!("multi_mappings''={}", multi_mappings.len());
            for j in 0..marks_for_dst_trees.len() {
                if marks_for_dst_trees[j] == false {
                    dst_trees.open_tree(&current_height_dst_trees[j]);
                }
            }

            src_trees.update_height();
            dst_trees.update_height();
        }
    }

    #[allow(unused)] // alternative
    fn similarity(&self, src: &M::Src, dst: &M::Dst) -> f64 {
        let p_src = self.src_arena.parent(src).unwrap();
        let p_dst = self.dst_arena.parent(dst).unwrap();
        let jaccard = similarity_metrics::jaccard_similarity(
            &self.src_arena.descendants(self.node_store, &p_src),
            &self.dst_arena.descendants(self.node_store, &p_dst),
            &self.mappings,
        );
        let pos_src = if self.src_arena.has_parent(src) {
            zero()
        } else {
            self.src_arena.position_in_parent(src).unwrap()
        };
        let pos_dst = if self.dst_arena.has_parent(dst) {
            zero()
        } else {
            self.dst_arena.position_in_parent(dst).unwrap()
        };

        let max_src_pos = if self.src_arena.has_parent(src) {
            one()
        } else {
            self.node_store
                .resolve(&self.src_arena.original(&p_src))
                .child_count()
        };
        let max_dst_pos = if self.dst_arena.has_parent(dst) {
            one()
        } else {
            self.node_store
                .resolve(&self.dst_arena.original(&p_dst))
                .child_count()
        };
        let max_pos_diff = std::cmp::max(max_src_pos, max_dst_pos);
        let pos: f64 = 1.0_f64
            - ((Ord::max(pos_src, pos_dst) - Ord::min(pos_dst, pos_src))
                .to_f64()
                .unwrap()
                / max_pos_diff.to_f64().unwrap());
        let po: f64 = 1.0_f64
            - ((Ord::max(src.to_usize().unwrap(), dst.to_usize().unwrap())
                - Ord::min(dst.to_usize().unwrap(), src.to_usize().unwrap()))
            .to_f64()
            .unwrap()
                / self.get_max_tree_size().to_f64().unwrap());
        100. * jaccard + 10. * pos + po
    }

    fn get_max_tree_size(&self) -> usize {
        Ord::max(self.src_arena.len(), self.dst_arena.len())
    }

    pub(crate) fn isomorphic(&self, src: &M::Src, dst: &M::Dst) -> bool {
        let src = self.src_arena.original(src);
        let dst = self.dst_arena.original(dst);

        self.isomorphic_aux::<true>(&src, &dst)
    }

    /// if H then test the hash otherwise do not test it,
    /// considering hash colisions testing it should only be useful once.
    pub(crate) fn isomorphic_aux<const H: bool>(&self, src: &T::TreeId, dst: &T::TreeId) -> bool {
        if src == dst {
            return true;
        }
        let src = self.node_store.resolve(src);
        let src_h = if H {
            Some(src.hash(&mut &T::HK::label()))
        } else {
            None
        };
        let src_t = src.get_type();
        let src_l = if src.has_label() {
            Some(src.get_label())
        } else {
            None
        };
        let src_c: Option<Vec<_>> = src.children().map(|x| x.iter_children().collect());

        let dst = self.node_store.resolve(dst);

        if let Some(src_h) = src_h {
            let dst_h = dst.hash(&mut &T::HK::label());
            if src_h != dst_h {
                return false;
            }
        }
        let dst_t = dst.get_type();
        if src_t != dst_t {
            return false;
        }
        if dst.has_label() {
            if src_l.is_none() || src_l.unwrap() != dst.get_label() {
                return false;
            }
        };

        let dst_c: Option<Vec<_>> = dst.children().map(|x| x.iter_children().collect());

        match (src_c, dst_c) {
            (None, None) => true,
            (Some(src_c), Some(dst_c)) => {
                if src_c.len() != dst_c.len() {
                    false
                } else {
                    for (src, dst) in src_c.iter().zip(dst_c.iter()) {
                        if !self.isomorphic_aux::<false>(src, dst) {
                            return false;
                        }
                    }
                    true
                }
            }
            _ => false,
        }
    }
}

struct PriorityTreeList<'a, 'b, D, IdD, T: Tree, S, const MIN_HEIGHT: usize> {
    trees: Vec<Option<Vec<IdD>>>,

    store: &'a S,
    arena: &'b D,

    max_height: usize,

    current_idx: isize,

    phantom: PhantomData<*const T>,
}

impl<
        'a,
        'b,
        D: DecompressedTreeStore<'a, T, IdD>,
        IdD: PrimInt,
        T: Tree,
        S: 'a + NodeStore<T::TreeId, R<'a> = T>,
        const MIN_HEIGHT: usize,
    > PriorityTreeList<'a, 'b, D, IdD, T, S, MIN_HEIGHT>
where
    T::TreeId: Clone,
{
    pub(super) fn new(store: &'a S, arena: &'b D, tree: IdD) -> Self {
        let h = height(store, &arena.original(&tree)); // TODO subtree opti, use metadata
        let list_size = if h >= MIN_HEIGHT {
            h + 1 - MIN_HEIGHT
        } else {
            0
        };
        let mut r = Self {
            trees: vec![None; list_size],
            store,
            arena,
            max_height: h,
            current_idx: if list_size == 0 { -1 } else { 0 },
            phantom: PhantomData,
        };
        r.add_tree_aux(tree, h);
        r
    }

    fn idx(&self, height: usize) -> usize {
        self.max_height - height
    }

    fn height(&self, idx: usize) -> usize {
        self.max_height - idx
    }

    fn add_tree(&mut self, tree: IdD) {
        let h = height(self.store, &self.arena.original(&tree)) as usize;
        self.add_tree_aux(tree, h)
    }

    fn add_tree_aux(&mut self, tree: IdD, h: usize) {
        if h >= MIN_HEIGHT {
            let idx = self.idx(h);
            if self.trees[idx].is_none() {
                self.trees[idx] = Some(vec![]);
            };
            self.trees[idx].as_mut().unwrap().push(tree);
        }
    }

    pub(super) fn open(&mut self) -> Option<Vec<IdD>> {
        if let Some(pop) = self.pop() {
            for tree in &pop {
                self.open_tree(tree);
            }
            self.update_height();
            Some(pop)
        } else {
            None
        }
    }

    pub(super) fn pop(&mut self) -> Option<Vec<IdD>> {
        if self.current_idx < 0 {
            None
        } else {
            self.trees[self.current_idx as usize].take()
        }
    }

    pub(super) fn open_tree(&mut self, tree: &IdD) {
        for c in self.arena.children(self.store, tree) {
            self.add_tree(c);
        }
    }

    pub(super) fn peek_height(&self) -> isize {
        if self.current_idx == -1 {
            -1
        } else {
            self.height(self.current_idx as usize) as isize
        }
    }

    pub(super) fn update_height(&mut self) {
        self.current_idx = -1;
        for i in 0..self.trees.len() {
            if self.trees[i].is_some() {
                self.current_idx = i as isize;
                break;
            }
        }
    }
}
