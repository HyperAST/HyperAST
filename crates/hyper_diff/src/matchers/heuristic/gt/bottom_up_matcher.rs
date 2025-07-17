use crate::decompressed_tree_store::{DecompressedTreeStore, DecompressedWithParent};
use crate::matchers::{Mapper, mapping_store::MonoMappingStore};
use crate::utils::sequence_algorithms::longest_common_subsequence;
use hyperast::PrimInt;
use hyperast::types::{self, HyperAST, NodeId, NodeStore, TypeStore, WithHashs};
use num_traits::ToPrimitive;
use std::{collections::HashMap, hash::Hash};
use types::Tree;

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
            while let Some(parent) = self.dst_arena.parent(&seed) {
                if visited[parent.to_usize().unwrap()] {
                    break;
                }
                visited.set(parent.to_usize().unwrap(), true);

                let p = &self.dst_arena.original(&parent);
                if self.hyperast.resolve_type(p) == t
                    && !self.mappings.is_dst(&parent)
                    && parent != self.dst_arena.root()
                {
                    candidates.push(parent);
                }
                seed = parent;
            }
        }
        candidates
    }

    pub(super) fn get_src_candidates(&self, dst: &M::Dst) -> Vec<M::Src> {
        let mut seeds = vec![];
        let s = &self.dst_arena.original(dst);
        for c in self.dst_arena.descendants(dst) {
            if self.mappings.is_dst(&c) {
                let m = self.mappings.get_src_unchecked(&c);
                seeds.push(m);
            }
        }
        let mut candidates = vec![];
        let mut visited = bitvec::bitbox![0;self.src_arena.len()];

        let t = self.hyperast.resolve_type(s);
        for seed in seeds {
            while let Some(parent) = self.src_arena.parent(&seed) {
                if visited[parent.to_usize().unwrap()] {
                    break;
                }
                visited.set(parent.to_usize().unwrap(), true);

                let p = &self.src_arena.original(&parent);
                if self.hyperast.resolve_type(p) == t
                    && !self.mappings.is_src(&parent)
                    && parent != self.src_arena.root()
                {
                    candidates.push(parent);
                }
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
            let psrc = self.src_arena.parent(src).unwrap();
            let opsrc = self.src_arena.original(&psrc);
            let src_type = self.hyperast.resolve_type(&opsrc);
            let pdst = self.dst_arena.parent(dst).unwrap();
            let opdst = self.dst_arena.original(&pdst);
            let dst_type = self.hyperast.resolve_type(&opdst);
            if src_type == dst_type {
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

    /// Return true if src has *any* children
    pub(super) fn src_has_children(&mut self, src: M::Src) -> bool {
        self.hyperast
            .node_store()
            .resolve(&self.src_arena.original(&src))
            .has_children()
    }

    // Matches all strictly isomorphic nodes in the descendants of src and dst (step 1 of simple recovery)
    pub(super) fn lcs_equal_matching(&mut self, src: &M::Src, dst: &M::Dst) {
        self.lcs_matching(src, dst, move |s, src, dst| s.isomorphic::<false>(src, dst))
        // NOTE the following impl would not be resilent to collisions
        // self.lcs_matching(src, dst, move |s, src, dst| {
        //     let a = s.hyperast.node_store().resolve(&s.src_arena.original(src));
        //     let b = s.hyperast.node_store().resolve(&s.dst_arena.original(dst));

        //     let a = WithHashs::hash(&a, &HashKind::label());
        //     let b = WithHashs::hash(&b, &HashKind::label());
        //     a == b
        // })
    }

    // Matches all structurally isomorphic nodes in the descendants of src and dst (step 2 of simple recovery)
    pub(super) fn lcs_structure_matching(&mut self, src: &M::Src, dst: &M::Dst) {
        self.lcs_matching(src, dst, move |s, src, dst| s.isomorphic::<true>(src, dst))
        // NOTE the following impl would not be resilent to collisions
        // self.lcs_matching(src, dst, move |s, src, dst| {
        //     let a = s.hyperast.node_store().resolve(&s.src_arena.original(src));
        //     let b = s.hyperast.node_store().resolve(&s.dst_arena.original(dst));

        //     let a = WithHashs::hash(&a, &HashKind::structural());
        //     let b = WithHashs::hash(&b, &HashKind::structural());
        //     a == b
        // })
    }

    pub(crate) fn isomorphic<const STRUCTURAL: bool>(&self, src: &M::Src, dst: &M::Dst) -> bool {
        let src = self.src_arena.original(src);
        let dst = self.dst_arena.original(dst);
        super::isomorphic::<_, true, STRUCTURAL>(self.hyperast, &src, &dst)
    }

    pub(super) fn histogram_matching(&mut self, src: &M::Src, dst: &M::Dst) {
        // both src and dst -histogram have type Map<Type, List<ITree>>
        let src_histogram: HashMap<_, Vec<M::Src>> = self
            .src_arena
            .children(src)
            .into_iter()
            .filter(|child| !self.mappings.is_src(&child))
            .fold(HashMap::new(), |mut acc, child| {
                let t = self.hyperast.resolve_type(&self.src_arena.original(&child));
                acc.entry(t).or_insert_with(Vec::new).push(child);
                acc
            });
        let dst_histogram: HashMap<_, Vec<M::Dst>> = self
            .dst_arena
            .children(dst)
            .into_iter()
            .filter(|child| !self.mappings.is_dst(&child))
            .fold(HashMap::new(), |mut acc, child| {
                let t = self.hyperast.resolve_type(&self.dst_arena.original(&child));
                acc.entry(t).or_insert_with(Vec::new).push(child);
                acc
            });

        for t in src_histogram.keys() {
            if dst_histogram.contains_key(t)
                && src_histogram[t].len() == 1
                && dst_histogram[t].len() == 1
            {
                // TODO use an option instead of a vec
                // we are only retrieving the first element anyway,
                // we just have to set to None on the second insertion to keep them same behavior
                let src = src_histogram[t][0];
                let dst = dst_histogram[t][0];
                self.mappings.link_if_both_unmapped(src, dst);
                self.last_chance_match_histogram(&src, &dst);
            }
        }
    }
}
