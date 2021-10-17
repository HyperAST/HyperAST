use std::{
    collections::HashMap, fmt::Debug, marker::PhantomData, num::NonZeroU16, ops::Deref,
    ptr::NonNull,
};

use bitvec::order::Lsb0;
use num_traits::{cast, PrimInt};

use crate::{
    matchers::{
        decompressed_tree_store::{
            BreathFirst, BreathFirstContigousSiblings, DecompressedTreeStore,
            DecompressedWithParent,
        },
        mapping_store::{DefaultMappingStore, MappingStore, MonoMappingStore},
    },
    tree::tree::Stored,
    utils::sequence_algorithms::longest_common_subsequence,
};

pub trait Actions {
    fn len(&self) -> usize;
}

pub struct ActionsVec<A>(Vec<A>);

#[derive(PartialEq, Eq)]
pub enum SimpleAction<Src, Dst> {
    Delete {
        tree: Src,
    },
    Update {
        src: Src,
        dst: Dst,
        label: Label,
    },
    Move {
        sub: Src,
        parent: Dst,
        idx: usize,
    },
    Insert {
        sub: Src,
        parent: Dst,
        idx: usize,
    },
    // Duplicate { sub: Src, parent: Dst, idx: usize },
    MoveUpdate {
        sub: Src,
        parent: Dst,
        idx: usize,
        label: Label,
    },
}

impl<IdD> Actions for ActionsVec<IdD> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

pub trait TestActions<IdD> {
    fn has_items(&self, items: &[SimpleAction<IdD, IdD>]) -> bool;
}

impl<IdD: std::cmp::PartialEq> TestActions<IdD> for ActionsVec<SimpleAction<IdD, IdD>> {
    fn has_items(&self, items: &[SimpleAction<IdD, IdD>]) -> bool {
        items.iter().all(|x| self.0.contains(x))
    }
}

/// try to use it to differentiate src and dst situations
trait Duet {
    type Src;
    type Dst;
}

pub struct ScriptGenerator<
    IdD: PrimInt + Debug,
    T: Stored,
    SS, //:DecompressedTreeStore<T::TreeId, IdD> + DecompressedWithParent<IdD>,
    SD, //:DecompressedTreeStore<T::TreeId, IdD> + DecompressedWithParent<IdD>,
> {
    origMappings: DefaultMappingStore<IdD>,
    origDst: IdD,
    origSrc: IdD,
    src_arena: SS,
    mid_arena: SuperTreeStore,
    dst_arena: SD,
    cpyMappings: DefaultMappingStore<IdD>,
    inserted: Vec<Inserted<T::TreeId, IdD>>,
    actions: ActionsVec<SimpleAction<IdD, IdD>>,

    srcInOrder: InOrderNodes<IdD>,
    dstInOrder: InOrderNodes<IdD>,

    phantom: PhantomData<*const T>,
}

struct Inserted<IdC, IdD> {
    original: IdC,
    parent: IdD,
}

struct InOrderNodes<IdD> {
    a: IdD,
}

struct SemVer {
    major: u8,
    minor: u8,
    patch: u8,
}

struct CommmitId(String);

pub trait VecStore {}

pub struct DenseVecStore<Idx, V> {
    v: Vec<V>,
    phantom: PhantomData<*const Idx>,
}

impl<Idx: ReservableIndex, T> core::ops::Index<Idx::Reserved> for DenseVecStore<Idx, T> {
    type Output = T;

    fn index(&self, index: Idx::Reserved) -> &Self::Output {
        &self.v[index.into()]
    }
}

pub struct SparseVecStore<Idx, V> {
    v: Vec<(Idx, V)>,
    phantom: PhantomData<*const Idx>,
}

mod internal {
    use super::*;
    pub trait InternalReservable: From<Self::T> + Into<usize> {
        type T;
    }
    impl<T: PrimInt> InternalReservable for Restricted<T> {
        type T = T;
    }
}

pub trait Reservable: internal::InternalReservable + Copy {}

#[derive(Copy, Clone)]
struct Restricted<T>(T);

impl<T: PrimInt> From<T> for Restricted<T> {
    fn from(x: T) -> Self {
        Restricted(cast(x).unwrap())
    }
}

impl<T: PrimInt> Into<usize> for Restricted<T> {
    fn into(self) -> usize {
        cast(self.0).unwrap()
    }
}

impl<T: PrimInt> Reservable for Restricted<T> {}

pub trait ReservableIndex {
    type Reserved: Reservable;
    type Unpacked;
    fn value(&self) -> Self::Unpacked;
}

struct VersionIndex<Idx> {
    value: Idx,
    phantom: PhantomData<*const Idx>,
}

enum UnpackedVersionIndex<Idx> {
    FirstCommit,
    Default(Idx),
    LastCommit,
}

impl<Idx: PrimInt> ReservableIndex for VersionIndex<Idx> {
    type Reserved = Restricted<Idx>;
    type Unpacked = UnpackedVersionIndex<Self::Reserved>;

    fn value(&self) -> Self::Unpacked {
        if self.value == num_traits::Bounded::max_value() {
            Self::Unpacked::FirstCommit
        } else if self.value + num_traits::one() == num_traits::Bounded::max_value() {
            Self::Unpacked::LastCommit
        } else {
            Self::Unpacked::Default(Restricted(self.value))
        }
    }
}

struct Versions<IdV: ReservableIndex> {
    // names: DenseVecStore<IdV, SemVer>,
    commits: DenseVecStore<IdV, CommmitId>,
    first_parents: DenseVecStore<IdV, IdV>,
    second_parents: SparseVecStore<IdV, IdV>,
    other_parents: SparseVecStore<IdV, IdV>,
}

fn f(x: Versions<VersionIndex<u16>>) {
    let b = Restricted::from(0);
    let a = &x.commits[b];
    let c = &x.first_parents[b];
    let d = match c.value() {
        UnpackedVersionIndex::FirstCommit => UnpackedVersionIndex::FirstCommit,
        UnpackedVersionIndex::Default(i) => UnpackedVersionIndex::Default(&x.first_parents[i]),
        UnpackedVersionIndex::LastCommit => UnpackedVersionIndex::LastCommit,
    };
}

struct MegaTreeStore {
    projects: Vec<SuperTreeStore>,
}
struct Versioned<IdV, T> {
    insert: VersionIndex<IdV>,
    delete: VersionIndex<IdV>,
    content: T,
}
struct Descendant<T> {
    path: CompressedTreePath<u16>,
    tree: T,
}
type IdC = u32;
type IdV = u16;
enum SuperTree {
    InsertionsPhase {
        node: Box<SuperTree>,
        insert: VersionIndex<IdV>,
        descendants: Vec<Descendant<IdC>>,
    },
    ManyVersion {
        node: IdC,
        children: Vec<Versioned<IdV, SuperTree>>,
    },
    ManyFarVersion {
        node: IdC,
        descendants: Vec<Versioned<IdV, Descendant<SuperTree>>>,
    },
    Far {
        node: IdC,
        descendants: Vec<Descendant<SuperTree>>,
    },
    FixedChildren {
        node: IdC,
        children: Box<[SuperTree]>,
    },
    CompressedFixedDiamond {
        node: IdC,
        children: Box<[Versioned<IdV, IdC>]>,
    },
    Basic {
        node: IdC,
    },
}
struct SuperTreeStore {
    versions: Versions<VersionIndex<u16>>,
    root: SuperTree,
    // can always be used as src ?
    // can "split" actions
    // should be easy to traverse in post order if used as src
    // should be easy to traverse in bfs is used as dst
    // should be able to insert new subtrees
    //                   delete old ones
    //                   materialize moves
    //                   duplicates
    // should allow easy reserialize at any version
    //        or combination of elements from different versions

    // a good middle ground would be to use Rc<> for higher nodes
    // also maybe nodes with a path, thus no need to dup nodes not changed
}

impl SuperTreeStore {
    fn from_version_and_path(
        &self,
        version: VersionIndex<u16>,
        path: CompressedTreePath<u32>,
    ) -> SuperTree {
        // self.root.;

        todo!()
    }
    // post_order accessors
    // *****_in_post_order

    // bfs accessors
}

/// id for nodes in multi ast
// type IdM = u32;
type Label = u16;

/// FEATURE: share parents
const COMPRESSION: bool = false;

impl<
        IdD: PrimInt + Debug,
        T: Stored,
        // SS: DecompressedTreeStore<T::TreeId, IdD> + DecompressedWithParent<IdD> +,
        // SD: DecompressedTreeStore<T::TreeId, IdD> + DecompressedWithParent<IdD> + BreathFirst,
    > ScriptGenerator<IdD, T, SS<IdD>, SD<IdD>>
{
    pub fn compute_actions(ms: DefaultMappingStore<IdD>) -> ActionsVec<SimpleAction<IdD, IdD>> {
        let mut s = Self::new();
        s.init_cpy(ms);
        s.generate();
        s.actions
    }

    fn new() -> Self {
        todo!()
    }

    fn init_cpy(&mut self, ms: DefaultMappingStore<IdD>) {
        todo!();
        // copy mapping
        // copy src
        // relate src to copied src
    }

    fn generate(&mut self) {
        // fake root ?
        // fake root link ?

        self.ins_mov_upd();

        self.del();
    }

    fn ins_mov_upd(&mut self) {
        if COMPRESSION {
            todo!()
        }
        self.auxilary_ins_mov_upd();
    }

    fn auxilary_ins_mov_upd(&mut self) {
        for x in self.bfs_dst() {
            let w;
            let y = self.dst_arena.parent(&x).unwrap();
            let z = self.cpyMappings.get_src(&y);

            if self.cpyMappings.is_dst(&x) {
                // insertion
                let k = self.findPos(&x, &y);
                w = self.make_inserted_node(&x, &z, &k);
                // self.apply_insert(&w, &z, &k);
                self.cpyMappings.link(w, x);
                let action = SimpleAction::Insert {
                    sub: w,
                    parent: z,
                    idx: k,
                };
                self.apply_insert(&action);
                self.actions.push(action);
            } else {
                w = self.cpyMappings.get_src(&x);
                if x != self.origDst {
                    let v = self.src_arena.parent(&w).unwrap();
                    let w_l = self.src_arena.label(&w);
                    let x_l = self.dst_arena.label(&x);

                    if w_l != x_l && z != v {
                        // rename + move
                        let k = self.findPos(&x, &y);
                        // self.apply_insert(&w, &z, &k);
                        self.cpyMappings.link(w, x);
                        let action = SimpleAction::MoveUpdate {
                            sub: x,
                            parent: z,
                            idx: k,
                            label: x_l,
                        };
                        self.apply_insert(&action);
                        self.actions.push(action);
                    } else if w_l != x_l {
                        // rename
                        self.cpyMappings.link(w, x);
                        // self.apply_update(&w, &z, &x_l);
                        let action = SimpleAction::Update {
                            src: w,
                            dst: x,
                            label: x_l,
                        };
                        self.apply_update(&action);
                        self.actions.push(action);
                    } else if z != v {
                        // move
                        let k = self.findPos(&x, &y);
                        // self.apply_insert(&w, &z, &k);
                        self.cpyMappings.link(w, x);
                        let action = SimpleAction::Move {
                            sub: x,
                            parent: z,
                            idx: k,
                        };
                        self.apply_insert(&action);
                        self.actions.push(action);
                    } else {
                        // not changed
                        // and no changes to parents
                        // postentially try to share parent in super ast
                        if COMPRESSION {
                            todo!()
                        }
                    }
                    self.mdForMiddle(&x, &w);
                }
            }

            self.srcInOrder.push(w);
            self.dstInOrder.push(x);
            self.alignChildren(&w, &x);
        }
    }

    fn del(&self) {
        for w in self.iterCpySrcInPostOrder() {
            if self.cpyMappings.is_src(&w) {
                let action = SimpleAction::Delete { tree: w };
                self.apply_delete(&action);
                self.actions.push(action);
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

    pub(crate) fn alignChildren(&mut self, w: &IdD, x: &IdD) {
        let w_c = self.src_arena.children(w);
        self.srcInOrder.removeAll(&w_c);
        let x_c = self.dst_arena.children(x);
        self.dstInOrder.removeAll(&x_c);

        let mut s1 = vec![];
        for c in &w_c {
            if self.cpyMappings.is_src(c) {
                if w_c.contains(&self.cpyMappings.get_src(c)) {
                    s1.push(*c);
                }
            }
        }
        let mut s2 = vec![];
        for c in &x_c {
            if self.cpyMappings.is_dst(c) {
                if x_c.contains(&self.cpyMappings.get_dst(c)) {
                    s2.push(*c);
                }
            }
        }

        let lcs = self.lcs(&s1, &s2);

        for m in &lcs {
            self.srcInOrder.push(m.0);
            self.dstInOrder.push(m.1);
        }
        for a in &s1 {
            for b in &s2 {
                if self.cpyMappings.has(&a, &b) && !lcs.contains(&(*a, *b)) {
                    let k = self.findPos(b, x);
                    let action = SimpleAction::Move {
                        sub: *a,
                        parent: *w,
                        idx: k,
                    };
                    self.apply_move(&action);
                    self.actions.push(action);
                    self.srcInOrder.push(*a);
                    self.dstInOrder.push(*b);
                }
            }
        }
    }

    /// find position of x in parent on dst_arena
    pub(crate) fn findPos(&self, x: &IdD, parent: &IdD) -> usize {
        let y = parent;
        let siblings = self.dst_arena.children(y);

        for c in &siblings {
            if self.dstInOrder.contains(c) {
                if c == x {
                    return 0;
                } else {
                    break;
                }
            }
        }
        let xpos = self.src_arena.child_postion(x); //child.positionInParent();
        let mut v: Option<IdD> = None;
        for i in 0..xpos {
            let c: &IdD = &siblings[i];
            if self.dstInOrder.contains(c) {
                v = Some(*c);
            };
        }

        if v.is_none() {
            return 0;
        }

        let u = self.cpyMappings.get_src(&v.unwrap());
        let upos = self.src_arena.child_postion(&u);
        upos + 1
    }

    pub(crate) fn lcs(&self, src_children: &[IdD], dst_children: &[IdD]) -> Vec<(IdD, IdD)> {
        longest_common_subsequence(src_children, dst_children, |src, dst| {
            self.cpyMappings.has(src, dst)
        })
    }

    pub(crate) fn mdForMiddle(&self, x: &IdD, w: &IdD) {
        // todo maybe later
    }

    pub(crate) fn make_inserted_node(&self, x: &IdD, z: &IdD, k: &usize) -> IdD {
        todo!()
    }

    pub(crate) fn apply_insert(&self, a: &SimpleAction<IdD, IdD>) {
        todo!()
    }

    pub(crate) fn apply_update(&self, a: &SimpleAction<IdD, IdD>) {
        todo!()
    }

    pub(crate) fn apply_delete(&self, a: &SimpleAction<IdD, IdD>) {
        todo!()
    }

    pub(crate) fn apply_move(&self, action: &SimpleAction<IdD, IdD>) {
        // let oldk = self.src_arena.child_postion(&a);
        todo!()
    }

    pub(crate) fn iterCpySrcInPostOrder(&self) -> Vec<IdD> {
        todo!()
    }

    fn bfs_dst(&self) -> Vec<IdD> {
        todo!()
    }
}

pub(crate) struct SS<IdD> {
    a: IdD,
}
pub(crate) struct SD<IdD> {
    a: IdD,
}

impl<IdD> SD<IdD> {
    pub(crate) fn parent(&self, x: &IdD) -> Option<IdD> {
        todo!()
    }

    pub(crate) fn label(&self, x: &IdD) -> Label {
        todo!()
    }

    pub(crate) fn children(&self, x: &IdD) -> Vec<IdD> {
        todo!()
    }
}
impl<IdD> SS<IdD> {
    pub(crate) fn parent(&self, w: &IdD) -> Option<IdD> {
        todo!()
    }

    pub(crate) fn label(&self, w: &IdD) -> Label {
        todo!()
    }

    fn children(&self, w: &IdD) -> Vec<IdD> {
        todo!()
    }

    pub(crate) fn child_postion(&self, a: &IdD) -> usize {
        todo!()
    }
}

impl<IdD> ActionsVec<SimpleAction<IdD, IdD>> {
    pub(crate) fn push(&self, action: SimpleAction<IdD, IdD>) {
        todo!()
    }
}

impl<IdD> InOrderNodes<IdD> {
    fn removeAll(&self, w: &Vec<IdD>) {
        todo!()
    }

    pub(crate) fn push(&self, arg0: IdD) {
        todo!()
    }

    fn contains(&self, c: &IdD) -> bool {
        todo!()
    }
}

pub trait TreePath<'a, Idx> {
    type ItemIterator: Iterator<Item = Idx>;
    fn iter(&'a self) -> Self::ItemIterator;
    fn extend(&self, path:&[Idx]) -> Self;
}

struct SimpleTreePath<Idx> {
    vec: Vec<Idx>,
}

impl<'a, Idx: 'a + PrimInt> TreePath<'a, Idx> for SimpleTreePath<Idx> {
    type ItemIterator = IterSimple<'a, Idx>;
    fn iter(&'a self) -> Self::ItemIterator {
        IterSimple {
            internal: self.vec.iter(),
        }
    }

    fn extend(&self, path:&[Idx]) -> Self {
        let mut vec = vec![];
        vec.extend(self.vec);
        vec.extend_from_slice(path);
        Self {
            vec,
        }
    }
}

pub struct CompressedTreePath<Idx> {
    bits: Box<[u8]>,
    phantom: PhantomData<*const Idx>,
}

impl<Idx: PrimInt> CompressedTreePath<Idx> {
    fn iter(&self) -> impl Iterator<Item = Idx> + '_ {
        Iter {
            side: false,
            slice: &self.bits,
            phantom: PhantomData,
        }
    }
}

impl<'a, Idx: 'a + PrimInt> TreePath<'a, Idx> for CompressedTreePath<Idx> {
    type ItemIterator = Iter<'a, Idx>;
    fn iter(&'a self) -> Self::ItemIterator {
        Iter {
            side: false,
            slice: &self.bits,
            phantom: PhantomData,
        }
    }

    fn extend(&self, path:&[Idx]) -> Self {
        // todo maybe try dev something more efficient (should be easy if useful)
        let mut vec = vec![];
        vec.extend(self.iter());
        vec.extend_from_slice(path);
        Self::from(vec)
    }
}

impl<Idx: PrimInt> From<Vec<Idx>> for CompressedTreePath<Idx> {
    fn from(x: Vec<Idx>) -> Self {
        let mut v: Vec<u8> = vec![];
        let mut side = false;
        for x in x {
            let mut a: usize = cast(x).unwrap();
            loop {
                let mut br = false;
                let b = if a >= 128 {
                    a = a - 128;
                    2 ^ 4 - 1 as u8
                } else if a >= 32 {
                    a = a - 32;
                    2 ^ 4 - 2 as u8
                } else if a >= 2 ^ 4 - 2 {
                    a = a - 2 ^ 4 - 3;
                    2 ^ 4 - 3 as u8
                } else {
                    br = true;
                    a as u8
                };

                if side {
                    v.push(b << 4)
                } else {
                    v.push(b)
                }
                side = !side;
                if br {
                    break;
                }
            }
        }
        Self {
            bits: v.into_boxed_slice(),
            phantom: PhantomData,
        }
    }
}

/// dumb wrapper to avoid problems with iterators typing
struct IterSimple<'a, Idx: 'a> {
    internal: core::slice::Iter<'a, Idx>,
}

impl<'a, Idx: 'a + Copy> Iterator for IterSimple<'a, Idx> {
    type Item = Idx;

    fn next(&mut self) -> Option<Self::Item> {
        self.internal.next().and_then(|x| Some(*x))
    }
}

/// advanced iterator used to get back path as Idx from compressed path
struct Iter<'a, Idx> {
    side: bool,
    slice: &'a [u8],
    phantom: PhantomData<*const Idx>,
}

impl<'a, Idx: 'a + PrimInt> Iterator for Iter<'a, Idx> {
    type Item = Idx;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            return None;
        }
        let mut c = num_traits::zero();
        loop {
            let a = &self.slice[0];
            let a = if self.side {
                (a & 0b11110000) >> 4
            } else {
                a & 0b00001111
            };
            let mut br = false;
            let b = if a == 2 ^ 4 - 1 {
                128
            } else if a == 2 ^ 4 - 2 {
                32
            } else if a == 2 ^ 4 - 3 {
                2 ^ 4 - 2
            } else {
                br = true;
                a
            };
            c = c + cast(b).unwrap();
            self.slice = &self.slice[1..];
            self.side = !self.side;
            if br {
                break;
            }
        }
        Some(c)
    }
}
