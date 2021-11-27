use std::fmt::Debug;

use bitvec::order::Lsb0;
use num_traits::{cast, PrimInt};

use crate::{
    matchers::{
        decompressed_tree_store::{
            BreathFirstIterable, DecompressedTreeStore, DecompressedWithParent, PostOrder,
        },
        mapping_store::{DefaultMappingStore, MappingStore, MonoMappingStore},
    },
    tree::{
        tree::{Labeled, NodeStore, Stored, WithChildren},
        tree_path::{CompressedTreePath, TreePath},
    },
    utils::sequence_algorithms::longest_common_subsequence,
};

pub trait Actions {
    fn len(&self) -> usize;
}

#[derive(Debug)]
pub struct ActionsVec<A: Debug>(Vec<A>);

#[derive(PartialEq, Eq)]
pub enum Act<T: Stored + Labeled + WithChildren> {
    Delete {},
    Update {
        new: T::Label,
    },
    Move {
        from: CompressedTreePath<T::ChildIdx>,
    },
    Insert {
        sub: T::TreeId,
    },
}

#[derive(PartialEq, Eq)]
pub struct SimpleAction<T: Stored + Labeled + WithChildren> {
    pub(crate) path: CompressedTreePath<T::ChildIdx>,
    pub(crate) action: Act<T>,
}

impl<T: Stored + Labeled + WithChildren> Debug for SimpleAction<T>
where
    T::Label: Debug,
    T::TreeId: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.action {
            Act::Delete {} => write!(f, "Del {:?}", self.path),
            Act::Update { new } => write!(f, "Upd {:?} {:?}", new, self.path),
            Act::Move { from: sub } => write!(f, "Mov {:?} {:?}", sub, self.path),
            Act::Insert { sub } => write!(f, "Ins {:?} {:?}", sub, self.path),
        }
    }
}

impl<IdD: Debug> Actions for ActionsVec<IdD> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

pub trait TestActions<T: Stored + Labeled + WithChildren> {
    fn has_actions(&self, items: &[SimpleAction<T>]) -> bool;
}

impl<T: Stored + Labeled + WithChildren + std::cmp::PartialEq> TestActions<T>
    for ActionsVec<SimpleAction<T>>
where
    T::Label: Debug,
    T::TreeId: Debug,
{
    fn has_actions(&self, items: &[SimpleAction<T>]) -> bool {
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
    T: Stored + Labeled + WithChildren,
    SS,
    SD: BreathFirstIterable<'a, T::TreeId, IdD> + DecompressedWithParent<IdD>,
    S: for<'b> NodeStore<'b, T>,
> where
    T::Label: Debug,
    T::TreeId: Debug,
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

    actions: ActionsVec<SimpleAction<T>>,

    src_in_order: InOrderNodes<IdD>,
    dst_in_order: InOrderNodes<IdD>,
}

impl<
        'a,
        IdD: PrimInt + Debug,
        T: Stored + Labeled + WithChildren,
        SS: DecompressedTreeStore<T::TreeId, IdD>
            + DecompressedWithParent<IdD>
            + PostOrder<T::TreeId, IdD>,
        SD: DecompressedTreeStore<T::TreeId, IdD>
            + DecompressedWithParent<IdD>
            + BreathFirstIterable<'a, T::TreeId, IdD>,
        S: for<'b> NodeStore<'b, T>,
    > ScriptGenerator<'a, IdD, T, SS, SD, S>
where
    T::Label: Debug + Copy,
    T::TreeId: Debug,
{
    pub fn compute_actions(
        store: &'a S,
        src_arena: &'a SS,
        dst_arena: &'a SD,
        ms: &'a DefaultMappingStore<IdD>,
    ) -> ActionsVec<SimpleAction<T>> {
        Self::new(store, src_arena, dst_arena)
            .init_cpy(ms)
            .generate()
            .actions
    }

    fn new(store: &'a S, src_arena: &'a SS, dst_arena: &'a SD) -> Self {
        Self {
            store,
            src_arena_dont_use: src_arena,
            mid_arena: vec![],
            mid_root: src_arena.root(),
            dst_arena,
            ori_mappings: None,
            cpy_mappings: DefaultMappingStore::new(),
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
        for x in self.src_arena_dont_use.iter_df_post() {
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

    fn generate(mut self) -> Self {
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
            let z = y.and_then(|y| Some(self.cpy_mappings.get_src(&y)));
            if !self.cpy_mappings.is_dst(&x) {
                // insertion
                let k = if let Some(y) = y {
                    self.find_pos(&x, &y)
                } else {
                    num_traits::zero()
                };
                w = self.make_inserted_node(&x, &z);
                let path = if let Some(z) = z {
                    self.path(z).extend(&[k])
                    // self.src_arena_dont_use
                    // .path(&self.src_arena_dont_use.root(), &z)
                } else {
                    CompressedTreePath::from(vec![k])
                };
                let action = SimpleAction {
                    path,
                    action: Act::Insert {
                        sub: self.dst_arena.original(&x),
                        // parent: z,
                        // idx: k,
                    },
                };
                {
                    if let Some(z) = z {
                        let z: usize = cast(z).unwrap();
                        if let Some(cs) = self.mid_arena[z].children.as_mut() {
                            cs.insert(cast(k).unwrap(), w)
                        } else {
                            self.mid_arena[z].children = Some(vec![w])
                        };
                    } else {
                        self.mid_root = w;
                    }
                };
                self.actions.push(action);
            } else {
                w = self.cpy_mappings.get_src(&x);
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

                if z != v {
                    // move
                    let k = if let Some(y) = y {
                        self.find_pos(&x, &y)
                    } else {
                        num_traits::zero()
                    };
                    let path = if let Some(z) = z {
                        // FIXME should be path in mid_arena as z might be new and even more if moved
                        // let z = self.copy_to_orig(z);
                        // self.src_arena_dont_use
                        //     .path(&self.src_arena_dont_use.root(), &z)
                        //     .extend(&[k])
                        self.path(z).extend(&[k])
                    } else {
                        CompressedTreePath::from(vec![k])
                    };
                    let sub = self
                        .src_arena_dont_use
                        .path(&self.src_arena_dont_use.root(), &self.copy_to_orig(w));
                    let action = SimpleAction {
                        path,
                        action: Act::Move {
                            from: sub,
                            // parent: z,
                            // idx: k,
                        },
                    };
                    self.actions.push(action);

                    let upd = if w_l != x_l {
                        // rename
                        let path = self
                            .src_arena_dont_use
                            .path(&self.src_arena_dont_use.root(), &self.copy_to_orig(w));
                        let action = SimpleAction::<T> {
                            path,
                            action: Act::Update {
                                // src: self.copy_to_orig(w),
                                // dst: x,
                                // old: w_l,
                                new: x_l,
                            },
                        };
                        Some(action)
                    } else {
                        None
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
                            .remove(idx);
                    };
                    {
                        // TODO do not mutate existing node
                        if let Some(z) = z {
                            let z: usize = cast(z).unwrap();
                            if let Some(cs) = self.mid_arena[z].children.as_mut() {
                                cs.insert(cast(k).unwrap(), w)
                            } else {
                                self.mid_arena[z].children = Some(vec![w])
                            };
                            self.mid_arena[cast::<_, usize>(w).unwrap()].parent = cast(z).unwrap();
                        } else {
                            self.mid_root = w;
                            self.mid_arena[cast::<_, usize>(w).unwrap()].parent = cast(w).unwrap();
                        }
                    };
                    if let Some(action) = upd {
                        self.mid_arena[cast::<_, usize>(w).unwrap()].compressed =
                            self.dst_arena.original(&x);
                        self.actions.push(action);
                    }
                } else if w_l != x_l {
                    // rename
                    let path = self
                        .src_arena_dont_use
                        .path(&self.src_arena_dont_use.root(), &self.copy_to_orig(w));
                    let action = SimpleAction::<T> {
                        path,
                        action: Act::Update {
                            // src: self.copy_to_orig(w),
                            // dst: x,
                            // old: w_l,
                            new: x_l,
                        },
                    };
                    self.mid_arena[cast::<_, usize>(w).unwrap()].compressed =
                        self.dst_arena.original(&x);
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
                let path = self
                    .src_arena_dont_use
                    .path(&self.src_arena_dont_use.root(), &self.copy_to_orig(w));
                let action = SimpleAction {
                    path,
                    action: Act::Delete {
                        // tree: self.copy_to_orig(w),
                    },
                };
                // TODO self.apply_action(&action, &w);
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
                if x_c.contains(&self.cpy_mappings.get_dst(c)) {
                    s1.push(*c);
                }
            }
        }
        let mut s2 = vec![];
        for c in &x_c {
            if self.cpy_mappings.is_dst(c) {
                if w_c.contains(&self.cpy_mappings.get_src(c)) {
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
                if self.ori_mappings.unwrap().has(&a, &b) && !lcs.contains(&(*a, *b)) {
                    let k = self.find_pos(b, x);
                    let path = self
                        .src_arena_dont_use
                        .path(&self.src_arena_dont_use.root(), &self.ori_to_copy(*w))
                        .extend(&[k]);
                    let sub = self
                        .src_arena_dont_use
                        .path(&self.src_arena_dont_use.root(), &self.ori_to_copy(*a));
                    let action = SimpleAction {
                        path,
                        action: Act::Move { from: sub },
                    };
                    // let action = SimpleAction::Move {
                    //     sub: self.ori_to_copy(*a),
                    //     parent: Some(*x),
                    //     idx: k,
                    // };
                    // self.apply_action(&action, &self.ori_to_copy(*a));
                    let z: usize = cast(*w).unwrap();
                    if let Some(cs) = self.mid_arena[z].children.as_mut() {
                        cs.insert(cast(k).unwrap(), *a)
                    } else {
                        self.mid_arena[z].children = Some(vec![*a])
                    };
                    // self.apply_move(&action, &Some(*w), &self.ori_to_copy(*a), b);
                    self.actions.push(action);
                    self.src_in_order.push(*a);
                    self.dst_in_order.push(*b);
                }
            }
        }
    }

    /// find position of x in parent on dst_arena
    pub(crate) fn find_pos(&self, x: &IdD, y: &IdD) -> T::ChildIdx {
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
        let xpos = cast(self.dst_arena.position_in_parent(self.store, x)).unwrap(); //child.positionInParent();
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

        let u = self.cpy_mappings.get_src(&v.unwrap());
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
        let w = cast(self.mid_arena.len()).unwrap();
        let z = if let Some(z) = z {
            cast(*z).unwrap()
        } else {
            w
        };
        self.mid_arena.push(MidNode {
            parent: cast(z).unwrap(),
            compressed: self.dst_arena.original(x),
            children: None,
        });
        self.moved.push(false);

        self.cpy_mappings.topit(
            self.cpy_mappings.src_to_dst.len() + 1,
            self.cpy_mappings.dst_to_src.len(),
        );
        self.cpy_mappings.link(w, *x);
        w
    }

    fn iter_mid_in_post_order<'b>(
        root: IdD,
        mid_arena: &'b [MidNode<T::TreeId, IdD>],
    ) -> Iter<'b, T::TreeId, IdD> {
        let parent: Vec<(IdD, usize)> = vec![(root, num_traits::zero())];
        Iter { parent, mid_arena }
    }

    fn copy_to_orig(&self, w: IdD) -> IdD {
        if self.src_arena_dont_use.len() <= cast(w).unwrap() {
            panic!()
        }
        w
    }

    pub(crate) fn ori_to_copy(&self, a: IdD) -> IdD {
        if self.src_arena_dont_use.len() <= cast(a).unwrap() {
            panic!()
        }
        a
    }

    fn path(&self, mut z: IdD) -> CompressedTreePath<T::ChildIdx> {
        let mut r = vec![];
        loop {
            let p = self.mid_arena[cast::<_, usize>(z).unwrap()].parent;
            if p == z {
                break;
            } else {
                let i = self.mid_arena[cast::<_, usize>(p).unwrap()]
                    .children
                    .as_ref()
                    .unwrap()
                    .iter()
                    .position(|x| x == &z)
                    .unwrap();
                r.push(cast(i).unwrap());
                z = p;
            }
        }
        r.reverse();
        r.into()
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

impl<T: Stored + Labeled + WithChildren> ActionsVec<SimpleAction<T>>
where
    T::Label: Debug,
    T::TreeId: Debug,
{
    pub(crate) fn push(&mut self, action: SimpleAction<T>) {
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
