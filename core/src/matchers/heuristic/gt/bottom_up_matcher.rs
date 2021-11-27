use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

use num_traits::{zero, PrimInt, ToPrimitive};

use crate::{
    matchers::{
        decompressed_tree_store::{DecompressedTreeStore, DecompressedWithParent},
        mapping_store::{DefaultMappingStore, MappingStore, MonoMappingStore},
    },
    tree::tree::{HashKind, NodeStore, Tree, Typed, WithHashs},
    utils::sequence_algorithms::longest_common_subsequence,
};

// use super::{decompressed_tree_store::DecompressedTreeStore, mapping_store::DefaultMappingStore, matcher::Matcher, similarity_metrics};

pub struct BottomUpMatcher<
    'a,
    D: DecompressedTreeStore<T::TreeId, IdD> + DecompressedWithParent<IdD>,
    IdD: PrimInt + Into<usize> + std::ops::SubAssign + Debug,
    T: Tree + WithHashs,
    S: for<'b> NodeStore<'b, T>,
    // const SIM_THRESHOLD: u64 = (0.4).bytes(),
> {
    pub(super) node_store: &'a S,
    pub(crate) src_arena: D,
    pub(crate) dst_arena: D,
    pub(crate) mappings: DefaultMappingStore<IdD>,
    pub(super) phantom: PhantomData<*const T>,
}

impl<
        'a,
        D: 'a + DecompressedTreeStore<T::TreeId, IdD> + DecompressedWithParent<IdD>,
        IdD: PrimInt + Into<usize> + std::ops::SubAssign + Debug,
        T: Tree + WithHashs,
        S: for<'b> NodeStore<'b, T>,
    > BottomUpMatcher<'a, D, IdD, T, S>
{
    pub(super) fn getDstCandidates(&self, src: &IdD) -> Vec<IdD> {
        let mut seeds = vec![];
        let s = &self.src_arena.original(src);
        for c in self.src_arena.descendants(self.node_store, src) {
            if self.mappings.is_src(&c) {
                let m = self.mappings.get_dst(&c);
                seeds.push(m);
            }
        }
        let mut candidates = vec![];
        let mut visited: Vec<bool> = vec![false; self.dst_arena.len()];
        for mut seed in seeds {
            while seed != zero() {
                let parent = if let Some(p) = self.dst_arena.parent(&seed) {
                    p
                } else {
                    break;
                };
                if visited[parent.to_usize().unwrap()] {
                    break;
                }
                visited[parent.to_usize().unwrap()] = true;
                let p = &self.dst_arena.original(&parent);
                if self.node_store.resolve(p).get_type() == self.node_store.resolve(s).get_type()
                    && !(self.mappings.is_dst(&parent) || parent == self.dst_arena.root())
                {
                    candidates.push(parent);
                }
                seed = parent;
            }
        }
        candidates
    }

    pub(super) fn last_chance_match_histogram(&mut self, src: &IdD, dst: &IdD) {
        self.lcs_equal_matching(src, dst);
        self.lcs_structure_matching(src, dst);
        if src != &zero() && dst != &zero() {
            self.histogram_matching(src, dst); //self.histogramMaking(src, dst),
        } else if !(src == &zero() || dst == &zero()) {
            if self
                .node_store
                .resolve(
                    &self
                        .src_arena
                        .original(&self.src_arena.parent(src).unwrap()),
                )
                .get_type()
                == self
                    .node_store
                    .resolve(
                        &self
                            .dst_arena
                            .original(&self.dst_arena.parent(dst).unwrap()),
                    )
                    .get_type()
            {
                self.histogram_matching(src, dst) //self.histogramMaking(src, dst),
            }
        }
    }

    // pub(crate) fn last_chance_match_zs<LS:LabelStore<I = T::Label>,ZsS:ZsStore<T::TreeId,IdD>+DecompressedTreeStore<T::TreeId,IdD>,const SIZE_THRESHOLD: usize>(&mut self, label_store:&LS, src: IdD, dst: IdD) {

    //     let x = self.src_arena.original(src);

    //     let y = self.dst_arena.original(dst);

    //     if size(self.compressed_node_store, x) < SIZE_THRESHOLD
    //             || size(self.compressed_node_store, y) < SIZE_THRESHOLD {
    //         let mappings = DefaultMappingStore::<IdD>::new();
    //         let matcher = ZsMatcher::<'a,
    //             ZsS,_,_,_,_
    //         >::matchh(
    //             self.compressed_node_store, label_store, &x, &y, mappings);
    //         // Matcher m = new ZsMatcher();
    //         // MappingStore zsMappings = m.match(src, dst, new MappingStore(src, dst));
    //         // for (Mapping candidate : zsMappings) {
    //         //     ITree srcCand = candidate.first;
    //         //     ITree dstCand = candidate.second;
    //         //     if (mappings.isMappingAllowed(srcCand, dstCand))
    //         //         mappings.addMapping(srcCand, dstCand);
    //         // }
    //     }
    //     todo!() // take inspiration from simple but not same as it needs zsmatcher
    // }

    pub(super) fn are_srcs_unmapped(&self, src: &IdD) -> bool {
        // look at descendants
        // in mappings
        self.src_arena
            .descendants(self.node_store, src)
            .iter()
            .all(|x| !self.mappings.is_src(x))
    }
    pub(super) fn are_dsts_unmapped(&self, dst: &IdD) -> bool {
        // look at descendants
        // in mappings
        self.dst_arena
            .descendants(self.node_store, dst)
            .iter()
            .all(|x| !self.mappings.is_src(x))
    }

    pub(crate) fn add_mapping_recursively(&mut self, src: &IdD, dst: &IdD) {
        self.src_arena
            .descendants(self.node_store, src)
            .iter()
            .zip(self.dst_arena.descendants(self.node_store, dst).iter())
            .for_each(|(src, dst)| self.mappings.link(*src, *dst));
    }

    pub(super) fn lcs_matching<F: Fn(&Self, &IdD, &IdD) -> bool>(
        &mut self,
        src: &IdD,
        dst: &IdD,
        cmp: F,
    ) {
        let src_children = &self.src_arena.children(self.node_store, src);
        let dst_children = &self.dst_arena.children(self.node_store, dst);

        // self.compressed_node_store
        // .get_node_at_id(&self.src_arena.original(src));
        let lcs =
            longest_common_subsequence::<_, usize, _>(src_children, dst_children, |src, dst| {
                cmp(self, src, dst)
            });
        for x in lcs {
            let t1 = src_children.get(x.0).unwrap();
            let t2 = dst_children.get(x.1).unwrap();
            if self.are_srcs_unmapped(t1) && self.are_dsts_unmapped(t2) {
                self.add_mapping_recursively(t1, t2);
            }
        }
    }

    fn lcs_hash_matching(&mut self, h: &T::HK, src: &IdD, dst: &IdD) {
        // todo with longestCommonSubsequenceWithIsomorphism
        self.lcs_matching(src, dst, |s, src, dst| {
            let a = {
                let r = s.node_store.resolve(&s.src_arena.original(src));
                let h = r.hash(h);
                h
            };
            let b = {
                let r = s.node_store.resolve(&s.dst_arena.original(dst));
                let h = r.hash(h);
                h
            };
            a == b
        })
    }

    pub(super) fn lcs_equal_matching(&mut self, src: &IdD, dst: &IdD) {
        self.lcs_hash_matching(&HashKind::label(), src, dst)
    }

    pub(super) fn lcs_structure_matching(&mut self, src: &IdD, dst: &IdD) {
        self.lcs_hash_matching(&HashKind::structural(), src, dst)
    }

    pub(super) fn histogram_matching(&mut self, src: &IdD, dst: &IdD) {
        let mut src_histogram: HashMap<<T as Typed>::Type, Vec<IdD>> = HashMap::new(); //Map<Type, List<ITree>>
        for c in self.src_arena.children(self.node_store, src) {
            let t = &self
                .node_store
                .resolve(&self.src_arena.original(&c))
                .get_type();
            if !src_histogram.contains_key(t) {
                src_histogram.insert(*t, vec![]);
            }
            src_histogram.get_mut(t).unwrap().push(c);
        }

        let mut dst_histogram: HashMap<<T as Typed>::Type, Vec<IdD>> = HashMap::new(); //Map<Type, List<ITree>>
        for c in self.dst_arena.children(self.node_store, dst) {
            let t = &self
                .node_store
                .resolve(&self.dst_arena.original(&c))
                .get_type();
            if !dst_histogram.contains_key(t) {
                dst_histogram.insert(*t, vec![]);
            }
            dst_histogram.get_mut(t).unwrap().push(c);
        }
        for t in src_histogram.keys() {
            if dst_histogram.contains_key(t)
                && src_histogram[t].len() == 1
                && dst_histogram[t].len() == 1
            {
                let t1 = src_histogram[t][0];
                let t2 = dst_histogram[t][0];
                if self.mappings.link_if_both_unmapped(t1, t2) {
                    self.last_chance_match_histogram(&t1, &t2);
                }
            }
        }
        todo!()
    }

    // pub(super) fn src_children(&self, src: &IdD) -> Vec<IdD> {
    //     Self::get_children(self.compressed_node_store, &self.src_arena, src)
    // }

    // pub(super) fn dst_children(&self, dst: &IdD) -> Vec<IdD> {
    //     Self::get_children(self.compressed_node_store, &self.dst_arena, dst)
    // }

    // pub(super) fn get_children(store: &S, arena: &D, dst: &IdD) -> Vec<IdD> {
    //     let s = arena.first_child(*dst).unwrap();
    //     let s: usize = cast(s).unwrap();
    //     let l = store.get_node_at_id(&arena.original(*dst)).child_count();
    //     let r = s..s + cast::<_, usize>(l).unwrap();
    //     r.into_iter().map(|x| cast(x).unwrap()).collect()
    // }

    // pub(super) fn getDescendants_decompressedG(&self, arena: &D, x: IdD) -> Vec<IdD> {
    //     // todo possible opti by also making descendants contigous in arena
    //     let mut id: Vec<IdD> = vec![x];
    //     let mut id_compressed: Vec<T::TreeId> = vec![arena.original(x)];
    //     let mut i: usize = cast(x).unwrap();

    //     while i < id.len() {
    //         let node = self.compressed_node_store.get_node_at_id(&id_compressed[i]);
    //         let l = node.get_children();
    //         id_compressed.extend_from_slice(l);

    //         i += 1;
    //     }
    //     id
    // }
}
