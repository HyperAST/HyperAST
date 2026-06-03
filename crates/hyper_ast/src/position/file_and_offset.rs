use core::fmt;
use std::{
    fmt::{Debug, Display},
    path::PathBuf,
};

use crate::PrimInt;

#[derive(PartialEq, Eq, Hash, Clone, Default)]
pub struct Position<F, T: PrimInt> {
    file: F,
    offset: T,
    len: T,
}

impl<F, T: PrimInt> Position<F, T> {
    pub fn new(file: F, offset: T, len: T) -> Self {
        Self { file, offset, len }
    }
    pub fn inc_offset(&mut self, x: T) {
        self.offset += x;
    }
    pub fn set_len(&mut self, x: T) {
        self.len = x;
    }
    pub fn range(&self) -> std::ops::Range<T> {
        self.offset..(self.offset + self.len)
    }
}

impl<F: std::ops::Deref, T: PrimInt> Position<F, T> {
    pub fn file(&self) -> &F::Target {
        self.file.deref()
    }
}

impl<T: PrimInt> Position<PathBuf, T> {
    pub fn inc_path(&mut self, s: &str) {
        self.file.push(s);
    }
}

impl<F: Debug, T: PrimInt> Debug for Position<F, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Position")
            .field("file", &self.file)
            .field("offset", &self.offset)
            .field("len", &self.len)
            .finish()
    }
}

impl<T: PrimInt + Display> Display for Position<PathBuf, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"offset\":{},\"len\":{},\"file\":{:?}}}",
            &self.offset, &self.len, &self.file
        )
    }
}

// TODO use an interface for TopDownPositionBuilder, should actually be the same as position here, this way you can see the generated pos as a DTO
// TODO in the same way finishing a prepare struct could directly be converted into a position, or be an accumulator itself (actually better for some structs)
impl<IdN, Idx, IdO: PrimInt> Into<Position<PathBuf, IdO>>
    for super::spaces_related::TopDownPositionBuilder<IdN, Idx, IdO>
{
    fn into(self) -> Position<PathBuf, IdO> {
        // TODO how to handle position of directory ?
        let range = self.range.unwrap();
        let len = range.end - range.start;
        Position {
            file: self.file,
            offset: range.start,
            len,
        }
    }
}

use super::spaces_related::{NoSpacePrepareParams, SealedFileTopDownPosBuilder, TopDownPosBuilder};

impl<IdN, Idx: PrimInt, IdO: PrimInt + Default>
    TopDownPosBuilder<IdN, Idx, IdO, NoSpacePrepareParams<Idx>> for Position<PathBuf, IdO>
{
    type Prepared = Position<PathBuf, IdO>;

    type SealedFile = Position<PathBuf, IdO>;

    fn seal_path(mut self, file_name: &str) -> Self::SealedFile {
        self.file.push(file_name);
        self
    }

    fn seal_without_path(self) -> Self::SealedFile {
        self
    }

    fn push(&mut self, _parent: IdN, _offset: Idx, dir_name: &str, _additional: ()) {
        self.file.push(dir_name);
    }

    fn finish(self, _node: IdN) -> Self::Prepared {
        todo!("how exactly should directories be handled")
    }
}
impl<IdN, Idx: PrimInt, IdO: PrimInt>
    SealedFileTopDownPosBuilder<IdN, Idx, IdO, NoSpacePrepareParams<Idx>>
    for Position<PathBuf, IdO>
{
    type Prepared = Position<PathBuf, IdO>;

    fn push(&mut self, _parent: IdN, _idx: Idx, offset: IdO, (_no_s_idx,): (Idx,)) {
        self.offset += offset;
    }

    fn finish(self, _node: IdN, len: Idx, _additional: ()) -> Self::Prepared {
        assert_eq!(self.len, num::zero());
        let len = num::cast(len).unwrap();
        Self::Prepared {
            file: self.file,
            offset: self.offset,
            len,
        }
    }
}
mod impl_receivers {
    use super::super::building;
    use crate::PrimInt;
    use building::bottom_up;
    use building::top_down;
    use std::path::PathBuf;

    impl<IdO: PrimInt> top_down::CreateBuilder for super::Position<PathBuf, IdO> {
        fn create() -> Self {
            Self {
                file: Default::default(),
                offset: num::zero(),
                len: num::zero(),
            }
        }
    }

    impl<IdO: PrimInt> bottom_up::CreateBuilder for super::Position<PathBuf, IdO> {
        fn create() -> Self {
            Self {
                file: Default::default(),
                offset: num::zero(),
                len: num::zero(),
            }
        }
    }

    impl<IdN, IdO: PrimInt> top_down::ReceiveParent<IdN, Self> for super::Position<PathBuf, IdO> {
        fn push(self, _parent: IdN) -> Self {
            self
        }
    }

    impl<IdN, IdO: PrimInt> bottom_up::ReceiveNode<IdN, Self> for super::Position<PathBuf, IdO> {
        fn push(self, _node: IdN) -> Self {
            self
        }
    }

    impl<IdN, IdO: PrimInt> bottom_up::SetRoot<IdN, Self> for super::Position<PathBuf, IdO> {
        fn set_root(self, _root: IdN) -> Self {
            self
        }
    }

    impl<IdN, IdO: PrimInt> top_down::SetNode<IdN, Self> for super::Position<PathBuf, IdO> {
        fn set_node(self, _node: IdN) -> Self {
            self
        }
    }

    impl<IdO: PrimInt> top_down::ReceiveDirName<Self> for super::Position<PathBuf, IdO> {
        fn push(mut self, dir_name: &str) -> Self {
            self.file.push(dir_name);
            self
        }
    }

    impl<IdO: PrimInt> bottom_up::ReceiveDirName<Self> for super::Position<PathBuf, IdO> {
        fn push(mut self, dir_name: &str) -> Self {
            self.file = std::path::PathBuf::from(dir_name).join(self.file);
            self
        }
    }

    impl<IdO: PrimInt> top_down::SetFileName<Self> for super::Position<PathBuf, IdO> {
        fn set_file_name(mut self, file_name: &str) -> Self {
            self.file.push(file_name);
            self
        }
    }

    impl<Idx, IdO: PrimInt> top_down::ReceiveIdx<Idx, Self> for super::Position<PathBuf, IdO> {
        fn push(self, _idx: Idx) -> Self {
            self
        }
    }

    impl<Idx, IdO: PrimInt> bottom_up::ReceiveIdx<Idx, Self> for super::Position<PathBuf, IdO> {
        fn push(self, _idx: Idx) -> Self {
            self
        }
    }

    impl<Idx, IdO: PrimInt> top_down::ReceiveIdxNoSpace<Idx, Self> for super::Position<PathBuf, IdO> {
        fn push(self, _idx: Idx) -> Self {
            self
        }
    }

    impl<IdO: PrimInt> top_down::ReceiveOffset<IdO, Self> for super::Position<PathBuf, IdO> {
        fn push(mut self, offset: IdO) -> Self {
            self.offset += offset;
            self
        }
    }

    impl<IdO: PrimInt, T> building::ReceiveRows<T, Self> for super::Position<PathBuf, IdO> {
        fn push(self, _row: T) -> Self {
            self
        }
    }

    impl<IdO: PrimInt, T> building::ReceiveColumns<T, Self> for super::Position<PathBuf, IdO> {
        fn push(self, _col: T) -> Self {
            self
        }
    }

    impl<IdO: PrimInt> bottom_up::ReceiveOffset<IdO, Self> for super::Position<PathBuf, IdO> {
        fn push(mut self, offset: IdO) -> Self {
            self.offset += offset;
            self
        }
    }

    impl<IdO: PrimInt> building::SetLen<IdO, Self> for super::Position<PathBuf, IdO> {
        fn set(mut self, len: IdO) -> Self {
            self.len = len;
            self
        }
    }

    impl<IdO: PrimInt, T> building::SetLineSpan<T, Self> for super::Position<PathBuf, IdO> {
        fn set(self, _row: T) -> Self {
            self
        }
    }

    impl<IdO: PrimInt> top_down::FileSysReceiver for super::Position<PathBuf, IdO> {
        type InFile<O> = Self;
    }

    impl<IdO: PrimInt> building::Transition<super::Position<PathBuf, IdO>>
        for super::Position<PathBuf, IdO>
    {
        fn transit(self) -> super::Position<PathBuf, IdO> {
            self
        }
    }

    // impl<IdN, Idx, IdO: PrimInt> top_down::ReceiveInFile<IdN, Idx, Self> for super::Position<PathBuf, IdO> {
    //     type S1 = Self;

    //     type S2 = Self;

    //     fn finish(self) -> Self {
    //         self
    //     }
    // }
    // impl<IdN, Idx, IdO: PrimInt> top_down::ReceiveDir<IdN, Idx, Self> for super::Position<PathBuf, IdO> {
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
