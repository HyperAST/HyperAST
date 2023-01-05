//! Path in tree optimized for memory footprint
//! - [x] ref iter
//! - [x] sliced box iter
//! - [x] allow to go up (at the start, it is a path in a tree anyway)
//! - [x] indexed box iter
//! - [ ] low compute cost extend
use std::{fmt::Debug, marker::PhantomData};

use num_traits::{cast, PrimInt, ToPrimitive};

pub trait TreePath<Idx> {
    // TODO move lifetime to associated type
    type ItemIterator<'a>: Iterator<Item = Idx>
    where
        Self: 'a;
    fn iter(&self) -> Self::ItemIterator<'_>;
    fn extend(&self, path: &[Idx]) -> Self;
}

pub trait TreePathUp<Idx> {
    type TP: TreePath<Idx>;
    fn up(&self) -> &Idx;
    fn path(&self) -> &Self::TP;
    fn create(&self, up: Idx, path: Self::TP) -> Self;
}

#[derive(Clone)]
struct CompressedTreePathUp<Idx> {
    up: Idx,
    compressed: CompressedTreePath<Idx>,
}

impl<Idx: PrimInt> TreePathUp<Idx> for CompressedTreePathUp<Idx> {
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

struct SimpleTreePath<Idx> {
    vec: Vec<Idx>,
}

impl<Idx: PrimInt> TreePath<Idx> for SimpleTreePath<Idx> {
    type ItemIterator<'a> = IterSimple<'a, Idx> where Idx: 'a;
    fn iter(&self) -> Self::ItemIterator<'_> {
        IterSimple {
            internal: self.vec.iter(),
        }
    }

    fn extend(&self, path: &[Idx]) -> Self {
        let mut vec = vec![];
        vec.extend(&self.vec);
        vec.extend_from_slice(path);
        Self { vec }
    }
}

#[derive(Clone)]
pub struct CompressedTreePath<Idx> {
    bits: Box<[u8]>,
    phantom: PhantomData<*const Idx>,
}

impl<Idx: PartialEq> CompressedTreePath<Idx> {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bits
    }
}

impl<Idx: PartialEq> PartialEq for CompressedTreePath<Idx> {
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits
    }
}
impl<Idx: Eq> Eq for CompressedTreePath<Idx> {}

pub enum SharedPath<P> {
    Exact(P),
    Remain(P),
    Submatch(P),
    Different(P),
}

impl<Idx: PrimInt> CompressedTreePath<Idx> {
    pub fn shared_ancestors(&self, other: &Self) -> SharedPath<Vec<Idx>> {
        let mut other = other.iter();
        let mut r = vec![];
        for s in self.iter() {
            if let Some(other) = other.next() {
                if s != other {
                    return SharedPath::Different(r);
                }
                r.push(s);
            } else {
                return SharedPath::Submatch(r);
            }
        }
        if other.next().is_some() {
            SharedPath::Remain(r)
        } else {
            SharedPath::Exact(r)
        }
    }
}

impl<Idx: PrimInt> Debug for CompressedTreePath<Idx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.iter()
                .map(|x| cast::<_, usize>(x).unwrap().to_string())
                .fold(String::new(), |a, b| if a.len() == 0 { a } else { a + "." }
                    + &b)
        )
    }
}

impl<Idx: PrimInt> CompressedTreePath<Idx> {
    fn iter(&self) -> impl Iterator<Item = Idx> + '_ {
        Iter {
            is_even: (self.bits[0] & 1) == 1,
            side: true,
            slice: &self.bits,
            phantom: PhantomData,
        }
    }
}

impl<Idx: PrimInt> IntoIterator for CompressedTreePath<Idx> {
    type Item = Idx;

    type IntoIter = IntoIter<Idx>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.bits)
    }
}

impl<Idx: PrimInt> TreePath<Idx> for CompressedTreePath<Idx> {
    type ItemIterator<'a> = Iter<'a, Idx> where Idx: 'a;
    fn iter(&self) -> Self::ItemIterator<'_> {
        Iter {
            is_even: (self.bits[0] & 1) == 1,
            side: true,
            slice: &self.bits,
            phantom: PhantomData,
        }
    }

    fn extend(&self, path: &[Idx]) -> Self {
        // todo maybe try dev something more efficient (should be easy if useful)
        let mut vec = vec![];
        vec.extend(self.iter());
        vec.extend_from_slice(path);
        Self::from(vec)
    }
}

impl<Idx: ToPrimitive> From<&[Idx]> for CompressedTreePath<Idx> {
    fn from(x: &[Idx]) -> Self {
        let mut bits = vec![];
        let mut i = 0;
        bits.push(0);
        let mut side = true;
        for x in x {
            let mut a: usize = x.to_usize().unwrap();
            loop {
                let mut br = false;
                let b = if a >= 128 {
                    a = a - 128;
                    16 - 1 as u8
                } else if a >= 32 {
                    a = a - 32;
                    16 - 2 as u8
                } else if a >= 16 - 3 {
                    a = a - (16 - 3);
                    16 - 3 as u8
                } else {
                    br = true;
                    a as u8
                };

                if side {
                    bits[i] |= b << 4;
                } else {
                    i += 1;
                    bits.push(b);
                }
                side = !side;
                if br {
                    break;
                }
            }
        }
        if side {
            bits[0] |= 1
        }
        Self {
            bits: bits.into_boxed_slice(),
            phantom: PhantomData,
        }
    }
}
impl<Idx: PrimInt> From<Vec<Idx>> for CompressedTreePath<Idx> {
    fn from(x: Vec<Idx>) -> Self {
        x.as_slice().into()
    }
}

/// dumb wrapper to avoid problems with iterators typing
struct IterSimple<'a, Idx: 'a> {
    internal: core::slice::Iter<'a, Idx>,
}

impl<'a, Idx: 'a + Copy> Iterator for IterSimple<'a, Idx> {
    type Item = Idx;

    fn next(&mut self) -> Option<Self::Item> {
        self.internal.next().and_then(|x| Some(*x))
    }
}

/// advanced iterator used to get back path as Idx from compressed path
#[derive(Clone)]
pub struct Iter<'a, Idx> {
    is_even: bool,
    side: bool,
    slice: &'a [u8],
    phantom: PhantomData<*const Idx>,
}

impl<'a, Idx: 'a + PrimInt> Iterator for Iter<'a, Idx> {
    type Item = Idx;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            return None;
        }
        if self.is_even && self.slice.len() == 1 && self.side {
            self.slice = &self.slice[1..];
            return None;
        }
        let mut c = num_traits::zero();
        loop {
            let a = &self.slice[0];
            let a = if self.side {
                (a & 0b11110000) >> 4
            } else {
                a & 0b00001111
            };
            let mut br = false;
            let b = if a == 16 - 1 {
                128
            } else if a == 16 - 2 {
                32
            } else if a == 16 - 3 {
                16 - 3
            } else {
                br = true;
                a
            };
            c = c + cast(b).unwrap();
            if self.side {
                self.slice = &self.slice[1..];
            }
            self.side = !self.side;
            if br {
                break;
            }
            if self.is_even && self.slice.len() == 1 && self.side {
                self.slice = &self.slice[1..];
                return Some(c);
            }
        }
        Some(c)
    }
}
pub use indexed::IntoIter;

pub mod slicing {
    use super::*;
    /// advanced iterator used to get back path as Idx from compressed path
    #[derive(Clone)]
    pub struct IntoIter<Idx> {
        is_even: bool,
        side: bool,
        slice: Box<[u8]>,
        phantom: PhantomData<*const Idx>,
    }

    impl<Idx: PrimInt> IntoIter<Idx> {
        pub fn new(bits: Box<[u8]>) -> Self {
            Self {
                is_even: (bits[0] & 1) == 1,
                side: true,
                slice: bits,
                phantom: PhantomData,
            }
        }
    }

    impl<Idx: PrimInt> Iterator for IntoIter<Idx> {
        type Item = Idx;

        fn next(&mut self) -> Option<Self::Item> {
            if self.slice.is_empty() {
                return None;
            }
            if self.is_even && self.slice.len() == 1 && self.side {
                self.slice = self.slice[1..].into();
                return None;
            }
            let mut c = num_traits::zero();
            loop {
                let a = &self.slice[0];
                let a = if self.side {
                    (a & 0b11110000) >> 4
                } else {
                    a & 0b00001111
                };
                let mut br = false;
                let b = if a == 16 - 1 {
                    128
                } else if a == 16 - 2 {
                    32
                } else if a == 16 - 3 {
                    16 - 3
                } else {
                    br = true;
                    a
                };
                c = c + cast(b).unwrap();
                if self.side {
                    self.slice = self.slice[1..].into();
                }
                self.side = !self.side;
                if br {
                    break;
                }
                if self.is_even && self.slice.len() == 1 && self.side {
                    self.slice = self.slice[1..].into();
                    return Some(c);
                }
            }
            Some(c)
        }
    }
}

pub mod indexed {
    use super::*;

    /// advanced iterator used to get back path as Idx from compressed path
    #[derive(Clone)]
    pub struct IntoIter<Idx> {
        is_even: bool,
        side: bool,
        slice: Box<[u8]>,
        adv: usize,
        phantom: PhantomData<*const Idx>,
    }

    impl<Idx: PrimInt> IntoIter<Idx> {
        pub fn new(bits: Box<[u8]>) -> Self {
            Self {
                is_even: (bits[0] & 1) == 1,
                side: true,
                slice: bits,
                adv: 0,
                phantom: PhantomData,
            }
        }
    }

    impl<Idx: PrimInt> Iterator for IntoIter<Idx> {
        type Item = Idx;

        fn next(&mut self) -> Option<Self::Item> {
            if self.adv >= self.slice.len() {
                return None;
            }
            if self.is_even && self.slice.len() - self.adv == 1 && self.side {
                self.adv += 1;
                return None;
            }
            let mut c = num_traits::zero();
            loop {
                let a = &self.slice[self.adv];
                let a = if self.side {
                    (a & 0b11110000) >> 4
                } else {
                    a & 0b00001111
                };
                let mut br = false;
                let b = if a == 16 - 1 {
                    128
                } else if a == 16 - 2 {
                    32
                } else if a == 16 - 3 {
                    16 - 3
                } else {
                    br = true;
                    a
                };
                c = c + cast(b).unwrap();
                if self.side {
                    self.adv += 1;
                }
                self.side = !self.side;
                if br {
                    break;
                }
                if self.is_even && self.slice.len() - self.adv == 1 && self.side {
                    self.adv += 1;
                    return Some(c);
                }
            }
            Some(c)
        }
    }

    #[test]
    fn identity() {
        let v = vec![
            1, 4684, 68, 46, 84, 684, 68, 46, 846, 4460, 0, 00, 8, 0, 8, 0, 0, 0, 1, 12, 1, 2, 1,
            21, 2, 1, 2, 12, 1,
        ];
        let path = CompressedTreePath::<u16>::from(v.clone());
        let bits = path.as_bytes();
        let res: Vec<_> = IntoIter::<u16>::new(bits.into()).collect();
        assert_eq!(v, res);
    }
}

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
