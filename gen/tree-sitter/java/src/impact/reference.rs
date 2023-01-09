use std::fmt::Display;

use bitvec::order::Lsb0;

use super::element::{Arguments, ExplorableRef, Nodes, RefPtr, RefsEnum, RawLabelPtr};

use hyper_ast::types::LabelStore;

pub struct Iter<'a> {
    pub(crate) refs: bitvec::slice::IterOnes<'a, usize, Lsb0>,
    pub(crate) nodes: &'a Nodes,
}

impl<'a> Iterator for Iter<'a> {
    type Item = ExplorableRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.refs.next() {
            Some(b) => {
                match &self.nodes[b] {
                    RefsEnum::Primitive(_) => panic!(),
                    _ => (),
                };
                let r = ExplorableRef {
                    rf: b,
                    nodes: self.nodes,
                };
                Some(r)
            }
            None => None,
        }
    }
}

pub struct DisplayRef<'a, 'b, LS: LabelStore<str>> {
    rf: RefPtr,
    nodes: &'a Nodes,
    leafs: &'b LS,
}

impl<'a, 'b, LS: LabelStore<str>> DisplayRef<'a, 'b, LS> {
    fn with(&self, rf: RefPtr) -> Self {
        Self {
            rf,
            nodes: self.nodes,
            leafs: self.leafs,
        }
    }
}

impl<'a, 'b, LS: LabelStore<str>> From<(ExplorableRef<'a>, &'b LS)> for DisplayRef<'a, 'b, LS> {
    fn from((s, leafs): (ExplorableRef<'a>, &'b LS)) -> Self {
        Self {
            rf: s.rf,
            nodes: s.nodes,
            leafs,
        }
    }
}

impl<'a, 'b, LS: LabelStore<str, I = RawLabelPtr>> Display for DisplayRef<'a, 'b, LS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.nodes[self.rf] {
            RefsEnum::Root => write!(f, "/"),
            RefsEnum::MaybeMissing => write!(f, "?"),
            RefsEnum::Primitive(i) => {
                write!(f, "p")?;
                Display::fmt(i, f)
            }
            RefsEnum::Array(o) => {
                write!(f, "[{}]", self.with(*o))
            }
            RefsEnum::ArrayAccess(o) => {
                write!(f, "{}[?]", self.with(*o))
            }
            RefsEnum::This(o) => {
                write!(f, "{}.pthis", self.with(*o))
            }
            RefsEnum::Super(o) => {
                write!(f, "{}.psuper", self.with(*o))
            }
            RefsEnum::Mask(o, v) => {
                write!(f, "{}", self.with(*o))?;
                write!(f, ".{{")?;
                for a in v.iter() {
                    write!(f, "{},", self.with(*a))?;
                }
                write!(f, "}}")
            }
            RefsEnum::Or(v) => {
                write!(f, "{{")?;
                let mut first = true;
                for a in v.iter() {
                    if first {
                        first = false;
                    } else {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", self.with(*a))?;
                }
                write!(f, "}}")
            }
            RefsEnum::ScopedIdentifier(o, i) => {
                write!(f, "{}", self.with(*o))?;
                write!(f, ".{}", self.leafs.resolve(i.as_ref()))
            }
            RefsEnum::TypeIdentifier(o, i) => {
                write!(f, "{}", self.with(*o))?;
                write!(f, "%{}", self.leafs.resolve(i.as_ref()))
            }
            RefsEnum::MethodReference(o, i) => {
                write!(f, "{}", self.with(*o))?;
                write!(f, "::{}", self.leafs.resolve(i.as_ref()))
            }
            RefsEnum::ConstructorReference(o) => {
                write!(f, "{}::new", self.with(*o))
            }
            RefsEnum::Invocation(o, i, a) => {
                write!(f, "{}", self.with(*o))?;
                write!(f, ".{}", self.leafs.resolve(i.as_ref()))?;
                match a {
                    Arguments::Unknown => write!(f, "(...)"), //Ok(()),
                    Arguments::Given(a) => {
                        write!(f, "(")?;
                        for a in a.iter() {
                            write!(f, "{},", self.with(*a))?;
                        }
                        write!(f, ")")
                    }
                }
            }
            RefsEnum::ConstructorInvocation(i, a) => {
                write!(f, "{}#constructor", self.with(*i))?;
                match a {
                    Arguments::Unknown => write!(f, "(...)"),
                    Arguments::Given(a) => {
                        write!(f, "(")?;
                        for a in a.iter() {
                            write!(f, "{},", self.with(*a))?;
                        }
                        write!(f, ")")
                    }
                }
            }
        }
    }
}
