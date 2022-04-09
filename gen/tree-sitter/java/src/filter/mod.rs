pub mod pearson_hashing;

use std::{hash::Hash, io::Result, marker::PhantomData, ops::Deref};

use bitvec::{order::Lsb0, store::BitStore, view::BitViewSized};

use crate::filter::pearson_hashing::pearson;

use self::pearson_hashing::{pearson_mod, xor_rot_mod};

#[derive(PartialEq, Eq)]
pub enum BloomSize {
    None,
    B16,
    B32,
    B64,
    B128,
    B256,
    B512,
    B1024,
    B2048,
    Much,
}

pub trait BF<T: ?Sized> {
    type Result;
    const SIZE: BloomSize;
    fn insert<U: AsRef<[u8]>>(&mut self, dups: usize, item: U) {
        panic!()
    }
    fn check<U: AsRef<[u8]>>(&self, dups: usize, item: U) -> Self::Result {
        panic!()
    }
}

/// (2^3)^S = 2^(3*S) bits = S bytes
pub struct Bloom<T, V: BitViewSized> {
    bits: bitvec::array::BitArray<Lsb0, V>,
    _phantom: PhantomData<*const T>,
}

unsafe impl<T, V: BitViewSized> Send for Bloom<T, V> {}
unsafe impl<T, V: BitViewSized> Sync for Bloom<T, V> {}

impl<T, V: BitViewSized> Default for Bloom<T, V> {
    fn default() -> Self {
        Self {
            bits: Default::default(),
            _phantom: Default::default(),
        }
    }
}

// impl<'a, T: ?Sized, V: BitViewSized, It: Iterator<Item = &'a Box<T>>> From<It>
//     for Bloom<&'static T, V>
// where
//     Self: BF<T>,
// {
//     fn from(it: It) -> Self {
//         let mut r = Self::default();
//         for x in it {
//             Bloom::insert(&mut r, 0, &x);
//         }
//         r
//     }
// }

// trait IntoBytes {
//     fn as_ref(self) -> &[u8];
// }

impl<T: ?Sized, U, V: BitViewSized, It: Iterator<Item = U>> From<It>
    for Bloom<&'static T, V>
where
    Self: BF<T>,
    U: AsRef<[u8]>
{
    fn from(it: It) -> Self {
        let mut r = Self::default();
        for x in it {
            Bloom::insert(&mut r, 0, x.as_ref());
        }
        r
    }
}

fn f() {
    let x = vec!["".as_bytes().to_owned().into_boxed_slice()];
    let a = x.iter().map(|x|x.deref());
    let b: Bloom<&'static [u8], [u64; 4]> = From::from(a);
}

pub enum BloomResult {
    MaybeContain,
    DoNotContain,
}

// impl<T:?Sized,V:BitViewSized> BF<T> for Bloom<&'static T, V> {
//     type Result = BloomResult;

//     fn insert<U: AsRef<T>>(&mut self, dups: usize, item: U) {
//         panic!()
//     }

//     fn check<U: AsRef<T>>(&self, dups: usize, item: U) -> Self::Result {
//         panic!()
//     }
// }

impl BF<[u8]> for Bloom<&'static [u8], u16> {
    type Result = BloomResult;
    const SIZE:BloomSize = BloomSize::B16;

    fn insert<U: AsRef<[u8]>>(&mut self, dups: usize, item: U) {
        for i in 0..=dups {
            let a = pearson(i, item.as_ref());
            let b = (a & 0xf) ^ (a >> 4);
            self.bits.set(b as usize, true);
        }
    }

    fn check<U: AsRef<[u8]>>(&self, dups: usize, item: U) -> Self::Result {
        log::trace!("{}", self.bits);
        for i in 0..=dups {
            let a = pearson(i, item.as_ref());
            let b = (a & 0xf) ^ (a >> 4);
            if !self.bits[b as usize] {
                return BloomResult::DoNotContain;
            }
        }
        BloomResult::MaybeContain
    }
}

impl BF<[u8]> for Bloom<&'static [u8], u32> {
    type Result = BloomResult;
    const SIZE:BloomSize = BloomSize::B32;

    fn insert<U: AsRef<[u8]>>(&mut self, dups: usize, item: U) {
        for i in 0..=dups {
            let a = pearson_mod::<_, 32>(i, item.as_ref());
            let b = a;
            self.bits.set(b as usize, true);
        }
    }

    fn check<U: AsRef<[u8]>>(&self, dups: usize, item: U) -> Self::Result {
        log::trace!("{}", self.bits);
        for i in 0..=dups {
            let a = pearson_mod::<_, 32>(i, item.as_ref());
            let b = a;
            if !self.bits[b as usize] {
                return BloomResult::DoNotContain;
            }
        }
        BloomResult::MaybeContain
    }
}

impl BF<[u8]> for Bloom<&'static [u8], u64> {
    type Result = BloomResult;
    const SIZE:BloomSize = BloomSize::B64;

    fn insert<U: AsRef<[u8]>>(&mut self, dups: usize, item: U) {
        for i in 0..=dups {
            let a = pearson_mod::<_, 64>(i, item.as_ref());
            let b = a;
            self.bits.set(b as usize, true);
        }
    }

    fn check<U: AsRef<[u8]>>(&self, dups: usize, item: U) -> Self::Result {
        log::trace!("{}", self.bits);
        for i in 0..=dups {
            let a = pearson_mod::<_, 64>(i, item.as_ref());
            let b = a;
            if !self.bits[b as usize] {
                return BloomResult::DoNotContain;
            }
        }
        BloomResult::MaybeContain
    }
}

impl BF<[u8]> for Bloom<&'static [u8], [u64; 2]> {
    type Result = BloomResult;
    const SIZE:BloomSize = BloomSize::B128;

    fn insert<U: AsRef<[u8]>>(&mut self, dups: usize, item: U) {
        for i in 0..=dups {
            let a = pearson_mod::<_, 128>(i, item.as_ref());
            let b = a;
            self.bits.set(b as usize, true);
        }
    }

    fn check<U: AsRef<[u8]>>(&self, dups: usize, item: U) -> Self::Result {
        log::trace!("{}", self.bits);
        for i in 0..=dups {
            let a = pearson_mod::<_, 128>(i, item.as_ref());
            let b = a;
            if !self.bits[b as usize] {
                return BloomResult::DoNotContain;
            }
        }
        BloomResult::MaybeContain
    }
}

impl BF<[u8]> for Bloom<&'static [u8], [u64; 4]> {
    type Result = BloomResult;
    const SIZE:BloomSize = BloomSize::B256;

    fn insert<U: AsRef<[u8]>>(&mut self, dups: usize, item: U) {
        for i in 0..=dups {
            let a = pearson(i, item.as_ref());
            let b = a;
            self.bits.set(b as usize, true);
        }
    }

    fn check<U: AsRef<[u8]>>(&self, dups: usize, item: U) -> Self::Result {
        log::trace!("{}", self.bits);
        for i in 0..=dups {
            let a = pearson(i, item.as_ref());
            let b = a;
            if !self.bits[b as usize] {
                return BloomResult::DoNotContain;
            }
        }
        BloomResult::MaybeContain
    }
}

//TODO
impl BF<[u8]> for Bloom<&'static [u8], [u64; 8]> {
    type Result = BloomResult;
    const SIZE:BloomSize = BloomSize::B512;

    fn insert<U: AsRef<[u8]>>(&mut self, dups: usize, item: U) {
        for i in 0..=dups {
            let a = xor_rot_mod::<_,512>(i, item.as_ref());
            let b = a;
            self.bits.set(b as usize, true);
        }
    }

    fn check<U: AsRef<[u8]>>(&self, dups: usize, item: U) -> Self::Result {
        log::trace!("{}", self.bits);
        for i in 0..=dups {
            let a = xor_rot_mod::<_,512>(i, item.as_ref());
            let b = a;
            if !self.bits[b as usize] {
                return BloomResult::DoNotContain;
            }
        }
        BloomResult::MaybeContain
    }
}
impl BF<[u8]> for Bloom<&'static [u8], [u64; 16]> {
    type Result = BloomResult;
    const SIZE:BloomSize = BloomSize::B1024;

    fn insert<U: AsRef<[u8]>>(&mut self, dups: usize, item: U) {
        for i in 0..=dups {
            let a = xor_rot_mod::<_,1024>(i, item.as_ref());
            let b = a;
            self.bits.set(b as usize, true);
        }
    }

    fn check<U: AsRef<[u8]>>(&self, dups: usize, item: U) -> Self::Result {
        log::trace!("{}", self.bits);
        for i in 0..=dups {
            let a = xor_rot_mod::<_,1024>(i, item.as_ref());
            let b = a;
            if !self.bits[b as usize] {
                return BloomResult::DoNotContain;
            }
        }
        BloomResult::MaybeContain
    }
}

impl BF<[u8]> for Bloom<&'static [u8], [u64; 32]> {
    type Result = BloomResult;
    const SIZE:BloomSize = BloomSize::B2048;

    fn insert<U: AsRef<[u8]>>(&mut self, dups: usize, item: U) {
        for i in 0..=dups {
            let a = xor_rot_mod::<_,2048>(i, item.as_ref());
            let b = a;
            self.bits.set(b as usize, true);
        }
    }

    fn check<U: AsRef<[u8]>>(&self, dups: usize, item: U) -> Self::Result {
        log::trace!("{}", self.bits);
        for i in 0..=dups {
            let a = xor_rot_mod::<_,2048>(i, item.as_ref());
            let b = a;
            if !self.bits[b as usize] {
                return BloomResult::DoNotContain;
            }
        }
        BloomResult::MaybeContain
    }
}

// impl BF<String> for Bloom<String, u16> {
//     type Result = BloomResult;
//     const Size:BloomSize = BloomSize::B16;

//     fn insert<U: Borrow<String>>(&mut self, dups: usize, item: U) {
//         for i in 0..=dups {
//             let a = pearson(i, item.as_ref().as_bytes());
//             let b = (a & 0xf) ^ (a >> 4);
//             self.bits.set(b as usize, true);
//         }
//     }

//     fn check<U: AsRef<[u8]>>(&self, dups: usize, item: U) -> Self::Result {
//         for i in 0..=dups {
//             let a = pearson(i, item.as_ref().as_bytes());
//             let b = (a & 0xf) ^ (a >> 4);
//             if self.bits[b as usize] {
//                 return BloomResult::DoNotContain;
//             }
//         }
//         BloomResult::MaybeContain
//     }
// }
