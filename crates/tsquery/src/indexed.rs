use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use num::ToPrimitive;

use crate::{utils::SafeUpcast, Capture};

// TODO use indexes on typed collections, it will for me to remove some casts and will help to normalize/generify indexes.
// it will also make it easier to maintain and change stuff later.
#[repr(transparent)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) struct StateId(u32);

impl StateId {
    pub(crate) const MAX: Self = Self(u32::MAX);
    pub(crate) const ZERO: Self = Self(0);
    pub(crate) fn inc(&mut self) {
        self.0 += 1;
    }
}

#[repr(transparent)]
#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Debug, Hash)]
pub(crate) struct StepId(pub(crate) u16);
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub(crate) struct PredStepId(u16);
impl PredStepId {
    pub(super) fn new(i: u16) -> Self {
        Self(i)
    }
}

impl StepId {
    pub(crate) const NONE: Self = Self(u16::MAX);

    pub(crate) fn next_step_index(&self) -> StepId {
        Self(self.0 + 1)
    }

    pub(crate) fn dec(&mut self) -> bool {
        if self.0 == 0 {
            false
        } else {
            self.0 -= 1;
            true
        }
    }

    pub(crate) fn inc(&mut self) {
        self.0 += 1;
    }

    pub(crate) fn new(index: u16) -> Self {
        Self(index)
    }
}
impl std::ops::Add for StepId {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.0 += rhs.0;
        self
    }
}

impl std::ops::AddAssign for StepId {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl std::ops::SubAssign for StepId {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}

impl std::fmt::Display for StepId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.0)
    }
}

#[derive(Clone)]
pub(crate) struct Steps(Vec<crate::query::QueryStep>);
impl Steps {
    pub(crate) fn contains(&self, step_index: StepId) -> bool {
        let step_index = step_index.0 as usize;
        step_index < self.0.len()
    }
    pub(crate) fn iter<'a>(&'a self) -> impl Iterator<Item = &'a crate::query::QueryStep> {
        self.0.iter()
    }

    pub(crate) fn set_immediate_pred(&mut self, s: StepId, i: u32) -> bool {
        self.0[s.0 as usize].set_immediate_pred(i)
    }

    pub(crate) fn extend(&mut self, steps: Steps) {
        let mut i = StepId::new(0);
        let mut j = StepId::new(num::cast(self.0.len()).unwrap());
        for s in steps.0 {
            dbg!(i, j, s.alternative_index());
            let s = s.adapt(i, j);
            self.0.push(s);
            i.inc();
            j.inc();
        }
    }

    pub(crate) fn iter_mut<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = &'a mut crate::query::QueryStep> {
        self.0.iter_mut()
    }

    pub(crate) fn count(&self) -> StepId {
        StepId(num::cast(self.0.len()).unwrap())
    }

    // pub(crate) fn set_neg(&mut self, sid: StepId) {
    //     self.0[sid.0 as usize].set_neg()
    // }
}

impl From<&crate::utils::Array<super::ffi_extra::TSQueryStep>> for Steps {
    fn from(value: &super::utils::Array<super::ffi_extra::TSQueryStep>) -> Self {
        Self(value.iter().map(|x| x.into()).collect())
    }
}

impl std::ops::Index<StepId> for Steps {
    type Output = crate::query::QueryStep;

    fn index(&self, index: StepId) -> &Self::Output {
        &self.0[index.0 as usize]
    }
}

// impl<T: Indexed> std::ops::IndexMut<T::I> for Array<T> {
//     fn index_mut(&mut self, index: T::I) -> &mut Self::Output {
//         &mut self.0[index.index_as_usize()]
//     }
// }

// impl<U> FromIterator<U> for Array<U> {
//     fn from_iter<T: IntoIterator<Item = U>>(iter: T) -> Self {
//         Self(iter.into_iter().collect())
//     }
// }

#[repr(transparent)]
#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Debug)]
pub struct CaptureId(pub(crate) u16);

impl Default for CaptureId {
    fn default() -> Self {
        Self::ZERO // TODO check that
    }
}

impl From<u32> for CaptureId {
    fn from(value: u32) -> Self {
        CaptureId(value.to_u16().expect("an u16 and not an u32"))
    }
}

impl CaptureId {
    pub(crate) const NONE: Self = Self(u16::MAX);
    pub(crate) const ZERO: Self = Self(0);
    // const MAX: Self = Self(u16::MAX);
    pub(super) fn new(i: u32) -> Self {
        assert!(i <= u16::MAX as u32, "{}", i);
        Self(i as u16)
    }

    pub(crate) fn inc(&mut self) {
        self.0 += 1;
    }

    // fn index_from_usize(i: usize) -> Self {
    //     assert!(i < u16::MAX as usize);
    //     Self(i as u16)
    // }

    // fn zero() -> Self {
    //     Self(0)
    // }

    // fn one() -> Self {
    //     Self(1)
    // }
    pub(crate) fn to_usize(&self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for CaptureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.0)
    }
}

#[repr(transparent)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(crate) struct CaptureListId(usize);

impl CaptureListId {
    pub(crate) const MAX: Self = Self(usize::MAX);
}

pub(crate) struct CaptureListPool<Node> {
    list: Vec<Option<Captures<Node>>>,
    // The maximum number of lists that we are allowed to allocate. We
    // never allow `list` to allocate more entries than this, dropping pending
    // matches if needed to stay under the limit.
    max_list_count: u32,
    // The number of lists allocated in `list` that are not currently in
    // use. We reuse those existing-but-unused lists before trying to
    // allocate any new ones. We use an invalid value (UINT32_MAX) for a
    // list's length to indicate that it's not in use.
    free_list_count: u32,
}

impl<Node> Default for CaptureListPool<Node> {
    fn default() -> Self {
        Self {
            list: Default::default(),
            max_list_count: u32::MAX,
            // max_list_count: 20,
            free_list_count: Default::default(),
        }
    }
}

#[repr(transparent)]
pub struct Captures<Node>(Vec<Capture<Node>>);
impl<Node> Captures<Node> {
    pub(crate) fn nodes_for_capture_index<'a>(
        &'a self,
        index: CaptureId,
    ) -> impl Iterator<Item = &'a Node> {
        self.0
            .iter()
            .filter(move |x| x.index == index)
            .map(|x| &x.node)
    }
    pub(crate) fn captures(&self) -> &[Capture<Node>] {
        &self.0
    }

    pub(crate) fn clear(&mut self) {
        self.0.clear()
    }

    pub(crate) fn push(&mut self, capture: Capture<Node>) {
        self.0.push(capture)
    }
}

impl<Node> std::ops::Deref for Captures<Node> {
    type Target = CaptureSlice<Node>;

    fn deref(&self) -> &Self::Target {
        let src: &[Capture<Node>] = self.0.as_slice();
        CaptureSlice::conv(src)
    }
}

impl<Node> std::borrow::Borrow<CaptureSlice<Node>> for Captures<Node> {
    fn borrow(&self) -> &CaptureSlice<Node> {
        let src: &[Capture<Node>] = self.0.as_slice();
        CaptureSlice::conv(src)
    }
}

impl<Node> Index<CaptureId> for Captures<Node> {
    type Output = Capture<Node>;

    fn index(&self, index: CaptureId) -> &Self::Output {
        let to: usize = index.0.to();
        &self.0[to]
    }
}

impl<'a, Node: Clone> IntoIterator for &'a Captures<Node> {
    type Item = &'a Capture<Node>;

    type IntoIter = std::slice::Iter<'a, Capture<Node>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(ref_cast::RefCastCustom)]
#[repr(transparent)]
pub struct CaptureSlice<Node>([Capture<Node>]);

impl<Node> CaptureSlice<Node> {
    #[ref_cast::ref_cast_custom]
    pub(crate) const fn conv(bytes: &[Capture<Node>]) -> &Self;

    const fn empty<'a>() -> &'a Self {
        CaptureSlice::conv(&[])
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub(crate) fn contains(&self, i: CaptureId) -> bool {
        self.0.len() > i.0.to()
    }
}

impl<Node> Index<CaptureId> for CaptureSlice<Node> {
    type Output = Capture<Node>;

    fn index(&self, index: CaptureId) -> &Self::Output {
        let to: usize = index.0.to();
        &self.0[to]
    }
}

impl<Node: Clone> ToOwned for CaptureSlice<Node> {
    type Owned = Captures<Node>;

    fn to_owned(&self) -> Self::Owned {
        Captures(self.0.to_vec())
    }
}

impl<'a, Node: Clone> IntoIterator for &'a CaptureSlice<Node> {
    type Item = &'a Capture<Node>;

    type IntoIter = std::slice::Iter<'a, Capture<Node>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<Node> Index<CaptureListId> for CaptureListPool<Node> {
    type Output = Captures<Node>;

    fn index(&self, index: CaptureListId) -> &Self::Output {
        self.list[index.0.to_usize()].as_ref().unwrap()
    }
}

impl<Node> IndexMut<CaptureListId> for CaptureListPool<Node> {
    fn index_mut(&mut self, index: CaptureListId) -> &mut Self::Output {
        self.list[index.0].as_mut().unwrap()
    }
}
impl<Node> CaptureListPool<Node> {
    pub(super) fn release(&mut self, id: CaptureListId) {
        if id.0 >= self.list.len() {
            return;
        }
        // self.list[id.index_0].clear();
        self.list[id.0] = None;
        self.free_list_count += 1;
    }
    pub(super) fn get(&self, id: CaptureListId) -> &CaptureSlice<Node> {
        if id.0 >= self.list.len() {
            return CaptureSlice::empty();
        };
        return self.list[id.0].as_ref().unwrap();
    }
    pub(super) fn pop(&mut self, id: CaptureListId) -> Captures<Node> {
        if id.0 >= self.list.len() {
            return Captures(vec![]);
        };
        let r = self.list[id.0].take();
        self.free_list_count += 1;
        return r.unwrap();
    }
    pub(super) fn acquire(&mut self) -> CaptureListId {
        // First see if any already allocated list is currently unused.
        if self.free_list_count > 0 {
            for i in 0..self.list.len() {
                if self.list[i].is_none() {
                    self.list[i] = Some(Captures(vec![]));
                    self.free_list_count -= 1;
                    return CaptureListId(i);
                }
                // if self.list[i].len() == 0 {
                //     self.list[i].clear();
                //     self.free_list_count -= 1;
                //     return CaptureListId(i);
                // }
            }
        }

        // Otherwise allocate and initialize a new list, as long as that
        // doesn't put us over the requested maximum.
        let i = self.list.len();
        if i >= self.max_list_count as usize {
            return CaptureListId::MAX;
        }
        self.list.push(Some(Captures(vec![])));
        return CaptureListId(i);
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PatternId(usize);

impl PatternId {
    pub(crate) const NONE: PatternId = PatternId(usize::MAX);
    pub(crate) const fn new(i: usize) -> Self {
        Self(i)
    }

    pub fn to_usize(self) -> usize {
        self.0
    }

    pub fn is_none(&self) -> bool {
        self == &Self::NONE
    }
}

impl Display for PatternId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Patterns(Vec<crate::query::QueryPattern>);
impl Patterns {
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn extend(&mut self, patterns: Patterns, offset: StepId, byte_offset: u32) {
        for p in patterns.0 {
            let p = p.adapt(offset, byte_offset);
            self.0.push(p);
        }
    }
}
impl Index<PatternId> for Patterns {
    type Output = crate::query::QueryPattern;

    fn index(&self, index: PatternId) -> &Self::Output {
        &self.0[index.0.to_usize()]
    }
}
impl IndexMut<PatternId> for Patterns {
    fn index_mut(&mut self, index: PatternId) -> &mut Self::Output {
        &mut self.0[index.0.to_usize()]
    }
}

impl Into<Patterns> for &crate::utils::Array<crate::ffi_extra::QueryPattern> {
    fn into(self) -> Patterns {
        Patterns(self.iter().map(|x| x.into()).collect())
    }
}

#[derive(Clone)]
pub(crate) struct NegatedFields(Vec<crate::ffi::TSFieldId>);
impl NegatedFields {
    pub(crate) fn get(
        &self,
        negated_field_list_id: u16,
    ) -> impl Iterator<Item = crate::ffi::TSFieldId> + '_ {
        self.0[negated_field_list_id as usize..]
            .iter()
            .take_while(|i| **i != 0)
            .map(|x| *x)
    }

    pub(crate) fn extend(&self, negated_fields: NegatedFields) -> Vec<u16> {
        let mut map = vec![];
        for n in negated_fields.0 {
            if n == 0 {
                continue;
            }
            dbg!(n);
            todo!()
        }
        map
    }
}

impl Into<NegatedFields> for &crate::utils::Array<crate::ffi::TSFieldId> {
    fn into(self) -> NegatedFields {
        NegatedFields(self.iter().map(|x| *x).collect())
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
#[repr(transparent)]
pub struct Symbol(u16);

impl Symbol {
    pub(crate) const ERROR: Symbol = Symbol(u16::MAX - 1);
    pub(crate) const NONE: Symbol = Symbol(u16::MAX);
    pub(crate) const END: Symbol = Symbol(0);
    pub(crate) const WILDCARD_SYMBOL: Symbol = Symbol(0);
    // const WILDCARD_SYMBOL: index::Symbol = index::Symbol(0);

    pub(crate) fn to_usize(&self) -> usize {
        self.0 as usize
    }
    pub fn is_error(&self) -> bool {
        self == &Self::ERROR
    }
}

impl From<u16> for Symbol {
    fn from(value: u16) -> Self {
        Symbol(value)
    }
}
