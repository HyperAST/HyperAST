use std::{
    borrow::Borrow,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use super::pearson_hashing::T;

pub fn hash16<T: Borrow<[u8]>>(x0: usize, x: T) -> u16 {
    let ret = T[x0] as u16;
    let v = x.borrow();
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    ret ^ Hasher::finish(&hasher) as u16
}

pub fn hash16_mod<T: Borrow<[u8]>, const MOD: u16>(x0: usize, x: T) -> u16 {
    let ret = T[x0] as u16;
    let v = x.borrow();
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    ret ^ (Hasher::finish(&hasher) % (MOD as u64)) as u16
}

#[derive(Clone)]
pub struct Pearson<const MOD: usize> {
    acc: u8,
}

pub trait VaryHasher<R>: Clone {
    const MOD: usize;
    fn new(init: usize) -> Self;
    fn finish(&self) -> R;
    fn write_u8(&mut self, i: u8);
    fn write_u16(&mut self, i: u16);
    fn write(&mut self, bytes: &[u8]);
}

impl<const MOD: usize> VaryHasher<u8> for Pearson<MOD> {
    const MOD: usize = MOD;
    fn new(init: usize) -> Self {
        Self { acc: T[init % 256] }
    }
    fn finish(&self) -> u8 {
        self.acc
    }
    fn write_u8(&mut self, i: u8) {
        if MOD as u8 == 0 {
            self.acc = T[(self.acc ^ i) as usize];
        } else {
            self.acc = T[(self.acc ^ i) as usize] % (MOD as u8);
        }
    }
    fn write_u16(&mut self, _: u16) {
        panic!()
    }
    fn write(&mut self, bytes: &[u8]) {
        for i in bytes {
            VaryHasher::<u8>::write_u8(self, *i);
        }
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct MyDefaultHasher<const MOD: usize>(DefaultHasher);

impl<const MOD: usize> VaryHasher<u16> for MyDefaultHasher<MOD> {
    const MOD: usize = MOD;
    fn new(init: usize) -> Self {
        let mut r = MyDefaultHasher(DefaultHasher::new());
        r.0.write_usize(init);
        r
    }
    fn finish(&self) -> u16 {
        if MOD as u64 == 0 {
            self.0.finish() as u16
        } else {
            (self.0.finish() % (MOD as u64)) as u16
        }
    }
    fn write_u8(&mut self, i: u8) {
        self.0.write_u8(i)
    }
    fn write_u16(&mut self, i: u16) {
        self.0.write_u16(i)
    }
    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes)
    }
}
