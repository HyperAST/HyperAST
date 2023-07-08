use std::marker::PhantomData;

use super::{position_accessors, tags, PrimInt};

// TODO remove PrimInt, also in impls
pub struct Offsets<Idx: PrimInt, Config = tags::TopDownFull> {
    /// offsets to go through a tree from top to bottom
    offsets: Vec<Idx>,
    _phantom: PhantomData<Config>,
}
impl<Idx: PrimInt, C> Into<Vec<Idx>> for Offsets<Idx, C> {
    fn into(self) -> Vec<Idx> {
        self.offsets
    }
}

pub struct OffsetsRef<'a, Idx: PrimInt, Config = tags::TopDownFull> {
    /// offsets to go through a tree from top to bottom
    offsets: &'a [Idx],
    _phantom: PhantomData<Config>,
}
impl<'a, Idx: PrimInt> From<&'a [Idx]> for OffsetsRef<'a, Idx> {
    fn from(offsets: &'a [Idx]) -> Self {
        Self {
            offsets,
            _phantom: PhantomData,
        }
    }
}
impl<'a, Idx: PrimInt> OffsetsRef<'a, Idx> {
    pub fn with_root<IdN>(self, root: IdN) -> RootedOffsetsRef<'a, IdN, Idx> {
        RootedOffsetsRef {
            root,
            offsets: self.offsets,
        }
    }
}

pub struct RootedOffsets<IdN, Idx> {
    root: IdN,
    /// offsets to go through a tree from top to bottom
    offsets: Vec<Idx>,
}

pub struct RootedOffsetsRef<'a, IdN, Idx> {
    root: IdN,
    /// offsets to go through a tree from top to bottom
    offsets: &'a [Idx],
}

impl<'a, IdN, Idx> super::node_filter_traits::Full for RootedOffsetsRef<'a, IdN, Idx> {}

impl<'a, IdN: Copy, Idx> position_accessors::RootedPosition<IdN> for RootedOffsetsRef<'a, IdN, Idx> {
    fn root(&self) -> IdN {
        self.root
    }
}
impl<'a, IdN: Copy, Idx: PrimInt> position_accessors::WithOffsets
    for RootedOffsetsRef<'a, IdN, Idx>
{
    type Idx = Idx;
}

impl<'a, IdN: Copy, Idx: PrimInt> position_accessors::WithPreOrderOffsets
    for RootedOffsetsRef<'a, IdN, Idx>
{
    type It = std::iter::Copied<std::slice::Iter<'a, Idx>>;

    fn iter(&self) -> Self::It {
        self.offsets.iter().copied()
    }
}

impl<'a, IdN: Copy, Idx: PrimInt> RootedOffsetsRef<'a, IdN, Idx> {
    pub fn with_store<'store, HAST>(
        &'a self,
        stores: &'store HAST,
    ) -> super::WithHyperAstPositionConverter<'store, 'a, Self, HAST> {
        super::PositionConverter::new(self).with_stores(stores)
    }
}

pub(super) struct SolvedPathPosition<IdN, Idx> {
    root: IdN,
    /// offsets to go through a tree from top to bottom
    offsets: Vec<Idx>,
    node: IdN,
}

mod impl_receivers {
    use super::super::building;
    use building::top_down;
    use super::super::PrimInt;
    use super::tags;
    use super::Offsets;

    impl<Idx: PrimInt, C> building::top_down::CreateBuilder for Offsets<Idx, C> {
        fn create() -> Self {
            Self {
                offsets: vec![],
                _phantom: std::marker::PhantomData,
            }
        }
    }

    impl<IdN, Idx: PrimInt, C> top_down::ReceiveParent<IdN, Self> for Offsets<Idx, C> {
        fn push(self, _parent: IdN) -> Self {
            self
        }
    }

    impl<Idx: PrimInt, C> building::top_down::ReceiveDirName<Self> for Offsets<Idx, C> {
        fn push(self, _dir_name: &str) -> Self {
            self
        }
    }

    impl<Idx: PrimInt> building::top_down::ReceiveIdx<Idx, Self> for Offsets<Idx> {
        fn push(mut self, idx: Idx) -> Self {
            self.offsets.push(idx);
            self
        }
    }

    impl<Idx: PrimInt> building::top_down::ReceiveIdx<Idx, Self> for Offsets<Idx, tags::TopDownNoSpace> {
        fn push(self, _idx: Idx) -> Self {
            // self.offsets.push(idx);
            self
        }
    }

    impl<Idx: PrimInt> building::top_down::ReceiveIdxNoSpace<Idx, Self> for Offsets<Idx> {
        fn push(self, _idx: Idx) -> Self {
            //self.offsets.push(idx);
            self
        }
    }

    impl<Idx: PrimInt> building::top_down::ReceiveIdxNoSpace<Idx, Self> for Offsets<Idx, tags::TopDownNoSpace> {
        fn push(mut self, idx: Idx) -> Self {
            self.offsets.push(idx);
            self
        }
    }

    impl<Idx: PrimInt, C> top_down::FileSysReceiver for Offsets<Idx, C> {
        type InFile<O> = Self;
    }

    impl<Idx: PrimInt, IdO: PrimInt, C> building::top_down::ReceiveOffset<IdO, Self> for Offsets<Idx, C> {
        fn push(self, _bytes: IdO) -> Self {
            self
        }
    }
    impl<Idx: PrimInt, IdO, C> building::SetLen<IdO, Self> for Offsets<Idx, C> {
        fn set(self, _len: IdO) -> Self {
            self
        }
    }
    impl<IdN, Idx: PrimInt, C> top_down::SetNode<IdN, Self> for Offsets<Idx, C> {
        fn set_node(self, _node: IdN) -> Self {
            self
        }
    }
    impl<Idx: PrimInt, C> top_down::SetFileName<Self> for Offsets<Idx, C> {
        fn set_file_name(self, _file_name: &str) -> Self {
            self
        }
    }
    impl<Idx: PrimInt, C> building::Transition<Self> for Offsets<Idx, C> {
        fn transit(self) -> Self {
            self
        }
    }

    // impl<IdN, Idx, IdO: PrimInt> top_down::ReceiveInFile<IdN, Idx, Self> for Offsets<Idx> {
    //     type S1 = Self;

    //     type S2 = Self;

    //     fn finish(self) -> Self {
    //         self
    //     }
    // }
    // impl<IdN, Idx, IdO: PrimInt> top_down::ReceiveDir<IdN, Idx, Self> for Offsets<Idx> {
    //     type SA1 = Self;

    //     type SA2 = Self;

    //     type SB1 = Self;

    //     fn go_inside_file(mut self, file_name: &str) -> Self::SB1 {
    //         self.file.push(file_name);
    //         self
    //     }

    //     fn finish(self) -> Self {
    //         self
    //     }
    // }
}
