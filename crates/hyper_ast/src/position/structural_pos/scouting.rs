use super::super::TreePathMut;
use super::{Position, StructuralPosition, StructuralPositionStore, TreePath};
use crate::types::{
    AnyType, Children, Childrn, HyperAST, HyperType, LabelStore, Labeled, NodeId, Typed,
    WithChildren, WithSerialization,
};
use crate::PrimInt;
use num::{one, traits::NumAssign, zero};
use std::fmt::Debug;

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
        HAST: HyperAST<IdN = IdN::IdN>,
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
        if self.path.parents.is_empty() {
            self.ancestors != zero()
        } else {
            true
        }
    }
}

impl<IdN: Eq + Copy, Idx: PrimInt> Scout<IdN, Idx> {
    pub fn _up(&mut self) {
        self.path.pop();
        assert_eq!(self.path.parents.len(), self.path.offsets.len());
    }
    pub fn make_child(&self, node: IdN, i: Idx) -> Self {
        let mut s = self.clone();
        s.path.goto(node, i);
        s
    }
    pub fn up(&mut self, x: &StructuralPositionStore<IdN, Idx>) -> Option<IdN> {
        if self.path.parents.is_empty() {
            self.path = StructuralPosition::empty();
            assert_eq!(self.path.parents.len(), self.path.offsets.len());
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
    pub fn make_position<'store, HAST>(
        &self,
        sp: &StructuralPositionStore<HAST::IdN, Idx>,
        stores: &'store HAST,
    ) -> Position
    where
        HAST: HyperAST<IdN = IdN, Idx = Idx>,
        for<'t> <HAST as crate::types::AstLending<'t>>::RT:
            Typed<Type = AnyType> + WithSerialization,
        HAST::Idx: Debug,
        IdN: Copy + Debug + NodeId<IdN = IdN>,
    {
        self.check(stores).unwrap();
        let mut from_file = false;
        let x = self.node_always(sp);
        let b = stores.resolve(&x);
        let t = stores.resolve_type(&x);
        let len = if let Some(y) = b.try_bytes_len() {
            if !(t.is_file() || t.is_directory()) {
                from_file = true;
            }
            y as usize
        } else {
            0
        };
        let mut offset = 0;
        let mut path = vec![];
        if self.path.parents.is_empty() {
            return sp
                .get(super::SpHandle(self.ancestors + 1))
                .make_position_aux(stores, from_file, len, offset, path);
        }
        let mut i = self.path.parents.len() - 1;
        if from_file {
            while i > 0 {
                let p = self.path.parents[i - 1];
                let b = stores.resolve(&p);
                let t = stores.resolve_type(&p);
                let o = self.path.offsets[i];
                let o: HAST::Idx = num::cast(o).unwrap();
                let c: usize = {
                    let v: Vec<_> = b
                        .children()
                        .unwrap()
                        .before(o - one())
                        .iter_children()
                        .collect();
                    v.iter()
                        .map(|x| {
                            let b = stores.resolve(x);
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
        if self.path.parents.is_empty() {
        } else if !from_file {
            loop {
                from_file = false;
                let n = self.path.parents[i];
                let b = stores.resolve(&n);
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
                self.path.parents[i - 1]
            };
            let b = stores.resolve(&p);
            let t = stores.resolve_type(&p);
            let o = self.path.offsets[i];
            let o: HAST::Idx = num::cast(o).unwrap();
            let c: usize = {
                b.children()
                    .unwrap()
                    .before(o - one())
                    .iter_children()
                    .map(|x| stores.resolve(&x).try_bytes_len().unwrap())
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
            (path.parents[1..].to_owned(), path.offsets[1..].to_owned()).into()
        } else {
            path
        };
        Self { path, ancestors }
    }
}
