use std::hash::Hash;

use num_traits::ToPrimitive;

use crate::decompressed_tree_store::{
    BreadthFirstContiguousSiblings, DecompressedTreeStore, DecompressedWithParent,
};
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::{Mapper, similarity_metrics};
use hyperast::types::{HyperAST, NodeId, Tree, WithHashs};

type IdD = u16;

pub struct SimpleBottomUpMatcher<Dsrc, Ddst, S, M>
where
    M: MonoMappingStore<Src = IdD, Dst = IdD>,
{
    internal: Mapper<S, Dsrc, Ddst, M>,
}

impl<
    'a,
    Dsrc: DecompressedTreeStore<HAST, IdD>
        + DecompressedWithParent<HAST, IdD>
        + BreadthFirstContiguousSiblings<HAST, IdD>,
    Ddst: DecompressedTreeStore<HAST, IdD>
        + DecompressedWithParent<HAST, IdD>
        + BreadthFirstContiguousSiblings<HAST, IdD>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore<Src = IdD, Dst = IdD>,
> SimpleBottomUpMatcher<Dsrc, Ddst, HAST, M>
where
    for<'b> <HAST as hyperast::types::AstLending<'b>>::RT: WithHashs,
    HAST::Label: Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    pub fn execute(&mut self) {
        for i in (0..self.internal.src_arena.len()).rev() {
            let a: IdD = num_traits::cast(i).unwrap();
            if !(self.internal.mappings.is_src(&a) || !self.internal.src_arena.has_children(&a)) {
                let candidates = self.internal.get_dst_candidates(&a);
                let mut found = false;
                let mut best = 0;
                let mut max: f64 = -1.;
                let t_size = self.internal.src_arena.descendants(&(i as IdD)).len();

                for cand in candidates {
                    let threshold = (1.0 as f64)
                        / (1.0 as f64
                            + ((self.internal.src_arena.descendants(&cand).len() + t_size)
                                .to_f64()
                                .unwrap())
                            .log10());
                    let sim = similarity_metrics::chawathe_similarity(
                        &self.internal.src_arena.descendants(&(i as IdD)),
                        &self.internal.dst_arena.descendants(&cand),
                        &self.internal.mappings,
                    );
                    if sim > max && sim >= threshold {
                        max = sim;
                        best = cand;
                        found = true;
                    }
                }

                if found {
                    self.internal.last_chance_match_histogram(&a, &best);
                    self.internal.mappings.link(a, best);
                }
            }
        }
        // self.mappings.link(0, 0);
        // self.lastChanceMatch(0, 0);
    }
}
