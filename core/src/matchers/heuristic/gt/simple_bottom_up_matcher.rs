use std::{cell::Ref, collections::HashMap, marker::PhantomData};

use num_traits::{cast, PrimInt, ToPrimitive};

use crate::{
    matchers::{
        decompressed_tree_store::{
            BreathFirstContiguousSiblings, DecompressedTreeStore, DecompressedWithParent,
        },
        mapping_store::{DefaultMappingStore, MappingStore, MonoMappingStore},
        matcher::Matcher,
        similarity_metrics,
    },
    tree::tree::{HashKind, NodeStore, Tree, Typed, WithHashs},
    utils::sequence_algorithms::longest_common_subsequence,
};

use super::bottom_up_matcher::BottomUpMatcher;

// use super::{decompressed_tree_store::DecompressedTreeStore, mapping_store::DefaultMappingStore, matcher::Matcher, similarity_metrics};

type IdD = u16;

const SIM_THRESHOLD: f64 = 0.4;

pub struct SimpleBottomUpMatcher<
    'a,
    D: DecompressedTreeStore<T::TreeId, IdD>
        + DecompressedWithParent<IdD>
        + BreathFirstContiguousSiblings<T::TreeId, IdD>,
    T: Tree + WithHashs,
    S: for<'b> NodeStore<'b,T>,
    // const SIM_THRESHOLD: u64 = (0.4).bytes(),
> {
    internal: BottomUpMatcher<'a, D, IdD, T, S>,
    // compressed_node_store: &'a S,
    // src_arena: D,
    // dst_arena: D,
    // mappings: DefaultMappingStore<IdD>,
    // phantom: PhantomData<*const T>,
}

impl<
        'a,
        D: 'a
            + DecompressedTreeStore<T::TreeId, IdD>
            + DecompressedWithParent<IdD>
            + BreathFirstContiguousSiblings<T::TreeId, IdD>,
        T: Tree + WithHashs,
        S: for<'b> NodeStore<'b,T>,
    > Matcher<'a, D, T, S> for SimpleBottomUpMatcher<'a, D, T, S>
{
    type Store = DefaultMappingStore<IdD>;

    type Ele = IdD;

    fn matchh(
        compressed_node_store: &'a S,
        src: &'a T::TreeId,
        dst: &'a T::TreeId,
        mappings: Self::Store,
    ) -> Self::Store {
        let mut matcher = Self {
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
        D: 'a
            + DecompressedTreeStore<T::TreeId, IdD>
            + DecompressedWithParent<IdD>
            + BreathFirstContiguousSiblings<T::TreeId, IdD>,
        T: Tree + WithHashs,
        S: for<'b> NodeStore<'b,T>,
    > SimpleBottomUpMatcher<'a, D, T, S>
{
    fn execute(&mut self) {
        for i in (0..self.internal.src_arena.len()).rev() {
            let a: IdD = num_traits::cast(i).unwrap();
            if !(self.internal.mappings.is_src(&a) || !self.internal.src_arena.has_children(&a)) {
                let candidates = self.internal.getDstCandidates(&a);
                let mut found = false;
                let mut best = 0;
                let mut max: f64 = -1.;
                let t_size = self
                    .internal
                    .src_arena
                    .descendants(self.internal.node_store, &(i as IdD))
                    .len();

                for cand in candidates {
                    let threshold = (1.0 as f64)
                        / (1.0 as f64
                            + ((self
                                .internal
                                .src_arena
                                .descendants(self.internal.node_store, &cand)
                                .len()
                                + t_size)
                                .to_f64()
                                .unwrap())
                            .log10());
                    let sim = similarity_metrics::chawathe_similarity(
                        &self
                            .internal
                            .src_arena
                            .descendants(self.internal.node_store, &(i as IdD)),
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
        // self.mappings.link(0, 0);
        // self.lastChanceMatch(0, 0);
    }

    // fn getDescendants_decompressed(&self, src: IdD) -> Vec<IdD> {
    //     self.getDescendants_decompressedG(&self.src_arena,src)
    // }

    // fn getDescendants_decompressedG(&self, arena: &D, x: IdD) -> Vec<IdD> {
    //     // todo possible opti by also making descendants contigous in arena
    //     let mut id: Vec<IdD> = vec![x];
    //     let mut id_compressed: Vec<T::TreeId> = vec![arena.original(x)];
    //     let mut i: usize = x as usize;

    //     while i < id.len() {
    //         let node = self.compressed_node_store.get_node_at_id(&id_compressed[i]);
    //         let l = node.get_children();
    //         id_compressed.extend_from_slice(l);

    //         i += 1;
    //     }
    //     id
    // }

    // fn getDstCandidates(&self, src: IdD) -> Vec<IdD> {
    //     let mut seeds = vec![];
    //     let s = &self.internal.src_arena.original(src);
    //     for c in self.internal.src_arena.descendants(self.internal.compressed_node_store, src) {
    //         if self.internal.mappings.is_src(&c) {
    //             let m = self.internal.mappings.get_dst(&c);
    //             seeds.push(m);
    //         }
    //     }
    //     let mut candidates = vec![];
    //     let mut visited: Vec<bool> = vec![];
    //     for mut seed in seeds {
    //         while seed != 0 {
    //             let parent = self.internal.src_arena.parent(seed).unwrap();
    //             if visited[parent.to_usize().unwrap()] {
    //                 break;
    //             }
    //             visited[parent.to_usize().unwrap()] = true;
    //             let p = &self.internal.src_arena.original(parent);
    //             if self.internal.compressed_node_store.get_node_at_id(p).get_type()
    //                 == self.internal.compressed_node_store.get_node_at_id(s).get_type()
    //                 && !self.internal.mappings.is_dst(&parent)
    //                 && parent == 0
    //             {
    //                 candidates.push(parent);
    //             }
    //             seed = parent;
    //         }
    //     }
    //     candidates
    // }

    // fn lastChanceMatch(&mut self, src: IdD, dst: IdD) {
    //     self.internal.lcsEqualMatching(src, dst);
    //     self.internal.lcsStructureMatching(src, dst);
    //     if src != 0 && dst != 0 {
    //         self.internal.histogramMatching(src, dst); //self.internal.histogramMaking(src, dst),
    //     } else if !(src == 0 || dst == 0) {
    //         if self
    //             .internal.compressed_node_store
    //             .get_node_at_id(&self.internal.src_arena.original(self.internal.src_arena.parent(src).unwrap()))
    //             .get_type()
    //             == self
    //                 .internal.compressed_node_store
    //                 .get_node_at_id(&self.internal.dst_arena.original(self.internal.dst_arena.parent(dst).unwrap()))
    //                 .get_type()
    //         {
    //             self.internal.histogramMatching(src, dst) //self.internal.histogramMaking(src, dst),
    //         }
    //     }
    // }

    // fn are_srcs_unmapped(&self, src: &IdD) -> bool {
    //     // look at descendants
    //     // in mappings
    //     self.internal.src_arena
    //         .descendants(self.internal.compressed_node_store, *src)
    //         .iter()
    //         .all(|x| !self.internal.mappings.is_src(x))
    // }
    // fn are_dsts_unmapped(&self, dst: &IdD) -> bool {
    //     // look at descendants
    //     // in mappings
    //     self.internal.dst_arena
    //         .descendants(self.internal.compressed_node_store, *dst)
    //         .iter()
    //         .all(|x| !self.internal.mappings.is_src(x))
    // }

    // pub(crate) fn add_mapping_recursively(&mut self, src: &IdD, dst: &IdD) {
    //     self.internal.src_arena
    //         .descendants(self.internal.compressed_node_store, *src)
    //         .iter()
    //         .zip(
    //             self.internal.dst_arena
    //                 .descendants(self.internal.compressed_node_store, *dst)
    //                 .iter(),
    //         )
    //         .for_each(|(src, dst)| self.internal.mappings.link(*src, *dst));
    // }

    // fn src_children(&self, src: &IdD) -> Vec<IdD> {
    //     {
    //         let s = self.src_arena.first_child(*src).unwrap();
    //         let l = self
    //             .compressed_node_store
    //             .get_node_at_id(&self.src_arena.original(*src))
    //             .child_count();
    //         s..s + cast::<_, IdD>(l).unwrap()
    //     }
    //     .collect::<Vec<IdD>>()
    // }

    // fn dst_children(&self, dst: &IdD) -> Vec<IdD> {
    //     {
    //         let s = self.dst_arena.first_child(*dst).unwrap();
    //         let l = self
    //             .compressed_node_store
    //             .get_node_at_id(&self.dst_arena.original(*dst))
    //             .child_count();
    //         s..s + cast::<_, IdD>(l).unwrap()
    //     }
    //     .collect::<Vec<IdD>>()
    // }

    // fn histogramMaking(
    //     &self,
    //     src: <Self as Matcher<'a,D,T,I,S>>::Ele,
    //     dst: <Self as Matcher<'a,D,T,I,S>>::Ele,
    // ) -> (
    //     HashMap<Type, Vec<<Self as Matcher<'a,D,T,I,S>>::Ele>>,
    //     HashMap<Type, Vec<<Self as Matcher<'a,D,T,I,S>>::Ele>>,
    // ) {
    //     let srcChildren = self.src_children(&src); //self.src_arena[src as usize].get_children();
    //     let dstChildren = self.dst_children(&dst); //self.dst_arena[dst as usize].get_children();

    //     // let a: EnumMap<Type, Vec<_>> = Default::default();
    //     let mut srcHistogram: HashMap<Type, Vec<<Self as Matcher<'a,D,T,I,S>>::Ele>> =
    //         HashMap::new(); //Map<Type, List<ITree>>
    //     for c in srcChildren {
    //         let t = &self.src_arena[*c as usize].get_type();
    //         if !srcHistogram.contains_key(t) {
    //             srcHistogram.insert(*t, vec![]);
    //         }
    //         srcHistogram.get_mut(t).unwrap().push(*c);
    //     }

    //     let mut dstHistogram: HashMap<<T as Typed>::Type, Vec<<Self as Matcher<'a,D,T,I,S>>::Ele>> =
    //         HashMap::new(); //Map<Type, List<ITree>>
    //     for c in dstChildren {
    //         todo!()
    //         //     //    if (!dstHistogram.containsKey(c.getType()))
    //         //     //        dstHistogram.put(c.getType(), new ArrayList<>());
    //         //     //    dstHistogram.get(c.getType()).add(c);
    //     }
    //     (srcHistogram, dstHistogram)
    // }

    // fn histogramMatching(
    //     &mut self,
    //     // (srcHistogram,dstHistogram):(
    //     // HashMap<<T as Typed>::Type, Vec<<Self as Matcher>::Ele>>,
    //     // HashMap<<T as Typed>::Type, Vec<<Self as Matcher>::Ele>>,
    //     // ),
    //     src: <Self as Matcher<'a, D, T, S>>::Ele,
    //     dst: <Self as Matcher<'a, D, T, S>>::Ele,
    // ) {
    //     let mut src_histogram: HashMap<
    //         <T as Typed>::Type,
    //         Vec<<Self as Matcher<'a, D, T, S>>::Ele>,
    //     > = HashMap::new(); //Map<Type, List<ITree>>
    //     for c in self.src_arena.children(self.compressed_node_store, src) {
    //         let t = &self
    //             .compressed_node_store
    //             .get_node_at_id(&self.src_arena.original(c))
    //             .get_type();
    //         if !src_histogram.contains_key(t) {
    //             src_histogram.insert(*t, vec![]);
    //         }
    //         src_histogram.get_mut(t).unwrap().push(c);
    //     }

    //     let mut dst_histogram: HashMap<
    //         <T as Typed>::Type,
    //         Vec<<Self as Matcher<'a, D, T, S>>::Ele>,
    //     > = HashMap::new(); //Map<Type, List<ITree>>
    //     for c in self.dst_arena.children(self.compressed_node_store, dst) {
    //         let t = &self
    //             .compressed_node_store
    //             .get_node_at_id(&self.dst_arena.original(c))
    //             .get_type();
    //         if !dst_histogram.contains_key(t) {
    //             dst_histogram.insert(*t, vec![]);
    //         }
    //         dst_histogram.get_mut(t).unwrap().push(c);
    //     }
    //     for t in src_histogram.keys() {
    //         if dst_histogram.contains_key(t)
    //             && src_histogram[t].len() == 1
    //             && dst_histogram[t].len() == 1
    //         {
    //             let t1 = src_histogram[t][0];
    //             let t2 = dst_histogram[t][0];
    //             if self.mappings.link_if_both_unmapped(t1, t2) {
    //                 self.lastChanceMatch(t1, t2);
    //             }
    //         }
    //     }
    //     todo!()
    // }
}
