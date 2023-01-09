use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

use hyper_ast::compat::HashMap;
use num_traits::{cast, one, zero, PrimInt};

pub trait MappingStore: Clone + Default {
    type Src;
    type Dst;
    fn topit(&mut self, left: usize, right: usize);
    fn len(&self) -> usize;
    fn capacity(&self) -> (usize, usize);
    fn has(&self, src: &Self::Src, dst: &Self::Dst) -> bool;
    fn link(&mut self, src: Self::Src, dst: Self::Dst);
    fn cut(&mut self, src: Self::Src, dst: Self::Dst);
    fn is_src(&self, src: &Self::Src) -> bool;
    fn is_dst(&self, dst: &Self::Dst) -> bool;
}
pub type DefaultMappingStore<T> = VecStore<T>;

pub trait MonoMappingStore: MappingStore {
    type Iter<'a>: Iterator<Item = (Self::Src, Self::Dst)>
    where
        Self: 'a;
    fn get_src_unchecked(&self, dst: &Self::Dst) -> Self::Src;
    fn get_dst_unchecked(&self, src: &Self::Src) -> Self::Dst;
    fn get_src(&self, dst: &Self::Dst) -> Option<Self::Src>;
    fn get_dst(&self, src: &Self::Src) -> Option<Self::Dst>;
    fn link_if_both_unmapped(&mut self, t1: Self::Src, t2: Self::Dst) -> bool;
    fn iter(&self) -> Self::Iter<'_>;
}

pub trait MultiMappingStore: MappingStore {
    type Iter1<'a>: Iterator<Item = Self::Src>
    where
        Self: 'a;
    type Iter2<'a>: Iterator<Item = Self::Dst>
    where
        Self: 'a;
    fn get_srcs(&self, dst: &Self::Dst) -> &[Self::Src];
    fn get_dsts(&self, src: &Self::Src) -> &[Self::Dst];
    fn all_mapped_srcs(&self) -> Self::Iter1<'_>;
    fn all_mapped_dsts(&self) -> Self::Iter2<'_>;
    fn is_src_unique(&self, dst: &Self::Src) -> bool;
    fn is_dst_unique(&self, src: &Self::Dst) -> bool;
}
pub type DefaultMultiMappingStore<T> = MultiVecStore<T>;

/// TODO try using umax
#[derive(Debug)]
pub struct VecStore<T> {
    pub src_to_dst: Vec<T>,
    pub dst_to_src: Vec<T>,
}

impl<T> Default for VecStore<T> {
    fn default() -> Self {
        Self {
            src_to_dst: Default::default(),
            dst_to_src: Default::default(),
        }
    }
}

impl<T: PrimInt + Debug> VecStore<T> {
    pub fn _iter(&self) -> impl Iterator<Item = (T, T)> + '_ {
        self.src_to_dst
            .iter()
            .enumerate()
            .filter(|x| *x.1 != zero())
            .map(|(src, dst)| (cast::<_, T>(src).unwrap(), *dst - one()))
    }

    pub fn link_if_both_unmapped(&mut self, t1: T, t2: T) -> bool {
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

impl<T: PrimInt + Debug> Clone for VecStore<T> {
    fn clone(&self) -> Self {
        Self {
            src_to_dst: self.src_to_dst.clone(),
            dst_to_src: self.dst_to_src.clone(),
        }
    }
}

impl<T: PrimInt + Debug> MappingStore for VecStore<T> {
    type Src = T;
    type Dst = T;

    fn len(&self) -> usize {
        self.src_to_dst.iter().filter(|x| **x != zero()).count()
    }

    fn capacity(&self) -> (usize, usize) {
        (self.src_to_dst.len(), self.dst_to_src.len())
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
        self.src_to_dst.resize(left + 1, zero());
        self.dst_to_src.resize(right + 1, zero());
    }

    fn has(&self, src: &Self::Src, dst: &Self::Src) -> bool {
        self.src_to_dst[src.to_usize().unwrap()] == *dst + one()
            && self.dst_to_src[dst.to_usize().unwrap()] == *src + one()
    }
}

impl<T: PrimInt + Debug> MonoMappingStore for VecStore<T> {
    fn get_src_unchecked(&self, dst: &T) -> T {
        self.dst_to_src[dst.to_usize().unwrap()] - one()
    }

    fn get_dst_unchecked(&self, src: &T) -> T {
        self.src_to_dst[src.to_usize().unwrap()] - one()
    }

    fn get_src(&self, dst: &T) -> Option<T> {
        self.dst_to_src
            .get(dst.to_usize().unwrap())
            .map(|x| *x - one())
    }

    fn get_dst(&self, src: &T) -> Option<T> {
        self.src_to_dst
            .get(src.to_usize().unwrap())
            .map(|x| *x - one())
    }

    fn link_if_both_unmapped(&mut self, t1: T, t2: T) -> bool {
        if self.is_src(&t1) && self.is_dst(&t2) {
            self.link(t1, t2);
            true
        } else {
            false
        }
    }

    type Iter<'a> = MonoIter<'a,T,T>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        MonoIter {
            v: self.src_to_dst.iter().enumerate(),
            // .filter(|x|*x.1 != zero()),
            // .map(|(src, dst)| (cast::<_, T>(src).unwrap(), *dst - one())),
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct MonoIter<'a, T: 'a + PrimInt, U: 'a> {
    v: std::iter::Enumerate<core::slice::Iter<'a, U>>,
    _phantom: std::marker::PhantomData<*const T>,
}

impl<'a, T: PrimInt, U: PrimInt> Iterator for MonoIter<'a, T, U> {
    type Item = (T, U);

    fn next(&mut self) -> Option<Self::Item> {
        let mut a = self.v.next();
        loop {
            let (i, x) = a?;
            if x.to_usize().unwrap() != 0 {
                return Some((cast::<_, T>(i).unwrap(), *x - one()));
            } else {
                a = self.v.next();
            }
        }
    }
}

pub struct MultiVecStore<T> {
    pub src_to_dsts: Vec<Option<Vec<T>>>,
    pub dst_to_srcs: Vec<Option<Vec<T>>>,
}

impl<T: PrimInt> Clone for MultiVecStore<T> {
    fn clone(&self) -> Self {
        Self {
            src_to_dsts: self.src_to_dsts.clone(),
            dst_to_srcs: self.dst_to_srcs.clone(),
        }
    }
}

impl<T> Default for MultiVecStore<T> {
    fn default() -> Self {
        Self {
            src_to_dsts: Default::default(),
            dst_to_srcs: Default::default(),
        }
    }
}

impl<T: PrimInt> MappingStore for MultiVecStore<T> {
    type Src = T;
    type Dst = T;

    fn len(&self) -> usize {
        self.src_to_dsts
            .iter()
            .filter_map(|x| x.as_ref())
            .map(|x| x.len())
            .sum()
    }

    fn capacity(&self) -> (usize, usize) {
        (self.src_to_dsts.len(), self.dst_to_srcs.len())
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

    fn has(&self, src: &Self::Src, dst: &Self::Dst) -> bool {
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

impl<T: PrimInt> MultiMappingStore for MultiVecStore<T> {
    type Iter1<'a> = Iter<'a,T> where T: 'a  ;
    type Iter2<'a> = Iter<'a,T> where T: 'a ;
    fn get_srcs(&self, dst: &Self::Dst) -> &[Self::Src] {
        self.dst_to_srcs[cast::<_, usize>(*dst).unwrap()]
            .as_ref()
            .and_then(|x| Some(x.as_slice()))
            .unwrap_or(&[])
    }

    fn get_dsts(&self, src: &Self::Src) -> &[Self::Dst] {
        self.src_to_dsts[cast::<_, usize>(*src).unwrap()]
            .as_ref()
            .and_then(|x| Some(x.as_slice()))
            .unwrap_or(&[])
    }

    fn all_mapped_srcs(&self) -> Iter<Self::Src> {
        Iter {
            v: self.src_to_dsts.iter().enumerate(),
        }
    }

    fn all_mapped_dsts(&self) -> Iter<Self::Dst> {
        Iter {
            v: self.dst_to_srcs.iter().enumerate(),
        }
    }

    fn is_src_unique(&self, src: &Self::Src) -> bool {
        self.get_dsts(src).len() == 1
    }

    fn is_dst_unique(&self, dst: &Self::Dst) -> bool {
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

// Debug/Display related helpers

impl<T: PrimInt + Debug> VecStore<T> {
    pub fn display<'b, Src, Dst>(
        &self,
        src_store: &'b Src,
        dst_store: &'b Dst,
    ) -> DisplayVecStore<'_, 'b, T, Src, Dst> {
        DisplayVecStore {
            mappings: self,
            src_store,
            dst_store,
        }
    }
}

pub struct DisplayVecStore<'a, 'b, T, Src, Dst> {
    mappings: &'a VecStore<T>,
    src_store: &'b Src,
    dst_store: &'b Dst,
}

impl<'a, 'b, T: PrimInt + TryFrom<usize>, Src, Dst, D: Display> Display
    for DisplayVecStore<'a, 'b, T, Src, Dst>
where
    Src: Fn(T) -> D,
    Dst: Fn(T) -> D,
    <T as TryFrom<usize>>::Error: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, x) in self.mappings.src_to_dst.iter().enumerate() {
            if !x.is_zero() {
                writeln!(
                    f,
                    "({},{})",
                    &(self.src_store)(i.try_into().unwrap()),
                    &(self.dst_store)(((*x).to_usize().unwrap() - 1).try_into().unwrap())
                )?;
            }
        }
        Ok(())
    }
}

// mappings
//     .src_to_dst
//     .to_owned()
//     .iter()
//     .enumerate()
//     .filter_map(|(i, t)| {
//         if *t == 0 {
//             None
//         } else {
//             Some((
//                 {
//                     let g = src_arena.original(&cast(i - 1).unwrap());
//                     let n = node_store.resolve(&g).label;
//                     std::str::from_utf8(&label_store.resolve(&n).to_owned())
//                         .unwrap()
//                         .to_owned()
//                 },
//                 {
//                     let g = dst_arena.original(&(*t - 2));
//                     let n = node_store.resolve(&g).label;
//                     let a = label_store.resolve(&n).to_owned();
//                     std::str::from_utf8(&a).unwrap().to_owned()
//                 },
//             ))
//         }
//     })
//     .for_each(|x| println!("{:?}", x))
// };

#[derive(Debug)]
pub struct HashStore<T> {
    pub src_to_dst: HashMap<T, T>,
    pub dst_to_src: HashMap<T, T>,
}

impl<T> Default for HashStore<T> {
    fn default() -> Self {
        Self {
            src_to_dst: Default::default(),
            dst_to_src: Default::default(),
        }
    }
}

impl<T: PrimInt + Debug + Hash> HashStore<T> {
    pub fn iter(&self) -> impl Iterator<Item = (T, T)> + '_ {
        self.src_to_dst.iter().map(|(src, dst)| (*src, *dst))
    }

    pub fn link_if_both_unmapped(&mut self, t1: T, t2: T) -> bool {
        if self.is_src(&t1) && self.is_dst(&t2) {
            self.link(t1, t2);
            true
        } else {
            false
        }
    }
}

impl<T: PrimInt + Debug> Clone for HashStore<T> {
    fn clone(&self) -> Self {
        Self {
            src_to_dst: self.src_to_dst.clone(),
            dst_to_src: self.dst_to_src.clone(),
        }
    }
}

impl<T: PrimInt + Debug + Hash> MappingStore for HashStore<T> {
    type Src = T;
    type Dst = T;

    fn len(&self) -> usize {
        self.src_to_dst.len()
    }

    fn capacity(&self) -> (usize, usize) {
        (self.src_to_dst.len(), self.dst_to_src.len())
    }

    fn link(&mut self, src: T, dst: T) {
        // assert_eq!(self.src_to_dst[src.to_usize().unwrap()], zero()); // maybe too strong req
        // assert_eq!(self.dst_to_src[dst.to_usize().unwrap()], zero()); // maybe too strong req
        self.src_to_dst.insert(src, dst);
        self.dst_to_src.insert(dst, src);
    }

    fn cut(&mut self, src: T, dst: T) {
        self.src_to_dst.remove(&src);
        self.dst_to_src.remove(&dst);
    }

    fn is_src(&self, src: &T) -> bool {
        self.src_to_dst.contains_key(&src)
    }

    fn is_dst(&self, dst: &T) -> bool {
        self.dst_to_src.contains_key(&dst)
    }

    fn topit(&mut self, _left: usize, _right: usize) {}

    fn has(&self, src: &Self::Src, dst: &Self::Src) -> bool {
        self.src_to_dst.contains_key(&src) && self.dst_to_src.contains_key(&dst)
    }
}

impl<T: PrimInt + Debug + Hash> MonoMappingStore for HashStore<T> {
    fn get_src_unchecked(&self, dst: &T) -> T {
        *self.dst_to_src.get(dst).unwrap()
    }

    fn get_dst_unchecked(&self, src: &T) -> T {
        *self.src_to_dst.get(src).unwrap()
    }

    fn get_src(&self, dst: &T) -> Option<T> {
        self.dst_to_src.get(dst).cloned()
    }

    fn get_dst(&self, src: &T) -> Option<T> {
        self.src_to_dst.get(src).cloned()
    }

    fn link_if_both_unmapped(&mut self, t1: T, t2: T) -> bool {
        if self.is_src(&t1) && self.is_dst(&t2) {
            self.link(t1, t2);
            true
        } else {
            false
        }
    }

    type Iter<'a> = HMIter<'a,T,T>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        HMIter {
            v: self.src_to_dst.iter(),
        }
    }
}

pub struct HMIter<'a, T: 'a + PrimInt, U: 'a> {
    v: hyper_ast::compat::hash_map::Iter<'a, T, U>,
}

impl<'a, T: PrimInt, U: PrimInt> Iterator for HMIter<'a, T, U> {
    type Item = (T, U);

    fn next(&mut self) -> Option<Self::Item> {
        let (x, y) = self.v.next()?;
        Some((*x, *y))
    }
}
