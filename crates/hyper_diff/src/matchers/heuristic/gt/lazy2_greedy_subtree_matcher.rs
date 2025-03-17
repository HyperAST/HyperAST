use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, LazyDecompressed,
    LazyDecompressedTreeStore, Shallow,
};
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::Mapper;
use crate::matchers::{mapping_store::MultiMappingStore, similarity_metrics};
use crate::utils::sequence_algorithms::longest_common_subsequence;
use hyperast::compat::HashMap;
use hyperast::types::{
    Childrn, HashKind, HyperAST, Labeled, NodeId, NodeStore, Tree, WithChildren, WithHashs,
    WithStats,
};
use hyperast::PrimInt;
use num_traits::ToPrimitive;
use std::fmt::Debug;
use std::hash::Hash;

pub struct LazyGreedySubtreeMatcher<HAST, Dsrc, Ddst, M, const MIN_HEIGHT: usize = 1> {
    internal: Mapper<HAST, Dsrc, Ddst, M>,
}

impl<
        Dsrc: LazyDecompressed<M::Src>,
        Ddst: LazyDecompressed<M::Dst>,
        HAST: HyperAST + Copy,
        M: MonoMappingStore,
        const MIN_HEIGHT: usize, // = 2
    > LazyGreedySubtreeMatcher<HAST, Dsrc, Ddst, M, MIN_HEIGHT>
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs + WithStats,
    HAST::IdN: Clone + Eq,
    HAST::Label: Clone + Eq,
    Dsrc::IdD: PrimInt + Hash,
    Ddst::IdD: PrimInt + Hash,
    M::Src: PrimInt + Hash,
    M::Dst: PrimInt + Hash,
    Dsrc: DecompressedWithParent<HAST, Dsrc::IdD>
        + ContiguousDescendants<HAST, Dsrc::IdD, M::Src>
        + LazyDecompressedTreeStore<HAST, M::Src>,
    Ddst: DecompressedWithParent<HAST, Ddst::IdD>
        + ContiguousDescendants<HAST, Ddst::IdD, M::Dst>
        + LazyDecompressedTreeStore<HAST, M::Dst>,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    pub fn match_it<MM>(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
    where
        MM: MultiMappingStore<Src = Dsrc::IdD, Dst = Ddst::IdD> + Default,
    {
        let mut matcher = Self { internal: mapping };
        matcher.internal.mapping.mappings.topit(
            matcher.internal.mapping.src_arena.len(),
            matcher.internal.mapping.dst_arena.len(),
        );
        Self::execute::<MM>(&mut matcher);
        matcher.internal
    }

    // [2022-12-19T17:00:02.948Z WARN] considering_stats(), Elapsed=383.306235ms
    // [2022-12-19T17:00:03.334Z WARN] considering_stats(), Elapsed=385.759276ms
    // [2022-12-19T17:00:03.725Z WARN] matchh_to_be_filtered(), Elapsed=388.976068ms
    // [2022-12-19T17:00:03.732Z WARN] filter_mappings(), Elapsed=7.145745ms

    // with WithStats to get height through metadata
    // [2022-12-19T17:11:48.121Z WARN] matchh_to_be_filtered(), Elapsed=16.639973ms

    pub fn execute<MM: MultiMappingStore<Src = Dsrc::IdD, Dst = Ddst::IdD> + Default>(&mut self) {
        let mm: MM = Self::compute_multi_mapping(&mut self.internal);
        Self::filter_mappings(&mut self.internal, &mm);
    }

    pub fn compute_multi_mapping<
        MM: MultiMappingStore<Src = Dsrc::IdD, Dst = Ddst::IdD> + Default,
    >(
        internal: &mut Mapper<HAST, Dsrc, Ddst, M>,
    ) -> MM {
        let mut mm: MM = Default::default();
        mm.topit(internal.src_arena.len(), internal.dst_arena.len());
        Mapper::<HAST, Dsrc, Ddst, M>::compute_multimapping::<_, MIN_HEIGHT>(
            internal.hyperast,
            &mut internal.mapping.src_arena,
            &mut internal.mapping.dst_arena,
            &mut mm,
        );
        mm
    }
}

impl<
        Dsrc: LazyDecompressed<M::Src>,
        Ddst: LazyDecompressed<M::Dst>,
        HAST: HyperAST + Copy,
        M: MonoMappingStore,
        const MIN_HEIGHT: usize, // = 2
    > LazyGreedySubtreeMatcher<HAST, Dsrc, Ddst, M, MIN_HEIGHT>
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs + WithStats,
    HAST::IdN: Clone,
    HAST::Label: Clone + Eq,
    Dsrc::IdD: PrimInt + Hash,
    Ddst::IdD: PrimInt + Hash,
    M::Src: PrimInt + Hash,
    M::Dst: PrimInt + Hash,
    Dsrc: DecompressedWithParent<HAST, Dsrc::IdD>
        + ContiguousDescendants<HAST, Dsrc::IdD, M::Src>
        + LazyDecompressedTreeStore<HAST, M::Src>,
    Ddst: DecompressedWithParent<HAST, Ddst::IdD>
        + ContiguousDescendants<HAST, Ddst::IdD, M::Dst>
        + LazyDecompressedTreeStore<HAST, M::Dst>,
{
    // #[time("warn")]
    pub fn filter_mappings<MM: MultiMappingStore<Src = Dsrc::IdD, Dst = Ddst::IdD>>(
        mapper: &mut Mapper<HAST, Dsrc, Ddst, M>,
        multi_mappings: &MM,
    ) {
        // Select unique mappings first and extract ambiguous mappings.
        let mut ambiguous_list: Vec<(Dsrc::IdD, Ddst::IdD)> = vec![];
        let mut ignored = bitvec::bitbox![0;mapper.src_arena.len()];
        let mut src_ignored = bitvec::bitbox![0;mapper.src_arena.len()];
        let mut dst_ignored = bitvec::bitbox![0;mapper.dst_arena.len()];
        for src in multi_mappings.all_mapped_srcs() {
            let mut is_mapping_unique = false;
            if multi_mappings.is_src_unique(&src) {
                let dst = multi_mappings.get_dsts(&src)[0];
                if multi_mappings.is_dst_unique(&dst) {
                    mapper.add_mapping_recursively(&src, &dst); // TODO subtree opti, do not do explicitly
                    is_mapping_unique = true;
                }
            }

            if !(ignored[src.shallow().to_usize().unwrap()] || is_mapping_unique) {
                let adsts = multi_mappings.get_dsts(&src);
                let asrcs = multi_mappings.get_srcs(&multi_mappings.get_dsts(&src)[0]);
                for asrc in asrcs {
                    ignored.set(asrc.shallow().to_usize().unwrap(), true);
                    for adst in adsts {
                        ambiguous_list.push((*asrc, *adst));
                    }
                }
            }
        }

        let mapping_list: Vec<_> = {
            Self::sort(mapper, &mut ambiguous_list);
            ambiguous_list
        };

        // Select the best ambiguous mappings
        for (src, dst) in mapping_list {
            let src_i = src.shallow().to_usize().unwrap();
            let dst_i = dst.shallow().to_usize().unwrap();
            if !(src_ignored[src_i] || dst_ignored[dst_i]) {
                mapper.add_mapping_recursively(&src, &dst);
                src_ignored.set(src_i, true);
                mapper
                    .src_arena
                    .descendants(&src)
                    .iter()
                    .for_each(|src| src_ignored.set(src.to_usize().unwrap(), true));
                dst_ignored.set(dst_i, true);
                mapper
                    .dst_arena
                    .descendants(&dst)
                    .iter()
                    .for_each(|dst| dst_ignored.set(dst.to_usize().unwrap(), true));
            }
            // TODO return additional mappings
        }
    }

    fn sort(
        mapper: &Mapper<HAST, Dsrc, Ddst, M>,
        ambiguous_mappings: &mut Vec<(Dsrc::IdD, Ddst::IdD)>,
    ) {
        let mut sib_sim = HashMap::<(Dsrc::IdD, Ddst::IdD), f64>::default();
        let mut psib_sim = HashMap::<(Dsrc::IdD, Ddst::IdD), f64>::default();
        let mut p_in_p_sim = HashMap::<(Dsrc::IdD, Ddst::IdD), f64>::default();
        dbg!(&ambiguous_mappings.len());
        ambiguous_mappings.sort_by(|a, b| {
            let cached_coef_sib = |l: &(Dsrc::IdD, Ddst::IdD)| {
                sib_sim
                    .entry(*l)
                    .or_insert_with(|| Self::coef_sib(mapper, &l))
                    .clone()
            };
            let cached_coef_parent = |l: &(Dsrc::IdD, Ddst::IdD)| {
                psib_sim
                    .entry(*l)
                    .or_insert_with(|| Self::coef_parent(mapper, &l))
                    .clone()
            };
            let (alink, blink) = (a, b);
            if Self::same_parents(mapper, alink, blink) {
                std::cmp::Ordering::Equal
            } else {
                Self::cached_compare(cached_coef_sib, a, b)
                    .reverse()
                    .then_with(|| Self::cached_compare(cached_coef_parent, a, b).reverse())
            }
            .then_with(|| {
                Self::cached_compare(
                    |l: &(Dsrc::IdD, Ddst::IdD)| {
                        p_in_p_sim
                            .entry(*l)
                            .or_insert_with(|| Self::coef_pos_in_parent(mapper, &l))
                            .clone()
                    },
                    a,
                    b,
                )
            })
            .then_with(|| Self::compare_delta_pos(alink, blink))
        });
    }

    fn cached_compare<I, F: FnMut(&I) -> O, O: PartialOrd>(
        mut cached: F,
        a: &I,
        b: &I,
    ) -> std::cmp::Ordering {
        cached(a)
            .partial_cmp(&cached(b))
            .unwrap_or(std::cmp::Ordering::Equal)
    }

    fn coef_sib(mapper: &Mapper<HAST, Dsrc, Ddst, M>, l: &(Dsrc::IdD, Ddst::IdD)) -> f64 {
        let (p_src, p_dst) = Self::parents(mapper, l);
        similarity_metrics::SimilarityMeasure::range(
            &mapper.src_arena.descendants_range(&p_src), //descendants
            &mapper.dst_arena.descendants_range(&p_dst),
            &mapper.mappings,
        )
        .dice()
    }

    fn parents(
        mapper: &Mapper<HAST, Dsrc, Ddst, M>,
        l: &(Dsrc::IdD, Ddst::IdD),
    ) -> (Dsrc::IdD, Ddst::IdD) {
        let p_src = mapper.src_arena.parent(&l.0).unwrap();
        let p_dst = mapper.dst_arena.parent(&l.1).unwrap();
        (p_src, p_dst)
    }

    fn coef_parent(mapper: &Mapper<HAST, Dsrc, Ddst, M>, l: &(Dsrc::IdD, Ddst::IdD)) -> f64 {
        let s1: Vec<_> = Dsrc::parents(&mapper.src_arena, l.0).collect();
        let s2: Vec<_> = Ddst::parents(&mapper.dst_arena, l.1).collect();
        let common = longest_common_subsequence::<_, _, usize, _>(&s1, &s2, |a, b| {
            let (t, l) = {
                let o = mapper.src_arena.original(a);
                let n = mapper.hyperast.node_store().resolve(&o);
                let t = mapper.hyperast.resolve_type(&o);
                (t, n.try_get_label().cloned())
            };
            let o = mapper.dst_arena.original(b);
            let n = mapper.hyperast.node_store().resolve(&o);
            let t2 = mapper.hyperast.resolve_type(&o);
            t == t2 && l.as_ref() == n.try_get_label()
        });
        (2 * common.len()).to_f64().unwrap() / (s1.len() + s2.len()).to_f64().unwrap()
    }

    fn coef_pos_in_parent(mapper: &Mapper<HAST, Dsrc, Ddst, M>, l: &(Dsrc::IdD, Ddst::IdD)) -> f64 {
        let srcs = vec![l.0]
            .into_iter()
            .chain(mapper.src_arena.parents(l.0))
            .filter_map(|x| {
                mapper.src_arena.parent(&x).map(|p| {
                    mapper
                        .src_arena
                        .position_in_parent::<usize>(&x)
                        .unwrap()
                        .to_f64()
                        .unwrap()
                        / mapper.src_arena.children(&p).len().to_f64().unwrap()
                })
            });
        let dsts = vec![l.1]
            .into_iter()
            .chain(mapper.dst_arena.parents(l.1))
            .filter_map(|x| {
                mapper.dst_arena.parent(&x).map(|p| {
                    mapper
                        .dst_arena
                        .position_in_parent::<usize>(&x)
                        .unwrap()
                        .to_f64()
                        .unwrap()
                        / mapper.dst_arena.children(&p).len().to_f64().unwrap()
                })
            });
        srcs.zip(dsts)
            .map(|(src, dst)| (src - dst) * (src - dst))
            .sum::<f64>()
            .sqrt()
    }

    fn same_parents(
        mapper: &Mapper<HAST, Dsrc, Ddst, M>,
        alink: &(Dsrc::IdD, Ddst::IdD),
        blink: &(Dsrc::IdD, Ddst::IdD),
    ) -> bool {
        let ap = Self::mapping_parents(mapper, &alink);
        let bp = Self::mapping_parents(mapper, &blink);
        ap.0 == bp.0 && ap.1 == bp.1
    }

    fn mapping_parents(
        mapper: &Mapper<HAST, Dsrc, Ddst, M>,
        l: &(Dsrc::IdD, Ddst::IdD),
    ) -> (Option<Dsrc::IdD>, Option<Ddst::IdD>) {
        (mapper.src_arena.parent(&l.0), mapper.dst_arena.parent(&l.1))
    }

    fn compare_delta_pos(
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

impl<'a, HAST: HyperAST + Copy, Dsrc, Ddst, M: MonoMappingStore>
    crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
where
    M::Src: Debug + Copy,
    M::Dst: Debug + Copy,
{
    pub(crate) fn add_mapping_recursively<Src, Dst>(&mut self, src: &Src, dst: &Dst)
    where
        Src: Shallow<M::Src>,
        Dst: Shallow<M::Dst>,
        Dsrc: DecompressedWithParent<HAST, Src> + DecompressedTreeStore<HAST, Src, M::Src>,
        Ddst: DecompressedWithParent<HAST, Dst> + DecompressedTreeStore<HAST, Dst, M::Dst>,
    {
        self.mappings
            .link(src.shallow().clone(), dst.shallow().clone());
        // WARN check if it works well
        let src = self.src_arena.descendants(src);
        let dst = self.dst_arena.descendants(dst);
        src.iter()
            .zip(dst.iter())
            .for_each(|(src, dst)| self.mappings.link(*src, *dst));
    }
}

impl<
        Dsrc: LazyDecompressed<M::Src>,
        Ddst: LazyDecompressed<M::Dst>,
        HAST: HyperAST + Copy,
        M: MonoMappingStore,
    > crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs + WithStats,
    HAST::IdN: Clone + Eq,
    HAST::Label: Eq,
    Dsrc::IdD: Clone,
    Ddst::IdD: Clone,
    M::Src: Debug + Copy,
    M::Dst: Debug + Copy,
    Dsrc: DecompressedWithParent<HAST, Dsrc::IdD> + LazyDecompressedTreeStore<HAST, M::Src>,
    Ddst: DecompressedWithParent<HAST, Ddst::IdD> + LazyDecompressedTreeStore<HAST, M::Dst>,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    // #[time("warn")]
    pub fn compute_multimapping<
        MM: MultiMappingStore<Src = Dsrc::IdD, Dst = Ddst::IdD>,
        const MIN_HEIGHT: usize,
    >(
        hyperast: HAST,
        src_arena: &mut Dsrc,
        dst_arena: &mut Ddst,
        multi_mappings: &mut MM,
    ) {
        // use crate::matchers::heuristic::gt::lazy_greedy_subtree_matcher::PriorityTreeList;
        let now = std::time::Instant::now();
        let mut src_trees = PriorityTreeList::<'_, Dsrc, M::Src, Dsrc::IdD, HAST, MIN_HEIGHT>::new(
            hyperast,
            src_arena.starter(),
            src_arena,
        );
        let mut dst_trees = PriorityTreeList::<'_, Ddst, M::Dst, Ddst::IdD, HAST, MIN_HEIGHT>::new(
            hyperast,
            dst_arena.starter(),
            dst_arena,
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
                        Self::isomorphic_aux::<true>(hyperast, &src, &dst)
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
        stores: HAST,
        src: &HAST::IdN,
        dst: &HAST::IdN,
    ) -> bool {
        if src == dst {
            return true;
        }
        let src = stores.node_store().resolve(src);
        let dst = stores.node_store().resolve(dst);
        if H {
            let src_h = WithHashs::hash(&src, &<HAST::RT as WithHashs>::HK::label());
            let dst_h = WithHashs::hash(&dst, &<HAST::RT as WithHashs>::HK::label());
            if src_h != dst_h {
                return false;
            }
        };
        if !stores.type_eq(&src, &dst) {
            return false;
        }
        if dst.has_label() && src.has_label() {
            if src.get_label_unchecked() != dst.get_label_unchecked() {
                return false;
            }
        } else if dst.has_label() || src.has_label() {
            return false;
        };

        if src.child_count() != dst.child_count() {
            return false;
        }
        if !src.has_children() {
            return true;
        }
        let r = match (src.children(), dst.children()) {
            (None, None) => true,
            (Some(src_c), Some(dst_c)) => {
                for (src, dst) in src_c.iter_children().zip(dst_c.iter_children()) {
                    if !Self::isomorphic_aux::<false>(stores, &src, &dst) {
                        return false;
                    }
                }
                true
            }
            _ => false,
        };
        r
    }
}

// can be inlined and modified if needed
use super::lazy_greedy_subtree_matcher::PriorityTreeList;
