use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::{BuildHasher, BuildHasherDefault, Hash};
use std::marker::PhantomData;
use std::ops::{Deref, Index, IndexMut};
use std::str::Utf8Error;

use hyper_ast::filter::default::{Pearson, VaryHasher};
use num::ToPrimitive;
use serde::de::VariantAccess;
use serde::ser::SerializeSeq;
use serde::Serialize;
use string_interner::symbol::SymbolU16;
use string_interner::{DefaultSymbol, StringInterner, Symbol};

use crate::store::vec_map_store::{self, VecMapStore};
use crate::utils::hash;

use super::bytes_interner::{BytesInterner, BytesMap};
use super::java_element::Primitive;
use super::label_value::LabelValue;

pub type RefPtr = usize;

pub type RawLabelPtr = DefaultSymbol;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct LabelPtr(RawLabelPtr, IdentifierFormat);

impl LabelPtr {
    pub fn format(&self) -> IdentifierFormat {
        self.1
    }
    pub fn new(l: RawLabelPtr, f: IdentifierFormat) -> Self {
        Self(l, f)
    }
}

impl AsRef<DefaultSymbol> for LabelPtr {
    fn as_ref(&self) -> &DefaultSymbol {
        &self.0
    }
}

/// https://en.wikipedia.org/wiki/Naming_convention_(programming)#Examples_of_multiple-word_identifier_formats
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum IdentifierFormat {
    FlatCase,           // twowords
    UpperFlatCase,      // TWOWORDS
    LowerCamelCase,     // twoWords
    UpperCamelCase,     // TwoWords
    SnakeCase,          // two_words
    ScreamingSnakeCase, // TWO_WORDS
    CamelSnakeCase,     // two_Words
    PascalSnakeCase,    // Two_Words
    None,               //compareToCI_UTF16
}

impl From<&str> for IdentifierFormat {
    fn from(label: &str) -> Self {
        let mut contains_underscore = false;
        let mut full_upper = true;
        let mut full_lower = true;
        let mut first_upper = false;
        let mut first_lower = false;
        let mut rest_full_upper = true;
        let mut rest_full_lower = true;
        // but not first char of identifier
        let mut just_after_underscore_upper = true;
        let mut just_after_underscore_lower = true;
        let mut first = true;
        let mut just_after_underscore = false;
        for c in label.chars() {
            if first {
                first = false;
                if c == '_' {
                    // contains_underscore = true;
                    // todo!("{}", c)
                } else if c.is_ascii_lowercase() {
                    first_lower = true;
                    full_upper = false;
                } else if c.is_ascii_uppercase() {
                    first_upper = true;
                    full_lower = false;
                } else {
                    // todo!("{}", c)
                }
            } else {
                if c == '_' {
                    contains_underscore = true;
                    just_after_underscore = true;
                } else {
                    if c.is_ascii_lowercase() {
                        full_upper = false;
                        if just_after_underscore {
                            just_after_underscore_upper = false;
                        } else {
                            rest_full_upper = false;
                        }
                    } else if c.is_ascii_uppercase() {
                        full_lower = false;
                        if just_after_underscore {
                            just_after_underscore_lower = false;
                        } else {
                            rest_full_lower = false;
                        }
                    } else {
                        // todo!("{}", c)
                    }
                    just_after_underscore = false;
                }
            }
        }
        if contains_underscore {
            if full_lower {
                Self::SnakeCase
            } else if full_upper {
                Self::ScreamingSnakeCase
            } else if first_lower && just_after_underscore_upper && rest_full_lower {
                Self::CamelSnakeCase
            // } else if first_lower && just_after_underscore_lower {
            //     Self::SnakeCase
            } else if first_upper && just_after_underscore_upper && rest_full_lower {
                Self::PascalSnakeCase
            } else {
                Self::None
            }
        } else {
            if full_lower {
                Self::FlatCase
            } else if full_upper {
                Self::UpperFlatCase
            } else if first_lower {
                Self::LowerCamelCase
            } else if first_upper {
                Self::UpperCamelCase
            } else {
                Self::None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(IdentifierFormat::FlatCase, "org".into());
        assert_eq!(IdentifierFormat::FlatCase, "twowords".into());
        assert_eq!(IdentifierFormat::UpperFlatCase, "TWOWORDS".into());
        assert_eq!(IdentifierFormat::LowerCamelCase, "twoWords".into());
        assert_eq!(IdentifierFormat::UpperCamelCase, "TwoWords".into());
        assert_eq!(IdentifierFormat::SnakeCase, "two_words".into());
        assert_eq!(IdentifierFormat::ScreamingSnakeCase, "TWO_WORDS".into());
        assert_eq!(IdentifierFormat::CamelSnakeCase, "two_Words".into());
        assert_eq!(IdentifierFormat::PascalSnakeCase, "Two_Words".into());
    }
}

#[derive(Debug, Clone)]
pub struct ListSet<Node>(Box<[Node]>);
impl<Node: Eq + Hash + Clone> ListSet<Node> {
    // TODO search nodes with hash with dichotomy
    pub fn push(&mut self, x: Node) {
        if !self.contains(&x) {
            let mut r = vec![];
            // r.extend_from_slice(&self.0[..p]);
            r.extend_from_slice(&self.0[..]);
            r.push(x);
            // r.extend_from_slice(&self.0[p..]);
            self.0 = r.into_boxed_slice();
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = &Node> {
        self.0.iter()
    }
}
impl<Node> ListSet<Node> {
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl<Node> Default for ListSet<Node> {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<Node: Eq + Hash> ListSet<Node> {
    // fn position(&self, x: &Node) -> Option<usize> {
    //     let mut i = 0;
    //     for y in &(*self.0) {
    //         if x == y {
    //             return Some(i);
    //         }
    //         i+=1;
    //     }
    //     None
    // }

    pub fn contains(&self, x: &Node) -> bool {
        for y in &(*self.0) {
            if x == y {
                return true;
            }
        }
        false
    }
}

impl<Node: Eq + Hash> PartialEq for ListSet<Node> {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        for y in &(*self.0) {
            if !other.contains(y) {
                return false;
            }
        }
        true
    }
}

impl<Node: Eq + Hash> Eq for ListSet<Node> {}

impl<Node: Eq + Hash> Hash for ListSet<Node> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut h = 0;
        for x in self.0.iter() {
            h ^= hash(x);
        }
        h.hash(state);
    }
}

impl<Node: Eq + Hash> FromIterator<Node> for ListSet<Node> {
    fn from_iter<T: IntoIterator<Item = Node>>(iter: T) -> Self {
        let mut r = vec![];
        for x in iter.into_iter() {
            if r.contains(&x) {
                r.push(x);
            }
        }
        Self(r.into_boxed_slice())
    }
}

impl<Node: Eq + Hash+Clone> From<Box<[Node]>> for ListSet<Node> {
    fn from(x: Box<[Node]>) -> Self {
        x.into_iter().map(|x|x.clone()).collect()
    }
}

impl<Node: Eq + Hash> From<Vec<Node>> for ListSet<Node> {
    fn from(x: Vec<Node>) -> Self {
        x.into_iter().collect()
    }
}

impl<Node:Clone> IntoIterator for ListSet<Node> {
    type Item=Node;

    type IntoIter=std::vec::IntoIter<Node>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.to_vec().into_iter()
    }
}

#[derive(Debug, Clone)]
pub enum RefsEnum<Node, Leaf> {
    // * Meta References
    /// rest of resolution and rest of refs masking it
    Mask(Node, Box<[Node]>),
    Or(ListSet<Node>),
    // XOr(Box<[Node]>),
    // Choices(Box<[Node]>,Box<[Node]>), // Union, Disjunction

    // * Basic References
    Root,
    MaybeMissing, // TODO replace ? with ~
    ScopedIdentifier(Node, Leaf),
    TypeIdentifier(Node, Leaf),
    // TODO Anonymous(Id)
    // no need instance of type for cases where there is a cast ie. to access static members as static do not overload .ie thus error
    MethodReference(Node, Leaf), // equivalent to Invocation(Node, Leaf, Arguments::Unknown) but it does not represent a call that is actually made
    ConstructorReference(Node), // equivalent to ConstructorInvocation(Node, Arguments::Unknown) but it does not represent a call that is actually made
    Invocation(Node, Leaf, Arguments<Node>),
    ConstructorInvocation(Node, Arguments<Node>), // equivalent to Invocation(Node, 'new', Arguments<Node>)

    // * Special References ie. specific to java
    Primitive(Primitive),
    Array(Node),
    This(Node),
    Super(Node),
    ArrayAccess(Node),
}
impl<Node: Eq + Hash + Clone, Leaf: Eq + Hash> Hash for RefsEnum<Node, Leaf> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            RefsEnum::Mask(_,_) => 1,
            RefsEnum::Or(_) => 1,
            RefsEnum::Root => 2,
            RefsEnum::MaybeMissing => 3,
            RefsEnum::ScopedIdentifier(_, _) => 4,
            RefsEnum::TypeIdentifier(_, _) => 4,
            RefsEnum::MethodReference(_, _) => 5,
            RefsEnum::ConstructorReference(_) => 6,
            RefsEnum::Invocation(_, _, _) => 7,
            RefsEnum::ConstructorInvocation(_, _) => 8,
            RefsEnum::Primitive(_) => 9,
            RefsEnum::Array(_) => 10,
            RefsEnum::This(_) => 11,
            RefsEnum::Super(_) => 12,
            RefsEnum::ArrayAccess(_) => 13,
        }
        .hash(state);
    }
}
impl<Node: Eq + Hash + Clone, Leaf: Eq + Hash> Eq for RefsEnum<Node, Leaf> {}
impl<Node: Eq + Hash + Clone, Leaf: Eq + Hash> PartialEq for RefsEnum<Node, Leaf> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Or(l0), Self::Or(r0)) => l0 == r0,
            (Self::ScopedIdentifier(l0, l1), Self::ScopedIdentifier(r0, r1)) => {
                l0 == r0 && l1 == r1
            }
            (Self::ScopedIdentifier(l0, l1), Self::TypeIdentifier(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::TypeIdentifier(l0, l1), Self::ScopedIdentifier(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::TypeIdentifier(l0, l1), Self::TypeIdentifier(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::MethodReference(l0, l1), Self::MethodReference(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::ConstructorReference(l0), Self::ConstructorReference(r0)) => l0 == r0,
            (Self::Invocation(l0, l1, l2), Self::Invocation(r0, r1, r2)) => {
                l0 == r0 && l1 == r1 && l2 == r2
            }
            (Self::ConstructorInvocation(l0, l1), Self::ConstructorInvocation(r0, r1)) => {
                l0 == r0 && l1 == r1
            }
            (Self::Primitive(l0), Self::Primitive(r0)) => l0 == r0,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            (Self::This(l0), Self::This(r0)) => l0 == r0,
            (Self::Super(l0), Self::Super(r0)) => l0 == r0,
            (Self::ArrayAccess(l0), Self::ArrayAccess(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl<Node: Eq + Hash + Clone, Leaf: Eq + Hash> RefsEnum<Node, Leaf> {
    pub(crate) fn strict_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Or(l0), Self::Or(r0)) => l0 == r0,
            (Self::ScopedIdentifier(l0, l1), Self::ScopedIdentifier(r0, r1)) => {
                l0 == r0 && l1 == r1
            }
            (Self::TypeIdentifier(l0, l1), Self::TypeIdentifier(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::MethodReference(l0, l1), Self::MethodReference(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::ConstructorReference(l0), Self::ConstructorReference(r0)) => l0 == r0,
            (Self::Invocation(l0, l1, l2), Self::Invocation(r0, r1, r2)) => {
                l0 == r0 && l1 == r1 && l2 == r2
            }
            (Self::ConstructorInvocation(l0, l1), Self::ConstructorInvocation(r0, r1)) => {
                l0 == r0 && l1 == r1
            }
            (Self::Primitive(l0), Self::Primitive(r0)) => l0 == r0,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            (Self::This(l0), Self::This(r0)) => l0 == r0,
            (Self::Super(l0), Self::Super(r0)) => l0 == r0,
            (Self::ArrayAccess(l0), Self::ArrayAccess(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl<Node: Clone, Leaf> RefsEnum<Node, Leaf> {
    pub(crate) fn object(&self) -> Option<Node> {
        let r = match self {
            RefsEnum::Mask(o, _) => o,
            RefsEnum::Or(_) => panic!(),
            RefsEnum::ScopedIdentifier(o, _) => o,
            RefsEnum::MethodReference(o, _) => o,
            RefsEnum::ConstructorReference(o) => o,
            RefsEnum::Invocation(o, _, _) => o,
            RefsEnum::ConstructorInvocation(o, _) => o,
            RefsEnum::Array(o) => o,
            RefsEnum::This(o) => o,
            RefsEnum::Super(o) => o,
            RefsEnum::ArrayAccess(o) => o,
            RefsEnum::Root => return None,
            RefsEnum::MaybeMissing => return None,
            RefsEnum::TypeIdentifier(o, _) => o,
            RefsEnum::Primitive(_) => return None,
            // _ => return None,
        };
        Some(r.clone())
    }
}

impl<Node: Clone, Leaf: Clone> RefsEnum<Node, Leaf> {
    pub(crate) fn with_object(&self, o: Node) -> Self {
        match self {
            RefsEnum::Mask(_, b) => RefsEnum::Mask(o, b.clone()),
            RefsEnum::Or(_) => panic!(),
            RefsEnum::ScopedIdentifier(_, i) => RefsEnum::ScopedIdentifier(o, i.clone()),
            RefsEnum::TypeIdentifier(_, i) => RefsEnum::TypeIdentifier(o, i.clone()),
            RefsEnum::MethodReference(_, i) => RefsEnum::MethodReference(o, i.clone()),
            RefsEnum::ConstructorReference(_) => RefsEnum::ConstructorReference(o),
            RefsEnum::Invocation(_, i, p) => RefsEnum::Invocation(o, i.clone(), p.clone()),
            RefsEnum::ConstructorInvocation(_, p) => RefsEnum::ConstructorInvocation(o, p.clone()),
            RefsEnum::Array(_) => RefsEnum::Array(o),
            RefsEnum::This(_) => RefsEnum::This(o),
            RefsEnum::Super(_) => RefsEnum::Super(o),
            RefsEnum::ArrayAccess(_) => RefsEnum::ArrayAccess(o),
            RefsEnum::Root => panic!(),
            RefsEnum::MaybeMissing => panic!(),
            RefsEnum::Primitive(_) => panic!(),
            // _ => panic!(),
        }
    }
}

impl<Node, Leaf: Eq> RefsEnum<Node, Leaf> {
    pub(crate) fn similar(&self, other: &Self) -> bool {
        match (self, other) {
            (RefsEnum::Root, RefsEnum::Root) => true,
            (RefsEnum::MaybeMissing, RefsEnum::MaybeMissing) => true,
            (RefsEnum::Primitive(i), RefsEnum::Primitive(j)) => i == j,
            (RefsEnum::Array(_), RefsEnum::Array(_)) => true,
            (RefsEnum::ArrayAccess(_), RefsEnum::ArrayAccess(_)) => true,
            (RefsEnum::This(_), RefsEnum::This(_)) => true,
            (RefsEnum::Super(_), RefsEnum::Super(_)) => true,
            (RefsEnum::Mask(_, u), RefsEnum::Mask(_, v)) => u.len() == v.len(),
            (RefsEnum::Or(u), RefsEnum::Or(v)) => u.len() == v.len(),
            (RefsEnum::ScopedIdentifier(_, i), RefsEnum::ScopedIdentifier(_, j)) => i == j,
            (RefsEnum::TypeIdentifier(_, i), RefsEnum::TypeIdentifier(_, j)) => i == j,
            (RefsEnum::MethodReference(_, i), RefsEnum::MethodReference(_, j)) => i == j,
            (RefsEnum::ConstructorReference(_), RefsEnum::ConstructorReference(_)) => true,
            (RefsEnum::Invocation(_, i, _), RefsEnum::Invocation(_, j, _)) => i == j, // TODO count parameters
            (RefsEnum::ConstructorInvocation(_, _), RefsEnum::ConstructorInvocation(_, _)) => true, // TODO count parameters
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Arguments<Node> {
    Unknown,
    Given(Box<[Node]>),
}
impl<Node: Eq + Hash> Hash for Arguments<Node> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}
impl<Node: Eq + Hash> Eq for Arguments<Node> {}
impl<Node: Eq + Hash> PartialEq for Arguments<Node> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Given(l0), Self::Given(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl<Node: Eq + Hash> Arguments<Node> {
    pub fn map<T: Eq + Hash, F: FnMut(&Node) -> T>(&self, f: F) -> Arguments<T> {
        match self {
            Arguments::Unknown => Arguments::Unknown,
            Arguments::Given(x) => Arguments::Given(x.iter().map(f).collect()),
        }
    }
}

#[derive(Clone)]
pub struct ExplorableRef<'a> {
    pub rf: RefPtr,
    pub nodes: &'a Nodes,
}

impl<'a> AsRef<RefsEnum<RefPtr, LabelPtr>> for ExplorableRef<'a> {
    fn as_ref(&self) -> &RefsEnum<RefPtr, LabelPtr> {
        &self.nodes[self.rf]
    }
}

// impl<'a> Clone for ExplorableRef<'a> {
//     fn clone(&self) -> Self {
//         Self {
//             rf:self.rf,
//             nodes:self.nodes,
//         }
//     }
// }

impl<'a> ExplorableRef<'a> {
    pub fn with(&self, rf: RefPtr) -> Self {
        Self {
            rf,
            nodes: self.nodes,
        }
    }
    /// in case a ref can branch ie. case of masking is a sort of branch
    fn iter(self) -> LabelValue {
        todo!()
        // let mut r = vec![];
        // self.ser(&mut r);
        // r.into()
    }
    pub fn bytes(self) -> Box<[u8]> {
        let mut r = vec![];
        self.ser(&mut r);
        r.into()
    }
}

impl<'a> ExplorableRef<'a> {
    fn ser(&self, out: &mut Vec<u8>) {
        match &self.nodes[self.rf] {
            RefsEnum::Root => out.extend(b"/"),
            RefsEnum::MaybeMissing => out.extend(b"?"),
            RefsEnum::Primitive(i) => {
                out.extend(b"p");
                out.extend(i.to_string().as_bytes())
            }
            RefsEnum::Array(o) => {
                assert_ne!(*o, self.rf);
                out.extend(b"[");
                self.with(*o).ser(out);
                out.extend(b"]");
            }
            RefsEnum::ArrayAccess(o) => {
                assert_ne!(*o, self.rf);
                self.with(*o).ser(out);
                out.extend(b"[?]");
            }
            RefsEnum::This(o) => {
                assert_ne!(*o, self.rf);
                self.with(*o).ser(out);
                out.extend(b".pthis");
            }
            RefsEnum::Super(o) => {
                assert_ne!(*o, self.rf);
                self.with(*o).ser(out);
                out.extend(b".psuper");
            }
            RefsEnum::Or(v) => {
                todo!()
                // out.extend(b"[|");
                // for p in v.iter() {
                //     assert_ne!(*p, self.rf);
                //     self.with(*p).ser(out);
                //     out.push(b"|"[0]);
                // }
                // out.extend(b"|]");
            }
            RefsEnum::Mask(o, v) => {
                assert_ne!(*o, self.rf);
                self.with(*o).ser(out);
                // out.extend(b".{");
                // for p in v.deref() {
                //     assert_ne!(*p, self.rf);
                //     self.with(*p).ser(out);
                //     out.push(b","[0]);
                // }
                // out.extend(b"}");
            }
            RefsEnum::ScopedIdentifier(o, i) => {
                assert_ne!(*o, self.rf);
                self.with(*o).ser(out);
                out.push(b"."[0]);
                // let b: [u8; 4] = (i.to_usize() as u32).to_be_bytes();
                let b = i.as_ref().to_usize().to_string();
                let b = b.as_bytes();
                out.extend(b);
            }
            RefsEnum::TypeIdentifier(o, i) => {
                assert_ne!(*o, self.rf);
                self.with(*o).ser(out);
                out.push(b"."[0]);
                // let b: [u8; 4] = (i.to_usize() as u32).to_be_bytes();
                let b = i.as_ref().to_usize().to_string();
                let b = b.as_bytes();
                out.extend(b);
            }
            RefsEnum::MethodReference(o, i) => {
                assert_ne!(*o, self.rf);
                self.with(*o).ser(out);
                out.extend(b"::");
                let b = i.as_ref().to_usize().to_string();
                let b = b.as_bytes();
                out.extend(b);
            }
            RefsEnum::ConstructorReference(o) => {
                assert_ne!(*o, self.rf);
                self.with(*o).ser(out);
                out.extend(b"::new");
            }
            RefsEnum::Invocation(o, i, p) => {
                assert_ne!(*o, self.rf);
                self.with(*o).ser(out);
                out.push(b"."[0]);
                let b = i.as_ref().to_usize().to_string();
                let b = b.as_bytes();
                out.extend(b);
                out.push(b"("[0]);
                match p {
                    Arguments::Unknown => out.extend(b"..."),
                    Arguments::Given(p) => {
                        for p in p.deref() {
                            assert_ne!(*p, self.rf);
                            self.with(*p).ser(out);
                            out.push(b","[0]);
                        }
                    }
                }
                out.push(b")"[0]);
            }
            RefsEnum::ConstructorInvocation(i, p) => {
                assert_ne!(*i, self.rf);
                self.with(*i).ser(out);
                out.extend(b"#(");
                match p {
                    Arguments::Unknown => out.extend(b"..."),
                    Arguments::Given(p) => {
                        for p in p.deref() {
                            assert_ne!(*p, self.rf);
                            self.with(*p).ser(out);
                            out.push(b","[0]);
                        }
                    }
                }
                out.push(b")"[0]);
            }
        }
    }

    fn ser_cached<'b>(&'b self, cache: &'b mut HashMap<RefPtr, Box<[u8]>>) -> &'b [u8] {
        if let Some(x) = cache.get(&self.rf) {
            // x.as_ref()
            todo!()
        } else {
            let mut out = vec![];
            match &self.nodes[self.rf] {
                RefsEnum::Root => out.extend(b"/"),
                RefsEnum::MaybeMissing => out.extend(b"?"),
                RefsEnum::Primitive(i) => {
                    out.extend(b"p");
                    out.extend(i.to_string().as_bytes())
                }
                RefsEnum::Array(o) => {
                    assert_ne!(*o, self.rf);
                    out.extend(b"[");
                    out.extend(self.with(*o).ser_cached(cache));
                    out.extend(b"]");
                }
                RefsEnum::ArrayAccess(o) => {
                    assert_ne!(*o, self.rf);
                    out.extend(self.with(*o).ser_cached(cache));
                    out.extend(b"[?]");
                }
                RefsEnum::This(o) => {
                    assert_ne!(*o, self.rf);
                    out.extend(self.with(*o).ser_cached(cache));
                    out.extend(b".pthis");
                }
                RefsEnum::Super(o) => {
                    assert_ne!(*o, self.rf);
                    out.extend(self.with(*o).ser_cached(cache));
                    out.extend(b".psuper");
                }
                RefsEnum::Or(v) => {
                    out.extend(b".[|");
                    for p in v.iter() {
                        assert_ne!(*p, self.rf);
                        out.extend(self.with(*p).ser_cached(cache));
                        out.push(b"|"[0]);
                    }
                    out.extend(b"|]");
                }
                RefsEnum::Mask(o, v) => {
                    assert_ne!(*o, self.rf);
                    out.extend(self.with(*o).ser_cached(cache));
                    // out.extend(b".{");
                    // for p in v.deref() {
                    //     assert_ne!(*p, self.rf);
                    //     out.extend(self.with(*p).ser_cached(cache));
                    //     out.push(b","[0]);
                    // }
                    // out.extend(b"}");
                }
                RefsEnum::ScopedIdentifier(o, i) => {
                    assert_ne!(*o, self.rf);
                    out.extend(self.with(*o).ser_cached(cache));
                    out.push(b"."[0]);
                    // let b: [u8; 4] = (i.to_usize() as u32).to_be_bytes();
                    let b = i.as_ref().to_usize().to_string();
                    let b = b.as_bytes();
                    out.extend(b);
                }
                RefsEnum::TypeIdentifier(o, i) => {
                    assert_ne!(*o, self.rf);
                    out.extend(self.with(*o).ser_cached(cache));
                    out.push(b"."[0]);
                    // let b: [u8; 4] = (i.to_usize() as u32).to_be_bytes();
                    let b = i.as_ref().to_usize().to_string();
                    let b = b.as_bytes();
                    out.extend(b);
                }
                RefsEnum::MethodReference(o, i) => {
                    assert_ne!(*o, self.rf);
                    out.extend(self.with(*o).ser_cached(cache));
                    out.extend(b"::");
                    let b = i.as_ref().to_usize().to_string();
                    let b = b.as_bytes();
                    out.extend(b);
                }
                RefsEnum::ConstructorReference(o) => {
                    assert_ne!(*o, self.rf);
                    out.extend(self.with(*o).ser_cached(cache));
                    out.extend(b"::new");
                }
                RefsEnum::Invocation(o, i, p) => {
                    assert_ne!(*o, self.rf);
                    out.extend(self.with(*o).ser_cached(cache));
                    out.push(b"."[0]);
                    let b = i.as_ref().to_usize().to_string();
                    let b = b.as_bytes();
                    out.extend(b);
                    out.push(b"("[0]);
                    match p {
                        Arguments::Unknown => out.extend(b"..."),
                        Arguments::Given(p) => {
                            for p in p.deref() {
                                assert_ne!(*p, self.rf);
                                out.extend(self.with(*p).ser_cached(cache));
                                out.push(b","[0]);
                            }
                        }
                    }
                    out.push(b")"[0]);
                }
                RefsEnum::ConstructorInvocation(i, p) => {
                    assert_ne!(*i, self.rf);
                    out.extend(self.with(*i).ser_cached(cache));
                    out.extend(b"#(");
                    match p {
                        Arguments::Unknown => out.extend(b"..."),
                        Arguments::Given(p) => {
                            for p in p.deref() {
                                assert_ne!(*p, self.rf);
                                out.extend(self.with(*p).ser_cached(cache));
                                out.push(b","[0]);
                            }
                        }
                    }
                    out.push(b")"[0]);
                }
            };
            cache.insert(self.rf, out.into_boxed_slice());
            cache.get(&self.rf).unwrap()
        }
    }
}

pub trait MySerializePar {
    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Must match the `Error` type of our `Serializer`.
    type Error: serde::ser::Error;

    /// Serialize a sequence element.
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: MySerialize + Keyed<usize>;

    /// Finish serializing a sequence.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}
pub trait MySerializeSco {
    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Must match the `Error` type of our `Serializer`.
    type Error: serde::ser::Error;

    /// Serialize a sequence element.
    fn serialize_object<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: MySerialize + Keyed<usize>;

    /// Finish serializing a sequence.
    fn end(self, s: &str) -> Result<Self::Ok, Self::Error>;
}

pub trait Keyed<T> {
    fn key(&self) -> T;
}

pub trait MySerializer: Sized {
    /// The output type produced by this `Serializer` during successful
    /// serialization. Most serializers that produce text or binary output
    /// should set `Ok = ()` and serialize into an [`io::Write`] or buffer
    /// contained within the `Serializer` instance. Serializers that build
    /// in-memory data structures may be simplified by using `Ok` to propagate
    /// the data structure around.
    ///
    /// [`io::Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
    type Ok;

    /// The error type when some error occurs during serialization.
    type Error: serde::ser::Error;

    /// Type returned from [`serialize_seq`] for serializing the content of the
    /// sequence.
    ///
    /// [`serialize_seq`]: #tymethod.serialize_seq
    type SerializePar: MySerializePar<Ok = Self::Ok, Error = Self::Error>;
    type SerializeSco: MySerializeSco<Ok = Self::Ok, Error = Self::Error>;

    fn collect_str<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Display,
    {
        todo!()
        // self.serialize_str(&value.to_string())
    }

    fn serialize_par(self, len: Option<usize>) -> Result<Self::SerializePar, Self::Error>;

    fn serialize_sco(self, len: Option<usize>) -> Result<Self::SerializeSco, Self::Error>;
}

pub trait MySerialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: MySerializer;
}

impl<'a> Keyed<usize> for ExplorableRef<'a> {
    fn key(&self) -> usize {
        self.rf
    }
}

impl<'a> MySerialize for ExplorableRef<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: MySerializer,
    {
        match &self.nodes[self.rf] {
            RefsEnum::Root => serializer.collect_str("/"),
            RefsEnum::MaybeMissing => serializer.collect_str("?"),
            RefsEnum::Primitive(i) => {
                let b = "p".to_string() + &i.to_string();
                serializer.collect_str(&b)
            }
            RefsEnum::Array(o) => {
                assert_ne!(*o, self.rf);
                let mut s = serializer.serialize_sco(Some(1))?;
                // serializer.collect_str("[")?;
                s.serialize_object(&self.with(*o))?;
                // serializer.collect_str("]")
                s.end("]")
            }
            RefsEnum::ArrayAccess(o) => {
                assert_ne!(*o, self.rf);
                let mut s = serializer.serialize_sco(Some(1))?;
                s.serialize_object(&self.with(*o))?;
                s.end("[?]")
            }
            RefsEnum::This(o) => {
                assert_ne!(*o, self.rf);
                let mut s = serializer.serialize_sco(Some(1))?;
                s.serialize_object(&self.with(*o))?;
                s.end(".pthis")
            }
            RefsEnum::Super(o) => {
                assert_ne!(*o, self.rf);
                let mut s = serializer.serialize_sco(Some(1))?;
                s.serialize_object(&self.with(*o))?;
                s.end(".psuper")
            }
            RefsEnum::Or(v) => {
                let mut s = serializer.serialize_par(Some(v.len()))?;
                for p in v.iter() {
                    assert_ne!(*p, self.rf);
                    s.serialize_element(&self.with(*p))?;
                }
                s.end()
            }
            RefsEnum::Mask(o, v) => {
                assert_ne!(*o, self.rf);
                self.with(*o).serialize(serializer)
            }
            RefsEnum::ScopedIdentifier(o, i) => {
                assert_ne!(*o, self.rf);
                let mut s = serializer.serialize_sco(Some(1))?;
                s.serialize_object(&self.with(*o))?;
                let b = ".".to_string() + &i.as_ref().to_usize().to_string();
                s.end(&b)
            }
            RefsEnum::TypeIdentifier(o, i) => {
                assert_ne!(*o, self.rf);
                let mut s = serializer.serialize_sco(Some(1))?;
                s.serialize_object(&self.with(*o))?;
                let b = ".".to_string() + &i.as_ref().to_usize().to_string();
                s.end(&b)
            }
            RefsEnum::MethodReference(o, i) => {
                assert_ne!(*o, self.rf);
                let mut s = serializer.serialize_sco(Some(1))?;
                s.serialize_object(&self.with(*o))?;
                let b = "::".to_string() + &i.as_ref().to_usize().to_string();
                s.end(&b)
            }
            RefsEnum::ConstructorReference(o) => {
                assert_ne!(*o, self.rf);
                let mut s = serializer.serialize_sco(Some(1))?;
                s.serialize_object(&self.with(*o))?;
                s.end("::new")
            }
            RefsEnum::Invocation(o, i, p) => {
                assert_ne!(*o, self.rf);
                // TODO handle executables fully
                let mut s = serializer.serialize_sco(Some(1))?;
                s.serialize_object(&self.with(*o))?;
                s.end("()")
            }
            RefsEnum::ConstructorInvocation(i, p) => {
                assert_ne!(*i, self.rf);
                // TODO handle executables fully
                let mut s = serializer.serialize_sco(Some(1))?;
                s.serialize_object(&self.with(*i))?;
                s.end("#()")
            }
        }
    }
}

impl<'a> Debug for ExplorableRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = vec![];
        self.ser(&mut out);
        write!(f, "{}", std::str::from_utf8(&out).unwrap())
    }
}

impl<'a> Into<Box<[u8]>> for ExplorableRef<'a> {
    fn into(self) -> Box<[u8]> {
        let mut r = vec![];
        self.ser(&mut r);
        r.into()
    }
}

#[derive(Debug, Clone)]
pub struct Nodes(Vec<RefsEnum<RefPtr, LabelPtr>>);

impl Index<RefPtr> for Nodes {
    type Output = RefsEnum<RefPtr, LabelPtr>;

    fn index(&self, index: RefPtr) -> &Self::Output {
        &self.0[index]
    }
}

impl From<Vec<RefsEnum<RefPtr, LabelPtr>>> for Nodes {
    fn from(x: Vec<RefsEnum<RefPtr, LabelPtr>>) -> Self {
        Self(x)
    }
}

impl Nodes {
    pub(crate) fn iter(&self) -> core::slice::Iter<'_, RefsEnum<RefPtr, LabelPtr>> {
        self.0.iter()
    }
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
    pub(crate) fn push(&mut self, x: RefsEnum<RefPtr, LabelPtr>) {
        self.0.push(x)
    }

    pub fn with(&self, rf: RefPtr) -> ExplorableRef {
        ExplorableRef { rf, nodes: self }
    }
}

pub struct NodesIter<'a> {
    pub(crate) rf: RefPtr,
    pub(crate) nodes: &'a Nodes,
}

impl<'a> Iterator for NodesIter<'a> {
    type Item = ExplorableRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rf >= self.nodes.len() {
            None
        } else {
            let r = ExplorableRef {
                rf: self.rf,
                nodes: &self.nodes,
            };
            self.rf += 1;
            Some(r)
        }
    }
}

#[derive(Debug)]
pub struct CachedHasherError(String);

impl Display for CachedHasherError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
impl From<std::io::Error> for CachedHasherError {
    fn from(e: std::io::Error) -> Self {
        Self(e.to_string())
    }
}
impl From<Utf8Error> for CachedHasherError {
    fn from(e: Utf8Error) -> Self {
        Self(e.to_string())
    }
}

impl std::error::Error for CachedHasherError {}

impl serde::ser::Error for CachedHasherError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self(msg.to_string())
    }
}

struct Table<T> {
    offsets: Vec<u32>,
    choices: Vec<u8>,
    buf: Vec<T>,
}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Self {
            offsets: Default::default(),
            choices: Default::default(),
            buf: Default::default(),
        }
    }
}

impl<T> Index<SymbolU16> for Table<T> {
    type Output = [T];

    fn index(&self, index: SymbolU16) -> &Self::Output {
        let index = index.to_usize();
        let o = self.offsets[index] as usize;
        let c = self.choices[index] as usize;
        &self.buf[o..o + c]
    }
}

impl<T> Table<T> {
    fn insert(&mut self, index: usize, v: Vec<T>) -> SymbolU16 {
        assert_ne!(v.len(), 0);
        if self.offsets.len() <= index {
            self.offsets.resize(index + 1, 0);
            self.choices.resize(index + 1, 0);
        }
        if self.offsets[index] != 0 {
            panic!();
        }
        self.offsets[index] = self.buf.len().to_u32().unwrap();
        self.choices[index] = v.len().to_u8().unwrap();
        self.buf.extend(v);
        SymbolU16::try_from_usize(index).unwrap()
    }
}

/// Could simplify structurally, fusioning Auxilary serializers
pub struct CachedHasher<'a, I, S, H: VaryHasher<S>> {
    index: I,
    table: &'a mut Table<S>,
    phantom: PhantomData<*const H>,
}

pub struct BulkHasher<'a, S, H: VaryHasher<S>> {
    table: Table<S>,
    it: NodesIter<'a>,
    branched: Vec<S>,
    phantom: PhantomData<*const H>,
}

impl<'a, S, H: VaryHasher<S>> From<NodesIter<'a>> for BulkHasher<'a, S, H> {
    fn from(it: NodesIter<'a>) -> Self {
        Self {
            table: Default::default(),
            it,
            branched: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<'a, H: VaryHasher<u8>> Iterator for BulkHasher<'a, u8, H> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if let Some(x) = self.branched.pop() {
            return Some(x);
        }
        let x = self.it.next()?;
        let x = x
            .serialize(CachedHasher::<usize, u8, H> {
                index: x.rf,
                table: &mut self.table,
                phantom: PhantomData,
            })
            .unwrap();
        let x = &self.table[x];
        self.branched = x.to_vec();
        self.next()
    }
}

impl<'a, H: VaryHasher<u16>> Iterator for BulkHasher<'a, u16, H> {
    type Item = u16;
    fn next(&mut self) -> Option<u16> {
        if let Some(x) = self.branched.pop() {
            return Some(x);
        }
        let x = self.it.next()?;
        let x = x
            .serialize(CachedHasher::<usize, u16, H> {
                index: x.rf,
                table: &mut self.table,
                phantom: PhantomData,
            })
            .unwrap();
        let x = &self.table[x];
        self.branched = x.to_vec();
        self.next()
    }
}

impl<'a, H: VaryHasher<u8>> MySerializer for CachedHasher<'a, usize, u8, H> {
    type Ok = SymbolU16; // TODO use an u8 symbol

    type Error = CachedHasherError;

    type SerializePar = CachedHasherAux<'a, usize, u8, H>;
    type SerializeSco = CachedHasherAux<'a, usize, u8, H>;

    fn serialize_par(self, _: Option<usize>) -> Result<Self::SerializePar, Self::Error> {
        Ok(CachedHasherAux {
            index: self.index,
            table: self.table,
            acc: Default::default(),
        })
    }

    fn serialize_sco(self, _: Option<usize>) -> Result<Self::SerializeSco, Self::Error> {
        Ok(CachedHasherAux {
            index: self.index,
            table: self.table,
            acc: Default::default(),
        })
    }
}

impl<'a, H: VaryHasher<u16>> MySerializer for CachedHasher<'a, usize, u16, H> {
    type Ok = SymbolU16;

    type Error = CachedHasherError;

    type SerializePar = CachedHasherAux<'a, usize, u16, H>;
    type SerializeSco = CachedHasherAux<'a, usize, u16, H>;

    fn serialize_par(self, _: Option<usize>) -> Result<Self::SerializePar, Self::Error> {
        Ok(CachedHasherAux {
            index: self.index,
            table: self.table,
            acc: Default::default(),
        })
    }

    fn serialize_sco(self, _: Option<usize>) -> Result<Self::SerializeSco, Self::Error> {
        Ok(CachedHasherAux {
            index: self.index,
            table: self.table,
            acc: Default::default(),
        })
    }
}

pub struct CachedHasherAux<'a, I, S, H: VaryHasher<S>> {
    index: I,
    table: &'a mut Table<S>,
    acc: Vec<H>,
}

impl<'a, H: VaryHasher<u8>> MySerializePar for CachedHasherAux<'a, usize, u8, H> {
    type Ok = SymbolU16;

    type Error = CachedHasherError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: MySerialize + Keyed<usize>,
    {
        let x = value.serialize(CachedHasher::<_, _, H> {
            index: value.key(),
            table: self.table,
            phantom: PhantomData,
        })?;
        for x in &self.table[x] {
            let x = *x;
            let mut h = H::new(0);
            h.write(b"|");
            h.write_u8(x);
            self.acc.push(h);
        }
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        for x in &mut self.acc {
            x.write(b"|]");
        }
        let v = self.acc.iter().map(VaryHasher::finish).collect();
        Ok(self.table.insert(self.index, v))
    }
}

impl<'a, H: VaryHasher<u16>> MySerializePar for CachedHasherAux<'a, usize, u16, H> {
    type Ok = SymbolU16;

    type Error = CachedHasherError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: MySerialize + Keyed<usize>,
    {
        let x = value.serialize(CachedHasher::<_, _, H> {
            index: value.key(),
            table: self.table,
            phantom: PhantomData,
        })?;
        for x in &self.table[x] {
            let x = *x;
            let mut h = H::new(0);
            h.write(b"|");
            h.write_u16(x);
            self.acc.push(h);
        }
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        for x in &mut self.acc {
            x.write(b"|]");
        }
        let v = self.acc.iter().map(VaryHasher::finish).collect();
        Ok(self.table.insert(self.index, v))
    }
}
impl<'a, H: VaryHasher<u8>> MySerializeSco for CachedHasherAux<'a, usize, u8, H> {
    type Ok = SymbolU16;

    type Error = CachedHasherError;

    fn serialize_object<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: MySerialize + Keyed<usize>,
    {
        let x = value.serialize(CachedHasher::<_, _, H> {
            index: value.key(),
            table: self.table,
            phantom: PhantomData,
        })?;
        for x in &self.table[x] {
            let x = *x;
            let mut h = H::new(0);
            h.write_u8(x);
            self.acc.push(h);
        }
        Ok(())
    }

    fn end(mut self, s: &str) -> Result<Self::Ok, Self::Error> {
        for x in &mut self.acc {
            x.write(s.as_bytes());
        }
        let v = self.acc.iter().map(VaryHasher::finish).collect();
        Ok(self.table.insert(self.index, v))
    }
}
impl<'a, H: VaryHasher<u16>> MySerializeSco for CachedHasherAux<'a, usize, u16, H> {
    type Ok = SymbolU16;

    type Error = CachedHasherError;

    fn serialize_object<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: MySerialize + Keyed<usize>,
    {
        let x = value.serialize(CachedHasher::<_, _, H> {
            index: value.key(),
            table: self.table,
            phantom: PhantomData,
        })?;
        for x in &self.table[x] {
            let x = *x;
            let mut h = H::new(0);
            h.write_u16(x);
            self.acc.push(h);
        }
        Ok(())
    }

    fn end(mut self, s: &str) -> Result<Self::Ok, Self::Error> {
        for x in &mut self.acc {
            x.write(s.as_bytes());
        }
        let v = self.acc.iter().map(VaryHasher::finish).collect();
        Ok(self.table.insert(self.index, v))
    }
}
