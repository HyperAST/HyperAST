use crate::{
    decompressed_tree_store::{DecompressedTreeStore, DecompressedWithParent, Shallow},
    matchers::mapping_store::MonoMappingStore,
    utils::sequence_algorithms::longest_common_subsequence,
};
use hyperast::PrimInt;
use hyperast::types::{
    self, Childrn, DecompressedFrom, HashKind, HyperAST, HyperASTShared, Labeled, NodeId,
    NodeStore, Tree, TypeStore, WithChildren, WithHashs,
};
use num_traits::{ToPrimitive, Zero};
use std::{collections::HashMap, hash::Hash};

pub struct BottomUpMatcher<Dsrc, Ddst, HAST, M> {
    pub(in crate::matchers) stores: HAST,
    pub src_arena: Dsrc,
    pub dst_arena: Ddst,
    pub mappings: M,
}

impl<
    Dsrc: DecompressedTreeStore<HAST, M::Src> + DecompressedWithParent<HAST, M::Src>,
    Ddst: DecompressedTreeStore<HAST, M::Dst> + DecompressedWithParent<HAST, M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore,
> BottomUpMatcher<Dsrc, Ddst, HAST, M>
where
    M::Src: PrimInt,
    M::Dst: PrimInt,
{
    pub(in crate::matchers) fn get_dst_candidates(&self, src: &M::Src) -> Vec<M::Dst> {
        let mut seeds = vec![];
        let s = &self.src_arena.original(src);
        for c in self.src_arena.descendants(src) {
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
    Dsrc: DecompressedTreeStore<HAST, M::Src> + DecompressedWithParent<HAST, M::Src>,
    Ddst: DecompressedTreeStore<HAST, M::Dst> + DecompressedWithParent<HAST, M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore,
> BottomUpMatcher<Dsrc, Ddst, HAST, M>
where
    <HAST::TS as TypeStore>::Ty: Copy + Send + Sync + Eq + Hash,
    M::Src: PrimInt,
    M::Dst: PrimInt,
    for<'t> <HAST as types::AstLending<'t>>::RT: WithHashs,
    HAST::Label: Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    pub fn last_chance_match_histogram(&mut self, src: &M::Src, dst: &M::Dst) {
        self.lcs_equal_matching(src, dst);
        self.lcs_structure_matching(src, dst);
        let src_is_root = self.src_arena.parent(&src).is_none();
        let dst_is_root = self.dst_arena.parent(&dst).is_none();
        if src_is_root && dst_is_root {
            self.histogram_matching(src, dst);
        } else if !(src_is_root || dst_is_root) {
            if self.stores.resolve_type(
                &self
                    .src_arena
                    .original(&self.src_arena.parent(src).unwrap()),
            ) == self.stores.resolve_type(
                &self
                    .dst_arena
                    .original(&self.dst_arena.parent(dst).unwrap()),
            ) {
                self.histogram_matching(src, dst)
            }
        }
    }

    pub(super) fn are_srcs_unmapped(&self, src: &M::Src) -> bool {
        // look at descendants in mappings
        self.src_arena
            .descendants(src)
            .iter()
            .all(|x| !self.mappings.is_src(x))
    }
    pub(super) fn are_dsts_unmapped(&self, dst: &M::Dst) -> bool {
        // look at descendants in mappings
        self.dst_arena
            .descendants(dst)
            .iter()
            .all(|x| !self.mappings.is_dst(x))
    }

    pub(super) fn has_unmapped_src_children(&self, src: &M::Src) -> bool {
        // look at descendants in mappings
        self.src_arena
            .descendants(src)
            .iter()
            .any(|x| !self.mappings.is_src(x))
    }
    pub(super) fn has_unmapped_dst_children(&self, dst: &M::Dst) -> bool {
        // look at descendants in mappings
        self.dst_arena
            .descendants(dst)
            .iter()
            .any(|x| !self.mappings.is_dst(x))
    }

    pub(crate) fn add_mapping_recursively(&mut self, src: &M::Src, dst: &M::Dst) {
        self.mappings.link(*src, *dst);
        self.src_arena
            .descendants(src)
            .iter()
            .zip(self.dst_arena.descendants(dst).iter())
            .for_each(|(src, dst)| self.mappings.link(*src, *dst));
    }

    pub(super) fn lcs_matching<F: Fn(&Self, &M::Src, &M::Dst) -> bool>(
        &mut self,
        src: &M::Src,
        dst: &M::Dst,
        cmp: F,
    ) {
        let src_children = &mut Vec::new();
        for c in &self.src_arena.children(src) {
            if !self.mappings.is_src(c) {
                src_children.push(*c);
            }
        }
        let dst_children = &mut Vec::new();
        for c in &self.dst_arena.children(dst) {
            if !self.mappings.is_dst(c) {
                dst_children.push(*c);
            }
        }


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

    pub(crate) fn isomorphic<const structural: bool>(&self, src: &M::Src, dst: &M::Dst) -> bool {
        let src = self.src_arena.original(src);
        let dst = self.dst_arena.original(dst);

        self.isomorphic_aux::<true, structural>(&src, &dst)
    }

    pub(crate) fn isomorphic_aux<const use_hash: bool, const structural: bool>(
        &self,
        src: &HAST::IdN,
        dst: &HAST::IdN,
    ) -> bool {
        if src == dst {
            return true;
        }
        
        let _src = self.stores.node_store().resolve(src);
        let _dst = self.stores.node_store().resolve(dst);
        if use_hash {
            let src_hash = WithHashs::hash(&_src, &HashKind::label());
            let dst_hash = WithHashs::hash(&_dst, &HashKind::label());
            if src_hash != dst_hash {
                return false;
            }
        }

        let src_type = self.stores.resolve_type(&src);
        let dst_type = self.stores.resolve_type(&dst);
        if src_type != dst_type {
            return false;
        }

        if !structural {
            let src_label = _src.try_get_label();
            let dst_label = _dst.try_get_label();
            if src_label != dst_label {
                return false;
            }
        }

        let src_children: Option<Vec<_>> = _src.children().map(|x| x.iter_children().collect());
        let dst_children: Option<Vec<_>> = _dst.children().map(|x| x.iter_children().collect());
        match (src_children, dst_children) {
            (None, None) => true,
            (Some(src_c), Some(dst_c)) => {
                if src_c.len() != dst_c.len() {
                    false
                } else {
                    for (src, dst) in src_c.iter().zip(dst_c.iter()) {
                        if !self.isomorphic_aux::<false, structural>(src, dst) {
                            return false;
                        }
                    }
                    true
                }
            }
            _ => false,
        }
    }

    pub(super) fn lcs_equal_matching(&mut self, src: &M::Src, dst: &M::Dst) {
        self.lcs_matching(src, dst, move |s, src, dst| s.isomorphic::<false>(src, dst))
    }

    pub(super) fn lcs_structure_matching(&mut self, src: &M::Src, dst: &M::Dst) {
        self.lcs_matching(src, dst, move |s, src, dst| s.isomorphic::<true>(src, dst))
    }

    pub(super) fn histogram_matching(&mut self, src: &M::Src, dst: &M::Dst) {
        let mut src_histogram: HashMap<<HAST::TS as TypeStore>::Ty, Vec<M::Src>> = HashMap::new(); //Map<Type, List<ITree>>
        for c in self.src_arena.children(src) {
            if self.mappings.is_src(&c) {
                continue;
            }
            let t = &self.stores.resolve_type(&self.src_arena.original(&c));
            if !src_histogram.contains_key(t) {
                src_histogram.insert(*t, vec![]);
            }
            src_histogram.get_mut(t).unwrap().push(c);
        }

        let mut dst_histogram: HashMap<<HAST::TS as TypeStore>::Ty, Vec<M::Dst>> = HashMap::new(); //Map<Type, List<ITree>>
        for c in self.dst_arena.children(dst) {
            if self.mappings.is_dst(&c) {
                continue;
            }
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
                self.mappings.link(t1, t2);
                self.last_chance_match_histogram(&t1, &t2);
            }
        }
    }
}

// impl<
//     HAST: HyperAST + Copy,
//     Dsrc: DecompressedTreeStore<HAST, M::Src> + DecompressedWithParent<HAST, M::Src>,
//     Ddst: DecompressedTreeStore<HAST, M::Dst> + DecompressedWithParent<HAST, M::Dst>,
//     M: MonoMappingStore,
// > crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
// where
//     M::Src: PrimInt,
//     M::Dst: PrimInt,
// {
//     pub fn get_dst_candidates(&self, src: &M::Src) -> Vec<M::Dst> {
//         let src_arena = &self.mapping.src_arena;
//         let dst_arena = &self.mapping.dst_arena;
//         let mappings = &self.mapping.mappings;
//         let mut seeds = vec![];
//         let s = &src_arena.original(src);
//         for c in src_arena.descendants(src) {
//             if mappings.is_src(&c) {
//                 let m = mappings.get_dst_unchecked(&c);
//                 seeds.push(m);
//             }
//         }
//         let mut candidates = vec![];
//         let mut visited = bitvec::bitbox![0;dst_arena.len()];
//         let t = self.hyperast.resolve_type(s);
//         for mut seed in seeds {
//             loop {
//                 let Some(parent) = dst_arena.parent(&seed) else {
//                     break;
//                 };
//                 if visited[parent.to_usize().unwrap()] {
//                     break;
//                 }
//                 visited.set(parent.to_usize().unwrap(), true);
//                 let p = &dst_arena.original(&parent);
//                 if self.hyperast.resolve_type(p) == t
//                     && !(mappings.is_dst(&parent) || parent == dst_arena.root())
//                 {
//                     candidates.push(parent);
//                 }
//                 seed = parent;
//             }
//         }
//         candidates
//     }
// }
// 
// impl<HAST: HyperAST + Copy, Dsrc, Ddst, M: MonoMappingStore>
//     crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
// where
//     for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
//     <HAST::TS as TypeStore>::Ty: Copy + Send + Sync + Eq + Hash,
//     M::Src: PrimInt + Shallow<M::Src>,
//     M::Dst: PrimInt + Shallow<M::Dst>,
//     Dsrc: DecompressedTreeStore<HAST, M::Src> + DecompressedWithParent<HAST, M::Src>,
//     Ddst: DecompressedTreeStore<HAST, M::Dst> + DecompressedWithParent<HAST, M::Dst>,
// {
//     pub fn last_chance_match_histogram(&mut self, src: &M::Src, dst: &M::Dst) {
//         self.lcs_equal_matching(src, dst);
//         self.lcs_structure_matching(src, dst);
//         if !src.is_zero() && !dst.is_zero() {
//             self.histogram_matching(src, dst); //self.histogramMaking(src, dst),
//         } else if !(src.is_zero() || dst.is_zero()) {
//             if self.hyperast.resolve_type(
//                 &self
//                     .src_arena
//                     .original(&self.src_arena.parent(src).unwrap()),
//             ) == self.hyperast.resolve_type(
//                 &self
//                     .dst_arena
//                     .original(&self.dst_arena.parent(dst).unwrap()),
//             ) {
//                 self.histogram_matching(src, dst) //self.histogramMaking(src, dst),
//             }
//         }
//     }
// 
//     pub(super) fn are_srcs_unmapped(&self, src: &M::Src) -> bool {
//         // look at descendants in mappings
//         self.src_arena
//             .descendants(src)
//             .iter()
//             .all(|x| !self.mappings.is_src(x))
//     }
// 
//     pub(super) fn are_dsts_unmapped(&self, dst: &M::Dst) -> bool {
//         // look at descendants in mappings
//         self.dst_arena
//             .descendants(dst)
//             .iter()
//             .all(|x| !self.mappings.is_dst(x))
//     }
// 
//     pub(super) fn lcs_matching<F: Fn(&Self, &M::Src, &M::Dst) -> bool>(
//         &mut self,
//         src: &M::Src,
//         dst: &M::Dst,
//         cmp: F,
//     ) {
//         let src_children = &self.src_arena.children(src);
//         let dst_children = &self.dst_arena.children(dst);
// 
//         // self.compressed_node_store
//         // .get_node_at_id(&self.src_arena.original(src));
//         let lcs =
//             longest_common_subsequence::<_, _, usize, _>(src_children, dst_children, |src, dst| {
//                 cmp(self, src, dst)
//             });
//         for x in lcs {
//             let t1 = src_children.get(x.0).unwrap();
//             let t2 = dst_children.get(x.1).unwrap();
//             if self.are_srcs_unmapped(t1) && self.are_dsts_unmapped(t2) {
//                 self.add_mapping_recursively(t1, t2);
//             }
//         }
//     }
// 
//     pub(super) fn lcs_equal_matching(&mut self, src: &M::Src, dst: &M::Dst) {
//         self.lcs_matching(src, dst, move |s, src, dst| {
//             let h = HashKind::label();
//             let a = s.hyperast.resolve(&s.mapping.src_arena.original(src));
//             let a = WithHashs::hash(&a, &h);
//             let b = s.hyperast.resolve(&s.mapping.dst_arena.original(dst));
//             let b = WithHashs::hash(&b, &h);
//             a == b
//         })
//     }
// 
//     pub(super) fn lcs_structure_matching(&mut self, src: &M::Src, dst: &M::Dst) {
//         self.lcs_matching(src, dst, move |s, src, dst| {
//             let h = HashKind::structural();
//             let a = s.hyperast.resolve(&s.mapping.src_arena.original(src));
//             let a = WithHashs::hash(&a, &h);
//             let b = s.hyperast.resolve(&s.mapping.dst_arena.original(dst));
//             let b = WithHashs::hash(&b, &h);
//             a == b
//         })
//     }
// 
//     pub(super) fn histogram_matching(&mut self, src: &M::Src, dst: &M::Dst) {
//         let mut src_histogram: HashMap<<HAST::TS as TypeStore>::Ty, Vec<M::Src>> = HashMap::new(); //Map<Type, List<ITree>>
//         let children2: Vec<M::Src> = self.src_arena.children(src);
//         for c in children2 {
//             let t = &self.hyperast.resolve_type(&self.src_arena.original(&c));
//             if !src_histogram.contains_key(t) {
//                 src_histogram.insert(*t, vec![]);
//             }
//             src_histogram.get_mut(t).unwrap().push(c);
//         }
// 
//         let mut dst_histogram: HashMap<<HAST::TS as TypeStore>::Ty, Vec<M::Dst>> = HashMap::new(); //Map<Type, List<ITree>>
//         for c in self.dst_arena.children(dst) {
//             let t = &self.hyperast.resolve_type(&self.dst_arena.original(&c));
//             if !dst_histogram.contains_key(t) {
//                 dst_histogram.insert(*t, vec![]);
//             }
//             dst_histogram.get_mut(t).unwrap().push(c);
//         }
//         for t in src_histogram.keys() {
//             if dst_histogram.contains_key(t)
//                 && src_histogram[t].len() == 1
//                 && dst_histogram[t].len() == 1
//             {
//                 let t1 = src_histogram[t][0];
//                 let t2 = dst_histogram[t][0];
//                 if self.mappings.link_if_both_unmapped(t1, t2) {
//                     self.last_chance_match_histogram(&t1, &t2);
//                 }
//             }
//         }
//         todo!()
//     }
// }
