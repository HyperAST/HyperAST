use super::{Position, PrimInt, TreePath};
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

pub type StructuralPosition<IdN = NodeIdentifier, Idx = u16> =
    super::offsets_and_nodes::Position<IdN, Idx>;

impl<IdN, Idx: num::Zero> StructuralPosition<IdN, Idx> {
    pub fn new(node: IdN) -> Self {
        Self {
            nodes: vec![node],
            offsets: vec![zero()],
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
pub struct SpHandle(pub(super) usize);

pub struct ExploreStructuralPositions<'a, IdN, Idx = usize> {
    sps: &'a StructuralPositionStore<IdN, Idx>,
    i: usize,
}
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
impl<'a, IdN, Idx> ExploreStructuralPositions<'a, IdN, Idx> {
    pub(super) fn new(sps: &'a StructuralPositionStore<IdN, Idx>, x: usize) -> Self {
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

mod scouting;
pub use scouting::*;

mod typed_scouting;
pub use typed_scouting::*;

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
