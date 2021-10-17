use std::fmt::Debug;

use num_traits::{cast, one, zero, PrimInt};

pub trait MappingStore: Clone {
    type Ele: Eq;
    fn topit(&mut self, left: usize, right: usize);
    fn len(&self) -> usize;
    fn has(&self, src: &Self::Ele, dst: &Self::Ele) -> bool;
    fn link(&mut self, src: Self::Ele, dst: Self::Ele);
    fn cut(&mut self, src: Self::Ele, dst: Self::Ele);
    fn is_src(&self, src: &Self::Ele) -> bool;
    fn is_dst(&self, dst: &Self::Ele) -> bool;
}

pub trait MonoMappingStore: MappingStore {
    fn get_src(&self, dst: &Self::Ele) -> Self::Ele;
    fn get_dst(&self, src: &Self::Ele) -> Self::Ele;
}

pub trait MultiMappingStore: MappingStore {
    fn get_srcs(&self, dst: &Self::Ele) -> &[Self::Ele];
    fn get_dsts(&self, src: &Self::Ele) -> &[Self::Ele];
    fn allMappedSrcs(&self) -> Iter<Self::Ele>;
    fn allMappedDsts(&self) -> Iter<Self::Ele>;
    fn isSrcUnique(&self, dst: &Self::Ele) -> bool;
    fn isDstUnique(&self, src: &Self::Ele) -> bool;
}

/// TODO try using umax
pub struct DefaultMappingStore<T> {
    pub src_to_dst: Vec<T>,
    pub dst_to_src: Vec<T>,
}

impl<T: PrimInt + Debug> DefaultMappingStore<T> {
    pub fn new() -> Self {
        Self {
            src_to_dst: vec![zero()],
            dst_to_src: vec![zero()],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (T, T)> + '_ {
        self.src_to_dst
            .iter()
            .enumerate()
            .filter(|x| *x.1 != zero())
            .map(|(src, dst)| (cast::<_, T>(src).unwrap(), *dst - one()))
    }

    pub(crate) fn link_if_both_unmapped(&mut self, t1: T, t2: T) -> bool {
        if self.is_src(&t1) && self.is_dst(&t2) {
            self.link(t1, t2);
            true
        } else {
            false
        }
    }
}

// struct Iter<T, It:Iterator<Item = (T,T)>> {
//     internal:It,
// }

// impl<T, It:Iterator<Item = (T,T)>> Iterator for Iter<T,It> {
//     type Item = (T,T);

//     fn next(&mut self) -> Option<Self::Item> {
//         todo!()
//     }
// }

impl<T: PrimInt + Debug> Clone for DefaultMappingStore<T> {
    fn clone(&self) -> Self {
        Self {
            src_to_dst: self.src_to_dst.clone(),
            dst_to_src: self.dst_to_src.clone(),
        }
    }
}

impl<T: PrimInt + Debug> MappingStore for DefaultMappingStore<T> {
    type Ele = T;

    fn len(&self) -> usize {
        self.src_to_dst.iter().filter(|x| **x != zero()).count()
    }

    fn link(&mut self, src: T, dst: T) {
        // assert_eq!(self.src_to_dst[src.to_usize().unwrap()], zero()); // maybe too strong req
        // assert_eq!(self.dst_to_src[dst.to_usize().unwrap()], zero()); // maybe too strong req
        self.src_to_dst[src.to_usize().unwrap()] = dst + one();
        self.dst_to_src[dst.to_usize().unwrap()] = src + one();
    }

    fn cut(&mut self, src: T, dst: T) {
        self.src_to_dst[src.to_usize().unwrap()] = zero();
        self.dst_to_src[dst.to_usize().unwrap()] = zero();
    }

    fn is_src(&self, src: &T) -> bool {
        self.src_to_dst[src.to_usize().unwrap()] != zero()
    }

    fn is_dst(&self, dst: &T) -> bool {
        self.dst_to_src[dst.to_usize().unwrap()] != zero()
    }

    fn topit(&mut self, left: usize, right: usize) {
        // let m = left.max(right);
        self.src_to_dst.resize(left, zero());
        self.dst_to_src.resize(right, zero());
    }

    fn has(&self, src: &Self::Ele, dst: &Self::Ele) -> bool {
        self.src_to_dst[src.to_usize().unwrap()] == *dst + one()
            && self.dst_to_src[dst.to_usize().unwrap()] == *src + one()
    }
}

impl<T: PrimInt + Debug> MonoMappingStore for DefaultMappingStore<T> {
    fn get_src(&self, dst: &T) -> T {
        self.dst_to_src[dst.to_usize().unwrap()] - one()
    }

    fn get_dst(&self, src: &T) -> T {
        self.src_to_dst[src.to_usize().unwrap()] - one()
    }
}

pub struct DefaultMultiMappingStore<T> {
    pub src_to_dsts: Vec<Option<Vec<T>>>,
    pub dst_to_srcs: Vec<Option<Vec<T>>>,
}

impl<T: PrimInt> Clone for DefaultMultiMappingStore<T> {
    fn clone(&self) -> Self {
        Self {
            src_to_dsts: self.src_to_dsts.clone(),
            dst_to_srcs: self.dst_to_srcs.clone(),
        }
    }
}

impl<T: PrimInt> MappingStore for DefaultMultiMappingStore<T> {
    type Ele = T;

    fn len(&self) -> usize {
        self.src_to_dsts.iter().filter(|x| (**x).is_some()).count()
    }

    fn link(&mut self, src: T, dst: T) {
        // self.src_to_dsts[src.to_usize().unwrap()].get_or_insert_default().push(dst); // todo when not unstable feature
        if self.src_to_dsts[src.to_usize().unwrap()].is_none() {
            self.src_to_dsts[src.to_usize().unwrap()] = Some(vec![])
        }
        self.src_to_dsts[src.to_usize().unwrap()]
            .as_mut()
            .unwrap()
            .push(dst);
        if self.dst_to_srcs[dst.to_usize().unwrap()].is_none() {
            self.dst_to_srcs[dst.to_usize().unwrap()] = Some(vec![])
        }
        self.dst_to_srcs[dst.to_usize().unwrap()]
            .as_mut()
            .unwrap()
            .push(src);
    }

    fn cut(&mut self, src: T, dst: T) {
        if let Some(i) = self.src_to_dsts[src.to_usize().unwrap()]
            .as_ref()
            .and_then(|v| v.iter().position(|x| x == &dst))
        {
            if self.src_to_dsts[src.to_usize().unwrap()]
                .as_ref()
                .unwrap()
                .len()
                == 1
            {
                self.src_to_dsts[src.to_usize().unwrap()] = None;
            } else {
                self.src_to_dsts[src.to_usize().unwrap()]
                    .as_mut()
                    .unwrap()
                    .remove(i);
            }
        }
        if let Some(i) = self.dst_to_srcs[dst.to_usize().unwrap()]
            .as_ref()
            .and_then(|v| v.iter().position(|x| x == &src))
        {
            if self.dst_to_srcs[dst.to_usize().unwrap()]
                .as_ref()
                .unwrap()
                .len()
                == 1
            {
                self.dst_to_srcs[dst.to_usize().unwrap()] = None;
            } else {
                self.dst_to_srcs[dst.to_usize().unwrap()]
                    .as_mut()
                    .unwrap()
                    .remove(i);
            }
        }
    }

    fn is_src(&self, src: &T) -> bool {
        self.src_to_dsts[src.to_usize().unwrap()].is_some()
    }

    fn is_dst(&self, dst: &T) -> bool {
        self.dst_to_srcs[dst.to_usize().unwrap()].is_some()
    }

    fn topit(&mut self, left: usize, right: usize) {
        self.src_to_dsts.resize(left, None);
        self.dst_to_srcs.resize(right, None);
    }

    fn has(&self, src: &Self::Ele, dst: &Self::Ele) -> bool {
        self.src_to_dsts[src.to_usize().unwrap()]
            .as_ref()
            .and_then(|v| Some(v.contains(dst)))
            .unwrap_or(false)
            && self.dst_to_srcs[dst.to_usize().unwrap()]
                .as_ref()
                .and_then(|v| Some(v.contains(src)))
                .unwrap_or(false)
    }
}

impl<T: PrimInt> MultiMappingStore for DefaultMultiMappingStore<T> {
    fn get_srcs(&self, dst: &Self::Ele) -> &[Self::Ele] {
        self.dst_to_srcs[cast::<_, usize>(*dst).unwrap()]
            .as_ref()
            .and_then(|x| Some(x.as_slice()))
            .unwrap_or(&[])
    }

    fn get_dsts(&self, src: &Self::Ele) -> &[Self::Ele] {
        self.src_to_dsts[cast::<_, usize>(*src).unwrap()]
            .as_ref()
            .and_then(|x| Some(x.as_slice()))
            .unwrap_or(&[])
    }

    fn allMappedSrcs(&self) -> Iter<Self::Ele> {
        Iter {
            v: self.src_to_dsts.iter().enumerate(),
        }
    }

    fn allMappedDsts(&self) -> Iter<Self::Ele> {
        Iter {
            v: self.dst_to_srcs.iter().enumerate(),
        }
    }

    fn isSrcUnique(&self, src: &Self::Ele) -> bool {
        self.get_dsts(src).len() == 1
    }

    fn isDstUnique(&self, dst: &Self::Ele) -> bool {
        self.get_srcs(dst).len() == 1
    }
}

pub struct Iter<'a, T: 'a> {
    v: std::iter::Enumerate<core::slice::Iter<'a, Option<Vec<T>>>>,
}

impl<'a, T: PrimInt> Iterator for Iter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut a = self.v.next();
        loop {
            if let Some((i, x)) = a {
                if let Some(_) = x {
                    return Some(cast::<_, T>(i).unwrap());
                } else {
                    a = self.v.next();
                }
            } else {
                return None;
            }
        }
    }
}
