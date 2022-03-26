use std::{borrow::Borrow, collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};

use super::pearson_hashing::{T};

pub fn hash16<T: Borrow<[u8]>>(x0: usize, x: T) -> u16 {
    let mut ret = T[x0] as u16;
    let v = x.borrow();
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    ret ^ hasher.finish() as u16
}

pub fn hash16_mod<T: Borrow<[u8]>, const MOD: u16>(x0: usize, x: T) -> u16 {
    let mut ret = T[x0] as u16;
    let v = x.borrow();
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    ret ^ (hasher.finish() % (MOD as u64)) as u16
}