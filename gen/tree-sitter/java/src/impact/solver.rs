use std::{ops::{Index, Deref}, collections::HashMap, error::Error};

use string_interner::Symbol;

use super::{
    declaration::{self, DeclType, Declarator},
    element::{self, Arguments, ExplorableRef, LabelPtr, Nodes, RefPtr, RefsEnum, IdentifierFormat},
    java_element::Primitive,
    label_value::LabelValue,
    reference,
};

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

impl<T:Eq> FromIterator<T> for MultiResult<T> {
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

impl Solver {

    pub fn decls_count(&self) -> usize {
        self.decls.len()
    }

    fn is_length_token(&self, id: LabelPtr) -> bool {
        id.as_ref().to_usize() == 0 // TODO verify/model statically
    }
    fn is_package_token(&self, id: LabelPtr) -> bool {
        let f =id.format();
        IdentifierFormat::FlatCase.eq(&f) || IdentifierFormat::SnakeCase.eq(&f)
    }

    pub fn intern(&mut self, other: RefsEnum<RefPtr, LabelPtr>) -> RefPtr {
        // TODO analyze perfs to find if Vec or HashSet or something else works better
        match self.nodes.iter().position(|x| x == &other) {
            Some(x) => x,
            None => {
                let r = self.nodes.len();
                self.nodes.push(other);
                r
            }
        }
    }
    pub(crate) fn get(&self, other: RefsEnum<RefPtr, LabelPtr>) -> Option<RefPtr> {
        // TODO analyze perfs to find if Vec or HashSet or something else works better
        self.nodes.iter().position(|x| x == &other)
    }

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

    pub(crate) fn add_decl(&mut self, d: Declarator<RefPtr>, t: DeclType<RefPtr>) {
        self.decls.insert(d, t);
    }
    // pub(crate) fn add_decl_simple(&mut self, d: Declarator<RefPtr>, t: RefPtr) {
    //     self.decls
    //         .insert(d, DeclType::Compile(t, None, Default::default()));
    // }

    pub(crate) fn try_solve_node_with(&mut self, t: RefPtr, p: RefPtr) -> Option<RefPtr> {
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
            RefsEnum::Root => None,//("fully qualified node cannot be qualified further")),
            RefsEnum::MaybeMissing => Some(p),
            RefsEnum::Array(i) => {
                let x = self.try_solve_node_with(i, p)?;
                let tmp = RefsEnum::Array(x);
                Some(refs!(tmp))
            }
            RefsEnum::ArrayAccess(i) => {
                let x = self.try_solve_node_with(i, p)?;
                let tmp = RefsEnum::ArrayAccess(x);
                Some(refs!(tmp))
            }
            RefsEnum::This(i) => {
                let x = self.try_solve_node_with(i, p)?;
                let tmp = RefsEnum::This(x);
                Some(refs!(tmp))
            }
            RefsEnum::Super(i) => {
                let x = self.try_solve_node_with(i, p)?;
                let tmp = RefsEnum::Super(x);
                Some(refs!(tmp))
            }
            RefsEnum::Mask(i, y) => {
                let x = self.try_solve_node_with(i, p)?; // TODO not sure
                let tmp = RefsEnum::Mask(x, y);
                Some(refs!(tmp))
            }
            RefsEnum::ScopedIdentifier(i, y) => {
                let x = self.try_solve_node_with(i, p)?;
                let tmp = RefsEnum::ScopedIdentifier(x, y);
                Some(refs!(tmp))
            }
            RefsEnum::Invocation(o, i, args) => {
                let x = self.try_solve_node_with(o, p)?;
                let tmp = RefsEnum::Invocation(x, i, args);
                Some(refs!(tmp))
            }
            RefsEnum::ConstructorInvocation(o, args) => {
                let x = self.try_solve_node_with(o, p)?;
                let tmp = RefsEnum::ConstructorInvocation(x, args);
                Some(refs!(tmp))
            }
            x => todo!("not sure how {:?} should be handled", x),
        }
    }

    pub fn refs(&self) -> impl Iterator<Item = LabelValue> + '_ {
        self.refs
            .iter_ones()
            // iter().enumerate()
            // .filter_map(|(x,b)| if *b {Some(x)} else {None})
            .map(|x| {
                self.nodes.with(x).into()
            })
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

    fn iter_nodes<'a>(&'a self) -> element::NodesIter<'a> {
        element::NodesIter {
            rf: 0,
            nodes: &self.nodes,
        }
    }

    /// dedicated to solving references to localvariables
    fn local_solve_intern_external(
        &mut self,
        cache: &mut HashMap<RefPtr, RefPtr>,
        other: ExplorableRef,
    ) -> RefPtr {
        if let Some(x) = cache.get(&other.rf) {
            return *x;
        }
        // println!("other: {:?}", other);
        let r = match other.as_ref() {
            RefsEnum::Root => self.intern(RefsEnum::Root),
            RefsEnum::MaybeMissing => self.intern(RefsEnum::MaybeMissing),
            RefsEnum::Primitive(i) => self.intern(RefsEnum::Primitive(*i)),
            RefsEnum::Array(o) => {
                let o = self.local_solve_intern_external(cache, other.with(*o));
                self.intern(RefsEnum::Array(o))
            }
            RefsEnum::ArrayAccess(o) => {
                let o = self.local_solve_intern_external(cache, other.with(*o));
                match &self.nodes[o] {
                    RefsEnum::Array(x) => *x,
                    _ => self.intern(RefsEnum::ArrayAccess(o)),
                }
            }
            RefsEnum::This(o) => {
                let o = self.local_solve_intern_external(cache, other.with(*o));
                self.intern(RefsEnum::This(o))
            }
            RefsEnum::Super(o) => {
                let o = self.local_solve_intern_external(cache, other.with(*o));
                self.intern(RefsEnum::Super(o))
            }
            RefsEnum::Mask(o, v) => {
                // println!("try solve mask: {:?}", other);
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
            RefsEnum::ScopedIdentifier(o, i) => {
                // println!("try solve scoped id: {:?}", other);
                let o = self.local_solve_intern_external(cache, other.with(*o));
                self.intern(RefsEnum::ScopedIdentifier(o, *i))
            }
            RefsEnum::MethodReference(o, i) => {
                let o = self.local_solve_intern_external(cache, other.with(*o));
                self.intern(RefsEnum::MethodReference(o, *i))
            }
            RefsEnum::ConstructorReference(o) => {
                let o = self.local_solve_intern_external(cache, other.with(*o));
                self.intern(RefsEnum::ConstructorReference(o))
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
                // println!("try solve constructor: {:?}", other);
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
                // println!("solved local variable: {:?}", r);
                // self.solve_intern_external(cache, other.with(r))
                *r
            }
            None => r,
        };
        // TODO handle class statements
        cache.insert(other.rf, r);
        r
    }

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

    /// no internalization needed
    /// not used on blocks, only bodies, declarations and whole programs
    pub(crate) fn solve_aux(
        &mut self,
        cache: &mut HashMap<RefPtr, MultiResult<RefPtr>>,
        other: RefPtr,
    ) -> MultiResult<RefPtr> {
        if let Some(x) = cache.get(&other) {
            if x.is_empty() {
                // println!(
                //     "solving {:?}: {:?} from cache into nothing",
                //     other,
                //     ExplorableRef {
                //         rf: other,
                //         nodes: &self.nodes
                //     }
                // );
            } else {
                // for r in x.iter() {
                //     println!(
                //         "solving {:?}: {:?} from cache into {:?}",
                //         other,
                //         ExplorableRef {
                //             rf: other,
                //             nodes: &self.nodes
                //         },
                //         ExplorableRef {
                //             rf: *r,
                //             nodes: &self.nodes
                //         }
                //     );
                // }
            }
            return x.clone();
        }
        // println!(
        //     "solving : {:?} {:?}",
        //     other,
        //     ExplorableRef {
        //         rf: other,
        //         nodes: &self.nodes
        //     }
        // );

        // TODO decls should be searched without masks

        let no_mask = self.no_mask(other); // maybe return just after match
        let other = if let Some(r) = (&self.decls).get(&Declarator::Type(no_mask)).cloned() {
            // println!("t through cache {:?}", other);
            match r {
                DeclType::Compile(r, _, _) => r,
                DeclType::Runtime(b) => {
                    return Default::default();
                }
                x => todo!("{:?}", x),
            }
        } else if let Some(r) = (&self.decls).get(&Declarator::Field(no_mask)).cloned() {
            // println!("f through cache {:?}", other);
            match r {
                DeclType::Compile(r, _, _) => r,
                DeclType::Runtime(b) => {
                    if b.len() == 1 {
                        b[0]
                    } else if b.len() == 0 {
                        other
                    } else {
                        return b.iter().flat_map(|r| {
                            self.solve_aux(cache, *r).iter().map(|x| *x).collect::<Vec<_>>()
                        }).collect()
                    }
                }
            }
        } else {
            other
        };
        let r: MultiResult<RefPtr> = match self.nodes[other].clone() {
            RefsEnum::Root => [other].iter().map(|x| *x).collect(),
            RefsEnum::MaybeMissing => [other].iter().map(|x| *x).collect(), //if let Some(p) = self.root { p } else { other }),
            RefsEnum::Primitive(i) => [self.intern(RefsEnum::Primitive(i))]
                .iter()
                .map(|x| *x)
                .collect(),
            RefsEnum::Array(o) => {
                let r: MultiResult<RefPtr> = self
                    .solve_aux(cache, o)
                    .iter()
                    .map(|o| self.intern(RefsEnum::Array(*o)))
                    .collect();
                // TODO there should be more/other things to do
                cache.insert(other, r.clone());
                r
            }
            RefsEnum::ArrayAccess(o) => {
                let r: MultiResult<RefPtr> = self
                    .solve_aux(cache, o)
                    .iter()
                    .map(|o| match &self.nodes[*o] {
                        RefsEnum::Array(x) => *x,
                        _ => self.intern(RefsEnum::ArrayAccess(*o)),
                    })
                    .collect();
                // TODO there should be more/other things to do
                cache.insert(other, r.clone());
                r
            }
            RefsEnum::This(o) => {
                let r: MultiResult<RefPtr> = self
                    .solve_aux(cache, o)
                    .iter()
                    .map(|o| match &self.nodes[*o] {
                        RefsEnum::Mask(o, _) => *o,
                        _ => *o,
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|o| self.intern(RefsEnum::This(o)))
                    .collect();
                // TODO there should be more/other things to do
                if r.is_empty() {
                    cache.insert(other, r);
                    return Default::default();
                }
                let r: MultiResult<RefPtr> = r
                    .iter()
                    .flat_map(|r| {
                        let r = *r;
                        let r = if let Some(r) = (&self.decls).get(&Declarator::Type(r)).cloned() {
                            //println!("solved class: {:?}", r);
                            // None // TODO not 100% sure Some of None
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    //println!("solved class: {:?}", r);
                                    vec![] //Some(r)
                                }
                                DeclType::Runtime(b) => {
                                    //println!("solved runtime: {:?}", b);
                                    vec![]
                                }
                                x => todo!("{:?}", x),
                            }
                        } else if r != other {
                            self.solve_aux(cache, r).iter().map(|x| *x).collect()
                        } else {
                            vec![r]
                        };
                        r
                    })
                    // .flat_map(|x| x.into_iter().map(|x| *x))
                    .collect();
                r
            }
            RefsEnum::Super(o) => {
                let r: MultiResult<RefPtr> = self
                    .solve_aux(cache, o)
                    .iter()
                    .map(|o| match &self.nodes[*o] {
                        RefsEnum::Mask(o, _) => *o,
                        _ => *o,
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|o| self.intern(RefsEnum::Super(o)))
                    .collect();
                // TODO there should be more/other things to do
                if r.is_empty() {
                    cache.insert(other, r);
                    return Default::default();
                }

                let r: MultiResult<RefPtr> = r
                    .iter()
                    .flat_map(|r| {
                        let r = *r;
                        let r = if let Some(r) = (&self.decls).get(&Declarator::Type(r)).cloned() {
                            //println!("solved class: {:?}", r);
                            // None // TODO not 100% sure Some of None
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    //println!("solved class: {:?}", r);
                                    vec![] //Some(r)
                                }
                                DeclType::Runtime(b) => {
                                    //println!("solved runtime: {:?}", b);
                                    vec![]
                                }
                                x => todo!("{:?}", x),
                            }
                        } else if r != other {
                            self.solve_aux(cache, r).iter().map(|x| *x).collect()
                        } else {
                            vec![r]
                        };
                        r
                    })
                    .collect();
                r
            }
            RefsEnum::Mask(o, v) => {
                // println!("solving mask {:?}", other);
                let v: Vec<RefPtr> = v // TODO infinite loop
                    .iter()
                    .flat_map(|x| {
                        assert_ne!(other,*x);
                        // println!("mask {:?}", *x);
                        self.solve_aux(cache, *x) // TODO infinite loop
                            .iter()
                            .map(|x| *x)
                            .collect::<Vec<_>>() // TODO handle None properly
                    })
                    .collect();

                let r: MultiResult<RefPtr> = self.solve_aux(cache, o);

                if r.is_empty() {
                    // println!("solving {:?} an object of a mask into nothing", o);
                    cache.insert(other, r);
                    return Default::default();
                }

                let r: MultiResult<RefPtr> = r
                    .iter()
                    .map(|o| {
                        if v.is_empty() {
                            *o
                        } else {
                            self.intern(RefsEnum::Mask(*o, v.clone().into_boxed_slice()))
                        }
                    })
                    .collect();
                // TODO should look for declarations solving the masking
                // either the masked thing is declared by thing in mask
                // or the masked thing is surely not declared and remove the mask
                r
            }
            RefsEnum::Or(v) => {
                todo!()
            }
            RefsEnum::ScopedIdentifier(o, i) => {
                // println!("solving cioped id {:?}", other);
                let mut m: Option<Box<[usize]>> = None;
                let r: MultiResult<RefPtr> = self.solve_aux(cache, o); // TODO infinite loop
                if r.is_empty() {
                    // println!("solving {:?} an object into nothing", o);
                    cache.insert(other, r);
                    return Default::default();
                }
                let r: MultiResult<RefPtr> = r
                    .iter()
                    .filter_map(|o| {
                        let o = *o;
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
                                    return None
                                }
                                let no_mask = self.intern(RefsEnum::ScopedIdentifier(no_mask, i));
                                let x = self.solve_aux(cache, no_mask);
                                if !x.is_empty() {
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
                    })
                    .collect();
                if r.is_empty() {
                    // println!("solving {:?} into nothing", other);
                    cache.insert(other, r);
                    return Default::default();
                }

                let r: MultiResult<RefPtr> = r
                    .iter()
                    .flat_map(|r| {
                        let r = *r;
                        // println!(
                        //     "then {:?}",
                        //     ExplorableRef {
                        //         rf: r,
                        //         nodes: &self.nodes
                        //     }
                        // );
                        let r = if let Some(r) = (&self.decls).get(&Declarator::Type(r)).cloned() {
                            //println!("solved class: {:?}", r);
                            // None // TODO not 100% sure Some of None
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    //println!("solved class: {:?}", r);
                                    vec![] //Some(r)
                                }
                                DeclType::Runtime(b) => {
                                    //println!("solved runtime: {:?}", b);
                                    vec![]
                                }
                                x => todo!("{:?}", x),
                            }
                        } else if let Some(r) = (&self.decls).get(&Declarator::Field(r)).cloned() {
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    // println!("solved field: {:?}", r);
                                    self.solve_aux(cache, r).iter().map(|x| *x).collect()
                                }
                                _ => todo!(),
                            }
                        } else if let Some(r) = (&self.decls).get(&Declarator::Variable(r)).cloned()
                        {
                            // TODO should not need
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    // println!("solved local variable: {:?}", r);
                                    self.solve_aux(cache, r).iter().map(|x| *x).collect()
                                }
                                DeclType::Runtime(v) => {
                                    // println!("solved local variable: {:?}", r);
                                    v.iter().flat_map(|r| {
                                        self.solve_aux(cache, *r).iter().map(|x| *x).collect::<Vec<_>>()
                                    }).collect()
                                }
                            }
                        } else if r != other {
                            self.solve_aux(cache, r).iter().map(|x| *x).collect()
                        } else {
                            vec![r]
                        };
                        r
                    })
                    .collect();

                r
            }
            RefsEnum::MethodReference(o, i) => {
                let r: MultiResult<RefPtr> = self
                    .solve_aux(cache, o)
                    .iter()
                    .map(|o| self.intern(RefsEnum::MethodReference(*o, i)))
                    .collect();
                if r.is_empty() {
                    cache.insert(other, r);
                    return Default::default();
                }

                let r: MultiResult<RefPtr> = r
                    .iter()
                    .flat_map(|r| {
                        let r = *r;
                        let r = if let Some(r) =
                            (&self.decls).get(&Declarator::Executable(r)).cloned()
                        {
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    //println!("solved method ref: {:?}", r);
                                    self.solve_aux(cache, r).iter().map(|x| *x).collect()
                                }
                                _ => todo!(),
                            }
                        } else if r != other {
                            self.solve_aux(cache, r).iter().map(|x| *x).collect()
                        } else {
                            vec![r]
                        };
                        r
                    })
                    .collect();
                r
            }
            RefsEnum::ConstructorReference(o) => {
                let r: MultiResult<RefPtr> = self
                    .solve_aux(cache, o)
                    .iter()
                    .map(|o| self.intern(RefsEnum::ConstructorReference(*o)))
                    .collect();
                if r.is_empty() {
                    cache.insert(other, r);
                    return Default::default();
                }

                let r: MultiResult<RefPtr> = r
                    .iter()
                    .flat_map(|r| {
                        let r = *r;
                        let r = if let Some(r) =
                            (&self.decls).get(&Declarator::Executable(r)).cloned()
                        {
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    //println!("solved constructor ref: {:?}", r);
                                    self.solve_aux(cache, r).iter().map(|x| *x).collect()
                                }
                                _ => todo!(),
                            }
                        } else if r != other {
                            self.solve_aux(cache, r).iter().map(|x| *x).collect()
                        } else {
                            vec![r]
                        };
                        r
                    })
                    .collect();
                r
            }
            RefsEnum::Invocation(o, i, p) => {
                let r: MultiResult<RefPtr> = self
                    .solve_aux(cache, o)
                    .iter()
                    .flat_map(|o| {
                        let o = *o;
                        let mask_o = match &self.nodes[o] {
                            RefsEnum::Mask(o, _) => Some(*o),
                            _ => None,
                        };
                        let mm = self.intern(RefsEnum::MaybeMissing);
                        let mm = self.intern(RefsEnum::Mask(mm, Default::default()));
                        let r = if mask_o.is_some() && cache.get(&mm).is_some() {
                            //&& self.root.is_some() {
                            vec![]
                        } else {
                            let mut b = false;
                            let p = match &p {
                                Arguments::Unknown => Arguments::Unknown,
                                Arguments::Given(p) => {
                                    b = p.is_empty();
                                    let mut v = vec![];
                                    for x in p.deref() {
                                        let r = self.solve_aux(cache, *x);
                                        if r.is_empty() {
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
                                vec![self.intern(RefsEnum::Invocation(o, i, p))]
                            } else {
                                vec![]
                            }
                        };
                        r
                    })
                    .collect();
                if r.is_empty() {
                    cache.insert(other, r);
                    return Default::default();
                }

                let r: MultiResult<RefPtr> = r
                    .iter()
                    .flat_map(|r| {
                        let r = *r;
                        let r = if let Some(r) =
                            (&self.decls).get(&Declarator::Executable(r)).cloned()
                        {
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    //println!("solved method: {:?}", r);
                                    self.solve_aux(cache, r).iter().map(|x| *x).collect()
                                }
                                DeclType::Runtime(v) => {
                                    //println!("solved method: {:?}", r);
                                    v.iter().flat_map(|r| {
                                        self.solve_aux(cache, *r).iter().map(|x| *x).collect::<Vec<_>>()
                                    }).collect()
                                }
                            }
                        } else if r != other {
                            self.solve_aux(cache, r).iter().map(|x| *x).collect()
                        } else {
                            vec![r]
                        };
                        r
                    })
                    .collect();
                r
            }
            RefsEnum::ConstructorInvocation(o, p) => {
                let r: MultiResult<RefPtr> = self
                    .solve_aux(cache, o)
                    .iter()
                    .flat_map(|o| {
                        let o = *o;
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
                            let r =
                                self.intern(RefsEnum::ConstructorInvocation(o, Arguments::Unknown));
                            if let Some(r) = (&self.decls).get(&Declarator::Executable(r)).cloned()
                            {
                                match r {
                                    DeclType::Compile(r, s, i) => {
                                        //println!("solved super constructor type: {:?} {:?} {:?}", r, s, i);
                                        s.iter().flat_map(|r| {
                                            self.solve_aux(cache, *r).iter().map(|x| *x).collect::<Vec<_>>()
                                        }).collect()
                                        // self.solve_aux(cache, s.unwrap())
                                    }
                                    _ => todo!(),
                                }
                                // TODO if outside class body should return None ?
                            } else {
                                [o].iter().map(|x| *x).collect::<MultiResult<RefPtr>>()
                            }
                        } else if this {
                            let r =
                                self.intern(RefsEnum::ConstructorInvocation(o, Arguments::Unknown));
                            if let Some(r) = (&self.decls).get(&Declarator::Executable(r)).cloned()
                            {
                                match r {
                                    DeclType::Compile(r, s, i) => {
                                        //println!("solved super constructor type: {:?} {:?} {:?}", r, s, i);
                                        self.solve_aux(cache, r)
                                    }
                                    _ => todo!(),
                                }
                                // TODO if outside class body should return None ?
                            } else {
                                [o].iter().map(|x| *x).collect::<MultiResult<RefPtr>>()
                            }
                        } else {
                            [o].iter().map(|x| *x).collect::<MultiResult<RefPtr>>()
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
                                        if r.is_empty() {
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
                    })
                    .collect();
                if r.is_empty() {
                    cache.insert(other, r);
                    return Default::default();
                }

                let r: MultiResult<RefPtr> = r
                    .iter()
                    .flat_map(|r| {
                        let r = *r;
                        let r = if let Some(r) =
                            (&self.decls).get(&Declarator::Executable(r)).cloned()
                        {
                            match r {
                                DeclType::Compile(r, _, _) => {
                                    //println!("solved constructor: {:?} {:?} {:?}", r, s, i);
                                    self.solve_aux(cache, r).iter().map(|x| *x).collect()
                                }
                                DeclType::Runtime(v) => {
                                    //println!("solved constructor: {:?} {:?} {:?}", r, s, i);
                                    v.iter().flat_map(|r| {
                                        self.solve_aux(cache, *r).iter().map(|x| *x).collect::<Vec<_>>()
                                    }).collect()
                                }
                            }
                        } else if r != other {
                            self.solve_aux(cache, r).iter().map(|x| *x).collect()
                        } else {
                            vec![r]
                        };
                        r
                    })
                    .collect();
                r
            }
        };

        if r.is_empty() {
            // println!("solving {:?} into nothing", other);
            cache.insert(other, Default::default());
        } else {
            for r in r.iter() {
                // println!(
                //     "solving {:?} into {:?}",
                //     other,
                //     ExplorableRef {
                //         rf: *r,
                //         nodes: &self.nodes
                //     }
                // );
            }
            let r = r.iter().map(|x| *x).collect(); //r.iter().filter(|r| other.ne(*r)).collect();
            cache.insert(other, r);
        }
        r
    }

    fn print_decls(&self) {
        println!("sd: ");
        for (k, v) in self.decls.iter() {
            let kr = match k.node() {
                None => ExplorableRef {
                    rf: 0,
                    nodes: &self.nodes,
                },
                Some(rf) => ExplorableRef {
                    rf: *rf,
                    nodes: &self.nodes,
                },
            };
            match v {
                DeclType::Runtime(b) => {
                    // TODO print more things
                    println!("   {:?}: {:?} =>", k, kr);
                    for v in b.iter() {
                        let r = ExplorableRef {
                            rf: *v,
                            nodes: &self.nodes,
                        };
                        print!(" ({:?}) {:?}", *v, r);
                    }
                    println!();
                }
                DeclType::Compile(v, s, b) => {
                    // TODO print more things
                    let r = ExplorableRef {
                        rf: *v,
                        nodes: &self.nodes,
                    };
                    print!("   {:?}: {:?} => {:?}", k, kr, r);
                    if s.len() > 0 {
                        print!(" extends");
                    }
                    for v in s.iter() {
                        let v = ExplorableRef {
                            rf: *v,
                            nodes: &self.nodes,
                        };
                        print!(" {:?},", v);
                    }
                    if b.len() > 0 {
                        print!(" implements");
                    }
                    for v in b.iter() {
                        let v = ExplorableRef {
                            rf: *v,
                            nodes: &self.nodes,
                        };
                        print!(" {:?},", v);
                    }
                    println!();
                }
            }
        }
    }

    fn intern_external(
        &mut self,
        map: &mut HashMap<RefPtr, RefPtr>,
        cache: &mut HashMap<RefPtr, RefPtr>,
        other: ExplorableRef,
    ) -> RefPtr {
        // println!("int_ext   {:?} {:?}", other.rf, other);
        if let Some(x) = map.get(&other.rf) {
            // println!(
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
            // println!(
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
            RefsEnum::Array(o) => {
                let o = self.intern_external(map, cache, other.with(*o));
                self.intern(RefsEnum::Array(o))
            }
            RefsEnum::ArrayAccess(o) => {
                let o = self.intern_external(map, cache, other.with(*o));
                self.intern(RefsEnum::ArrayAccess(o))
            }
            RefsEnum::This(o) => {
                let o = self.intern_external(map, cache, other.with(*o));
                self.intern(RefsEnum::This(o))
            }
            RefsEnum::Super(o) => {
                let o = self.intern_external(map, cache, other.with(*o));
                self.intern(RefsEnum::Super(o))
            }
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
            RefsEnum::ScopedIdentifier(o, i) => {
                let o = self.intern_external(map, cache, other.with(*o));
                self.intern(RefsEnum::ScopedIdentifier(o, *i))
            }
            RefsEnum::MethodReference(o, i) => {
                let o = self.intern_external(map, cache, other.with(*o));
                // println!("{:?}", o);
                // println!("{:?}", self.nodes);
                self.intern(RefsEnum::MethodReference(o, *i))
            }
            RefsEnum::ConstructorReference(o) => {
                let o = self.intern_external(map, cache, other.with(*o));
                // println!("{:?}", o);
                // println!("{:?}", self.nodes);
                self.intern(RefsEnum::ConstructorReference(o))
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
        };
        assert!(
            self.nodes[r].similar(other.as_ref()),
            "{:?} ~ {:?}",
            other.as_ref(),
            self.nodes[r],
        );
        // println!(
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
        // println!("primed cache for extend_map");
        // for (k, v) in map.iter() {
        //     print!(
        //         "   {:?} {:?}: ",
        //         *k,
        //         ExplorableRef {
        //             rf: *k,
        //             nodes: &solver.nodes
        //         }
        //     );
        //     print!(
        //         "{:?} ",
        //         ExplorableRef {
        //             rf: *v,
        //             nodes: &self.nodes
        //         }
        //     );
        //     println!();
        // }
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

    pub(crate) fn resolve(
        self,
        mut cache: HashMap<RefPtr, MultiResult<RefPtr>>,
    ) -> (HashMap<RefPtr, MultiResult<RefPtr>>, Solver) {
        // let mut r = self.clone();
        let mut r = Solver {
            nodes: self.nodes.clone(),
            refs: Default::default(),
            decls: self.decls.clone(),
        };
        // self.print_decls();
        println!("primed cache for resolve");
        for (k, v) in &cache {
            print!(
                "   {:?}: ",
                ExplorableRef {
                    rf: *k,
                    nodes: &self.nodes
                }
            );
            for r in v.iter() {
                print!(
                    "{:?} ",
                    ExplorableRef {
                        rf: *r,
                        nodes: &self.nodes
                    }
                );
            }
            println!();
        }
        for s in self.iter_refs() {
            // TODO make it imperative ?
            for s in r.solve_aux(&mut cache, s.rf).iter() {
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

    // pub(crate) fn hierarchy_solve_extend<'a>(
    //     &mut self,
    //     solver: &'a Solver,
    // ) -> (HashMap<usize, Option<usize>>, Solver) {
    //     // let mut r = self.clone();
    //     let mut r = Solver {
    //         nodes: self.nodes.clone(),
    //         refs: Default::default(),
    //         decls: self.decls.clone(),
    //     };
    //     // self.print_decls();
    //     let mut cache = Default::default();
    //     for s in self.iter_refs() {
    //         // TODO make it imperative ?
    //         if let Some(s) = r.hierarchy_solve_intern_external(&mut cache, s) {
    //             match &r.nodes[s] {
    //                 RefsEnum::Primitive(_) => {}
    //                 _ => {
    //                     if s >= r.refs.len() {
    //                         r.refs.resize(s + 1, false);
    //                     }
    //                     r.refs.set(s, true);
    //                 }
    //             };
    //         }
    //     }
    //     // TODO try better
    //     (cache, r)
    // }

    // fn hierarchy_solve_intern_external(
    //     &mut self,
    //     cache: &mut HashMap<RefPtr, Option<RefPtr>>,
    //     other: ExplorableRef,
    // ) -> Option<RefPtr> {
    //     todo!()
    //     // let r = self.intern_external(&mut cache, other);
    //     // let r = self.solve_aux(&mut cache, r);
    //     // r
    // }
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
