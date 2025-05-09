use std::collections::HashMap;
use super::bottom_up_matcher::BottomUpMatcher;
use crate::decompressed_tree_store::SimpleZsTree as ZsTree;
use crate::decompressed_tree_store::{
    ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent, POBorrowSlice, PostOrder,
    PostOrderIterable, PostOrderKeyRoots,
};
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::{Decompressible, Mapper};
use crate::matchers::{optimal::zs::ZsMatcher, similarity_metrics};
use hyperast::types::{DecompressedFrom, HyperAST, LabelStore, NodeId, NodeStore, Tree, WithHashs};
use hyperast::PrimInt;
use num_traits::{cast, one};
use std::fmt::Debug;
use hyperast::types::Labeled;

/// TODO wait for `#![feature(adt_const_params)]` #95174 to be improved
///
/// it will allow to make use complex types as const generics
/// ie. make the different threshold neater
pub struct XYBottomUpMatcher<
    Dsrc,
    Ddst,
    HAST,
    M: MonoMappingStore,
    const SIM_THRESHOLD_NUM: u64 = 1,
    const SIM_THRESHOLD_DEN: u64 = 2,
> {
    internal: BottomUpMatcher<Dsrc, Ddst, HAST, M>,
}

/// Enable using a slice instead of recreating a ZsTree for each call to ZsMatch, see last_chance_match
const SLICE: bool = true;

impl<
    Dsrc,
    Ddst,
    HAST: HyperAST,
    M: MonoMappingStore,
    const SIM_THRESHOLD_NUM: u64, // = 1,
    const SIM_THRESHOLD_DEN: u64, // = 2,
> Into<BottomUpMatcher<Dsrc, Ddst, HAST, M>>
for XYBottomUpMatcher<
    Dsrc,
    Ddst,
    HAST,
    M,
    SIM_THRESHOLD_NUM,
    SIM_THRESHOLD_DEN,
>
{
    fn into(self) -> BottomUpMatcher<Dsrc, Ddst, HAST, M> {
        self.internal
    }
}

/// TODO PostOrder might not be necessary
impl<
    'a,
    Dsrc: DecompressedTreeStore<HAST, M::Src>
    + DecompressedWithParent<HAST, M::Src>
    + PostOrder<HAST, M::Src>
    + PostOrderIterable<HAST, M::Src>
    + DecompressedFrom<HAST, Out = Dsrc>
    + ContiguousDescendants<HAST, M::Src>
    + POBorrowSlice<HAST, M::Src>,
    Ddst: DecompressedTreeStore<HAST, M::Dst>
    + DecompressedWithParent<HAST, M::Dst>
    + PostOrder<HAST, M::Dst>
    + PostOrderIterable<HAST, M::Dst>
    + DecompressedFrom<HAST, Out = Ddst>
    + ContiguousDescendants<HAST, M::Dst>
    + POBorrowSlice<HAST, M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore + Default,
    const SIM_THRESHOLD_NUM: u64,
    const SIM_THRESHOLD_DEN: u64,
>
XYBottomUpMatcher<Dsrc, Ddst, HAST, M, SIM_THRESHOLD_NUM, SIM_THRESHOLD_DEN>
where
        for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
        M::Src: PrimInt,
        M::Dst: PrimInt,
        HAST::Label: Eq,
        HAST::IdN: Debug,
        HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    pub fn new(stores: HAST, src_arena: Dsrc, dst_arena: Ddst, mappings: M) -> Self {
        Self {
            internal: BottomUpMatcher {
                stores,
                src_arena,
                dst_arena,
                mappings,
            },
        }
    }

    pub fn match_it(
        mapping: crate::matchers::Mapper<HAST, Dsrc, Ddst, M>,
    ) -> crate::matchers::Mapper<HAST, Dsrc, Ddst, M> {
        let mut matcher = Self {
            internal: BottomUpMatcher {
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
        Self::execute(&mut matcher);
        crate::matchers::Mapper {
            hyperast: mapping.hyperast,
            mapping: crate::matchers::Mapping {
                src_arena: matcher.internal.src_arena,
                dst_arena: matcher.internal.dst_arena,
                mappings: matcher.internal.mappings,
            },
        }
    }

    pub fn matchh(store: HAST, src: &'a HAST::IdN, dst: &'a HAST::IdN, mappings: M) -> Self {
        let mut matcher = Self::new(
            store,
            Dsrc::decompress(store, src),
            Ddst::decompress(store, dst),
            mappings,
        );
        matcher.internal.mappings.topit(
            matcher.internal.src_arena.len(),
            matcher.internal.dst_arena.len(),
        );
        Self::execute(&mut matcher);
        matcher
    }

    pub fn execute<'b>(&mut self) {
        assert_eq!(
            // TODO move it inside the arena ...
            self.internal.src_arena.root(),
            cast::<_, M::Src>(self.internal.src_arena.len()).unwrap() - one()
        );
        assert!(self.internal.src_arena.len() > 0);
        // // WARN it is in postorder and it depends on decomp store
        // // -1 as root is handled after forloop
        for a in self.internal.src_arena.iter_df_post::<true>() {
            if self.internal.src_arena.parent(&a).is_none() {
                // TODO remove and flip const param of iter_df_post
                break;
            }
            if !(self.internal.mappings.is_src(&a) || !self.src_has_children(a)) {
                let candidates = self.internal.get_dst_candidates(&a);
                let mut best = None;
                let mut max: f64 = -1.;
                for cand in candidates {
                    let sim = similarity_metrics::SimilarityMeasure::range(
                        &self.internal.src_arena.descendants_range(&a),
                        &self.internal.dst_arena.descendants_range(&cand),
                        &self.internal.mappings,
                    )
                        .jaccard();
                    if sim > max && sim >= SIM_THRESHOLD_NUM as f64 / SIM_THRESHOLD_DEN as f64 {
                        max = sim;
                        best = Some(cand);
                    }
                }

                if let Some(best) = best {
                    self.last_chance_match(a, best);
                    self.internal.mappings.link(a, best);
                }
            }
        }
        // for root
        self.internal.mappings.link(
            self.internal.src_arena.root(),
            self.internal.dst_arena.root(),
        );
        self.last_chance_match(
            self.internal.src_arena.root(),
            self.internal.dst_arena.root(),
        );
    }

    fn src_has_children(&mut self, src: M::Src) -> bool {
        use num_traits::ToPrimitive;
        let r = self
            .internal
            .stores
            .node_store()
            .resolve(&self.internal.src_arena.original(&src))
            .has_children();
        assert_eq!(
            r,
            self.internal.src_arena.lld(&src) < src,
            "{:?} {:?}",
            self.internal.src_arena.lld(&src),
            src.to_usize()
        );
        r
    }
    fn last_chance_match(&mut self, src: M::Src, dst: M::Dst) {
        let mut src_types: HashMap<String, Vec<M::Src>> = HashMap::new();
        let mut dst_types: HashMap<String, Vec<M::Dst>> = HashMap::new();

        for src_child in self.internal.src_arena.children(&src) {
            let original = self.internal.src_arena.original(&src_child);
            let resolved = self.internal.stores.node_store().resolve(&original);
            let label = resolved.try_get_label();
            if let Some(label) = label {
                let src_type = self.internal.stores.label_store().resolve(label).to_string();
                src_types.entry(src_type).or_default().push(src_child);
            }
        }

        for dst_child in self.internal.dst_arena.children(&dst) {
            let original = self.internal.dst_arena.original(&dst_child);
            let resolved = self.internal.stores.node_store().resolve(&original);
            let label = resolved.try_get_label();
            if let Some(label) = label {
                let src_type = self.internal.stores.label_store().resolve(label).to_string();
                dst_types.entry(src_type).or_default().push(dst_child);
            }
        }

        for (src_type, src_list) in src_types.iter() {
            if src_list.len() == 1 {
                if let Some(dst_list) = dst_types.get(src_type) {
                    if dst_list.len() == 1 {
                        self.internal.mappings.link(src_list[0], dst_list[0]);
                    }
                }
            }
        }
    }
}
