//! Declares interfaces for position builders,
//! while offering statemachine traits to orchestrate them statisticaly.
//!
//! Converting positions is a frequent operation, so performances is a major concern.
//!
//! The statemachines qre a "zero cost" abstraction, as they are completely monomorphised.
//!
//! With all these traits it is also easier to do multiple conversions with a single traversal,
//! with no impact to performances of "mono" convertions.

pub trait Transition<O> {
    fn transit(self) -> O;
}
pub trait SetLen<IdO, O> {
    fn set(self, len: IdO) -> O;
}
pub mod top_down {
    use super::*;
    pub trait CreateBuilder {
        fn create() -> Self;
    }
    pub trait ReceiveDirName<O> {
        fn push(self, dir_name: &str) -> O;
    }
    pub trait ReceiveIdx<Idx, O> {
        fn push(self, idx: Idx) -> O;
    }
    pub trait ReceiveIdxNoSpace<Idx, O> {
        fn push(self, idx: Idx) -> O;
    }
    pub trait ReceiveOffset<IdO, O> {
        fn push(self, bytes: IdO) -> O;
    }
    pub trait ReceiveParent<IdN, O> {
        fn push(self, parent: IdN) -> O;
    }
    pub trait SetNode<IdN, O> {
        fn set_node(self, node: IdN) -> O;
    }
    pub trait SetFileName<O> {
        fn set_file_name(self, file_name: &str) -> O;
    }
    // pub trait FileSysReceiver {
    //     type InFile;
    // }

    pub trait ReceiveDir<IdN, Idx, O>:
        Sized
        + ReceiveParent<IdN, Self::SA1>
        + SetNode<IdN, Self::O0>
        + SetFileName<Self::SB1<O>>
        + Transition<Self::SB1<O>>
    {
        type SA1: ReceiveIdx<Idx, Self::SA2>;
        type SA2: ReceiveDirName<Self>;
        type SB1<OO>;
        type O0: Transition<O>;
    }
    pub trait FileSysReceiver {
        type InFile<O>;
    }
    impl<IdN, Idx, O, T: FileSysReceiver> ReceiveDir<IdN, Idx, O> for T
    where
        T: ReceiveParent<IdN, T>
            + SetNode<IdN, T>
            + ReceiveIdx<Idx, T>
            + ReceiveDirName<T>
            + SetFileName<T::InFile<O>>
            + Transition<T::InFile<O>>,
        T: Transition<O>,
    {
        type SA1 = T;
        type SA2 = T;
        type SB1<OO> = T::InFile<OO>;
        type O0 = T;
    }

    pub trait ReceiveInFile<IdN, Idx, IdO, O>:
        Sized + ReceiveParent<IdN, Self::S1> + SetNode<IdN, Self::O0>
    {
        type S1: ReceiveIdx<Idx, Self::S2>;
        type S2: ReceiveOffset<IdO, Self::S3>;
        type S3: ReceiveIdxNoSpace<Idx, Self>;
        type O0: SetLen<IdO, Self::O1>;
        type O1: Transition<O>;
    }
    impl<IdN, Idx, IdO, O, T> ReceiveInFile<IdN, Idx, IdO, O> for T
    where
        T: ReceiveParent<IdN, T>
            + SetNode<IdN, T>
            + ReceiveOffset<IdO, T>
            + ReceiveIdx<Idx, T>
            + SetLen<IdO, T>
            + ReceiveIdxNoSpace<Idx, T>,
        T: Transition<O>,
    {
        type S1 = T;
        type S2 = T;
        type S3 = T;

        type O0 = T;
        type O1 = T;
    }

    // Great bu try to fusion with `ReceiveInFile`s
    pub trait ReceiveInFileNoSpace<IdN, Idx, IdO, O>:
        Sized + ReceiveParent<IdN, Self::S1> + SetNode<IdN, Self::O0>
    {
        type S1: ReceiveIdx<Idx, Self::S2>;
        type S2: ReceiveOffset<IdO, Self>;
        type O0: SetLen<IdO, Self::O1>;
        type O1: Transition<O>;
    }
    impl<IdN, Idx, IdO, O, T> ReceiveInFileNoSpace<IdN, Idx, IdO, O> for T
    where
        T: ReceiveParent<IdN, T>
            + SetNode<IdN, T>
            + ReceiveOffset<IdO, T>
            + ReceiveIdx<Idx, T>
            + SetLen<IdO, T>
            + ReceiveIdxNoSpace<Idx, T>,
        T: Transition<O>,
    {
        type S1 = T;
        type S2 = T;

        type O0 = T;
        type O1 = T;
    }
}
pub mod bottom_up {
    use super::*;
    pub trait CreateBuilder {
        fn create() -> Self;
    }
    pub trait ReceiveDirName<O> {
        fn push(self, dir_name: &str) -> O;
    }
    pub trait ReceiveIdx<Idx, O> {
        fn push(self, idx: Idx) -> O;
    }
    pub trait ReceiveIdxNoSpace<Idx, O> {
        fn push(self, idx: Idx) -> O;
    }
    pub trait ReceiveOffset<IdO, O> {
        fn push(self, bytes: IdO) -> O;
    }
    pub trait ReceiveNode<IdN, O> {
        fn push(self, node: IdN) -> O;
    }
    pub trait SetRoot<IdN, O> {
        fn set_root(self, root: IdN) -> O;
    }
    pub trait FileSysReceiver {
        type InFile<O>;
    }

    pub trait ReceiveInFile<IdN, Idx, IdO, O>:
        Sized + SetLen<IdO, Self::SA1> + Transition<Self::SB1<O>>
    {
        type SA1: ReceiveNode<IdN, Self::SA2> + ReceiveDirName<Self::SB1<O>> + SetRoot<IdN, O>;
        type SA2: ReceiveOffset<IdO, Self::SA3>;
        type SA3: ReceiveIdx<Idx, Self::SA1>;
        type SB1<OO>;
    }
    impl<IdN, Idx, IdO, O, T> ReceiveInFile<IdN, Idx, IdO, O> for T
    where
        T: ReceiveIdx<Idx, T>
            + ReceiveNode<IdN, T>
            + SetRoot<IdN, O>
            + ReceiveOffset<IdO, T>
            + ReceiveIdx<Idx, T>
            + ReceiveDirName<T>
            + SetLen<IdO, T>,
        T: Transition<T>,
        T: Transition<O>,
    {
        type SA1 = T;
        type SA2 = T;
        type SA3 = T;
        type SB1<OO> = T;
    }
    pub trait ReceiveDir<IdN, Idx, O>:
        Sized + ReceiveNode<IdN, Self::S1> + SetRoot<IdN, O>
    {
        type S1: ReceiveIdx<Idx, Self>;
        type S2: ReceiveDirName<Self>;
    }
    impl<IdN, Idx, O, T> ReceiveDir<IdN, Idx, O> for T
    where
        T: ReceiveIdx<Idx, T> + ReceiveNode<IdN, T> + ReceiveDirName<T> + SetRoot<IdN, O>,
    {
        type S1 = T;
        type S2 = T;
    }
}

pub struct CompoundPositionPreparer<A, B>(A, B);

mod impl_c_p_p_receivers2 {

    use super::super::file_and_offset::Position;
    use super::super::PrimInt;
    use super::bottom_up;
    use super::top_down;
    use super::CompoundPositionPreparer;
    use super::Transition;

    impl<A: top_down::CreateBuilder, B: top_down::CreateBuilder> top_down::CreateBuilder
        for CompoundPositionPreparer<A, B>
    {
        fn create() -> Self {
            Self(
                top_down::CreateBuilder::create(),
                top_down::CreateBuilder::create(),
            )
        }
    }

    // impl<IdN, A: top_down::ReceiveParent<IdN, A>, B: top_down::ReceiveParent<IdN, B>>
    //     top_down::ReceiveParent<IdN, Self> for CompoundPositionPreparer<A, B>
    // {
    //     fn push(self, parent: IdN) -> Self {
    //         Self(self.0.push(parent), self.1.push(parent))
    //     }
    // }

    impl<IdN, IdO: PrimInt, B: top_down::ReceiveParent<IdN, B>> top_down::ReceiveParent<IdN, Self>
        for CompoundPositionPreparer<Position<std::path::PathBuf, IdO>, B>
    {
        fn push(self, parent: IdN) -> Self {
            Self(self.0, self.1.push(parent))
        }
    }

    impl<A: top_down::ReceiveDirName<A>, B: top_down::ReceiveDirName<B>>
        top_down::ReceiveDirName<Self> for CompoundPositionPreparer<A, B>
    {
        fn push(self, dir_name: &str) -> Self {
            Self(self.0.push(dir_name), self.1.push(dir_name))
        }
    }

    impl<A: bottom_up::ReceiveDirName<A>, B: bottom_up::ReceiveDirName<B>>
        bottom_up::ReceiveDirName<Self> for CompoundPositionPreparer<A, B>
    {
        fn push(self, dir_name: &str) -> Self {
            Self(self.0.push(dir_name), self.1.push(dir_name))
        }
    }

    // impl<IdN, Idx: PrimInt, IdO: PrimInt, C> top_down::ReceiveIdx<Idx, Self> for CompoundPositionPreparer<IdN, Idx, IdO, C> {
    //     fn push(mut self, idx: Idx) -> Self {
    //         self.offsets.push(idx);
    //         self
    //     }
    // }

    impl<Idx: PrimInt, A: top_down::ReceiveIdx<Idx, A>, B: top_down::ReceiveIdx<Idx, B>>
        top_down::ReceiveIdx<Idx, Self> for CompoundPositionPreparer<A, B>
    {
        fn push(self, idx: Idx) -> Self {
            Self(self.0.push(idx), self.1.push(idx))
        }
    }

    // impl<IdN, Idx: PrimInt, IdO: PrimInt, C> top_down::ReceiveIdxNoSpace<Idx, Self> for CompoundPositionPreparer<IdN, Idx, IdO, C> {
    //     fn push(self, _idx: Idx) -> Self {
    //         //self.offsets.push(idx);
    //         self
    //     }
    // }

    impl<Idx: PrimInt, A: top_down::ReceiveParent<Idx, A>, B: top_down::ReceiveParent<Idx, B>>
        top_down::ReceiveIdxNoSpace<Idx, Self> for CompoundPositionPreparer<A, B>
    {
        fn push(self, idx: Idx) -> Self {
            Self(self.0.push(idx), self.1.push(idx))
        }
    }

    impl<A, B> top_down::FileSysReceiver for CompoundPositionPreparer<A, B> {
        type InFile<O> = Self;
    }

    impl<IdO: PrimInt, A: top_down::ReceiveOffset<IdO, A>, B: top_down::ReceiveOffset<IdO, B>>
        top_down::ReceiveOffset<IdO, Self> for CompoundPositionPreparer<A, B>
    {
        fn push(self, bytes: IdO) -> Self {
            Self(self.0.push(bytes), self.1.push(bytes))
        }
    }
    impl<IdO: PrimInt, A: super::SetLen<IdO, A>, B: super::SetLen<IdO, B>> super::SetLen<IdO, Self>
        for CompoundPositionPreparer<A, B>
    {
        fn set(self, len: IdO) -> Self {
            Self(self.0.set(len), self.1.set(len))
        }
    }
    // impl<IdN, A: top_down::SetNode<IdN, A>, B: top_down::SetNode<IdN, B>>
    //     top_down::SetNode<IdN, Self> for CompoundPositionPreparer<A, B>
    // {
    //     fn set_node(self, node: IdN) -> Self {
    //         Self(self.0.set(len), self.1.set(len))
    //     }
    // }
    impl<IdN, IdO: PrimInt, B: top_down::SetNode<IdN, B>> top_down::SetNode<IdN, Self>
        for CompoundPositionPreparer<
            super::super::file_and_offset::Position<std::path::PathBuf, IdO>,
            B,
        >
    {
        fn set_node(self, node: IdN) -> Self {
            Self(self.0, self.1.set_node(node))
        }
    }

    impl<A: top_down::SetFileName<A>, B: top_down::SetFileName<B>> top_down::SetFileName<Self>
        for CompoundPositionPreparer<A, B>
    {
        fn set_file_name(self, file_name: &str) -> Self {
            Self(
                self.0.set_file_name(file_name),
                self.1.set_file_name(file_name),
            )
        }
    }
    impl<A: Transition<A>, B: Transition<B>> Transition<Self> for CompoundPositionPreparer<A, B> {
        fn transit(self) -> Self {
            Self(self.0.transit(), self.1.transit())
        }
    }
    impl<A: Into<AA>, B: Into<BB>, AA, BB> Transition<(AA, BB)> for CompoundPositionPreparer<A, B> {
        fn transit(self) -> (AA, BB) {
            (self.0.into(), self.1.into())
        }
    }
}
