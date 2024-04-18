use super::{tags, TreePath, TreePathMut};
use crate::types::{HyperAST, NodeId, NodeStore, Tree, WithChildren};
use crate::PrimInt;

/// BottomUp content
#[derive(Clone, Debug)]
pub struct StructuralPosition<IdN, Idx, Config = tags::TopDownFull> {
    pub(super) parents: Vec<IdN>, //parents? // most likely parents
    pub(super) offsets: Vec<Idx>,
    _phantom: std::marker::PhantomData<Config>,
}
impl<IdN, Idx, C> StructuralPosition<IdN, Idx, C> {
    pub(crate) fn empty() -> Self {
        Self {
            parents: vec![],
            offsets: vec![],
            _phantom: Default::default(),
        }
    }
    pub(crate) fn solved(self, node: IdN) -> SolvedStructuralPosition<IdN, Idx, C> {
        SolvedStructuralPosition {
            parents: self.parents,
            offsets: self.offsets,
            node,
            _phantom: Default::default(),
        }
    }
}

impl<IdN, Idx: PrimInt> super::position_accessors::WithOffsets for StructuralPosition<IdN, Idx> {
    type Idx = Idx;
}

impl<IdN, Idx: PrimInt> super::position_accessors::WithPath<IdN> for StructuralPosition<IdN, Idx> {}

impl<IdN, Idx: PrimInt> super::position_accessors::WithPreOrderOffsets
    for StructuralPosition<IdN, Idx>
{
    type It<'a> = SPIter<'a, Idx> where Idx: 'a, Self: 'a;

    fn iter_offsets(&self) -> Self::It<'_> {
        let mut iter = self.offsets.iter();
        iter.next().unwrap();
        SPIter(iter)
    }
}

pub struct SPIter<'a, Idx>(std::slice::Iter<'a, Idx>);

impl<'a, Idx: PrimInt> Iterator for SPIter<'a, Idx> {
    type Item = Idx;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|x| *x - num::one())
    }
}

/// BottomUp content
#[derive(Clone, Debug)]
pub struct SolvedStructuralPosition<IdN, Idx, Config = tags::TopDownFull> {
    pub(super) parents: Vec<IdN>,
    pub(super) offsets: Vec<Idx>,
    pub(super) node: IdN,
    _phantom: std::marker::PhantomData<Config>,
}
impl<IdN, Idx, C> Into<(IdN, Vec<Idx>)> for SolvedStructuralPosition<IdN, Idx, C> {
    fn into(self) -> (IdN, Vec<Idx>) {
        (self.node, self.offsets)
    }
}
impl<IdN, Idx, C> From<SolvedStructuralPosition<IdN, Idx, C>> for StructuralPosition<IdN, Idx, C> {
    fn from(value: SolvedStructuralPosition<IdN, Idx, C>) -> Self {
        Self {
            parents: value.parents,
            offsets: value.offsets,
            _phantom: Default::default(),
        }
    }
}
// #[derive(Clone, Debug)]
// pub struct RootedStructuralPosition<IdN, Idx> {
//     pub(super) nodes: Vec<IdN>,
//     pub(super) offsets: Vec<Idx>,
//     pub(super) root: IdN,
// }

impl<IdN: Copy, Idx: PrimInt> TreePath<IdN, Idx> for StructuralPosition<IdN, Idx> {
    fn node(&self) -> Option<&IdN> {
        self.parents.last()
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
        assert_eq!(self.offsets.len(), self.parents.len());
        if self.parents.is_empty() {
            return Ok(());
        }
        let mut i = self.parents.len() - 1;

        while i > 0 {
            let e = self.parents[i];
            let o = self.offsets[i] - one();
            let p = self.parents[i - 1];
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
        Some((self.parents.pop()?, self.offsets.pop()?))
    }

    fn goto(&mut self, node: IdN, i: Idx) {
        use num::one;
        self.parents.push(node);
        // self.offsets.push(i);
        // TODO remove or justify usage right here
        self.offsets.push(i + one());
    }

    fn inc(&mut self, node: IdN) {
        use num::one;
        *self.parents.last_mut().unwrap() = node;
        *self.offsets.last_mut().unwrap() += one();
    }

    fn dec(&mut self, node: IdN) {
        use num::one;
        *self.parents.last_mut().unwrap() = node;
        if let Some(offsets) = self.offsets.last_mut() {
            assert!(*offsets >= one());
            *offsets -= one();
        }
    }
}

impl<IdN, Idx: num::Zero, C> StructuralPosition<IdN, Idx, C> {
    pub fn new(node: IdN) -> Self {
        Self {
            parents: vec![node],
            offsets: vec![num::zero()],
            _phantom: Default::default(),
        }
    }
}

impl<IdN, Idx> From<(Vec<IdN>, Vec<Idx>, IdN)> for StructuralPosition<IdN, Idx> {
    fn from(mut x: (Vec<IdN>, Vec<Idx>, IdN)) -> Self {
        assert_eq!(x.0.len() + 1, x.1.len());
        x.0.push(x.2);
        Self {
            parents: x.0,
            offsets: x.1,
            _phantom: Default::default(),
        }
    }
}
impl<IdN, Idx> From<(Vec<IdN>, Vec<Idx>)> for StructuralPosition<IdN, Idx> {
    fn from(x: (Vec<IdN>, Vec<Idx>)) -> Self {
        assert_eq!(x.0.len(), x.1.len());
        Self {
            parents: x.0,
            offsets: x.1,
            _phantom: Default::default(),
        }
    }
}
impl<IdN, Idx: num::Zero> From<IdN> for StructuralPosition<IdN, Idx> {
    fn from(node: IdN) -> Self {
        Self::new(node)
    }
}

mod impl_c_p_p_receivers {

    use super::super::building;
    use super::PrimInt;
    use super::SolvedStructuralPosition;
    use super::StructuralPosition;
    use building::top_down;

    impl<IdN, Idx: PrimInt, C> top_down::CreateBuilder for StructuralPosition<IdN, Idx, C> {
        fn create() -> Self {
            Self {
                offsets: vec![],
                parents: vec![],
                _phantom: Default::default(),
            }
        }
    }

    impl<IdN, Idx: PrimInt, C> top_down::ReceiveParent<IdN, Self> for StructuralPosition<IdN, Idx, C> {
        fn push(self, _parent: IdN) -> Self {
            self
        }
    }

    impl<IdN, Idx: PrimInt, C> building::top_down::ReceiveDirName<Self>
        for StructuralPosition<IdN, Idx, C>
    {
        fn push(self, _dir_name: &str) -> Self {
            self
        }
    }

    impl<IdN, Idx: PrimInt, C> building::bottom_up::ReceiveDirName<Self>
        for StructuralPosition<IdN, Idx, C>
    {
        fn push(self, _dir_name: &str) -> Self {
            self
        }
    }

    // impl<IdN, Idx: PrimInt, C> top_down::ReceiveIdx<Idx, Self> for SolvedStructuralPosition<IdN, Idx, C> {
    //     fn push(mut self, idx: Idx) -> Self {
    //         self.offsets.push(idx);
    //         self
    //     }
    // }

    impl<IdN, Idx: PrimInt, C> building::top_down::ReceiveIdx<Idx, Self>
        for StructuralPosition<IdN, Idx, C>
    {
        fn push(self, _idx: Idx) -> Self {
            // self.offsets.push(idx);
            self
        }
    }

    // impl<IdN, Idx: PrimInt, C> top_down::ReceiveIdxNoSpace<Idx, Self> for SolvedStructuralPosition<IdN, Idx, C> {
    //     fn push(self, _idx: Idx) -> Self {
    //         //self.offsets.push(idx);
    //         self
    //     }
    // }

    impl<IdN, Idx: PrimInt, C> building::top_down::ReceiveIdxNoSpace<Idx, Self>
        for StructuralPosition<IdN, Idx, C>
    {
        fn push(mut self, idx: Idx) -> Self {
            self.offsets.push(idx);
            self
        }
    }

    impl<IdN, Idx: PrimInt, C> top_down::FileSysReceiver for StructuralPosition<IdN, Idx, C> {
        type InFile<O> = Self;
    }

    impl<IdN, Idx: PrimInt, IdO, C> building::top_down::ReceiveOffset<IdO, Self>
        for StructuralPosition<IdN, Idx, C>
    {
        fn push(self, _bytes: IdO) -> Self {
            self
        }
    }
    impl<IdN, Idx: PrimInt, IdO, C> building::SetLen<IdO, Self> for StructuralPosition<IdN, Idx, C> {
        fn set(self, _len: IdO) -> Self {
            self
        }
    }
    // impl<IdN, Idx: PrimInt, C> top_down::SetNode<IdN, SolvedStructuralPosition<IdN, Idx, C>>
    //     for StructuralPosition<IdN, Idx, C>
    // {
    //     fn set_node(self, node: IdN) -> SolvedStructuralPosition<IdN, Idx, C> {
    //         self.solved(node)
    //     }
    // }
    impl<IdN, Idx: PrimInt, C> top_down::SetNode<IdN, SolvedStructuralPosition<IdN, Idx, C>>
        for StructuralPosition<IdN, Idx, C>
    {
        fn set_node(self, node: IdN) -> SolvedStructuralPosition<IdN, Idx, C> {
            self.solved(node)
        }
    }
    impl<IdN, Idx: PrimInt, C> top_down::SetFileName<Self> for StructuralPosition<IdN, Idx, C> {
        fn set_file_name(self, file_name: &str) -> Self {
            self
        }
    }
    impl<IdN, Idx: PrimInt, C> building::Transition<Self> for StructuralPosition<IdN, Idx, C> {
        fn transit(self) -> Self {
            self
        }
    }
}
