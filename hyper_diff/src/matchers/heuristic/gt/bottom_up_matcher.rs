use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData};

use num_traits::{PrimInt, ToPrimitive, Zero};

use crate::{
    decompressed_tree_store::{DecompressedTreeStore, DecompressedWithParent, Shallow},
    matchers::mapping_store::MonoMappingStore,
    utils::sequence_algorithms::longest_common_subsequence,
};
use hyper_ast::types::{HashKind, NodeStore, Tree, TypeStore, WithHashs};

pub struct BottomUpMatcher<'a, Dsrc, Ddst, T, HAST, M> {
    pub(super) stores: &'a HAST,
    pub src_arena: Dsrc,
    pub dst_arena: Ddst,
    pub mappings: M,
    pub(super) _phantom: PhantomData<*const T>,
}

impl<
        'a,
        Dsrc: DecompressedTreeStore<'a, T, M::Src> + DecompressedWithParent<'a, T, M::Src>,
        Ddst: DecompressedTreeStore<'a, T, M::Dst> + DecompressedWithParent<'a, T, M::Dst>,
        T: 'a + Tree,
        HAST: HyperAST<'a, IdN = T::TreeId, T = T>,
        M: MonoMappingStore,
    > BottomUpMatcher<'a, Dsrc, Ddst, T, HAST, M>
where
    // T::Type: Eq + Copy + Send + Sync,
    M::Src: PrimInt + std::ops::SubAssign + Debug,
    M::Dst: PrimInt + std::ops::SubAssign + Debug,
{
    pub(super) fn get_dst_candidates(&self, src: &M::Src) -> Vec<M::Dst> {
        let mut seeds = vec![];
        let s = &self.src_arena.original(src);
        for c in self.src_arena.descendants(self.stores.node_store(), src) {
            if self.mappings.is_src(&c) {
                let m = self.mappings.get_dst_unchecked(&c);
                seeds.push(m);
            }
        }
        let mut candidates = vec![];
        let mut visited = bitvec::bitbox![0;self.dst_arena.len()];

        let t = self.stores.resolve_type(s);
        for mut seed in seeds {
            loop {
                let Some(parent) = self.dst_arena.parent(&seed) else {
                    break;
                };
                if visited[parent.to_usize().unwrap()] {
                    break;
                }
                visited.set(parent.to_usize().unwrap(), true);
                let p = &self.dst_arena.original(&parent);
                if self.stores.resolve_type(p) == t
                    && !(self.mappings.is_dst(&parent) || parent == self.dst_arena.root())
                {
                    candidates.push(parent);
                }
                seed = parent;
            }
        }
        candidates
    }
}

impl<
        'a,
        Dsrc: DecompressedTreeStore<'a, T, M::Src> + DecompressedWithParent<'a, T, M::Src>,
        Ddst: DecompressedTreeStore<'a, T, M::Dst> + DecompressedWithParent<'a, T, M::Dst>,
        T: 'a + Tree + WithHashs,
        HAST: HyperAST<'a, IdN = T::TreeId, T = T>,
        M: MonoMappingStore,
    > BottomUpMatcher<'a, Dsrc, Ddst, T, HAST, M>
where
    <HAST::TS as TypeStore<T>>::Ty: Copy + Send + Sync + Eq + Hash,
    M::Src: PrimInt + std::ops::SubAssign + Debug,
    M::Dst: PrimInt + std::ops::SubAssign + Debug,
{
    pub(super) fn last_chance_match_histogram(&mut self, src: &M::Src, dst: &M::Dst) {
        self.lcs_equal_matching(src, dst);
        self.lcs_structure_matching(src, dst);
        if !src.is_zero() && !dst.is_zero() {
            self.histogram_matching(src, dst); //self.histogramMaking(src, dst),
        } else if !(src.is_zero() || dst.is_zero()) {
            if self.stores.resolve_type(
                &self
                    .src_arena
                    .original(&self.src_arena.parent(src).unwrap()),
            ) == self.stores.resolve_type(
                &self
                    .dst_arena
                    .original(&self.dst_arena.parent(dst).unwrap()),
            ) {
                self.histogram_matching(src, dst) //self.histogramMaking(src, dst),
            }
        }
    }

    pub(super) fn are_srcs_unmapped(&self, src: &M::Src) -> bool {
        // look at descendants
        // in mappings
        self.src_arena
            .descendants(self.stores.node_store(), src)
            .iter()
            .all(|x| !self.mappings.is_src(x))
    }
    pub(super) fn are_dsts_unmapped(&self, dst: &M::Dst) -> bool {
        // look at descendants
        // in mappings
        self.dst_arena
            .descendants(self.stores.node_store(), dst)
            .iter()
            .all(|x| !self.mappings.is_dst(x))
    }

    pub(crate) fn add_mapping_recursively(&mut self, src: &M::Src, dst: &M::Dst) {
        self.src_arena
            .descendants(self.stores.node_store(), src)
            .iter()
            .zip(
                self.dst_arena
                    .descendants(self.stores.node_store(), dst)
                    .iter(),
            )
            .for_each(|(src, dst)| self.mappings.link(*src, *dst));
    }

    pub(super) fn lcs_matching<F: Fn(&Self, &M::Src, &M::Dst) -> bool>(
        &mut self,
        src: &M::Src,
        dst: &M::Dst,
        cmp: F,
    ) {
        let src_children = &self.src_arena.children(self.stores.node_store(), src);
        let dst_children = &self.dst_arena.children(self.stores.node_store(), dst);

        // self.compressed_node_store
        // .get_node_at_id(&self.src_arena.original(src));
        let lcs =
            longest_common_subsequence::<_, _, usize, _>(src_children, dst_children, |src, dst| {
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

    fn lcs_hash_matching(&mut self, h: &T::HK, src: &M::Src, dst: &M::Dst) {
        // todo with longestCommonSubsequenceWithIsomorphism
        self.lcs_matching(src, dst, |s, src, dst| {
            let a = {
                let r = s.stores.node_store().resolve(&s.src_arena.original(src));
                let h = r.hash(h);
                h
            };
            let b = {
                let r = s.stores.node_store().resolve(&s.dst_arena.original(dst));
                let h = r.hash(h);
                h
            };
            a == b
        })
    }

    pub(super) fn lcs_equal_matching(&mut self, src: &M::Src, dst: &M::Dst) {
        self.lcs_hash_matching(&HashKind::label(), src, dst)
    }

    pub(super) fn lcs_structure_matching(&mut self, src: &M::Src, dst: &M::Dst) {
        self.lcs_hash_matching(&HashKind::structural(), src, dst)
    }

    pub(super) fn histogram_matching(&mut self, src: &M::Src, dst: &M::Dst) {
        let mut src_histogram: HashMap<<HAST::TS as TypeStore<T>>::Ty, Vec<M::Src>> =
            HashMap::new(); //Map<Type, List<ITree>>
        for c in self.src_arena.children(self.stores.node_store(), src) {
            let t = &self.stores.type_store().resolve_type(
                &self
                    .stores
                    .node_store()
                    .resolve(&self.src_arena.original(&c)),
            );
            if !src_histogram.contains_key(t) {
                src_histogram.insert(*t, vec![]);
            }
            src_histogram.get_mut(t).unwrap().push(c);
        }

        let mut dst_histogram: HashMap<<HAST::TS as TypeStore<T>>::Ty, Vec<M::Dst>> =
            HashMap::new(); //Map<Type, List<ITree>>
        for c in self.dst_arena.children(self.stores.node_store(), dst) {
            let t = &self.stores.resolve_type(&self.dst_arena.original(&c));
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
}

use hyper_ast::types::HyperAST;

impl<
        'a,
        HAST: HyperAST<'a>,
        Dsrc: DecompressedTreeStore<'a, HAST::T, M::Src> + DecompressedWithParent<'a, HAST::T, M::Src>,
        Ddst: DecompressedTreeStore<'a, HAST::T, M::Dst> + DecompressedWithParent<'a, HAST::T, M::Dst>,
        M: MonoMappingStore,
    > crate::matchers::Mapper<'a, HAST, Dsrc, Ddst, M>
where
    HAST::T: 'a + Tree,
    // <HAST::T as Typed>::Type: Eq + Copy + Send + Sync,
    M::Src: PrimInt + std::ops::SubAssign + Debug,
    M::Dst: PrimInt + std::ops::SubAssign + Debug,
{
    pub(super) fn get_dst_candidates(&self, src: &M::Src) -> Vec<M::Dst> {
        let node_store = self.hyperast.node_store();
        let src_arena = &self.mapping.src_arena;
        let dst_arena = &self.mapping.dst_arena;
        let mappings = &self.mapping.mappings;
        let mut seeds = vec![];
        let s = &src_arena.original(src);
        for c in src_arena.descendants(node_store, src) {
            if mappings.is_src(&c) {
                let m = mappings.get_dst_unchecked(&c);
                seeds.push(m);
            }
        }
        let mut candidates = vec![];
        let mut visited = bitvec::bitbox![0;dst_arena.len()];
        let t = self.hyperast.resolve_type(s);
        for mut seed in seeds {
            loop {
                let Some(parent) = dst_arena.parent(&seed) else {
                    break;
                };
                if visited[parent.to_usize().unwrap()] {
                    break;
                }
                visited.set(parent.to_usize().unwrap(), true);
                let p = &dst_arena.original(&parent);
                if self.hyperast.resolve_type(p) == t
                    && !(mappings.is_dst(&parent) || parent == dst_arena.root())
                {
                    candidates.push(parent);
                }
                seed = parent;
            }
        }
        candidates
    }
}

impl<
        'a,
        HAST: 'a + HyperAST<'a>,
        Dsrc: DecompressedTreeStore<'a, HAST::T, M::Src> + DecompressedWithParent<'a, HAST::T, M::Src>,
        Ddst: DecompressedTreeStore<'a, HAST::T, M::Dst> + DecompressedWithParent<'a, HAST::T, M::Dst>,
        M: MonoMappingStore,
    > crate::matchers::Mapper<'a, HAST, Dsrc, Ddst, M>
where
    HAST::T: 'a + Tree + WithHashs,
    <HAST::TS as TypeStore<HAST::T>>::Ty: Copy + Send + Sync + Eq + Hash,
    M::Src: PrimInt + std::ops::SubAssign + Debug,
    M::Dst: PrimInt + std::ops::SubAssign + Debug,
    M::Src: Shallow<M::Src>,
    M::Dst: Shallow<M::Dst>,
{
    pub(super) fn last_chance_match_histogram(&mut self, src: &M::Src, dst: &M::Dst) {
        self.lcs_equal_matching(src, dst);
        self.lcs_structure_matching(src, dst);
        if !src.is_zero() && !dst.is_zero() {
            self.histogram_matching(src, dst); //self.histogramMaking(src, dst),
        } else if !(src.is_zero() || dst.is_zero()) {
            if self.hyperast.resolve_type(
                &self
                    .src_arena
                    .original(&self.src_arena.parent(src).unwrap()),
            ) == self.hyperast.resolve_type(
                &self
                    .dst_arena
                    .original(&self.dst_arena.parent(dst).unwrap()),
            ) {
                self.histogram_matching(src, dst) //self.histogramMaking(src, dst),
            }
        }
    }

    pub(super) fn are_srcs_unmapped(&self, src: &M::Src) -> bool {
        // look at descendants
        // in mappings
        self.src_arena
            .descendants(self.hyperast.node_store(), src)
            .iter()
            .all(|x| !self.mappings.is_src(x))
    }
    pub(super) fn are_dsts_unmapped(&self, dst: &M::Dst) -> bool {
        // look at descendants
        // in mappings
        self.dst_arena
            .descendants(self.hyperast.node_store(), dst)
            .iter()
            .all(|x| !self.mappings.is_dst(x))
    }

    // pub(crate) fn add_mapping_recursively(&mut self, src: &M::Src, dst: &M::Dst) {
    //     self.src_arena
    //         .descendants(self.hyperast.node_store(), src)
    //         .iter()
    //         .zip(self.dst_arena.descendants(self.hyperast.node_store(), dst).iter())
    //         .for_each(|(src, dst)| self.mappings.link(*src, *dst));
    // }

    pub(super) fn lcs_matching<F: Fn(&Self, &M::Src, &M::Dst) -> bool>(
        &mut self,
        src: &M::Src,
        dst: &M::Dst,
        cmp: F,
    ) {
        let src_children = &self.src_arena.children(self.hyperast.node_store(), src);
        let dst_children = &self.dst_arena.children(self.hyperast.node_store(), dst);

        // self.compressed_node_store
        // .get_node_at_id(&self.src_arena.original(src));
        let lcs =
            longest_common_subsequence::<_, _, usize, _>(src_children, dst_children, |src, dst| {
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

    fn lcs_hash_matching(&mut self, h: &<HAST::T as WithHashs>::HK, src: &M::Src, dst: &M::Dst) {
        // todo with longestCommonSubsequenceWithIsomorphism
        self.lcs_matching(src, dst, |s, src, dst| {
            let a = {
                let r = s.hyperast.node_store().resolve(&s.src_arena.original(src));
                let h = r.hash(h);
                h
            };
            let b = {
                let r = s.hyperast.node_store().resolve(&s.dst_arena.original(dst));
                let h = r.hash(h);
                h
            };
            a == b
        })
    }

    pub(super) fn lcs_equal_matching(&mut self, src: &M::Src, dst: &M::Dst) {
        self.lcs_hash_matching(&HashKind::label(), src, dst)
    }

    pub(super) fn lcs_structure_matching(&mut self, src: &M::Src, dst: &M::Dst) {
        self.lcs_hash_matching(&HashKind::structural(), src, dst)
    }

    pub(super) fn histogram_matching(&mut self, src: &M::Src, dst: &M::Dst) {
        let mut src_histogram: HashMap<<HAST::TS as TypeStore<HAST::T>>::Ty, Vec<M::Src>> =
            HashMap::new(); //Map<Type, List<ITree>>
        for c in self.src_arena.children(self.hyperast.node_store(), src) {
            let t = &self.hyperast.resolve_type(&self.src_arena.original(&c));
            if !src_histogram.contains_key(t) {
                src_histogram.insert(*t, vec![]);
            }
            src_histogram.get_mut(t).unwrap().push(c);
        }

        let mut dst_histogram: HashMap<<HAST::TS as TypeStore<HAST::T>>::Ty, Vec<M::Dst>> =
            HashMap::new(); //Map<Type, List<ITree>>
        for c in self.dst_arena.children(self.hyperast.node_store(), dst) {
            let t = &self.hyperast.resolve_type(&self.dst_arena.original(&c));
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
}
