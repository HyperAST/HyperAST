/// inspired by the implementation in gumtree
use std::{collections::HashSet, fmt::Debug, hash::Hash};

use num_traits::{cast, PrimInt, ToPrimitive};

use crate::{
    actions::Actions,
    decompressed_tree_store::{
        BreadthFirstIterable, DecompressedTreeStore, DecompressedWithParent, PostOrder,
        PostOrderIterable,
    },
    matchers::{mapping_store::MonoMappingStore, Mapping},
    tree::tree_path::TreePath,
    utils::sequence_algorithms::longest_common_subsequence,
};
use hyper_ast::types::{HyperAST, Labeled, NodeStore, Stored, Tree, WithChildren};

use super::action_vec::ActionsVec;

pub struct ApplicablePath<P> {
    pub ori: P,
    pub mid: P,
}

impl<P: PartialEq> PartialEq for ApplicablePath<P> {
    fn eq(&self, other: &Self) -> bool {
        self.ori == other.ori && self.mid == other.mid
    }
}
impl<P: Eq> Eq for ApplicablePath<P> {}

impl<P: Debug> Debug for ApplicablePath<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplicablePath")
            .field("orig", &self.ori)
            .field("mid", &self.mid)
            .finish()
    }
}

pub enum Act<L, P, I> {
    Delete {},
    Update { new: L },
    Move { from: ApplicablePath<P> },
    MovUpd { from: ApplicablePath<P>, new: L },
    Insert { sub: I },
}

impl<L: PartialEq, Idx: PartialEq, I: PartialEq> PartialEq for Act<L, Idx, I> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Update { new: l_new }, Self::Update { new: r_new }) => l_new == r_new,
            (Self::Move { from: l_from }, Self::Move { from: r_from }) => l_from == r_from,
            (
                Self::MovUpd {
                    from: l_from,
                    new: l_new,
                },
                Self::MovUpd {
                    from: r_from,
                    new: r_new,
                },
            ) => l_from == r_from && l_new == r_new,
            (Self::Insert { sub: l_sub }, Self::Insert { sub: r_sub }) => l_sub == r_sub,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
impl<L: Eq, P: Eq, I: Eq> Eq for Act<L, P, I> {}

pub struct SimpleAction<L, P, I> {
    pub path: ApplicablePath<P>,
    pub action: Act<L, P, I>,
}
impl<L: PartialEq, P: PartialEq, I: PartialEq> PartialEq for SimpleAction<L, P, I> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.action == other.action
    }
}
impl<L: Eq, P: Eq, I: Eq> Eq for SimpleAction<L, P, I> {}

impl<L: Debug, P: Debug, I: Debug> Debug for SimpleAction<L, P, I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.action {
            Act::Delete {} => write!(f, "Del {:?}", self.path),
            Act::Update { new } => write!(f, "Upd {:?} {:?}", new, self.path),
            Act::Move { from } => write!(f, "Mov {:?} {:?}", from, self.path),
            Act::MovUpd { from, new } => write!(f, "MoU {:?} {:?} {:?}", from, new, self.path),
            Act::Insert { sub } => write!(f, "Ins {:?} {:?}", sub, self.path),
        }
    }
}

struct InOrderNodes<IdD: Hash + PartialEq + Eq>(Option<Vec<IdD>>, HashSet<IdD>);

/// FEATURE: share parents
static COMPRESSION: bool = false;

struct MidNode<IdC, IdD> {
    parent: IdD,
    compressed: IdC,
    children: Option<Vec<IdD>>,
    action: Option<usize>,
}

impl<IdC: Debug, IdD: Debug> Debug for MidNode<IdC, IdD> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MidNode")
            .field("parent", &self.parent)
            .field("compressed", &self.compressed)
            .field("children", &self.children)
            .finish()
    }
}

pub struct ScriptGenerator<'store, 'a1, 'a2, 'm, IdD, T, SS, SD, S, M, P>
where
    T::Label: Debug,
    T::TreeId: Debug,
    IdD: PrimInt + Debug + Hash + PartialEq + Eq,
    T: 'store + Stored + Labeled + WithChildren,
    SS: DecompressedWithParent<'a1, T, IdD>,
    M: MonoMappingStore<Src = IdD, Dst = IdD>,
{
    pub store: &'store S,
    src_arena_dont_use: &'a1 SS,
    cpy2ori: Vec<IdD>,
    ori2cpy: Vec<usize>,
    mid_arena: Vec<MidNode<T::TreeId, IdD>>, //SuperTreeStore<T::TreeId>,
    mid_root: Vec<IdD>,
    dst_arena: &'a2 SD,
    // ori_to_copy: DefaultMappingStore<IdD>,
    ori_mappings: Option<&'m M>,
    cpy_mappings: M,
    // moved: bitvec::vec::BitVec,
    pub actions: ActionsVec<SimpleAction<T::Label, P, T::TreeId>>,

    src_in_order: InOrderNodes<IdD>,
    dst_in_order: InOrderNodes<IdD>,
}

static MERGE_SIM_ACTIONS: bool = false;

// TODO split IdD in 2 to help typecheck ids
impl<
        'store: 'a1 + 'a2 + 'm,
        'a1: 'm,
        'a2: 'm,
        'm,
        IdD: 'store + PrimInt + Debug + Hash + PartialEq + Eq,
        T: 'store + Tree,
        SS: DecompressedTreeStore<'a1, T, IdD>
            + DecompressedWithParent<'a1, T, IdD>
            + PostOrder<'a1, T, IdD>
            + PostOrderIterable<'a1, T, IdD>
            + Debug,
        SD: DecompressedTreeStore<'a2, T, IdD>
            + DecompressedWithParent<'a2, T, IdD>
            + BreadthFirstIterable<'a2, T, IdD>,
        S,
        M: MonoMappingStore<Src = IdD, Dst = IdD>,
        P: TreePath<Item = T::ChildIdx>,
    > ScriptGenerator<'store, 'a1, 'a2, 'm, IdD, T, SS, SD, S, M, P>
where
    S: NodeStore<T::TreeId, R<'store> = T>,
    T::Label: Debug + Copy,
    T::TreeId: Debug,
    P: From<Vec<T::ChildIdx>> + Debug,
{
    pub fn compute_actions<'a: 'a1 + 'a2, HAST: HyperAST<'store, NS = S>>(
        hyperast: &'store HAST,
        mapping: &'a Mapping<SS, SD, M>,
    ) -> Result<ActionsVec<SimpleAction<T::Label, P, T::TreeId>>, String>
    where
        S: 'store,
    {
        let store = hyperast.node_store();
        Ok(
            ScriptGenerator::new(store, &mapping.src_arena, &mapping.dst_arena)
                .init_cpy(&mapping.mappings)
                .generate()?
                .actions,
        )
    }

    pub fn _compute_actions(
        store: &'store S,
        src_arena: &'a1 SS,
        dst_arena: &'a2 SD,
        ms: &'m M,
    ) -> Result<ActionsVec<SimpleAction<T::Label, P, T::TreeId>>, String> {
        Ok(
            ScriptGenerator::<'store, 'a1, 'a2, 'm, IdD, T, SS, SD, S, M, P>::new(
                store, src_arena, dst_arena,
            )
            .init_cpy(ms)
            .generate()?
            .actions,
        )
    }

    pub fn precompute_actions(
        store: &'store S,
        src_arena: &'a1 SS,
        dst_arena: &'a2 SD,
        ms: &'m M,
    ) -> Self {
        Self::new(store, src_arena, dst_arena).init_cpy(ms)
    }

    fn new(store: &'store S, src_arena: &'a1 SS, dst_arena: &'a2 SD) -> Self {
        Self {
            store,
            src_arena_dont_use: src_arena,
            cpy2ori: vec![],
            ori2cpy: vec![],
            mid_arena: vec![],
            mid_root: vec![],
            dst_arena,
            ori_mappings: None,
            cpy_mappings: Default::default(),
            actions: ActionsVec::new(),
            src_in_order: InOrderNodes(None, Default::default()),
            dst_in_order: InOrderNodes(None, Default::default()),
            // moved: bitvec::bitvec![],
        }
    }

    fn init_cpy(mut self, ms: &'m M) -> Self {
        // copy mapping
        // let now = Instant::now();
        self.ori_mappings = Some(ms);
        self.cpy_mappings = ms.clone();
        // dbg!(&self.src_arena_dont_use);
        // dbg!("aaaaaaaaaaaa");
        // let len = self.src_arena_dont_use.len();
        let root = self.src_arena_dont_use.root();
        // self.moved.resize(len, false);
        for x in self.src_arena_dont_use.iter_df_post::<true>() {
            let children = self.src_arena_dont_use.children(self.store, &x);
            let children = if children.len() > 0 {
                Some(children)
            } else {
                None
            };
            self.mid_arena.push(MidNode {
                parent: self.src_arena_dont_use.parent(&x).unwrap_or(root),
                compressed: self.src_arena_dont_use.original(&x),
                children,
                action: None,
            });
        }
        // self.mid_arena[self.src_arena_dont_use.root().to_usize().unwrap()].parent =
        // self.src_arena_dont_use.root();
        self.mid_root = vec![root];
        // dbg!(&self.mid_arena);
        // let t = now.elapsed().as_secs_f64();
        // dbg!(t);
        self
    }

    pub fn generate(mut self) -> Result<Self, String> {
        // fake root ?
        // fake root link ?
        // let now = Instant::now();
        self.ins_mov_upd()?;
        // let t = now.elapsed().as_secs_f64();
        // dbg!(t);
        // let now = Instant::now();
        self.del();
        // let t = now.elapsed().as_secs_f64();
        // dbg!(t);
        Ok(self)
    }

    fn ins_mov_upd(&mut self) -> Result<(), String> {
        if COMPRESSION {
            todo!()
        }
        self.auxilary_ins_mov_upd()
    }

    fn auxilary_ins_mov_upd(&mut self) -> Result<(), String> {
        for x in self.dst_arena.iter_bf() {
            log::trace!("{:?}", self.actions);
            let w;
            let y = self.dst_arena.parent(&x);
            let z = y.map(|y| self.cpy_mappings.get_src_unchecked(&y));
            if !self.cpy_mappings.is_dst(&x) {
                // insertion
                let k = if let Some(y) = y {
                    Some(self.find_pos(&x, &y))
                } else {
                    None
                };
                w = self.make_inserted_node(&x, &z);
                let ori = self.path_dst(&self.dst_arena.root(), &x);
                let mid = if let Some(z) = z {
                    let p: P = self.path(z).into();
                    p.extend(&[k.unwrap()])
                } else if let Some(k) = k {
                    vec![k].into()
                } else {
                    vec![num_traits::one()].into()
                };
                let path = ApplicablePath { ori, mid };
                let action = SimpleAction {
                    path,
                    action: Act::Insert {
                        sub: self.dst_arena.original(&x),
                    },
                };
                if let Some(z) = z {
                    let z: usize = cast(z).unwrap();
                    if let Some(cs) = self.mid_arena[z].children.as_mut() {
                        cs.insert(cast(k.unwrap()).unwrap(), w);
                    } else {
                        self.mid_arena[z].children = Some(vec![w])
                    }
                    if MERGE_SIM_ACTIONS {
                        if let Some((
                            p_act,
                            Some(SimpleAction {
                                action: Act::Insert { .. },
                                ..
                            }),
                        )) = self.mid_arena[z].action.map(|x| (x, self.actions.get(x)))
                        {
                            self.mid_arena[w.to_usize().unwrap()].action = Some(p_act);
                        } else {
                            self.mid_arena[w.to_usize().unwrap()].action = Some(self.actions.len());
                            self.actions.push(action);
                        }
                    } else {
                        self.mid_arena[w.to_usize().unwrap()].action = Some(self.actions.len());
                        self.actions.push(action);
                    }
                } else {
                    self.mid_root.push(w);
                    self.mid_arena[w.to_usize().unwrap()].action = Some(self.actions.len());
                    self.actions.push(action);
                }
                assert!({
                    self.path(w);
                    true
                });
                // assert_eq!(CompressedTreePath::from(vec![0,0,12]).iter().collect::<Vec<_>>(),vec![0,0,12]);
                // assert_eq!(CompressedTreePath::from(vec![0,0,0,12]).iter().collect::<Vec<_>>(),vec![0,0,0,12]);
                // assert_eq!(CompressedTreePath::from(vec![0,0,0,0,12]).iter().collect::<Vec<_>>(),vec![0,0,0,0,12]);
                // assert_eq!(CompressedTreePath::from(vec![0,0,0,0,0,12]).iter().collect::<Vec<_>>(),vec![0,0,0,0,0,12]);
                // assert_eq!(CompressedTreePath::from(vec![0,0,0,0,0,20]).iter().collect::<Vec<_>>(),vec![0,0,0,0,0,20]);
                // assert_eq!(CompressedTreePath::from(vec![20,0,0,0,0,12]).iter().collect::<Vec<_>>(),vec![20,0,0,0,0,12]);
                // assert_eq!(
                //     self.access(&action.path.mid)
                //         .unwrap_or_else(|_| panic!("wrong insertion path {:?}", &action.path.mid))
                //         ,w
                // );
            } else {
                // dbg!(&self.mid_arena);
                w = self.cpy_mappings.get_src_unchecked(&x);
                let v = {
                    let v = self.mid_arena[w.to_usize().unwrap()].parent;
                    if v == w {
                        None
                    } else {
                        Some(v)
                    }
                };
                let w_l = {
                    let c = &self.mid_arena[w.to_usize().unwrap()].compressed;
                    self.store.resolve(c).try_get_label().cloned()
                };
                let x_l = {
                    let c = &self.dst_arena.original(&x);
                    self.store.resolve(c).try_get_label().cloned()
                };

                if z != v {
                    // move
                    let from = ApplicablePath {
                        ori: self.orig_src(w),
                        mid: self.path(w),
                    };
                    if let Some(z) = z {
                        assert!({
                            self.path(z);
                            let mut z = z;
                            loop {
                                let p = self.mid_arena[z.to_usize().unwrap()].parent;
                                if p == z {
                                    break;
                                } else {
                                    if w == z {
                                        return Err(format!(
                                            "w is a child of z and v, thus w and z cannot be equal, but w={:?} z={:?} v={:?}",
                                            w,z,v));
                                    }
                                    assert_ne!(w, z, "{v:?}");
                                    z = p;
                                }
                            }
                            true
                        });
                    }
                    // remove moved node
                    // TODO do not mutate existing node
                    if let Some(v) = v {
                        let cs = self.mid_arena[v.to_usize().unwrap()]
                            .children
                            .as_mut()
                            .unwrap();
                        let idx = cs.iter().position(|x| x == &w).unwrap();
                        cs.remove(idx);
                    }
                    if let Some(z) = z {
                        assert!({
                            self.path(z);
                            true
                        });
                    }

                    let k = if let Some(y) = y {
                        self.find_pos(&x, &y)
                    } else {
                        num_traits::zero()
                    };
                    let mid = if let Some(z) = z {
                        self.path(z).extend(&[k])
                    } else {
                        vec![k].into()
                    };
                    let ori = self.path_dst(&self.dst_arena.root(), &x);
                    // let ori = if let Some(z) = z {
                    //     self.orig_src(z).extend(&[k])
                    // } else {
                    //     CompressedTreePath::from(vec![k])
                    // };

                    let act = if w_l != x_l {
                        // and also rename
                        // Act::MovUpd {
                        //     from,
                        //     new: x_l.unwrap(),
                        // }
                        let mid = if let Some(z) = z {
                            self.path(z).extend(&[k])
                        } else {
                            vec![k].into()
                        };
                        let ori = self.path_dst(&self.dst_arena.root(), &x);
                        let path = ApplicablePath { ori, mid };
                        let action = SimpleAction {
                            path,
                            action: Act::Update { new: x_l.unwrap() },
                        };
                        // dbg!(&action);
                        self.mid_arena[w.to_usize().unwrap()].compressed =
                            self.dst_arena.original(&x);
                        self.mid_arena[w.to_usize().unwrap()].action = Some(self.actions.len());
                        self.actions.push(action);
                        Act::Move { from }
                    } else {
                        Act::Move { from }
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
                            self.mid_arena[w.to_usize().unwrap()].parent = cast(z).unwrap();
                        } else {
                            self.mid_arena[w.to_usize().unwrap()].parent = cast(w).unwrap();
                        }
                        self.mid_arena[w.to_usize().unwrap()].action = Some(self.actions.len());
                        assert!({
                            self.path(w);
                            true
                        });
                    };
                    if let Act::MovUpd { .. } = act {
                        self.mid_arena[w.to_usize().unwrap()].compressed =
                            self.dst_arena.original(&x);
                    }
                    let path = ApplicablePath { ori, mid };
                    let action = SimpleAction { path, action: act };
                    self.actions.push(action);
                } else if w_l != x_l {
                    // rename
                    let path = ApplicablePath {
                        ori: self.orig_src(w),
                        mid: self.path(w),
                    };
                    let action = SimpleAction {
                        path,
                        action: Act::Update { new: x_l.unwrap() },
                    };
                    self.mid_arena[w.to_usize().unwrap()].compressed = self.dst_arena.original(&x);
                    self.mid_arena[w.to_usize().unwrap()].action = Some(self.actions.len());
                    self.actions.push(action);
                } else {
                    // not changed
                    // and no changes to parents
                    // postentially try to share/map parent in super ast
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
        Ok(())
    }

    fn del(&mut self) {
        let root = *self.mid_root.last().unwrap();
        let mut parent: Vec<(IdD, usize)> = vec![(root, num_traits::zero())];
        // let mut it = Self::iter_mid_in_post_order(*self.mid_root.last().unwrap(), &mut self.mid_arena);
        loop {
            let mut next = None;
            loop {
                let (id, idx) = if let Some(id) = parent.pop() {
                    id
                } else {
                    break;
                };
                let curr = &self.mid_arena[id.to_usize().unwrap()];
                if let Some(cs) = &curr.children {
                    if cs.len() == idx {
                        next = Some(id);
                        break;
                    } else {
                        parent.push((id, idx + 1));
                        parent.push((cs[idx], 0));
                    }
                } else {
                    next = Some(id);
                    break;
                }
            }
            let w = if let Some(w) = next {
                w
            } else {
                break;
            };
            if !self.cpy_mappings.is_src(&w) {
                //todo mutate mid arena ?
                let ori = self.orig_src(w);
                let path = ApplicablePath {
                    ori,
                    mid: self.path(w),
                };
                let v = self.mid_arena[w.to_usize().unwrap()].parent;
                if v != w {
                    let cs = self.mid_arena[v.to_usize().unwrap()]
                        .children
                        .as_mut()
                        .unwrap();
                    let idx = cs.iter().position(|x| x == &w).unwrap();
                    cs.remove(idx);
                    let i = parent.len() - 1;
                    parent[i].1 -= 1;
                } // TODO how to materialize nothing ?
                self.mid_arena[v.to_usize().unwrap()].action = Some(self.actions.len());
                let action = SimpleAction {
                    path,
                    action: Act::Delete {
                        // tree: self.copy_to_orig(w),
                    },
                };
                // TODO self.apply_action(&action, &w);
                self.actions.push(action);
                log::trace!("{:?}", self.actions);
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
        let w_c = self.mid_arena[(*w).to_usize().unwrap()]
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
                if self.ori_mappings.unwrap().has(&a, &b) && !lcs.contains(&(*a, *b)) {
                    let k = self.find_pos(&b, x);
                    let path = ApplicablePath {
                        ori: self.orig_src(*w).extend(&[k]),
                        mid: self.path(*w),
                    };
                    let action = SimpleAction {
                        path,
                        action: Act::Move {
                            from: ApplicablePath {
                                ori: self.orig_src(*a),
                                mid: self.path(*a),
                            },
                        },
                    };
                    // let action = SimpleAction::Move {
                    //     sub: self.ori_to_copy(*a),
                    //     parent: Some(*x),
                    //     idx: k,
                    // };
                    // self.apply_action(&action, &self.ori_to_copy(*a));
                    let z: usize = cast(*w).unwrap();
                    let cs = self.mid_arena[z.to_usize().unwrap()]
                        .children
                        .as_mut()
                        .unwrap();
                    let idx = cs.iter().position(|x| x == a).unwrap();
                    cs.remove(idx);
                    if let Some(cs) = self.mid_arena[z].children.as_mut() {
                        let k = cast(k).unwrap();
                        if k < cs.len() {
                            cs.insert(k, *a)
                        } else {
                            cs.push(*a)
                        }
                    } else {
                        self.mid_arena[z].children = Some(vec![*a])
                    };
                    self.mid_arena[a.to_usize().unwrap()].parent = cast(z).unwrap();
                    self.mid_arena[a.to_usize().unwrap()].action = Some(self.actions.len());
                    assert!({
                        self.path(*a);
                        true
                    });
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
        let xpos = cast(self.dst_arena.position_in_parent(x).unwrap()).unwrap(); //child.positionInParent();
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
            let p = self.mid_arena[u.to_usize().unwrap()].parent;
            let r = self.mid_arena[p.to_usize().unwrap()]
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
                src_children[m.0.to_usize().unwrap()],
                dst_children[m.1.to_usize().unwrap()],
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
            action: None,
        });
        // self.moved.push(false);
        let (src_to_dst_l, dst_to_src_l) = self.cpy_mappings.capacity();
        self.cpy_mappings.topit(src_to_dst_l, dst_to_src_l);
        self.cpy_mappings.link(w, *x);
        w
    }

    fn copy_to_orig(&self, w: IdD) -> IdD {
        if self.src_arena_dont_use.len() <= cast(w).unwrap() {
            let w: usize = cast(w).unwrap();
            return self.cpy2ori[w - self.src_arena_dont_use.len()];
        }
        w
    }

    fn orig_src(&self, v: IdD) -> P {
        self.src_arena_dont_use
            .path(&self.src_arena_dont_use.root(), &self.copy_to_orig(v))
            .into()
    }

    fn path_dst(&self, root: &IdD, x: &IdD) -> P {
        let mut r = vec![];
        let mut x = *x;
        loop {
            let p = self.dst_arena.parent(&x);
            if let Some(p) = p {
                r.push(self.dst_arena.position_in_parent(&x).unwrap());
                x = p
            } else {
                assert_eq!(root, &x);
                break;
            }
        }
        r.reverse();
        // dbg!(&r.iter().map(|x| x.to_usize()).collect::<Vec<_>>());
        r.into()
    }

    fn path(&self, mut z: IdD) -> P {
        let mut r = vec![];
        loop {
            let p = self.mid_arena[z.to_usize().unwrap()].parent;
            if p == z {
                let i = self
                    .mid_root
                    .iter()
                    .position(|x| x == &z)
                    .expect("expect the position of z in children of mid_root");
                r.push(cast(i).unwrap());
                break;
            } else {
                let i = self.mid_arena[p.to_usize().unwrap()]
                    .children
                    .as_ref()
                    .expect(
                        "parent should have children, current node should actually be one of them",
                    )
                    .iter()
                    .position(|x| x == &z)
                    .expect("expect the position of z in children of p");
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
    mid_arena: &'a mut [MidNode<IdC, IdD>],
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
            let curr = &self.mid_arena[id.to_usize().unwrap()];
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

impl<IdD: Hash + Eq + Clone> InOrderNodes<IdD> {
    /// TODO add precondition to try to linerarly remove element (if both ordered the same way it's easy to remove without looking at lists multiple times)
    /// Maybe use a bloom filter with a collision set ? do we have a good estimate of the number of element to store ?
    fn remove_all(&mut self, w: &[IdD]) {
        w.iter().for_each(|x| {
            self.1.remove(x);
        });
        // if let Some(a) = self.0.take() {
        //     let mut i = 0;
        //     let c: Vec<IdD> = a
        //         .into_iter()
        //         .filter(|x| {
        //             // if i < w.len() && !w[i..].contains(x) {
        //             //     i += 1;
        //             //     true
        //             // } else {
        //             //     false
        //             // }
        //             !w.contains(x)
        //         })
        //         .collect();

        //     assert_eq!(c.len(), self.1.len());
        //     if c.len() > 0 {
        //         self.0 = Some(c);
        //     }
        // } else {
        //     assert!(self.1.is_empty());
        // }
    }

    pub(crate) fn push(&mut self, x: IdD) {
        self.1.insert(x.clone());
        // if let Some(l) = self.0.as_mut() {
        //     if !l.contains(&x) {
        //         l.push(x)
        //     }
        //     assert_eq!(l.len(), self.1.len());
        // } else {
        //     assert_eq!(1, self.1.len());
        //     self.0 = Some(vec![x])
        // }
    }

    fn contains(&self, x: &IdD) -> bool {
        self.1.contains(x)
        // if let Some(l) = &self.0 {
        //     let r = l.contains(x);
        //     assert_eq!(r, self.1.contains(x));
        //     r
        // } else {
        //     assert!(self.1.is_empty());
        //     false
        // }
    }
}
