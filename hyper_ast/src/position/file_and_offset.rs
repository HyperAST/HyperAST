use core::fmt;
use std::{
    fmt::{Debug, Display},
    path::PathBuf,
};

use super::PrimInt;

#[derive(PartialEq, Eq, Hash, Clone, Default)]
pub struct Position<F, T: PrimInt> {
    file: F,
    offset: T,
    len: T,
}

impl<F: std::ops::Deref, T: PrimInt> Position<F, T> {
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
