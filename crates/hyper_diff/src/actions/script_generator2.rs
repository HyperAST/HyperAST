#![allow(unused)] // WIP
/// inspired by the implementation in gumtree
/// WIP
use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    hash::Hash,
};

use hyperast::PrimInt;
use num_traits::{ToPrimitive, cast};

use super::action_vec::ActionsVec;
use crate::{
    actions::Actions,
    decompressed_tree_store::{
        BreadthFirstIterable, DecompressedTreeStore, DecompressedWithParent, PostOrder,
        PostOrderIterable,
    },
    matchers::{Mapping, mapping_store::MonoMappingStore},
    tree::tree_path::TreePath,
    utils::sequence_algorithms::longest_common_subsequence,
};
use hyperast::types::{HyperAST, Labeled};

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
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

impl<L: Debug, P: Debug, I: Debug> super::action_tree::NodeSummary
    for super::action_tree::Node<SimpleAction<L, P, I>>
{
    fn pretty(&self) -> impl std::fmt::Display + '_ {
        struct D<'a, L: Debug, P: Debug, I: Debug>(
            &'a super::action_tree::Node<SimpleAction<L, P, I>>,
        );
        impl<'a, L: Debug, P: Debug, I: Debug> Display for D<'a, L, P, I> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let t = aux(self.0);
                let a = match self.0.action.action {
                    Act::Delete {} => "Delete",
                    Act::Update { .. } => "Update",
                    Act::Move { .. } => "Move",
                    Act::MovUpd { .. } => "MovUpd",
                    Act::Insert { .. } => "Insert",
                };
                let ori = &self.0.action.path.ori;
                write!(
                    f,
                    "{} {:?} d:{} u:{} m:{} M:{} i:{}",
                    a, ori, t.0, t.1, t.2, t.3, t.4
                )
            }
        }
        struct T(usize, usize, usize, usize, usize);
        impl std::ops::Add for T {
            type Output = T;

            fn add(self, rhs: Self) -> Self::Output {
                T(
                    self.0 + rhs.0,
                    self.1 + rhs.1,
                    self.2 + rhs.2,
                    self.3 + rhs.3,
                    self.4 + rhs.4,
                )
            }
        }
        impl std::iter::Sum for T {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(T(0, 0, 0, 0, 0), std::ops::Add::add)
            }
        }
        fn aux<L: Debug, P: Debug, I: Debug>(
            s: &super::action_tree::Node<SimpleAction<L, P, I>>,
        ) -> T {
            let t = match &s.action.action {
                Act::Delete {} => T(1, 0, 0, 0, 0),
                Act::Update { .. } => T(0, 1, 0, 0, 0),
                Act::Move { .. } => T(0, 0, 1, 0, 0),
                Act::MovUpd { .. } => T(0, 0, 0, 1, 0),
                Act::Insert { .. } => T(0, 0, 0, 0, 1),
            };
            t + s.children.iter().map(|x| aux(x)).sum()
        }
        D(self)
    }
}

struct InOrderNodes<IdD: Hash + PartialEq + Eq>(Option<Vec<IdD>>, HashSet<IdD>);

/// FEATURE: share parents
static COMPRESSION: bool = false;
static SUBTREE_DEL: bool = true;

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

pub struct ScriptGenerator<'a1, 'a2, 'm, IdD, SS, SD, HAST, M, P>
where
    HAST: HyperAST,
    HAST::Label: Debug,
    HAST::IdN: Debug,
    IdD: PrimInt + Debug + Hash + PartialEq + Eq,
    // T: Stored + Labeled + WithChildren,
    // SS: DecompressedWithParent<T, IdD>,
    M: MonoMappingStore<Src = IdD, Dst = IdD>,
{
    pub store: HAST,
    src_arena_dont_use: &'a1 SS,
    cpy2ori: Vec<IdD>,
    #[allow(unused)]
    // TODO remove it after making sure it is not needed to construct an action_tree
    ori2cpy: Vec<usize>,
    mid_arena: Vec<MidNode<HAST::IdN, IdD>>, //SuperTreeStore<HAST::IdN>,
    mid_root: Vec<IdD>,
    dst_arena: &'a2 SD,
    // ori_to_copy: DefaultMappingStore<IdD>,
    ori_mappings: Option<&'m M>,
    cpy_mappings: M,
    // moved: bitvec::vec::BitVec,
    dirty: bitvec::vec::BitVec,
    pub actions: ActionsVec<SimpleAction<HAST::Label, P, HAST::IdN>>,

    src_in_order: InOrderNodes<IdD>,
    dst_in_order: InOrderNodes<IdD>,
}

static MERGE_SIM_ACTIONS: bool = false;

// TODO split IdD in 2 to help typecheck ids
impl<
    'a1: 'm,
    'a2: 'm,
    'm,
    IdD: PrimInt + Debug + Hash + PartialEq + Eq,
    SS: DecompressedTreeStore<HAST, IdD>
        + DecompressedWithParent<HAST, IdD>
        + PostOrder<HAST, IdD>
        + PostOrderIterable<HAST, IdD>
        + Debug,
    SD: DecompressedTreeStore<HAST, IdD>
        + DecompressedWithParent<HAST, IdD>
        + BreadthFirstIterable<HAST, IdD>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore<Src = IdD, Dst = IdD> + Default + Clone,
    P: TreePath<Item = HAST::Idx>,
> ScriptGenerator<'a1, 'a2, 'm, IdD, SS, SD, HAST, M, P>
where
    HAST::Label: Debug + Eq + Copy,
    HAST::IdN: Debug + Clone,
    P: From<Vec<HAST::Idx>> + Debug,
{
    pub fn compute_actions<'a: 'a1 + 'a2>(
        hast: HAST,
        mapping: &'a Mapping<SS, SD, M>,
    ) -> Result<ActionsVec<SimpleAction<HAST::Label, P, HAST::IdN>>, String> {
        Ok(
            ScriptGenerator::new(hast, &mapping.src_arena, &mapping.dst_arena)
                .init_cpy(&mapping.mappings)
                .generate()?
                .actions,
        )
    }
}
// TODO split IdD in 2 to help typecheck ids
impl<
    'a1: 'm,
    'a2: 'm,
    'm,
    IdD: PrimInt + Debug + Hash + PartialEq + Eq,
    SS: DecompressedTreeStore<HAST, IdD>
        + DecompressedWithParent<HAST, IdD>
        + PostOrder<HAST, IdD>
        + PostOrderIterable<HAST, IdD>
        + Debug,
    SD: DecompressedTreeStore<HAST, IdD>
        + DecompressedWithParent<HAST, IdD>
        + BreadthFirstIterable<HAST, IdD>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore<Src = IdD, Dst = IdD> + Default + Clone,
    P: TreePath<Item = HAST::Idx>,
> ScriptGenerator<'a1, 'a2, 'm, IdD, SS, SD, HAST, M, P>
where
    HAST::Label: Debug + Eq + Copy,
    HAST::IdN: Debug,
    P: From<Vec<HAST::Idx>> + Debug,
{
    pub fn new(store: HAST, src_arena: &'a1 SS, dst_arena: &'a2 SD) -> Self {
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
            dirty: Default::default(),
            actions: ActionsVec::new(),
            src_in_order: InOrderNodes(None, Default::default()),
            dst_in_order: InOrderNodes(None, Default::default()),
            // moved: bitvec::bitvec![],
        }
    }
}
// TODO split IdD in 2 to help typecheck ids
impl<
    'a1: 'm,
    'a2: 'm,
    'm,
    IdD: PrimInt + Debug + Hash + PartialEq + Eq,
    SS: DecompressedTreeStore<HAST, IdD>
        + DecompressedWithParent<HAST, IdD>
        + PostOrder<HAST, IdD>
        + PostOrderIterable<HAST, IdD>
        + Debug,
    SD: DecompressedTreeStore<HAST, IdD>
        + DecompressedWithParent<HAST, IdD>
        + BreadthFirstIterable<HAST, IdD>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore<Src = IdD, Dst = IdD> + Default + Clone,
    P: TreePath<Item = HAST::Idx>,
> ScriptGenerator<'a1, 'a2, 'm, IdD, SS, SD, HAST, M, P>
where
    HAST::Label: Debug + Eq + Copy,
    HAST::IdN: Debug,
    P: From<Vec<HAST::Idx>> + Debug,
{
    pub fn init_cpy(mut self, ms: &'m M) -> Self {
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
            let children = self.src_arena_dont_use.children(&x);
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
            self.dirty.push(false);
        }
        // self.mid_arena[self.src_arena_dont_use.root().to_usize().unwrap()].parent =
        // self.src_arena_dont_use.root();
        self.mid_root = vec![root];
        // dbg!(&self.mid_arena);
        // let t = now.elapsed().as_secs_f64();
        // dbg!(t);
        self
    }
}
// TODO split IdD in 2 to help typecheck ids
impl<
    'a1: 'm,
    'a2: 'm,
    'm,
    IdD: PrimInt + Debug + Hash + PartialEq + Eq,
    SS: DecompressedTreeStore<HAST, IdD>
        + DecompressedWithParent<HAST, IdD>
        + PostOrder<HAST, IdD>
        + PostOrderIterable<HAST, IdD>
        + Debug,
    SD: DecompressedTreeStore<HAST, IdD>
        + DecompressedWithParent<HAST, IdD>
        + BreadthFirstIterable<HAST, IdD>,
    HAST: HyperAST + Copy,
    M: MonoMappingStore<Src = IdD, Dst = IdD> + Default + Clone,
    P: TreePath<Item = HAST::Idx>,
> ScriptGenerator<'a1, 'a2, 'm, IdD, SS, SD, HAST, M, P>
where
    HAST::Label: Debug + Eq + Copy,
    HAST::IdN: Debug,
    P: From<Vec<HAST::Idx>> + Debug,
{
    pub fn _compute_actions(
        store: HAST,
        src_arena: &'a1 SS,
        dst_arena: &'a2 SD,
        ms: &'m M,
    ) -> Result<ActionsVec<SimpleAction<HAST::Label, P, HAST::IdN>>, String> {
        Ok(
            ScriptGenerator::<'a1, 'a2, 'm, IdD, SS, SD, HAST, M, P>::new(
                store, src_arena, dst_arena,
            )
            .init_cpy(ms)
            .generate()?
            .actions,
        )
    }

    pub fn precompute_actions(
        store: HAST,
        src_arena: &'a1 SS,
        dst_arena: &'a2 SD,
        ms: &'m M,
    ) -> Self {
        Self::new(store, src_arena, dst_arena).init_cpy(ms)
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
        self.auxilary_ins_mov_upd(&|_, _| ())
    }

    pub fn auxilary_ins_mov_upd(
        &mut self,
        f: &impl Fn(&HAST::IdN, &HAST::IdN),
    ) -> Result<(), String> {
        for x in self.dst_arena.iter_bf() {
            // log::trace!("{:?}", self.actions);
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
                    if v == w { None } else { Some(v) }
                };
                let w_t;
                let x_t;
                let w_l = {
                    let c = self.mid_arena[w.to_usize().unwrap()].compressed.clone();
                    w_t = c.clone();
                    self.store.resolve(&c).try_get_label().cloned()
                };
                let x_l = {
                    let c = self.dst_arena.original(&x).clone();
                    x_t = c.clone();
                    self.store.resolve(&c).try_get_label().cloned()
                };
                f(&w_t, &x_t);

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
                                            w, z, v
                                        ));
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
                        let _v: &mut MidNode<HAST::IdN, IdD> =
                            &mut self.mid_arena[v.to_usize().unwrap()];
                        let cs = _v.children.as_mut().unwrap();
                        let idx = cs.iter().position(|x| x == &w).unwrap();
                        cs.remove(idx);
                        self.dirty.set(v.to_usize().unwrap(), true);
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

    pub fn del(&mut self) {
        let root = *self.mid_root.last().unwrap();
        struct Ele<IdD, Idx, W> {
            // id in arena
            id: IdD,
            // curent child offset
            idx: Idx,
            // true if not only deletes
            // TODO use a bitset to reduce mem.
            w: Vec<W>,
        }
        impl<IdD, Idx: num_traits::Zero, W> Ele<IdD, Idx, W> {
            fn new(id: IdD) -> Self {
                let idx = num_traits::zero();
                let _b = false;
                let w = vec![];
                Self { id, idx, w }
            }
        }
        impl<IdD, Idx: num_traits::PrimInt, W> Ele<IdD, Idx, W> {
            fn inc(mut self) -> Self {
                self.idx = self.idx + num_traits::one();
                self
            }
        }
        let mut parent: Vec<Ele<IdD, usize, _>> = vec![Ele::new(root)];
        loop {
            let next;
            let waiting;
            loop {
                let Some(ele) = parent.pop() else {
                    next = None;
                    waiting = vec![];
                    break;
                };
                let id = ele.id.to_usize().unwrap();
                let curr: &MidNode<HAST::IdN, IdD> = &self.mid_arena[id];
                let Some(cs) = &curr.children else {
                    next = Some(ele.id);
                    waiting = ele.w;
                    if curr.action.is_some() {
                        // dbg!(curr.action);
                    }
                    break;
                };
                if cs.len() == ele.idx {
                    next = Some(ele.id);
                    waiting = ele.w;
                    break;
                }
                let child = cs[ele.idx];
                parent.push(ele.inc());
                parent.push(Ele::new(child));
            }
            let Some(w) = next else {
                break;
            };
            if self.dirty[w.to_usize().unwrap()] {
                // dbg!(w);
            }
            if !self.cpy_mappings.is_src(&w) {
                //todo mutate mid arena ?
                let ori = self.orig_src(w);
                let mid = self.path(w);
                // dbg!(&mid);
                let path = ApplicablePath { ori, mid };
                let _w: &mut MidNode<HAST::IdN, IdD> = &mut self.mid_arena[w.to_usize().unwrap()];
                let v = _w.parent;
                let _v: &mut MidNode<HAST::IdN, IdD> = &mut self.mid_arena[v.to_usize().unwrap()];
                if v != w {
                    let cs = _v.children.as_mut().unwrap();
                    let idx = cs.iter().position(|x| x == &w).unwrap();
                    cs.remove(idx);
                    let i = parent.len() - 1;
                    parent[i].idx -= 1;
                } // TODO how to materialize nothing ?
                _v.action = Some(self.actions.len());
                // TODO self.apply_action(&action, &w);
                let action = SimpleAction {
                    path,
                    action: Act::Delete {},
                };
                if SUBTREE_DEL {
                    if self.dirty[w.to_usize().unwrap()] {
                        // non uniform del.
                        // dbg!(waiting.len());
                        self.actions.extend(waiting);
                        log::trace!("{:?}", action);
                        self.actions.push(action);
                        // transitively
                        self.dirty.set(v.to_usize().unwrap(), true);
                    } else if let Some(i) = parent.len().checked_sub(1) {
                        // dbg!(waiting.len());
                        // uniform, so wait in parent
                        parent[i].w.push(action);
                    } else {
                        // dbg!(waiting.len());
                        log::trace!("{:?}", action);
                        self.actions.push(action);
                    }
                } else {
                    log::trace!("{:?}", action);
                    self.actions.push(action);
                }
            } else {
                if SUBTREE_DEL {
                    self.actions.extend(waiting);
                }
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
        let x_c = self.dst_arena.children(x);
        self.dst_in_order.remove_all(x_c.as_slice());

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
    pub(crate) fn find_pos(&self, x: &IdD, y: &IdD) -> HAST::Idx {
        let siblings = self.dst_arena.children(y);

        for c in &siblings {
            if self.dst_in_order.contains(c) {
                if c == x {
                    return num_traits::zero();
                } else {
                    break;
                }
            }
        }
        let xpos: usize = self.dst_arena.position_in_parent(x).unwrap(); //child.positionInParent();
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
        let upos: HAST::Idx = {
            let p = self.mid_arena[u.to_usize().unwrap()].parent;
            let r = self.mid_arena[p.to_usize().unwrap()]
                .children
                .as_ref()
                .unwrap()
                .iter()
                .position(|y| *y == u)
                .unwrap();
            cast::<usize, HAST::Idx>(r).unwrap()
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
        self.dirty.push(false);
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

impl<'a, IdC, IdD: PrimInt> Iterator for Iter<'a, IdC, IdD> {
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
