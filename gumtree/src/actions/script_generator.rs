/// inspired by the implementation in gumtree
use std::fmt::Debug;

use bitvec::order::Lsb0;
use hyper_ast::types::{Labeled, NodeStore, Stored, WithChildren};
use num_traits::{cast, PrimInt};

use crate::{
    decompressed_tree_store::{
        BreadthFirstIterable, DecompressedTreeStore, DecompressedWithParent, PostOrder,
        PostOrderIterable,
    },
    matchers::mapping_store::{DefaultMappingStore, MappingStore, MonoMappingStore},
    utils::sequence_algorithms::longest_common_subsequence,
};

pub trait Actions {
    fn len(&self) -> usize;
}
pub struct ActionsVec<A>(Vec<A>);

impl<A: Debug> Debug for ActionsVec<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(PartialEq, Eq)]
pub enum SimpleAction<Src, Dst, T: Stored + Labeled + WithChildren> {
    Delete {
        tree: Src,
    },
    Update {
        src: Src,
        dst: Dst,
        old: T::Label,
        new: T::Label,
    },
    Move {
        sub: Src,
        parent: Option<Dst>,
        idx: T::ChildIdx,
    },
    // Duplicate { sub: Src, parent: Dst, idx: T::ChildIdx },
    MoveUpdate {
        sub: Src,
        parent: Option<Dst>,
        idx: T::ChildIdx,
        old: T::Label,
        new: T::Label,
    },
    Insert {
        sub: T::TreeId,
        parent: Option<Dst>,
        idx: T::ChildIdx,
    },
}

impl<Src: Debug, Dst: Debug, T: Stored + Labeled + WithChildren> Debug for SimpleAction<Src, Dst, T>
where
    T::TreeId: Debug,
    T::ChildIdx: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimpleAction::Delete { tree } => write!(f, "Del {:?}", tree),
            SimpleAction::Update {
                src,
                dst,
                old: _,
                new: _,
            } => write!(f, "Upd {:?} {:?}", src, dst),
            SimpleAction::Move { sub, parent, idx } => {
                write!(f, "Mov {:?} {:?} {:?}", sub, parent, idx)
            }
            SimpleAction::MoveUpdate {
                sub,
                parent,
                idx,
                old: _,
                new: _,
            } => write!(f, "MovUpd {:?} {:?} {:?}", sub, parent, idx),
            SimpleAction::Insert { sub, parent, idx } => {
                write!(f, "Ins {:?} {:?} {:?}", sub, parent, idx)
            }
        }
    }
}

impl<IdD: Debug> Actions for ActionsVec<IdD> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

pub trait TestActions<IdD, T: Stored + Labeled + WithChildren> {
    fn has_actions(&self, items: &[SimpleAction<IdD, IdD, T>]) -> bool;
}

impl<
        T: Stored + Labeled + WithChildren + std::cmp::PartialEq,
        IdD: std::cmp::PartialEq + Debug,
    > TestActions<IdD, T> for ActionsVec<SimpleAction<IdD, IdD, T>>
where
    T::TreeId: Debug,
    T::ChildIdx: Debug,
{
    fn has_actions(&self, items: &[SimpleAction<IdD, IdD, T>]) -> bool {
        items.iter().all(|x| self.0.contains(x))
    }
}

struct InOrderNodes<IdD>(Option<Vec<IdD>>);

/// FEATURE: share parents
static COMPRESSION: bool = false;

struct MidNode<IdC, IdD> {
    parent: IdD,
    compressed: IdC,
    children: Option<Vec<IdD>>,
}

pub struct ScriptGenerator<
    'a,
    IdD: PrimInt + Debug,
    T: 'a + Stored + Labeled + WithChildren,
    SS,
    SD: BreadthFirstIterable<'a, T, IdD> + DecompressedWithParent<'a, T, IdD>,
    S,
>
{
    store: &'a S,
    src_arena_dont_use: &'a SS,
    mid_arena: Vec<MidNode<T::TreeId, IdD>>, //SuperTreeStore<T::TreeId>,
    mid_root: IdD,
    dst_arena: &'a SD,
    // ori_to_copy: DefaultMappingStore<IdD>,
    ori_mappings: Option<&'a DefaultMappingStore<IdD>>,
    cpy_mappings: DefaultMappingStore<IdD>,
    moved: bitvec::vec::BitVec,

    pub actions: ActionsVec<SimpleAction<IdD, IdD, T>>,

    src_in_order: InOrderNodes<IdD>,
    dst_in_order: InOrderNodes<IdD>,
}

impl<
        'a,
        IdD: PrimInt + Debug,
        T: 'a + Stored + Labeled + WithChildren,
        SS: DecompressedTreeStore<'a, T, IdD>
            + DecompressedWithParent<'a, T, IdD>
            + PostOrderIterable<'a, T, IdD>
            + PostOrder<'a, T, IdD>,
        SD: DecompressedTreeStore<'a, T, IdD>
            + DecompressedWithParent<'a, T, IdD>
            + BreadthFirstIterable<'a, T, IdD>,
        S: 'a, //:'a + NodeStore2<T::TreeId, R<'a> = T>, //NodeStore<'a, T::TreeId, T>,
    > ScriptGenerator<'a, IdD, T, SS, SD, S>
where
    S: NodeStore<T::TreeId, R<'a> = T>,
    // S: 'a + NodeStore<T::TreeId>,
    // for<'c> <<S as NodeStore2<T::TreeId>>::R as GenericItem<'c>>::Item:
    //     hyper_ast::types::Tree<TreeId = T::TreeId, Label = T::Label, ChildIdx = T::ChildIdx>+WithChildren,
    // S::R<'a>: hyper_ast::types::Tree<TreeId = T::TreeId, Label = T::Label, ChildIdx = T::ChildIdx>,
    T::Label: Copy,
    T::TreeId: Debug,
    T::ChildIdx: Debug,
{
    pub fn compute_actions(
        store: &'a S,
        src_arena: &'a SS,
        dst_arena: &'a SD,
        ms: &'a DefaultMappingStore<IdD>,
    ) -> ActionsVec<SimpleAction<IdD, IdD, T>> {
        ScriptGenerator::<'a, IdD, T, SS, SD, S>::new(store, src_arena, dst_arena)
            .init_cpy(ms)
            .generate()
            .actions
    }
    pub fn precompute_actions(
        store: &'a S,
        src_arena: &'a SS,
        dst_arena: &'a SD,
        ms: &'a DefaultMappingStore<IdD>,
    ) -> ScriptGenerator<'a, IdD, T, SS, SD, S> {
        ScriptGenerator::<'a, IdD, T, SS, SD, S>::new(store, src_arena, dst_arena).init_cpy(ms)
    }

    fn new(store: &'a S, src_arena: &'a SS, dst_arena: &'a SD) -> Self {
        Self {
            store,
            src_arena_dont_use: src_arena,
            mid_arena: vec![],
            mid_root: src_arena.root(),
            dst_arena,
            ori_mappings: None,
            cpy_mappings: Default::default(),
            actions: ActionsVec::new(),
            src_in_order: InOrderNodes(None),
            dst_in_order: InOrderNodes(None),
            moved: bitvec::bitvec![],
        }
    }

    fn init_cpy(mut self, ms: &'a DefaultMappingStore<IdD>) -> Self {
        // copy mapping
        self.ori_mappings = Some(ms);
        self.cpy_mappings = ms.clone();
        self.moved.resize(self.src_arena_dont_use.len(), false);
        for x in self.src_arena_dont_use.iter_df_post::<true>() {
            let children = self.src_arena_dont_use.children(self.store, &x);
            let children = if children.len() > 0 {
                Some(children)
            } else {
                None
            };
            self.mid_arena.push(MidNode {
                parent: self
                    .src_arena_dont_use
                    .parent(&x)
                    .unwrap_or(num_traits::zero()),
                compressed: self.src_arena_dont_use.original(&x),
                children,
            });
        }

        self.mid_arena[cast::<_, usize>(self.mid_root).unwrap()].parent = self.mid_root;

        self
    }

    pub fn generate(mut self) -> Self {
        // fake root ?
        // fake root link ?

        self.ins_mov_upd();

        self.del();
        self
    }

    fn ins_mov_upd(&mut self) {
        if COMPRESSION {
            todo!()
        }
        self.auxilary_ins_mov_upd();
    }

    fn auxilary_ins_mov_upd(&mut self) {
        for x in self.dst_arena.iter_bf() {
            let w;
            let y = self.dst_arena.parent(&x);
            let z = y.and_then(|y| Some(self.cpy_mappings.get_src_unchecked(&y)));
            if !self.cpy_mappings.is_dst(&x) {
                // insertion
                let k = if let Some(y) = y {
                    self.find_pos(&x, &y)
                } else {
                    num_traits::zero()
                };
                w = self.make_inserted_node(&x, &z);
                let action = SimpleAction::Insert {
                    sub: self.dst_arena.original(&x),
                    parent: y, // different from original gt because in the general case parent might not exist in src
                    idx: k,
                };
                self.apply_insert(&action, &z, &w, &x);
                self.actions.push(action);
            } else {
                w = self.cpy_mappings.get_src_unchecked(&x);
                let v = {
                    let v = self.mid_arena[cast::<_, usize>(w).unwrap()].parent;
                    if v == w {
                        None
                    } else {
                        Some(v)
                    }
                };
                let w_l = {
                    let c = &self.mid_arena[cast::<_, usize>(w).unwrap()].compressed;
                    *self.store.resolve(c).get_label()
                };
                let x_l = {
                    let c = &self.dst_arena.original(&x);
                    *self.store.resolve(c).get_label()
                };

                if w_l != x_l && z != v {
                    // rename + move
                    let k = if let Some(y) = y {
                        self.find_pos(&x, &y)
                    } else {
                        num_traits::zero()
                    };
                    let action = SimpleAction::MoveUpdate {
                        sub: x,
                        parent: y,
                        idx: k,
                        old: w_l,
                        new: x_l,
                    };

                    if let Some(v) = v {
                        let idx = self.mid_arena[cast::<_, usize>(v).unwrap()]
                            .children
                            .as_ref()
                            .unwrap()
                            .iter()
                            .position(|x| x == &w)
                            .unwrap();

                        self.mid_arena[cast::<_, usize>(v).unwrap()]
                            .children
                            .as_mut()
                            .unwrap()
                            .swap_remove(idx);
                    };
                    self.apply_move(&action, &z, &w, &x);
                    self.actions.push(action);
                } else if w_l != x_l {
                    // rename
                    let action = SimpleAction::Update {
                        src: w,
                        dst: x,
                        old: w_l,
                        new: x_l,
                    };
                    self.apply_update(&action, &w, &x);
                    self.actions.push(action);
                } else if z != v {
                    // move
                    let k = if let Some(y) = y {
                        self.find_pos(&x, &y)
                    } else {
                        num_traits::zero()
                    };
                    let action = SimpleAction::Move {
                        sub: x,
                        parent: y,
                        idx: k,
                    };

                    if let Some(v) = v {
                        let idx = self.mid_arena[cast::<_, usize>(v).unwrap()]
                            .children
                            .as_ref()
                            .unwrap()
                            .iter()
                            .position(|x| x == &w)
                            .unwrap();

                        self.mid_arena[cast::<_, usize>(v).unwrap()]
                            .children
                            .as_mut()
                            .unwrap()
                            .swap_remove(idx);
                    };
                    self.apply_move(&action, &z, &w, &x);
                    self.actions.push(action);
                } else {
                    // not changed
                    // and no changes to parents
                    // postentially try to share parent in super ast
                    if COMPRESSION {
                        todo!()
                    }
                }
                self.md_for_middle(&x, &w);
            }

            self.src_in_order.push(w);
            self.dst_in_order.push(x);
            self.align_children(&w, &x);
        }
    }

    fn del(&mut self) {
        for w in Self::iter_mid_in_post_order(self.mid_root, &self.mid_arena) {
            if !self.cpy_mappings.is_src(&w) {
                //todo mutate mid arena ?
                let action = SimpleAction::Delete {
                    tree: self.copy_to_orig(w),
                };
                self.actions.push(action)
            } else {
                // not modified
                // all parents were not modified
                // maybe do the resources sharing now
                if COMPRESSION {
                    todo!()
                }
            }
        }
        if COMPRESSION {
            // postorder compression ?
            todo!()
        }
    }

    pub(crate) fn align_children(&mut self, w: &IdD, x: &IdD) {
        let d = vec![];
        let w_c = self.mid_arena[cast::<_, usize>(*w).unwrap()]
            .children
            .as_ref()
            .unwrap_or(&d); //self.src_arena.children(self.store, w);
        self.src_in_order.remove_all(&w_c);
        let x_c = self.dst_arena.children(self.store, x);
        self.dst_in_order.remove_all(&x_c);

        // todo use iter filter collect
        let mut s1 = vec![];
        for c in w_c {
            if self.cpy_mappings.is_src(c) {
                if x_c.contains(&self.cpy_mappings.get_dst_unchecked(c)) {
                    s1.push(*c);
                }
            }
        }
        let mut s2 = vec![];
        for c in &x_c {
            if self.cpy_mappings.is_dst(c) {
                if w_c.contains(&self.cpy_mappings.get_src_unchecked(c)) {
                    s2.push(*c);
                }
            }
        }

        let lcs = self.lcs(&s1, &s2);

        for m in &lcs {
            self.src_in_order.push(m.0);
            self.dst_in_order.push(m.1);
        }

        for a in &s1 {
            for b in &s2 {
                if self.ori_mappings.unwrap().has(a, b) && !lcs.contains(&(*a, *b)) {
                    let k = self.find_pos(b, x);
                    let action = SimpleAction::Move {
                        sub: self.ori_to_copy(*a),
                        parent: Some(*x),
                        idx: k,
                    };
                    self.apply_move(&action, &Some(*w), &self.ori_to_copy(*a), b);
                    self.actions.push(action);
                    self.src_in_order.push(*a);
                    self.dst_in_order.push(*b);
                }
            }
        }
    }

    /// find position of x in parent on dst_arena
    pub(crate) fn find_pos(&self, x: &IdD, parent: &IdD) -> T::ChildIdx {
        let y = parent;
        let siblings = self.dst_arena.children(self.store, y);

        for c in &siblings {
            if self.dst_in_order.contains(c) {
                if c == x {
                    return num_traits::zero();
                } else {
                    break;
                }
            }
        }
        let xpos = cast(self.dst_arena.position_in_parent(x).unwrap()).unwrap();
        let mut v: Option<IdD> = None;
        for i in 0..xpos {
            let c: &IdD = &siblings[i];
            if self.dst_in_order.contains(c) {
                v = Some(*c);
            };
        }

        if v.is_none() {
            return num_traits::zero();
        }

        let u = self.cpy_mappings.get_src_unchecked(&v.unwrap());
        // // let upos = self.src_arena.position_in_parent(self.store, &u);
        let upos: T::ChildIdx = {
            let p = self.mid_arena[cast::<_, usize>(u).unwrap()].parent;
            let r = self.mid_arena[cast::<_, usize>(p).unwrap()]
                .children
                .as_ref()
                .unwrap()
                .iter()
                .position(|y| *y == u)
                .unwrap();
            cast::<usize, T::ChildIdx>(r).unwrap()
        };
        upos + num_traits::one()
    }

    pub(crate) fn lcs(&self, src_children: &[IdD], dst_children: &[IdD]) -> Vec<(IdD, IdD)> {
        longest_common_subsequence(src_children, dst_children, |src, dst| {
            self.cpy_mappings.has(src, dst)
        })
        .into_iter()
        .map(|m: (IdD, IdD)| {
            (
                src_children[cast::<_, usize>(m.0).unwrap()],
                dst_children[cast::<_, usize>(m.1).unwrap()],
            )
        })
        .collect()
    }

    pub(crate) fn md_for_middle(&self, _x: &IdD, _w: &IdD) {
        // todo maybe later
    }

    pub(crate) fn make_inserted_node(&mut self, x: &IdD, z: &Option<IdD>) -> IdD {
        let child = cast(self.mid_arena.len()).unwrap();
        let z = if let Some(z) = z {
            cast(*z).unwrap()
        } else {
            child
        };
        self.mid_arena.push(MidNode {
            parent: cast(z).unwrap(),
            compressed: self.dst_arena.original(x),
            children: None,
        });
        self.moved.push(false);
        child
    }

    pub(crate) fn apply_insert(
        &mut self,
        a: &SimpleAction<IdD, IdD, T>,
        z: &Option<IdD>,
        w: &IdD,
        x: &IdD,
    ) {
        self.cpy_mappings.topit(
            self.cpy_mappings.src_to_dst.len(),
            self.cpy_mappings.dst_to_src.len(),
        );
        if *w > num_traits::zero() {
            self.cpy_mappings.link(*w, *x);
        } else {
            panic!()
        }

        let idx = match a {
            SimpleAction::Insert { idx, .. } => idx,
            SimpleAction::Move { idx, .. } => idx,
            SimpleAction::MoveUpdate { idx, .. } => idx,
            _ => panic!(),
        };
        let child = *w;
        if let Some(z) = z {
            let z: usize = cast(*z).unwrap();
            if let Some(cs) = self.mid_arena[z].children.as_mut() {
                cs.insert(cast(*idx).unwrap(), child)
            } else {
                self.mid_arena[z].children = Some(vec![child])
            };
            self.mid_arena[cast::<_, usize>(*w).unwrap()].parent = cast(z).unwrap();
        } else {
            self.mid_arena[cast::<_, usize>(*w).unwrap()].parent = cast(child).unwrap();
        };

        if z.is_none() {
            self.mid_root = *w;
        }
    }

    pub(crate) fn apply_update(&mut self, action: &SimpleAction<IdD, IdD, T>, w: &IdD, x: &IdD) {
        match action {
            SimpleAction::Update { .. } => {}
            SimpleAction::MoveUpdate { .. } => {}
            _ => panic!(),
        }
        self.mid_arena[cast::<_, usize>(*w).unwrap()].compressed = self.dst_arena.original(x);
    }

    pub(crate) fn apply_move(
        &mut self,
        action: &SimpleAction<IdD, IdD, T>,
        z: &Option<IdD>,
        w: &IdD,
        x: &IdD,
    ) {
        match action {
            SimpleAction::MoveUpdate { .. } => {
                self.apply_update(action, w, x);
            }
            _ => (),
        }
        // self.moved.set(cast::<_, usize>(*w).unwrap(), true);
        self.cpy_mappings.cut(*w, *x);
        self.apply_insert(action, z, w, x);
    }

    fn iter_mid_in_post_order<'b>(
        root: IdD,
        mid_arena: &'b [MidNode<T::TreeId, IdD>],
    ) -> Iter<'b, T::TreeId, IdD> {
        let parent: Vec<(IdD, usize)> = vec![(root, num_traits::zero())];
        Iter { parent, mid_arena }
    }

    fn copy_to_orig(&self, w: IdD) -> IdD {
        w
    }

    pub(crate) fn ori_to_copy(&self, a: IdD) -> IdD {
        a
    }
}

struct Iter<'a, IdC, IdD: PrimInt> {
    parent: Vec<(IdD, usize)>,
    mid_arena: &'a [MidNode<IdC, IdD>],
}

impl<'a, IdC, IdD: num_traits::PrimInt> Iterator for Iter<'a, IdC, IdD> {
    type Item = IdD;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (id, idx) = if let Some(id) = self.parent.pop() {
                id
            } else {
                return None;
            };
            let curr = &self.mid_arena[cast::<_, usize>(id).unwrap()];
            if let Some(cs) = &curr.children {
                if cs.len() == idx {
                    return Some(id);
                } else {
                    self.parent.push((id, idx + 1));
                    self.parent.push((cs[idx], 0));
                }
            } else {
                return Some(id);
            }
        }
    }
}

impl<T: Stored + Labeled + WithChildren, IdD: Debug> ActionsVec<SimpleAction<IdD, IdD, T>>
where
    T::TreeId: Debug,
    T::ChildIdx: Debug,
{
    pub(crate) fn push(&mut self, action: SimpleAction<IdD, IdD, T>) {
        self.0.push(action)
    }

    pub(crate) fn new() -> Self {
        Self(Default::default())
    }
}

impl<IdD: Eq> InOrderNodes<IdD> {
    /// TODO add precondition to try to linerarly remove element (if both ordered the same way it's easy to remove without looking multiple times in both lists)
    fn remove_all(&mut self, w: &[IdD]) {
        if let Some(a) = self.0.take() {
            let c: Vec<IdD> = a.into_iter().filter(|x| !w.contains(x)).collect();
            if c.len() > 0 {
                self.0 = Some(c);
            }
        }
    }

    pub(crate) fn push(&mut self, x: IdD) {
        if let Some(l) = self.0.as_mut() {
            if !l.contains(&x) {
                l.push(x)
            }
        } else {
            self.0 = Some(vec![x])
        }
    }

    fn contains(&self, x: &IdD) -> bool {
        if let Some(l) = &self.0 {
            l.contains(x)
        } else {
            false
        }
    }
}
