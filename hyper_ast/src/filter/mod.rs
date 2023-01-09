pub mod default;
pub mod pearson_hashing;

use std::fmt::Debug;
use std::marker::PhantomData;

use bitvec::{order::Lsb0, view::BitViewSized};

use self::default::{MyDefaultHasher, Pearson, VaryHasher};

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
    B4096,
    Much,
}

pub trait BF<T: ?Sized> {
    type Result;
    type S;
    type H: VaryHasher<Self::S>;
    const SIZE: BloomSize;
    fn bulk_insert<It: Iterator<Item = Self::S>>(&mut self, it: It);
    fn check_raw(&self, item: Self::S) -> Self::Result;
}

/// (2^3)^S = 2^(3*S) bits = S bytes
pub struct Bloom<T, V: BitViewSized> {
    bits: bitvec::array::BitArray<V, Lsb0>,
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
impl<T, V: BitViewSized> Debug for Bloom<T, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bloom").field("bits", &self.bits).finish()
    }
}

impl<T: ?Sized, V: BitViewSized, It: Iterator<Item = <Self as BF<T>>::S>> From<It>
    for Bloom<&'static T, V>
where
    Self: BF<T>,
{
    fn from(it: It) -> Self {
        let mut r = Self::default();
        // for x in it {
        //     Bloom::insert(&mut r, 0, x.as_ref());
        // }
        r.bulk_insert(it);
        r
    }
}

#[derive(PartialEq, Eq)]
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
    type S = u8;
    type H = Pearson<16>;
    const SIZE: BloomSize = BloomSize::B16;

    fn bulk_insert<It: Iterator<Item = Self::S>>(&mut self, it: It) {
        it.for_each(|b| self.bits.set(b as usize, true));
    }

    // fn insert<U: AsRef<[u8]>>(&mut self, dups: usize, item: U) {
    //     todo!()
    //     // for i in 0..=dups {
    //     //     let a = pearson(i, item.as_ref());
    //     //     let b = (a & 0xf) ^ (a >> 4);
    //     //     self.bits.set(b as usize, true);
    //     // }
    // }

    // fn check<U: AsRef<[u8]>>(&self, dups: usize, item: U) -> Self::Result {
    //     todo!()
    //     // log::trace!("{}", self.bits);
    //     // for i in 0..=dups {
    //     //     let a = pearson(i, item.as_ref());
    //     //     let b = (a & 0xf) ^ (a >> 4);
    //     //     if !self.bits[b as usize] {
    //     //         return BloomResult::DoNotContain;
    //     //     }
    //     // }
    //     // BloomResult::MaybeContain
    // }

    fn check_raw(&self, b: Self::S) -> Self::Result {
        log::trace!("{}", self.bits);
        if self.bits[b as usize] {
            BloomResult::MaybeContain
        } else {
            BloomResult::DoNotContain
        }
    }
}

impl BF<[u8]> for Bloom<&'static [u8], u32> {
    type Result = BloomResult;
    type S = u8;
    type H = Pearson<32>;
    const SIZE: BloomSize = BloomSize::B32;

    fn bulk_insert<It: Iterator<Item = Self::S>>(&mut self, it: It) {
        it.for_each(|b| self.bits.set(b as usize, true));
    }

    fn check_raw(&self, b: Self::S) -> Self::Result {
        log::trace!("{}", self.bits);
        if self.bits[b as usize] {
            BloomResult::MaybeContain
        } else {
            BloomResult::DoNotContain
        }
    }
}

impl BF<[u8]> for Bloom<&'static [u8], u64> {
    type Result = BloomResult;
    type S = u8;
    type H = Pearson<64>;
    const SIZE: BloomSize = BloomSize::B64;

    fn bulk_insert<It: Iterator<Item = Self::S>>(&mut self, it: It) {
        it.for_each(|b| self.bits.set(b as usize, true));
    }

    fn check_raw(&self, b: Self::S) -> Self::Result {
        log::trace!("{}", self.bits);
        if self.bits[b as usize] {
            BloomResult::MaybeContain
        } else {
            BloomResult::DoNotContain
        }
    }
}

impl BF<[u8]> for Bloom<&'static [u8], [u64; 2]> {
    type Result = BloomResult;
    type S = u8;
    type H = Pearson<128>;
    const SIZE: BloomSize = BloomSize::B128;

    fn bulk_insert<It: Iterator<Item = Self::S>>(&mut self, it: It) {
        it.for_each(|b| self.bits.set(b as usize, true));
    }

    fn check_raw(&self, b: Self::S) -> Self::Result {
        log::trace!("{}", self.bits);
        if self.bits[b as usize] {
            BloomResult::MaybeContain
        } else {
            BloomResult::DoNotContain
        }
    }
}

impl BF<[u8]> for Bloom<&'static [u8], [u64; 4]> {
    type Result = BloomResult;
    type S = u8;
    type H = Pearson<256>;
    const SIZE: BloomSize = BloomSize::B256;

    fn bulk_insert<It: Iterator<Item = Self::S>>(&mut self, it: It) {
        it.for_each(|b| self.bits.set(b as usize, true));
    }

    fn check_raw(&self, b: Self::S) -> Self::Result {
        log::trace!("{}", self.bits);
        if self.bits[b as usize] {
            BloomResult::MaybeContain
        } else {
            BloomResult::DoNotContain
        }
    }
}

//TODO
impl BF<[u8]> for Bloom<&'static [u8], [u64; 8]> {
    type Result = BloomResult;
    type S = u16;
    type H = MyDefaultHasher<512>;
    const SIZE: BloomSize = BloomSize::B512;

    fn bulk_insert<It: Iterator<Item = Self::S>>(&mut self, it: It) {
        it.for_each(|b| self.bits.set(b as usize, true));
    }

    fn check_raw(&self, b: Self::S) -> Self::Result {
        log::trace!("{}", self.bits);
        if self.bits[b as usize] {
            BloomResult::MaybeContain
        } else {
            BloomResult::DoNotContain
        }
    }
}

impl BF<[u8]> for Bloom<&'static [u8], [u64; 16]> {
    type Result = BloomResult;
    type S = u16;
    type H = MyDefaultHasher<1024>;
    const SIZE: BloomSize = BloomSize::B1024;

    fn bulk_insert<It: Iterator<Item = Self::S>>(&mut self, it: It) {
        it.for_each(|b| self.bits.set(b as usize, true));
    }

    fn check_raw(&self, b: Self::S) -> Self::Result {
        log::trace!("{}", self.bits);
        if self.bits[b as usize] {
            BloomResult::MaybeContain
        } else {
            BloomResult::DoNotContain
        }
    }
}

impl BF<[u8]> for Bloom<&'static [u8], [u64; 32]> {
    type Result = BloomResult;
    type S = u16;
    type H = MyDefaultHasher<2048>;
    const SIZE: BloomSize = BloomSize::B2048;

    fn bulk_insert<It: Iterator<Item = Self::S>>(&mut self, it: It) {
        it.for_each(|b| self.bits.set(b as usize, true));
    }

    fn check_raw(&self, b: Self::S) -> Self::Result {
        log::trace!("{}", self.bits);
        if self.bits[b as usize] {
            BloomResult::MaybeContain
        } else {
            BloomResult::DoNotContain
        }
    }
}

impl BF<[u8]> for Bloom<&'static [u8], [u64; 64]> {
    type Result = BloomResult;
    type S = u16;
    type H = MyDefaultHasher<4096>;
    const SIZE: BloomSize = BloomSize::B4096;

    fn bulk_insert<It: Iterator<Item = Self::S>>(&mut self, it: It) {
        it.for_each(|b| self.bits.set(b as usize, true));
    }

    fn check_raw(&self, b: Self::S) -> Self::Result {
        log::trace!("{}", self.bits);
        if self.bits[b as usize] {
            BloomResult::MaybeContain
        } else {
            BloomResult::DoNotContain
        }
    }
}
