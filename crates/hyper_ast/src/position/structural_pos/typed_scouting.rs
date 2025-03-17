use super::super::{TreePathMut, TypedTreePath};
use super::{Position, Scout, SpHandle, StructuralPosition, StructuralPositionStore, TreePath};
use std::{fmt::Debug, marker::PhantomData};

use num::zero;

use crate::types::{NodeId, Typed, TypedNodeId, WithChildren};

use crate::PrimInt;
use crate::{
    store::defaults::LabelIdentifier,
    types::{self, HyperAST, WithSerialization},
};

#[derive(Clone, Debug)]
pub struct TypedScout<TIdN: TypedNodeId, Idx> {
    pub(super) path: StructuralPosition<TIdN::IdN, Idx>,
    pub(super) ancestors: usize,
    pub(super) tdepth: i16,
    pub(super) phantom: PhantomData<TIdN>,
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
        HAST: HyperAST<IdN = <TIdN::IdN as NodeId>::IdN>,
        // for<'t> <HAST as crate::types::AstLending<'t>>::RT: WithChildren<ChildIdx = Idx>,
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
        let i = self.path.parents.len() - 1;
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
            self.path = StructuralPosition::empty();
            self.ancestors = x.parents[self.ancestors];
            let tdepth = -self.tdepth as usize;
            let i = self.path.parents.len();
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
            self.path = StructuralPosition::empty();
            self.ancestors = x.parents[self.ancestors];
            Ok(self)
        } else {
            let path = StructuralPosition::empty();
            let ancestors = x.parents[self.ancestors];
            Err(Scout { path, ancestors })
        }
    }
    pub fn _up(&mut self) {
        self.tdepth -= 1;
        self.path.pop();
        assert_eq!(self.path.parents.len(), self.path.offsets.len());
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
        let len = self.path.parents.len() - 1;
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
        _sp: &StructuralPositionStore<HAST::IdN, Idx>,
        _stores: &'store HAST,
    ) -> Position
    where
        HAST: HyperAST<
        // IdN = TIdN::IdN, 
        Label = LabelIdentifier>,
        HAST: crate::types::TypedHyperAST<TIdN>,
        // <HAST as crate::types::TypedHyperAST<'store, TIdN>>::TT:
        //     Typed<Type = TIdN::Ty> + WithSerialization + WithChildren,
        // <HAST as crate::types::HyperAST<'store>>::T: WithSerialization + WithChildren,
        // <<HAST as crate::types::TypedHyperAST<'store, TIdN>>::TT as types::WithChildren>::ChildIdx:
        //     Debug,
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

impl<IdN: Copy, Idx: PrimInt> StructuralPositionStore<IdN, Idx> {
    pub fn push_typed<TIdN: TypedNodeId<IdN = IdN>>(
        &mut self,
        x: &mut TypedScout<TIdN, Idx>,
    ) -> SpHandle {
        assert_eq!(x.path.parents.len(), x.path.offsets.len());
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
            let l = x.path.parents.len() - 2;
            let o = self.parents.len();
            self.nodes.extend(&x.path.parents[1..]);

            self.parents.push(x.ancestors);
            self.parents
                .extend((o..o + l).into_iter().collect::<Vec<_>>());

            self.offsets.extend(&x.path.offsets[1..]);
            x.ancestors = self.nodes.len() - 1;
            x.path = StructuralPosition::empty()
        } else {
            let l = x.path.parents.len() - 1;
            let o = self.parents.len();
            self.nodes.extend(x.path.parents.clone());
            self.parents.push(x.ancestors);
            self.parents
                .extend((o..o + l).into_iter().collect::<Vec<_>>());
            self.offsets.extend(&x.path.offsets);
            // self.ends.push(self.nodes.len() - 1);
            x.ancestors = self.nodes.len() - 1;
            x.path = StructuralPosition::empty()
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

impl<IdN: Copy + Eq, Idx: PrimInt> StructuralPositionStore<IdN, Idx> {
    pub fn type_scout<TIdN: TypedNodeId<IdN = IdN>>(
        &mut self,
        s: &mut Scout<IdN, Idx>,
        i: &TIdN,
    ) -> TypedScout<TIdN, Idx> {
        assert!(&s.node_always(self) == i.as_id());
        self.push(s);
        assert_eq!(s.path.parents.len(), 0);
        TypedScout {
            path: StructuralPosition::empty(),
            ancestors: s.ancestors,
            tdepth: 1,
            phantom: PhantomData,
        }
    }
}
