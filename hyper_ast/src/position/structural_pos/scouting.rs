use super::super::{TreePathMut, TypedTreePath};
use super::{
    ExploreStructuralPositions, Position, PrimInt, StructuralPosition, StructuralPositionStore,
    TreePath,
};
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

#[derive(Clone, Debug)]
pub struct Scout<IdN, Idx> {
    pub(super) path: StructuralPosition<IdN, Idx>,
    pub(super) ancestors: usize,
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

impl<IdN: Eq + Copy, Idx: PrimInt> Scout<IdN, Idx> {
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
            return sp
                .get(super::SpHandle(self.ancestors + 1))
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
        sp.get(super::SpHandle(self.ancestors + 1))
            .make_position_aux(stores, from_file, len, offset, path)
    }
}

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
