use std::hash::Hash;
use std::{fmt::Debug, marker::PhantomData};

use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent,
    LazyDecompressedTreeStore, Shallow,
};
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::Mapper;
use crate::matchers::{mapping_store::MultiMappingStore, similarity_metrics};
use crate::utils::sequence_algorithms::longest_common_subsequence;
use hyper_ast::compat::HashMap;
use hyper_ast::types::{
    DecompressedSubtree, HashKind, HyperAST, IterableChildren, Labeled, NodeStore, Stored, Tree,
    Typed, WithChildren, WithHashs, WithStats,
};
use logging_timer::time;
use num_traits::{PrimInt, ToPrimitive};

pub struct LazyGreedySubtreeMatcher<'a, HAST, Dsrc, Ddst, M, const MIN_HEIGHT: usize = 1> {
    internal: Mapper<'a, HAST, Dsrc, Ddst, M>,
}

impl<
        'a,
        Dsrc: 'a
            + DecompressedWithParent<'a, HAST::T, Dsrc::IdD>
            + ContiguousDescendants<'a, HAST::T, Dsrc::IdD, M::Src>
            + DecompressedSubtree<'a, HAST::T>
            + LazyDecompressedTreeStore<'a, HAST::T, M::Src>,
        Ddst: 'a
            + DecompressedWithParent<'a, HAST::T, Ddst::IdD>
            + ContiguousDescendants<'a, HAST::T, Ddst::IdD, M::Dst>
            + DecompressedSubtree<'a, HAST::T>
            + LazyDecompressedTreeStore<'a, HAST::T, M::Dst>,
        HAST: HyperAST<'a>,
        // T: Tree + WithHashs + WithStats,
        // S: 'a + NodeStore<T::TreeId, R<'a> = T>,
        M: MonoMappingStore,
        const MIN_HEIGHT: usize, // = 2
    > LazyGreedySubtreeMatcher<'a, HAST, Dsrc, Ddst, M, MIN_HEIGHT>
where
    HAST::T: Tree + WithHashs + WithStats,
    HAST::IdN: Clone + Eq,
    HAST::Label: Clone + Eq,
    Dsrc::IdD: Debug + Hash + Eq + PrimInt,
    Ddst::IdD: Debug + Hash + Eq + PrimInt,
    M::Src: 'a + PrimInt + Debug + Hash,
    M::Dst: 'a + PrimInt + Debug + Hash,
{
    pub fn match_it<MM>(
        mapping: crate::matchers::Mapper<'a, HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<'a, HAST, Dsrc, Ddst, M>
    where
        Self: 'a,
        MM: MultiMappingStore<Src = Dsrc::IdD, Dst = Ddst::IdD>,
    {
        let mut matcher = Self { internal: mapping };
        matcher.internal.mapping.mappings.topit(
            matcher.internal.mapping.src_arena.len(),
            matcher.internal.mapping.dst_arena.len(),
        );
        Self::execute::<MM>(&mut matcher);
        matcher.internal
    }

    // pub fn matchh<MM: MultiMappingStore<Src = Dsrc::IdD, Dst = Ddst::IdD>>(
    //     node_store: &'a S,
    //     src: &'a T::TreeId,
    //     dst: &'a T::TreeId,
    //     mappings: M,
    // ) -> LazyGreedySubtreeMatcher<'a, Dsrc, Ddst, T, S, M, MIN_HEIGHT>
    // where
    //     Self: 'a,
    // {
    //     let src_arena = Dsrc::decompress(node_store, src);
    //     let dst_arena = Ddst::decompress(node_store, dst);
    //     let mut matcher = Self::new(node_store, src_arena, dst_arena, mappings);
    //     Self::execute::<MM>(&mut matcher);
    //     matcher
    // }

    // [2022-12-19T17:00:02.948Z WARN] considering_stats(), Elapsed=383.306235ms
    // [2022-12-19T17:00:03.334Z WARN] considering_stats(), Elapsed=385.759276ms
    // [2022-12-19T17:00:03.725Z WARN] matchh_to_be_filtered(), Elapsed=388.976068ms
    // [2022-12-19T17:00:03.732Z WARN] filter_mappings(), Elapsed=7.145745ms

    // with WithStats to get height through metadata
    // [2022-12-19T17:11:48.121Z WARN] matchh_to_be_filtered(), Elapsed=16.639973ms

    pub fn execute<MM: MultiMappingStore<Src = Dsrc::IdD, Dst = Ddst::IdD>>(&mut self) {
        let mm: MM = self.compute_multi_mapping();
        self.filter_mappings(&mm);
    }

    pub fn compute_multi_mapping<MM: MultiMappingStore<Src = Dsrc::IdD, Dst = Ddst::IdD>>(
        &mut self,
    ) -> MM {
        let mut mm: MM = Default::default();
        mm.topit(self.internal.src_arena.len(), self.internal.dst_arena.len());
        self.internal.compute_multimapping::<_, MIN_HEIGHT>(&mut mm);
        mm
    }
}

impl<
        'a,
        Dsrc: 'a
            + DecompressedWithParent<'a, HAST::T, Dsrc::IdD>
            + ContiguousDescendants<'a, HAST::T, Dsrc::IdD, M::Src>
            + DecompressedSubtree<'a, HAST::T>
            + LazyDecompressedTreeStore<'a, HAST::T, M::Src>,
        Ddst: 'a
            + DecompressedWithParent<'a, HAST::T, Ddst::IdD>
            + ContiguousDescendants<'a, HAST::T, Ddst::IdD, M::Dst>
            + DecompressedSubtree<'a, HAST::T>
            + LazyDecompressedTreeStore<'a, HAST::T, M::Dst>,
        HAST: HyperAST<'a>,
        // T: Tree + WithHashs + WithStats,
        // S: 'a + NodeStore<T::TreeId, R<'a> = T>,
        M: MonoMappingStore,
        const MIN_HEIGHT: usize, // = 2
    > LazyGreedySubtreeMatcher<'a, HAST, Dsrc, Ddst, M, MIN_HEIGHT>
where
    HAST::T: Tree + WithHashs + WithStats,
    HAST::IdN: Clone,
    HAST::Label: Clone + Eq,
    Dsrc::IdD: Debug + Hash + Eq + PrimInt,
    Ddst::IdD: Debug + Hash + Eq + PrimInt,
    M::Src: 'a + PrimInt + Debug + Hash,
    M::Dst: 'a + PrimInt + Debug + Hash,
{
    //         T: Tree + WithHashs + WithStats,
    //         S: 'a + NodeStore<T::TreeId, R<'a> = T>,
    //         M: MonoMappingStore,
    //         const MIN_HEIGHT: usize, // = 2
    //     > LazyGreedySubtreeMatcher<'a, Dsrc, Ddst, T, S, M, MIN_HEIGHT>
    // where
    //     T::TreeId: Clone,
    //     T::Label: Clone,
    //     Dsrc::IdD: Debug + Hash + Eq + Copy,
    //     Ddst::IdD: Debug + Hash + Eq + Copy,
    //     M::Src: 'a + PrimInt + Debug + Hash,
    //     M::Dst: 'a + PrimInt + Debug + Hash,
    // {
    // pub fn new(
    //     node_store: &HAST::NS,
    //     src_arena: Dsrc,
    //     dst_arena: Ddst,
    //     mappings: M,
    // ) -> Self {
    //     let mut matcher = LazyGreedySubtreeMatcher {
    //         internal: Mapper {
    //             node_store,
    //             src_arena,
    //             dst_arena,
    //             mappings,
    //             phantom: PhantomData,
    //         },
    //     };
    //     matcher.internal.mappings.topit(
    //         matcher.internal.src_arena.len() + 1,
    //         matcher.internal.dst_arena.len() + 1,
    //     );
    //     matcher
    // }

    #[time("warn")]
    pub fn filter_mappings<MM: MultiMappingStore<Src = Dsrc::IdD, Dst = Ddst::IdD>>(
        &mut self,
        multi_mappings: &MM,
    ) {
        // Select unique mappings first and extract ambiguous mappings.
        let mut ambiguous_list: Vec<(Dsrc::IdD, Ddst::IdD)> = vec![];
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

            if !(ignored[src.shallow().to_usize().unwrap()] || is_mapping_unique) {
                let adsts = multi_mappings.get_dsts(&src);
                let asrcs = multi_mappings.get_srcs(&multi_mappings.get_dsts(&src)[0]);
                for asrc in asrcs {
                    for adst in adsts {
                        ambiguous_list.push((*asrc, *adst));
                    }
                }
                asrcs
                    .iter()
                    .for_each(|x| ignored.set(x.shallow().to_usize().unwrap(), true))
            }
        }

        let mapping_list: Vec<_> = {
            self.sort(&mut ambiguous_list);
            ambiguous_list
        };

        // Select the best ambiguous mappings
        for (src, dst) in mapping_list {
            let src_i = src.shallow().to_usize().unwrap();
            let dst_i = dst.shallow().to_usize().unwrap();
            if !(src_ignored[src_i] || dst_ignored[dst_i]) {
                self.internal.add_mapping_recursively(&src, &dst);
                src_ignored.set(src_i, true);
                self.internal
                    .src_arena
                    .descendants(self.internal.hyperast.node_store(), &src)
                    .iter()
                    .for_each(|src| src_ignored.set(src.to_usize().unwrap(), true));
                dst_ignored.set(dst_i, true);
                self.internal
                    .dst_arena
                    .descendants(self.internal.hyperast.node_store(), &dst)
                    .iter()
                    .for_each(|dst| dst_ignored.set(dst.to_usize().unwrap(), true));
            }
        }
    }

    fn sort(&self, ambiguous_mappings: &mut Vec<(Dsrc::IdD, Ddst::IdD)>) {
        let mut sib_sim = HashMap::<(Dsrc::IdD, Ddst::IdD), f64>::default();
        let mut psib_sim = HashMap::<(Dsrc::IdD, Ddst::IdD), f64>::default();
        let mut p_in_p_sim = HashMap::<(Dsrc::IdD, Ddst::IdD), f64>::default();
        dbg!(&ambiguous_mappings.len());
        ambiguous_mappings.sort_by(|a, b| {
            let cached_coef_sib = |l: &(Dsrc::IdD, Ddst::IdD)| {
                sib_sim
                    .entry(*l)
                    .or_insert_with(|| self.coef_sib(&l))
                    .clone()
            };
            let cached_coef_parent = |l: &(Dsrc::IdD, Ddst::IdD)| {
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
                    |l: &(Dsrc::IdD, Ddst::IdD)| {
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

    fn coef_sib(&self, l: &(Dsrc::IdD, Ddst::IdD)) -> f64 {
        let (p_src, p_dst) = self.parents(l);
        similarity_metrics::SimilarityMeasure::range(
            &self.internal.src_arena.descendants_range(&p_src), //descendants
            &self.internal.dst_arena.descendants_range(&p_dst),
            &self.internal.mappings,
        )
        .dice()
    }

    fn parents(&self, l: &(Dsrc::IdD, Ddst::IdD)) -> (Dsrc::IdD, Ddst::IdD) {
        let p_src = self.internal.src_arena.parent(&l.0).unwrap();
        let p_dst = self.internal.dst_arena.parent(&l.1).unwrap();
        (p_src, p_dst)
    }

    fn coef_parent(&self, l: &(Dsrc::IdD, Ddst::IdD)) -> f64 {
        let s1: Vec<_> = Dsrc::parents(&self.internal.src_arena, l.0).collect();
        let s2: Vec<_> = Ddst::parents(&self.internal.dst_arena, l.1).collect();
        let common = longest_common_subsequence::<_, _, usize, _>(&s1, &s2, |a, b| {
            let (t, l) = {
                let o = self.internal.src_arena.original(a);
                let n = self.internal.hyperast.node_store().resolve(&o);
                (n.get_type(), n.try_get_label().cloned())
            };
            let o = self.internal.dst_arena.original(b);
            let n = self.internal.hyperast.node_store().resolve(&o);
            t == n.get_type() && l.as_ref() == n.try_get_label()
        });
        (2 * common.len()).to_f64().unwrap() / (s1.len() + s2.len()).to_f64().unwrap()
    }

    fn coef_pos_in_parent(&self, l: &(Dsrc::IdD, Ddst::IdD)) -> f64 {
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
                            .children(self.internal.hyperast.node_store(), &p)
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
                            .children(self.internal.hyperast.node_store(), &p)
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

    fn same_parents(&self, alink: &(Dsrc::IdD, Ddst::IdD), blink: &(Dsrc::IdD, Ddst::IdD)) -> bool {
        let ap = self.mapping_parents(&alink);
        let bp = self.mapping_parents(&blink);
        ap.0 == bp.0 && ap.1 == bp.1
    }

    fn mapping_parents(
        &self,
        l: &(Dsrc::IdD, Ddst::IdD),
    ) -> (Option<Dsrc::IdD>, Option<Ddst::IdD>) {
        (
            self.internal.src_arena.parent(&l.0),
            self.internal.dst_arena.parent(&l.1),
        )
    }

    fn compare_delta_pos(
        &self,
        alink: &(Dsrc::IdD, Ddst::IdD),
        blink: &(Dsrc::IdD, Ddst::IdD),
    ) -> std::cmp::Ordering {
        return (alink
            .0
            .shallow()
            .to_usize()
            .unwrap()
            .abs_diff(alink.1.shallow().to_usize().unwrap()))
        .cmp(
            &blink
                .0
                .shallow()
                .to_usize()
                .unwrap()
                .abs_diff(blink.1.shallow().to_usize().unwrap()),
        );
    }
}

// impl<
//         'a,
//         HAST: HyperAST<'a>,
//         Dsrc: 'a
//             + DecompressedWithParent<'a, HAST::T, Dsrc::IdD>
//             + LazyDecompressedTreeStore<'a, HAST::T, M::Src>,
//         Ddst: 'a
//             + DecompressedWithParent<'a, HAST::T, Ddst::IdD>
//             + LazyDecompressedTreeStore<'a, HAST::T, M::Dst>,
//         M: MonoMappingStore,
//     > crate::matchers::Mapper<'a, HAST, Dsrc, Ddst, M>
// where
//     HAST::T: 'a + WithStats,
//     M::Src: Debug + Copy,
//     M::Dst: Debug + Copy,
// {
//     pub(crate) fn add_mapping_recursively(&mut self, src: &Dsrc::IdD, dst: &Ddst::IdD) {
//         self.mappings
//             .link(src.shallow().clone(), dst.shallow().clone());
//         // WARN check if it works well
//         self.src_arena
//             .descendants(self.node_store(), src)
//             .iter()
//             .zip(self.dst_arena.descendants(self.node_store(), dst).iter())
//             .for_each(|(src, dst)| self.mappings.link(*src, *dst));
//     }
// }
impl<'a, HAST: HyperAST<'a>, Dsrc, Ddst, M: MonoMappingStore>
    crate::matchers::Mapper<'a, HAST, Dsrc, Ddst, M>
where
    M::Src: Debug + Copy,
    M::Dst: Debug + Copy,
{
    pub(crate) fn add_mapping_recursively<Src, Dst>(&mut self, src: &Src, dst: &Dst)
    where
        Src: Shallow<M::Src>,
        Dst: Shallow<M::Dst>,
        Dsrc: DecompressedWithParent<'a, HAST::T, Src>
            + DecompressedTreeStore<'a, HAST::T, Src, M::Src>,
        Ddst: DecompressedWithParent<'a, HAST::T, Dst>
            + DecompressedTreeStore<'a, HAST::T, Dst, M::Dst>,
    {
        self.mappings
            .link(src.shallow().clone(), dst.shallow().clone());
        // WARN check if it works well
        self.src_arena
            .descendants(self.hyperast.node_store(), src)
            .iter()
            .zip(
                self.dst_arena
                    .descendants(self.hyperast.node_store(), dst)
                    .iter(),
            )
            .for_each(|(src, dst)| self.mappings.link(*src, *dst));
    }
}

impl<
        'a,
        HAST: 'a + HyperAST<'a>,
        Dsrc: 'a
            + DecompressedWithParent<'a, HAST::T, Dsrc::IdD>
            + LazyDecompressedTreeStore<'a, HAST::T, M::Src>,
        Ddst: 'a
            + DecompressedWithParent<'a, HAST::T, Ddst::IdD>
            + LazyDecompressedTreeStore<'a, HAST::T, M::Dst>,
        M: MonoMappingStore,
    > crate::matchers::Mapper<'a, HAST, Dsrc, Ddst, M>
where
    HAST::T: 'a + Tree + WithHashs + WithStats,
    HAST::IdN: Clone + Eq,
    HAST::Label: Eq,
    Dsrc::IdD: Clone,
    Ddst::IdD: Clone,
    M::Src: Debug + Copy,
    M::Dst: Debug + Copy,
{
    #[time("warn")]
    fn compute_multimapping<
        MM: MultiMappingStore<Src = Dsrc::IdD, Dst = Ddst::IdD>,
        const MIN_HEIGHT: usize,
    >(
        &mut self,
        multi_mappings: &mut MM,
    ) {
        let now = std::time::Instant::now();
        let mut src_trees = PriorityTreeList::<
            'a,
            '_,
            Dsrc,
            M::Src,
            Dsrc::IdD,
            HAST::T,
            HAST::NS,
            MIN_HEIGHT,
        >::new(
            self.hyperast.node_store(),
            self.mapping.src_arena.starter(),
            &mut self.mapping.src_arena,
        );
        let mut dst_trees = PriorityTreeList::<
            'a,
            '_,
            Ddst,
            M::Dst,
            Ddst::IdD,
            HAST::T,
            HAST::NS,
            MIN_HEIGHT,
        >::new(
            self.hyperast.node_store(),
            self.mapping.dst_arena.starter(),
            &mut self.mapping.dst_arena,
        );
        let match_init_t = now.elapsed().as_secs_f64();
        dbg!(match_init_t);
        while src_trees.peek_height() != -1 && dst_trees.peek_height() != -1 {
            // println!("multi_mappings={}", multi_mappings.len());
            while src_trees.peek_height() != dst_trees.peek_height() {
                // open larger
                if src_trees.peek_height() > dst_trees.peek_height() {
                    src_trees.open();
                } else {
                    dst_trees.open();
                }
                // TODO uncomment ?
                // if src_trees.peek_height() == -1 || dst_trees.peek_height() == -1 {
                //     break;
                // }
            }

            let current_height_src_trees = src_trees.pop().unwrap();
            let current_height_dst_trees = dst_trees.pop().unwrap();

            let mut marks_for_src_trees = bitvec::bitbox![0;current_height_src_trees.len()];
            let mut marks_for_dst_trees = bitvec::bitbox![0;current_height_dst_trees.len()];

            for i in 0..current_height_src_trees.len() {
                for j in 0..current_height_dst_trees.len() {
                    let src = current_height_src_trees[i].clone();
                    let dst = current_height_dst_trees[j].clone();
                    let is_iso = {
                        let src = src_trees.arena.original(&src);
                        let dst = dst_trees.arena.original(&dst);
                        Self::isomorphic_aux::<true>(self.hyperast.node_store(), &src, &dst)
                    };
                    if is_iso {
                        multi_mappings.link(src, dst);
                        marks_for_src_trees.set(i, true);
                        marks_for_dst_trees.set(j, true);
                    }
                }
            }
            for i in 0..marks_for_src_trees.len() {
                if marks_for_src_trees[i] == false {
                    src_trees.open_tree(&current_height_src_trees[i]);
                }
            }
            for j in 0..marks_for_dst_trees.len() {
                if marks_for_dst_trees[j] == false {
                    dst_trees.open_tree(&current_height_dst_trees[j]);
                }
            }

            src_trees.update_height();
            dst_trees.update_height();
        }
    }

    /// if H then test the hash otherwise do not test it,
    /// considering hash colisions testing it should only be useful once.
    pub(crate) fn isomorphic_aux<const H: bool>(
        node_store: &'a HAST::NS,
        src: &<HAST::T as Stored>::TreeId,
        dst: &<HAST::T as Stored>::TreeId,
    ) -> bool {
        if src == dst {
            return true;
        }
        let src = node_store.resolve(src);
        let src_h = if H {
            Some(src.hash(&mut &<HAST::T as WithHashs>::HK::label()))
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

        let dst = node_store.resolve(dst);

        if let Some(src_h) = src_h {
            let dst_h = dst.hash(&mut &<HAST::T as WithHashs>::HK::label());
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
                        if !Self::isomorphic_aux::<false>(node_store, src, dst) {
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

struct PriorityTreeList<'a, 'b, D, IdS, IdD, T: Tree, S, const MIN_HEIGHT: usize> {
    trees: Vec<Option<Vec<IdD>>>,

    store: &'a S,
    arena: &'b mut D,

    max_height: usize,

    current_idx: isize,

    phantom: PhantomData<*const (T, IdS)>,
}

impl<
        'a,
        'b,
        D: LazyDecompressedTreeStore<'a, T, IdD>,
        IdD,
        T: Tree + WithStats,
        S: 'a + NodeStore<T::TreeId, R<'a> = T>,
        const MIN_HEIGHT: usize,
    > PriorityTreeList<'a, 'b, D, IdD, D::IdD, T, S, MIN_HEIGHT>
where
    T::TreeId: Clone,
    D::IdD: Clone,
{
    pub(super) fn new(store: &'a S, tree: D::IdD, arena: &'b mut D) -> Self {
        let h = store.resolve(&arena.original(&tree)).height() - 1;
        let list_size = if h >= MIN_HEIGHT {
            h + 1 - MIN_HEIGHT
        } else {
            0
        };
        let mut r = Self {
            trees: vec![Default::default(); list_size],
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

    fn add_tree(&mut self, tree: D::IdD) {
        let h = self.store.resolve(&self.arena.original(&tree)).height() - 1;
        self.add_tree_aux(tree, h)
    }

    fn add_tree_aux(&mut self, tree: D::IdD, h: usize) {
        if h >= MIN_HEIGHT {
            let idx = self.idx(h);
            if self.trees[idx].is_none() {
                self.trees[idx] = Some(vec![]);
            };
            self.trees[idx].as_mut().unwrap().push(tree);
        }
    }

    pub(super) fn open(&mut self) -> Option<Vec<D::IdD>> {
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

    pub(super) fn pop(&mut self) -> Option<Vec<D::IdD>> {
        if self.current_idx < 0 {
            None
        } else {
            self.trees[self.current_idx as usize].take()
        }
    }

    pub(super) fn open_tree(&mut self, tree: &D::IdD) {
        for c in self.arena.decompress_children(self.store, tree) {
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
