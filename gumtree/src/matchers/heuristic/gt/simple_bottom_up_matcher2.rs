use std::{fmt::Debug, marker::PhantomData};

use num_traits::{cast, zero, PrimInt, ToPrimitive};

use crate::decompressed_tree_store::{BreathFirstContiguousSiblings, DecompressedWithParent};
use crate::matchers::mapping_store::MonoMappingStore;
use crate::matchers::{matcher::Matcher, similarity_metrics};
use hyper_ast::types::{NodeStore, Tree, WithHashs};

use super::bottom_up_matcher::BottomUpMatcher;

// use super::{decompressed_tree_store::BreathFirstContigousSiblings, mapping_store::DefaultMappingStore, matcher::Matcher, similarity_metrics};

pub struct SimpleBottomUpMatcher<
    'a,
    Dsrc,
    Ddst,
    IdD: PrimInt + Into<usize> + std::ops::SubAssign + Debug,
    T: 'a + Tree + WithHashs,
    S, //: 'a + NodeStore2<T::TreeId, R<'a> = T>, //NodeStore<'a, T::TreeId, T>,
    // const SIM_THRESHOLD: u64 = (0.4).bytes(),
    M: MonoMappingStore<Ele = IdD>,
> {
    internal: BottomUpMatcher<'a, Dsrc, Ddst, IdD, T, S, M>,
}

impl<
        'a,
        Dsrc: 'a
            + BreathFirstContiguousSiblings<'a, T, IdD>
            + DecompressedWithParent<'a, T, IdD>,
        Ddst: 'a
            + BreathFirstContiguousSiblings<'a, T, IdD>
            + DecompressedWithParent<'a, T, IdD>,
        IdD: 'a + PrimInt + Into<usize> + std::ops::SubAssign + Debug,
        T: 'a + Tree + WithHashs,
        S, //: 'a + NodeStore2<T::TreeId, R<'a> = T>, //NodeStore<'a, T::TreeId, T>,
        M: MonoMappingStore<Ele = IdD>,
    > Matcher<'a, Dsrc, Ddst, T, S> for SimpleBottomUpMatcher<'a, Dsrc, Ddst, IdD, T, S, M>
where
    S: 'a + NodeStore<T::TreeId>,
    // for<'c> < <S as NodeStore2<T::TreeId>>::R  as GenericItem<'c>>::Item:Tree<TreeId = T::TreeId,Type = T::Type,Label = T::Label,ChildIdx = T::ChildIdx> + WithHashs<HK = T::HK, HP = T::HP>,
    S::R<'a>: Tree<TreeId = T::TreeId, Type = T::Type> + WithHashs<HK = T::HK, HP = T::HP>,
{
    type Store = M;

    type Ele = IdD;

    fn matchh(
        compressed_node_store: &'a S,
        src: &T::TreeId,
        dst: &T::TreeId,
        mappings: Self::Store,
    ) -> Self::Store {
        let mut matcher = SimpleBottomUpMatcher {
            internal: BottomUpMatcher::<'a, Dsrc, Ddst, IdD, T, S, M> {
                node_store: compressed_node_store,
                src_arena: Dsrc::new(compressed_node_store, src),
                dst_arena: Ddst::new(compressed_node_store, dst),
                mappings,
                phantom: PhantomData,
            },
        };
        matcher.internal.mappings.topit(
            matcher.internal.src_arena.len(),
            matcher.internal.dst_arena.len(),
        );
        Self::execute(&mut matcher);
        matcher.internal.mappings
    }
}

impl<
        'a,
        Dsrc: 'a
            + BreathFirstContiguousSiblings<'a, T, IdD>
            + DecompressedWithParent<'a, T, IdD>,
        Ddst: 'a
            + BreathFirstContiguousSiblings<'a, T, IdD>
            + DecompressedWithParent<'a, T, IdD>,
        IdD: PrimInt + Into<usize> + std::ops::SubAssign + Debug,
        T: 'a + Tree + WithHashs,
        S, //: 'a+NodeStore2<T::TreeId,R<'a>=T>,//NodeStore<'a, T::TreeId, T>,
        M: MonoMappingStore<Ele = IdD>,
    > SimpleBottomUpMatcher<'a, Dsrc, Ddst, IdD, T, S, M>
where
    S: 'a + NodeStore<T::TreeId>,
    // for<'c> < <S as NodeStore2<T::TreeId>>::R  as GenericItem<'c>>::Item:Tree<TreeId = T::TreeId,Type = T::Type,Label = T::Label,ChildIdx = T::ChildIdx> + WithHashs<HK = T::HK, HP = T::HP>,
    S::R<'a>: Tree<TreeId = T::TreeId, Type = T::Type> + WithHashs<HK = T::HK, HP = T::HP>,
{
    fn execute(&mut self) {
        for i in (0..self.internal.src_arena.len()).rev() {
            let i = cast(i).unwrap();
            let a: IdD = num_traits::cast(i).unwrap();
            if !(self.internal.mappings.is_src(&a) || !self.internal.src_arena.has_children(&i)) {
                let candidates = self.internal.get_dst_candidates(&a);
                let mut found = false;
                let mut best = zero();
                let mut max: f64 = -1.;
                let size = self
                    .internal
                    .src_arena
                    .descendants(self.internal.node_store, &i)
                    .len();

                for cand in candidates {
                    // let b = &self.internal.src_arena.original(cand);
                    let threshold = (1.0 as f64)
                        / (1.0 as f64
                            + ((self
                                .internal
                                .src_arena
                                .descendants(self.internal.node_store, &cand)
                                .len()
                                + size)
                                .to_f64()
                                .unwrap())
                            .log10());
                    let sim = similarity_metrics::chawathe_similarity(
                        &self
                            .internal
                            .src_arena
                            .descendants(self.internal.node_store, &i),
                        &self
                            .internal
                            .dst_arena
                            .descendants(self.internal.node_store, &cand),
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
    }
}
