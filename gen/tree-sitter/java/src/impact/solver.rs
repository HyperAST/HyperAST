use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Deref, Index},
};

use num::ToPrimitive;
use string_interner::Symbol;

use crate::impact::declaration::DebugDecls;

use super::{
    declaration::{self, DeclType, Declarator},
    element::{
        self, Arguments, ExplorableRef, IdentifierFormat, LabelPtr, ListSet, Nodes, RefPtr,
        RefsEnum,
    },
    java_element::Primitive,
    reference,
};

/// Allow to handle referencial relations i.e. resolve references
#[derive(Debug, Clone)]
pub struct Solver {
    // leafs: LeafSet,
    pub nodes: Nodes,
    pub(crate) refs: bitvec::vec::BitVec,
    decls: HashMap<Declarator<RefPtr>, DeclType<RefPtr>>,
    // root: Option<RefPtr>,
}

impl Default for Solver {
    fn default() -> Self {
        Self {
            // leafs: Default::default(),
            nodes: vec![RefsEnum::Root, RefsEnum::MaybeMissing].into(),
            refs: Default::default(),
            decls: Default::default(),
            // root: None,
        }
    }
}

pub struct MultiResult<T>(Option<Box<[T]>>);

impl<T> Default for MultiResult<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<T: Clone> Clone for MultiResult<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Eq> FromIterator<T> for MultiResult<T> {
    fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
        let mut r = Default::default();
        let mut v = vec![];
        for x in iter.into_iter() {
            if !v.contains(&x) {
                v.push(x)
            }
        }
        if !v.is_empty() {
            r = Some(v.into());
        }
        Self(r)
    }
}
impl<'a, T: Copy> FromIterator<&'a T> for MultiResult<T> {
    fn from_iter<U: IntoIterator<Item = &'a T>>(iter: U) -> Self {
        let mut r = Default::default();
        let b: Box<[T]> = iter.into_iter().map(|x| *x).collect();
        if !b.is_empty() {
            r = Some(b);
        }
        Self(r)
    }
}
// impl<'a, T> IntoIterator for MultiResult<T> {
//     type Item=T;

//     type IntoIter=std::iter::FlatMap<core::option::IntoIter<Box<[T]>>, dyn core::iter::IntoIterator<Item=T,IntoIter = >, fn(Box<[T]>) -> _>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.0.into_iter().flat_map(|x| x.into_iter())
//     }
// }

// impl<T:'static+Copy> MultiResult<T> {
//     fn into_iter(self) -> impl Iterator<Item = T> {
//         self.0.iter().flat_map(|x| x.iter()).map(|x|*x)
//     }
// }
impl<T> MultiResult<T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter().flat_map(|x| x.iter())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }
}

/// accessors to references and declarations
impl Solver {
    pub fn decls_count(&self) -> usize {
        self.decls.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.refs.is_empty() && self.decls.is_empty()
    }

    pub(crate) fn lower_estimate_refs_count(&self) -> u32 {
        // self.refcount
        self.refs.count_ones().to_u32().unwrap()
    }
    // pub fn refs(&self) -> impl Iterator<Item = LabelValue> + '_ {
    //     self.refs
    //         .iter_ones()
    //         // iter().enumerate()
    //         // .filter_map(|(x,b)| if *b {Some(x)} else {None})
    //         .map(|x| self.nodes.with(x).bytes().into())
    // }

    pub fn iter_refs<'a>(&'a self) -> reference::Iter<'a> {
        reference::Iter {
            nodes: &self.nodes,
            refs: self.refs.iter_ones(),
        }
    }

    pub(crate) fn iter_decls<'a>(&'a self) -> declaration::DeclsIter<'a> {
        declaration::DeclsIter {
            nodes: &self.nodes,
            decls: self.decls.iter(),
        }
    }
    pub(crate) fn get(&self, other: RefsEnum<RefPtr, LabelPtr>) -> Option<RefPtr> {
        self.nodes.get(other)
    }

    pub fn iter_nodes<'a>(&'a self) -> element::NodesIter<'a> {
        element::NodesIter {
            rf: 0,
            nodes: &self.nodes,
        }
    }
}

/// basic insertions of references
impl Solver {
    /// add a node to the solver
    pub fn intern(&mut self, other: RefsEnum<RefPtr, LabelPtr>) -> RefPtr {
        // TODO analyze perfs to find if Vec or HashSet or something else works better
        match self.nodes.iter().position(|x| other.strict_eq(x)) {
            Some(x) => x,
            None => {
                let r = self.nodes.len();
                self.nodes.push(other);
                r
            }
        }
    }

    /// add a reference to the solver
    pub fn intern_ref(&mut self, other: RefsEnum<RefPtr, LabelPtr>) -> RefPtr {
        match other {
            RefsEnum::Primitive(_) => panic!(),
            _ => (),
        };
        let r = self.intern(other);
        match &self.nodes[r] {
            RefsEnum::Primitive(_) => panic!(),
            _ => (),
        };
        if r >= self.refs.len() {
            self.refs.resize(r + 1, false);
        }
        self.refs.set(r, true);
        r
    }

    /// add a declaration to the solver
    pub(crate) fn add_decl(&mut self, d: Declarator<RefPtr>, t: DeclType<RefPtr>) {
        self.decls.insert(d, t);
    }
    // pub(crate) fn add_decl_simple(&mut self, d: Declarator<RefPtr>, t: RefPtr) {
    //     self.decls
    //         .insert(d, DeclType::Compile(t, None, Default::default()));
    // }

    /// copy a referencial element from another solver to the current one
    fn intern_external(
        &mut self,
        map: &mut HashMap<RefPtr, RefPtr>,
        cache: &mut HashMap<RefPtr, RefPtr>,
        other: ExplorableRef,
    ) -> RefPtr {
        if let Some(x) = map.get(&other.rf) {
            return *x;
        }
        if let Some(x) = cache.get(&other.rf) {
            assert!(
                self.nodes[*x].similar(other.as_ref()),
                "{:?} ~ {:?}",
                other,
                ExplorableRef {
                    nodes: &self.nodes,
                    rf: *x
                },
            );
            return *x;
        }

        let mut rec = |&x: &usize| self.intern_external(map, cache, other.with(x));

        let r = match other.as_ref() {
            RefsEnum::Root => self.intern(RefsEnum::Root),
            RefsEnum::MaybeMissing => self.intern(RefsEnum::MaybeMissing),
            RefsEnum::Primitive(i) => self.intern(RefsEnum::Primitive(*i)),
            RefsEnum::Mask(o, p) => {
                let o = rec(o);
                let p = p.iter().map(rec).collect();
                self.intern(RefsEnum::Mask(o, p))
            }
            RefsEnum::Or(p) => {
                let p = p.iter().map(rec).collect();
                self.intern(RefsEnum::Or(p))
            }
            RefsEnum::Invocation(o, i, p) => {
                let o = rec(o);
                let p = p.map(rec);
                self.intern(RefsEnum::Invocation(o, *i, p))
            }
            RefsEnum::ConstructorInvocation(i, p) => {
                let i = rec(i);
                let p = p.map(rec);
                let r = self.intern(RefsEnum::ConstructorInvocation(i, p));
                assert_ne!(r, i);
                r
            }
            x => {
                let o = x
                    .object()
                    .unwrap_or_else(|| panic!("should have an object {:?}", x));
                let o = rec(&o);
                self.intern(x.with_object(o))
            }
        };
        assert!(
            self.nodes[r].similar(other.as_ref()),
            "{:?} ~ {:?}",
            other.as_ref(),
            self.nodes[r],
        );
        cache.insert(other.rf, r);
        r
    }
}

/// basic modifications of references
impl Solver {
    /// try to reconstruct `t` with `p` replacing the main MaybeMissing (?).
    ///
    /// returns None if `t` does not end with (?).
    pub fn try_solve_node_with(&mut self, t: RefPtr, p: RefPtr) -> Option<RefPtr> {
        macro_rules! refs {
            ( $x:expr ) => {
                if t < self.refs.len() && self.refs[t] {
                    self.intern_ref($x)
                } else {
                    self.intern($x)
                }
            };
        }
        match self.nodes[t].clone() {
            RefsEnum::Root => None, //("fully qualified node cannot be qualified further")),
            RefsEnum::MaybeMissing => Some(p),
            RefsEnum::Or(v) => {
                let o = v
                    .iter()
                    .flat_map(|&o| {
                        self.try_solve_node_with(o, p).map_or(vec![].into(), |x| {
                            match &self.nodes[x] {
                                RefsEnum::Or(v) => v.clone(),
                                _ => vec![x].into(),
                            }
                        })
                    })
                    .collect();
                let tmp = RefsEnum::Or(o);
                Some(refs!(tmp))
            }
            x => {
                let o = x
                    .object()
                    .unwrap_or_else(|| panic!("should have an object {:?}", x));
                let o = self.try_solve_node_with(o, p)?;
                let tmp = x.with_object(o);
                Some(refs!(tmp))
            }
        }
    }

    /// try to reconstruct `t` with ? replacing `p`.
    ///
    /// returns None if `t` does not end (start for serialized order) with `p`.
    pub fn try_unsolve_node_with(&mut self, t: RefPtr, p: RefPtr) -> Option<RefPtr> {
        macro_rules! refs {
            ( $x:expr ) => {
                if t < self.refs.len() && self.refs[t] {
                    self.intern_ref($x)
                } else {
                    self.intern($x)
                }
            };
        }
        if p == t {
            return Some(self.intern(RefsEnum::MaybeMissing));
        }
        match self.nodes[t].clone() {
            RefsEnum::Root => None,
            RefsEnum::MaybeMissing => None,
            RefsEnum::Or(y) => {
                let x =
                    y.iter()
                        .flat_map(|&o| {
                            self.try_unsolve_node_with(o, p)
                                .map_or(vec![].into(), |x| match &self.nodes[x] {
                                    RefsEnum::Or(v) => v.clone(),
                                    _ => vec![x].into(),
                                })
                        })
                        .collect(); // TODO not sure
                let tmp = RefsEnum::Or(x);
                Some(refs!(tmp))
            }
            RefsEnum::Mask(i, y) => {
                let x = self.try_unsolve_node_with(i, p)?; // TODO not sure
                let tmp = RefsEnum::Mask(x, y);
                Some(refs!(tmp))
            }
            x => {
                let o = x
                    .object()
                    .unwrap_or_else(|| panic!("should have an object {:?}", x));
                let o = self.try_unsolve_node_with(o, p)?;
                let tmp = x.with_object(o);
                Some(refs!(tmp))
            }
        }
    }

    /// reconstruct the [`other`] referential element without masks
    fn no_mask(&mut self, other: RefPtr) -> RefPtr {
        let o = self.nodes[other].object();
        let o = if let Some(o) = o {
            self.no_mask(o)
        } else {
            return other;
        };
        if let RefsEnum::Mask(_, _) = self.nodes[other] {
            return o;
        }
        let x = self.nodes[other].with_object(o);
        self.intern(x)
    }

    pub fn no_choice(&mut self, other: RefPtr) -> RefPtr {
        let o = self.nodes[other].object();
        let o = if let Some(o) = o {
            self.no_choice(o)
        } else {
            return other;
        };
        if let RefsEnum::Or(_) = self.nodes[other] {
            return o;
        }
        let x = self.nodes[other].with_object(o);
        self.intern(x)
    }
    pub fn has_choice(&self, other: RefPtr) -> bool {
        if let RefsEnum::Or(_) = self.nodes[other] {
            return true;
        }
        let o = self.nodes[other].object();
        if let Some(o) = o {
            self.has_choice(o)
        } else {
            false
        }
    }
}

/// advanced insertions with local solving
impl Solver {
    /// dedicated to solving references to localvariables
    pub(crate) fn local_solve_extend<'a>(&mut self, solver: &'a Solver) -> Internalizer<'a> {
        let mut cached = Internalizer {
            solve: true,
            cache: Default::default(),
            cache_decls: Default::default(),
            solver,
        };
        // self.print_decls();
        for r in solver.iter_refs() {
            // TODO make it imperative ?
            let r = self.local_solve_intern_external(&mut cached.cache, r);
            match &self.nodes[r] {
                RefsEnum::Primitive(_) => {}
                RefsEnum::Array(o) => {
                    if let RefsEnum::Primitive(_) = &self.nodes[*o] {
                    } else {
                        if r >= self.refs.len() {
                            self.refs.resize(r + 1, false);
                        }
                        self.refs.set(r, true);
                    }
                }
                _ => {
                    if r >= self.refs.len() {
                        self.refs.resize(r + 1, false);
                    }
                    self.refs.set(r, true);
                }
            };
        }
        cached
    }

    /// dedicated to solving references in scopes that contain localvariables
    fn local_solve_intern_external(
        &mut self,
        cache: &mut HashMap<RefPtr, RefPtr>,
        other: ExplorableRef,
    ) -> RefPtr {
        if let Some(x) = cache.get(&other.rf) {
            return *x;
        }
        // log::trace!("other: {:?}", other);
        let r = match other.as_ref() {
            RefsEnum::Root => self.intern(RefsEnum::Root),
            RefsEnum::MaybeMissing => self.intern(RefsEnum::MaybeMissing),
            RefsEnum::Primitive(i) => self.intern(RefsEnum::Primitive(*i)),
            RefsEnum::ArrayAccess(o) => {
                let o = self.local_solve_intern_external(cache, other.with(*o));
                match &self.nodes[o] {
                    RefsEnum::Array(x) => *x,
                    _ => self.intern(RefsEnum::ArrayAccess(o)),
                }
            }
            RefsEnum::Mask(o, v) => {
                // log::trace!("try solve mask: {:?}", other);
                let o = self.local_solve_intern_external(cache, other.with(*o));
                let v = v
                    .iter()
                    .map(|x| self.local_solve_intern_external(cache, other.with(*x)))
                    .collect();
                // TODO should look for declarations solving the masking
                // either the masked thing is declared by thing in mask
                // or the masked thing is surely not declared and remove the mask
                self.intern(RefsEnum::Mask(o, v))
            }
            RefsEnum::Or(v) => {
                let v = v
                    .iter()
                    .flat_map(|x| {
                        let x = self.local_solve_intern_external(cache, other.with(*x));
                        match &self.nodes[x] {
                            RefsEnum::Or(v) => v.clone(),
                            _ => vec![x].into(),
                        }
                    })
                    .collect();
                self.intern(RefsEnum::Or(v))
            }
            RefsEnum::Invocation(o, i, p) => {
                let o = self.local_solve_intern_external(cache, other.with(*o));
                let p = match p {
                    Arguments::Unknown => Arguments::Unknown,
                    Arguments::Given(p) => {
                        let mut v = vec![];
                        for x in p.deref() {
                            let r = self.local_solve_intern_external(cache, other.with(*x));
                            v.push(r);
                        }
                        let p = v.into_boxed_slice();
                        Arguments::Given(p)
                    }
                };
                self.intern(RefsEnum::Invocation(o, *i, p))
            }
            RefsEnum::ConstructorInvocation(o, p) => {
                // log::trace!("try solve constructor: {:?}", other);
                let i = self.local_solve_intern_external(cache, other.with(*o));
                let p = match p {
                    Arguments::Unknown => Arguments::Unknown,
                    Arguments::Given(p) => {
                        let p = p
                            .deref()
                            .iter()
                            .map(|x| self.local_solve_intern_external(cache, other.with(*x)))
                            .collect();
                        Arguments::Given(p)
                    }
                };
                let r = self.intern(RefsEnum::ConstructorInvocation(i, p));
                assert_ne!(i, r);
                r
            }
            x => {
                let o = x
                    .object()
                    .unwrap_or_else(|| panic!("should have an object {:?}", x));
                let o = self.local_solve_intern_external(cache, other.with(o));
                self.intern(x.with_object(o))
            }
        };
        let r = match self.decls.get(&Declarator::Variable(r)) {
            Some(DeclType::Runtime(b)) => {
                if b.len() == 1 {
                    b[0]
                } else {
                    log::trace!("TODO");
                    b[0] // TODO
                }
            }
            Some(DeclType::Compile(r, _s, _i)) => {
                // log::trace!("solved local variable: {:?}", r);
                // self.solve_intern_external(cache, other.with(r))
                *r
            }
            None => r,
        };
        // TODO handle class statements
        cache.insert(other.rf, r);
        r
    }
}

/// advanced insertions
impl Solver {
    /// copy all references in [`solver`] to current solver
    pub(crate) fn extend<'a>(&mut self, solver: &'a Solver) -> Internalizer<'a> {
        self.extend_map(solver, &mut Default::default(), Default::default())
    }

    pub(crate) fn extend_map<'a>(
        &mut self,
        solver: &'a Solver,
        map: &mut HashMap<usize, usize>,
        map_decls: HashMap<usize, usize>,
    ) -> Internalizer<'a> {
        let mut cached = Internalizer {
            solve: false,
            cache: Default::default(),
            cache_decls: map_decls,
            solver,
        };
        for r in solver.iter_refs() {
            // TODO make it imperative ?
            let r = self.intern_external(map, &mut cached.cache, r);
            match &self.nodes[r] {
                RefsEnum::Primitive(_) => {}
                RefsEnum::Array(o) => {
                    if let RefsEnum::Primitive(_) = &self.nodes[*o] {
                    } else {
                        if r >= self.refs.len() {
                            self.refs.resize(r + 1, false);
                        }
                        self.refs.set(r, true);
                    }
                }
                _ => {
                    if r >= self.refs.len() {
                        self.refs.resize(r + 1, false);
                    }
                    self.refs.set(r, true);
                }
            };
        }
        // no need to extend decls, handled specifically given state
        cached
    }
}

#[derive(Clone)]
struct CountedRefPtr {
    ptr: RefPtr,
    count: usize,
}

impl CountedRefPtr {
    fn new(ptr: RefPtr) -> Self {
        Self { ptr, count: 1 }
    }
}

/// advanced counting insertions
impl Solver {
    /// copy a referencial element from another solver to the current one
    /// also count exected number of unique flatten references
    fn counted_intern_external(
        &mut self,
        cache: &mut HashMap<RefPtr, CountedRefPtr>,
        other: ExplorableRef,
    ) -> CountedRefPtr {
        if let Some(x) = cache.get(&other.rf) {
            assert!(
                self.nodes[x.ptr].similar(other.as_ref()),
                "{:?} ~ {:?}",
                other,
                ExplorableRef {
                    nodes: &self.nodes,
                    rf: x.ptr
                },
            );
            return x.clone();
        }

        let mut rec = |x: usize| self.counted_intern_external(cache, other.with(x));

        let r = match other.as_ref() {
            RefsEnum::Root => CountedRefPtr::new(self.intern(RefsEnum::Root)),
            RefsEnum::MaybeMissing => CountedRefPtr::new(self.intern(RefsEnum::MaybeMissing)),
            RefsEnum::Primitive(i) => CountedRefPtr::new(self.intern(RefsEnum::Primitive(*i))),
            RefsEnum::Mask(o, p) => {
                let o = rec(*o);
                let p = p.iter().map(|&x| rec(x).ptr).collect();
                CountedRefPtr {
                    ptr: self.intern(RefsEnum::Mask(o.ptr, p)),
                    count: o.count,
                }
            }
            RefsEnum::Or(p) => {
                let mut count = 0;
                let p = p
                    .iter()
                    .map(|&x| {
                        let x = rec(x);
                        count += x.count;
                        x.ptr
                    })
                    .collect();

                CountedRefPtr {
                    ptr: self.intern(RefsEnum::Or(p)),
                    count,
                }
            }
            RefsEnum::Invocation(o, i, p) => {
                let o = rec(*o);
                let p = p.map(|x| rec(*x).ptr);

                CountedRefPtr {
                    ptr: self.intern(RefsEnum::Invocation(o.ptr, *i, p)),
                    count: o.count,
                }
            }
            RefsEnum::ConstructorInvocation(i, p) => {
                let i = rec(*i);
                let p = p.map(|x| rec(*x).ptr);
                let ptr = self.intern(RefsEnum::ConstructorInvocation(i.ptr, p));
                assert_ne!(ptr, i.ptr);
                CountedRefPtr {
                    ptr,
                    count: i.count,
                }
            }
            x => {
                let o = x
                    .object()
                    .unwrap_or_else(|| panic!("should have an object {:?}", x));
                let o = rec(o);
                let ptr = self.intern(x.with_object(o.ptr));
                CountedRefPtr {
                    ptr,
                    count: o.count,
                }
            }
        };
        assert!(
            self.nodes[r.ptr].similar(other.as_ref()),
            "{:?} ~ {:?}",
            other.as_ref(),
            self.nodes[r.ptr],
        );
        cache.insert(other.rf, r.clone());
        r
    }

    pub(crate) fn counted_extend<'a>(&mut self, solver: &'a Solver) -> CountedInternalizer<'a> {
        let mut cached = CountedInternalizer {
            count: 0,
            cache: Default::default(),
            solver,
        };
        for r in solver.iter_refs() {
            // TODO make it imperative ?
            let r = self.counted_intern_external(&mut cached.cache, r);
            cached.count += r.count;
            let r = r.ptr;
            if r >= self.refs.len() {
                self.refs.resize(r + 1, false);
            }
            self.refs.set(r, true);
        }
        // no need to extend decls, handled specifically given state
        cached
    }
}

#[derive(Debug, Clone)]
pub struct SolvingResult {
    /// Result of successful matches
    matched: ListSet<RefPtr>,
    /// The reference that we are atempting to solve, it may contain RefsEnum::Mask and RefsEnum::Or
    pub(crate) waiting: Option<RefPtr>,
}

impl SolvingResult {
    pub fn new(result: RefPtr, matched: ListSet<RefPtr>) -> Self {
        Self {
            matched,
            waiting: Some(result),
        }
    }
    pub fn is_matched(&self) -> bool {
        !self.matched.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = &RefPtr> {
        self.matched.iter()
    }
}

impl Default for SolvingResult {
    fn default() -> Self {
        Self {
            waiting: None,
            matched: Default::default(),
        }
    }
}
impl<Node: Eq> FromIterator<Node> for SolvingResult {
    fn from_iter<T: IntoIterator<Item = Node>>(_iter: T) -> Self {
        panic!("should not be possible")
        // let mut r = vec![];
        // for x in iter.into_iter() {
        //     if r.contains(&x) {
        //         r.push(x);
        //     }
        // }
        // Self {
        //     matched: false,
        //     result: todo!(),
        //     digested: todo!(),
        // }
    }
}

struct DebugSolvingResult<'a> {
    pub(crate) nodes: &'a Nodes,
    pub(crate) result: &'a SolvingResult,
}

impl<'a> Debug for DebugSolvingResult<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} := {:?}",
            self.result.waiting.map(|x| self.nodes.with(x)),
            self.result
                .matched
                .iter()
                .map(|&x| self.nodes.with(x))
                .collect::<Vec<_>>()
        )
    }
}

pub struct SolvingAssocTable {
    intern: Vec<Option<SolvingResult>>,
}

impl Index<RefPtr> for SolvingAssocTable {
    type Output = Option<SolvingResult>;

    fn index(&self, index: RefPtr) -> &Self::Output {
        &self.intern[index]
    }
}

impl Default for SolvingAssocTable {
    fn default() -> Self {
        Self {
            intern: Default::default(),
        }
    }
}

impl SolvingAssocTable {
    const N: Option<SolvingResult> = None;
    pub fn get(&self, index: &RefPtr) -> &Option<SolvingResult> {
        if index >= &self.intern.len() {
            return &Self::N;
        }
        &self.intern[*index]
    }
    pub fn insert(&mut self, index: RefPtr, v: SolvingResult) {
        if index >= self.intern.len() {
            self.intern.resize(index + 1, None)
        }
        self.intern[index] = Some(v);
    }
    pub fn len(&self) -> usize {
        self.intern.len()
    }
    pub fn is_empty(&self) -> bool {
        self.intern.is_empty()
    }
}

/// main logic to resolve references
impl Solver {
    /// resolve references in bodies, class declarations, programs and directories
    pub(crate) fn resolve(self, mut cache: SolvingAssocTable) -> (SolvingAssocTable, Solver) {
        // let mut r = self.clone();
        let mut r = Solver {
            nodes: self.nodes.clone(),
            refs: Default::default(),
            decls: self.decls.clone(),
        };

        log::trace!(
            "sd:\n{:?}",
            DebugDecls {
                decls: &self.decls,
                nodes: &self.nodes
            }
        );
        for s in self.iter_refs() {
            // TODO make it imperative ?
            let rr = r.solve_aux(&mut cache, s.rf);
            log::trace!(
                "solve_aux({:?}) produced {:?}",
                self.nodes.with(s.rf),
                DebugSolvingResult {
                    nodes: &r.nodes,
                    result: &rr
                }
            );
            if rr.is_matched() {
                continue;
            }
            if let Some(s) = rr.waiting {
                match &r.nodes[s] {
                    RefsEnum::Primitive(_) => {}
                    _ => {
                        if s >= r.refs.len() {
                            r.refs.resize(s + 1, false);
                        }
                        r.refs.set(s, true);
                    }
                };
            }
        }
        // TODO try better
        (cache, r)
    }

    // pub(crate) fn solving_result(&mut self, refs: &[RefPtr]) -> SolvingResult {
    //     if refs.is_empty() {
    //         panic!()
    //     } else if refs.len() == 1 {
    //         SolvingResult::new(refs[0], refs.iter().cloned().collect())
    //     } else {
    //         let refs = refs.iter().cloned();
    //         let result = self.intern(RefsEnum::Or(refs.clone().collect()));
    //         SolvingResult::new(result, refs.collect())
    //     }
    // }

    // /// flatten Or and filter Masks
    // pub(crate) fn possibilities(&mut self, other: RefPtr) -> Vec<RefPtr> {
    //     let o = &self.nodes[other].clone();
    //     if let RefsEnum::Mask(oo, _) = o {
    //         self.possibilities(*oo)
    //     } else if let RefsEnum::Or(v) = o {
    //         v.iter().flat_map(|&o| self.possibilities(o)).collect()
    //     } else if let Some(o) = o.object() {
    //         self.possibilities(o)
    //             .into_iter()
    //             .map(|o| {
    //                 let x = self.nodes[other].with_object(o);
    //                 self.intern(x)
    //             })
    //             .collect()
    //     } else {
    //         vec![other]
    //     }
    // }

    /// flatten Or and filter Masks
    /// do not create new references
    /// useful to cut short search for declarations,
    /// indeed as we share `self.nodes` with declarations
    /// if we cannot flatten a case we are sure
    /// that there is no corresponding declaration
    pub(crate) fn straight_possibilities(&self, other: RefPtr) -> Vec<RefPtr> {
        self.nodes.straight_possibilities(other)
    }

    /// no internalization needed
    /// not used on blocks, only bodies, declarations and whole programs
    pub(crate) fn solve_aux(
        &mut self,
        cache: &mut SolvingAssocTable,
        other: RefPtr,
    ) -> SolvingResult {
        if let Some(x) = cache.get(&other) {
            log::trace!(
                "solve: {:?} {:?} from cache {:?}",
                other,
                self.nodes.with(other),
                DebugSolvingResult {
                    nodes: &self.nodes,
                    result: x
                }
            );
            return x.clone();
        }

        macro_rules! search {
            ( $($e:expr,  $f:expr;)+ ) => {{
                $(if let Some(r) = (&self.decls).get(&$e).cloned() {
                    $f(r)
                } else )+ {
                    vec![]
                }
            }};
            ( { $d:expr } $($e:expr,  $f:expr;)+ ) => {{
                $(if let Some(r) = (&self.decls).get(&$e).cloned() {
                    $f(r)
                } else )+ {
                    $d
                }
            }};
        }

        log::trace!("solving: {:?} {:?}", other, self.nodes.with(other));

        let decl_type_handling = |r: DeclType<RefPtr>| match r {
            DeclType::Compile(r, r1, r2) => {
                let mut r = vec![r];
                r.extend_from_slice(&r1);
                r.extend_from_slice(&r2);
                r
            }
            DeclType::Runtime(b) => b.to_vec(),
        };

        // let (not_matched, matched) = if let RefsEnum::TypeIdentifier(o, i) = self.nodes[other].clone() {
        //     let possibilities = self.straight_possibilities(o);
        //     let mut not_matched = vec![];
        //     let mut matched = vec![];
        //     for o in possibilities {
        //         let simp = self.intern(RefsEnum::TypeIdentifier(o, i));
        //         search![
        //             { not_matched.push(simp) }
        //             Declarator::Type(simp), |r| {matched.extend(decl_type_handling(r));} ;
        //         ]
        //     }

        //         // .collect();
        //     let matched: ListSet<RefPtr> = possibilities
        //         .into_iter()
        //         .flat_map(|o| {
        //             let simp = self.intern(RefsEnum::TypeIdentifier(o, i));
        //             let mut f = || {not_matched+=1;vec![simp]};
        //             search![
        //                 { f() }
        //                 Declarator::Type(simp), decl_type_handling;
        //             ]
        //         })
        //         .collect();
        //     let matched: ListSet<RefPtr> = matched
        //         .iter()
        //         .flat_map(|&x| {
        //             if x == other {
        //                 vec![x].into()
        //             } else {
        //                 let x = self.solve_aux(cache, x).waiting.unwrap();
        //                 match &self.nodes[x] {
        //                     RefsEnum::Or(v) => v.clone(),
        //                     _ => vec![x].into(),
        //                 }
        //             }
        //         })
        //         .collect();
        //     (not_matched, matched)
        // } else {
        //     let matched = self.straight_possibilities(other);
        //     log::trace!("àà {:?}", matched);
        //     let mut not_matched = 0;
        //     let matched: ListSet<RefPtr> = matched
        //         .into_iter()
        //         .flat_map(|simp| {
        //             log::trace!("&& {:?} {:?}", simp, self.nodes.with(simp));
        //             let mut f = || {not_matched+=1;vec![simp]};
        //             search![
        //                 { f() }
        //                 Declarator::Field(simp), decl_type_handling;
        //                 Declarator::Variable(simp), decl_type_handling;
        //                 Declarator::Type(simp), decl_type_handling;
        //             ]
        //         })
        //         .collect();
        //     log::trace!("== {:?}", matched);
        //     let matched: ListSet<RefPtr> = matched
        //         .iter()
        //         .flat_map(|&x| {
        //             log::trace!("éé {:?} {:?}", x, self.nodes.with(x));
        //             if x == other {
        //                 vec![x].into()
        //             } else {
        //                 let x = self.solve_aux(cache, x).waiting.unwrap();
        //                 match &self.nodes[x] {
        //                     RefsEnum::Or(v) => v.clone(),
        //                     _ => vec![x].into(),
        //                 }
        //             }
        //         })
        //         .collect();
        //     (not_matched, matched)
        // };

        // if matched.len() > not_matched {
        //     let waiting = if matched.len() == 1 {
        //         matched.iter().next().map(|&x|x)
        //     } else {
        //         Some(self.intern(RefsEnum::Or(matched.clone())))
        //     };
        //     let r = SolvingResult { matched, waiting };
        //     log::trace!(
        //         "solved early: {} produced {:?}",
        //         other,
        //         DebugSolvingResult {
        //             nodes: &self.nodes,
        //             result: &r
        //         }
        //     );
        //     cache.insert(other, r.clone());
        //     return r;
        // }

        let r: SolvingResult = match self.nodes[other].clone() {
            RefsEnum::Root => SolvingResult::new(other, vec![other].into()),
            RefsEnum::MaybeMissing => SolvingResult::new(other, vec![other].into()), //if let Some(p) = self.root { p } else { other }),
            RefsEnum::Primitive(_) => SolvingResult::new(other, vec![other].into()),
            RefsEnum::Array(o) => {
                let rr = self.solve_aux(cache, o);
                let r: ListSet<RefPtr> = rr
                    .iter()
                    .map(|o| self.intern(RefsEnum::Array(*o)))
                    // .chain(matched.into_iter())
                    .collect();
                SolvingResult {
                    matched: r,
                    waiting: rr.waiting.map(|x| self.intern(RefsEnum::Array(x))),
                }
            }
            RefsEnum::ArrayAccess(o) => {
                let rr = self.solve_aux(cache, o);
                let r: ListSet<RefPtr> = rr
                    .iter()
                    .map(|o| match &self.nodes[*o] {
                        RefsEnum::Array(x) => *x,
                        _ => self.intern(RefsEnum::ArrayAccess(*o)),
                    })
                    // .chain(matched.into_iter())
                    .collect();
                SolvingResult {
                    matched: r,
                    waiting: rr.waiting.map(|x| match &self.nodes[x] {
                        RefsEnum::Array(x) => *x,
                        _ => self.intern(RefsEnum::ArrayAccess(x)),
                    }),
                }
            }
            RefsEnum::This(o) => {
                let possibilities: ListSet<RefPtr> = self.straight_possibilities(other).into();

                let mut e_matched = vec![];
                let mut e_t_matched = vec![];
                let mut e_not_matched = vec![];

                let mut decl_type_handling2 = |r: DeclType<RefPtr>| match r {
                    DeclType::Compile(r, r1, r2) => {
                        e_t_matched.push(r);
                        e_matched.extend_from_slice(&r1);
                        e_matched.extend_from_slice(&r2);
                    }
                    DeclType::Runtime(b) => e_matched.extend_from_slice(&b),
                };

                possibilities.iter().for_each(|&x| {
                    search![
                        { e_not_matched.push(x) }
                        Declarator::Type(x), decl_type_handling2;
                    ]
                });

                if !e_matched.is_empty() || !e_t_matched.is_empty() {
                    e_matched.extend_from_slice(&e_not_matched);
                    let x: ListSet<_> = e_matched.into();
                    let x: ListSet<_> = x
                        .iter()
                        .flat_map(|&x| {
                            log::trace!("éé {:?} {:?}", x, self.nodes.with(x));
                            if x == other {
                                vec![x].into()
                            } else {
                                let x = self.solve_aux(cache, x).waiting.unwrap();
                                match &self.nodes[x] {
                                    RefsEnum::Or(v) => v.clone(),
                                    _ => vec![x].into(),
                                }
                            }
                        })
                        .collect();
                    let waiting = if x.len() == 1 {
                        x.iter().next().copied()
                    } else {
                        Some(self.intern(RefsEnum::Or(x.clone())))
                    };
                    let r = SolvingResult {
                        matched: e_t_matched.into_iter().chain(x.into_iter()).collect(),
                        waiting,
                    };
                    log::trace!(
                        "solved early: {} produced {:?}",
                        other,
                        DebugSolvingResult {
                            nodes: &self.nodes,
                            result: &r
                        }
                    );
                    cache.insert(other, r.clone());
                    return r;
                }

                let matched_o = self.solve_aux(cache, o);

                let ext: ListSet<RefPtr> = matched_o
                    .iter()
                    .flat_map(|&o| match &self.nodes[o] {
                        RefsEnum::Or(v) => v.clone(),
                        _ => vec![o].into(),
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|o| self.intern(RefsEnum::This(o)))
                    .collect();

                let waiting = matched_o.waiting;
                let matched: ListSet<RefPtr> = ext
                    .iter()
                    .flat_map(|r| {
                        let r = self.straight_possibilities(*r);
                        r.into_iter().flat_map(|r| {
                            search![
                                Declarator::Type(r), decl_type_handling;
                            ]
                        })
                    })
                    .collect();

                let waiting = if !matched.is_empty() || ext.is_empty() {
                    waiting
                } else {
                    let v: ListSet<_> = matched_o
                        .matched
                        .into_iter()
                        .chain(waiting.into_iter())
                        .collect();
                    if v.len() == 1 {
                        log::trace!("4");
                        v.iter().next().cloned()
                    } else {
                        log::trace!("5");
                        let v: ListSet<_> = v
                            .into_iter()
                            .flat_map(|x| match &self.nodes[x] {
                                RefsEnum::Or(v) => v.clone(),
                                _ => vec![x].into(),
                            })
                            .collect();
                        let w: ListSet<_> = v
                            .iter()
                            .filter(|&&x| {
                                if let RefsEnum::TypeIdentifier(_, _) = &self.nodes[x] {
                                    !self.solve_aux(cache, x).is_matched()
                                } else if let RefsEnum::ScopedIdentifier(_, _) = &self.nodes[x] {
                                    !self.solve_aux(cache, x).is_matched()
                                } else {
                                    true
                                }
                            })
                            .cloned()
                            .collect();
                        if w.is_empty() {
                            Some(self.intern(RefsEnum::Or(v)))
                        } else {
                            Some(self.intern(RefsEnum::Or(w)))
                        }
                    }
                }
                .map(|o| self.intern(RefsEnum::This(o)));

                SolvingResult { matched, waiting }
            }
            RefsEnum::Super(o) => {
                let matched_o = self.solve_aux(cache, o);

                let ext: ListSet<RefPtr> = matched_o
                    .iter()
                    .map(|o| match &self.nodes[*o] {
                        // TODO check
                        RefsEnum::Mask(o, _) => *o,
                        _ => *o,
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|o| self.intern(RefsEnum::Super(o)))
                    .collect();

                let waiting = matched_o.waiting;
                let matched: ListSet<RefPtr> = ext
                    .iter()
                    .flat_map(|r| {
                        let r = self.straight_possibilities(*r);
                        r.into_iter().flat_map(|r| {
                            search![
                                Declarator::Type(r), |r: DeclType<RefPtr>| match r {
                                    DeclType::Compile(_, r1, r2) => {
                                        let mut r = vec![];
                                        r.extend_from_slice(&r1);
                                        r.extend_from_slice(&r2); // TODO check if is used
                                        r
                                    }
                                    DeclType::Runtime(b) => b.to_vec(),
                                };
                            ]
                        })
                    })
                    .collect();

                let waiting = if !matched.is_empty() || ext.is_empty() {
                    waiting
                } else {
                    let mut v: Vec<_> = matched_o.matched.into_iter().collect();
                    if let Some(w) = waiting {
                        v.push(w);
                    }
                    let v = v
                        .into_iter()
                        .flat_map(|x| match &self.nodes[x] {
                            RefsEnum::Or(v) => v.clone(),
                            _ => vec![x].into(),
                        })
                        .collect();
                    Some(self.intern(RefsEnum::Or(v)))
                }
                .map(|o| match &self.nodes[o] {
                    RefsEnum::Mask(o, _) => *o,
                    _ => o,
                })
                .map(|o| self.intern(RefsEnum::Super(o)));

                SolvingResult { matched, waiting }
            }
            RefsEnum::Mask(o, v) => {
                log::trace!("solving mask {:?}", other);
                let mut v_matched = vec![];
                let mut v_waiting = vec![];
                v.iter().for_each(|x| {
                    assert_ne!(other, *x);
                    log::trace!("mask {:?}", *x);
                    let rr = self.solve_aux(cache, *x);
                    if let Some(r) = rr.waiting {
                        v_waiting.push(r);
                    }
                    v_matched.extend(rr.matched);
                });

                let matched_o = self.solve_aux(cache, o);

                if !v_matched.is_empty() {
                    SolvingResult {
                        matched: v_matched.into(),
                        waiting: matched_o.waiting.map(|o| {
                            self.intern(RefsEnum::Mask(o, v_waiting.clone().into_boxed_slice()))
                        }),
                    }
                } else {
                    SolvingResult {
                        matched: matched_o.matched,
                        waiting: matched_o
                            .waiting
                            .map(|o| self.intern(RefsEnum::Mask(o, v_waiting.into_boxed_slice()))),
                    }
                }
            }
            RefsEnum::Or(v) => {
                // TODO if one case is matched, we should be able to ditch the rest
                // NOTE removing cases should be done later (on the RefsEnum containing the Or)
                log::trace!("solving Or: {:?} {:?}", other, self.nodes.with(other));
                let mut matched = vec![];
                let mut waiting = vec![];
                v.iter().for_each(|&x| {
                    assert_ne!(other, x);
                    log::trace!("or {:?}", x);
                    let rr = self.solve_aux(cache, x);
                    log::trace!("after or {:?}", x);
                    if let Some(r) = rr.waiting {
                        // waiting.push(r);
                        match &self.nodes[r] {
                            RefsEnum::Or(r) => {
                                waiting.extend(r.iter());
                            }
                            _ => waiting.push(r),
                        }
                    }
                    matched.extend(rr.matched.iter().flat_map(|&o| match &self.nodes[o] {
                        RefsEnum::Or(o) => o.iter().cloned().collect::<Vec<_>>(),
                        _ => vec![o].into(),
                    }));
                });
                let waiting: ListSet<_> = waiting.into();
                let r = if waiting.is_empty() {
                    SolvingResult {
                        matched: matched.into(),
                        waiting: Some(self.intern(RefsEnum::Or(waiting))),
                    }
                } else if waiting.len() == 1 {
                    SolvingResult {
                        matched: matched.into(),
                        waiting: Some(*waiting.iter().next().unwrap()),
                    }
                } else {
                    SolvingResult {
                        matched: matched.into(),
                        waiting: Some(self.intern(RefsEnum::Or(waiting))),
                    }
                };

                log::trace!(
                    "solved Or: {} produced {:?}",
                    other,
                    DebugSolvingResult {
                        nodes: &self.nodes,
                        result: &r
                    }
                );
                r
            }
            RefsEnum::TypeIdentifier(oo, i) => {
                log::trace!("solving scoped type: {:?}", other);

                let possibilities: ListSet<RefPtr> = self.straight_possibilities(other).into();

                let mut e_matched = vec![];
                let mut e_t_matched = vec![];
                let mut e_not_matched = vec![];

                let mut decl_type_handling2 = |r: DeclType<RefPtr>| match r {
                    DeclType::Compile(r, r1, r2) => {
                        e_t_matched.push(r);
                        e_matched.extend_from_slice(&r1);
                        e_matched.extend_from_slice(&r2);
                    }
                    DeclType::Runtime(b) => e_matched.extend_from_slice(&b),
                };

                possibilities.iter().for_each(|&x| {
                    search![
                        { e_not_matched.push(x) }
                        Declarator::Type(x), decl_type_handling2;
                    ]
                });

                if !e_matched.is_empty() || !e_t_matched.is_empty() {
                    e_matched.extend_from_slice(&e_not_matched);
                    let x: ListSet<_> = e_matched.into();
                    let x: ListSet<_> = x
                        .iter()
                        .flat_map(|&x| {
                            log::trace!("&& {:?} {:?}", x, self.nodes.with(x));
                            if x == other {
                                vec![x].into()
                            } else {
                                let x = self.solve_aux(cache, x).waiting.unwrap();
                                match &self.nodes[x] {
                                    RefsEnum::Or(v) => v.clone(),
                                    _ => vec![x].into(),
                                }
                            }
                        })
                        .collect();
                    let waiting = if x.len() == 1 {
                        x.iter().next().copied()
                    } else {
                        Some(self.intern(RefsEnum::Or(x.clone())))
                    };
                    let r = SolvingResult {
                        matched: e_t_matched.into_iter().chain(x.into_iter()).collect(),
                        waiting,
                    };
                    log::trace!(
                        "solved early: {} produced {:?}",
                        other,
                        DebugSolvingResult {
                            nodes: &self.nodes,
                            result: &r
                        }
                    );
                    cache.insert(other, r.clone());
                    return r;
                }

                // if matched return all matched and unmatched
                // if unmatched continue with the rest

                let matched_o = self.solve_aux(cache, oo);

                log::trace!(
                    "solving sco t: {} with {:?}",
                    other,
                    DebugSolvingResult {
                        nodes: &self.nodes,
                        result: &matched_o
                    }
                );

                fn handle(
                    sss: &mut Solver,
                    cache: &mut SolvingAssocTable,
                    oo: usize,
                    &o: &usize,
                    i: LabelPtr,
                ) -> Option<usize> {
                    if sss.is_mm(oo) {
                        if !sss.is_root(o) && sss.is_package(o) && sss.is_package_token(i) {
                            return None;
                        }
                    }
                    let o_enum = &sss.nodes[o];

                    // // /.A -> None
                    // if let RefsEnum::Root= o_enum && !sss.is_package_token(i) {
                    //     return None
                    // }

                    // x[].length -> int
                    if let RefsEnum::Array(_)= o_enum && sss.is_length_token(i) {
                        return Some(sss.intern(RefsEnum::Primitive(Primitive::Int)))
                    }

                    let m = if let RefsEnum::Mask(_, x) = o_enum {
                        x.clone()
                    } else {
                        return Some(sss.intern(RefsEnum::TypeIdentifier(o, i)));
                    };

                    for m in m.iter() {
                        let no_mask = sss.no_mask(*m);
                        if sss.intern(RefsEnum::Root) == no_mask && !sss.is_package_token(i) {
                            continue;
                        }
                        let no_mask = sss.intern(RefsEnum::TypeIdentifier(no_mask, i));
                        let x = sss.solve_aux(cache, no_mask);
                        log::trace!("for {:?} choose between:", no_mask);
                        x.iter().for_each(|x| {
                            let x = sss.nodes.with(*x);
                            log::trace!("@:; {:?}", x);
                        });
                        if !x.is_matched() {
                            // TODO check, was using is_empty
                            let x = *x.iter().next().unwrap();
                            if x != no_mask {
                                return Some(x);
                            }
                        }
                    }
                    Some(sss.intern(RefsEnum::TypeIdentifier(o, i)))
                }
                let ext: ListSet<RefPtr> = matched_o
                    .iter()
                    .filter_map(|o| handle(self, cache, oo, o, i))
                    .collect();
                log::trace!(
                    "solving sco t: {} with {:?}",
                    other,
                    ext.iter().map(|&x| self.nodes.with(x)).collect::<Vec<_>>()
                );

                let mut t_matched = vec![];
                let waiting = matched_o.waiting;
                let matched: ListSet<RefPtr> =
                    ext.iter()
                        .flat_map(|r| {
                            let r = self.straight_possibilities(*r);
                            r.into_iter()
                                .flat_map(|r| {
                                    search![
                                        Declarator::Type(r), |r: DeclType<RefPtr>| match r {
                                            DeclType::Compile(r, r1, r2) => {
                                                t_matched.extend(match &self.nodes[r] {
                                                    RefsEnum::Or(v) => v.clone(),
                                                    _ => vec![r].into(),
                                                });
                                                let mut r = vec![];
                                                r.extend_from_slice(&r1);
                                                r.extend_from_slice(&r2);
                                                r
                                            }
                                            DeclType::Runtime(b) => b.to_vec(),
                                        };
                                    ]
                                    .into_iter()
                                    .flat_map(|x| match &self.nodes[x] {
                                        RefsEnum::Or(v) => v.clone(),
                                        _ => vec![x].into(),
                                    })
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect();
                let mut matched: ListSet<RefPtr> = matched
                    .iter()
                    .flat_map(|&x| {
                        let x = self.solve_aux(cache, x).waiting.unwrap();
                        match &self.nodes[x] {
                            RefsEnum::Or(v) => v.clone(),
                            _ => vec![x].into(),
                        }
                    })
                    .collect();
                let t_matched: ListSet<RefPtr> = t_matched
                    .iter()
                    .flat_map(|&x| {
                        let x = self.solve_aux(cache, x).waiting.unwrap();
                        match &self.nodes[x] {
                            RefsEnum::Or(v) => v.clone(),
                            _ => vec![x].into(),
                        }
                    })
                    .collect();

                let waiting = if !t_matched.is_empty() {
                    log::trace!("0");
                    let w = t_matched.iter().next().cloned();
                    matched = matched.into_iter().chain(t_matched.into_iter()).collect();
                    w
                } else if !matched.is_empty() {
                    log::trace!("2");
                    let w = if matched.len() == 1 {
                        Some(*matched.iter().next().unwrap())
                    } else {
                        Some(self.intern(RefsEnum::Or(matched.clone())))
                    };
                    matched = matched.into_iter().chain(t_matched.into_iter()).collect();
                    w
                } else if ext.is_empty() {
                    log::trace!("3");
                    waiting.and_then(|o| handle(self, cache, oo, &o, i))
                } else {
                    let v: ListSet<_> = matched_o
                        .matched
                        .into_iter()
                        .chain(waiting.into_iter())
                        .collect();
                    if v.len() == 1 {
                        log::trace!("4");
                        v.iter().next().cloned()
                    } else {
                        let v: ListSet<_> = v
                            .into_iter()
                            .flat_map(|x| match &self.nodes[x] {
                                RefsEnum::Or(v) => v.clone(),
                                _ => vec![x].into(),
                            })
                            .collect();
                        let w: ListSet<_> = v
                            .iter()
                            .filter(|&&x| {
                                if let RefsEnum::TypeIdentifier(_, _) = &self.nodes[x] {
                                    !self.solve_aux(cache, x).is_matched()
                                } else if let RefsEnum::ScopedIdentifier(_, _) = &self.nodes[x] {
                                    !self.solve_aux(cache, x).is_matched()
                                } else {
                                    true
                                }
                            })
                            .cloned()
                            .collect();
                        if w.is_empty() {
                            Some(self.intern(RefsEnum::Or(v)))
                        } else {
                            Some(self.intern(RefsEnum::Or(w)))
                        }
                    }
                    .and_then(|o| handle(self, cache, oo, &o, i))
                };

                let r = SolvingResult { matched, waiting };

                log::trace!(
                    "solved scoped type: {} produced {:?}",
                    other,
                    DebugSolvingResult {
                        nodes: &self.nodes,
                        result: &r
                    }
                );
                r
            }
            RefsEnum::ScopedIdentifier(oo, i) => {
                log::trace!("solving scoped id: {:?}", other);

                let possibilities: ListSet<RefPtr> = self.straight_possibilities(other).into();

                let mut e_matched = vec![];
                let mut e_t_matched = vec![];
                let mut e_not_matched = vec![];

                let mut decl_type_handling2 = |r: DeclType<RefPtr>| match r {
                    DeclType::Compile(r, r1, r2) => {
                        e_t_matched.push(r);
                        e_matched.extend_from_slice(&r1);
                        e_matched.extend_from_slice(&r2);
                    }
                    DeclType::Runtime(b) => e_matched.extend_from_slice(&b),
                };

                possibilities.iter().for_each(|&x| {
                    search![
                        { e_not_matched.push(x) }
                        Declarator::Field(x), decl_type_handling2;
                        Declarator::Variable(x), decl_type_handling2;
                        Declarator::Type(x), decl_type_handling2;
                    ]
                });

                if !e_matched.is_empty() || !e_t_matched.is_empty() {
                    e_matched.extend_from_slice(&e_not_matched);
                    let x: ListSet<_> = e_matched.into();
                    let x: ListSet<_> = x
                        .iter()
                        .flat_map(|&x| {
                            log::trace!("éé {:?} {:?}", x, self.nodes.with(x));
                            if x == other {
                                vec![x].into()
                            } else {
                                let x = self.solve_aux(cache, x).waiting.unwrap();
                                match &self.nodes[x] {
                                    RefsEnum::Or(v) => v.clone(),
                                    _ => vec![x].into(),
                                }
                            }
                        })
                        .collect();
                    let waiting = if x.len() == 1 {
                        x.iter().next().copied()
                    } else {
                        Some(self.intern(RefsEnum::Or(x.clone())))
                    };
                    let r = SolvingResult {
                        matched: e_t_matched.into_iter().chain(x.into_iter()).collect(),
                        waiting,
                    };
                    log::trace!(
                        "solved early: {} produced {:?}",
                        other,
                        DebugSolvingResult {
                            nodes: &self.nodes,
                            result: &r
                        }
                    );
                    cache.insert(other, r.clone());
                    return r;
                }

                let matched_o = self.solve_aux(cache, oo);

                log::trace!(
                    "solving sco i: {} with {:?}",
                    other,
                    DebugSolvingResult {
                        nodes: &self.nodes,
                        result: &matched_o
                    }
                );

                fn handle(
                    sss: &mut Solver,
                    cache: &mut SolvingAssocTable,
                    oo: usize,
                    &o: &usize,
                    i: LabelPtr,
                ) -> Option<usize> {
                    if sss.is_mm(oo) {
                        if !sss.is_root(o) && sss.is_package(o) && sss.is_package_token(i) {
                            return None;
                        }
                    }
                    let o_enum = &sss.nodes[o];

                    // // /.A -> None
                    // if let RefsEnum::Root= o_enum && !sss.is_package_token(i) {
                    //     return None
                    // }

                    // x[].length -> int
                    if let RefsEnum::Array(_)= o_enum && sss.is_length_token(i) {
                        return Some(sss.intern(RefsEnum::Primitive(Primitive::Int)))
                    }

                    let m = if let RefsEnum::Mask(_, x) = o_enum {
                        x.clone()
                    } else {
                        return Some(sss.intern(RefsEnum::ScopedIdentifier(o, i)));
                    };

                    for m in m.iter() {
                        let no_mask = sss.no_mask(*m);
                        if sss.intern(RefsEnum::Root) == no_mask && !sss.is_package_token(i) {
                            continue;
                        }
                        let no_mask = sss.intern(RefsEnum::ScopedIdentifier(no_mask, i));
                        let x = sss.solve_aux(cache, no_mask);
                        log::trace!("for {:?} choose between:", no_mask);
                        x.iter().for_each(|x| {
                            let x = sss.nodes.with(*x);
                            log::trace!("@:; {:?}", x);
                        });
                        if !x.is_matched() {
                            // TODO check, was using is_empty
                            let x = *x.iter().next().unwrap();
                            if x != no_mask {
                                return Some(x);
                            }
                        }
                    }
                    Some(sss.intern(RefsEnum::ScopedIdentifier(o, i)))
                }

                let ext: ListSet<RefPtr> = matched_o
                    .iter()
                    .filter_map(|o| handle(self, cache, oo, o, i))
                    .collect();
                log::trace!(
                    "solving sco i: {} with {:?}",
                    other,
                    ext.iter().map(|&x| self.nodes.with(x)).collect::<Vec<_>>()
                );

                let mut t_matched = vec![];
                let waiting = matched_o.waiting;
                let matched: ListSet<RefPtr> =
                    ext.iter()
                        .flat_map(|r| {
                            let r = self.straight_possibilities(*r);
                            r.into_iter()
                                .flat_map(|r| {
                                    search![
                                        Declarator::Field(r), decl_type_handling;
                                        Declarator::Variable(r), decl_type_handling;
                                        Declarator::Type(r), |r: DeclType<RefPtr>| match r {
                                            DeclType::Compile(r, r1, r2) => {
                                                t_matched.extend(match &self.nodes[r] {
                                                    RefsEnum::Or(v) => v.clone(),
                                                    _ => vec![r].into(),
                                                });
                                                let mut r = vec![];
                                                r.extend_from_slice(&r1);
                                                r.extend_from_slice(&r2);
                                                r
                                            }
                                            DeclType::Runtime(b) => b.to_vec(),
                                        };
                                    ]
                                    .into_iter()
                                    .flat_map(|x| match &self.nodes[x] {
                                        RefsEnum::Or(v) => v.clone(),
                                        _ => vec![x].into(),
                                    })
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect();
                let mut matched: ListSet<RefPtr> = matched
                    .iter()
                    .flat_map(|&x| {
                        let x = self.solve_aux(cache, x).waiting.unwrap();
                        match &self.nodes[x] {
                            RefsEnum::Or(v) => v.clone(),
                            _ => vec![x].into(),
                        }
                    })
                    .collect();
                let t_matched: ListSet<RefPtr> = t_matched
                    .iter()
                    .flat_map(|&x| {
                        let x = self.solve_aux(cache, x).waiting.unwrap();
                        match &self.nodes[x] {
                            RefsEnum::Or(v) => v.clone(),
                            _ => vec![x].into(),
                        }
                    })
                    .collect();
                let waiting = if !t_matched.is_empty() {
                    log::trace!("0");
                    let w = t_matched.iter().next().cloned();
                    matched = matched.into_iter().chain(t_matched.into_iter()).collect();
                    w
                } else if !matched.is_empty() {
                    log::trace!("2");
                    let w = if matched.len() == 1 {
                        Some(*matched.iter().next().unwrap())
                    } else {
                        Some(self.intern(RefsEnum::Or(matched.clone())))
                    };
                    matched = matched.into_iter().chain(t_matched.into_iter()).collect();
                    w
                } else if ext.is_empty() {
                    log::trace!("3");
                    waiting.and_then(|o| handle(self, cache, oo, &o, i))
                } else {
                    let v: ListSet<_> = matched_o
                        .matched
                        .into_iter()
                        .chain(waiting.into_iter())
                        .collect();
                    if v.len() == 1 {
                        log::trace!("4");
                        v.iter().next().cloned()
                    } else {
                        log::trace!("5");
                        let v: ListSet<_> = v
                            .into_iter()
                            .flat_map(|x| match &self.nodes[x] {
                                RefsEnum::Or(v) => v.clone(),
                                _ => vec![x].into(),
                            })
                            .collect();
                        let w: ListSet<_> = v
                            .iter()
                            .filter(|&&x| {
                                if let RefsEnum::TypeIdentifier(_, _) = &self.nodes[x] {
                                    !self.solve_aux(cache, x).is_matched()
                                } else if let RefsEnum::ScopedIdentifier(_, _) = &self.nodes[x] {
                                    !self.solve_aux(cache, x).is_matched()
                                } else {
                                    true
                                }
                            })
                            .cloned()
                            .collect();
                        if w.is_empty() {
                            Some(self.intern(RefsEnum::Or(v)))
                        } else {
                            Some(self.intern(RefsEnum::Or(w)))
                        }
                    }
                    .and_then(|o| handle(self, cache, oo, &o, i))
                };

                let r = SolvingResult { matched, waiting };

                log::trace!(
                    "solved scoped id: {} produced {:?}",
                    other,
                    DebugSolvingResult {
                        nodes: &self.nodes,
                        result: &r
                    }
                );
                r
            }
            RefsEnum::MethodReference(o, i) => {
                let matched_o = self.solve_aux(cache, o);

                let ext: ListSet<RefPtr> = matched_o
                    .iter()
                    .map(|&o| self.intern(RefsEnum::MethodReference(o, i)))
                    .collect();

                let waiting = matched_o.waiting;
                let matched: ListSet<RefPtr> = ext
                    .iter()
                    .flat_map(|r| {
                        let r = self.straight_possibilities(*r);
                        r.into_iter().flat_map(|r| {
                            search![
                                Declarator::Type(r), decl_type_handling;
                            ]
                        })
                    })
                    .collect();

                let waiting = if !matched.is_empty() || ext.is_empty() {
                    waiting
                } else {
                    let v: ListSet<_> = matched_o
                        .matched
                        .into_iter()
                        .chain(waiting.into_iter())
                        .collect();
                    if v.len() == 1 {
                        log::trace!("4");
                        v.iter().next().cloned()
                    } else {
                        log::trace!("5");
                        let v: ListSet<_> = v
                            .into_iter()
                            .flat_map(|x| match &self.nodes[x] {
                                RefsEnum::Or(v) => v.clone(),
                                _ => vec![x].into(),
                            })
                            .collect();
                        let w: ListSet<_> = v
                            .iter()
                            .filter(|&&x| {
                                if let RefsEnum::TypeIdentifier(_, _) = &self.nodes[x] {
                                    !self.solve_aux(cache, x).is_matched()
                                } else if let RefsEnum::ScopedIdentifier(_, _) = &self.nodes[x] {
                                    !self.solve_aux(cache, x).is_matched()
                                } else {
                                    true
                                }
                            })
                            .cloned()
                            .collect();
                        if w.is_empty() {
                            Some(self.intern(RefsEnum::Or(v)))
                        } else {
                            Some(self.intern(RefsEnum::Or(w)))
                        }
                    }
                }
                .map(|o| self.intern(RefsEnum::MethodReference(o, i)));

                SolvingResult { matched, waiting }
            }
            RefsEnum::ConstructorReference(o) => {
                let matched_o = self.solve_aux(cache, o);

                let ext: ListSet<RefPtr> = matched_o
                    .iter()
                    .map(|&o| self.intern(RefsEnum::ConstructorReference(o)))
                    .collect();

                let waiting = matched_o.waiting;
                let matched: ListSet<RefPtr> = ext
                    .iter()
                    .flat_map(|r| {
                        let r = self.straight_possibilities(*r);
                        r.into_iter().flat_map(|r| {
                            search![
                                Declarator::Type(r), decl_type_handling;
                            ]
                        })
                    })
                    .collect();

                let waiting = if !matched.is_empty() || ext.is_empty() {
                    waiting
                } else {
                    let v: ListSet<_> = matched_o
                        .matched
                        .into_iter()
                        .chain(waiting.into_iter())
                        .collect();
                    if v.len() == 1 {
                        log::trace!("4");
                        v.iter().next().cloned()
                    } else {
                        log::trace!("5");
                        let v: ListSet<_> = v
                            .into_iter()
                            .flat_map(|x| match &self.nodes[x] {
                                RefsEnum::Or(v) => v.clone(),
                                _ => vec![x].into(),
                            })
                            .collect();
                        let w: ListSet<_> = v
                            .iter()
                            .filter(|&&x| {
                                if let RefsEnum::TypeIdentifier(_, _) = &self.nodes[x] {
                                    !self.solve_aux(cache, x).is_matched()
                                } else if let RefsEnum::ScopedIdentifier(_, _) = &self.nodes[x] {
                                    !self.solve_aux(cache, x).is_matched()
                                } else {
                                    true
                                }
                            })
                            .cloned()
                            .collect();
                        if w.is_empty() {
                            Some(self.intern(RefsEnum::Or(v)))
                        } else {
                            Some(self.intern(RefsEnum::Or(w)))
                        }
                    }
                }
                .map(|o| self.intern(RefsEnum::ConstructorReference(o)));

                SolvingResult { matched, waiting }
            }
            RefsEnum::Invocation(o, i, p) => {
                let matched_o = self.solve_aux(cache, o);

                log::trace!(
                    "solving invo: {} with {:?}",
                    other,
                    DebugSolvingResult {
                        nodes: &self.nodes,
                        result: &matched_o
                    }
                );

                let matched_p = p.map(|&p| {
                    let r = self.solve_aux(cache, p);
                    let x = r.waiting.unwrap();
                    let b = match &self.nodes[x] {
                        RefsEnum::Or(v) => v.is_empty(),
                        _ => false,
                    };
                    if b {
                        self.intern(RefsEnum::Or(r.matched))
                    } else {
                        x
                    }
                });

                let ext: ListSet<RefPtr> = matched_o
                    .iter()
                    .map(|&o| self.intern(RefsEnum::Invocation(o, i, matched_p.clone())))
                    .collect();

                let waiting = matched_o.waiting;
                let matched: ListSet<_> = ext
                    .iter()
                    .flat_map(|r| {
                        let r = self.straight_possibilities(*r);
                        r.into_iter().flat_map(|r| {
                            search![
                                Declarator::Executable(r), decl_type_handling;
                            ]
                        })
                    })
                    .collect();
                let mut matched: ListSet<RefPtr> = matched
                    .iter()
                    .flat_map(|&x| {
                        let x = self.solve_aux(cache, x).waiting.unwrap();
                        match &self.nodes[x] {
                            RefsEnum::Or(v) => v.clone(),
                            _ => vec![x].into(),
                        }
                    })
                    .collect();

                let waiting = if !matched.is_empty() {
                    log::trace!("2");
                    let w = if matched.len() == 1 {
                        Some(*matched.iter().next().unwrap())
                    } else {
                        Some(self.intern(RefsEnum::Or(matched.clone())))
                    };
                    matched = matched.into_iter().collect();
                    w
                } else if ext.is_empty() {
                    log::trace!("3");
                    waiting.map(|o| self.intern(RefsEnum::Invocation(o, i, matched_p)))
                } else {
                    let v: ListSet<_> = matched_o
                        .matched
                        .into_iter()
                        .chain(waiting.into_iter())
                        .collect();
                    if v.len() == 1 {
                        log::trace!("4");
                        v.iter().next().cloned()
                    } else {
                        log::trace!("5");
                        let v: ListSet<_> = v
                            .into_iter()
                            .flat_map(|x| match &self.nodes[x] {
                                RefsEnum::Or(v) => v.clone(),
                                _ => vec![x].into(),
                            })
                            .collect();
                        let w: ListSet<_> = v
                            .iter()
                            .filter(|&&x| {
                                if let RefsEnum::TypeIdentifier(_, _) = &self.nodes[x] {
                                    !self.solve_aux(cache, x).is_matched()
                                } else if let RefsEnum::ScopedIdentifier(_, _) = &self.nodes[x] {
                                    !self.solve_aux(cache, x).is_matched()
                                } else {
                                    true
                                }
                            })
                            .cloned()
                            .collect();
                        if w.is_empty() {
                            Some(self.intern(RefsEnum::Or(v)))
                        } else {
                            Some(self.intern(RefsEnum::Or(w)))
                        }
                    }
                    .map(|o| self.intern(RefsEnum::Invocation(o, i, matched_p)))
                };

                SolvingResult { matched, waiting }
            }
            RefsEnum::ConstructorInvocation(o, p) => {
                let matched_o = self.solve_aux(cache, o);

                let matched_p = p.map(|&p| {
                    let r = self.solve_aux(cache, p);
                    let x = r.waiting.unwrap();
                    let b = match &self.nodes[x] {
                        RefsEnum::Or(v) => v.is_empty(),
                        _ => false,
                    };
                    if b && r.matched.len() == 1 {
                        *r.matched.iter().next().unwrap()
                    } else if b {
                        self.intern(RefsEnum::Or(r.matched))
                    } else {
                        x
                    }
                });

                let ext: ListSet<RefPtr> = matched_o
                    .iter()
                    .map(|&o| self.intern(RefsEnum::ConstructorInvocation(o, matched_p.clone())))
                    .collect();

                let waiting = matched_o.waiting;
                let matched: ListSet<RefPtr> = ext
                    .iter()
                    .flat_map(|r| {
                        let r = self.straight_possibilities(*r);
                        r.into_iter().flat_map(|r| {
                            search![
                                Declarator::Executable(r), decl_type_handling;
                            ]
                        })
                    })
                    .collect();
                let mut matched: ListSet<RefPtr> = matched
                    .iter()
                    .flat_map(|&x| {
                        let x = self.solve_aux(cache, x).waiting.unwrap();
                        match &self.nodes[x] {
                            RefsEnum::Or(v) => v.clone(),
                            _ => vec![x].into(),
                        }
                    })
                    .collect();

                let waiting = if !matched.is_empty() {
                    log::trace!("2");
                    let w = if matched.len() == 1 {
                        Some(*matched.iter().next().unwrap())
                    } else {
                        Some(self.intern(RefsEnum::Or(matched.clone())))
                    };
                    matched = matched.into_iter().collect();
                    w
                } else if ext.is_empty() {
                    log::trace!("3");
                    waiting.map(|o| self.intern(RefsEnum::ConstructorInvocation(o, matched_p)))
                } else {
                    let v: ListSet<_> = matched_o
                        .matched
                        .into_iter()
                        .chain(waiting.into_iter())
                        .collect();
                    if v.len() == 1 {
                        log::trace!("4");
                        v.iter().next().cloned()
                    } else {
                        let v: ListSet<_> = v
                            .into_iter()
                            .flat_map(|x| match &self.nodes[x] {
                                RefsEnum::Or(v) => v.clone(),
                                _ => vec![x].into(),
                            })
                            .collect();
                        let w: ListSet<_> = v
                            .iter()
                            .filter(|&&x| {
                                if let RefsEnum::TypeIdentifier(_, _) = &self.nodes[x] {
                                    !self.solve_aux(cache, x).is_matched()
                                } else if let RefsEnum::ScopedIdentifier(_, _) = &self.nodes[x] {
                                    !self.solve_aux(cache, x).is_matched()
                                } else {
                                    true
                                }
                            })
                            .cloned()
                            .collect();
                        if w.is_empty() {
                            Some(self.intern(RefsEnum::Or(v)))
                        } else {
                            Some(self.intern(RefsEnum::Or(w)))
                        }
                    }
                    .map(|o| self.intern(RefsEnum::ConstructorInvocation(o, matched_p)))
                };

                SolvingResult { matched, waiting }
            }
        };

        log::trace!(
            "solved: {} into {:?}",
            other,
            DebugSolvingResult {
                nodes: &self.nodes,
                result: &r
            }
        );
        cache.insert(other, r.clone());
        r
    }
}

/// internal utilities to take decisions
impl Solver {
    fn is_length_token(&self, id: LabelPtr) -> bool {
        id.as_ref().to_usize() == 0 // TODO verify/model statically
    }
    fn is_package_token(&self, id: LabelPtr) -> bool {
        let f = id.format();
        IdentifierFormat::FlatCase.eq(&f)
            || IdentifierFormat::LowerCamelCase.eq(&f)
            || IdentifierFormat::SnakeCase.eq(&f) // not sure about this one
    }
    fn is_package(&self, n: RefPtr) -> bool {
        let r = match &self.nodes[n] {
            RefsEnum::Root => return true,
            RefsEnum::Array(_) => return false,
            RefsEnum::ScopedIdentifier(o, i) => {
                if self.is_package_token(*i) {
                    *o
                } else {
                    return false;
                }
            }
            RefsEnum::Mask(o, _) => *o,
            _ => return false,
        };
        self.is_package(r)
    }
    fn is_root(&self, n: RefPtr) -> bool {
        let r = match &self.nodes[n] {
            RefsEnum::Root => return true,
            RefsEnum::Mask(o, _) => *o,
            _ => return false,
        };
        self.is_root(r)
    }
    fn is_mm(&self, n: RefPtr) -> bool {
        let r = match &self.nodes[n] {
            RefsEnum::MaybeMissing => return true,
            RefsEnum::Mask(o, _) => *o,
            _ => return false,
        };
        self.is_mm(r)
    }
}

pub(crate) struct CountedInternalizer<'a> {
    pub count: usize,
    cache: HashMap<RefPtr, CountedRefPtr>,
    solver: &'a Solver,
}
impl<'a> CountedInternalizer<'a> {
    pub(crate) fn intern_external(&mut self, solver: &mut Solver, other: RefPtr) -> RefPtr {
        let other = ExplorableRef {
            rf: other,
            nodes: &self.solver.nodes,
        };
        let r = solver.counted_intern_external(&mut self.cache, other);
        self.count += r.count;
        r.ptr
    }
}

pub(crate) struct Internalizer<'a> {
    solve: bool,
    cache: HashMap<RefPtr, RefPtr>,
    cache_decls: HashMap<RefPtr, RefPtr>,
    solver: &'a Solver,
}

impl<'a> Internalizer<'a> {
    pub(crate) fn intern_external(&mut self, solver: &mut Solver, other: RefPtr) -> RefPtr {
        let other = ExplorableRef {
            rf: other,
            nodes: &self.solver.nodes,
        };
        let r = if self.solve {
            solver.local_solve_intern_external(&mut self.cache, other)
        } else {
            solver.intern_external(&mut Default::default(), &mut self.cache, other)
        };
        // should not be needed as we aleady interned external refs in extend
        // if self.solver.refs[other] {
        //     if r >= solver.refs.len() {
        //         solver.refs.resize(r + 1, false);
        //     }
        //     solver.refs.set(r, true);
        // }
        r
    }
    pub(crate) fn intern_external_decl(&mut self, solver: &mut Solver, other: RefPtr) -> RefPtr {
        let other = ExplorableRef {
            rf: other,
            nodes: &self.solver.nodes,
        };
        let r = if self.solve {
            solver.local_solve_intern_external(&mut self.cache_decls, other)
        } else {
            solver.intern_external(&mut Default::default(), &mut self.cache_decls, other)
        };
        // should not be needed as we aleady interned external refs in extend
        // if self.solver.refs[other] {
        //     if r >= solver.refs.len() {
        //         solver.refs.resize(r + 1, false);
        //     }
        //     solver.refs.set(r, true);
        // }
        r
    }
}

impl Index<RefPtr> for Solver {
    type Output = RefsEnum<RefPtr, LabelPtr>;

    fn index(&self, index: RefPtr) -> &Self::Output {
        &self.nodes[index]
    }
}
