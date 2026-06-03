use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedWithParent, LazyDecompressedTreeStore, Shallow,
};
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::{mapping_store::MultiMappingStore, similarity_metrics};
use crate::utils::sequence_algorithms::longest_common_subsequence;
use hyperast::compat::HashMap;
use hyperast::types::{
    Childrn, DecompressedFrom, HashKind, HyperAST, Labeled, NodeId, NodeStore, Tree, WithChildren,
    WithHashs, WithStats,
};
use hyperast::PrimInt;
use logging_timer::time;
use num_traits::ToPrimitive;
use std::fmt::Debug;
use std::hash::Hash;
pub struct LazyGreedySubtreeMatcher<Dsrc, Ddst, HAST, M, const MIN_HEIGHT: usize = 1> {
    internal: SubtreeMatcher<Dsrc, Ddst, HAST, M, MIN_HEIGHT>,
}

impl<
        Dsrc: DecompressedWithParent<HAST, Dsrc::IdD>
            + ContiguousDescendants<HAST, Dsrc::IdD, M::Src>
            + LazyDecompressedTreeStore<HAST, M::Src>,
        Ddst: DecompressedWithParent<HAST, Ddst::IdD>
            + ContiguousDescendants<HAST, Ddst::IdD, M::Dst>
            + LazyDecompressedTreeStore<HAST, M::Dst>,
        HAST: HyperAST + Copy,
        M: MonoMappingStore,
        const MIN_HEIGHT: usize, // = 2
    > LazyGreedySubtreeMatcher<Dsrc, Ddst, HAST, M, MIN_HEIGHT>
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs + WithStats + Labeled,
    Dsrc::IdD: PrimInt + Hash,
    Ddst::IdD: PrimInt + Hash,
    M::Src: PrimInt + Hash,
    M::Dst: PrimInt + Hash,
    HAST::Label: Eq + Clone,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    pub fn match_it<MM>(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
    where
        MM: MultiMappingStore<Src = Dsrc::IdD, Dst = Ddst::IdD> + Default,
    {
        let mut matcher = Self {
            internal: SubtreeMatcher {
                stores: mapping.hyperast,
                src_arena: mapping.mapping.src_arena,
                dst_arena: mapping.mapping.dst_arena,
                mappings: mapping.mapping.mappings,
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

    pub fn matchh<MM: MultiMappingStore<Src = Dsrc::IdD, Dst = Ddst::IdD> + Default>(
        stores: HAST,
        src: &HAST::IdN,
        dst: &HAST::IdN,
        mappings: M,
    ) -> LazyGreedySubtreeMatcher<Dsrc, Ddst, HAST, M, MIN_HEIGHT>
    where
        Dsrc: DecompressedFrom<HAST, Out = Dsrc>,
        Ddst: DecompressedFrom<HAST, Out = Ddst>,
    {
        let src_arena = Dsrc::decompress(stores, src);
        let dst_arena = Ddst::decompress(stores, dst);
        let mut matcher = Self::new(stores, src_arena, dst_arena, mappings);
        Self::execute::<MM>(&mut matcher);
        matcher
    }

    // [2022-12-19T17:00:02.948Z WARN] considering_stats(), Elapsed=383.306235ms
    // [2022-12-19T17:00:03.334Z WARN] considering_stats(), Elapsed=385.759276ms
    // [2022-12-19T17:00:03.725Z WARN] matchh_to_be_filtered(), Elapsed=388.976068ms
    // [2022-12-19T17:00:03.732Z WARN] filter_mappings(), Elapsed=7.145745ms

    // with WithStats to get height through metadata
    // [2022-12-19T17:11:48.121Z WARN] matchh_to_be_filtered(), Elapsed=16.639973ms

    pub fn execute<MM: MultiMappingStore<Src = Dsrc::IdD, Dst = Ddst::IdD> + Default>(&mut self) {
        let mm: MM = self.compute_multi_mapping();
        self.filter_mappings(&mm);
    }

    pub fn compute_multi_mapping<
        MM: MultiMappingStore<Src = Dsrc::IdD, Dst = Ddst::IdD> + Default,
    >(
        &mut self,
    ) -> MM {
        let mut mm: MM = Default::default();
        mm.topit(self.internal.src_arena.len(), self.internal.dst_arena.len());
        self.internal.matchh_to_be_filtered(&mut mm);
        mm
    }
}

impl<
        Dsrc: DecompressedWithParent<HAST, Dsrc::IdD>
            + ContiguousDescendants<HAST, Dsrc::IdD, M::Src>
            + LazyDecompressedTreeStore<HAST, M::Src>,
        Ddst: DecompressedWithParent<HAST, Ddst::IdD>
            + ContiguousDescendants<HAST, Ddst::IdD, M::Dst>
            + LazyDecompressedTreeStore<HAST, M::Dst>,
        HAST: HyperAST + Copy,
        M: MonoMappingStore,
        const MIN_HEIGHT: usize, // = 2
    > LazyGreedySubtreeMatcher<Dsrc, Ddst, HAST, M, MIN_HEIGHT>
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs + WithStats,
    Dsrc::IdD: PrimInt + Hash,
    Ddst::IdD: PrimInt + Hash,
    M::Src: PrimInt + Hash,
    M::Dst: PrimInt + Hash,
    HAST::Label: Eq + Clone,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    pub fn new(
        stores: HAST,
        src_arena: Dsrc,
        dst_arena: Ddst,
        mappings: M,
    ) -> LazyGreedySubtreeMatcher<Dsrc, Ddst, HAST, M, MIN_HEIGHT> {
        let mut matcher = LazyGreedySubtreeMatcher {
            internal: SubtreeMatcher {
                stores,
                src_arena,
                dst_arena,
                mappings,
            },
        };
        matcher.internal.mappings.topit(
            matcher.internal.src_arena.len() + 1,
            matcher.internal.dst_arena.len() + 1,
        );
        matcher
    }

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
                    .descendants(&src)
                    .iter()
                    .for_each(|src| src_ignored.set(src.to_usize().unwrap(), true));
                dst_ignored.set(dst_i, true);
                self.internal
                    .dst_arena
                    .descendants(&dst)
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
                let n = self.internal.stores.node_store().resolve(&o);
                (
                    self.internal.stores.resolve_type(&o),
                    n.try_get_label().cloned(),
                )
            };
            let o = self.internal.dst_arena.original(b);
            let n = self.internal.stores.node_store().resolve(&o);
            t == self.internal.stores.resolve_type(&o) && l.as_ref() == n.try_get_label()
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
                        .position_in_parent::<usize>(&x)
                        .unwrap()
                        .to_f64()
                        .unwrap()
                        / self.internal.src_arena.children(&p).len().to_f64().unwrap()
                })
            });
        let dsts = vec![l.1]
            .into_iter()
            .chain(self.internal.dst_arena.parents(l.1))
            .filter_map(|x| {
                self.internal.dst_arena.parent(&x).map(|p| {
                    self.internal
                        .dst_arena
                        .position_in_parent::<usize>(&x)
                        .unwrap()
                        .to_f64()
                        .unwrap()
                        / self.internal.dst_arena.children(&p).len().to_f64().unwrap()
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

impl<'a, Dsrc, Ddst, S, M: MonoMappingStore, const MIN_HEIGHT: usize>
    Into<SubtreeMatcher<Dsrc, Ddst, S, M, MIN_HEIGHT>>
    for LazyGreedySubtreeMatcher<Dsrc, Ddst, S, M, MIN_HEIGHT>
{
    fn into(self) -> SubtreeMatcher<Dsrc, Ddst, S, M, MIN_HEIGHT> {
        self.internal
    }
}

pub struct SubtreeMatcher<Dsrc, Ddst, HAST, M, const MIN_HEIGHT: usize> {
    pub(super) stores: HAST,
    pub src_arena: Dsrc,
    pub dst_arena: Ddst,
    pub mappings: M,
}

impl<
        Dsrc: DecompressedWithParent<HAST, Dsrc::IdD> + LazyDecompressedTreeStore<HAST, M::Src>,
        Ddst: DecompressedWithParent<HAST, Ddst::IdD> + LazyDecompressedTreeStore<HAST, M::Dst>,
        HAST: HyperAST + Copy,
        M: MonoMappingStore,
        const MIN_HEIGHT: usize,
    > SubtreeMatcher<Dsrc, Ddst, HAST, M, MIN_HEIGHT>
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs + WithStats,
    Dsrc::IdD: Clone,
    Ddst::IdD: Clone,
    M::Src: Debug + Copy,
    M::Dst: Debug + Copy,
    HAST::Label: Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    pub(crate) fn add_mapping_recursively(&mut self, src: &Dsrc::IdD, dst: &Ddst::IdD) {
        self.mappings
            .link(src.shallow().clone(), dst.shallow().clone());
        // WARN check if it works well
        self.src_arena
            .descendants(src)
            .iter()
            .zip(self.dst_arena.descendants(dst).iter())
            .for_each(|(src, dst)| self.mappings.link(*src, *dst));
    }

    #[time("warn")]
    fn matchh_to_be_filtered<MM: MultiMappingStore<Src = Dsrc::IdD, Dst = Ddst::IdD>>(
        &mut self,
        multi_mappings: &mut MM,
    ) {
        let now = std::time::Instant::now();
        let mut src_trees = PriorityTreeList::<_, _, _, HAST, MIN_HEIGHT>::new(
            self.stores,
            self.src_arena.starter(),
            &mut self.src_arena,
        );
        let mut dst_trees = PriorityTreeList::<_, _, _, HAST, MIN_HEIGHT>::new(
            self.stores,
            self.dst_arena.starter(),
            &mut self.dst_arena,
        );
        let match_init_t = now.elapsed().as_secs_f64();
        dbg!(match_init_t);
        let pop_larger = |src_trees: &mut PriorityTreeList<
            '_,
            Dsrc,
            M::Src,
            Dsrc::IdD,
            HAST,
            MIN_HEIGHT,
        >,

                          dst_trees: &mut PriorityTreeList<
            '_,
            Ddst,
            M::Dst,
            Ddst::IdD,
            HAST,
            MIN_HEIGHT,
        >| {
            if src_trees.peek_height() > dst_trees.peek_height() {
                src_trees.open();
            } else {
                dst_trees.open();
            }
        };
        while src_trees.peek_height() != -1 && dst_trees.peek_height() != -1 {
            // println!("multi_mappings={}", multi_mappings.len());
            while src_trees.peek_height() != dst_trees.peek_height() {
                pop_larger(&mut src_trees, &mut dst_trees);
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
                        Self::isomorphic_aux::<true>(self.stores, &src, &dst)
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
        let _src = stores.node_store().resolve(src);
        let src_h = if H {
            Some(WithHashs::hash(&_src, &HashKind::label()))
        } else {
            None
        };
        let src_t = stores.resolve_type(&src);
        let src_l = if _src.has_label() {
            Some(_src.get_label_unchecked())
        } else {
            None
        };
        let src_c: Option<Vec<_>> = _src.children().map(|x| x.iter_children().collect());

        let _dst = stores.node_store().resolve(dst);

        if let Some(src_h) = src_h {
            let dst_h = WithHashs::hash(&_dst, &HashKind::label());
            if src_h != dst_h {
                return false;
            }
        }
        let dst_t = stores.resolve_type(&dst);
        if src_t != dst_t {
            return false;
        }
        if _dst.has_label() {
            if src_l.is_none() || src_l.unwrap() != _dst.get_label_unchecked() {
                return false;
            }
        };

        let dst_c: Option<Vec<_>> = _dst.children().map(|x| x.iter_children().collect());

        match (src_c, dst_c) {
            (None, None) => true,
            (Some(src_c), Some(dst_c)) => {
                if src_c.len() != dst_c.len() {
                    false
                } else {
                    for (src, dst) in src_c.iter().zip(dst_c.iter()) {
                        if !Self::isomorphic_aux::<false>(stores, src, dst) {
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

pub(super) struct PriorityTreeList<'b, D, IdS, IdD, S, const MIN_HEIGHT: usize> {
    pub trees: Vec<Option<Vec<IdD>>>,

    pub store: S,
    pub(super) arena: &'b mut D,

    pub max_height: usize,

    pub current_idx: isize,

    pub phantom: std::marker::PhantomData<IdS>,
}

impl<'b, D, IdD, HAST, const MIN_HEIGHT: usize>
    PriorityTreeList<'b, D, IdD, D::IdD, HAST, MIN_HEIGHT>
where
    D::IdD: Clone,
    D: LazyDecompressedTreeStore<HAST, IdD>,
    HAST: HyperAST + Copy,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithStats,
{
    pub(super) fn new(store: HAST, tree: D::IdD, arena: &'b mut D) -> Self {
        let id = arena.original(&tree);
        let h = store.resolve(&id).height() - 1;
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
            phantom: std::marker::PhantomData,
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
        let id = self.arena.original(&tree);
        let h = self.store.resolve(&id).height() - 1;
        self.add_tree_aux(tree, h)
    }

    pub(super) fn add_tree_aux(&mut self, tree: D::IdD, h: usize) {
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
        for c in self.arena.decompress_children(tree) {
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
