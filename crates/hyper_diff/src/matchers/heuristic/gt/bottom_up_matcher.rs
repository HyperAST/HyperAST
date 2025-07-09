use crate::decompressed_tree_store::{DecompressedTreeStore, DecompressedWithParent};
use crate::matchers::{Mapper, mapping_store::MonoMappingStore};
use crate::utils::sequence_algorithms::longest_common_subsequence;
use hyperast::PrimInt;
use hyperast::types::{
    self, Childrn, HashKind, HyperAST, Labeled, NodeId, NodeStore, TypeStore, WithChildren,
    WithHashs,
};
use num_traits::ToPrimitive;
use std::{collections::HashMap, hash::Hash};

impl<
    Dsrc: DecompressedTreeStore<HAST, M::Src> + DecompressedWithParent<HAST, M::Src>,
    Ddst: DecompressedTreeStore<HAST, M::Dst> + DecompressedWithParent<HAST, M::Dst>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore,
> Mapper<HAST, Dsrc, Ddst, M>
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

        let t = self.hyperast.resolve_type(s);
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
                if self.hyperast.resolve_type(p) == t
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
> Mapper<HAST, Dsrc, Ddst, M>
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
            if self.hyperast.resolve_type(
                &self
                    .src_arena
                    .original(&self.src_arena.parent(src).unwrap()),
            ) == self.hyperast.resolve_type(
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

    pub(super) fn lcs_matching<F: Fn(&Self, &M::Src, &M::Dst) -> bool>(
        &mut self,
        src: &M::Src,
        dst: &M::Dst,
        cmp: F,
    ) {
        let src_children = self
            .src_arena
            .children(src)
            .into_iter()
            .filter(|x| !self.mappings.is_src(x))
            .collect::<Vec<_>>();
        let dst_children = self
            .dst_arena
            .children(dst)
            .into_iter()
            .filter(|x| !self.mappings.is_dst(x))
            .collect::<Vec<_>>();

        let lcs = longest_common_subsequence::<_, _, usize, _>(
            &src_children,
            &dst_children,
            |src, dst| cmp(self, src, dst),
        );
        for x in lcs {
            let t1 = src_children.get(x.0).unwrap();
            let t2 = dst_children.get(x.1).unwrap();
            if self.are_srcs_unmapped(t1) && self.are_dsts_unmapped(t2) {
                self.add_mapping_recursively(t1, t2);
            }
        }
    }

    pub(crate) fn add_mapping_recursively(&mut self, src: &M::Src, dst: &M::Dst) {
        self.mappings.link(*src, *dst);
        self.src_arena
            .descendants(src)
            .iter()
            .zip(self.dst_arena.descendants(dst).iter())
            .for_each(|(src, dst)| self.mappings.link(*src, *dst));
    }

    pub(crate) fn isomorphic<const STRUCTURAL: bool>(&self, src: &M::Src, dst: &M::Dst) -> bool {
        let src = self.src_arena.original(src);
        let dst = self.dst_arena.original(dst);

        self.isomorphic_aux2::<true, STRUCTURAL>(&src, &dst)
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
            let t = &self.hyperast.resolve_type(&self.src_arena.original(&c));
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
                self.mappings.link(t1, t2);
                self.last_chance_match_histogram(&t1, &t2);
            }
        }
    }
}

impl<Dsrc, Ddst, HAST: HyperAST + Copy, M: MonoMappingStore>
    crate::matchers::Mapper<HAST, Dsrc, Ddst, M>
where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithHashs,
    HAST::IdN: Clone + Eq,
    HAST::Label: Eq,
    M::Src: Copy,
    M::Dst: Copy,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
{
    /// if H then test the hash otherwise do not test it,
    /// considering hash colisions testing it should only be useful once.
    pub(crate) fn isomorphic_aux<const H: bool>(
        stores: HAST,
        src: &HAST::IdN,
        dst: &HAST::IdN,
    ) -> bool {
        use types::Tree;
        if src == dst {
            return true;
        }
        let src = stores.node_store().resolve(src);
        let dst = stores.node_store().resolve(dst);
        if H {
            let src_h = WithHashs::hash(&src, &<HAST::RT as WithHashs>::HK::label());
            let dst_h = WithHashs::hash(&dst, &<HAST::RT as WithHashs>::HK::label());
            if src_h != dst_h {
                return false;
            }
        };
        if !stores.type_eq(&src, &dst) {
            return false;
        }
        if dst.has_label() && src.has_label() {
            if src.get_label_unchecked() != dst.get_label_unchecked() {
                return false;
            }
        } else if dst.has_label() || src.has_label() {
            return false;
        };

        if src.child_count() != dst.child_count() {
            return false;
        }
        if !src.has_children() {
            return true;
        }
        let r = match (src.children(), dst.children()) {
            (None, None) => true,
            (Some(src_c), Some(dst_c)) => {
                for (src, dst) in src_c.iter_children().zip(dst_c.iter_children()) {
                    if !Self::isomorphic_aux::<false>(stores, &src, &dst) {
                        return false;
                    }
                }
                true
            }
            _ => false,
        };
        r
    }

    pub(crate) fn isomorphic_aux2<const USE_HASH: bool, const STRUCTURAL: bool>(
        &self,
        src: &HAST::IdN,
        dst: &HAST::IdN,
    ) -> bool {
        if src == dst {
            return true;
        }

        let _src = self.hyperast.node_store().resolve(src);
        let _dst = self.hyperast.node_store().resolve(dst);
        if USE_HASH {
            let src_hash = WithHashs::hash(&_src, &HashKind::label());
            let dst_hash = WithHashs::hash(&_dst, &HashKind::label());
            if src_hash != dst_hash {
                return false;
            }
        }

        let src_type = self.hyperast.resolve_type(&src);
        let dst_type = self.hyperast.resolve_type(&dst);
        if src_type != dst_type {
            return false;
        }

        if !STRUCTURAL {
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
                        if !self.isomorphic_aux2::<false, STRUCTURAL>(src, dst) {
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
