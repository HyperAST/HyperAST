use std::{fmt::Debug, marker::PhantomData, ops::Deref};

use num_traits::{cast, one, zero, PrimInt};

use crate::{
    matchers::{
        decompressed_tree_store::{
            DecompressedTreeStore, DecompressedWithParent, PostOrder, SimpleZsTree,
        },
        mapping_store::{DefaultMappingStore, MappingStore, MonoMappingStore},
        optimal::zs::ZsMatcher,
        similarity_metrics,
    },
    tree::tree::{LabelStore, NodeStore, OwnedLabel, Tree, WithHashs},
};

use super::bottom_up_matcher::BottomUpMatcher;

/// todo PostOrder might not be necessary
pub struct GreedyBottomUpMatcher<
    'a,
    D: DecompressedTreeStore<T::TreeId, IdD> + DecompressedWithParent<IdD> + PostOrder<T::TreeId, IdD>,
    IdD: PrimInt + Into<usize> + std::ops::SubAssign + Debug,
    T: Tree + WithHashs,
    S: for<'b> NodeStore<'b,T>,
    LS: LabelStore<OwnedLabel, I = T::Label>,
    const SIZE_THRESHOLD: usize,  // = 1000,
    const SIM_THRESHOLD_NUM: u64, // = 1,
    const SIM_THRESHOLD_DEN: u64, // = 2,
> {
    label_store: &'a LS,
    internal: BottomUpMatcher<'a, D, IdD, T, S>,
    // compressed_node_store: &'a S,
    // pub(crate) src_arena: D,
    // pub(crate) dst_arena: D,
    // pub mappings: DefaultMappingStore<IdD>,
}

impl<
        'a,
        D: 'a
            + DecompressedTreeStore<T::TreeId, IdD>
            + DecompressedWithParent<IdD>
            + PostOrder<T::TreeId, IdD>,
        IdD: PrimInt + Into<usize> + std::ops::SubAssign + Debug,
        T: Tree + WithHashs,
        S: for<'b> NodeStore<'b,T>,
        LS: LabelStore<OwnedLabel, I = T::Label>,
        const SIZE_THRESHOLD: usize,  // = 1000,
        const SIM_THRESHOLD_NUM: u64, // = 1,
        const SIM_THRESHOLD_DEN: u64, // = 2,
    > Into<BottomUpMatcher<'a, D, IdD, T, S>>
    for GreedyBottomUpMatcher<
        'a,
        D,
        IdD,
        T,
        S,
        LS,
        SIZE_THRESHOLD,
        SIM_THRESHOLD_NUM,
        SIM_THRESHOLD_DEN,
    >
{
    fn into(self) -> BottomUpMatcher<'a, D, IdD, T, S> {
        self.internal
    }
}

impl<
        'a,
        D: 'a
            + DecompressedTreeStore<T::TreeId, IdD>
            + DecompressedWithParent<IdD>
            + PostOrder<T::TreeId, IdD>,
        IdD: 'a + PrimInt + Into<usize> + std::ops::SubAssign + Debug,
        T: Tree + WithHashs,
        S: for<'b> NodeStore<'b,T>,
        LS: 'a + LabelStore<OwnedLabel, I = T::Label>,
        const SIZE_THRESHOLD: usize, // = 1000,
        // Integer.parseInt(System.getProperty("gt.bum.szt", "1000"));
        const SIM_THRESHOLD_NUM: u64, // = 1,
        const SIM_THRESHOLD_DEN: u64, // = 2,
                                      // Double.parseDouble(System.getProperty("gt.bum.smt", "0.5"));
    >
    GreedyBottomUpMatcher<
        'a,
        D,
        IdD,
        T,
        S,
        LS,
        SIZE_THRESHOLD,
        SIM_THRESHOLD_NUM,
        SIM_THRESHOLD_DEN,
    >
where
    T::TreeId: 'a + PrimInt,
{
    pub fn new(
        node_store: &'a S,
        label_store: &'a LS,
        src_arena: D,
        dst_arena: D,
        mappings: DefaultMappingStore<IdD>,
    ) -> GreedyBottomUpMatcher<
        'a,
        D,
        IdD,
        T,
        S,
        LS,
        SIZE_THRESHOLD,
        SIM_THRESHOLD_NUM,
        SIM_THRESHOLD_DEN,
    > {
        Self {
            label_store,
            internal: BottomUpMatcher {
                node_store,
                src_arena,
                dst_arena,
                mappings,
                phantom: PhantomData,
            },
        }
    }

    pub fn matchh(
        compressed_node_store: &'a S,
        label_store: &'a LS,
        src: &'a T::TreeId,
        dst: &'a T::TreeId,
        mappings: DefaultMappingStore<IdD>,
    ) -> GreedyBottomUpMatcher<
        'a,
        D,
        IdD,
        T,
        S,
        LS,
        SIZE_THRESHOLD,
        SIM_THRESHOLD_NUM,
        SIM_THRESHOLD_DEN,
    > {
        let mut matcher = Self::new(
            compressed_node_store,
            label_store,
            D::new(compressed_node_store, src),
            D::new(compressed_node_store, dst),
            mappings,
        );
        matcher.internal.mappings.topit(
            matcher.internal.src_arena.len() + 1,
            matcher.internal.dst_arena.len() + 1,
        );
        Self::execute(&mut matcher);
        matcher
    }

    pub(crate) fn execute(&mut self) {
        assert_eq!(
            self.internal.src_arena.root(),
            cast::<_, IdD>(self.internal.src_arena.len()).unwrap() - one()
        );
        assert!(self.internal.src_arena.len() > 0);
        // todo caution about, it is in postorder and it depends on decomp store
        // -1 as root is handled after forloop
        for i in 0..self.internal.src_arena.len() - 1 {
            let a: IdD = num_traits::cast(i).unwrap(); //unsafe as it depends on decompressed store
            println!("{}", self.src_has_children(a));
            if !(self.internal.mappings.is_src(&a) || !self.src_has_children(a)) {
                let candidates = self.internal.getDstCandidates(&a);
                let mut found = false;
                let mut best = num_traits::zero();
                let mut max: f64 = -1.;

                for cand in candidates {
                    let sim = similarity_metrics::dice_similarity(
                        &self
                            .internal
                            .src_arena
                            .descendants(self.internal.node_store, &cast(i).unwrap()),
                        &self
                            .internal
                            .dst_arena
                            .descendants(self.internal.node_store, &cand),
                        &self.internal.mappings,
                    );
                    if sim > max && sim >= SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64 {
                        max = sim;
                        best = cand;
                        found = true;
                    }
                }

                if found {
                    // self.internal.lastChanceMatch_Zs::<_,SIZE_THRESHOLD>(self.label_store,a, best);
                    self.last_chance_match_zs::<IdD>(a, best);
                    self.internal.mappings.link(a, best);
                }
            }
        }
        // for root
        self.internal.mappings.link(
            self.internal.src_arena.root(),
            self.internal.dst_arena.root(),
        );
        self.last_chance_match_zs::<IdD>(
            self.internal.src_arena.root(),
            self.internal.dst_arena.root(),
        );
        // todo!()
    }

    fn src_has_children(&mut self, src: IdD) -> bool {
        self.internal
            .node_store
            .resolve(&self.internal.src_arena.original(&src))
            .has_children()
    }

    pub(crate) fn last_chance_match_zs<IdDZs>(&mut self, src: IdD, dst: IdD)
    where
        IdDZs: 'a + PrimInt + Debug + Into<usize> + std::ops::SubAssign,
    {
        let src_offset = self.internal.src_arena.first_descendant(&src);
        let dst_offset = self.internal.dst_arena.first_descendant(&dst);
        let src_s = self
            .internal
            .src_arena
            .descendants_count(self.internal.node_store, &src);
        let dst_s = self
            .internal
            .dst_arena
            .descendants_count(self.internal.node_store, &dst);
        let src = self.internal.src_arena.original(&src);
        let dst = self.internal.dst_arena.original(&dst);
        if src_s < cast(SIZE_THRESHOLD).unwrap() || dst_s < cast(SIZE_THRESHOLD).unwrap() {
            let mappings = DefaultMappingStore::<IdDZs>::new();
            let mappings = {
                let matcher = ZsMatcher::<'a, SimpleZsTree<T::TreeId, IdDZs>, _, _, _, _>::matchh(
                    self.internal.node_store,
                    self.label_store,
                    src,
                    dst,
                    mappings,
                );

                matcher.mappings
            };
            for (i, t) in mappings
                .src_to_dst
                .iter()
                .enumerate()
                .filter(|x| *x.1 != num_traits::zero())
                .map(|(src, dst)| (cast::<_, IdDZs>(src).unwrap(), *dst))
            {
                //remapping
                let src: IdD = src_offset + cast(i - num_traits::one()).unwrap();
                let dst: IdD =
                    dst_offset + cast(t - num_traits::one() - num_traits::one()).unwrap();
                // use it
                if !self.internal.mappings.is_src(&src) && !self.internal.mappings.is_dst(&dst) {
                    let tsrc = self
                        .internal
                        .node_store
                        .resolve(&self.internal.src_arena.original(&src))
                        .get_type();
                    let tdst = self
                        .internal
                        .node_store
                        .resolve(&self.internal.dst_arena.original(&dst))
                        .get_type();

                    if tsrc == tdst {
                        self.internal.mappings.link(src, dst);
                    }
                }
            }
        }
    }
}
