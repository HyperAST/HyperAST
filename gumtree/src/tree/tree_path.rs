/// Path in tree optimized for memory footprint
use std::{fmt::Debug, marker::PhantomData};

use num_traits::{cast, PrimInt, ToPrimitive};

pub trait TreePath<'a, Idx> {
    type ItemIterator: Iterator<Item = Idx>;
    fn iter(&'a self) -> Self::ItemIterator;
    fn extend(&self, path: &[Idx]) -> Self;
}

struct SimpleTreePath<Idx> {
    vec: Vec<Idx>,
}

impl<'a, Idx: 'a + PrimInt> TreePath<'a, Idx> for SimpleTreePath<Idx> {
    type ItemIterator = IterSimple<'a, Idx>;
    fn iter(&'a self) -> Self::ItemIterator {
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

impl<Idx: PartialEq> PartialEq for CompressedTreePath<Idx> {
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits
        // let mut other = other.iter();
        // for s in self.iter() {
        //     if let Some(other) = other.next() {
        //         if s != other {
        //             return false;
        //         }
        //     } else {
        //         return false;
        //     }
        // }
        // true
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

impl<'a, Idx: 'a + PrimInt> TreePath<'a, Idx> for CompressedTreePath<Idx> {
    type ItemIterator = Iter<'a, Idx>;
    fn iter(&'a self) -> Self::ItemIterator {
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
