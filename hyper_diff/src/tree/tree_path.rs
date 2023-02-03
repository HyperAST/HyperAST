//! Path in tree optimized for memory footprint
//! - [x] ref iter
//! - [x] sliced box iter
//! - [x] allow to go up (at the start, it is a path in a tree anyway)
//! - [x] indexed box iter
//! - [ ] low compute cost extend
use std::{fmt::Debug, marker::PhantomData};

use num_traits::{cast, PrimInt, ToPrimitive};

pub trait TreePath: IntoIterator {
    type ItemIterator<'a>: Iterator<Item = Self::Item>
    where
        Self: 'a;
    fn iter(&self) -> Self::ItemIterator<'_>;
    fn extend(self, path: &[Self::Item]) -> Self;
}

pub trait TreePathUp {
    type TP: TreePath;
    fn up(&self) -> &<Self::TP as IntoIterator>::Item;
    fn path(&self) -> &Self::TP;
    fn create(&self, up: <Self::TP as IntoIterator>::Item, path: Self::TP) -> Self;
}

#[derive(Clone)]
struct CompressedTreePathUp<Idx> {
    up: Idx,
    compressed: CompressedTreePath<Idx>,
}

impl<Idx: PrimInt> TreePathUp for CompressedTreePathUp<Idx> {
    type TP = CompressedTreePath<Idx>;

    fn up(&self) -> &Idx {
        &self.up
    }

    fn path(&self) -> &Self::TP {
        &self.compressed
    }

    fn create(&self, up: Idx, path: Self::TP) -> Self {
        Self {
            up,
            compressed: path,
        }
    }
}

impl<Idx: PartialEq> PartialEq for CompressedTreePathUp<Idx> {
    fn eq(&self, other: &Self) -> bool {
        self.up == other.up && self.compressed == other.compressed
    }
}

impl<Idx: Eq> Eq for CompressedTreePathUp<Idx> {}

pub mod simple;
pub use crate::tree::tree_path::simple::*;

mod compressed;
pub use crate::tree::tree_path::compressed::*;
pub use indexed::IntoIter;

pub mod slicing;

pub mod indexed;

#[cfg(test)]
mod small_vec_stuff_for_compressed_path {
    use std::mem::size_of;

    #[test]
    fn vec() {
        dbg!(size_of::<[u8; 4]>());
        dbg!(size_of::<Vec<u8>>());
        enum Small<T, const N: usize> {
            Vec(Vec<T>),
            Arr(u8, [T; N]),
        }
        dbg!(size_of::<Small<u8, 1>>());
        dbg!(size_of::<Small<u8, 2>>());
        dbg!(size_of::<Small<u8, 3>>());
        dbg!(size_of::<Small<u8, 4>>());
        dbg!(size_of::<Small<u8, 10>>());
        dbg!(size_of::<Small<u8, 15>>());
        dbg!(size_of::<Small<u8, 16>>());
        dbg!(size_of::<Small<u8, 17>>());
        dbg!(size_of::<Small<u8, 18>>());
        dbg!(size_of::<Small<u8, 20>>());
    }

    #[test]
    fn boxed() {
        use std::mem::size_of;
        dbg!(size_of::<[u8; 4]>());
        dbg!(size_of::<Box<[u8]>>());
        dbg!(size_of::<Vec<u8>>());
        enum Small<T, const N: usize> {
            Box(Box<[T]>),
            Arr(u8, [T; N]),
        }
        dbg!(size_of::<Small<u8, 1>>());
        dbg!(size_of::<Small<u8, 2>>());
        dbg!(size_of::<Small<u8, 3>>());
        dbg!(size_of::<Small<u8, 4>>());
        dbg!(size_of::<Small<u8, 5>>());
        dbg!(size_of::<Small<u8, 6>>());
        dbg!(size_of::<Small<u8, 7>>());
        dbg!(size_of::<Small<u8, 8>>());
        dbg!(size_of::<Small<u8, 9>>());
        dbg!(size_of::<Small<u8, 10>>());
        dbg!(size_of::<Small<u8, 15>>());
        dbg!(size_of::<Small<u8, 16>>());
        dbg!(size_of::<Small<u8, 17>>());
        dbg!(size_of::<Small<u8, 18>>());
        dbg!(size_of::<Small<u8, 20>>());
    }
}
