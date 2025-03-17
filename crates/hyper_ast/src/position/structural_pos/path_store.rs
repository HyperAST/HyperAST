use super::{
    super::Position, ExploreStructuralPositions, Scout, SpHandle, StructuralPosition,
    StructuralPositionStore,
};
use crate::{
    position::TreePath,
    store::defaults::LabelIdentifier,
    types::{
        self, AnyType, HyperAST, NodeId, NodeStore, Tree, Typed, WithChildren, WithSerialization,
        WithStats,
    },
    PrimInt,
};
use num::{one, zero};
use std::fmt::Debug;

impl<IdN, Idx: PrimInt> StructuralPositionStore<IdN, Idx> {
    pub fn with_position(x: StructuralPosition<IdN, Idx>) -> Self {
        let l = x.parents.len();
        assert!(!x.offsets[1..].contains(&zero()));
        let nodes = x.parents;
        Self {
            nodes,
            parents: (0..l).into_iter().collect(),
            offsets: x.offsets,
            // ends: vec![],
        }
    }
    pub fn new(root: IdN) -> Self {
        Self::with_position(StructuralPosition::new(root))
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

impl<IdN: NodeId, Idx: PrimInt> StructuralPositionStore<IdN, Idx> {
    pub fn get(&self, s: SpHandle) -> ExploreStructuralPositions<IdN, Idx> {
        ExploreStructuralPositions {
            sps: self,
            i: s.0,
            _phantom: Default::default(),
        }
    }

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
        HAST: HyperAST<IdN = IdN, Label = LabelIdentifier, Idx = Idx>,
        for<'t> <HAST as crate::types::AstLending<'t>>::RT: Typed<Type = AnyType> + WithSerialization + WithChildren + WithStats,
        // HAST::Types: Eq + TypeTrait,
        HAST::Idx:  Debug,
        IdN: Copy + Eq + Debug + NodeId,
        IdN: NodeId<IdN = IdN> + Debug,
    {
        let mut r = vec![];
        for x in ends.iter() {
            // let parents = self.parents.iter().peekable();
            let it = self.get(*x);
            let position_converter =
                &crate::position::PositionConverter::new(&it).with_stores(stores);
            r.push(position_converter.compute_pos_post_order::<_, Position>())
            // r.push(it.make_position(stores));
        }
        r
    }

    /// would ease approximate comparisons with other ASTs eg. spoon
    /// the basic idea would be to take the position of the parent.
    /// would be better to directly use a relaxed comparison.
    pub fn to_relaxed_positions<HAST: HyperAST>(
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
            // T = HashedNodeRef<'store>,
            IdN = IdN,
            Label = LabelIdentifier,
        >,
        for<'t> <HAST as crate::types::AstLending<'t>>::RT: WithChildren<ChildIdx = Idx>,
        HAST::Idx:  Debug,
        IdN: Copy + Eq + Debug + NodeId<IdN = IdN>,
    {
        scout.path.check(stores).map_err(|_| "bad path")?;
        if self.nodes.is_empty() {
            return Ok(());
        }
        let mut i = scout.ancestors;
        if !scout.path.parents.is_empty() {
            let e = scout.path.parents[0];
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
        HAST: HyperAST<IdN = IdN::IdN>,
        for<'t> <HAST as crate::types::AstLending<'t>>::RT: WithChildren,
        HAST::Idx:  Debug,
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
            let o: HAST::Idx = num::cast(o).unwrap();
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
