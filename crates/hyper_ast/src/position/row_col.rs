use core::fmt;
use std::fmt::{Debug, Display};

use crate::PrimInt;

#[derive(PartialEq, Eq, Hash, Clone, Default)]
pub struct RowCol<T: PrimInt> {
    row: T,
    col: T,
}

impl<T: PrimInt> RowCol<T> {
    pub fn new(row: T, col: T) -> Self {
        Self { row, col }
    }
    pub fn inc_row(&mut self, x: T) {
        self.row += x;
    }
    pub fn inc_col(&mut self, x: T) {
        self.col += x;
    }
    pub fn row(&self) -> T {
        self.row
    }
    pub fn col(&self) -> T {
        self.col
    }
}

impl<T: PrimInt> Debug for RowCol<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RowCol")
            .field("row", &self.row)
            .field("col", &self.col)
            .finish()
    }
}

impl<T: PrimInt + Display> Display for RowCol<T> {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

// use super::spaces_related::{SealedFileTopDownPosBuilder, TopDownPosBuilder};

// impl<IdN, Idx: PrimInt, IdO: PrimInt + Default>
//     TopDownPosBuilder<IdN, Idx, IdO, NoSpacePrepareParams<Idx>> for Position<PathBuf, IdO>
// {
//     type Prepared = Position<PathBuf, IdO>;

//     type SealedFile = Position<PathBuf, IdO>;

//     fn seal_path(mut self, file_name: &str) -> Self::SealedFile {
//         self.file.push(file_name);
//         self
//     }

//     fn seal_without_path(self) -> Self::SealedFile {
//         self
//     }

//     fn push(&mut self, _parent: IdN, _offset: Idx, dir_name: &str, _additional: ()) {
//         self.file.push(dir_name);
//     }

//     fn finish(self, _node: IdN) -> Self::Prepared {
//         todo!("how exactly should directories be handled")
//     }
// }
// impl<IdN, Idx: PrimInt, IdO: PrimInt>
//     SealedFileTopDownPosBuilder<IdN, Idx, IdO, NoSpacePrepareParams<Idx>>
//     for Position<PathBuf, IdO>
// {
//     type Prepared = Position<PathBuf, IdO>;

//     fn push(&mut self, _parent: IdN, _idx: Idx, offset: IdO, (_no_s_idx,): (Idx,)) {
//         self.offset += offset;
//     }

//     fn finish(self, _node: IdN, len: Idx, _additional: ()) -> Self::Prepared {
//         assert_eq!(self.len, num::zero());
//         let len = num::cast(len).unwrap();
//         Self::Prepared {
//             file: self.file,
//             offset: self.offset,
//             len,
//         }
//     }
// }
mod impl_receivers {
    use super::super::building;
    use crate::PrimInt;
    use building::bottom_up;
    use building::top_down;

    impl<T: PrimInt> top_down::CreateBuilder for super::RowCol<T> {
        fn create() -> Self {
            Self {
                row: num::zero(),
                col: num::zero(),
            }
        }
    }

    impl<T: PrimInt> bottom_up::CreateBuilder for super::RowCol<T> {
        fn create() -> Self {
            Self {
                row: num::zero(),
                col: num::zero(),
            }
        }
    }

    impl<IdN, T: PrimInt> top_down::ReceiveParent<IdN, Self> for super::RowCol<T> {
        fn push(self, _parent: IdN) -> Self {
            self
        }
    }

    impl<IdN, T: PrimInt> bottom_up::ReceiveNode<IdN, Self> for super::RowCol<T> {
        fn push(self, _node: IdN) -> Self {
            self
        }
    }

    impl<IdN, T: PrimInt> bottom_up::SetRoot<IdN, Self> for super::RowCol<T> {
        fn set_root(self, _root: IdN) -> Self {
            self
        }
    }

    impl<IdN, T: PrimInt> top_down::SetNode<IdN, Self> for super::RowCol<T> {
        fn set_node(self, _node: IdN) -> Self {
            self
        }
    }

    impl<T: PrimInt> top_down::ReceiveDirName<Self> for super::RowCol<T> {
        fn push(self, _dir_name: &str) -> Self {
            self
        }
    }

    impl<T: PrimInt> bottom_up::ReceiveDirName<Self> for super::RowCol<T> {
        fn push(self, _dir_name: &str) -> Self {
            self
        }
    }

    impl<T: PrimInt> top_down::SetFileName<Self> for super::RowCol<T> {
        fn set_file_name(self, _file_name: &str) -> Self {
            self
        }
    }

    impl<Idx, T: PrimInt> top_down::ReceiveIdx<Idx, Self> for super::RowCol<T> {
        fn push(self, _idx: Idx) -> Self {
            self
        }
    }

    impl<Idx, T: PrimInt> bottom_up::ReceiveIdx<Idx, Self> for super::RowCol<T> {
        fn push(self, _idx: Idx) -> Self {
            self
        }
    }

    impl<Idx, T: PrimInt> top_down::ReceiveIdxNoSpace<Idx, Self> for super::RowCol<T> {
        fn push(self, _idx: Idx) -> Self {
            self
        }
    }

    impl<T: PrimInt, IdO> top_down::ReceiveOffset<IdO, Self> for super::RowCol<T> {
        fn push(self, _offset: IdO) -> Self {
            self
        }
    }

    impl<T: PrimInt, IdO> bottom_up::ReceiveOffset<IdO, Self> for super::RowCol<T> {
        fn push(self, _offset: IdO) -> Self {
            self
        }
    }

    impl<T: PrimInt> building::ReceiveRows<T, Self> for super::RowCol<T> {
        fn push(mut self, row: T) -> Self {
            self.row += row;
            self
        }
    }

    impl<T: PrimInt> building::ReceiveColumns<T, Self> for super::RowCol<T> {
        fn push(mut self, col: T) -> Self {
            self.col += col;
            self
        }
    }

    impl<T: PrimInt, IdO> building::SetLen<IdO, Self> for super::RowCol<T> {
        fn set(self, _len: IdO) -> Self {
            self
        }
    }

    impl<T: PrimInt> building::SetLineSpan<T, Self> for super::RowCol<T> {
        fn set(self, _lines: T) -> Self {
            self
        }
    }

    impl<T: PrimInt> top_down::FileSysReceiver for super::RowCol<T> {
        type InFile<O> = Self;
    }

    impl<T: PrimInt> building::Transition<super::RowCol<T>> for super::RowCol<T> {
        fn transit(self) -> super::RowCol<T> {
            self
        }
    }

    // impl<IdN, Idx, T: PrimInt> top_down::ReceiveInFile<IdN, Idx, Self> for super::Position<PathBuf, IdO> {
    //     type S1 = Self;

    //     type S2 = Self;

    //     fn finish(self) -> Self {
    //         self
    //     }
    // }
    // impl<IdN, Idx, T: PrimInt> top_down::ReceiveDir<IdN, Idx, Self> for super::Position<PathBuf, IdO> {
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
