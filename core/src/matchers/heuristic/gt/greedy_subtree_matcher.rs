use std::{fmt::Debug, marker::PhantomData};

use crate::matchers::heuristic::gt::height;
use crate::tree::tree::HashKind;
use crate::{
    matchers::{
        decompressed_tree_store::{DecompressedTreeStore, DecompressedWithParent},
        mapping_store::{
            DefaultMappingStore, DefaultMultiMappingStore, MappingStore, MultiMappingStore,
        },
        similarity_metrics,
    },
    tree::tree::{NodeStore, Tree, WithHashs},
};
use bitvec::order::Lsb0;
use num_traits::{cast, one, zero, PrimInt};

//System.getProperty("gt.stm.mh", System.getProperty("gumtree.match.gt.minh", "2"))

// trait SubTreeMatcherWithFilter {
//     fn filterMappings(&self, multiMappings: MultiMappingStore<ITree>);
// }
pub struct GreedySubtreeMatcher<
    'a,
    D: DecompressedTreeStore<T::TreeId, IdD> + DecompressedWithParent<IdD>,
    IdD: PrimInt, // + Into<usize> + std::ops::SubAssign + Debug,
    T: Tree,      // + WithHashs,
    S: NodeStore<T>,
    const MIN_HEIGHT: usize, // = 2
> {
    internal: SubtreeMatcher<'a, D, IdD, T, S, MIN_HEIGHT>,
    // pub(super) node_store: &'a S,
    // pub(crate) src_arena: D,
    // pub(crate) dst_arena: D,
    // pub(crate) mappings: DefaultMappingStore<IdD>,
    // pub(super) phantom: PhantomData<*const T>
}

impl<
        'a,
        D: DecompressedTreeStore<T::TreeId, IdD> + DecompressedWithParent<IdD>,
        IdD: PrimInt + Debug, // + Into<usize> + std::ops::SubAssign,
        T: Tree + WithHashs,
        S: NodeStore<T>,
        const MIN_HEIGHT: usize, // = 2
    > GreedySubtreeMatcher<'a, D, IdD, T, S, MIN_HEIGHT>
{
    pub fn matchh(
        node_store: &'a S,
        // label_store: &'a LS,
        src: &'a T::TreeId,
        dst: &'a T::TreeId,
        mappings: DefaultMappingStore<IdD>,
    ) -> Self {
        let mut matcher = Self {
            // label_store,
            internal: SubtreeMatcher {
                node_store,
                src_arena: D::new(node_store, src),
                dst_arena: D::new(node_store, dst),
                mappings,
                phantom: PhantomData,
            },
        };
        matcher.internal.mappings.topit(
            matcher.internal.src_arena.len() + 1,
            matcher.internal.dst_arena.len() + 1,
        );
        Self::execute(&mut matcher);
        matcher
    }

    pub(crate) fn execute(&mut self) {
        let m = self.internal.matchh_to_be_filtered();
        self.filter_mappings(m);
    }

    // @Override
    fn filter_mappings(&mut self, multi_mappings: DefaultMultiMappingStore<IdD>) {
        // Select unique mappings first and extract ambiguous mappings.
        let mut ambiguous_list: Vec<Mapping<IdD>> = vec![];
        let mut ignored = vec![false; self.internal.src_arena.len()];
        multi_mappings.allMappedSrcs();
        for src in multi_mappings.allMappedSrcs() {
            let mut is_mapping_unique = false;
            if multi_mappings.isSrcUnique(&src) {
                let dst = multi_mappings.get_dsts(&src)[0];
                if multi_mappings.isDstUnique(&dst) {
                    self.internal.add_mapping_recursively(&src, &dst);
                    is_mapping_unique = true;
                }
            }

            if !(ignored[cast::<_, usize>(src).unwrap()] || is_mapping_unique) {
                let adsts = multi_mappings.get_dsts(&src);
                let asrcs = multi_mappings.get_srcs(&multi_mappings.get_dsts(&src)[0]);
                for asrc in asrcs {
                    for adst in adsts {
                        ambiguous_list.push((*asrc, *adst));
                    }
                }
                asrcs
                    .iter()
                    .for_each(|x| ignored[cast::<_, usize>(*x).unwrap()] = true)
            }
        }

        let mapping_list = self.sort(ambiguous_list);

        // Select the best ambiguous mappings
        let mut src_ignored = bitvec::bitvec![];
        src_ignored.resize(self.internal.src_arena.len(), false);
        let mut dst_ignored = bitvec::bitvec![];
        dst_ignored.resize(self.internal.dst_arena.len(), false);
        for (src, dst) in mapping_list {
            let src_i: usize = cast(src).unwrap();
            let dst_i: usize = cast(dst).unwrap();
            if !(src_ignored[src_i] || dst_ignored[dst_i]) {
                self.internal.add_mapping_recursively(&src, &dst);
                src_ignored.set(src_i, true);
                self.internal
                    .src_arena
                    .descendants(self.internal.node_store, &src)
                    .iter()
                    .for_each(|src| src_ignored.set(cast::<_, usize>(*src).unwrap(), true));
                dst_ignored.set(dst_i, true);
                self.internal
                    .dst_arena
                    .descendants(self.internal.node_store, &dst)
                    .iter()
                    .for_each(|dst| dst_ignored.set(cast::<_, usize>(*dst).unwrap(), true));
            }
        }
    }

    fn sort(&self, ambiguous_mappings: Vec<Mapping<IdD>>) -> impl Iterator<Item = Mapping<IdD>> {
        let mut similarities = vec![];

        for p in ambiguous_mappings {
            let sim = self.internal.similarity(&p.0, &p.1);
            similarities.push((p, sim));
        }

        similarities.sort_by(|(alink, asim), (blink, bsim)| -> std::cmp::Ordering {
            if asim != bsim {
                // todo caution about exact comparing of floats
                if let Some(r) = asim.partial_cmp(&bsim) {
                    return r;
                }
            }
            if alink.0 != alink.0 {
                return alink.0.cmp(&blink.0);
            }
            return alink.1.cmp(&blink.1);
        });
        similarities.into_iter().map(|(x, _)| x)
    }
}
impl<
        'a,
        D: DecompressedTreeStore<T::TreeId, IdD> + DecompressedWithParent<IdD>,
        IdD: PrimInt, // + Into<usize> + std::ops::SubAssign + Debug,
        T: Tree,      // + WithHashs,
        S: NodeStore<T>,
        const MIN_HEIGHT: usize,
    > Into<SubtreeMatcher<'a, D, IdD, T, S, MIN_HEIGHT>>
    for GreedySubtreeMatcher<'a, D, IdD, T, S, MIN_HEIGHT>
{
    fn into(self) -> SubtreeMatcher<'a, D, IdD, T, S, MIN_HEIGHT> {
        self.internal
    }
}

type Mapping<T> = (T, T);

pub struct SubtreeMatcher<
    'a,
    D: DecompressedTreeStore<T::TreeId, IdD> + DecompressedWithParent<IdD>,
    IdD: PrimInt, // + Into<usize> + std::ops::SubAssign + Debug,
    T: Tree,      // + WithHashs,
    S: NodeStore<T>,
    const MIN_HEIGHT: usize,
> {
    pub(super) node_store: &'a S,
    pub(crate) src_arena: D,
    pub(crate) dst_arena: D,
    pub(crate) mappings: DefaultMappingStore<IdD>,
    pub(super) phantom: PhantomData<*const T>,
}
impl<
        'a,
        D: DecompressedTreeStore<T::TreeId, IdD> + DecompressedWithParent<IdD>,
        IdD: PrimInt + Debug, // + Into<usize> + std::ops::SubAssign + Debug,
        T: Tree + WithHashs,
        S: NodeStore<T>,
        const MIN_HEIGHT: usize,
    > SubtreeMatcher<'a, D, IdD, T, S, MIN_HEIGHT>
{
    pub(crate) fn add_mapping_recursively(&mut self, src: &IdD, dst: &IdD) {
        self.mappings.link(*src, *dst);
        self.src_arena
            .descendants(self.node_store, src)
            .iter()
            .zip(self.dst_arena.descendants(self.node_store, dst).iter())
            .for_each(|(src, dst)| self.mappings.link(*src, *dst));
    }

    fn pop_larger(
        &self,
        src_trees: &mut PriorityTreeList<'a, D, IdD, T, S, MIN_HEIGHT>,
        dst_trees: &mut PriorityTreeList<'a, D, IdD, T, S, MIN_HEIGHT>,
    ) {
        if src_trees.peek_height() > dst_trees.peek_height() {
            src_trees.open();
        } else {
            dst_trees.open();
        }
    }

    fn matchh_to_be_filtered(&self) -> DefaultMultiMappingStore<IdD> {
        let mut multi_mappings = DefaultMultiMappingStore::<IdD> {
            src_to_dsts: vec![None; self.src_arena.len()],
            dst_to_srcs: vec![None; self.dst_arena.len()],
        };

        let mut src_trees =
            PriorityTreeList::new(self.node_store, &self.src_arena, self.src_arena.root());
        let mut dst_trees =
            PriorityTreeList::new(self.node_store, &self.dst_arena, self.dst_arena.root());

        while src_trees.peek_height() != -1 && dst_trees.peek_height() != -1 {
            while src_trees.peek_height() != dst_trees.peek_height() {
                self.pop_larger(&mut src_trees, &mut dst_trees);
            }

            let current_height_src_trees = src_trees.pop().unwrap();
            let current_height_dst_trees = dst_trees.pop().unwrap();

            let mut marks_for_src_trees = vec![false; current_height_src_trees.len()];
            let mut marks_for_dst_trees = vec![false; current_height_dst_trees.len()];

            for i in 0..current_height_src_trees.len() {
                for j in 0..current_height_dst_trees.len() {
                    let src = current_height_src_trees[i];
                    let dst = current_height_dst_trees[j];

                    if self.isomorphic(&src, &dst) {
                        multi_mappings.link(src, dst);
                        marks_for_src_trees[i] = true;
                        marks_for_dst_trees[j] = true;
                    }
                }
            }

            for i in 0..marks_for_src_trees.len() {
                if marks_for_src_trees[i] == false {
                    src_trees.open_tree(&current_height_src_trees[i]);
                }
            }
            for j in 0..marks_for_dst_trees.len() {
                if marks_for_dst_trees[j] == false {
                    dst_trees.open_tree(&current_height_dst_trees[j]);
                }
            }

            src_trees.update_height();
            dst_trees.update_height();
        }

        multi_mappings
    }

    // fn abstract filterMappings(MultiMappingStore multiMappings);

    fn similarity(&self, src: &IdD, dst: &IdD) -> f64 {
        let p_src = self.src_arena.parent(src).unwrap();
        let p_dst = self.dst_arena.parent(dst).unwrap();
        let jaccard = similarity_metrics::jaccard_similarity(
            &self.src_arena.descendants(self.node_store, &p_src),
            &self.dst_arena.descendants(self.node_store, &p_dst),
            &self.mappings,
        );
        let pos_src = if self.src_arena.has_parent(src) {
            zero()
        } else {
            self.src_arena.position_in_parent(self.node_store, src)
        };
        let pos_dst = if self.dst_arena.has_parent(dst) {
            zero()
        } else {
            self.dst_arena.position_in_parent(self.node_store, dst)
        };

        let max_src_pos = if self.src_arena.has_parent(src) {
            one()
        } else {
            self.node_store
                .get_node_at_id(&self.src_arena.original(&p_src))
                .child_count()
        };
        let max_dst_pos = if self.dst_arena.has_parent(dst) {
            one()
        } else {
            self.node_store
                .get_node_at_id(&self.dst_arena.original(&p_dst))
                .child_count()
        };
        let max_pos_diff = std::cmp::max(max_src_pos, max_dst_pos);
        let pos: f64 = 1.0_f64
            - (cast::<_, f64>(pos_src.max(pos_dst) - pos_dst.min(pos_src)).unwrap()
                / cast::<_, f64>(max_pos_diff).unwrap());
        let po: f64 = 1.0_f64
            - (cast::<_, f64>(*src.max(dst) - *dst.min(src)).unwrap()
                / cast::<_, f64>(self.get_max_tree_size()).unwrap());
        100. * jaccard + 10. * pos + po
    }

    fn get_max_tree_size(&self) -> usize {
        self.src_arena.len().max(self.dst_arena.len())
    }

    pub(crate) fn isomorphic(&self, src: &IdD, dst: &IdD) -> bool {
        let src = self.src_arena.original(src);
        let dst = self.dst_arena.original(dst);

        self.isomorphic2(&src, &dst)
    }

    pub(crate) fn isomorphic2(&self, src: &T::TreeId, dst: &T::TreeId) -> bool {
        if src == dst {
            return true;
        }
        let src = self.node_store.get_node_at_id(src);
        let src_h = src.hash(&T::HK::label());
        let src_t = src.get_type();
        let src_l = if src.has_label() {
            Some(src.get_label())
        } else {
            None
        };
        let src_c = src.get_children().to_owned();

        let dst = self.node_store.get_node_at_id(dst);

        let dst_h = dst.hash(&T::HK::label());
        if src_h != dst_h {
            return false;
        }
        let dst_t = dst.get_type();
        if src_t != dst_t {
            return false;
        }
        if dst.has_label() {
            if src_l.is_none() || src_l.unwrap() != dst.get_label() {
                return false;
            }
        };

        let dst_c = dst.get_children().to_owned();
        if src_c.len() != dst_c.len() {
            return false;
        }

        for (src, dst) in src_c.iter().zip(dst_c.iter()) {
            if !self.isomorphic2(src, dst) {
                return false;
            }
        }

        true
    }
}

struct PriorityTreeList<'a, D, IdD, T: Tree, S, const MIN_HEIGHT: usize> {
    trees: Vec<Option<Vec<IdD>>>, //List<ITree>[]

    store: &'a S,
    arena: &'a D,

    max_height: usize,

    current_idx: isize,

    phantom: PhantomData<*const T>,
}

impl<
        'a,
        D: DecompressedTreeStore<T::TreeId, IdD>,
        IdD: PrimInt,
        T: Tree,
        S: NodeStore<T>,
        const MIN_HEIGHT: usize,
    > PriorityTreeList<'a, D, IdD, T, S, MIN_HEIGHT>
{
    pub(super) fn new(store: &'a S, arena: &'a D, tree: IdD) -> Self {
        let h = height(store, &arena.original(&tree));
        let list_size = if h >= MIN_HEIGHT {
            h + 1 - MIN_HEIGHT
        } else {
            0
        };
        let mut r = Self {
            trees: vec![None; list_size],
            store,
            arena,
            max_height: h,
            current_idx: if list_size == 0 { -1 } else { 0 },
            phantom: PhantomData,
        };
        r.add_tree2(tree, h);
        r
    }

    // fn idx_tree(&self, tree: &IdD) -> usize {
    //     self.idx(height(self.store, self.arena.original(tree)) as usize) //tree.getMetrics().height())
    // }

    fn idx(&self, height: usize) -> usize {
        self.max_height - height
    }

    fn height(&self, idx: usize) -> usize {
        self.max_height - idx
    }

    fn add_tree(&mut self, tree: IdD) {
        let h = height(self.store, &self.arena.original(&tree)) as usize;
        self.add_tree2(tree, h)
    }

    fn add_tree2(&mut self, tree: IdD, h: usize) {
        if h >= MIN_HEIGHT {
            let idx = self.idx(h);
            if self.trees[idx].is_none() {
                self.trees[idx] = Some(vec![]);
            };
            self.trees[idx].as_mut().unwrap().push(tree);
        }
    }

    pub(super) fn open(&mut self) -> Option<Vec<IdD>> {
        if let Some(pop) = self.pop() {
            for tree in &pop {
                self.open_tree(tree);
            }
            self.update_height();
            Some(pop)
        } else {
            None
        }
    }

    pub(super) fn pop(&mut self) -> Option<Vec<IdD>> {
        if self.current_idx < 0 {
            None
        } else {
            self.trees[self.current_idx as usize].take()
        }
    }

    pub(super) fn open_tree(&mut self, tree: &IdD) {
        for c in self.arena.children(self.store, tree) {
            self.add_tree(c);
        }
    }

    pub(super) fn peek_height(&self) -> isize {
        if self.current_idx == -1 {
            -1
        } else {
            self.height(self.current_idx as usize) as isize
        }
    }

    pub(super) fn update_height(&mut self) {
        self.current_idx = -1;
        for i in 0..self.trees.len() {
            if self.trees[i].is_some() {
                self.current_idx = i as isize;
                break;
            }
        }
    }
}
