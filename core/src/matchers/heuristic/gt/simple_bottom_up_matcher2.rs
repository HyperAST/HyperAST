use std::{cell::Ref, collections::HashMap, fmt::Debug, marker::PhantomData};

use num_traits::{cast, zero, PrimInt, ToPrimitive};

use crate::{
    matchers::{
        decompressed_tree_store::BreathFirstContigousSiblings,
        mapping_store::{DefaultMappingStore, MappingStore, MonoMappingStore},
        matcher::{self, Matcher},
        similarity_metrics,
    },
    tree::tree::{HashKind, NodeStore, Tree, Typed, WithHashs},
    utils::sequence_algorithms::longest_common_subsequence,
};

use super::bottom_up_matcher::BottomUpMatcher;

// use super::{decompressed_tree_store::BreathFirstContigousSiblings, mapping_store::DefaultMappingStore, matcher::Matcher, similarity_metrics};

pub struct SimpleBottomUpMatcher<
    'a,
    D: BreathFirstContigousSiblings<T::TreeId, IdD>,
    IdD: PrimInt + Into<usize> + std::ops::SubAssign + Debug,
    T: Tree + WithHashs,
    S: NodeStore<T>,
    // const SIM_THRESHOLD: u64 = (0.4).bytes(),
> {
    internal: BottomUpMatcher<'a, D, IdD, T, S>,
}

impl<
        'a,
        D: 'a + BreathFirstContigousSiblings<T::TreeId, IdD>,
        IdD: PrimInt + Into<usize> + std::ops::SubAssign + Debug,
        T: Tree + WithHashs,
        S: NodeStore<T>,
    > Matcher<'a, D, T, S> for SimpleBottomUpMatcher<'a, D, IdD, T, S>
{
    type Store = DefaultMappingStore<IdD>;

    type Ele = IdD;

    fn matchh(
        compressed_node_store: &'a S,
        src: &'a T::TreeId,
        dst: &'a T::TreeId,
        mappings: Self::Store,
    ) -> Self::Store {
        let mut matcher = SimpleBottomUpMatcher {
            internal: BottomUpMatcher::<'a, D, IdD, T, S> {
                node_store: compressed_node_store,
                src_arena: D::new(compressed_node_store, src),
                dst_arena: D::new(compressed_node_store, dst),
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
        D: 'a + BreathFirstContigousSiblings<T::TreeId, IdD>,
        IdD: PrimInt + Into<usize> + std::ops::SubAssign + Debug,
        T: Tree + WithHashs,
        S: NodeStore<T>,
    > SimpleBottomUpMatcher<'a, D, IdD, T, S>
{
    fn execute(&mut self) {
        for i in (0..self.internal.src_arena.len()).rev() {
            let i = cast(i).unwrap();
            let a: IdD = num_traits::cast(i).unwrap();
            if !(self.internal.mappings.is_src(&a) || !self.internal.src_arena.has_children(&i)) {
                let candidates = self.internal.getDstCandidates(&a);
                let mut found = false;
                let mut best = zero();
                let mut max: f64 = -1.;
                let tSize = self
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
                                + tSize)
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
