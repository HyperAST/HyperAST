use super::{PrimInt, StructuralPosition, TreePath, TreePathMut};

use crate::types::{HyperAST, NodeId, NodeStore, Tree, WithChildren};

#[derive(Clone, Debug)]
pub struct Position<IdN, Idx> {
    pub(super) nodes: Vec<IdN>,
    pub(super) offsets: Vec<Idx>,
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
        use num::one;
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

impl<IdN: Copy, Idx: PrimInt> TreePathMut<IdN, Idx> for StructuralPosition<IdN, Idx> {
    fn pop(&mut self) -> Option<(IdN, Idx)> {
        Some((self.nodes.pop()?, self.offsets.pop()?))
    }

    fn goto(&mut self, node: IdN, i: Idx) {
        use num::one;
        self.nodes.push(node);
        self.offsets.push(i + one());
    }

    fn inc(&mut self, node: IdN) {
        use num::one;
        *self.nodes.last_mut().unwrap() = node;
        *self.offsets.last_mut().unwrap() += one();
    }

    fn dec(&mut self, node: IdN) {
        use num::one;
        *self.nodes.last_mut().unwrap() = node;
        if let Some(offsets) = self.offsets.last_mut() {
            assert!(*offsets > one());
            *offsets -= one();
        }
    }
}
