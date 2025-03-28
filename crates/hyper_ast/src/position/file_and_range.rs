use crate::PrimInt;
use core::fmt;
use std::fmt::{Debug, Display};
use std::path::PathBuf;

#[derive(PartialEq, Eq, Hash, Clone, Default)]
pub struct Position<F, T: PrimInt> {
    file: F,
    start: T,
    len: T,
}

impl<F, T: PrimInt> Position<F, T> {
    pub fn new(file: F, start: T, len: T) -> Self {
        Self { file, start, len }
    }
    pub fn range(&self) -> std::ops::Range<T> {
        self.start..(self.start + self.len)
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
            .field("start", &self.start)
            .field("len", &self.len)
            .finish()
    }
}

impl<T: PrimInt + Display> Display for Position<PathBuf, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"start\":{},\"len\":{},\"file\":{:?}}}",
            &self.start, &self.len, &self.file
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
            start: range.start,
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

    impl<T: PrimInt> top_down::CreateBuilder for super::Position<PathBuf, T> {
        fn create() -> Self {
            Self {
                file: Default::default(),
                start: num::zero(),
                len: num::zero(),
            }
        }
    }

    impl<T: PrimInt> bottom_up::CreateBuilder for super::Position<PathBuf, T> {
        fn create() -> Self {
            Self {
                file: Default::default(),
                start: num::zero(),
                len: num::zero(),
            }
        }
    }

    impl<IdN, T: PrimInt> top_down::ReceiveParent<IdN, Self> for super::Position<PathBuf, T> {
        fn push(self, _parent: IdN) -> Self {
            self
        }
    }

    impl<IdN, T: PrimInt> bottom_up::ReceiveNode<IdN, Self> for super::Position<PathBuf, T> {
        fn push(self, _node: IdN) -> Self {
            self
        }
    }

    impl<IdN, T: PrimInt> bottom_up::SetRoot<IdN, Self> for super::Position<PathBuf, T> {
        fn set_root(self, _root: IdN) -> Self {
            self
        }
    }

    impl<IdN, T: PrimInt> top_down::SetNode<IdN, Self> for super::Position<PathBuf, T> {
        fn set_node(self, _node: IdN) -> Self {
            self
        }
    }

    impl<T: PrimInt> top_down::ReceiveDirName<Self> for super::Position<PathBuf, T> {
        fn push(mut self, dir_name: &str) -> Self {
            self.file.push(dir_name);
            self
        }
    }

    impl<T: PrimInt> bottom_up::ReceiveDirName<Self> for super::Position<PathBuf, T> {
        fn push(mut self, dir_name: &str) -> Self {
            self.file = std::path::PathBuf::from(dir_name).join(self.file);
            self
        }
    }

    impl<T: PrimInt> top_down::SetFileName<Self> for super::Position<PathBuf, T> {
        fn set_file_name(mut self, file_name: &str) -> Self {
            self.file.push(file_name);
            self
        }
    }

    impl<Idx, T: PrimInt> top_down::ReceiveIdx<Idx, Self> for super::Position<PathBuf, T> {
        fn push(self, _idx: Idx) -> Self {
            self
        }
    }

    impl<Idx, T: PrimInt> bottom_up::ReceiveIdx<Idx, Self> for super::Position<PathBuf, T> {
        fn push(self, _idx: Idx) -> Self {
            self
        }
    }

    impl<Idx, T: PrimInt> top_down::ReceiveIdxNoSpace<Idx, Self> for super::Position<PathBuf, T> {
        fn push(self, _idx: Idx) -> Self {
            self
        }
    }

    impl<IdO, T: PrimInt> top_down::ReceiveOffset<IdO, Self> for super::Position<PathBuf, T> {
        fn push(self, _offset: IdO) -> Self {
            self
        }
    }

    impl<T: PrimInt> building::ReceiveRows<T, Self> for super::Position<PathBuf, T> {
        fn push(mut self, row: T) -> Self {
            self.start += row;
            self
        }
    }

    impl<T: PrimInt> building::ReceiveColumns<T, Self> for super::Position<PathBuf, T> {
        fn push(self, _col: T) -> Self {
            self
        }
    }

    impl<IdO, T: PrimInt> bottom_up::ReceiveOffset<IdO, Self> for super::Position<PathBuf, T> {
        fn push(self, _offset: IdO) -> Self {
            self
        }
    }

    impl<IdO, T: PrimInt> building::SetLen<IdO, Self> for super::Position<PathBuf, T> {
        fn set(self, _len: IdO) -> Self {
            self
        }
    }

    impl<T: PrimInt> building::SetLineSpan<T, Self> for super::Position<PathBuf, T> {
        fn set(mut self, lines: T) -> Self {
            self.len = lines;
            self
        }
    }

    impl<T: PrimInt> top_down::FileSysReceiver for super::Position<PathBuf, T> {
        type InFile<O> = Self;
    }

    impl<T: PrimInt> building::Transition<super::Position<PathBuf, T>> for super::Position<PathBuf, T> {
        fn transit(self) -> super::Position<PathBuf, T> {
            self
        }
    }

    // impl<IdN, Idx, IdO: PrimInt> top_down::ReceiveInFile<IdN, Idx, Self> for super::Position<PathBuf, T> {
    //     type S1 = Self;

    //     type S2 = Self;

    //     fn finish(self) -> Self {
    //         self
    //     }
    // }
    // impl<IdN, Idx, IdO: PrimInt> top_down::ReceiveDir<IdN, Idx, Self> for super::Position<PathBuf, T> {
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
