use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasher, BuildHasherDefault, Hash};
use std::ops::{Deref, Index};

use string_interner::{DefaultSymbol, Symbol};

use crate::store::vec_map_store::{self, VecMapStore};
use crate::utils::hash;

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
pub struct ListSet<Node: Eq + Hash>(Box<[Node]>);
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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum RefsEnum<Node: Eq + Hash, Leaf: Eq + Hash> {
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

impl<Node: Eq + Hash + Clone, Leaf: Eq + Hash> RefsEnum<Node, Leaf> {
    pub(crate) fn object(&self) -> Option<Node> {
        let r = match self {
            RefsEnum::Mask(o, _) => o,
            RefsEnum::ScopedIdentifier(o, _) => o,
            RefsEnum::MethodReference(o, _) => o,
            RefsEnum::ConstructorReference(o) => o,
            RefsEnum::Invocation(o, _, _) => o,
            RefsEnum::ConstructorInvocation(o, _) => o,
            RefsEnum::Array(o) => o,
            RefsEnum::This(o) => o,
            RefsEnum::Super(o) => o,
            RefsEnum::ArrayAccess(o) => o,
            _ => return None,
        };
        Some(r.clone())
    }
}

impl<Node: Eq + Hash + Clone, Leaf: Eq + Hash + Clone> RefsEnum<Node, Leaf> {
    pub(crate) fn with_object(&self, o: Node) -> Self {
        match self {
            RefsEnum::Mask(_, b) => RefsEnum::Mask(o, b.clone()),
            RefsEnum::ScopedIdentifier(_, i) => RefsEnum::ScopedIdentifier(o, i.clone()),
            RefsEnum::MethodReference(_, i) => RefsEnum::MethodReference(o, i.clone()),
            RefsEnum::ConstructorReference(_) => RefsEnum::ConstructorReference(o),
            RefsEnum::Invocation(_, i, p) => RefsEnum::Invocation(o, i.clone(), p.clone()),
            RefsEnum::ConstructorInvocation(_, p) => RefsEnum::ConstructorInvocation(o, p.clone()),
            RefsEnum::Array(_) => RefsEnum::Array(o),
            RefsEnum::This(_) => RefsEnum::This(o),
            RefsEnum::Super(_) => RefsEnum::Super(o),
            RefsEnum::ArrayAccess(_) => RefsEnum::ArrayAccess(o),
            _ => panic!(),
        }
    }
}

impl<Node: Eq + Hash, Leaf: Eq + Hash> RefsEnum<Node, Leaf> {
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
            (RefsEnum::ScopedIdentifier(_, i), RefsEnum::ScopedIdentifier(_, j)) => i == j,
            (RefsEnum::MethodReference(_, i), RefsEnum::MethodReference(_, j)) => i == j,
            (RefsEnum::ConstructorReference(i), RefsEnum::ConstructorReference(j)) => i == j,
            (RefsEnum::Invocation(_, i, _), RefsEnum::Invocation(_, j, _)) => i == j, // TODO count parameters
            (RefsEnum::ConstructorInvocation(_, _), RefsEnum::ConstructorInvocation(_, _)) => true, // TODO count parameters
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Arguments<Node = LabelValue>
where
    Node: Eq + Hash,
{
    Unknown,
    Given(Box<[Node]>),
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
                out.extend(b".[|");
                // for p in v.iter() {
                //     assert_ne!(*p, self.rf);
                //     self.with(*p).ser(out);
                //     out.push(b"|"[0]);
                // }
                out.extend(b"|]");
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

impl<'a> Debug for ExplorableRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = vec![];
        self.ser(&mut out);
        write!(f, "{}", std::str::from_utf8(&out).unwrap())
    }
}

impl<'a> Into<LabelValue> for ExplorableRef<'a> {
    fn into(self) -> LabelValue {
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
