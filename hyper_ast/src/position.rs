use core::fmt;
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    path::{Path, PathBuf},
};

use num::{one, traits::NumAssign, zero, ToPrimitive};

use crate::{
    store::{
        defaults::{LabelIdentifier, NodeIdentifier},
        nodes::HashedNodeRef,
    },
    types::{
        self, AnyType, Children, HyperAST, HyperType, IterableChildren, LabelStore, Labeled,
        NodeId, NodeStore, Tree, TypeStore, Typed, TypedNodeId, WithChildren, WithSerialization,
    },
};

pub trait PrimInt: num::PrimInt + NumAssign + Debug {}

impl<T> PrimInt for T where T: num::PrimInt + NumAssign + Debug {}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Position {
    file: PathBuf,
    offset: usize,
    len: usize,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            file: PathBuf::default(),
            offset: 0,
            len: 0,
        }
    }
}

impl Position {
    pub fn new(file: PathBuf, offset: usize, len: usize) -> Self {
        Self { file, offset, len }
    }
    pub fn inc_path(&mut self, s: &str) {
        self.file.push(s);
    }
    pub fn inc_offset(&mut self, x: usize) {
        self.offset += x;
    }
    pub fn set_len(&mut self, x: usize) {
        self.len = x;
    }
    pub fn range(&self) -> std::ops::Range<usize> {
        self.offset..(self.offset + self.len)
    }
    pub fn file(&self) -> &Path {
        &self.file
    }
}
impl Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Position")
            .field("file", &self.file)
            .field("offset", &self.offset)
            .field("len", &self.len)
            .finish()
    }
}
impl Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"offset\":{},\"len\":{},\"file\":{:?}}}",
            &self.offset, &self.len, &self.file
        )
    }
}

pub fn extract_file_postion<'store, HAST: HyperAST<'store>>(
    stores: &'store HAST,
    parents: &[HAST::IdN],
) -> Position {
    if parents.is_empty() {
        Position::default()
    } else {
        let p = &parents[parents.len() - 1];
        let b = stores.node_store().resolve(p);
        // println!("type {:?}", b.get_type());
        // if !b.has_label() {
        //     panic!("{:?} should have a label", b.get_type());
        // }
        let l = stores.label_store().resolve(b.get_label_unchecked());

        let mut r = extract_file_postion(stores, &parents[..parents.len() - 1]);
        r.inc_path(l);
        r
    }
}

pub fn extract_position<'store, HAST>(
    stores: &'store HAST,
    parents: &[HAST::IdN],
    offsets: &[usize],
) -> Position
where
    HAST: HyperAST<'store, IdN = NodeIdentifier, T = HashedNodeRef<'store>>,
    HAST::TS: TypeStore<HashedNodeRef<'store>, Ty = AnyType>,
{
    if parents.is_empty() {
        return Position::default();
    }
    let p = parents[parents.len() - 1];
    let o = offsets[offsets.len() - 1];

    let b = stores.node_store().resolve(&p);

    let c = {
        let v: Vec<_> = b.children().unwrap().before(o.to_u16().unwrap() - 1).into();
        v.iter()
            .map(|x| {
                let b = stores.node_store().resolve(x);

                // println!("{:?}", b.get_type());
                b.try_bytes_len().unwrap() as usize
            })
            .sum()
    };
    if stores.type_store().resolve_type(&b).is_file() {
        let mut r = extract_file_postion(stores, parents);
        r.inc_offset(c);
        r
    } else {
        let mut r = extract_position(
            stores,
            &parents[..parents.len() - 1],
            &offsets[..offsets.len() - 1],
        );
        r.inc_offset(c);
        r
    }
}

pub trait TreePath<IdN = NodeIdentifier, Idx = u16> {
    fn node(&self) -> Option<&IdN>;
    fn offset(&self) -> Option<&Idx>;
    fn check<'store, HAST>(&self, stores: &'store HAST) -> Result<(), ()>
    where
        HAST: HyperAST<'store, IdN = IdN::IdN>,
        HAST::T: WithChildren<ChildIdx = Idx>,
        HAST::IdN: Eq,
        IdN: NodeId,
        IdN::IdN: NodeId<IdN = IdN::IdN>;
}

pub trait TreePathMut<IdN, Idx>: TreePath<IdN, Idx> {
    fn pop(&mut self) -> Option<(IdN, Idx)>;
    fn goto(&mut self, node: IdN, i: Idx);
    fn inc(&mut self, node: IdN);
    fn dec(&mut self, node: IdN);
}

pub trait TypedTreePath<TIdN: TypedNodeId, Idx>: TreePath<TIdN::IdN, Idx> {
    fn node_typed(&self) -> Option<&TIdN>;
    fn pop_typed(&mut self) -> Option<(TIdN, Idx)>;
    fn goto_typed(&mut self, node: TIdN, i: Idx);
}

#[derive(Clone, Debug)]
pub struct StructuralPosition<IdN = NodeIdentifier, Idx = u16> {
    pub(crate) nodes: Vec<IdN>,
    pub(crate) offsets: Vec<Idx>,
}

impl<IdN: Copy, Idx: PrimInt> TreePath<IdN, Idx> for StructuralPosition<IdN, Idx> {
    fn node(&self) -> Option<&IdN> {
        self.nodes.last()
    }

    fn offset(&self) -> Option<&Idx> {
        self.offsets.last()
    }

    fn check<'store, HAST>(&self, stores: &'store HAST) -> Result<(), ()>
    where
        HAST: HyperAST<'store, IdN = IdN::IdN>,
        HAST::T: WithChildren<ChildIdx = Idx>,
        HAST::IdN: Eq,
        IdN: NodeId,
        IdN::IdN: NodeId<IdN = IdN::IdN>,
    {
        assert_eq!(self.offsets.len(), self.nodes.len());
        if self.nodes.is_empty() {
            return Ok(());
        }
        let mut i = self.nodes.len() - 1;

        while i > 0 {
            let e = self.nodes[i];
            let o = self.offsets[i] - one();
            let p = self.nodes[i - 1];
            let b = stores.node_store().resolve(&p.as_id());
            if !b.has_children()
                || Some(e.as_id()) != b.child(&num::cast(o).expect("too big")).as_ref()
            {
                return Err(());
            }
            i -= 1;
        }
        Ok(())
    }
}

impl<IdN: Copy, Idx: PrimInt + NumAssign> TreePathMut<IdN, Idx> for StructuralPosition<IdN, Idx> {
    fn pop(&mut self) -> Option<(IdN, Idx)> {
        Some((self.nodes.pop()?, self.offsets.pop()?))
    }

    fn goto(&mut self, node: IdN, i: Idx) {
        self.nodes.push(node);
        self.offsets.push(i + one());
    }

    fn inc(&mut self, node: IdN) {
        *self.nodes.last_mut().unwrap() = node;
        *self.offsets.last_mut().unwrap() += one();
    }

    fn dec(&mut self, node: IdN) {
        *self.nodes.last_mut().unwrap() = node;
        if let Some(offsets) = self.offsets.last_mut() {
            assert!(*offsets > one());
            *offsets -= one();
        }
    }
}

// impl<TIdN: TypedNodeId> StructuralPosition<TIdN> {
//     fn check_typed<'store, HAST>(&self, stores: &'store HAST) -> Result<(), ()>
//     where
//         TIdN::IdN: Eq,
//         HAST: HyperAST<'store, IdN = TIdN::IdN>,
//         HAST: crate::types::TypedHyperAST<'store, TIdN>,
//     {
//         use crate::types::TypedNodeStore;
//         assert_eq!(self.offsets.len(), self.nodes.len());
//         if self.nodes.is_empty() {
//             return Ok(());
//         }
//         let mut i = self.nodes.len() - 1;

//         while i > 0 {
//             let e = self.nodes[i];
//             let o = self.offsets[i] - 1;
//             let p = self.nodes[i - 1];
//             let b = stores.typed_node_store().resolve(&p);
//             if !b.has_children() || Some(e.as_id()) != b.child(&num::cast(o).expect("too big")).as_ref() {
//                 return Err(());
//             }
//             i -= 1;
//         }
//         Ok(())
//     }
// }

impl<IdN, Idx: num::Zero> StructuralPosition<IdN, Idx> {
    pub fn new(node: IdN) -> Self {
        Self {
            nodes: vec![node],
            offsets: vec![zero()],
        }
    }
}

impl StructuralPosition<NodeIdentifier, u16> {
    pub fn make_position<'store, HAST>(&self, stores: &'store HAST) -> Position
    where
        HAST: HyperAST<
            'store,
            T = HashedNodeRef<'store>,
            IdN = NodeIdentifier,
            Label = LabelIdentifier,
        >,
        HAST::TS: TypeStore<HashedNodeRef<'store>, Ty = AnyType>,
        // HAST::Types: 'static + TypeTrait + Debug,
    {
        self.check(stores).unwrap();
        // let parents = self.parents.iter().peekable();
        let mut from_file = false;
        // let mut len = 0;
        let x = *self.node().unwrap();
        let b = stores.node_store().resolve(&x);

        let t = stores.type_store().resolve_type(&b);
        // println!("t0:{:?}", t);
        let len = if let Some(y) = b.try_bytes_len() {
            if !t.is_file() {
                from_file = true;
            }
            y as usize
            // Some(x)
        } else {
            0
            // None
        };
        let mut offset = 0;
        let mut path = vec![];
        if self.nodes.is_empty() {
            let path = PathBuf::from_iter(path.iter().rev());
            return Position {
                file: path,
                offset,
                len,
            };
        }
        let mut i = self.nodes.len() - 1;
        if from_file {
            while i > 0 {
                let p = self.nodes[i - 1];
                let b = stores.node_store().resolve(&p);

                let t = stores.type_store().resolve_type(&b);
                // println!("t1:{:?}", t);
                let o = self.offsets[i];
                let c: usize = {
                    let v: Vec<_> = b.children().unwrap().before(o.to_u16().unwrap() - 1).into();
                    v.iter()
                        .map(|x| {
                            let b = stores.node_store().resolve(x);

                            // println!("{:?}", b.get_type());
                            b.try_bytes_len().unwrap() as usize
                        })
                        .sum()
                };
                offset += c;
                if t.is_file() {
                    from_file = false;
                    i -= 1;
                    break;
                } else {
                    i -= 1;
                }
            }
        }
        if self.nodes.is_empty() {
        } else if !from_file
        // || (i == 0 && stores.node_store().resolve(self.nodes[i]).get_type() == Type::Program)
        {
            loop {
                let n = self.nodes[i];
                let b = stores.node_store().resolve(&n);
                // println!("t2:{:?}", b.get_type());
                let l = stores.label_store().resolve(b.get_label_unchecked());
                path.push(l);
                if i == 0 {
                    break;
                } else {
                    i -= 1;
                }
            }
        } else {
            let p = self.nodes[i - 1];
            let b = stores.node_store().resolve(&p);
            let o = self.offsets[i];
            let c: usize = {
                let v: Vec<_> = b.children().unwrap().before(o.to_u16().unwrap() - 1).into();
                v.iter()
                    .map(|x| {
                        let b = stores.node_store().resolve(x);

                        // println!("{:?}", b.get_type());
                        b.try_bytes_len().unwrap() as usize
                    })
                    .sum()
            };
            offset += c;
        }

        let path = PathBuf::from_iter(path.iter().rev());
        Position {
            file: path,
            offset,
            len,
        }
    }
}

impl<IdN, Idx> From<(Vec<IdN>, Vec<Idx>, IdN)> for StructuralPosition<IdN, Idx> {
    fn from(mut x: (Vec<IdN>, Vec<Idx>, IdN)) -> Self {
        assert_eq!(x.0.len() + 1, x.1.len());
        x.0.push(x.2);
        Self {
            nodes: x.0,
            offsets: x.1,
        }
    }
}
impl<IdN, Idx> From<(Vec<IdN>, Vec<Idx>)> for StructuralPosition<IdN, Idx> {
    fn from(x: (Vec<IdN>, Vec<Idx>)) -> Self {
        assert_eq!(x.0.len(), x.1.len());
        Self {
            nodes: x.0,
            offsets: x.1,
        }
    }
}
impl<IdN, Idx: num::Zero> From<IdN> for StructuralPosition<IdN, Idx> {
    fn from(node: IdN) -> Self {
        Self::new(node)
    }
}

// #[derive(Clone, Debug)]
// pub struct StructuralPositionWithIndentation {
//     pub(crate) nodes: Vec<NodeIdentifier>,
//     pub(crate) offsets: Vec<usize>,
//     pub(crate) indentations: Vec<Box<[Space]>>,
// }

pub struct StructuralPositionStore<IdN = NodeIdentifier, Idx = u16> {
    pub nodes: Vec<IdN>,
    parents: Vec<usize>,
    offsets: Vec<Idx>,
    // ends: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
pub struct SpHandle(usize);

// struct IterStructuralPositions<'a> {
//     sps: &'a StructuralPositionStore,
//     ends: core::slice::Iter<'a, usize>,
// }

// impl<'a> Iterator for IterStructuralPositions<'a> {
//     type Item = StructuralPosition;

//     fn next(&mut self) -> Option<Self::Item> {
//         let x = *self.ends.next()?;
//         let it = ExploreStructuralPositions::new(self.sps, x);
//         // let r = Position;
//         todo!()
//     }
// }

#[derive(Clone, Debug)]
pub struct Scout<IdN, Idx> {
    path: StructuralPosition<IdN, Idx>,
    ancestors: usize,
}

#[derive(Clone, Debug)]
pub struct TypedScout<TIdN: TypedNodeId, Idx> {
    path: StructuralPosition<TIdN::IdN, Idx>,
    ancestors: usize,
    tdepth: i16,
    phantom: PhantomData<TIdN>,
}

impl<IdN: Eq + Copy, Idx: PrimInt + NumAssign> TreePathMut<IdN, Idx> for Scout<IdN, Idx> {
    fn pop(&mut self) -> Option<(IdN, Idx)> {
        self.path.pop()
    }

    fn goto(&mut self, node: IdN, i: Idx) {
        self.path.goto(node, i)
    }

    fn inc(&mut self, node: IdN) {
        self.path.inc(node)
    }

    fn dec(&mut self, node: IdN) {
        self.path.dec(node)
    }
}

impl<IdN: Eq + Copy, Idx: PrimInt> TreePath<IdN, Idx> for Scout<IdN, Idx> {
    fn node(&self) -> Option<&IdN> {
        self.path.node()
    }

    fn offset(&self) -> Option<&Idx> {
        self.path.offset()
    }
    fn check<'store, HAST>(&self, stores: &'store HAST) -> Result<(), ()>
    where
        HAST: HyperAST<'store, IdN = IdN::IdN>,
        HAST::T: WithChildren<ChildIdx = Idx>,
        HAST::IdN: Eq,
        IdN: NodeId,
        IdN::IdN: NodeId<IdN = IdN::IdN>,
    {
        self.path.check(stores)
    }
}

impl<TIdN: TypedNodeId, Idx: PrimInt> TreePath<TIdN::IdN, Idx> for TypedScout<TIdN, Idx>
where
    TIdN::IdN: Copy,
{
    fn node(&self) -> Option<&TIdN::IdN> {
        self.path.node()
    }

    fn offset(&self) -> Option<&Idx> {
        self.path.offset()
    }

    fn check<'store, HAST>(&self, stores: &'store HAST) -> Result<(), ()>
    where
        HAST: HyperAST<'store, IdN = <TIdN::IdN as NodeId>::IdN>,
        HAST::T: WithChildren<ChildIdx = Idx>,
        HAST::IdN: Eq,
        TIdN::IdN: NodeId,
        <TIdN::IdN as NodeId>::IdN: NodeId<IdN = <TIdN::IdN as NodeId>::IdN>,
    {
        self.path.check(stores)
    }
}

impl<TIdN: TypedNodeId, Idx: PrimInt> TypedTreePath<TIdN, Idx> for TypedScout<TIdN, Idx>
where
    TIdN::IdN: Copy,
{
    fn node_typed(&self) -> Option<&TIdN> {
        let n = self.path.node()?;
        if self.tdepth > 0 {
            return None;
        }
        // VALIDITY: self.tdepth <= 0
        let tdepth = -self.tdepth as usize;
        // VALIDITY: condition for Some variant out of self.path.node()
        let i = self.path.nodes.len() - 1;
        if i == tdepth {
            // VALIDITY: checked tdepth
            Some(unsafe { TIdN::from_ref_id(n) })
        } else {
            None
        }
    }

    fn pop_typed(&mut self) -> Option<(TIdN, Idx)> {
        todo!()
    }

    fn goto_typed(&mut self, node: TIdN, i: Idx) {
        self.path.goto(*node.as_id(), i);
        self.tdepth = self.tdepth + 1;
    }
}

impl<TIdN: TypedNodeId, Idx: PrimInt> TypedScout<TIdN, Idx>
where
    TIdN::IdN: Eq + Copy,
{
    fn _node(&self) -> Option<&TIdN> {
        let n = self.path.node()?;
        if self.tdepth > 0 {
            return None;
        }
        let tdepth = -self.tdepth as usize;
        // VALIDITY: condition for Some variant out of self.path.node()
        let len = self.path.nodes.len() - 1;
        if len < tdepth {
            // VALIDITY: checked tdepth
            Some(unsafe { TIdN::from_ref_id(n) })
        } else {
            None
        }
    }
    // pub fn node(&self) -> Option<Result<TIdN, TIdN::IdN>> {
    //     // [a] , 0 -> 1 > 0
    //     // [a,b] , -1 -> 0 > 0
    //     self.path.node().map(|x| if -self.path.len() < self.tdepth as isize {} else {})
    // }
    // pub fn node(&self) -> Option<Result<TIdN, TIdN::IdN>> {
    //     // [a] , 0 -> 1 > 0
    //     // [a,b] , -1 -> 0 > 0
    //     self.path.node().map(|x| if -self.path.len() < self.tdepth as isize {} else {})
    // }
    // pub fn node_always(&self, x: &StructuralPositionStore<TIdN::IdN>) -> TIdN::IdN {
    //     if let Some(y) = self.path.node() {
    //         *y.as_id()
    //     } else {
    //         assert!(self.ancestors.0 > 0);
    //         x.nodes[self.ancestors.1]
    //     }
    // }
    pub fn offset_always(&self, x: &StructuralPositionStore<TIdN::IdN, Idx>) -> Idx {
        if let Some(y) = self.path.offset() {
            *y
        } else {
            x.offsets[self.ancestors]
        }
    }
    // pub fn has_parents(&self) -> bool {
    //     if self.path.nodes.is_empty() {
    //         self.ancestors.1 != 0
    //     } else {
    //         true
    //     }
    // }

    pub fn make_position<'store, HAST>(
        &self,
        sp: &StructuralPositionStore<HAST::IdN, Idx>,
        stores: &'store HAST,
    ) -> Position
    where
        HAST: HyperAST<'store, IdN = TIdN::IdN, Label = LabelIdentifier>,
        HAST: crate::types::TypedHyperAST<'store, TIdN>,
        <HAST as crate::types::TypedHyperAST<'store, TIdN>>::T:
            Typed<Type = TIdN::Ty> + WithSerialization + WithChildren,
        <HAST as crate::types::HyperAST<'store>>::T: WithSerialization + WithChildren,
        <<HAST as crate::types::TypedHyperAST<'store, TIdN>>::T as types::WithChildren>::ChildIdx:
            Debug,
        HAST::IdN: Copy + Debug,
        TIdN: Debug,
        TIdN::IdN: NodeId<IdN = TIdN::IdN>,
    {
        todo!()
        // // NOTE: this algorithm works in post order
        // self.check(stores).unwrap();
        // // let parents = self.parents.iter().peekable();
        // let mut from_file = false;
        // // let mut len = 0;
        // if let Some(x) = self.path.node() {
        //     use crate::types::TypedNodeStore;
        //     let b = stores.typed_node_store().resolve(x);
        //     let t = b.get_type();
        //     let len = if let Some(y) = b.try_bytes_len() {
        //         if !t.is_file() {
        //             from_file = true;
        //         }
        //         y as usize
        //         // Some(x)
        //     } else {
        //         0
        //         // None
        //     };
        //     let mut offset = 0;
        //     let mut path = vec![];
        //     if self.path.nodes.is_empty() {
        //         todo!();
        //         // return ExploreStructuralPositions::new(sp, self.root)
        //         //     .make_position_aux(stores, from_file, len, offset, path);
        //     }
        //     let mut i = self.path.nodes.len() - 1;
        //     if from_file {
        //         while i > 0 {
        //             let p = self.path.nodes[i - 1];
        //             let b = stores.typed_node_store().resolve(&p);
        //             let t = b.get_type();
        //             // println!("t1:{:?}", t);
        //             let o = self.path.offsets[i];
        //             let o: <<HAST as TypedHyperAST<'store, TIdN>>::T as WithChildren>::ChildIdx =
        //                 num::cast(o).unwrap();
        //             let c: usize = {
        //                 let v: Vec<_> = b
        //                     .children()
        //                     .unwrap()
        //                     .before(o - one())
        //                     .iter_children()
        //                     .collect();
        //                 v.iter()
        //                     .map(|x| {
        //                         let b = stores.node_store().resolve(x);
        //                         // println!("{:?}", b.get_type());
        //                         b.try_bytes_len().unwrap() as usize
        //                     })
        //                     .sum()
        //             };
        //             offset += c;
        //             if t.is_file() {
        //                 from_file = false;
        //                 i -= 1;
        //                 break;
        //             } else {
        //                 i -= 1;
        //             }
        //         }
        //     }
        //     if self.path.nodes.is_empty() {
        //     } else if !from_file
        //     // || (i == 0 && stores.node_store().resolve(self.path.nodes[i]).get_type() == Type::Program)
        //     {
        //         loop {
        //             from_file = false;
        //             let n = &self.path.nodes[i];
        //             let b = stores.typed_node_store().resolve(n);
        //             // println!("t2:{:?}", b.get_type());
        //             let l = stores.label_store().resolve(b.get_label_unchecked());
        //             path.push(l);
        //             if i == 0 {
        //                 todo!("call aux make pos for Scout");
        //                 break;
        //             } else {
        //                 i -= 1;
        //             }
        //         }
        //     } else {
        //         let p = if i == 0 {
        //             todo!("call aux make pos for Scout");
        //             // sp.nodes[self.root]
        //         } else {
        //             self.path.nodes[i - 1]
        //         };
        //         let b = stores.typed_node_store().resolve(&p);
        //         let t = b.get_type();
        //         // println!("t3:{:?}", t);
        //         let o = self.path.offsets[i];
        //         let o: <<HAST as TypedHyperAST<'store, TIdN>>::T as WithChildren>::ChildIdx =
        //             num::cast(o).unwrap();
        //         let c: usize = {
        //             let v: Vec<_> = b
        //                 .children()
        //                 .unwrap()
        //                 .before(o - one())
        //                 .iter_children()
        //                 .collect();
        //             v.iter()
        //                 .map(|x| {
        //                     let b = stores.node_store().resolve(x);
        //                     // println!("{:?}", b.get_type());
        //                     b.try_bytes_len().unwrap() as usize
        //                 })
        //                 .sum()
        //         };
        //         offset += c;
        //         if t.is_file() {
        //             from_file = false;
        //         } else {
        //         }
        //     }
        //     todo!()
        //     // ExploreStructuralPositions::new(sp, self.root)
        //     //     .make_position_aux(stores, from_file, len, offset, path)
        // } else {
        //     todo!()
        // }
    }
    // fn check<'store, HAST>(&self, stores: &'store HAST) -> Result<(), ()>
    // where
    //     HAST: HyperAST<'store, IdN = TIdN::IdN, Label = LabelIdentifier>,
    //     HAST: crate::types::TypedHyperAST<'store, TIdN>,
    //     TIdN::IdN: NodeId<IdN = TIdN::IdN>,
    // {
    //     self.path.check(stores)
    // }
}

impl<IdN: Eq + Copy, Idx: PrimInt> Scout<IdN, Idx> {
    pub fn node_always(&self, x: &StructuralPositionStore<IdN, Idx>) -> IdN {
        if let Some(y) = self.path.node() {
            *y
        } else {
            x.nodes[self.ancestors]
        }
    }
    pub fn offset_always(&self, x: &StructuralPositionStore<IdN, Idx>) -> Idx {
        if let Some(y) = self.path.offset() {
            *y
        } else {
            x.offsets[self.ancestors]
        }
    }
    pub fn has_parents(&self) -> bool {
        if self.path.nodes.is_empty() {
            self.ancestors != zero()
        } else {
            true
        }
    }

    pub fn make_position<'store, HAST>(
        &self,
        sp: &StructuralPositionStore<HAST::IdN, Idx>,
        stores: &'store HAST,
    ) -> Position
    where
        HAST: HyperAST<'store, IdN = IdN, Label = LabelIdentifier>,
        HAST::T: Typed<Type = AnyType> + WithSerialization + WithChildren<ChildIdx = Idx>,
        // HAST::Types: Eq + TypeTrait,
        <<HAST as HyperAST<'store>>::T as types::WithChildren>::ChildIdx: Debug,
        IdN: Copy + Debug + NodeId<IdN = IdN>,
    {
        self.check(stores).unwrap();
        // let parents = self.parents.iter().peekable();
        let mut from_file = false;
        // let mut len = 0;
        let x = self.node_always(sp);
        let b = stores.node_store().resolve(&x);
        let t = stores.type_store().resolve_type(&b);
        // println!("t0:{:?}", t);
        let len = if let Some(y) = b.try_bytes_len() {
            if !t.is_file() {
                from_file = true;
            }
            y as usize
            // Some(x)
        } else {
            0
            // None
        };
        let mut offset = 0;
        let mut path = vec![];
        if self.path.nodes.is_empty() {
            return ExploreStructuralPositions::new(sp, self.ancestors)
                .make_position_aux(stores, from_file, len, offset, path);
        }
        let mut i = self.path.nodes.len() - 1;
        if from_file {
            while i > 0 {
                let p = self.path.nodes[i - 1];
                let b = stores.node_store().resolve(&p);
                let t = stores.type_store().resolve_type(&b);
                // println!("t1:{:?}", t);
                let o = self.path.offsets[i];
                let o: <HAST::T as WithChildren>::ChildIdx = num::cast(o).unwrap();
                let c: usize = {
                    let v: Vec<_> = b
                        .children()
                        .unwrap()
                        .before(o - one())
                        .iter_children()
                        .collect();
                    v.iter()
                        .map(|x| {
                            let b = stores.node_store().resolve(x);
                            // println!("{:?}", b.get_type());
                            b.try_bytes_len().unwrap() as usize
                        })
                        .sum()
                };
                offset += c;
                if t.is_file() {
                    from_file = false;
                    i -= 1;
                    break;
                } else {
                    i -= 1;
                }
            }
        }
        if self.path.nodes.is_empty() {
        } else if !from_file
        // || (i == 0 && stores.node_store().resolve(self.path.nodes[i]).get_type() == Type::Program)
        {
            loop {
                from_file = false;
                let n = self.path.nodes[i];
                let b = stores.node_store().resolve(&n);
                // println!("t2:{:?}", b.get_type());
                let l = stores.label_store().resolve(b.get_label_unchecked());
                path.push(l);
                if i == 0 {
                    break;
                } else {
                    i -= 1;
                }
            }
        } else {
            let p = if i == 0 {
                sp.nodes[self.ancestors]
            } else {
                self.path.nodes[i - 1]
            };
            let b = stores.node_store().resolve(&p);
            let t = stores.type_store().resolve_type(&b);
            // println!("t3:{:?}", t);
            let o = self.path.offsets[i];
            let o: <HAST::T as WithChildren>::ChildIdx = num::cast(o).unwrap();
            let c: usize = {
                let v: Vec<_> = b
                    .children()
                    .unwrap()
                    .before(o - one())
                    .iter_children()
                    .collect();
                v.iter()
                    .map(|x| {
                        let b = stores.node_store().resolve(x);
                        // println!("{:?}", b.get_type());
                        b.try_bytes_len().unwrap() as usize
                    })
                    .sum()
            };
            offset += c;
            if t.is_file() {
                from_file = false;
            } else {
            }
        }
        ExploreStructuralPositions::new(sp, self.ancestors)
            .make_position_aux(stores, from_file, len, offset, path)
    }
}

impl<TIdN: TypedNodeId, Idx> From<TypedScout<TIdN, Idx>> for Scout<TIdN::IdN, Idx> {
    fn from(value: TypedScout<TIdN, Idx>) -> Self {
        Self {
            ancestors: value.ancestors,
            path: value.path,
        }
    }
}

impl<TIdN: TypedNodeId + Eq + Copy, Idx: PrimInt> TypedScout<TIdN, Idx>
where
    TIdN::IdN: Eq + Copy,
{
    pub fn up(
        &mut self,
        x: &StructuralPositionStore<TIdN::IdN, Idx>,
    ) -> Option<Result<TIdN, TIdN::IdN>> {
        self.path.pop()?;
        Some(self.node_always(x))
    }
    pub fn up0(
        mut self,
        x: &StructuralPositionStore<TIdN::IdN, Idx>,
    ) -> Result<Self, Scout<TIdN::IdN, Idx>> {
        if let Some(_) = self.path.pop() {
            self.path = StructuralPosition {
                nodes: vec![],
                offsets: vec![],
            };
            self.ancestors = x.parents[self.ancestors];
            let tdepth = -self.tdepth as usize;
            let i = self.path.nodes.len();
            if i == tdepth {
                self.tdepth += 1;
                Ok(self)
            } else {
                Err(self.into())
            }
        } else if self.ancestors == 0 {
            let path = self.path;
            let ancestors = self.ancestors;
            Err(Scout { path, ancestors })
        } else if self.tdepth > 1 {
            self.tdepth -= 1;
            self.path = StructuralPosition {
                nodes: vec![],
                offsets: vec![],
            };
            self.ancestors = x.parents[self.ancestors];
            Ok(self)
        } else {
            let path = StructuralPosition {
                nodes: vec![],
                offsets: vec![],
            };
            let ancestors = x.parents[self.ancestors];
            Err(Scout { path, ancestors })
        }
    }
    pub fn _up(&mut self) {
        self.tdepth -= 1;
        self.path.pop();
        assert_eq!(self.path.nodes.len(), self.path.offsets.len());
    }
    pub fn node_always(
        &self,
        x: &StructuralPositionStore<TIdN::IdN, Idx>,
    ) -> Result<TIdN, TIdN::IdN> {
        if let Some(n) = self.node_typed() {
            Ok(n.clone())
        } else if let Some(y) = self.path.node() {
            Ok(unsafe { TIdN::from_id(*y) })
            // TODO do proper state
            // Err(*y)
        } else {
            Ok(unsafe { TIdN::from_id(x.nodes[self.ancestors]) })
            // TODO do proper state
            // if self.tdepth > 0 {
            //     Ok(unsafe { TIdN::from_id(x.nodes[self.ancestors]) })
            // } else {
            //     Err(x.nodes[self.ancestors])
            // }
        }
    }
    pub fn up2(
        &mut self,
        x: &StructuralPositionStore<TIdN::IdN, Idx>,
    ) -> Option<Result<TIdN, TIdN::IdN>> {
        if self.path.nodes.is_empty() {
            self.path = StructuralPosition {
                nodes: vec![],
                offsets: vec![],
            };
            assert_eq!(self.path.nodes.len(), self.path.offsets.len());
            if self.ancestors == 0 {
                None
            } else {
                self.ancestors = x.parents[self.ancestors];
                Some(self.node_always(x))
            }
        } else {
            self._up();
            Some(self.node_always(x))
        }
    }
}

impl<IdN: Eq + Copy, Idx: PrimInt> Scout<IdN, Idx> {
    pub fn _up(&mut self) {
        self.path.pop();
        assert_eq!(self.path.nodes.len(), self.path.offsets.len());
    }
    pub fn make_child(&self, node: IdN, i: Idx) -> Self {
        let mut s = self.clone();
        s.path.goto(node, i);
        s
    }
    pub fn up(&mut self, x: &StructuralPositionStore<IdN, Idx>) -> Option<IdN> {
        // println!("up {} {:?}", self.root, self.path);
        // if !self.path.offsets.is_empty() && self.path.offsets[0] == 0 {
        //     assert!(self.root == 0);
        // }
        if self.path.nodes.is_empty() {
            // let o = x.offsets[self.root];
            self.path = StructuralPosition {
                nodes: vec![],
                offsets: vec![],
            };
            // self.path = StructuralPosition::with_offset(x.nodes[self.root], o);
            assert_eq!(self.path.nodes.len(), self.path.offsets.len());
            if self.ancestors == 0 {
                None
            } else {
                self.ancestors = x.parents[self.ancestors];
                Some(self.node_always(x))
            }
            // if o == 0 {
            //     assert!(self.path.offsets[0] == 0);
            //     assert!(self.root == 0);
            // }
        } else {
            self._up();
            Some(self.node_always(x))
        }
        // if !self.path.offsets.is_empty() && self.path.offsets[0] == 0 {
        //     assert!(self.root == 0);
        // }
    }
}

// impl From<StructuralPosition> for Scout {
//     fn from(x: StructuralPosition) -> Self {
//         Self { root: 0, path: x }
//     }
// }

impl<IdN: Clone, Idx: PrimInt> From<(StructuralPosition<IdN, Idx>, usize)> for Scout<IdN, Idx> {
    fn from((path, ancestors): (StructuralPosition<IdN, Idx>, usize)) -> Self {
        let path = if !path.offsets.is_empty() && path.offsets[0].is_zero() {
            assert_eq!(ancestors, 0);
            StructuralPosition {
                nodes: path.nodes[1..].to_owned(),
                offsets: path.offsets[1..].to_owned(),
            }
        } else {
            path
        };
        Self { path, ancestors }
    }
}

pub struct ExploreStructuralPositions<'a, IdN, Idx = usize> {
    sps: &'a StructuralPositionStore<IdN, Idx>,
    i: usize,
}
/// precondition: root node do not contain a File node
/// TODO make whole thing more specific to a path in a tree
pub fn compute_range<'store, It, HAST>(
    root: HAST::IdN,
    offsets: &mut It,
    stores: &'store HAST,
) -> (usize, usize, HAST::IdN)
where
    It::Item: ToPrimitive,
    It: Iterator,
    HAST:
        HyperAST<'store, T = HashedNodeRef<'store>, IdN = NodeIdentifier, Label = LabelIdentifier>,
{
    let mut offset = 0;
    let mut x = root;
    for o in offsets {
        // dbg!(offset);
        let b = stores.node_store().resolve(&x);
        // dbg!(b.get_type());
        // dbg!(o.to_usize().unwrap());
        if let Some(cs) = b.children() {
            let cs = cs.clone();
            for y in 0..o.to_usize().unwrap() {
                let b = stores.node_store().resolve(&cs[y]);

                offset += b.try_bytes_len().unwrap_or(0).to_usize().unwrap();
            }
            // if o.to_usize().unwrap() >= cs.len() {
            //     // dbg!("fail");
            // }
            if let Some(a) = cs.get(o.to_u16().unwrap()) {
                x = *a;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    let b = stores.node_store().resolve(&x);

    (
        offset,
        offset + b.try_bytes_len().unwrap_or(0).to_usize().unwrap(),
        x,
    )
}
/// must be in a file
pub fn resolve_range<'store, HAST>(
    root: HAST::IdN,
    start: usize,
    end: Option<usize>,
    stores: &'store HAST,
) -> (HAST::IdN, Vec<usize>)
where
    HAST:
        HyperAST<'store, T = HashedNodeRef<'store>, IdN = NodeIdentifier, Label = LabelIdentifier>,
{
    enum RangeStatus {
        Inside(usize),
        Outside(usize),
        Right(usize, usize),
        Left(usize, usize),
    }

    fn range_status(start: usize, offset: usize, len: usize, end: usize) -> RangeStatus {
        if start < offset {
            if offset + len < end {
                RangeStatus::Inside((offset - start) + (offset + len - end))
            } else if offset + len == end {
                RangeStatus::Inside((offset - start) + (offset + len - end))
            } else {
                RangeStatus::Right(offset - start, offset + len - end)
            }
        } else if start == offset {
            if offset + len < end {
                RangeStatus::Inside((offset - start) + (offset + len - end))
            } else if offset + len == end {
                RangeStatus::Inside(0)
            } else {
                RangeStatus::Right(offset - start, offset + len - end)
            }
        } else {
            if offset + len < end {
                RangeStatus::Left(start - offset, end - (offset + len))
            } else if offset + len == end {
                RangeStatus::Left(start - offset, end - (offset + len))
            } else {
                RangeStatus::Inside((start - offset) + (end - (offset + len)))
            }
        }
    }
    let mut offset = 0;
    // let mut parent_status = RangeStatus::Outside(0);
    // let mut prev_status = RangeStatus::Outside(0);
    // let mut prev = root;
    let mut x = root;
    let mut offsets = vec![];
    'main: loop {
        let b = stores.node_store().resolve(&x);
        // dbg!(offset);
        // dbg!(b.get_type());
        // dbg!(o.to_usize().unwrap());
        if let Some(cs) = b.children() {
            let cs = cs.clone();
            for (y, child_id) in cs.iter_children().enumerate() {
                let b = stores.node_store().resolve(child_id);

                let len = b.try_bytes_len().unwrap_or(0).to_usize().unwrap();
                // let rs = range_status(start, offset, len, end);
                // dbg!(b.get_type(), start, offset, len, end);
                if offset + len < start {
                    // not yet reached something
                } else if end.map_or(true, |end| offset + len <= end) {
                    break 'main;
                } else {
                    offsets.push(y);
                    x = *child_id;
                    break;
                }
                offset += len;
            }
        } else {
            break;
        }
    }
    (x, offsets)
}

pub fn compute_position<'store, HAST, It>(
    root: HAST::IdN,
    offsets: &mut It,
    stores: &'store HAST,
) -> (Position, HAST::IdN)
where
    It::Item: Clone,
    HAST::IdN: Clone,
    HAST: HyperAST<'store>,
    HAST::T: WithSerialization + WithChildren,
    It: Iterator<Item = HAST::Idx>,
{
    let mut offset = 0;
    let mut x = root;
    let mut path = vec![];
    for o in &mut *offsets {
        // dbg!(offset);
        let b = stores.node_store().resolve(&x);
        // dbg!(b.get_type());
        // dbg!(o.to_usize().unwrap());

        let t = stores.type_store().resolve_type(&b);

        if t.is_directory() || t.is_file() {
            let l = stores.label_store().resolve(b.get_label_unchecked());
            path.push(l);
        }

        if let Some(cs) = b.children() {
            let cs = cs.clone();
            if !t.is_directory() {
                for y in cs.before(o.clone()).iter_children() {
                    let b = stores.node_store().resolve(y);
                    offset += b.try_bytes_len().unwrap().to_usize().unwrap();
                }
            } else {
                // for y in 0..o.to_usize().unwrap() {
                //     let b = stores.node_store().resolve(cs[y]);
                //     println!("{:?}",b.get_type());
                // }
            }
            // if o.to_usize().unwrap() >= cs.len() {
            //     // dbg!("fail");
            // }
            if let Some(a) = cs.get(o) {
                x = a.clone();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    assert!(offsets.next().is_none());
    let b = stores.node_store().resolve(&x);
    let t = stores.type_store().resolve_type(&b);
    if t.is_directory() || t.is_file() {
        let l = stores.label_store().resolve(b.get_label_unchecked());
        path.push(l);
    }

    let len = if !t.is_directory() {
        b.try_bytes_len().unwrap().to_usize().unwrap()
    } else {
        0
    };
    let path = PathBuf::from_iter(path.iter());
    (
        Position {
            file: path,
            offset,
            len,
        },
        x,
    )
}
pub fn compute_position_and_nodes<'store, HAST, It: Iterator>(
    root: HAST::IdN,
    offsets: &mut It,
    stores: &'store HAST,
) -> (Position, Vec<HAST::IdN>)
where
    It::Item: Clone,
    HAST::IdN: Clone,
    HAST: HyperAST<'store>,
    HAST::T: WithSerialization + WithChildren<ChildIdx = It::Item>,
{
    let mut offset = 0;
    let mut x = root;
    let mut path_ids = vec![];
    let mut path = vec![];
    for o in &mut *offsets {
        // dbg!(offset);
        let b = stores.node_store().resolve(&x);
        // dbg!(b.get_type());
        // dbg!(o.to_usize().unwrap());

        let t = stores.type_store().resolve_type(&b);

        if t.is_directory() || t.is_file() {
            let l = stores.label_store().resolve(b.get_label_unchecked());
            path.push(l);
        }

        if let Some(cs) = b.children() {
            let cs = cs.clone();
            if !t.is_directory() {
                for y in cs.before(o.clone()).iter_children() {
                    let b = stores.node_store().resolve(y);
                    offset += b.try_bytes_len().unwrap().to_usize().unwrap();
                }
            } else {
                // for y in 0..o.to_usize().unwrap() {
                //     let b = stores.node_store().resolve(cs[y]);
                //     println!("{:?}",b.get_type());
                // }
            }
            // if o.to_usize().unwrap() >= cs.len() {
            //     // dbg!("fail");
            // }
            if let Some(a) = cs.get(o) {
                x = a.clone();
                path_ids.push(x.clone());
            } else {
                break;
            }
        } else {
            break;
        }
    }
    assert!(offsets.next().is_none());
    let b = stores.node_store().resolve(&x);
    let t = stores.type_store().resolve_type(&b);
    if t.is_directory() || t.is_file() {
        let l = stores.label_store().resolve(b.get_label_unchecked());
        path.push(l);
    }

    let len = if !t.is_directory() {
        b.try_bytes_len().unwrap().to_usize().unwrap()
    } else {
        0
    };
    let path = PathBuf::from_iter(path.iter());
    path_ids.reverse();
    (
        Position {
            file: path,
            offset,
            len,
        },
        path_ids,
    )
}
pub fn compute_position_with_no_spaces<'store, HAST, It: Iterator>(
    root: HAST::IdN,
    offsets: &mut It,
    stores: &'store HAST,
) -> (Position, HAST::IdN, Vec<It::Item>)
where
    It::Item: Clone + PrimInt,
    HAST::IdN: Clone,
    HAST: HyperAST<'store>,
    HAST::T: WithSerialization + WithChildren<ChildIdx = It::Item>,
{
    let (pos, mut path_ids, no_spaces) =
        compute_position_and_nodes_with_no_spaces(root, offsets, stores);
    (pos, path_ids.remove(path_ids.len() - 1), no_spaces)
}

pub fn path_with_spaces<'store, HAST, It: Iterator>(
    root: HAST::IdN,
    no_spaces: &mut It,
    stores: &'store HAST,
) -> (Vec<It::Item>,)
where
    It::Item: Clone + PrimInt,
    HAST::IdN: Clone,
    HAST: HyperAST<'store>,
    HAST::T: WithSerialization + WithChildren<ChildIdx = It::Item>,
{
    let mut offset = 0;
    let mut x = root;
    let mut path_ids = vec![];
    let mut with_spaces = vec![];
    let mut path = vec![];
    for mut o in &mut *no_spaces {
        // dbg!(offset);
        let b = stores.node_store().resolve(&x);
        // dbg!(b.get_type());
        // dbg!(o.to_usize().unwrap());

        let t = stores.type_store().resolve_type(&b);

        if t.is_directory() || t.is_file() {
            let l = stores.label_store().resolve(b.get_label_unchecked());
            path.push(l);
        }
        let mut with_s_idx = zero();
        if let Some(cs) = b.children() {
            let cs = cs.clone();
            if !t.is_directory() {
                for y in cs.iter_children() {
                    let b = stores.node_store().resolve(y);
                    if !stores.type_store().resolve_type(&b).is_spaces() {
                        if o == zero() {
                            break;
                        }
                        o = o - one();
                    }
                    with_s_idx = with_s_idx + one();
                    offset += b.try_bytes_len().unwrap().to_usize().unwrap();
                }
            } else {
                with_s_idx = o;
                // for y in 0..o.to_usize().unwrap() {
                //     let b = stores.node_store().resolve(cs[y]);
                //     println!("{:?}",b.get_type());
                // }
            }
            // if o.to_usize().unwrap() >= cs.len() {
            //     // dbg!("fail");
            // }
            if let Some(a) = cs.get(with_s_idx) {
                x = a.clone();
                with_spaces.push(with_s_idx);
                path_ids.push(x.clone());
            } else {
                break;
            }
        } else {
            break;
        }
    }
    assert!(no_spaces.next().is_none());
    let b = stores.node_store().resolve(&x);
    let t = stores.type_store().resolve_type(&b);
    if t.is_directory() || t.is_file() {
        let l = stores.label_store().resolve(b.get_label_unchecked());
        path.push(l);
    }

    let len = if !t.is_directory() {
        b.try_bytes_len().unwrap().to_usize().unwrap()
    } else {
        0
    };
    let path = PathBuf::from_iter(path.iter());
    path_ids.reverse();
    (with_spaces,)
}

pub fn global_pos_with_spaces<'store, T, NS, It: Iterator>(
    root: T::TreeId,
    // increasing order
    no_spaces: &mut It,
    node_store: &'store NS,
) -> (Vec<It::Item>,)
where
    It::Item: Clone + PrimInt,
    T::TreeId: Clone,
    NS: 'store + types::NodeStore<T::TreeId, R<'store> = T>,
    T: types::Tree<ChildIdx = It::Item> + types::WithStats,
{
    todo!()
    // let mut offset_with_spaces = zero();
    // let mut offset_without_spaces = zero();
    // // let mut x = root;
    // let mut res = vec![];
    // let (cs, size_no_s) = {
    //     let b = stores.node_store().resolve(&root);
    //     (b.children().unwrap().iter_children().collect::<Vec<_>>(),b.get_size())
    // };
    // let mut stack = vec![(root, size_no_s, 0, cs)];
    // while let Some(curr_no_space) = no_spaces.next() {
    //     loop {

    //         if curr_no_space == offset_without_spaces {
    //             res.push(offset_with_spaces);
    //             break;
    //         }
    //     }
    // }

    // (
    //     res,
    // )
}

pub fn compute_position_and_nodes_with_no_spaces<'store, HAST, It: Iterator>(
    root: HAST::IdN,
    offsets: &mut It,
    stores: &'store HAST,
) -> (Position, Vec<HAST::IdN>, Vec<It::Item>)
where
    It::Item: Clone + PrimInt,
    HAST::IdN: Clone,
    HAST: HyperAST<'store>,
    HAST::T: WithSerialization + WithChildren<ChildIdx = It::Item>,
{
    let mut offset = 0;
    let mut x = root;
    let mut path_ids = vec![];
    let mut no_spaces = vec![];
    let mut path = vec![];
    for o in &mut *offsets {
        // dbg!(offset);
        let b = stores.node_store().resolve(&x);
        // dbg!(b.get_type());
        // dbg!(o.to_usize().unwrap());

        let t = stores.type_store().resolve_type(&b);

        if t.is_directory() || t.is_file() {
            let l = stores.label_store().resolve(b.get_label_unchecked());
            path.push(l);
        }
        let mut no_s_idx = zero();
        if let Some(cs) = b.children() {
            let cs = cs.clone();
            if !t.is_directory() {
                for y in cs.before(o.clone()).iter_children() {
                    let b = stores.node_store().resolve(y);
                    if !stores.type_store().resolve_type(&b).is_spaces() {
                        no_s_idx = no_s_idx + one();
                    }
                    offset += b.try_bytes_len().unwrap().to_usize().unwrap();
                }
            } else {
                no_s_idx = o;
                // for y in 0..o.to_usize().unwrap() {
                //     let b = stores.node_store().resolve(cs[y]);
                //     println!("{:?}",b.get_type());
                // }
            }
            // if o.to_usize().unwrap() >= cs.len() {
            //     // dbg!("fail");
            // }
            if let Some(a) = cs.get(o) {
                x = a.clone();
                no_spaces.push(no_s_idx);
                path_ids.push(x.clone());
            } else {
                dbg!();
                break;
            }
        } else {
            dbg!();
            break;
        }
    }
    assert!(offsets.next().is_none());
    let b = stores.node_store().resolve(&x);
    let t = stores.type_store().resolve_type(&b);
    if t.is_directory() || t.is_file() {
        let l = stores.label_store().resolve(b.get_label_unchecked());
        path.push(l);
    }

    let len = if !t.is_directory() {
        b.try_bytes_len().unwrap().to_usize().unwrap()
    } else {
        0
    };
    let path = PathBuf::from_iter(path.iter());
    path_ids.reverse();
    (
        Position {
            file: path,
            offset,
            len,
        },
        path_ids,
        no_spaces,
    )
}

impl<'a, IdN: NodeId + Eq + Copy, Idx: PrimInt> ExploreStructuralPositions<'a, IdN, Idx> {
    pub fn make_position<'store, HAST>(self, stores: &'store HAST) -> Position
    where
        'a: 'store,
        HAST: HyperAST<'store, IdN = IdN::IdN, Label = LabelIdentifier>,
        HAST::T: Typed<Type = AnyType> + WithSerialization + WithChildren,
        <<HAST as HyperAST<'store>>::T as types::WithChildren>::ChildIdx: Debug,
        IdN: Debug + NodeId,
        IdN::IdN: NodeId<IdN = IdN::IdN> + Eq + Debug,
    {
        self.sps.check(stores).unwrap();
        // let parents = self.parents.iter().peekable();
        let mut from_file = false;
        // let mut len = 0;
        let len = if let Some(x) = self.peek_node() {
            let b = stores.node_store().resolve(x.as_id());
            let t = stores.type_store().resolve_type(&b);
            if let Some(y) = b.try_bytes_len() {
                if t.is_file() {
                    from_file = true;
                }
                y as usize
                // Some(x)
            } else {
                0
                // None
            }
        } else {
            0
            // None
        };
        let offset = 0;
        let path = vec![];
        self.make_position_aux(stores, from_file, len, offset, path)
    }

    fn make_position_aux<'store: 'a, HAST>(
        mut self,
        stores: &'store HAST,
        from_file: bool,
        len: usize,
        mut offset: usize,
        mut path: Vec<&'a str>,
    ) -> Position
    where
        HAST: HyperAST<'store, IdN = IdN::IdN, Label = LabelIdentifier>,
        HAST::T: Typed<Type = AnyType> + WithSerialization + WithChildren,
        IdN: Copy + Debug + NodeId,
        IdN::IdN: NodeId<IdN = IdN::IdN> + Eq + Debug,
    {
        // println!(
        //     "it: {:?},{:?},{:?}",
        //     &it.sps.nodes, &it.sps.offsets, &it.sps.parents
        // );
        if from_file {
            while let Some(p) = self.peek_parent_node() {
                // println!("i: {}", it.i);
                assert_ne!(p, self.peek_node().unwrap());
                assert_eq!(p, self.sps.nodes[self.sps.parents[self.i - 1]]);
                assert_eq!(self.peek_node().unwrap(), self.sps.nodes[self.i - 1]);
                // println!("nodes: {}, parents:{}, offsets:{}",it.sps.nodes.len(),it.sps.parents.len(),it.sps.offsets.len());
                let b = stores.node_store().resolve(p.as_id());
                let t = stores.type_store().resolve_type(&b);
                // println!("T0:{:?}", t);
                // let o = it.sps.offsets[it]
                // println!("nodes: ({})", it.sps.nodes.len());
                // println!("offsets: ({}) {:?}", it.sps.offsets.len(), &it.sps.offsets);
                // println!("parents: ({}) {:?}", it.sps.parents.len(), &it.sps.parents);
                // println!(
                //     "o: {}, o p: {}",
                //     it.peek_offset().unwrap(),
                //     it.sps.offsets[it.sps.parents[it.i - 1]]
                // );
                let o = self.peek_offset().unwrap();
                let o: <HAST::T as WithChildren>::ChildIdx = num::cast(o).unwrap();
                if self.peek_node().unwrap().as_id() != &b.children().unwrap()[o - one()] {
                    if self.peek_node().unwrap().as_id() != &b.children().unwrap()[o - one()] {
                        log::error!("backtrace: {}", std::backtrace::Backtrace::force_capture());
                    }
                    assert_eq!(
                        self.peek_node().unwrap().as_id(),
                        &b.children().unwrap()[o - one()],
                        "p:{:?} b.cs:{:?} o:{:?} o p:{:?} i p:{}",
                        p,
                        b.children().unwrap().iter_children().collect::<Vec<_>>(),
                        self.peek_offset().unwrap(),
                        self.sps.offsets[self.sps.parents[self.i - 1]],
                        self.sps.parents[self.i - 1],
                    );
                }
                let c: usize = {
                    let v: Vec<_> = b
                        .children()
                        .unwrap()
                        .before(o - one())
                        .iter_children()
                        .collect();
                    v.iter()
                        .map(|x| {
                            let b = stores.node_store().resolve(x);
                            // println!("{:?}", b.get_type());
                            // println!("T1:{:?}", b.get_type());
                            b.try_bytes_len().unwrap() as usize
                        })
                        .sum()
                };
                offset += c;
                if t.is_file() {
                    self.next();
                    break;
                } else {
                    self.next();
                }
            }
        }
        for p in self {
            let b = stores.node_store().resolve(p.as_id());
            // println!("type {:?}", b.get_type());
            // if !b.has_label() {
            //     panic!("{:?} should have a label", b.get_type());
            // }
            let l = stores.label_store().resolve(b.get_label_unchecked());
            // println!("value: {}",l);
            // path = path.join(path)
            path.push(l)
        }
        let path = PathBuf::from_iter(path.iter().rev());
        Position {
            file: path,
            offset,
            len,
        }
    }

    fn peek_parent_node(&self) -> Option<IdN> {
        if self.i == 0 {
            return None;
        }
        let i = self.i - 1;
        let r = self.sps.nodes[self.sps.parents[i]];
        Some(r)
    }
    fn peek_offset(&self) -> Option<Idx> {
        if self.i == 0 {
            return None;
        }
        let i = self.i - 1;
        let r = self.sps.offsets[i];
        Some(r)
    }
    fn peek_node(&self) -> Option<IdN> {
        if self.i == 0 {
            return None;
        }
        let i = self.i - 1;
        let r = self.sps.nodes[i];
        Some(r)
    }
}
impl<'a, IdN, Idx> ExploreStructuralPositions<'a, IdN, Idx> {
    fn new(sps: &'a StructuralPositionStore<IdN, Idx>, x: usize) -> Self {
        Self { sps, i: x + 1 }
    }
}

impl<'a, IdN: Copy, Idx> Iterator for ExploreStructuralPositions<'a, IdN, Idx> {
    type Item = IdN;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == 0 {
            return None;
        } //println!("next: {} {}", self.i, self.sps.parents[self.i - 1]);
        let i = self.i - 1;
        let r = self.sps.nodes[i];
        if i > 0 {
            self.i = self.sps.parents[i] + 1;
        } else {
            self.i = i;
        }
        Some(r)
    }
}

impl<'a, IdN, Idx> From<(&'a StructuralPositionStore<IdN, Idx>, SpHandle)>
    for ExploreStructuralPositions<'a, IdN, Idx>
{
    fn from((sps, x): (&'a StructuralPositionStore<IdN, Idx>, SpHandle)) -> Self {
        Self::new(sps, x.0)
    }
}

impl<IdN: NodeId, Idx: PrimInt> StructuralPositionStore<IdN, Idx> {
    pub fn push_up_scout(&self, s: &mut Scout<IdN, Idx>) -> Option<IdN>
    where
        IdN: Copy + Eq + Debug,
    {
        s.up(self)
    }

    pub fn ends_positions<'store, HAST>(
        &'store self,
        stores: &'store HAST,
        ends: &[SpHandle],
    ) -> Vec<Position>
    where
        HAST: HyperAST<'store, IdN = IdN::IdN, Label = LabelIdentifier>,
        HAST::T: Typed<Type = AnyType> + WithSerialization + WithChildren,
        // HAST::Types: Eq + TypeTrait,
        <<HAST as HyperAST<'store>>::T as types::WithChildren>::ChildIdx: Debug,
        IdN: Copy + Eq + Debug + NodeId,
        IdN::IdN: NodeId<IdN = IdN::IdN> + Debug,
    {
        let mut r = vec![];
        for x in ends.iter() {
            let x = x.0;
            // let parents = self.parents.iter().peekable();
            let it = ExploreStructuralPositions::from((self, SpHandle(x)));
            r.push(it.make_position(stores));
        }
        r
    }

    /// would ease approximate comparisons with other ASTs eg. spoon
    /// the basic idea would be to take the position of the parent.
    /// would be better to directly use a relaxed comparison.
    pub fn to_relaxed_positions<'store, HAST: HyperAST<'store>>(
        &self,
        _stores: &HAST,
    ) -> Vec<Position> {
        todo!()
    }

    pub fn check_with<'store, HAST>(
        &self,
        stores: &'store HAST,
        scout: &Scout<IdN, Idx>,
    ) -> Result<(), String>
    where
        HAST: HyperAST<
            'store,
            // T = HashedNodeRef<'store>,
            IdN = IdN,
            Label = LabelIdentifier,
        >,
        HAST::T: WithChildren<ChildIdx = Idx>,
        <<HAST as HyperAST<'store>>::T as types::WithChildren>::ChildIdx: Debug,
        IdN: Copy + Eq + Debug + NodeId<IdN = IdN>,
    {
        scout.path.check(stores).map_err(|_| "bad path")?;
        if self.nodes.is_empty() {
            return Ok(());
        }
        let mut i = scout.ancestors;
        if !scout.path.nodes.is_empty() {
            let e = scout.path.nodes[0];
            let p = self.nodes[i];
            let o = scout.path.offsets[0];
            if o.is_zero() {
                if i != 0 {
                    return Err(format!("bad offset"));
                }
                return Ok(());
            }
            let o = o - one();
            let b = stores.node_store().resolve(&p);
            if !b.has_children() || Some(e) != b.child(&num::cast(o).unwrap()) {
                return Err(if b.has_children() {
                    format!("error on link: {:?} {:?} {:?}", b.child_count(), o, p,)
                } else {
                    format!("error no children on link: {:?} {:?}", o, p,)
                });
            }
        }

        while i > 0 {
            let e = self.nodes[i];
            let o = self.offsets[i] - one();
            let p = self.nodes[self.parents[i]];
            let b = stores.node_store().resolve(&p);
            if !b.has_children() || Some(e) != b.child(&num::cast(o).unwrap()) {
                return Err(if b.has_children() {
                    format!("error: {:?} {:?} {:?}", b.child_count(), o, p,)
                } else {
                    format!("error no children: {:?} {:?}", o, p,)
                });
            }
            i -= 1;
        }
        Ok(())
    }

    pub fn check<'store, HAST>(&self, stores: &'store HAST) -> Result<(), String>
    where
        HAST: HyperAST<'store, IdN = IdN::IdN, Label = LabelIdentifier>,
        HAST::T: WithChildren,
        <<HAST as HyperAST<'store>>::T as types::WithChildren>::ChildIdx: Debug,
        IdN: Copy + Eq + Debug + NodeId,
        IdN::IdN: NodeId<IdN = IdN::IdN>,
    {
        assert_eq!(self.offsets.len(), self.parents.len());
        assert_eq!(self.nodes.len(), self.parents.len());
        if self.nodes.is_empty() {
            return Ok(());
        }
        let mut i = self.nodes.len() - 1;

        while i > 0 {
            let e = self.nodes[i];
            let o = self.offsets[i] - one();
            let o: <HAST::T as WithChildren>::ChildIdx = num::cast(o).unwrap();
            let p = self.nodes[self.parents[i]];
            let b = stores.node_store().resolve(p.as_id());
            if !b.has_children() || Some(e.as_id()) != b.child(&o).as_ref() {
                return Err(if b.has_children() {
                    format!("error: {:?} {:?} {:?}", b.child_count(), o, p,)
                } else {
                    format!("error no children: {:?} {:?}", o, p,)
                });
            }
            i -= 1;
        }
        Ok(())
    }
}

impl<IdN: Copy, Idx: PrimInt> StructuralPositionStore<IdN, Idx> {
    pub fn push(&mut self, x: &mut Scout<IdN, Idx>) -> SpHandle {
        assert_eq!(x.path.nodes.len(), x.path.offsets.len());
        if x.path.offsets.is_empty() {
            return SpHandle(x.ancestors);
        }
        assert!(
            !x.path.offsets[1..].contains(&zero()),
            "{:?}",
            &x.path.offsets
        );
        if x.path.offsets[0].is_zero() {
            assert!(x.ancestors == 0, "{:?} {}", &x.path.offsets, &x.ancestors);
            if x.path.offsets.len() == 1 {
                return SpHandle(0);
            }
            let l = x.path.nodes.len() - 2;
            let o = self.parents.len();
            self.nodes.extend(&x.path.nodes[1..]);

            self.parents.push(x.ancestors);
            self.parents
                .extend((o..o + l).into_iter().collect::<Vec<_>>());

            self.offsets.extend(&x.path.offsets[1..]);
            x.ancestors = self.nodes.len() - 1;
            x.path = StructuralPosition {
                nodes: vec![],
                offsets: vec![],
            }
        } else {
            let l = x.path.nodes.len() - 1;
            let o = self.parents.len();
            self.nodes.extend(x.path.nodes.clone());
            self.parents.push(x.ancestors);
            self.parents
                .extend((o..o + l).into_iter().collect::<Vec<_>>());
            self.offsets.extend(&x.path.offsets);
            // self.ends.push(self.nodes.len() - 1);
            x.ancestors = self.nodes.len() - 1;
            x.path = StructuralPosition {
                nodes: vec![],
                offsets: vec![],
            }
            // x.path = StructuralPosition::with_offset(x.path.current_node(), x.path.current_offset());
        }

        // if !x.path.offsets.is_empty() && x.path.offsets[0] == 0 {
        //     assert!(x.root == 0, "{:?} {}", &x.path.offsets, &x.root);
        // }

        assert!(
            self.offsets.is_empty() || !self.offsets[1..].contains(&zero()),
            "{:?}",
            &self.offsets
        );
        assert_eq!(self.offsets.len(), self.parents.len());
        assert_eq!(self.nodes.len(), self.parents.len());
        SpHandle(self.nodes.len() - 1)
    }
    pub fn push_typed<TIdN: TypedNodeId<IdN = IdN>>(
        &mut self,
        x: &mut TypedScout<TIdN, Idx>,
    ) -> SpHandle {
        assert_eq!(x.path.nodes.len(), x.path.offsets.len());
        if x.path.offsets.is_empty() {
            return SpHandle(x.ancestors);
        }
        assert!(
            !x.path.offsets[1..].contains(&zero()),
            "{:?}",
            &x.path.offsets
        );
        if x.path.offsets[0].is_zero() {
            assert!(x.ancestors == 0, "{:?} {}", &x.path.offsets, &x.ancestors);
            if x.path.offsets.len() == 1 {
                return SpHandle(0);
            }
            let l = x.path.nodes.len() - 2;
            let o = self.parents.len();
            self.nodes.extend(&x.path.nodes[1..]);

            self.parents.push(x.ancestors);
            self.parents
                .extend((o..o + l).into_iter().collect::<Vec<_>>());

            self.offsets.extend(&x.path.offsets[1..]);
            x.ancestors = self.nodes.len() - 1;
            x.path = StructuralPosition {
                nodes: vec![],
                offsets: vec![],
            }
        } else {
            let l = x.path.nodes.len() - 1;
            let o = self.parents.len();
            self.nodes.extend(x.path.nodes.clone());
            self.parents.push(x.ancestors);
            self.parents
                .extend((o..o + l).into_iter().collect::<Vec<_>>());
            self.offsets.extend(&x.path.offsets);
            // self.ends.push(self.nodes.len() - 1);
            x.ancestors = self.nodes.len() - 1;
            x.path = StructuralPosition {
                nodes: vec![],
                offsets: vec![],
            }
            // x.path = StructuralPosition::with_offset(x.path.current_node(), x.path.current_offset());
        }

        // if !x.path.offsets.is_empty() && x.path.offsets[0] == 0 {
        //     assert!(x.root == 0, "{:?} {}", &x.path.offsets, &x.root);
        // }

        assert!(
            self.offsets.is_empty() || !self.offsets[1..].contains(&zero()),
            "{:?}",
            &self.offsets
        );
        assert_eq!(self.offsets.len(), self.parents.len());
        assert_eq!(self.nodes.len(), self.parents.len());
        SpHandle(self.nodes.len() - 1)
    }
}

impl<IdN, Idx: PrimInt> From<StructuralPosition<IdN, Idx>> for StructuralPositionStore<IdN, Idx> {
    fn from(x: StructuralPosition<IdN, Idx>) -> Self {
        let l = x.nodes.len();
        assert!(!x.offsets[1..].contains(&zero()));
        let nodes = x.nodes;
        Self {
            nodes,
            parents: (0..l).into_iter().collect(),
            offsets: x.offsets,
            // ends: vec![],
        }
    }
}

impl<IdN, Idx> Default for StructuralPositionStore<IdN, Idx> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            parents: Default::default(),
            offsets: Default::default(),
            // ends: Default::default(),
        }
    }
}

impl<IdN: Copy + Eq, Idx: PrimInt> StructuralPositionStore<IdN, Idx> {
    pub fn type_scout<TIdN: TypedNodeId<IdN = IdN>>(
        &mut self,
        s: &mut Scout<IdN, Idx>,
        i: &TIdN,
    ) -> TypedScout<TIdN, Idx> {
        assert!(&s.node_always(self) == i.as_id());
        self.push(s);
        assert_eq!(s.path.nodes.len(), 0);
        TypedScout {
            path: StructuralPosition {
                nodes: vec![],
                offsets: vec![],
            },
            ancestors: s.ancestors,
            tdepth: 1,
            phantom: PhantomData,
        }
    }
}
