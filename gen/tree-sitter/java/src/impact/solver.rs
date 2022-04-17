use std::{
    collections::HashMap,
    error::Error,
    ops::{Deref, Index},
};

use string_interner::Symbol;
use tuples::TupleCollect;

use crate::impact::declaration::{DebugDecls, DeclsIter};

use super::{
    declaration::{self, DeclType, Declarator},
    element::{
        self, Arguments, ExplorableRef, IdentifierFormat, LabelPtr, ListSet, Nodes, RefPtr,
        RefsEnum,
    },
    java_element::Primitive,
    label_value::LabelValue,
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

pub(crate) struct MultiResult<T>(Option<Box<[T]>>);

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

    fn is_empty(&self) -> bool {
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

    pub(crate) fn refs_count(&self) -> usize {
        self.refs.count_ones()
    }
    pub fn refs(&self) -> impl Iterator<Item = LabelValue> + '_ {
        self.refs
            .iter_ones()
            // iter().enumerate()
            // .filter_map(|(x,b)| if *b {Some(x)} else {None})
            .map(|x| self.nodes.with(x).bytes().into())
    }

    pub(crate) fn iter_refs<'a>(&'a self) -> reference::Iter<'a> {
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
        // TODO analyze perfs to find if Vec or HashSet or something else works better
        self.nodes.iter().position(|x| x == &other)
    }

    fn iter_nodes<'a>(&'a self) -> element::NodesIter<'a> {
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
    pub(crate) fn intern_ref(&mut self, other: RefsEnum<RefPtr, LabelPtr>) -> RefPtr {
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
        // log::trace!("int_ext   {:?} {:?}", other.rf, other);
        if let Some(x) = map.get(&other.rf) {
            // log::trace!(
            //     "int_ext m {:?} {:?}",
            //     other.rf,
            //     ExplorableRef {
            //         rf:*x,
            //         nodes: &self.nodes,
            //     }
            // );
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
            // log::trace!(
            //     "int_ext c {:?} {:?}",
            //     other.rf,
            //     ExplorableRef {
            //         rf:*x,
            //         nodes: &self.nodes,
            //     }
            // );
            return *x;
        }
        let r = match other.as_ref() {
            RefsEnum::Root => self.intern(RefsEnum::Root),
            RefsEnum::MaybeMissing => self.intern(RefsEnum::MaybeMissing),
            RefsEnum::Primitive(i) => self.intern(RefsEnum::Primitive(*i)),
            RefsEnum::Mask(o, p) => {
                let o = self.intern_external(map, cache, other.with(*o));
                let p = p
                    .iter()
                    .map(|x| self.intern_external(map, cache, other.with(*x)))
                    .collect();
                self.intern(RefsEnum::Mask(o, p))
            }
            RefsEnum::Or(p) => {
                let p = p
                    .iter()
                    .map(|x| self.intern_external(map, cache, other.with(*x)))
                    .collect();
                self.intern(RefsEnum::Or(p))
            }
            RefsEnum::Invocation(o, i, p) => {
                let o = self.intern_external(map, cache, other.with(*o));
                let p = match p {
                    Arguments::Unknown => Arguments::Unknown,
                    Arguments::Given(p) => {
                        let mut v = vec![];
                        for x in p.deref() {
                            let r = self.intern_external(map, cache, other.with(*x));
                            v.push(r);
                        }
                        let p = v.into_boxed_slice();
                        Arguments::Given(p)
                    }
                };
                self.intern(RefsEnum::Invocation(o, *i, p))
            }
            RefsEnum::ConstructorInvocation(o, p) => {
                let i = self.intern_external(map, cache, other.with(*o));
                let p = match p {
                    Arguments::Unknown => Arguments::Unknown,
                    Arguments::Given(p) => {
                        let p = p
                            .deref()
                            .iter()
                            .map(|x| self.intern_external(map, cache, other.with(*x)))
                            .collect();
                        Arguments::Given(p)
                    }
                };
                let r = self.intern(RefsEnum::ConstructorInvocation(i, p));
                assert_ne!(r, i);
                r
            }
            x => {
                let o = x.object().expect(&format!("should have an object {:?}", x));
                let o = self.intern_external(map, cache, other.with(o));
                self.intern(x.with_object(o))
            }
        };
        assert!(
            self.nodes[r].similar(other.as_ref()),
            "{:?} ~ {:?}",
            other.as_ref(),
            self.nodes[r],
        );
        // log::trace!(
        //     "int_ext r {:?} {:?}",
        //     other.rf,
        //     ExplorableRef {
        //         rf:r,
        //         nodes: &self.nodes,
        //     }
        // );
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
            x => {
                let o = x.object().expect(&format!("should have an object {:?}", x));
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
                let x = y
                    .iter()
                    .filter_map(|x| self.try_unsolve_node_with(*x, p))
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
                let o = x.object().expect(&format!("should have an object {:?}", x));
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

    fn no_choice(&mut self, other: RefPtr) -> RefPtr {
        let o = self.nodes[other].object();
        let o = if let Some(o) = o {
            self.no_mask(o)
        } else {
            return other;
        };
        if let RefsEnum::Or(_) = self.nodes[other] {
            return o;
        }
        let x = self.nodes[other].with_object(o);
        self.intern(x)
    }
}

/// advanced insertions
impl Solver {
    /// dedicated to solving references to localvariables
    pub(crate) fn local_solve_extend<'a>(&mut self, solver: &'a Solver) -> Internalizer<'a> {
        let mut cached = Internalizer {
            solve: true,
            cache: Default::default(),
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
                    .map(|x| self.local_solve_intern_external(cache, other.with(*x)))
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
                let o = x.object().expect(&format!("should have an object {:?}", x));
                let o = self.local_solve_intern_external(cache, other.with(o));
                self.intern(x.with_object(o))
            }
        };
        let r = match self.decls.get(&Declarator::Variable(r)) {
            Some(DeclType::Runtime(b)) => {
                if b.len() == 1 {
                    b[0]
                } else {
                    b[0] // TODO
                }
            }
            Some(DeclType::Compile(r, s, i)) => {
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

    /// copy all references in [`solver`] to current solver
    pub(crate) fn extend<'a>(&mut self, solver: &'a Solver) -> Internalizer<'a> {
        self.extend_map(solver, &mut Default::default())
    }

    pub(crate) fn extend_map<'a>(
        &mut self,
        solver: &'a Solver,
        map: &mut HashMap<usize, usize>,
    ) -> Internalizer<'a> {
        let mut cached = Internalizer {
            solve: false,
            cache: Default::default(),
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

#[derive(Debug, Clone)]
pub struct SolvingResult {
    /// Whenever the result is the consequence of a match to its declaration.
    /// It does not particularly buble up when solving, it depends on the type of RefsEnum.
    matched: bool,
    /// The reference that we are atempting to solve, it may contain RefsEnum::Mask and RefsEnum::Or
    result: Option<RefPtr>,
    /// References corresponding to the developped form of result ie. do not contain Mask and Or
    /// Important for Matching against declarations
    /// TODO check if building digesteds on the fly would be better
    /// NOTE I fear that precomputing them is a waste of time and won't help much finding the matched branch in result
    digested: ListSet<RefPtr>,
}

impl SolvingResult {
    pub fn new(result:RefPtr,digested:ListSet<RefPtr>) -> Self {
        Self {
            matched: false,
            result: Some(result),
            digested,
        }
    }
    pub fn is_matched(&self) -> bool {
        self.matched
    }
    pub fn iter(&self) -> impl Iterator<Item = &RefPtr> {
        self.digested.iter()
    }
}

impl Default for SolvingResult {
    fn default() -> Self {
        Self {
            matched: false,
            result: None,
            digested: Default::default(),
        }
    }
}
impl<Node: Eq> FromIterator<Node> for SolvingResult {
    fn from_iter<T: IntoIterator<Item = Node>>(iter: T) -> Self {
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
        Self { intern: Default::default() }
    }
}

impl SolvingAssocTable {
    const def:Option<SolvingResult> = None;
    pub fn get(&self, index: &RefPtr) -> &Option<SolvingResult> {
        if index >= &self.intern.len() {
            return &Self::def;
        }
        &self.intern[*index]
    }
    pub fn insert(&mut self, index: RefPtr, v: SolvingResult) {
        self.intern[index] = Some(v);
    }
    fn len(&self, index: &RefPtr) -> usize {
        self.intern.len()
    }
    fn is_empty(&self) -> bool {
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
            "sd: {:?}",
            DebugDecls {
                decls: &self.decls,
                nodes: &self.nodes
            }
        );
        for s in self.iter_refs() {
            // TODO make it imperative ?
            let rr = r.solve_aux(&mut cache, s.rf);
            if rr.is_matched() {
                continue;
            }
            for s in rr.iter() {
                let s = *s;
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

    /// no internalization needed
    /// not used on blocks, only bodies, declarations and whole programs
    pub(crate) fn solve_aux(
        &mut self,
        cache: &mut SolvingAssocTable,
        other: RefPtr,
    ) -> SolvingResult {
        if let Some(x) = cache.get(&other) {
            if x.is_matched() {
                log::trace!(
                    "solving {:?}: {:?} from cache into nothing",
                    other,
                    self.nodes.with(other)
                );
            } else {
                log::trace!("solve {:?}: {:?} from cache", other, self.nodes.with(other),);
                // for r in x.iter() {
                //     log::trace!(
                //         "solving {:?}: {:?} from cache into {:?}",
                //         other,
                //         self.nodes.with(other),
                //         self.nodes.with(*r)
                //     );
                // }
            }
            return x.clone();
        }
        log::trace!("solving : {:?} {:?}", other, self.nodes.with(other));

        let other = if let RefsEnum::TypeIdentifier(o, i) = self.nodes[other].clone() {
            let no_mask = self.no_mask(o); // maybe return just after match
            if no_mask != o {
                // let simp = self.intern(RefsEnum::ScopedIdentifier(o,i));
                // let no_mask = self.intern(RefsEnum::ScopedIdentifier(no_mask,i));
                let simp = self.intern(RefsEnum::TypeIdentifier(o, i));
                let no_mask = self.intern(RefsEnum::TypeIdentifier(no_mask, i));
                if let Some(r) = (&self.decls).get(&Declarator::Type(no_mask)).cloned() {
                    log::trace!("t t through cache {:?}", other);
                    match r {
                        DeclType::Compile(r, _, _) => r,
                        DeclType::Runtime(b) => {
                            // TODO should be about generics
                            if b.is_empty() {
                                return Default::default();
                            } else {
                                match self.nodes[other].clone() {
                                    RefsEnum::Mask(mo, ms) => {
                                        // TODO do more than b[0]
                                        // TODO mo might contain more masks
                                        match self.nodes[b[0]].clone() {
                                            RefsEnum::ScopedIdentifier(oo, ii) => {
                                                let n = self.intern(RefsEnum::Mask(oo, ms));
                                                self.intern(RefsEnum::ScopedIdentifier(n, ii))
                                            }
                                            RefsEnum::TypeIdentifier(oo, ii) => {
                                                let n = self.intern(RefsEnum::Mask(oo, ms));
                                                self.intern(RefsEnum::TypeIdentifier(n, ii))
                                            }
                                            _ => todo!(),
                                        }
                                    }
                                    _ => b[0],
                                }
                            }
                        }
                        x => todo!("{:?}", x),
                    }
                // } else if let Some(r) = (&self.decls).get(&Declarator::Field(no_mask)).cloned() {
                //     log::trace!("t f through cache {:?}", other);
                //     match r {
                //         DeclType::Compile(r, _, _) => r,
                //         DeclType::Runtime(b) => {
                //             if b.len() == 1 {
                //                 b[0]
                //             } else if b.len() == 0 {
                //                 simp
                //             } else {
                //                 return b.iter().flat_map(|r| {
                //                     self.solve_aux(cache, *r).iter().map(|x| *x).collect::<Vec<_>>()
                //                 }).collect()
                //             }
                //         }
                //     }
                // } else if let Some(r) = (&self.decls).get(&Declarator::Variable(no_mask)).cloned() {
                //     log::trace!("t v through cache {:?}", other);
                //     match r {
                //         DeclType::Compile(r, _, _) => r,
                //         DeclType::Runtime(b) => {
                //             if b.len() == 1 {
                //                 b[0]
                //             } else if b.len() == 0 {
                //                 simp
                //             } else {
                //                 return b.iter().flat_map(|r| {
                //                     self.solve_aux(cache, *r).iter().map(|x| *x).collect::<Vec<_>>()
                //                 }).collect()
                //             }
                //         }
                //     }
                } else {
                    simp //other
                }
            } else {
                other
            }
        } else {
            let no_mask = self.no_mask(other); // maybe return just after match
            if let Some(r) = (&self.decls).get(&Declarator::Field(no_mask)).cloned() {
                log::trace!("f through cache {:?}", other);
                match r {
                    DeclType::Compile(r, _, _) => r,
                    DeclType::Runtime(b) => {
                        if b.len() == 1 {
                            log::trace!("f through cache gg {:?}", other);
                            b[0]
                        } else if b.len() == 0 {
                            other
                        } else {
                            return b
                                .iter()
                                .flat_map(|r| {
                                    self.solve_aux(cache, *r)
                                        .iter()
                                        .map(|x| *x)
                                        .collect::<Vec<_>>()
                                })
                                .collect();
                        }
                    }
                }
            } else if let Some(r) = (&self.decls).get(&Declarator::Variable(no_mask)).cloned() {
                log::trace!("v through cache {:?}", other);
                match r {
                    DeclType::Compile(r, _, _) => r,
                    DeclType::Runtime(b) => {
                        if b.len() == 1 {
                            log::trace!("v through cache gg {:?}", other);
                            b[0]
                        } else if b.len() == 0 {
                            other
                        } else {
                            return b
                                .iter()
                                .flat_map(|r| {
                                    self.solve_aux(cache, *r)
                                        .iter()
                                        .map(|x| *x)
                                        .collect::<Vec<_>>()
                                })
                                .collect();
                        }
                    }
                }
            } else if let Some(r) = (&self.decls).get(&Declarator::Type(no_mask)).cloned() {
                log::trace!("t through cache {:?}", other);
                match r {
                    DeclType::Compile(r, _, _) => r,
                    DeclType::Runtime(b) => {
                        return Default::default();
                    }
                    x => todo!("{:?}", x),
                }
            } else {
                other
            }
        };
        let r: SolvingResult = match self.nodes[other].clone() {
            RefsEnum::Root => [other].iter().map(|x| *x).collect(),
            RefsEnum::MaybeMissing => [other].iter().map(|x| *x).collect(), //if let Some(p) = self.root { p } else { other }),
            RefsEnum::Primitive(i) => [self.intern(RefsEnum::Primitive(i))]
                .iter()
                .map(|x| *x)
                .collect(),
            RefsEnum::Array(o) => {
                let rr = self.solve_aux(cache, o);
                let r: ListSet<RefPtr> = rr
                    .iter()
                    .map(|o| self.intern(RefsEnum::Array(*o)))
                    .collect();
                let r = SolvingResult {
                    matched: rr.matched,
                    result: rr.result.map(|x| self.intern(RefsEnum::Array(x))),
                    digested: r,
                };
                // TODO there should be more/other things to do
                cache.insert(other, r.clone());
                r
            }
            RefsEnum::ArrayAccess(o) => {
                let rr = self.solve_aux(cache, o);
                let r: ListSet<RefPtr> = rr
                    .iter()
                    .map(|o| match &self.nodes[*o] {
                        RefsEnum::Array(x) => *x,
                        _ => self.intern(RefsEnum::ArrayAccess(*o)),
                    })
                    .collect();
                let r = SolvingResult {
                    matched: rr.matched,
                    result: rr.result.map(|x| match &self.nodes[x] {
                        RefsEnum::Array(x) => *x,
                        _ => self.intern(RefsEnum::ArrayAccess(x)),
                    }),
                    digested: r,
                };
                cache.insert(other, r.clone());
                r
            }
            RefsEnum::This(o) => {
                let rr = self.solve_aux(cache, o);
                let r: ListSet<RefPtr> = rr
                    .iter()
                    .map(|o| match &self.nodes[*o] {
                        RefsEnum::Mask(o, _) => *o,
                        _ => *o,
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|o| self.intern(RefsEnum::This(o)))
                    .collect();
                let r = SolvingResult {
                    matched: rr.matched,
                    result: rr
                        .result
                        .map(|o| match &self.nodes[o] {
                            RefsEnum::Mask(o, _) => *o,
                            _ => o,
                        })
                        .map(|o| self.intern(RefsEnum::This(o))),
                    digested: r,
                };
                // TODO there should be more/other things to do
                if r.digested.is_empty() {
                    cache.insert(other, r.clone());
                    return r;
                }
                let mut matched = r.matched;
                let old_result = r.result;
                let mut possibilities = vec![];
                let r: ListSet<RefPtr> = r
                    .iter()
                    .flat_map(|r| {
                        let r = *r;
                        let r = if let Some(r) = (&self.decls).get(&Declarator::Type(r)).cloned() {
                            matched = true;
                            //log::trace!("solved class: {:?}", r);
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    // TODO do I use the rest of the hierarchy
                                    possibilities.push(r);
                                    //log::trace!("solved class: {:?}", r);
                                    vec![r]
                                }
                                DeclType::Runtime(b) => {
                                    if b.len() == 1 {
                                        possibilities.push(b[1]);
                                    } else if b.is_empty() {
                                        panic!("{}", other)
                                    } else {
                                        // TODO maybe just extending possibilities is enough
                                        let r = self.intern(RefsEnum::Or(b.clone().into()));
                                        possibilities.push(r);
                                    }
                                    //log::trace!("solved runtime: {:?}", b);
                                    b.to_vec()
                                }
                                x => todo!("{:?}", x),
                            }
                        } else if r != other {
                            let rr = self.solve_aux(cache, r);
                            matched |= rr.matched;
                            rr.iter().map(|x| *x).collect()
                        } else {
                            vec![r]
                        };
                        r
                    })
                    .collect();
                let result = if possibilities.is_empty() {
                    old_result
                } else if possibilities.len() == 1 {
                    Some(possibilities[0])
                } else {
                    Some(self.intern(RefsEnum::Or(possibilities.into())))
                };
                let r = SolvingResult {
                    matched,
                    result,
                    digested: r,
                };
                r
            }
            RefsEnum::Super(o) => {
                let rr = self.solve_aux(cache, o);
                let r: ListSet<RefPtr> = rr
                    .iter()
                    .map(|o| match &self.nodes[*o] {
                        RefsEnum::Mask(o, _) => *o,
                        _ => *o,
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|o| self.intern(RefsEnum::Super(o)))
                    .collect();
                let r = SolvingResult {
                    matched: rr.matched,
                    result: rr
                        .result
                        .map(|o| match &self.nodes[o] {
                            RefsEnum::Mask(o, _) => *o,
                            _ => o,
                        })
                        .map(|o| self.intern(RefsEnum::Super(o))),
                    digested: r,
                };
                // TODO there should be more/other things to do
                if r.digested.is_empty() {
                    cache.insert(other, r.clone());
                    return r;
                }

                let mut matched = r.matched;
                let old_result = r.result;
                let mut possibilities = vec![];
                let r: ListSet<RefPtr> = r
                    .iter()
                    .flat_map(|r| {
                        let r = *r;
                        let r = if let Some(r) = (&self.decls).get(&Declarator::Type(r)).cloned() {
                            matched = true;
                            //log::trace!("solved class: {:?}", r);
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    // TODO should use hierarchy as it is an explicit super access, I would bet on superClass
                                    possibilities.push(r);
                                    //log::trace!("solved class: {:?}", r);
                                    vec![r]
                                }
                                DeclType::Runtime(b) => {
                                    // TODO check with a unit test how this case happened
                                    if b.len() == 1 {
                                        possibilities.push(b[1]);
                                    } else if b.is_empty() {
                                        panic!("{}", other)
                                    } else {
                                        // TODO maybe just extending possibilities is enough
                                        let r = self.intern(RefsEnum::Or(b.clone().into()));
                                        possibilities.push(r);
                                    }
                                    //log::trace!("solved runtime: {:?}", b);
                                    b.to_vec()
                                }
                                x => todo!("{:?}", x),
                            }
                        } else if r != other {
                            let rr = self.solve_aux(cache, r);
                            matched |= rr.matched;
                            if let Some(r) = rr.result {
                                possibilities.push(r);
                            }
                            rr.iter().map(|x| *x).collect()
                        } else {
                            vec![r]
                        };
                        r
                    })
                    .collect();
                let result = if possibilities.is_empty() {
                    old_result
                } else if possibilities.len() == 1 {
                    Some(possibilities[0])
                } else {
                    Some(self.intern(RefsEnum::Or(possibilities.into())))
                };
                let r = SolvingResult {
                    matched,
                    result,
                    digested: r,
                };
                r
            }
            RefsEnum::Mask(o, v) => {
                log::trace!("solving mask {:?}", other);
                let mut v_matched = vec![];
                let mut v_rest = vec![];
                let mut v_both = vec![];
                let mut v_results = vec![];
                let mut matched = false;
                v.iter().for_each(|x| {
                    assert_ne!(other, *x);
                    log::trace!("mask {:?}", *x);
                    let rr = self.solve_aux(cache, *x);
                    if let Some(r) = rr.result {
                        v_results.push(r);
                    }
                    v_both.extend(rr.digested.clone());
                    if rr.is_matched() {
                        matched = true;
                        v_matched.extend(rr.digested);
                    } else {
                        v_rest.extend(rr.digested);
                    }
                });

                let rr = self.solve_aux(cache, o);

                if rr.digested.is_empty() {
                    // log::trace!("solving {:?} an object of a mask into nothing", o);
                    cache.insert(other, rr.clone());
                    return rr;
                }

                let r: ListSet<RefPtr> =
                    rr.digested.into_iter().chain(v_both.into_iter()).collect();

                let r = SolvingResult {
                    matched: rr.matched | matched,
                    result: rr.result.map(|o| {
                        self.intern(RefsEnum::Mask(o, v_results.clone().into_boxed_slice()))
                    }),
                    digested: r,
                };

                // TODO should look for declarations solving the masking
                // either the masked thing is declared by thing in mask
                // or the masked thing is surely not declared and remove the mask
                r
            }
            RefsEnum::Or(_) => {
                // TODO if one case is matched, we should be able to ditch the rest
                todo!()
            }
            RefsEnum::TypeIdentifier(oo, i) => {
                // log::trace!("solving scoped type {:?}", other);
                let mut m: Option<Box<[usize]>> = None;
                let rr = self.solve_aux(cache, oo);
                if rr.digested.is_empty() {
                    // log::trace!("solving {:?} an object into nothing", o);
                    cache.insert(other, rr.clone());
                    return rr;
                }

                let mut handle = |o: &usize| {
                    let o = *o;
                    if self.is_mm(oo) {
                        if !self.is_root(o) && self.is_package(o) && self.is_package_token(i) {
                            return None;
                        }
                    }
                    let matched = match &self.nodes[o] {
                        // if /.A
                        RefsEnum::Root if !self.is_package_token(i) => return None,
                        // if x[].lenght -> int , thus None to signify its solved
                        RefsEnum::Array(_) if self.is_length_token(i) => None,
                        RefsEnum::Mask(o, x) => {
                            m = Some(x.clone());
                            Some(*o)
                        }
                        _ => Some(o),
                    };
                    let o = if let Some(m) = &m {
                        for m in m.iter() {
                            let no_mask = self.no_mask(*m);
                            if self.intern(RefsEnum::Root) == no_mask && !self.is_package_token(i) {
                                continue;
                            }
                            let no_mask = self.intern(RefsEnum::TypeIdentifier(no_mask, i));
                            let x = self.solve_aux(cache, no_mask);
                            log::trace!("for {:?} choose between:", no_mask);
                            x.iter().for_each(|x| {
                                let x = self.nodes.with(*x);
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
                        Some(o)
                    } else {
                        matched
                    };
                    if let Some(o) = o {
                        Some(self.intern(RefsEnum::TypeIdentifier(o, i)))
                    } else {
                        Some(self.intern(RefsEnum::Primitive(Primitive::Int)))
                    }
                };

                let r: ListSet<RefPtr> = rr.iter().filter_map(handle).collect();
                let mut handle = |o: &usize| {
                    let o = *o;
                    if self.is_mm(oo) {
                        if !self.is_root(o) && self.is_package(o) && self.is_package_token(i) {
                            return None;
                        }
                    }
                    let matched = match &self.nodes[o] {
                        // if /.A
                        RefsEnum::Root if !self.is_package_token(i) => return None,
                        // if x[].lenght -> int , thus None to signify its solved
                        RefsEnum::Array(_) if self.is_length_token(i) => None,
                        RefsEnum::Mask(o, x) => {
                            m = Some(x.clone());
                            Some(*o)
                        }
                        _ => Some(o),
                    };
                    let o = if let Some(m) = &m {
                        for m in m.iter() {
                            let no_mask = self.no_mask(*m);
                            if self.intern(RefsEnum::Root) == no_mask && !self.is_package_token(i) {
                                continue;
                            }
                            let no_mask = self.intern(RefsEnum::TypeIdentifier(no_mask, i));
                            let x = self.solve_aux(cache, no_mask);
                            log::trace!("for {:?} choose between:", no_mask);
                            x.iter().for_each(|x| {
                                let x = self.nodes.with(*x);
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
                        Some(o)
                    } else {
                        matched
                    };
                    if let Some(o) = o {
                        Some(self.intern(RefsEnum::TypeIdentifier(o, i)))
                    } else {
                        Some(self.intern(RefsEnum::Primitive(Primitive::Int)))
                    }
                };
                let result = rr.result.and_then(|o| handle(&o));
                let r = SolvingResult {
                    matched: rr.matched,
                    result,
                    digested: r,
                };

                if r.digested.is_empty() {
                    // log::trace!("solving {:?} into nothing", other);
                    cache.insert(other, r.clone());
                    return r;
                }

                let mut matched = r.matched;
                let old_result = r.result;
                let mut possibilities = vec![];
                let r: ListSet<RefPtr> = r
                    .iter()
                    .flat_map(|r| {
                        let r = *r;
                        let r = if let Some(r) = (&self.decls).get(&Declarator::Type(r)).cloned() {
                            matched = true;
                            //log::trace!("solved class: {:?}", r);
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    // TODO should use hierarchy as it is an explicit super access, I would bet on superClass
                                    possibilities.push(r);
                                    //log::trace!("solved class: {:?}", r);
                                    vec![r]
                                }
                                DeclType::Runtime(b) => {
                                    // TODO check with a unit test how this case happened
                                    if b.len() == 1 {
                                        possibilities.push(b[1]);
                                    } else if b.is_empty() {
                                        panic!("{}", other)
                                    } else {
                                        // TODO maybe just extending possibilities is enough
                                        let r = self.intern(RefsEnum::Or(b.clone().into()));
                                        possibilities.push(r);
                                    }
                                    //log::trace!("solved runtime: {:?}", b);
                                    b.to_vec()
                                }
                                x => todo!("{:?}", x),
                            }
                        } else if r != other {
                            let rr = self.solve_aux(cache, r);
                            matched |= rr.matched;
                            rr.iter().map(|x| *x).collect()
                        } else {
                            vec![r]
                        };
                        r
                    })
                    .collect();
                let result = if possibilities.is_empty() {
                    old_result
                } else if possibilities.len() == 1 {
                    Some(possibilities[0])
                } else {
                    Some(self.intern(RefsEnum::Or(possibilities.into())))
                };
                let r = SolvingResult {
                    matched,
                    result,
                    digested: r,
                };

                r
            }
            RefsEnum::ScopedIdentifier(oo, i) => {
                // log::trace!("solving cioped id {:?}", other);
                let mut m: Option<Box<[usize]>> = None;
                let rr = self.solve_aux(cache, oo);
                if rr.digested.is_empty() {
                    // log::trace!("solving {:?} an object into nothing", o);
                    cache.insert(other, rr.clone());
                    return rr;
                }

                let mut handle = |o: &usize| {
                    let o = *o;
                    if self.is_mm(oo) {
                        if !self.is_root(o) && self.is_package(o) && self.is_package_token(i) {
                            return None;
                        }
                    }
                    let matched = match &self.nodes[o] {
                        // if x[].lenght -> int
                        RefsEnum::Root if !self.is_package_token(i) => return None,
                        RefsEnum::Array(_) if self.is_length_token(i) => None,
                        RefsEnum::Mask(o, x) => {
                            m = Some(x.clone());
                            Some(*o)
                        }
                        _ => Some(o),
                    };
                    let o = if let Some(m) = &m {
                        for m in m.iter() {
                            let no_mask = self.no_mask(*m);
                            if self.intern(RefsEnum::Root) == no_mask && !self.is_package_token(i) {
                                return None;
                            }
                            let no_mask = self.intern(RefsEnum::ScopedIdentifier(no_mask, i));
                            let x = self.solve_aux(cache, no_mask);
                            if !x.is_matched() {
                                // TODO check, was using is_empty
                                let x = *x.iter().next().unwrap();
                                if x != no_mask {
                                    return Some(x);
                                }
                            }
                        }
                        Some(o)
                    } else {
                        matched
                    };
                    if let Some(o) = o {
                        Some(self.intern(RefsEnum::ScopedIdentifier(o, i)))
                    } else {
                        Some(self.intern(RefsEnum::Primitive(Primitive::Int)))
                    }
                };

                let r: ListSet<RefPtr> = rr.iter().filter_map(handle).collect();
                let mut handle = |o: &usize| {
                    let o = *o;
                    if self.is_mm(oo) {
                        if !self.is_root(o) && self.is_package(o) && self.is_package_token(i) {
                            return None;
                        }
                    }
                    let matched = match &self.nodes[o] {
                        // if x[].lenght -> int
                        RefsEnum::Root if !self.is_package_token(i) => return None,
                        RefsEnum::Array(_) if self.is_length_token(i) => None,
                        RefsEnum::Mask(o, x) => {
                            m = Some(x.clone());
                            Some(*o)
                        }
                        _ => Some(o),
                    };
                    let o = if let Some(m) = &m {
                        for m in m.iter() {
                            let no_mask = self.no_mask(*m);
                            if self.intern(RefsEnum::Root) == no_mask && !self.is_package_token(i) {
                                return None;
                            }
                            let no_mask = self.intern(RefsEnum::ScopedIdentifier(no_mask, i));
                            let x = self.solve_aux(cache, no_mask);
                            if !x.is_matched() {
                                // TODO check, was using is_empty
                                let x = *x.iter().next().unwrap();
                                if x != no_mask {
                                    return Some(x);
                                }
                            }
                        }
                        Some(o)
                    } else {
                        matched
                    };
                    if let Some(o) = o {
                        Some(self.intern(RefsEnum::ScopedIdentifier(o, i)))
                    } else {
                        Some(self.intern(RefsEnum::Primitive(Primitive::Int)))
                    }
                };
                let result = rr.result.and_then(|o| handle(&o));
                let r = SolvingResult {
                    matched: rr.matched,
                    result,
                    digested: r,
                };

                if r.digested.is_empty() {
                    // log::trace!("solving {:?} into nothing", other);
                    cache.insert(other, r.clone());
                    return r;
                }

                let mut matched = r.matched;
                let old_result = r.result;
                let mut possibilities = vec![];
                let r: ListSet<RefPtr> = r
                    .iter()
                    .flat_map(|r| {
                        let r = *r;
                        let r = if let Some(r) = (&self.decls).get(&Declarator::Type(r)).cloned() {
                            matched = true;
                            //log::trace!("solved class: {:?}", r);
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    // TODO should use hierarchy as it is an explicit super access, I would bet on superClass
                                    possibilities.push(r);
                                    //log::trace!("solved class: {:?}", r);
                                    vec![r]
                                }
                                DeclType::Runtime(b) => {
                                    // TODO check with a unit test how this case happened
                                    if b.len() == 1 {
                                        possibilities.push(b[1]);
                                    } else if b.is_empty() {
                                        panic!("{}", other)
                                    } else {
                                        // TODO maybe just extending possibilities is enough
                                        let r = self.intern(RefsEnum::Or(b.clone().into()));
                                        possibilities.push(r);
                                    }
                                    //log::trace!("solved runtime: {:?}", b);
                                    b.to_vec()
                                }
                                x => todo!("{:?}", x),
                            }
                        } else if let Some(r) = (&self.decls).get(&Declarator::Field(r)).cloned() {
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    // TODO should not append, I think
                                    possibilities.push(r);
                                    // log::trace!("solved field: {:?}", r);
                                    self.solve_aux(cache, r).iter().map(|x| *x).collect()
                                }
                                DeclType::Runtime(v) => {
                                    // log::trace!("solved field: {:?}", r);
                                    // TODO check with a unit test how this case happened
                                    if v.len() == 1 {
                                        possibilities.push(v[1]);
                                    } else if v.is_empty() {
                                        panic!("{}", other)
                                    } else {
                                        // TODO maybe just extending possibilities is enough
                                        let r = self.intern(RefsEnum::Or(v.clone().into()));
                                        possibilities.push(r);
                                    }
                                    v.iter()
                                        .flat_map(|r| {
                                            self.solve_aux(cache, *r)
                                                .iter()
                                                .map(|x| *x)
                                                .collect::<Vec<_>>()
                                        })
                                        .collect()
                                }
                            }
                        } else if let Some(r) = (&self.decls).get(&Declarator::Variable(r)).cloned()
                        {
                            // TODO should not need
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    // log::trace!("solved local variable: {:?}", r);
                                    possibilities.push(r);
                                    self.solve_aux(cache, r).iter().map(|x| *x).collect()
                                }
                                DeclType::Runtime(v) => {
                                    // log::trace!("solved local variable: {:?}", r);
                                    // TODO check with a unit test how this case happened
                                    if v.len() == 1 {
                                        possibilities.push(v[1]);
                                    } else if v.is_empty() {
                                        panic!("{}", other)
                                    } else {
                                        // TODO maybe just extending possibilities is enough
                                        let r = self.intern(RefsEnum::Or(v.clone().into()));
                                        possibilities.push(r);
                                    }
                                    v.iter()
                                        .flat_map(|r| {
                                            self.solve_aux(cache, *r)
                                                .iter()
                                                .map(|x| *x)
                                                .collect::<Vec<_>>()
                                        })
                                        .collect()
                                }
                            }
                        } else if r != other {
                            let rr = self.solve_aux(cache, r);
                            matched |= rr.matched;
                            rr.iter().map(|x| *x).collect()
                        } else {
                            vec![r]
                        };
                        r
                    })
                    .collect();
                let result = if possibilities.is_empty() {
                    old_result
                } else if possibilities.len() == 1 {
                    Some(possibilities[0])
                } else {
                    Some(self.intern(RefsEnum::Or(possibilities.into())))
                };
                let r = SolvingResult {
                    matched,
                    result,
                    digested: r,
                };

                r
            }
            RefsEnum::MethodReference(o, i) => {
                let rr = self.solve_aux(cache, o);
                let r: ListSet<RefPtr> = rr
                    .iter()
                    .map(|o| self.intern(RefsEnum::MethodReference(*o, i)))
                    .collect();
                let r = SolvingResult {
                    matched: rr.matched,
                    result: rr
                        .result
                        .map(|o| self.intern(RefsEnum::MethodReference(o, i))),
                    digested: r,
                };
                if r.digested.is_empty() {
                    cache.insert(other, r.clone());
                    return r;
                }

                let mut matched = r.matched;
                let old_result = r.result;
                let mut possibilities = vec![];
                let r: ListSet<RefPtr> = r
                    .iter()
                    .flat_map(|&r| {
                        let r = if let Some(r) =
                            (&self.decls).get(&Declarator::Executable(r)).cloned()
                        {
                            matched = true;
                            //log::trace!("solved method ref: {:?}", r);
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    let r = self.solve_aux(cache, r); // TODO check is solving here is needed
                                    if let Some(r) = r.result {
                                        possibilities.push(r);
                                    }
                                    r.digested.into_iter().collect()
                                }
                                DeclType::Runtime(b) => {
                                    // TODO check with a unit test how this case happened
                                    b.iter()
                                        .flat_map(|&r| {
                                            let r = self.solve_aux(cache, r);
                                            if let Some(r) = r.result {
                                                possibilities.push(r);
                                            }
                                            r.digested
                                        })
                                        .collect()
                                    //log::trace!("solved runtime: {:?}", b);
                                }
                                x => todo!("{:?}", x),
                            }
                        } else if r != other {
                            let rr = self.solve_aux(cache, r);
                            matched |= rr.matched;
                            if let Some(r) = rr.result {
                                possibilities.push(r);
                            }
                            rr.iter().map(|x| *x).collect()
                        } else {
                            vec![r]
                        };
                        r
                    })
                    .collect();
                let result = if possibilities.is_empty() {
                    old_result
                } else if possibilities.len() == 1 {
                    Some(possibilities[0])
                } else {
                    Some(self.intern(RefsEnum::Or(possibilities.into())))
                };
                let r = SolvingResult {
                    matched,
                    result,
                    digested: r,
                };
                r
            }
            RefsEnum::ConstructorReference(o) => {
                let rr = self.solve_aux(cache, o);
                let r: ListSet<RefPtr> = rr
                    .iter()
                    .map(|o| self.intern(RefsEnum::ConstructorReference(*o)))
                    .collect();
                let r = SolvingResult {
                    matched: rr.matched,
                    result: rr
                        .result
                        .map(|o| self.intern(RefsEnum::ConstructorReference(o))),
                    digested: r,
                };
                if r.digested.is_empty() {
                    cache.insert(other, r.clone());
                    return r;
                }

                let mut matched = r.matched;
                let old_result = r.result;
                let mut possibilities = vec![];
                let r: ListSet<RefPtr> = r
                    .iter()
                    .flat_map(|&r| {
                        let r = if let Some(r) =
                            (&self.decls).get(&Declarator::Executable(r)).cloned()
                        {
                            matched = true;
                            //log::trace!("solved method ref: {:?}", r);
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    let r = self.solve_aux(cache, r); // TODO check is solving here is needed
                                    if let Some(r) = r.result {
                                        possibilities.push(r);
                                    }
                                    r.digested.into_iter().collect()
                                }
                                DeclType::Runtime(b) => {
                                    // TODO check with a unit test how this case happened
                                    b.iter()
                                        .flat_map(|&r| {
                                            let r = self.solve_aux(cache, r);
                                            if let Some(r) = r.result {
                                                possibilities.push(r);
                                            }
                                            r.digested
                                        })
                                        .collect()
                                    //log::trace!("solved runtime: {:?}", b);
                                }
                                x => todo!("{:?}", x),
                            }
                        } else if r != other {
                            let rr = self.solve_aux(cache, r);
                            matched |= rr.matched;
                            if let Some(r) = rr.result {
                                possibilities.push(r);
                            }
                            rr.iter().map(|x| *x).collect()
                        } else {
                            vec![r]
                        };
                        r
                    })
                    .collect();
                let result = if possibilities.is_empty() {
                    old_result
                } else if possibilities.len() == 1 {
                    Some(possibilities[0])
                } else {
                    Some(self.intern(RefsEnum::Or(possibilities.into())))
                };
                let r = SolvingResult {
                    matched,
                    result,
                    digested: r,
                };
                r
            }
            RefsEnum::Invocation(o, i, p) => {
                let rr = self.solve_aux(cache, o);

                let mut handle = |&o: &usize| {
                    let mask_o = match &self.nodes[o] {
                        RefsEnum::Mask(o, _) => Some(*o),
                        _ => None,
                    };
                    let mm = self.intern(RefsEnum::MaybeMissing);
                    let mm = self.intern(RefsEnum::Mask(mm, Default::default()));
                    let r = if mask_o.is_some() && cache.get(&mm).is_some() {
                        None
                    } else {
                        let mut b = false;
                        let p = match &p {
                            Arguments::Unknown => Arguments::Unknown,
                            Arguments::Given(p) => {
                                b = p.is_empty();
                                let mut v = vec![];
                                for x in p.deref() {
                                    let r = self.solve_aux(cache, *x);
                                    if r.is_matched() {
                                        // TODO check, was using !is_empty
                                        v.push(*x); // TODO or MaybeMissing ?
                                    } else {
                                        for r in r.iter() {
                                            b = true;
                                            v.push(*r);
                                            break; // TODO handle combinatorial
                                        }
                                    }
                                }
                                let p = v.into_boxed_slice();
                                Arguments::Given(p)
                            }
                        };
                        if b {
                            Some(self.intern(RefsEnum::Invocation(o, i, p)))
                        } else {
                            None
                        }
                    };
                    r
                };
                let r: ListSet<RefPtr> = rr.iter().filter_map(handle).collect();
                let mut handle = |&o: &usize| {
                    let mask_o = match &self.nodes[o] {
                        RefsEnum::Mask(o, _) => Some(*o),
                        _ => None,
                    };
                    let mm = self.intern(RefsEnum::MaybeMissing);
                    let mm = self.intern(RefsEnum::Mask(mm, Default::default()));
                    let r = if mask_o.is_some() && cache.get(&mm).is_some() {
                        None
                    } else {
                        let mut b = false;
                        let p = match &p {
                            Arguments::Unknown => Arguments::Unknown,
                            Arguments::Given(p) => {
                                b = p.is_empty();
                                let mut v = vec![];
                                for x in p.deref() {
                                    let r = self.solve_aux(cache, *x);
                                    if r.is_matched() {
                                        // TODO check, was using !is_empty
                                        v.push(*x); // TODO or MaybeMissing ?
                                    } else {
                                        for r in r.iter() {
                                            b = true;
                                            v.push(*r);
                                            break; // TODO handle combinatorial
                                        }
                                    }
                                }
                                let p = v.into_boxed_slice();
                                Arguments::Given(p)
                            }
                        };
                        if b {
                            Some(self.intern(RefsEnum::Invocation(o, i, p)))
                        } else {
                            None
                        }
                    };
                    r
                };
                let r = SolvingResult {
                    matched: rr.matched,
                    result: rr
                        .result
                        // TODO
                        .and_then(|o| handle(&o)),
                    digested: r,
                };
                if r.digested.is_empty() {
                    cache.insert(other, r.clone());
                    return r;
                }

                let mut matched = r.matched;
                let old_result = r.result;
                let mut possibilities = vec![];
                let r: ListSet<RefPtr> = r
                    .iter()
                    .flat_map(|&r| {
                        let r = if let Some(r) =
                            (&self.decls).get(&Declarator::Executable(r)).cloned()
                        {
                            matched = true;
                            //log::trace!("solved method ref: {:?}", r);
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    let r = self.solve_aux(cache, r); // TODO check is solving here is needed
                                    if let Some(r) = r.result {
                                        possibilities.push(r);
                                    }
                                    r.digested.into_iter().collect()
                                }
                                DeclType::Runtime(b) => {
                                    // TODO check with a unit test how this case happened
                                    b.iter()
                                        .flat_map(|&r| {
                                            let r = self.solve_aux(cache, r);
                                            if let Some(r) = r.result {
                                                possibilities.push(r);
                                            }
                                            r.digested
                                        })
                                        .collect()
                                    //log::trace!("solved runtime: {:?}", b);
                                }
                                x => todo!("{:?}", x),
                            }
                        } else if r != other {
                            let rr = self.solve_aux(cache, r);
                            matched |= rr.matched;
                            if let Some(r) = rr.result {
                                possibilities.push(r);
                            }
                            rr.iter().map(|x| *x).collect()
                        } else {
                            vec![r]
                        };
                        r
                    })
                    .collect();
                let result = if possibilities.is_empty() {
                    old_result
                } else if possibilities.len() == 1 {
                    Some(possibilities[0])
                } else {
                    Some(self.intern(RefsEnum::Or(possibilities.into())))
                };
                let r = SolvingResult {
                    matched,
                    result,
                    digested: r,
                };
                r
            }
            RefsEnum::ConstructorInvocation(o, p) => {
                let rr = self.solve_aux(cache, o);

                let mut handle = |&o: &usize| {
                    let (sup, this) = match &self.nodes[o] {
                        RefsEnum::Super(_) => (true, false),
                        RefsEnum::This(_) => (false, true),
                        _ => (false, false),
                    };

                    let mask_o = match &self.nodes[o] {
                        RefsEnum::Mask(o, _) => Some(*o),
                        _ => None,
                    };
                    let mm = self.intern(RefsEnum::MaybeMissing);
                    let mm = self.intern(RefsEnum::Mask(mm, Default::default()));

                    let o = if sup {
                        let r = self.intern(RefsEnum::ConstructorInvocation(o, Arguments::Unknown));
                        if let Some(r) = (&self.decls).get(&Declarator::Executable(r)).cloned() {
                            match r {
                                DeclType::Compile(r, s, i) => {
                                    //log::trace!("solved super constructor type: {:?} {:?} {:?}", r, s, i);
                                    s.iter()
                                        .flat_map(|r| {
                                            self.solve_aux(cache, *r)
                                                .iter()
                                                .map(|x| *x)
                                                .collect::<Vec<_>>()
                                        })
                                        .collect()
                                    // self.solve_aux(cache, s.unwrap())
                                }
                                _ => todo!(),
                            }
                            // TODO if outside class body should return None ?
                        } else {
                            [o].iter().map(|x| *x).collect::<ListSet<RefPtr>>()
                        }
                    } else if this {
                        let r = self.intern(RefsEnum::ConstructorInvocation(o, Arguments::Unknown));
                        if let Some(r) = (&self.decls).get(&Declarator::Executable(r)).cloned() {
                            match r {
                                DeclType::Compile(r, s, i) => {
                                    //log::trace!("solved super constructor type: {:?} {:?} {:?}", r, s, i);
                                    self.solve_aux(cache, r).digested.into_iter().collect()
                                }
                                _ => todo!(),
                            }
                            // TODO if outside class body should return None ?
                        } else {
                            [o].iter().map(|x| *x).collect::<ListSet<RefPtr>>()
                        }
                    } else {
                        [o].iter().map(|x| *x).collect::<ListSet<RefPtr>>()
                    };

                    let r = if mask_o.is_some() && cache.get(&mm).is_some() {
                        //&& self.root.is_some() {
                        vec![]
                    } else {
                        let mut b = false;
                        let pp = match &p {
                            Arguments::Unknown => Arguments::Unknown,
                            Arguments::Given(p) => {
                                b = p.is_empty();
                                let mut v = vec![];
                                for x in p.deref() {
                                    let r = self.solve_aux(cache, *x);
                                    if r.digested.is_empty() {
                                        v.push(*x); // TODO or MaybeMissing ?
                                    } else {
                                        for r in r.iter() {
                                            b = true;
                                            v.push(*r);
                                            break; // TODO handle combinatorial
                                        }
                                    }
                                }
                                let p = v.into_boxed_slice();
                                Arguments::Given(p)
                            }
                        };
                        if b {
                            o.iter()
                                .map(|o| {
                                    self.intern(RefsEnum::ConstructorInvocation(*o, pp.clone()))
                                })
                                .collect()
                        } else {
                            vec![]
                        }
                    };
                    r
                };

                let r: ListSet<RefPtr> = rr.iter().flat_map(handle).collect();
                let mut handle = |&o: &usize| {
                    let (sup, this) = match &self.nodes[o] {
                        RefsEnum::Super(_) => (true, false),
                        RefsEnum::This(_) => (false, true),
                        _ => (false, false),
                    };

                    let mask_o = match &self.nodes[o] {
                        RefsEnum::Mask(o, _) => Some(*o),
                        _ => None,
                    };
                    let mm = self.intern(RefsEnum::MaybeMissing);
                    let mm = self.intern(RefsEnum::Mask(mm, Default::default()));

                    let o = if sup {
                        let r = self.intern(RefsEnum::ConstructorInvocation(o, Arguments::Unknown));
                        if let Some(r) = (&self.decls).get(&Declarator::Executable(r)).cloned() {
                            match r {
                                DeclType::Compile(r, s, i) => {
                                    //log::trace!("solved super constructor type: {:?} {:?} {:?}", r, s, i);
                                    s.iter()
                                        .flat_map(|r| {
                                            self.solve_aux(cache, *r)
                                                .iter()
                                                .map(|x| *x)
                                                .collect::<Vec<_>>()
                                        })
                                        .collect()
                                    // self.solve_aux(cache, s.unwrap())
                                }
                                _ => todo!(),
                            }
                            // TODO if outside class body should return None ?
                        } else {
                            [o].iter().map(|x| *x).collect::<ListSet<RefPtr>>()
                        }
                    } else if this {
                        let r = self.intern(RefsEnum::ConstructorInvocation(o, Arguments::Unknown));
                        if let Some(r) = (&self.decls).get(&Declarator::Executable(r)).cloned() {
                            match r {
                                DeclType::Compile(r, s, i) => {
                                    //log::trace!("solved super constructor type: {:?} {:?} {:?}", r, s, i);
                                    self.solve_aux(cache, r).digested.into_iter().collect()
                                }
                                _ => todo!(),
                            }
                            // TODO if outside class body should return None ?
                        } else {
                            [o].iter().map(|x| *x).collect::<ListSet<RefPtr>>()
                        }
                    } else {
                        [o].iter().map(|x| *x).collect::<ListSet<RefPtr>>()
                    };

                    let r = if mask_o.is_some() && cache.get(&mm).is_some() {
                        //&& self.root.is_some() {
                        vec![]
                    } else {
                        let mut b = false;
                        let pp = match &p {
                            Arguments::Unknown => Arguments::Unknown,
                            Arguments::Given(p) => {
                                b = p.is_empty();
                                let mut v = vec![];
                                for x in p.deref() {
                                    let r = self.solve_aux(cache, *x);
                                    if r.digested.is_empty() {
                                        v.push(*x); // TODO or MaybeMissing ?
                                    } else {
                                        for r in r.iter() {
                                            b = true;
                                            v.push(*r);
                                            break; // TODO handle combinatorial
                                        }
                                    }
                                }
                                let p = v.into_boxed_slice();
                                Arguments::Given(p)
                            }
                        };
                        if b {
                            o.iter()
                                .map(|o| {
                                    self.intern(RefsEnum::ConstructorInvocation(*o, pp.clone()))
                                })
                                .collect()
                        } else {
                            vec![]
                        }
                    };
                    r
                };
                let r = SolvingResult {
                    matched: rr.matched,
                    result: rr
                        .result
                        // TODO
                        .and_then(|o| handle(&o).first().map(|x| *x)), // TODO
                    digested: r,
                };
                if r.digested.is_empty() {
                    cache.insert(other, r.clone());
                    return r;
                }

                let mut matched = r.matched;
                let old_result = r.result;
                let mut possibilities = vec![];
                let r: ListSet<RefPtr> = r
                    .iter()
                    .flat_map(|&r| {
                        let r = if let Some(r) =
                            (&self.decls).get(&Declarator::Executable(r)).cloned()
                        {
                            matched = true;
                            //log::trace!("solved method ref: {:?}", r);
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    let r = self.solve_aux(cache, r); // TODO check is solving here is needed
                                    if let Some(r) = r.result {
                                        possibilities.push(r);
                                    }
                                    r.digested.into_iter().collect()
                                }
                                DeclType::Runtime(b) => {
                                    // TODO check with a unit test how this case happened
                                    b.iter()
                                        .flat_map(|&r| {
                                            let r = self.solve_aux(cache, r);
                                            if let Some(r) = r.result {
                                                possibilities.push(r);
                                            }
                                            r.digested
                                        })
                                        .collect()
                                    //log::trace!("solved runtime: {:?}", b);
                                }
                                x => todo!("{:?}", x),
                            }
                        } else if r != other {
                            let rr = self.solve_aux(cache, r);
                            matched |= rr.matched;
                            if let Some(r) = rr.result {
                                possibilities.push(r);
                            }
                            rr.iter().map(|x| *x).collect()
                        } else {
                            vec![r]
                        };
                        r
                    })
                    .collect();
                let result = if possibilities.is_empty() {
                    old_result
                } else if possibilities.len() == 1 {
                    Some(possibilities[0])
                } else {
                    Some(self.intern(RefsEnum::Or(possibilities.into())))
                };
                let r = SolvingResult {
                    matched,
                    result,
                    digested: r,
                };
                r
            }
        };

        if r.digested.is_empty() {
            // log::trace!("solving {:?} into nothing", other);
            cache.insert(other, r.clone());
        } else {
            for r in r.iter() {
                log::trace!("solving {:?} into {:?}", other, self.nodes.with(*r));
            }
            cache.insert(other, r.clone());
        }
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
            x => return false,
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

pub(crate) struct Internalizer<'a> {
    solve: bool,
    cache: HashMap<RefPtr, RefPtr>,
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
}

impl Index<RefPtr> for Solver {
    type Output = RefsEnum<RefPtr, LabelPtr>;

    fn index(&self, index: RefPtr) -> &Self::Output {
        &self.nodes[index]
    }
}
