use super::{Position, PrimInt, TreePath, WithHyperAstPositionConverter};
use std::{
    fmt::{Debug, Display},
    path::{Path, PathBuf},
};

use num::{one, zero};

use crate::{
    store::defaults::{LabelIdentifier, NodeIdentifier},
    types::{
        self, AnyType, Children, HyperAST, HyperType, IterableChildren, LabelStore, Labeled,
        NodeId, NodeStore, Tree, TypeStore, Typed, TypedNodeId, WithChildren, WithSerialization,
    },
};

pub use super::offsets_and_nodes::StructuralPosition;

mod path_store;

mod scouting;
pub use scouting::*;

mod typed_scouting;
pub use typed_scouting::*;

#[derive(Clone)]
pub struct ExploreStructuralPositions<'a, IdN, Idx = usize> {
    sps: &'a StructuralPositionStore<IdN, Idx>,
    i: usize,
}
mod esp_impl {
    use super::super::position_accessors::*;
    use super::*;
    impl<'a, IdN: NodeId + Eq + Copy, Idx: PrimInt> SolvedPositionT<IdN>
        for ExploreStructuralPositions<'a, IdN, Idx>
    {
        fn node(&self) -> IdN {
            self.peek_node().unwrap()
        }
    }
    impl<'a, IdN: NodeId + Eq + Copy, Idx: PrimInt> RootedPositionT<IdN>
        for ExploreStructuralPositions<'a, IdN, Idx>
    {
        fn root(&self) -> IdN {
            todo!("value must be computed")
        }
    }
    impl<'a, IdN: NodeId + Eq + Copy, Idx: PrimInt> PathPositionT<IdN>
        for ExploreStructuralPositions<'a, IdN, Idx>
    {
        type Idx = Idx;
    }
    impl<'a, IdN: NodeId + Eq + Copy, Idx: PrimInt> PostOrderPathPositionT<IdN>
        for ExploreStructuralPositions<'a, IdN, Idx>
    {
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpHandle(pub(super) usize);

pub struct StructuralPositionStore<IdN = NodeIdentifier, Idx = u16> {
    pub nodes: Vec<IdN>,
    parents: Vec<usize>,
    offsets: Vec<Idx>,
    // ends: Vec<usize>,
}

// #[derive(Clone, Debug)]
// pub struct StructuralPositionWithIndentation {
//     pub(crate) nodes: Vec<NodeIdentifier>,
//     pub(crate) offsets: Vec<usize>,
//     pub(crate) indentations: Vec<Box<[Space]>>,
// }
impl<'a, IdN: NodeId + Eq + Copy, Idx: PrimInt> ExploreStructuralPositions<'a, IdN, Idx> {
    pub(super) fn peek_parent_node(&self) -> Option<IdN> {
        if self.i == 0 {
            return None;
        }
        let i = self.i - 1;
        let r = self.sps.nodes[self.sps.parents[i]];
        Some(r)
    }
    pub(super) fn peek_offset(&self) -> Option<Idx> {
        if self.i == 0 {
            return None;
        }
        let i = self.i - 1;
        let r = self.sps.offsets[i];
        Some(r)
    }
    pub(super) fn peek_node(&self) -> Option<IdN> {
        if self.i == 0 {
            return None;
        }
        let i = self.i - 1;
        let r = self.sps.nodes[i];
        Some(r)
    }
}
// impl<'a, IdN, Idx> ExploreStructuralPositions<'a, IdN, Idx> {
//     pub(super) fn new(sps: &'a StructuralPositionStore<IdN, Idx>, x: usize) -> Self {
//         Self { sps, i: x + 1 }
//     }
// }

// impl<'a, IdN, Idx> From<(&'a StructuralPositionStore<IdN, Idx>, SpHandle)>
//     for ExploreStructuralPositions<'a, IdN, Idx>
// {
//     fn from((sps, x): (&'a StructuralPositionStore<IdN, Idx>, SpHandle)) -> Self {
//         Self::new(sps, x.0)
//     }
// }

impl<'a, IdN: Copy, Idx> Iterator for ExploreStructuralPositions<'a, IdN, Idx> {
    type Item = IdN;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_go_up().map(|i|self.sps.nodes[i.0])
        // if self.i == 0 {
        //     return None;
        // } //println!("next: {} {}", self.i, self.sps.parents[self.i - 1]);
        // let i = self.i - 1;
        // let r = self.sps.nodes[i];
        // if i > 0 {
        //     self.i = self.sps.parents[i] + 1;
        // } else {
        //     self.i = i;
        // }
        // Some(r)
    }
}
impl<'a, IdN, Idx> ExploreStructuralPositions<'a, IdN, Idx> {
    /// return previous index
    #[inline]
    fn try_go_up(&mut self) -> Option<SpHandle> {
        if self.i == 0 {
            return None;
        } //println!("next: {} {}", self.i, self.sps.parents[self.i - 1]);
        let i = self.i - 1;
        let r = i;
        if i > 0 {
            self.i = self.sps.parents[i] + 1;
        } else {
            self.i = i;
        }
        Some(SpHandle(r))
    }
}

impl<'store, 'src, 'a, IdN: NodeId + Eq + Copy, Idx: PrimInt, HAST>
    WithHyperAstPositionConverter<'store, 'src, ExploreStructuralPositions<'a, IdN, Idx>, HAST>
{
    pub fn make_file_and_offset(&self) -> Position
    where
        'a: 'store,
        HAST: HyperAST<'store, IdN = IdN::IdN, Label = LabelIdentifier>,
        HAST::T: Typed<Type = AnyType> + WithSerialization + WithChildren,
        <<HAST as HyperAST<'store>>::T as types::WithChildren>::ChildIdx: Debug,
        IdN: Debug + NodeId,
        IdN::IdN: NodeId<IdN = IdN::IdN> + Eq + Debug,
    {
        self.src.clone().make_position(self.stores)
    }
}

impl<'store, 'src, 'a, IdN: NodeId + Eq + Copy, Idx: PrimInt, HAST>
    From<
        WithHyperAstPositionConverter<'store, 'src, ExploreStructuralPositions<'a, IdN, Idx>, HAST>,
    > for Position
where
    'a: 'store,
    HAST: HyperAST<'store, IdN = IdN::IdN, Label = LabelIdentifier>,
    HAST::T: Typed<Type = AnyType> + WithSerialization + WithChildren,
    <<HAST as HyperAST<'store>>::T as types::WithChildren>::ChildIdx: Debug,
    IdN: Debug + NodeId,
    IdN::IdN: NodeId<IdN = IdN::IdN> + Eq + Debug,
{
    fn from(
        value: WithHyperAstPositionConverter<
            'store,
            'src,
            ExploreStructuralPositions<'a, IdN, Idx>,
            HAST,
        >,
    ) -> Self {
        WithHyperAstPositionConverter::make_file_and_offset(&value)
    }
}

// TODO separate concerns
// TODO make_position should be a From<ExploreStructuralPositions> for FileAndOffsetPostionT and moved to relevant place
// TODO here the remaining logic should be about giving an iterator through the structural position
impl<'a, IdN: NodeId + Eq + Copy, Idx: PrimInt> ExploreStructuralPositions<'a, IdN, Idx> {
    fn make_position<'store, HAST>(self, stores: &'store HAST) -> Position
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
        let file = PathBuf::from_iter(path.iter().rev());
        Position::new(file, offset, len)
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
