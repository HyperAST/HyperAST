use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Deref, Index},
};

use bitvec::order::Lsb0;
use enumset::{enum_set, EnumSet, EnumSetType};
use rusted_gumtree_core::tree::tree::{LabelStore, Type};
use string_interner::{DefaultSymbol, StringInterner, Symbol};

use super::label_value::LabelValue;

type RefPtr = usize;
type LabelPtr = DefaultSymbol;

struct Iter<'a> {
    refs: bitvec::slice::IterOnes<'a, Lsb0, usize>,
    nodes: &'a Nodes,
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

struct NodesIter<'a> {
    rf: RefPtr,
    nodes: &'a Nodes,
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

struct DeclsIter<'a> {
    decls: std::collections::hash_map::Iter<'a, Declarator<usize>, DeclType<usize>>,
    nodes: &'a Nodes,
}

impl<'a> Iterator for DeclsIter<'a> {
    type Item = ExplorableDecl<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.decls.next() {
            Some(b) => {
                let r = ExplorableDecl {
                    decl: b,
                    nodes: self.nodes,
                };
                Some(r)
            }
            None => None,
        }
    }
}

struct ExplorableDecl<'a> {
    decl: (&'a Declarator<RefPtr>, &'a DeclType<RefPtr>),
    nodes: &'a Nodes,
}

struct DisplayDecl<'a, 'b, LS: LabelStore<str>> {
    decl: (&'a Declarator<RefPtr>, &'a DeclType<RefPtr>),
    nodes: &'a Nodes,
    leafs: &'b LS,
}

impl<'a, 'b, LS: LabelStore<str>> DisplayDecl<'a, 'b, LS> {
    fn with(&self, decl: (&'a Declarator<RefPtr>, &'a DeclType<RefPtr>)) -> Self {
        Self {
            decl,
            nodes: self.nodes,
            leafs: self.leafs,
        }
    }
}

impl<'a, 'b, LS: LabelStore<str>> From<(ExplorableDecl<'a>, &'b LS)> for DisplayDecl<'a, 'b, LS> {
    fn from((s, leafs): (ExplorableDecl<'a>, &'b LS)) -> Self {
        Self {
            decl: s.decl,
            nodes: s.nodes,
            leafs,
        }
    }
}

impl<'a, 'b, LS: LabelStore<str, I = LabelPtr>> Display for DisplayDecl<'a, 'b, LS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (k, v) = &self.decl;
        let kr = match k.node() {
            None => DisplayRef {
                leafs: self.leafs,
                rf: 0,
                nodes: &self.nodes,
            },
            Some(rf) => DisplayRef {
                leafs: self.leafs,
                rf: *rf,
                nodes: &self.nodes,
            },
        };
        match v {
            DeclType::Runtime(b) => {
                // TODO print more things
                write!(f, "   {:?}: {} = ", k, kr)?;
                for v in b.iter() {
                    let r = ExplorableRef {
                        rf: *v,
                        nodes: &self.nodes,
                    };
                    let r: DisplayRef<'a, 'b, LS> = (r, self.leafs).into();
                    write!(f, "+ {}", r)?;
                }
                Ok(())
            }
            DeclType::Compile(v, s, b) => {
                // TODO print more things
                let r = ExplorableRef {
                    rf: *v,
                    nodes: &self.nodes,
                };
                let r: DisplayRef<'a, 'b, LS> = (r, self.leafs).into();
                write!(f, "   {:?}: {} => {:?} {}", k, kr, v, r)?;
                if let Some(s) = s {
                    let s = DisplayRef {
                        leafs: self.leafs,
                        rf: *s,
                        nodes: &self.nodes,
                    };
                    write!(f, " extends {}", s)?;
                };
                if b.len() > 0 {
                    write!(f, " implements {:?}", s)?;
                }
                for v in b.iter() {
                    let v = DisplayRef {
                        leafs: self.leafs,
                        rf: *v,
                        nodes: &self.nodes,
                    };
                    write!(f, " {}, ", v)?;
                }
                Ok(())
            }
        }
    }
}

pub struct ExplorableRef<'a> {
    pub rf: RefPtr,
    pub nodes: &'a Nodes,
}

impl<'a> AsRef<RefsEnum<RefPtr, LabelPtr>> for ExplorableRef<'a> {
    fn as_ref(&self) -> &RefsEnum<RefPtr, LabelPtr> {
        &self.nodes[self.rf]
    }
}

impl<'a> Clone for ExplorableRef<'a> {
    fn clone(&self) -> Self {
        Self {
            rf:self.rf,
            nodes:self.nodes,
        }
    }
}

impl<'a> ExplorableRef<'a> {
    pub fn with(&self, rf: RefPtr) -> Self {
        Self {
            rf,
            nodes: self.nodes,
        }
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
                let b = i.to_usize().to_string();
                let b = b.as_bytes();
                out.extend(b);
            }
            RefsEnum::MethodReference(o, i) => {
                assert_ne!(*o, self.rf);
                self.with(*o).ser(out);
                out.extend(b"::");
                let b = i.to_usize().to_string();
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
                let b = i.to_usize().to_string();
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

struct DisplayRef<'a, 'b, LS: LabelStore<str>> {
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

impl<'a, 'b, LS: LabelStore<str, I = LabelPtr>> Display for DisplayRef<'a, 'b, LS> {
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
            RefsEnum::ScopedIdentifier(o, i) => {
                write!(f, "{}", self.with(*o))?;
                write!(f, ".{}", self.leafs.resolve(i))
            }
            RefsEnum::MethodReference(o, i) => {
                write!(f, "{}", self.with(*o))?;
                write!(f, "::{}", self.leafs.resolve(i))
            }
            RefsEnum::ConstructorReference(o) => {
                write!(f, "{}::new", self.with(*o))
            }
            RefsEnum::Invocation(o, i, a) => {
                write!(f, "{}", self.with(*o))?;
                write!(f, ".{}", self.leafs.resolve(i))?;
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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum RefsEnum<Node: Eq + Hash, Leaf: Eq + Hash> {
    Root,
    MaybeMissing, // TODO replace ? with ~
    ScopedIdentifier(Node, Leaf),
    // TODO ArrayAccess(Node)
    // TODO Anonymous(Id)
    // no need instance of type for cases where there is a cast ie. to access static members as static do not overload .ie thus error
    MethodReference(Node, Leaf), // equivalent to Invocation(Node, Leaf, Arguments::Unknown) but it does not represent a call that is actually made
    ConstructorReference(Node), // equivalent to ConstructorInvocation(Node, Arguments::Unknown) but it does not represent a call that is actually made
    Invocation(Node, Leaf, Arguments<Node>),
    ConstructorInvocation(Node, Arguments<Node>), // equivalent to Invocation(Node, 'new', Arguments<Node>)

    // specific to java
    Primitive(Primitive),
    Array(Node),
    This(Node),
    Super(Node),
    ArrayAccess(Node),
    /// rest of resolution and rest of refs masking it
    Mask(Node, Box<[Node]>),
}

impl<Node: Eq + Hash + Clone, Leaf: Eq + Hash> RefsEnum<Node, Leaf> {
    fn object(&self) -> Option<Node> {
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
    fn with_object(&self, o: Node) -> Self {
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Primitive {
    Double,
    Float,
    Long,
    Int,
    Char,
    Short,
    Byte,
    Boolean,
    Null,
    Void,
}

impl From<Type> for Primitive {
    fn from(s: Type) -> Self {
        match s {
            Type::BooleanType => Self::Boolean,
            Type::VoidType => Self::Void,
            Type::FloatingPointType => Self::Float,
            Type::IntegralType => Self::Int,
            // Literals
            Type::True => Self::Boolean,
            Type::False => Self::Boolean,
            Type::OctalIntegerLiteral => Self::Int,
            Type::BinaryIntegerLiteral => Self::Int,
            Type::DecimalIntegerLiteral => Self::Int,
            Type::HexFloatingPointLiteral => Self::Float,
            Type::DecimalFloatingPointLiteral => Self::Float,
            Type::HexIntegerLiteral => Self::Float,
            Type::StringLiteral => panic!("{:?}", s),
            Type::CharacterLiteral => Self::Char,
            Type::NullLiteral => Self::Null,
            _ => panic!("{:?}", s),
        }
    }
}

impl From<&str> for Primitive {
    fn from(s: &str) -> Self {
        match s {
            "boolean" => Self::Boolean,
            "void" => Self::Void,
            "float" => Self::Float,
            "double" => Self::Double,
            "byte" => Self::Byte,
            "char" => Self::Char,
            "short" => Self::Short,
            "int" => Self::Int,
            "long" => Self::Long,
            // Literals
            "true" => Self::Boolean,
            "false" => Self::Boolean,
            "null" => Self::Null,
            s => panic!("{:?}", s),
        }
    }
}

impl Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Primitive::Double => "double",
                Primitive::Float => "float",
                Primitive::Long => "long",
                Primitive::Int => "int",
                Primitive::Char => "char",
                Primitive::Short => "short",
                Primitive::Byte => "byte",
                Primitive::Boolean => "boolean",
                Primitive::Null => "null",
                Primitive::Void => "void",
            }
        )
    }
}

trait SubTyping: PartialOrd {}

impl PartialOrd for Primitive {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        let r = match (self, other) {
            (x, y) if x == y => Some(Ordering::Equal),
            (Primitive::Double, Primitive::Double) => Some(Ordering::Equal),
            // double >1 float
            (Primitive::Double, Primitive::Float) => Some(Ordering::Greater),
            (Primitive::Float, Primitive::Float) => Some(Ordering::Equal),
            // float >1 long
            (Primitive::Float, Primitive::Long) => Some(Ordering::Greater),
            (Primitive::Long, Primitive::Long) => Some(Ordering::Equal),
            // long >1 int
            (Primitive::Long, Primitive::Int) => Some(Ordering::Greater),
            (Primitive::Int, Primitive::Int) => Some(Ordering::Equal),
            // int >1 char
            (Primitive::Int, Primitive::Char) => Some(Ordering::Greater),
            // int >1 short
            (Primitive::Int, Primitive::Short) => Some(Ordering::Greater),
            (Primitive::Char, Primitive::Char) => Some(Ordering::Equal),
            (Primitive::Short, Primitive::Short) => Some(Ordering::Equal),
            // short >1 byte
            (Primitive::Short, Primitive::Byte) => Some(Ordering::Greater),
            (Primitive::Byte, Primitive::Byte) => Some(Ordering::Equal),
            (Primitive::Boolean, Primitive::Boolean) => Some(Ordering::Equal),
            (Primitive::Null, Primitive::Null) => Some(Ordering::Equal),
            (Primitive::Void, Primitive::Void) => Some(Ordering::Equal),
            _ => None,
        };
        if r.is_none() {
            other.partial_cmp(self).map(Ordering::reverse)
        } else {
            r
        }
    }
}

impl SubTyping for Primitive {}

impl<Node: Eq + Hash, Leaf: Eq + Hash> RefsEnum<Node, Leaf> {
    fn similar(&self, other: &Self) -> bool {
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

// impl<Node:Eq + Hash + Clone> Clone for Arguments<Node> {
//     fn clone(&self) -> Self {
//         match self {
//             Self::Unknown => Self::Unknown,
//             Self::Given(arg0) => Self::Given(arg0.clone()),
//         }
//     }
// }

pub fn leaf_state(t: &Type, label: Option<LabelPtr>) -> State<RefPtr, LabelPtr> {
    let r = if t == &Type::Comment {
        State::None
    } else if t.is_primitive() {
        // State::SimpleTypeIdentifier(label.unwrap())
        panic!("{:?} {:?}", t, label);
    } else if t.is_literal() {
        // State::LiteralType(label.unwrap())
        panic!("{:?} {:?}", t, label);
    } else if t == &Type::ScopedIdentifier {
        panic!();
    } else if t == &Type::ScopedTypeIdentifier {
        panic!();
    } else if t == &Type::Asterisk {
        State::Asterisk
    } else if t == &Type::ArgumentList {
        State::Arguments(vec![])
    } else if t == &Type::FormalParameters {
        State::FormalParameters(vec![])
    } else if t == &Type::Super {
        panic!("{:?} {:?}", t, label);
    } else if t == &Type::This {
        //t.is_instance_ref() {
        panic!("{:?} {:?}", t, label);
    } else if t == &Type::TypeIdentifier {
        State::SimpleTypeIdentifier(label.unwrap())
    } else if t.is_identifier() {
        State::SimpleIdentifier(label.unwrap())
    } else if t == &Type::Spaces {
        State::None
    } else if t == &Type::Block {
        State::None
    } else if t == &Type::ElementValueArrayInitializer {
        State::None
    } else if t == &Type::Dimensions {
        State::Dimensions
    } else if t == &Type::TS86 {
        State::Modifiers(Visibility::None, enum_set!(NonVisibility::Static))
    } else if t == &Type::TS81 {
        State::Modifiers(Visibility::Public, enum_set!())
    } else {
        assert_eq!(t, &Type::Comment);
        State::Todo
    };
    // println!("init: {:?} {:?}", t, r);
    r
}

#[derive(Debug)]
pub struct PartialAnalysis {
    current_node: State<RefPtr, LabelPtr>,
    pub solver: Solver,
}

#[derive(Debug, Clone)]
pub struct Nodes(Vec<RefsEnum<RefPtr, LabelPtr>>);

impl Index<RefPtr> for Nodes {
    type Output = RefsEnum<RefPtr, LabelPtr>;

    fn index(&self, index: RefPtr) -> &Self::Output {
        &self.0[index]
    }
}

impl Nodes {
    fn iter(&self) -> core::slice::Iter<'_, RefsEnum<RefPtr, LabelPtr>> {
        self.0.iter()
    }
    fn len(&self) -> usize {
        self.0.len()
    }
    fn push(&mut self, x: RefsEnum<RefPtr, LabelPtr>) {
        self.0.push(x)
    }
}

#[derive(Debug, Clone)]
pub struct Solver {
    // leafs: LeafSet,
    pub nodes: Nodes,
    refs: bitvec::vec::BitVec,
    decls: HashMap<Declarator<RefPtr>, DeclType<RefPtr>>,
    // root: Option<RefPtr>,
}

impl Default for Solver {
    fn default() -> Self {
        Self {
            // leafs: Default::default(),
            nodes: Nodes(vec![RefsEnum::Root, RefsEnum::MaybeMissing]),
            refs: Default::default(),
            decls: Default::default(),
            // root: None,
        }
    }
}
// SymbolU32 { value: 1 }:"A"
// SymbolU32 { value: 2 }:"int"
// SymbolU32 { value: 3 }:"a"
// SymbolU32 { value: 4 }:"0"
// SymbolU32 { value: 5 }:"void"
// SymbolU32 { value: 6 }:"test"
// SymbolU32 { value: 7 }:"x"
// SymbolU32 { value: 8 }:"// a;"
// SymbolU32 { value: 9 }:"b"
// SymbolU32 { value: 10 }:"B"
// SymbolU32 { value: 11 }:"c"

struct MultiResult<T>(Option<Box<[T]>>);

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

impl<T> FromIterator<T> for MultiResult<T> {
    fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
        let mut r = Default::default();
        let b: Box<[T]> = iter.into_iter().collect();
        if !b.is_empty() {
            r = Some(b);
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
    fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter().flat_map(|x| x.iter())
    }

    fn is_empty(&self) -> bool {
        self.0.is_none()
    }
}

impl Solver {
    fn is_length_token(&self, id: LabelPtr) -> bool {
        id.to_usize() == 0 // TODO verify/model statically
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
    fn get(&self, other: RefsEnum<RefPtr, LabelPtr>) -> Option<RefPtr> {
        // TODO analyze perfs to find if Vec or HashSet or something else works better
        self.nodes.iter().position(|x| x == &other)
    }

    fn intern_ref(&mut self, other: RefsEnum<RefPtr, LabelPtr>) -> RefPtr {
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

    fn add_decl(&mut self, d: Declarator<RefPtr>, t: DeclType<RefPtr>) {
        self.decls.insert(d, t);
    }
    fn add_decl_simple(&mut self, d: Declarator<RefPtr>, t: RefPtr) {
        self.decls
            .insert(d, DeclType::Compile(t, None, Default::default()));
    }

    pub(crate) fn solve_node_with(&mut self, t: usize, p: usize) -> usize {
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
            RefsEnum::Root => panic!("fully qualified node cannot be qualified further"),
            RefsEnum::MaybeMissing => p,
            RefsEnum::Array(i) => {
                let x = self.solve_node_with(i, p);
                let tmp = RefsEnum::Array(x);
                refs!(tmp)
            }
            RefsEnum::ArrayAccess(i) => {
                let x = self.solve_node_with(i, p);
                let tmp = RefsEnum::ArrayAccess(x);
                refs!(tmp)
            }
            RefsEnum::This(i) => {
                let x = self.solve_node_with(i, p);
                let tmp = RefsEnum::This(x);
                refs!(tmp)
            }
            RefsEnum::Super(i) => {
                let x = self.solve_node_with(i, p);
                let tmp = RefsEnum::Super(x);
                refs!(tmp)
            }
            RefsEnum::Mask(i, y) => {
                let x = self.solve_node_with(i, p); // TODO not sure
                let tmp = RefsEnum::Mask(x, y);
                refs!(tmp)
            }
            RefsEnum::ScopedIdentifier(i, y) => {
                let x = self.solve_node_with(i, p);
                let tmp = RefsEnum::ScopedIdentifier(x, y);
                refs!(tmp)
            }
            RefsEnum::Invocation(o, i, args) => {
                let x = self.solve_node_with(o, p);
                let tmp = RefsEnum::Invocation(x, i, args);
                refs!(tmp)
            }
            RefsEnum::ConstructorInvocation(o, args) => {
                let x = self.solve_node_with(o, p);
                let tmp = RefsEnum::ConstructorInvocation(x, args);
                refs!(tmp)
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
                ExplorableRef {
                    rf: x,
                    nodes: &self.nodes,
                }
                .into()
            })
    }

    fn iter_refs<'a>(&'a self) -> Iter<'a> {
        Iter {
            nodes: &self.nodes,
            refs: self.refs.iter_ones(),
        }
    }

    fn iter_decls<'a>(&'a self) -> DeclsIter<'a> {
        DeclsIter {
            nodes: &self.nodes,
            decls: self.decls.iter(),
        }
    }

    fn iter_nodes<'a>(&'a self) -> NodesIter<'a> {
        NodesIter {
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
                panic!()
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
    fn solve_aux(
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
        let other = if let Some(r) = (&self.decls).get(&Declarator::Field(no_mask)).cloned() {
            match r {
                DeclType::Compile(r, _, _) => r,
                _ => todo!(),
            }
        } else if let Some(r) = (&self.decls).get(&Declarator::Type(no_mask)).cloned() {
            match r {
                DeclType::Compile(r, _, _) => r,
                DeclType::Runtime(b) => {
                    return Default::default();
                }
                x => todo!("{:?}", x),
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
                let v: Vec<RefPtr> = v
                    .iter()
                    .flat_map(|x| {
                        self.solve_aux(cache, *x)
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
            RefsEnum::ScopedIdentifier(o, i) => {
                let mut m: Option<Box<[usize]>> = None; // TODO use mask
                let r: MultiResult<RefPtr> = self.solve_aux(cache, o);
                if r.is_empty() {
                    // println!("solving {:?} an object into nothing", o);
                    cache.insert(other, r);
                    return Default::default();
                }
                let r: MultiResult<RefPtr> = r
                    .iter()
                    .map(|o| {
                        let o = *o;
                        let matched = match &self.nodes[o] {
                            // if x[].lenght -> int
                            RefsEnum::Array(_) if self.is_length_token(i) => None,
                            RefsEnum::Mask(o, x) => {
                                m = Some(x.clone()); // TODO use mask
                                Some(*o)
                            }
                            _ => Some(o),
                        };
                        let o = if let Some(m) = &m {
                            for m in m.iter() {
                                let no_mask = self.no_mask(*m);
                                let no_mask = self.intern(RefsEnum::ScopedIdentifier(no_mask, i));
                                let x = self.solve_aux(cache, no_mask);
                                if !x.is_empty() {
                                    let x = *x.iter().next().unwrap();
                                    if x != no_mask {
                                        return x
                                    }
                                }
                            }
                            Some(o)
                        } else {matched};
                        if let Some(o) = o {
                            self.intern(RefsEnum::ScopedIdentifier(o, i))
                        } else {
                            self.intern(RefsEnum::Primitive(Primitive::Int))
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
                                        self.solve_aux(cache, s.unwrap())
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
                    if let Some(s) = s {
                        let s = ExplorableRef {
                            rf: *s,
                            nodes: &self.nodes,
                        };
                        print!(" extends {:?}", s);
                    };
                    if b.len() > 0 {
                        print!(" implements {:?}", s);
                    }
                    for v in b.iter() {
                        let v = ExplorableRef {
                            rf: *v,
                            nodes: &self.nodes,
                        };
                        print!(" {:?}, ", v);
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

    fn resolve(
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
    fn intern_external(&mut self, solver: &mut Solver, other: RefPtr) -> RefPtr {
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

impl Default for PartialAnalysis {
    fn default() -> Self {
        Self {
            current_node: State::None,
            solver: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclType<Node> {
    Runtime(Box<[Node]>), // Typically erased types
    Compile(Node, Option<Node>, Box<[Node]>),
}

impl<Node: Copy> DeclType<Node> {
    pub fn map<N, FN: Fn(Node) -> N>(&self, f: FN) -> DeclType<N>
where {
        match self {
            DeclType::Runtime(b) => DeclType::Runtime(b.iter().map(|x| f(*x)).collect()),
            DeclType::Compile(t, s, i) => {
                DeclType::Compile(f(*t), s.map(|x| f(x)), i.iter().map(|x| f(*x)).collect())
            }
        }
    }
}

impl PartialAnalysis {
    // apply before commiting/saving subtree
    pub fn resolve(mut self) -> Self {
        let mut cache: HashMap<usize, MultiResult<RefPtr>> = Default::default();
        if let State::File {
            asterisk_imports,
            package,
            ..
        } = self.current_node.clone()
        {
            let mm = self.solver.intern(RefsEnum::MaybeMissing);
            let mask = self.solver.intern(RefsEnum::Mask(mm, Default::default()));
            let jlang = asterisk_imports[0];

            if let Some(package) = package {
                if asterisk_imports.is_empty() {
                    // self.solver.root = package;
                    // cache.insert(mm, Some(package));
                    panic!();
                } else {
                    cache.insert(
                        mask,
                        [package]
                            .iter()
                            .map(|x| *x)
                            .collect::<MultiResult<RefPtr>>(),
                    );
                    if package == jlang {
                        let a = asterisk_imports[1..].iter().map(|imp| {
                            self.solver
                                .intern(RefsEnum::Mask(*imp, vec![package].into_boxed_slice()))
                        });
                        let a = a.chain([package].into_iter());
                        cache.insert(mm, a.collect());
                    } else {
                        let a = asterisk_imports.iter().map(|imp| {
                            self.solver
                                .intern(RefsEnum::Mask(*imp, vec![package].into_boxed_slice()))
                        });
                        let a = a.chain([package].into_iter());
                        cache.insert(mm, a.collect());
                    };
                }
            } else {
                cache.insert(
                    mask,
                    [mm].iter().map(|x| *x).collect::<MultiResult<RefPtr>>(),
                );
            }
        }

        let (_, mut solver) = self.solver.resolve(cache);
        match &self.current_node {
            State::File { .. } => {
                let mut r = bitvec::vec::BitVec::<Lsb0, usize>::default();
                r.resize(solver.refs.len(), false);
                let mm = solver.intern(RefsEnum::MaybeMissing);
                for i in solver.refs.iter_ones() {
                    match solver.nodes[i] {
                        RefsEnum::ConstructorInvocation(o, _) if o == mm => {
                            panic!();
                        } // not possible ?
                        RefsEnum::Invocation(o, _, _) if o == mm => {}
                        _ => {
                            r.set(i, true);
                        }
                    }
                }
                // TODO also remove the ones that refs the one s removed as they cannot really be resolved anymore
                solver.refs = r;
            }
            _ => (),
        };

        Self {
            current_node: self.current_node,
            solver,
        }
    }

    pub fn refs(&self) -> impl Iterator<Item = LabelValue> + '_ {
        self.solver.refs()
    }
    pub fn display_refs<'a, LS: LabelStore<str, I = LabelPtr>>(
        &'a self,
        leafs: &'a LS,
    ) -> impl Iterator<Item = impl Display + 'a> + 'a {
        self.solver.iter_refs().map(move |x| {
            let r: DisplayRef<LS> = (x, leafs).into();
            r
        })
    }

    pub fn print_refs<LS: LabelStore<str, I = LabelPtr>>(&self, leafs: &LS) {
        for x in self.display_refs(leafs) {
            println!("    {}", x);
        }
    }

    pub fn refs_count(&self) -> usize {
        self.solver.refs.count_ones()
    }

    pub fn print_decls<LS: LabelStore<str, I = LabelPtr>>(&self, leafs: &LS) {
        let it = self.solver.iter_decls().map(move |x| {
            let r: DisplayDecl<LS> = (x, leafs).into();
            r
        });
        for x in it {
            println!("    {}", x);
        }
    }

    pub fn decls_count(&self) -> usize {
        self.solver.decls.len()
    }

    pub fn init<F: FnMut(&str) -> LabelPtr>(
        kind: &Type,
        label: Option<&str>,
        mut intern_label: F,
    ) -> Self {
        let mut solver: Solver = Default::default();
        if kind == &Type::Directory {
            PartialAnalysis {
                current_node: State::None,
                solver,
            }
        } else if kind == &Type::Program {
            // default_imports(&mut solver, intern_label);

            let i = solver.intern(RefsEnum::Root);
            let i = solver.intern(RefsEnum::ScopedIdentifier(i, intern_label("java")));
            let i = solver.intern(RefsEnum::ScopedIdentifier(i, intern_label("lang")));
            PartialAnalysis {
                current_node: State::File {
                    package: None,
                    asterisk_imports: vec![i],
                    top_level: None,
                    content: vec![],
                },
                solver,
            }
        } else if kind == &Type::PackageDeclaration {
            // default_imports(&mut solver, |x| intern_label(x));

            // let i = solver.intern(RefsEnum::Root);
            // let i = solver.intern(RefsEnum::ScopedIdentifier(i, intern_label("java")));
            // let i = solver.intern(RefsEnum::ScopedIdentifier(i, intern_label("lang")));
            PartialAnalysis {
                current_node: State::None, //ScopedIdentifier(i),
                solver,
            }
        } else if kind == &Type::This {
            let i = solver.intern(RefsEnum::MaybeMissing);
            let i = solver.intern(RefsEnum::This(i));
            PartialAnalysis {
                current_node: State::This(i),
                solver,
            }
        } else if kind == &Type::Super {
            let i = solver.intern(RefsEnum::MaybeMissing);
            let i = solver.intern(RefsEnum::Super(i));
            PartialAnalysis {
                current_node: State::Super(i),
                solver,
            }
        } else if kind.is_literal() {
            let i = if kind == &Type::StringLiteral {
                let i = solver.intern(RefsEnum::Root);
                let i = solver.intern(RefsEnum::ScopedIdentifier(i, intern_label("java")));
                let i = solver.intern(RefsEnum::ScopedIdentifier(i, intern_label("lang")));
                let i = solver.intern(RefsEnum::ScopedIdentifier(i, intern_label("String")));
                i
            } else {
                let p = Primitive::from(label.unwrap());
                let i = solver.intern(RefsEnum::Primitive(p));
                i
            };
            PartialAnalysis {
                current_node: State::LiteralType(i),
                solver,
            }
        } else if kind.is_primitive() {
            // println!("{:?}", label);
            let p = Primitive::from(label.unwrap());
            let i = solver.intern(RefsEnum::Primitive(p));
            // let i = label.unwrap();
            // let t = solver.intern(RefsEnum::MaybeMissing);
            // let i = solver.intern(RefsEnum::ScopedIdentifier(t, i));
            PartialAnalysis {
                current_node: State::ScopedTypeIdentifier(i),
                solver,
            }
            // panic!("{:?} {:?}",kind,label);
        } else if kind.is_type_declaration() {
            let r = solver.intern(RefsEnum::Root);
            let i = solver.intern(RefsEnum::ScopedIdentifier(r, intern_label("java")));
            let i = solver.intern(RefsEnum::ScopedIdentifier(i, intern_label("lang")));
            let s = solver.intern(RefsEnum::ScopedIdentifier(i, intern_label("Object")));

            let d = solver.intern(RefsEnum::Super(r));
            let d = solver.intern(RefsEnum::ConstructorInvocation(d, Arguments::Unknown));
            let d = Declarator::Executable(d);
            solver.add_decl_simple(d, s);

            PartialAnalysis {
                current_node: State::TypeDeclaration {
                    visibility: Visibility::None,
                    identifier: DeclType::Compile(0, Some(s), vec![].into_boxed_slice()),
                    members: vec![],
                },
                solver,
            }
        } else if kind == &Type::ClassBody {
            // TODO constructor solve
            // {
            //     let t = solver.intern(RefsEnum::MaybeMissing);
            //     let t = solver.intern(RefsEnum::This(t));
            //     let i = solver.intern(RefsEnum::ConstructorInvocation(t,Arguments::Given(vec![].into_boxed_slice())));
            //     let t = solver.intern(RefsEnum::Root);
            //     let t = solver.intern(RefsEnum::This(t));
            //     let d = Declarator::Executable(i);
            //     solver.add_decl_simple(d, t);
            // }
            // {
            //     let t = solver.intern(RefsEnum::MaybeMissing);
            //     let i = solver.intern(RefsEnum::ConstructorInvocation(t,Arguments::Given(vec![].into_boxed_slice())));
            //     let t = solver.intern(RefsEnum::Root);
            //     let t = solver.intern(RefsEnum::This(t));
            //     let d = Declarator::Executable(i);
            //     solver.add_decl_simple(d, t);
            // }

            PartialAnalysis {
                current_node: State::None,
                solver,
            }
        } else {
            let label = label.map(|x| intern_label(x));
            PartialAnalysis {
                current_node: leaf_state(kind, label),
                solver,
            }
        }
    }

    pub fn acc(self, kind: &Type, acc: &mut Self) {
        let current_node = self.current_node;
        println!(
            "{:?} {:?} {:?}\n**{:?}",
            &kind,
            &acc.current_node,
            &current_node,
            acc.refs().collect::<Vec<_>>()
        );

        macro_rules! mm {
            () => {
                acc.solver.intern(RefsEnum::MaybeMissing)
            };
        }
        macro_rules! scoped {
            ( $o:expr, $i:expr ) => {{
                let o = $o;
                acc.solver.intern_ref(RefsEnum::ScopedIdentifier(o, $i))
            }};
        }
        macro_rules! spec {
            ( $o:expr, $i:expr ) => {{
                let i = $i;
                let o = $o;
                match acc.solver.nodes[i].clone() {
                    RefsEnum::This(i) => {
                        assert_eq!(i, mm!());
                        acc.solver.intern_ref(RefsEnum::This(o))
                    }
                    RefsEnum::Super(i) => {
                        assert_eq!(i, mm!());
                        acc.solver.intern_ref(RefsEnum::Super(o))
                    }
                    x => panic!("{:?}", x),
                }
            }};
        }

        #[derive(Debug, PartialEq, Eq, Clone, Hash)]
        struct Old<T>(T)
        where
            T: std::cmp::Eq + std::hash::Hash + Clone;

        //main organization top down, through type kind
        acc.current_node = if kind == &Type::Error {
            panic!("{:?} {:?} {:?}", kind, acc.current_node, current_node)
        } else if kind == &Type::Program {
            // TODO should do things with RefsEnum:Mask
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }
            match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                (
                    State::File {
                        package: p,
                        asterisk_imports,
                        top_level,
                        mut content,
                    },
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: d,
                    },
                ) => {
                    // no package declaration at start of java file
                    content.push((d.with_changed_node(|x| sync!(x)), sync!(t)));
                    State::File {
                        package: p,
                        asterisk_imports,
                        top_level,
                        content,
                    }
                }
                (
                    State::File {
                        package: None,
                        asterisk_imports,
                        top_level,
                        mut content,
                    },
                    State::PackageDeclaration(p),
                ) => {
                    // for (d, t) in &self.solver.decls {
                    //     let d = d.with_changed_node(|x| sync!(Old(*x)));
                    //     let t = match t {
                    //         DeclType::Runtime(b) => {
                    //             DeclType::Runtime(b.iter().map(|x| sync!(Old(*x))).collect())
                    //         }
                    //         DeclType::Compile(t, s, i) => DeclType::Compile(
                    //             sync!(Old(*t)),
                    //             s.map(|x| sync!(Old(x))),
                    //             i.iter().map(|x| sync!(Old(*x))).collect(),
                    //         ),
                    //     };
                    //     acc.solver.add_decl(d, t);
                    // }
                    State::File {
                        package: Some(sync!(p)),
                        asterisk_imports,
                        top_level,
                        content,
                    }
                }
                (
                    State::File {
                        package: p,
                        asterisk_imports,
                        top_level,
                        mut content,
                    },
                    State::None,
                ) if kind == &Type::Program => State::None,
                // (
                //     State::File {
                //         package: p,
                //         asterisk_imports,
                //         top_level,
                //         mut content,
                //     },
                //     State::TypeDeclaration {
                //         visibility,
                //         identifier: d,
                //         members: _,
                //     },
                // ) => {
                //     // TODO check for file's class? visibility ? etc
                //     // TODO maybe bubleup members
                //     let top_level = match d {
                //         DeclType::Compile(d, _, _) => {
                //             let d = sync!(d);
                //             let i = Declarator::Type(d);
                //             content.push((i.clone(), d));
                //             acc.solver.add_decl_simple(i.clone(), d);
                //             if let Visibility::Public = visibility {
                //                 Some((i, d))
                //             } else {
                //                 None
                //             }
                //         }
                //         _ => panic!(),
                //     };

                //     State::File {
                //         package: p,
                //         asterisk_imports,
                //         top_level,
                //         content,
                //     }
                // }
                (
                    State::File {
                        package: p,
                        mut asterisk_imports,
                        top_level,
                        mut content,
                    },
                    State::ImportDeclaration {
                        identifier: i,
                        sstatic,
                        asterisk,
                    },
                ) => {
                    // TODO do something with sstatic and asterisk
                    let d = sync!(i);
                    let (o, i) = match &acc.solver.nodes[d] {
                        RefsEnum::ScopedIdentifier(o, i) => (*o, *i),
                        _ => panic!("must be a scoped id in an import"),
                    };
                    if asterisk {
                        // TODO static
                        asterisk_imports.push(d);
                    } else {
                        let c = scoped!(o, i);
                        let r = mm!();
                        let d = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                        acc.solver.add_decl_simple(Declarator::Type(d), c);
                    }
                    State::File {
                        package: p,
                        asterisk_imports,
                        top_level,
                        content,
                    }
                }
                (
                    State::File {
                        package: p,
                        asterisk_imports,
                        top_level,
                        mut content,
                    },
                    State::TypeDeclaration {
                        visibility,
                        identifier,
                        members,
                    },
                ) => {
                    // check for file's class? visibility? etc
                    // TODO maybe bubleup members
                    // remove asterisk import if declared in file
                    let identifier = match (identifier, p) {
                        (DeclType::Compile(d, _, _), Some(p)) => {
                            let d = sync!(d);
                            let solved = acc.solver.solve_node_with(d, p);
                            let i = Declarator::Type(solved);
                            content.push((i.clone(), solved));
                            acc.solver.add_decl_simple(i.clone(), solved);
                            let i = Declarator::Type(d);
                            content.push((i.clone(), solved));
                            acc.solver.add_decl_simple(i.clone(), solved);
                            d
                        }
                        (DeclType::Compile(d, _, _), None) => {
                            let d = sync!(d);
                            let i = Declarator::Type(d);
                            content.push((i.clone(), d));
                            acc.solver.add_decl_simple(i.clone(), d);
                            d
                        }
                        _ => panic!(),
                    };
                    println!("{}",members.len());
                    for (d, t) in members {
                        let d = d.with_changed_node(|i| sync!(*i));
                        let t = sync!(t); // TODO try solving t
                        println!("d:{:?} t:{:?}",&d,&t);

                        match &d {
                            Declarator::Executable(d) => {
                                // TODO constructor solve
                                if let Some(p) = p {
                                    let solved = acc.solver.solve_node_with(*d, p);
                                    let d = Declarator::Executable(*d);
                                    acc.solver.add_decl_simple(d, solved);
                                    let d = Declarator::Executable(t);
                                    acc.solver.add_decl_simple(d.clone(), t);
                                    content.push((d, t));
                                } else {
                                    let d = Declarator::Executable(*d);
                                    acc.solver.add_decl_simple(d, t);
                                }
                            }
                            Declarator::Field(d) => {
                                if let Some(p) = p {
                                    let solved = acc.solver.solve_node_with(*d, p);
                                    let d = Declarator::Field(solved);
                                    acc.solver.add_decl_simple(d.clone(), t);
                                    content.push((d, t));
                                } else {
                                    let d = Declarator::Field(*d);
                                    acc.solver.add_decl_simple(d, t);
                                }
                            }
                            Declarator::Type(d) => {
                                if let Some(p) = p {
                                    let solved = acc.solver.solve_node_with(*d, p);
                                    let d = Declarator::Type(*d);
                                    acc.solver.add_decl_simple(d, solved);
                                    let d = Declarator::Type(solved);
                                    acc.solver.add_decl_simple(d.clone(), solved);
                                    content.push((d, t));
                                } else {
                                    let d = Declarator::Type(*d);
                                    acc.solver.add_decl_simple(d, t);
                                }
                            }
                            x => panic!("{:?}", x),
                        }
                    }
                    let top_level = if let Visibility::Public = visibility {
                        assert!(top_level.is_none());
                        let d = Declarator::Type(identifier);
                        Some((d, identifier))
                    } else if let Some(_) = top_level {
                        top_level
                    } else {
                        None
                    };
                    State::File {
                        package: p,
                        asterisk_imports,
                        top_level,
                        content,
                    }
                }
                (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            }
        } else if kind == &Type::PackageDeclaration {
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }
            match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                (State::None, State::ScopedIdentifier(i)) => {
                    // TODO complete refs
                    let i = sync!(i);
                    // if jl == i {
                    //     acc.solver.decls = Default::default();
                    // }
                    State::PackageDeclaration(i)
                }
                (State::None, State::SimpleIdentifier(i)) => {
                    // TODO complete refs
                    let o = acc.solver.intern(RefsEnum::Root);
                    let i = acc.solver.intern(RefsEnum::ScopedIdentifier(o, i));
                    State::PackageDeclaration(i)
                }
                (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            }
        } else if kind == &Type::ImportDeclaration {
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }
            match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                (State::None, State::Modifiers(v, n)) => State::Modifiers(v, n),
                (State::Modifiers(Visibility::None, n), State::ScopedIdentifier(i)) => {
                    // println!("{:?}",n);
                    assert_eq!(n, enum_set!(NonVisibility::Static));
                    let i = sync!(i);

                    if i >= acc.solver.refs.len() {
                        acc.solver.refs.resize(i + 1, false);
                    }
                    acc.solver.refs.set(i, true);
                    State::ImportDeclaration {
                        identifier: i,
                        sstatic: true,
                        asterisk: false,
                    } // TODO use static
                }
                (State::None, State::ScopedIdentifier(i)) => {
                    let i = sync!(i);
                    if i >= acc.solver.refs.len() {
                        acc.solver.refs.resize(i + 1, false);
                    }
                    acc.solver.refs.set(i, true);
                    // println!(
                    //     "@@{:?}",
                    //     ExplorableRef {
                    //         rf: i,
                    //         nodes: &self.solver.nodes
                    //     }
                    // );
                    State::ImportDeclaration {
                        identifier: i,
                        sstatic: false,
                        asterisk: false,
                    }
                }
                (
                    State::ImportDeclaration {
                        identifier: i,
                        sstatic,
                        asterisk: false,
                    },
                    State::Asterisk,
                ) => {
                    // TODO say we import members/classes
                    acc.solver.refs.set(i, false);
                    State::ImportDeclaration {
                        identifier: i,
                        sstatic,
                        asterisk: true,
                    }
                }
                (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            }
        } else if kind.is_type_declaration() {
            match current_node.map(|x| Old(x), |x| x) {
                State::Modifiers(v, n) => {
                    let mut remapper = acc.solver.extend(&self.solver);
                    if let State::TypeDeclaration { visibility, .. } = &mut acc.current_node {
                        *visibility = v;
                        acc.current_node.take()
                    } else if State::None == acc.current_node {
                        assert_eq!(kind, &Type::EnumConstant);
                        State::TypeDeclaration {
                            visibility: Visibility::None,
                            identifier: DeclType::Compile(0, None, vec![].into_boxed_slice()),
                            members: vec![],
                        }
                    } else {
                        panic!("{:?} {:?}", kind, acc.current_node)
                    }
                }
                State::SimpleIdentifier(i) => {
                    if let State::TypeDeclaration { identifier, .. } = &mut acc.current_node {
                        if let DeclType::Compile(ii, _, _) = identifier {
                            let r = mm!();
                            let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                            *ii = i;
                        } else {
                            panic!("{:?}", acc.current_node)
                        }
                        acc.current_node.take()
                    } else {
                        assert_eq!(kind, &Type::EnumConstant);
                        let r = mm!();
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                        State::TypeDeclaration {
                            visibility: Visibility::None,
                            identifier: DeclType::Compile(i, None, vec![].into_boxed_slice()),
                            members: vec![],
                        }
                    }
                }
                State::Arguments(_) => {
                    assert_eq!(kind, &Type::EnumConstant);
                    // TODO materialize the construtor call
                    acc.current_node.take()
                }
                State::TypeParameters(ps) => {
                    assert!(kind == &Type::ClassDeclaration || kind == &Type::InterfaceDeclaration);
                    let mut remapper = acc.solver.extend(&self.solver);
                    macro_rules! sync {
                        ( $i:expr ) => {
                            remapper.intern_external(&mut acc.solver, $i.0)
                        };
                    }
                    // println!("typeParams {:?}", ps);
                    for (d, t) in ps {
                        let r = mm!();
                        let d = acc.solver.intern(RefsEnum::ScopedIdentifier(r, d));
                        let d = Declarator::Type(d);
                        let t = t.into_iter().map(|t| sync!(*t)).collect();
                        acc.solver.add_decl(d.clone(), DeclType::Runtime(t));
                    }
                    // println!("decls after added typeParams");
                    // acc.solver.print_decls();
                    // TODO use generics when creating ref from decl ie. searching for impacts
                    acc.current_node.take()
                }
                State::ScopedTypeIdentifier(s) => {
                    assert_eq!(kind, &Type::ClassDeclaration);
                    let mut remapper = acc.solver.extend(&self.solver);
                    macro_rules! sync {
                        ( $i:expr ) => {
                            remapper.intern_external(&mut acc.solver, $i.0)
                        };
                    }
                    if let State::TypeDeclaration { identifier: i, .. } = &mut acc.current_node {
                        let s = sync!(s);
                        match i {
                            DeclType::Compile(_, ss, _) => {
                                // ?.super#constructor(...) -> ?.S
                                let r = mm!();
                                let d = acc.solver.intern(RefsEnum::Super(r));
                                let d = acc
                                    .solver
                                    .intern(RefsEnum::ConstructorInvocation(d, Arguments::Unknown));
                                let d = Declarator::Executable(d);
                                acc.solver.add_decl_simple(d, s);
                                // TODO this one? ?.S.super#constructor(...) -> ?.S

                                *ss = Some(s)
                            }
                            x => panic!("{:?}", x),
                        };
                        // TODO use superclass value more
                    } else {
                        panic!()
                    }
                    acc.current_node.take()
                }
                State::Interfaces(i) => {
                    assert!(
                        kind == &Type::ClassDeclaration
                            || kind == &Type::InterfaceDeclaration
                            || kind == &Type::EnumDeclaration,
                        "{:?}",
                        kind
                    );
                    let mut remapper = acc.solver.extend(&self.solver);
                    macro_rules! sync {
                        ( $i:expr ) => {
                            remapper.intern_external(&mut acc.solver, $i.0)
                        };
                    }
                    if let State::TypeDeclaration { identifier, .. } = &mut acc.current_node {
                        let i = i.into_iter().map(|x| sync!(x)).collect();
                        match identifier {
                            DeclType::Compile(_, _, ii) => *ii = i,
                            x => panic!("{:?}", x),
                        };
                        // TODO use superclass value more
                    } else {
                        panic!()
                    }
                    acc.current_node.take()
                }
                State::None => {
                    // TODO there might be things to do but need tests
                    let mut remapper = acc.solver.extend(&self.solver);
                    // let (cache, solver) = acc.solver.hierarchy_solve_extend(&self.solver);
                    // acc.solver = solver;
                    // macro_rules! sync {
                    //     ( $i:expr ) => {{
                    //         let other = $i.0;
                    //         let other = ExplorableRef {
                    //             rf: other,
                    //             nodes: &acc.solver.nodes,
                    //         };
                    //         acc.solver.intern_external(&mut cache, other)
                    //     }};
                    // }
                    acc.current_node.take()
                }
                State::Declarations(ds) => {
                    if let State::TypeDeclaration {
                        identifier,
                        members,
                        ..
                    } = &mut acc.current_node
                    {
                        let id = match &identifier {
                            DeclType::Compile(i, _, _) => *i,
                            _ => panic!(),
                        };
                        // prime cache
                        let mut extend_cache = HashMap::<usize, usize>::default();
                        if let Some(mm) = self.solver.get(RefsEnum::MaybeMissing) {
                            assert!(
                                !(self.solver.refs.len() > mm && self.solver.refs[mm]),
                                "not sure what to do there"
                            );
                            // then ? -> ?.{A.B,C} and ?.this -> ?.A
                            let r = mm!();
                            let t = {
                                let d = acc.solver.intern(RefsEnum::This(r));
                                let d = Declarator::Type(d);

                                acc.solver.add_decl(d, identifier.clone()); //TODO also put it in cache

                                match &identifier {
                                    DeclType::Compile(i, s, is) => {
                                        let mut t = vec![*i];
                                        t.extend(s);
                                        t.extend(is.iter());
                                        t.into_boxed_slice()
                                    }
                                    _ => panic!(),
                                }
                            };
                            println!("class decl cache {:?}", &t);
                            // if let DeclType::Compile(_, Some(s), _) = &identifier {
                            //     let d = Declarator::Type(*s);
                            //     acc.solver.add_decl_simple(d, *s);
                            // }

                            for id in t.iter() {
                                let i = match &acc.solver.nodes[*id] {
                                    RefsEnum::ScopedIdentifier(_, i) => i.clone(),
                                    _ => panic!(),
                                };
                                // ?.X -> ?.X to protect from masking
                                if let Some(x) = self.solver.get(RefsEnum::ScopedIdentifier(mm, i))
                                {
                                    extend_cache.insert(x, *id);
                                }
                            }

                            // temporary
                            if let Some(i) = self.solver.get(RefsEnum::Super(mm)) {
                                extend_cache.insert(i, id);
                            }

                            let mask = acc.solver.intern(RefsEnum::Mask(r, t));
                            extend_cache.insert(mm, mask);
                        }
                        // // then stash refs from decl
                        // let hierarchical_decls_refs: Vec<_> = acc.solver.refs.iter_ones().collect();
                        // acc.solver.refs = Default::default(); // TODO not sure;

                        // then extend refs from body with a primed cache
                        let mut remapper = acc.solver.extend_map(&self.solver, &mut extend_cache);
                        macro_rules! sync {
                            ( $i:expr ) => {
                                remapper.intern_external(&mut acc.solver, $i.0)
                            };
                        }
                        // then handle members considering prev thing ie. either ?.this -> ?.A or ? -> ?.{B,C}
                        // then resolve
                        // then pop ref stash extend new solver with them
                        // {
                        //     // ?.this -> ?.A
                        // }
                        // // TODO an extend that replace ? -> ?.{B,C}
                        // // idem for the following types
                        // // then only call resolve with:
                        // { // for A extends B implements C
                        //      // ?.B -> ?.B
                        //      // ?.C -> ?.C
                        //      // ?.super -> ?.B
                        //      // ?.B.super -> ?.B
                        //      // ?.C.super -> ?.C
                        // }
                        {
                            // ?.super -> ?.super
                            let r = mm!();
                            let i = acc.solver.intern(RefsEnum::Super(r));
                            let d = Declarator::Type(i);
                            acc.solver.add_decl(d, identifier.clone());
                        }
                        {
                            // ?.A -> ?.A
                            let d = Declarator::Type(id);
                            acc.solver.add_decl(d, identifier.clone());
                        }
                        {
                            // ?.A.this -> ?.A
                            //     let d = acc.solver.intern(RefsEnum::This(id));
                            //     let d = Declarator::Type(d);
                            //     acc.solver.add_decl(d, identifier.clone());
                            //     // TODO this one? ?.S.super -> ?.S
                        }
                        {
                            // ?.A#() -> ?.A
                            // let d = acc.solver.intern(RefsEnum::ConstructorInvocation(
                            //     id,
                            //     Arguments::Given(vec![].into_boxed_slice()),
                            // ));
                            // let d = Declarator::Executable(d);
                            // acc.solver.add_decl(d, identifier.clone());
                        }
                        {
                            // ?.this#(...) -> ?.A
                            // let d = mm!();
                            // let d = acc.solver.intern(RefsEnum::This(d));
                            // let d = acc
                            //     .solver
                            //     .intern(RefsEnum::ConstructorInvocation(d, Arguments::Unknown));
                            // let d = Declarator::Executable(d);
                            // acc.solver.add_decl(d, identifier.clone());
                        }
                        {
                            // ?.this -> ?.A
                            let d = mm!();
                            let d = acc.solver.intern(RefsEnum::This(d));
                            let d = Declarator::Type(d);
                            acc.solver.add_decl(d, identifier.clone());
                        }
                        // let (mut cache, solver) = acc.solver.hierarchy_solve_extend(&self.solver);
                        // acc.solver = solver;
                        // macro_rules! sync {
                        //     ( $i:expr ) => {{
                        //         let other = $i.0;
                        //         let other = ExplorableRef {
                        //             rf: other,
                        //             nodes: &self.solver.nodes,
                        //         };
                        //         acc.solver
                        //             .hierarchy_solve_intern_external(&mut cache, other)
                        //             .unwrap()
                        //     }};
                        // }
                        println!("adding members");
                        for (d, t) in ds {
                            let d = d.with_changed_node(|i| sync!(*i));
                            let t = sync!(t);
                            println!("d:{:?} t:{:?}",&d,&t);
                            match &d {
                                Declarator::Executable(d) => {
                                    match acc.solver.nodes[*d].clone() {
                                        RefsEnum::ConstructorInvocation(o, p) => {
                                            // constructor solve
                                            {
                                                // TODO test if it does ?.A#(p) => ?.A
                                                let d = Declarator::Executable(*d);
                                                acc.solver.add_decl(d, identifier.clone());
                                            }
                                            {
                                                // TODO not sure how to change o
                                                // given class A, it might be better to solve ?.this#(p) here to ?.A.this#(p) and in general ?.A.this -> ?A.
                                                let solved = acc
                                                    .solver
                                                    .intern(RefsEnum::ConstructorInvocation(id, p));
                                                // acc.solver.solve_node_with(*d, i); // to spec ?.this#(p)
                                                let d = Declarator::Executable(solved);
                                                acc.solver.add_decl(d.clone(), identifier.clone());
                                                members.push((d, id));
                                            }
                                        }
                                        RefsEnum::Invocation(o, i, p) => {
                                            {
                                                let d = Declarator::Executable(*d);
                                                acc.solver.add_decl_simple(d, t);
                                            }
                                            {
                                                let solved = acc.solver.solve_node_with(*d, id);
                                                let d = Declarator::Executable(solved);
                                                acc.solver.add_decl_simple(d.clone(), t);
                                                members.push((d, t));
                                            }
                                            {
                                                let r = mm!();
                                                let r = acc.solver.intern(RefsEnum::This(r));
                                                let solved = acc.solver.solve_node_with(*d, id);
                                                let d = Declarator::Executable(solved);
                                                acc.solver.add_decl_simple(d.clone(), t);
                                                members.push((d, t));
                                            }
                                        }
                                        x => todo!("{:?}", x),
                                    }
                                }
                                Declarator::Field(d) => {
                                    {
                                        // ?.d => t
                                        let d = Declarator::Field(*d);
                                        acc.solver.add_decl_simple(d, t);
                                    }
                                    {
                                        // ?.id.d => t
                                        let solved = acc.solver.solve_node_with(*d, id);
                                        let d = Declarator::Field(solved);
                                        acc.solver.add_decl_simple(d.clone(), t);
                                        members.push((d, t));
                                    }
                                    {
                                        // ?.this.d => t
                                        let r = mm!();
                                        let r = acc.solver.intern(RefsEnum::This(r));
                                        let solved = acc.solver.solve_node_with(*d, r);
                                        let d = Declarator::Field(solved);
                                        acc.solver.add_decl_simple(d.clone(), t);
                                        members.push((d, t));
                                    }
                                }
                                Declarator::Type(d) => {
                                    {
                                        let d = Declarator::Type(*d);
                                        acc.solver.add_decl_simple(d, t);
                                    }
                                    {
                                        let solved = acc.solver.solve_node_with(*d, id);
                                        let d = Declarator::Type(solved);
                                        acc.solver.add_decl_simple(d.clone(), t);
                                        members.push((d, t));
                                    }
                                    {
                                        let r = mm!();
                                        let r = acc.solver.intern(RefsEnum::This(r));
                                        let solved = acc.solver.solve_node_with(*d, r);
                                        let d = Declarator::Type(solved);
                                        acc.solver.add_decl_simple(d.clone(), t);
                                        members.push((d, t));
                                    }
                                }
                                x => panic!("{:?}", x),
                            }
                        }
                        println!("members added");
                        // let (mut cache, solver) = acc.solver.resolve();
                        // acc.solver = solver;
                        // println!("class declaration solved");
                    } else {
                        panic!()
                    }
                    acc.current_node.take()
                }
                y => todo!("{:?} {:?} {:?}", kind, &acc.current_node, y),
            }
        } else if kind.is_type_body() {
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }
            match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                (
                    rest,
                    State::TypeDeclaration {
                        visibility,
                        identifier: d,
                        members,
                    },
                ) if kind == &Type::ClassBody
                    || kind == &Type::InterfaceBody
                    || kind == &Type::EnumBody
                    || kind == &Type::EnumBodyDeclarations =>
                {
                    // TODO also solve members ?
                    // TODO visibility
                    let mut v = match rest {
                        State::Declarations(u) => u,
                        State::None => vec![],
                        _ => panic!(),
                    };
                    match d {
                        DeclType::Runtime(_) => panic!(),
                        DeclType::Compile(t, _, _) => {
                            let t = sync!(t);
                            let d = Declarator::Type(t);
                            acc.solver.add_decl_simple(d.clone(), t);
                            v.push((d, t));
                        }
                    };
                    for (d,t) in members {
                        match d {
                            Declarator::None => {panic!()},
                            Declarator::Package(_) => {panic!()},
                            Declarator::Type(_) => {},
                            Declarator::Field(d) => {
                                let d = sync!(d);
                                let d = Declarator::Field(d);
                                let t = sync!(t);
                                acc.solver.add_decl_simple(d.clone(), t);
                                v.push((d, t));
                            },
                            Declarator::Executable(_) => {},
                            Declarator::Variable(_) => {panic!()},
                        };
                    }
                    State::Declarations(v)
                }
                (rest, State::None) if kind == &Type::ClassBody => {
                    match &rest {
                        State::Declarations(_) => (),
                        State::None => (),
                        _ => panic!(),
                    }
                    rest
                }
                (
                    rest,
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: d,
                    },
                ) if kind == &Type::ClassBody
                    || kind == &Type::InterfaceBody
                    || kind == &Type::AnnotationTypeBody
                    || kind == &Type::EnumBodyDeclarations =>
                {
                    let t = sync!(t);
                    let d = d.with_changed_node(|i| sync!(*i));
                    match &d {
                        Declarator::Type(_) => (),
                        Declarator::Field(_) => (),
                        Declarator::Executable(_) => (),
                        _ => panic!(),
                    };
                    acc.solver.add_decl_simple(d.clone(), t);
                    let mut v = match rest {
                        State::Declarations(u) => u,
                        State::None => vec![],
                        _ => panic!(),
                    };
                    v.push((d, t));
                    // TODO make a member declaration and make use of visibilty
                    State::Declarations(v)
                }
                (rest, State::Declarations(u)) if kind == &Type::EnumBody => {
                    let mut v = match rest {
                        State::Declarations(u) => u,
                        State::None => vec![],
                        _ => panic!(),
                    };
                    for (d, t) in u {
                        let t = sync!(t);
                        let d = d.with_changed_node(|i| sync!(*i));
                        v.push((d, t));
                    }
                    State::Declarations(v)
                }
                (rest, State::None) if kind == &Type::EnumBodyDeclarations => {
                    let mut v = match rest {
                        State::Declarations(u) => u,
                        State::None => vec![],
                        _ => panic!(),
                    };
                    State::Declarations(v)
                }
                (
                    rest,
                    State::MethodImplementation {
                        visibility,
                        kind: t,
                        identifier: d,
                        parameters: p,
                    },
                ) if kind == &Type::ClassBody
                    || kind == &Type::InterfaceBody
                    || kind == &Type::EnumBodyDeclarations =>
                {
                    let t = sync!(t.unwrap());
                    let r = mm!();
                    let p = p.into_iter().map(|(_, t)| sync!(*t)).collect();
                    let d = acc.solver.intern(RefsEnum::Invocation(
                        r,
                        d.unwrap(),
                        Arguments::Given(p),
                    ));
                    let d = Declarator::Executable(d);
                    acc.solver.add_decl_simple(d.clone(), t);
                    let mut v = match rest {
                        State::Declarations(u) => u,
                        State::None => vec![],
                        _ => panic!(),
                    };
                    v.push((d, t));
                    // TODO make a member declaration and make use of visibilty
                    State::Declarations(v)
                }
                (
                    rest,
                    State::ConstructorImplementation {
                        visibility,
                        identifier: i,
                        parameters: p,
                    },
                ) if kind == &Type::ClassBody || kind == &Type::EnumBodyDeclarations => {
                    let mut v = match rest {
                        State::Declarations(u) => u,
                        State::None => vec![],
                        _ => panic!(),
                    };
                    let p = p.into_iter().map(|(_, t)| sync!(*t)).collect();
                    let t = i.unwrap();
                    let r = mm!();
                    let t = acc.solver.intern(RefsEnum::ScopedIdentifier(r, t));
                    let i = acc.solver.intern(RefsEnum::MaybeMissing);
                    let i = acc.solver.intern(RefsEnum::This(i));
                    let i = acc
                        .solver
                        .intern(RefsEnum::ConstructorInvocation(i, Arguments::Given(p)));
                    let d = Declarator::Executable(i);
                    // TODO constructor solve
                    acc.solver.add_decl_simple(d.clone(), t);
                    v.push((d, t));

                    // TODO make a member declaration and make use of visibilty
                    State::Declarations(v)
                }
                (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            }
        } else if kind.is_value_member() {
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }
            // if kind == &Type::FieldDeclaration {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else if kind == &Type::ConstantDeclaration {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else if kind == &Type::EnumConstant {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else if kind == &Type::AnnotationTypeElementDeclaration {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else {
            //     panic!("{:?}",kind)
            // }
            match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                (State::SimpleTypeIdentifier(t), State::Declarator(Declarator::Variable(i)))
                    if kind == &Type::FieldDeclaration =>
                {
                    let t = scoped!(mm!(), t);
                    let Old(i) = i;
                    match self.solver.nodes[i] {
                        RefsEnum::Array(i) => {
                            let i = remapper.intern_external(&mut acc.solver, i);
                            let i = Declarator::Field(i);
                            let t = acc.solver.intern(RefsEnum::Array(t));
                            State::Declaration {
                                visibility: Visibility::None,
                                kind: t,
                                identifier: i,
                            }
                        }
                        _ => {
                            let i = remapper.intern_external(&mut acc.solver, i);
                            let i = Declarator::Field(i);
                            State::Declaration {
                                visibility: Visibility::None,
                                kind: t,
                                identifier: i,
                            }
                        }
                    }
                }
                (State::Modifiers(v, n), State::SimpleTypeIdentifier(t))
                    if kind == &Type::FieldDeclaration || kind == &Type::ConstantDeclaration =>
                {
                    // TODO simple type identifier should be a type identifier ie. already scoped
                    let t = scoped!(mm!(), t);
                    let i = Declarator::None;
                    // not used directly
                    // acc.solver.add_decl_simple(i.clone(), t);
                    State::Declaration {
                        visibility: v,
                        kind: t,
                        identifier: i,
                    }
                }
                (State::Modifiers(v, n), State::ScopedTypeIdentifier(t))
                    if kind == &Type::FieldDeclaration || kind == &Type::ConstantDeclaration =>
                {
                    // TODO simple type identifier should be a type identifier ie. already scoped
                    let t = sync!(t);
                    let i = Declarator::None;
                    // not used directly
                    // acc.solver.add_decl_simple(i.clone(), t);
                    State::Declaration {
                        visibility: v,
                        kind: t,
                        identifier: i,
                    }
                }
                (State::None, State::SimpleTypeIdentifier(t))
                    if kind == &Type::FieldDeclaration || kind == &Type::ConstantDeclaration =>
                {
                    // TODO simple type identifier should be a type identifier ie. already scoped
                    let t = scoped!(mm!(), t);
                    let i = Declarator::None;
                    // not used directly
                    // acc.solver.add_decl_simple(i.clone(), t);
                    State::Declaration {
                        visibility: Visibility::None,
                        kind: t,
                        identifier: i,
                    }
                }
                (State::None, State::ScopedTypeIdentifier(t))
                    if kind == &Type::FieldDeclaration || kind == &Type::ConstantDeclaration =>
                {
                    // TODO simple type identifier should be a type identifier ie. already scoped
                    let t = sync!(t);
                    let i = Declarator::None;
                    // not used directly
                    // acc.solver.add_decl_simple(i.clone(), t);
                    State::Declaration {
                        visibility: Visibility::None,
                        kind: t,
                        identifier: i,
                    }
                }
                (
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: _,
                    },
                    State::Declarator(Declarator::Variable(i)),
                ) if kind == &Type::FieldDeclaration || kind == &Type::ConstantDeclaration => {
                    let Old(i) = i;
                    match self.solver.nodes[i] {
                        RefsEnum::Array(i) => {
                            let i = remapper.intern_external(&mut acc.solver, i);
                            let i = Declarator::Field(i);
                            let t = acc.solver.intern(RefsEnum::Array(t));
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: i,
                            }
                        }
                        _ => {
                            let i = remapper.intern_external(&mut acc.solver, i);
                            let i = Declarator::Field(i);
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: i,
                            }
                        }
                    }
                }
                (State::Modifiers(v, n), State::ScopedTypeIdentifier(t))
                    if kind == &Type::FieldDeclaration =>
                {
                    // TODO simple type identifier should be a type identifier ie. already scoped
                    let t = sync!(t);
                    let i = Declarator::None;
                    // not used directly
                    // acc.solver.add_decl_simple(i.clone(), t);
                    State::Declaration {
                        visibility: v,
                        kind: t,
                        identifier: i,
                    }
                }
                (State::None, State::Modifiers(v, n))
                    if kind == &Type::FieldDeclaration
                        || kind == &Type::ConstantDeclaration
                        || kind == &Type::AnnotationTypeElementDeclaration =>
                {
                    State::Modifiers(v, n)
                }
                (State::None, State::SimpleTypeIdentifier(t))
                    if kind == &Type::FieldDeclaration
                        || kind == &Type::AnnotationTypeElementDeclaration =>
                {
                    let t = scoped!(mm!(), t);
                    State::Declaration {
                        visibility: Visibility::None,
                        kind: t,
                        identifier: Declarator::None,
                    }
                }
                (State::None, State::ScopedTypeIdentifier(t))
                    if kind == &Type::FieldDeclaration
                        || kind == &Type::AnnotationTypeElementDeclaration =>
                {
                    let t = sync!(t);
                    State::Declaration {
                        visibility: Visibility::None,
                        kind: t,
                        identifier: Declarator::None,
                    }
                }
                (State::Modifiers(v, n), State::SimpleTypeIdentifier(t))
                    if kind == &Type::AnnotationTypeElementDeclaration =>
                {
                    // TODO simple type identifier should be a type identifier ie. already scoped
                    let t = scoped!(mm!(), t);
                    let i = Declarator::None;
                    // not used directly
                    // acc.solver.add_decl_simple(i.clone(), t);
                    State::Declaration {
                        visibility: v,
                        kind: t,
                        identifier: i,
                    }
                }
                (
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: _,
                    },
                    State::SimpleIdentifier(i),
                ) if kind == &Type::AnnotationTypeElementDeclaration => {
                    // TODO simple type identifier should be a type identifier ie. already scoped
                    let r = mm!();
                    let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                    let i = Declarator::Field(i);
                    // not used directly
                    // acc.solver.add_decl_simple(i.clone(), t);
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: i,
                    }
                }
                (
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: i,
                    },
                    State::LiteralType(_),
                ) if kind == &Type::AnnotationTypeElementDeclaration && i != Declarator::None => {
                    // TODO do something with default value
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: i,
                    }
                }
                (
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: i,
                    },
                    State::None,
                ) if kind == &Type::AnnotationTypeElementDeclaration && i != Declarator::None => {
                    // TODO do something with default value
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: i,
                    }
                }
                (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            }
        } else if kind.is_executable_member() {
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }
            // if kind == &Type::MethodDeclaration {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else if kind == &Type::ConstructorDeclaration {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else {
            //     panic!("{:?}",kind)
            // }
            match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                (State::None, State::SimpleTypeIdentifier(t))
                    if kind == &Type::MethodDeclaration =>
                {
                    let t = scoped!(mm!(), t);
                    State::ScopedTypeIdentifier(t)
                }
                (
                    State::MethodImplementation {
                        visibility,
                        kind: _,
                        identifier: _,
                        parameters: _,
                    },
                    State::SimpleTypeIdentifier(t),
                ) if kind == &Type::MethodDeclaration => {
                    let t = scoped!(mm!(), t);
                    State::MethodImplementation {
                        visibility,
                        kind: Some(t),
                        identifier: None,
                        parameters: vec![].into_boxed_slice(),
                    }
                }
                (
                    State::MethodImplementation {
                        visibility,
                        kind: _,
                        identifier: _,
                        parameters: _,
                    },
                    State::ScopedTypeIdentifier(t),
                ) if kind == &Type::MethodDeclaration => {
                    let t = sync!(t);
                    State::MethodImplementation {
                        visibility,
                        kind: Some(t),
                        identifier: None,
                        parameters: vec![].into_boxed_slice(),
                    }
                }
                (State::None, State::ScopedTypeIdentifier(t))
                    if kind == &Type::MethodDeclaration =>
                {
                    let t = sync!(t);
                    State::ScopedTypeIdentifier(t)
                }
                (State::ScopedTypeIdentifier(t), State::SimpleIdentifier(i))
                    if kind == &Type::MethodDeclaration =>
                {
                    State::MethodImplementation {
                        visibility: Visibility::None,
                        kind: Some(t),
                        identifier: Some(i),
                        parameters: vec![].into_boxed_slice(),
                    }
                }
                (State::None, State::SimpleIdentifier(i))
                    if kind == &Type::ConstructorDeclaration =>
                {
                    State::ConstructorImplementation {
                        visibility: Visibility::None,
                        identifier: Some(i),
                        parameters: vec![].into_boxed_slice(),
                    }
                }
                (State::None, State::Modifiers(v, n))
                    if kind == &Type::MethodDeclaration
                        || kind == &Type::ConstructorDeclaration =>
                {
                    State::Modifiers(v, n)
                }
                (State::Modifiers(v, n), State::SimpleTypeIdentifier(t))
                    if kind == &Type::MethodDeclaration =>
                {
                    let t = scoped!(mm!(), t);
                    State::MethodImplementation {
                        visibility: v,
                        kind: Some(t),
                        identifier: None,
                        parameters: Default::default(),
                    }
                }
                (State::Modifiers(v, n), State::ScopedTypeIdentifier(t))
                    if kind == &Type::MethodDeclaration =>
                {
                    let t = sync!(t);
                    State::MethodImplementation {
                        visibility: v,
                        kind: Some(t),
                        identifier: None,
                        parameters: Default::default(),
                    }
                }
                (
                    State::MethodImplementation {
                        visibility: v,
                        kind: _,
                        identifier: _,
                        parameters: _,
                    },
                    State::SimpleTypeIdentifier(t),
                ) if kind == &Type::MethodDeclaration => {
                    let t = scoped!(mm!(), t);
                    State::MethodImplementation {
                        visibility: v,
                        kind: Some(t),
                        identifier: None,
                        parameters: Default::default(),
                    }
                }
                (State::None, State::TypeParameters(t)) if kind == &Type::MethodDeclaration => {
                    for (t, b) in t {
                        let r = mm!();
                        let t = acc.solver.intern(RefsEnum::ScopedIdentifier(r, t));
                        let b = b.into_iter().map(|t| sync!(*t)).collect();
                        acc.solver
                            .add_decl(Declarator::Type(t), DeclType::Runtime(b))
                    }

                    State::MethodImplementation {
                        visibility: Visibility::None,
                        kind: None,
                        identifier: None,
                        parameters: Default::default(),
                    }
                }
                (State::Modifiers(v, n), State::TypeParameters(t))
                    if kind == &Type::MethodDeclaration =>
                {
                    for (t, b) in t {
                        let r = mm!();
                        let t = acc.solver.intern(RefsEnum::ScopedIdentifier(r, t));
                        let b = b.into_iter().map(|t| sync!(*t)).collect();
                        acc.solver
                            .add_decl(Declarator::Type(t), DeclType::Runtime(b))
                    }

                    State::MethodImplementation {
                        visibility: v,
                        kind: None,
                        identifier: None,
                        parameters: Default::default(),
                    }
                }
                (State::Modifiers(v, n), State::SimpleIdentifier(i))
                    if kind == &Type::ConstructorDeclaration =>
                {
                    // TODO use visibility
                    State::ConstructorImplementation {
                        visibility: v,
                        identifier: Some(i),
                        parameters: Default::default(),
                    }
                }
                (
                    State::MethodImplementation {
                        visibility,
                        kind: t,
                        identifier: i,
                        parameters: _,
                    },
                    State::FormalParameters(p),
                ) if kind == &Type::MethodDeclaration => {
                    let p = p
                        .into_iter()
                        .map(|(i, t)| {
                            let i = sync!(i);
                            let t = sync!(t);
                            acc.solver.add_decl_simple(Declarator::Variable(i), t); // TODO use variable or parameter ?
                            (i, t)
                        })
                        .collect();
                    // let r = mm!();
                    // let i = acc
                    //     .solver
                    //     .intern(RefsEnum::Invocation(r, i, Arguments::Given(p)));
                    State::MethodImplementation {
                        visibility,
                        kind: t,
                        identifier: i,
                        parameters: p,
                    }
                }
                (
                    State::MethodImplementation {
                        visibility,
                        kind: t,
                        identifier: i0,
                        parameters: p,
                    },
                    State::SimpleIdentifier(i),
                ) if kind == &Type::MethodDeclaration => {
                    assert_eq!(i0, None);
                    State::MethodImplementation {
                        visibility,
                        kind: t,
                        identifier: Some(i),
                        parameters: p,
                    }
                }
                (
                    State::MethodImplementation {
                        visibility,
                        kind: t,
                        identifier,
                        parameters: p,
                    },
                    State::Dimensions,
                ) if kind == &Type::MethodDeclaration => {
                    let t = t.map(|t| acc.solver.intern(RefsEnum::Array(t)));
                    State::MethodImplementation {
                        visibility,
                        kind: t,
                        identifier,
                        parameters: p,
                    }
                }
                (
                    State::MethodImplementation {
                        visibility,
                        kind: t,
                        identifier: i,
                        parameters: p,
                    },
                    State::Throws,
                ) if kind == &Type::MethodDeclaration => State::MethodImplementation {
                    visibility,
                    kind: t,
                    identifier: i,
                    parameters: p,
                },
                (
                    State::ConstructorImplementation {
                        visibility,
                        identifier: i,
                        parameters: p,
                    },
                    State::Throws,
                ) if kind == &Type::ConstructorDeclaration => State::ConstructorImplementation {
                    visibility,
                    identifier: i,
                    parameters: p,
                },
                (
                    State::ConstructorImplementation {
                        visibility,
                        identifier: i,
                        parameters: _,
                    },
                    State::FormalParameters(p),
                ) if kind == &Type::ConstructorDeclaration => {
                    let p = p
                        .into_iter()
                        .map(|(i, t)| {
                            let i = sync!(i);
                            let t = sync!(t);
                            acc.solver.add_decl_simple(Declarator::Variable(i), t); // TODO use variable or parameter ?
                            (i, t)
                        })
                        .collect();
                    State::ConstructorImplementation {
                        visibility,
                        identifier: i,
                        parameters: p,
                    }
                }
                (
                    State::MethodImplementation {
                        visibility,
                        kind: t,
                        identifier: i,
                        parameters: p,
                    },
                    State::None,
                ) if kind == &Type::MethodDeclaration => {
                    let p: Box<[RefPtr]> = p.into_iter().map(|(i, t)| *t).collect();
                    let r = mm!();
                    let i =
                        acc.solver
                            .intern(RefsEnum::Invocation(r, i.unwrap(), Arguments::Given(p)));
                    State::Declaration {
                        visibility,
                        kind: t.unwrap(),
                        identifier: Declarator::Executable(i),
                    }
                }
                (
                    State::ConstructorImplementation {
                        visibility,
                        identifier,
                        parameters,
                    },
                    State::None,
                ) if kind == &Type::ConstructorDeclaration => {
                    let t = identifier.unwrap();
                    let r = mm!();
                    let t = acc.solver.intern(RefsEnum::ScopedIdentifier(r, t));
                    let p: Box<[RefPtr]> = parameters.into_iter().map(|(_, t)| *t).collect();
                    {
                        let p = p.clone();
                        let i = acc.solver.intern(RefsEnum::MaybeMissing);
                        let i = acc.solver.intern(RefsEnum::This(i));
                        let i = acc
                            .solver
                            .intern(RefsEnum::ConstructorInvocation(i, Arguments::Given(p)));
                        let d = Declarator::Executable(i);
                        acc.solver.add_decl_simple(d, t);
                    }
                    {
                        let i = acc
                            .solver
                            .intern(RefsEnum::ConstructorInvocation(t, Arguments::Given(p)));
                        let d = Declarator::Executable(i);
                        acc.solver.add_decl_simple(d, t);
                    }
                    State::ConstructorImplementation {
                        visibility,
                        identifier,
                        parameters,
                    }
                }
                (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            }
        } else if kind.is_statement() {
            if kind.is_declarative_statement() {
                let mut remapper = acc.solver.local_solve_extend(&self.solver);
                macro_rules! sync {
                    ( $i:expr ) => {
                        remapper.intern_external(&mut acc.solver, $i.0)
                    };
                }
                if kind == &Type::LocalVariableDeclaration {
                    match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                        (State::None, State::Modifiers(v, n)) => State::Modifiers(v, n),
                        (State::None, State::ScopedTypeIdentifier(t)) => {
                            let t = sync!(t);
                            State::ScopedTypeIdentifier(t)
                        }
                        (
                            State::ScopedTypeIdentifier(t),
                            State::Declarator(Declarator::Variable(i)),
                        ) => {
                            let v = vec![(Declarator::Variable(sync!(i)), t)];
                            let Old(i) = i;
                            match self.solver.nodes[i] {
                                RefsEnum::Array(i) => {
                                    let t = acc.solver.intern_ref(RefsEnum::Array(t));
                                }
                                _ => {}
                            };
                            State::Declarations(v)
                        }
                        (
                            State::Declarations(mut v),
                            State::Declarator(Declarator::Variable(i)),
                        ) => {
                            let (_, t) = v[0];
                            v.push((Declarator::Variable(sync!(i)), t));
                            let Old(i) = i;
                            match self.solver.nodes[i] {
                                RefsEnum::Array(_) => {
                                    acc.solver.intern_ref(RefsEnum::Array(t));
                                }
                                _ => {}
                            };
                            State::Declarations(v)
                        }
                        (State::None, State::SimpleTypeIdentifier(t)) => {
                            let t = scoped!(mm!(), t);
                            State::ScopedTypeIdentifier(t)
                        }
                        (State::Modifiers(v, n), State::SimpleTypeIdentifier(t)) => {
                            let t = scoped!(mm!(), t);
                            State::ScopedTypeIdentifier(t)
                        }
                        (State::Modifiers(v, n), State::ScopedTypeIdentifier(t)) => {
                            let t = sync!(t);
                            State::ScopedTypeIdentifier(t)
                        }
                        (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                    }
                // } else if kind == &Type::TryWithResourcesStatement {
                //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
                //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                //     }
                // } else if kind == &Type::CatchClause {
                //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
                //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                //     }
                // } else if kind == &Type::ForStatement {
                //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
                //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                //     }
                // } else if kind == &Type::EnhancedForStatement {
                //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
                //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                //     }
                // } else if kind == &Type::Scope {
                //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
                //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                //     }
                } else {
                    // panic!("{:?}",kind)
                    // }
                    match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                        (State::None, State::None) if kind == &Type::TryWithResourcesStatement => {
                            State::None
                        }
                        (State::None, State::FormalParameters(p))
                            if kind == &Type::TryWithResourcesStatement =>
                        {
                            // TODO it implicitly calls close on resource so need to materialize it
                            p.into_iter().for_each(|(i, t)| {
                                let i = sync!(i);
                                let t = sync!(t);
                                acc.solver.add_decl_simple(Declarator::Variable(i), t);
                                // TODO use variable or parameter ?
                            });
                            State::None
                        }
                        (State::None, State::None) if kind == &Type::CatchClause => State::None,
                        (
                            State::None,
                            State::CatchParameter {
                                kinds: b,
                                identifier: i,
                            },
                        ) if kind == &Type::CatchClause => {
                            let r = mm!();
                            let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                            let d = Declarator::Variable(i);
                            // TODO send whole intersection
                            // let b = b.into_iter().map(|t|
                            //     sync!(*t)
                            // ).collect();
                            let b = sync!(b[0]);
                            acc.solver.add_decl_simple(d.clone(), b);
                            State::None
                        }
                        (State::None, State::SimpleTypeIdentifier(t))
                            if kind == &Type::EnhancedForStatement =>
                        {
                            let t = scoped!(mm!(), t);
                            State::ScopedTypeIdentifier(t)
                        }
                        (State::None, State::ScopedTypeIdentifier(t))
                            if kind == &Type::EnhancedForStatement =>
                        {
                            let t = sync!(t);
                            State::ScopedTypeIdentifier(t)
                        }
                        (State::ScopedTypeIdentifier(t), State::SimpleIdentifier(i))
                            if kind == &Type::EnhancedForStatement =>
                        {
                            let r = mm!();
                            let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                            let i = Declarator::Variable(i);
                            acc.solver.add_decl_simple(i.clone(), t);
                            // TODO also make a special state for variable declarations
                            State::Declaration {
                                visibility: Visibility::None,
                                kind: t,
                                identifier: i,
                            }
                        }
                        (
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            },
                            State::SimpleIdentifier(i),
                        ) if kind == &Type::EnhancedForStatement => {
                            let r = mm!();
                            let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            }
                        }
                        (
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            },
                            State::This(i),
                        ) if kind == &Type::EnhancedForStatement => {
                            let i = sync!(i);
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            }
                        }
                        (
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            },
                            State::FieldIdentifier(i),
                        ) if kind == &Type::EnhancedForStatement => State::Declaration {
                            visibility,
                            kind: t,
                            identifier: d,
                        },
                        (
                            State::Declaration {
                                visibility: _,
                                kind: _,
                                identifier: _,
                            },
                            State::None,
                        ) if kind == &Type::EnhancedForStatement => State::None,
                        (
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            },
                            State::ScopedIdentifier(i),
                        ) if kind == &Type::EnhancedForStatement => {
                            let i = sync!(i); // Not necessary
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            }
                        }
                        (
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            },
                            State::Invocation(i),
                        ) if kind == &Type::EnhancedForStatement => {
                            let i = sync!(i); // Not necessary
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            }
                        }
                        (
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            },
                            State::ConstructorInvocation(i),
                        ) if kind == &Type::EnhancedForStatement => {
                            let i = sync!(i); // Not necessary
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            }
                        }

                        (rest, State::Declarations(v)) if kind == &Type::ForStatement => {
                            for (d, t) in v {
                                let t = sync!(t);
                                let d = d.with_changed_node(|i| sync!(*i));
                                match rest {
                                    State::None => (),
                                    _ => panic!(),
                                }
                                match &d {
                                    Declarator::Variable(_) => acc.solver.add_decl_simple(d, t),
                                    _ => todo!(),
                                };
                            }
                            // we do not need declarations outside of the map to solve local variable
                            // because a local variable declaration is never visible from outside
                            State::None
                        }

                        // (
                        //     State::None,
                        //     State::Declaration {
                        //         visibility,
                        //         kind: t,
                        //         identifier: d,
                        //     },
                        // ) if kind == &Type::ForStatement => {
                        //     let t = sync!(t);
                        //     let d = d.with_changed_node(|i| sync!(*i));
                        //     State::Declaration {a
                        //         visibility,
                        //         kind: t,
                        //         identifier: d,
                        //     }
                        // }
                        (State::None, State::SimpleIdentifier(i))
                            if kind == &Type::ForStatement || kind == &Type::DoStatement =>
                        {
                            scoped!(mm!(), i);
                            State::None
                        }
                        (State::None, _)
                            if kind == &Type::ForStatement || kind == &Type::DoStatement =>
                        {
                            State::None
                        }
                        (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                    }
                }
            } else if kind.is_block_related() {
                let mut remapper = acc.solver.local_solve_extend(&self.solver);
                macro_rules! sync {
                    ( $i:expr ) => {
                        remapper.intern_external(&mut acc.solver, $i.0)
                    };
                }
                // maybe fusion with structural statement
                if kind == &Type::StaticInitializer {
                    match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                        (State::None, State::Modifiers(_, _)) => State::None,
                        (State::None, State::None) => State::None,
                        (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                    }
                } else if kind == &Type::ConstructorBody {
                    match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                        (State::None, State::None) if kind == &Type::ConstructorBody => State::None,
                        (rest, State::ConstructorInvocation(i))
                            if kind == &Type::ConstructorBody =>
                        {
                            let i = sync!(i);
                            match rest {
                                State::None => (),
                                _ => panic!(),
                            }
                            State::None
                        }
                        (
                            rest,
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            },
                        ) if kind == &Type::ConstructorBody => {
                            let t = sync!(t);
                            let d = d.with_changed_node(|i| sync!(*i));
                            match rest {
                                State::None => (),
                                _ => panic!(),
                            }
                            match &d {
                                Declarator::Variable(_) => acc.solver.add_decl_simple(d, t),
                                _ => todo!(),
                            };
                            // we do not need declarations outside of the map to solve local variable
                            // because a local variable declaration is never visible from outside
                            // TODO declarations needed in ConstructorDeclaration
                            State::None
                        }
                        (rest, State::Declarations(v)) => {
                            for (d, t) in v {
                                let t = sync!(t);
                                let d = d.with_changed_node(|i| sync!(*i));
                                match rest {
                                    State::None => (),
                                    _ => panic!(),
                                }
                                match &d {
                                    Declarator::Variable(_) => acc.solver.add_decl_simple(d, t),
                                    _ => todo!(),
                                };
                            }
                            // we do not need declarations outside of the map to solve local variable
                            // because a local variable declaration is never visible from outside
                            State::None
                        }
                        (
                            rest,
                            State::TypeDeclaration {
                                visibility,
                                identifier: d,
                                members: _,
                            },
                        ) => {
                            match d {
                                DeclType::Runtime(_) => panic!(),
                                DeclType::Compile(t, _, _) => {
                                    let t = sync!(t);
                                    let d = Declarator::Type(t);
                                    acc.solver.add_decl_simple(d, t);
                                }
                            };
                            match rest {
                                State::None => (),
                                _ => panic!(),
                            }
                            // we do not need declarations outside of the map to solve local variable
                            // because a local variable declaration is never visible from outside
                            State::None
                        }
                        (State::None, State::None) => State::None,
                        (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                    }
                } else if kind == &Type::Block {
                    match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                        (rest, State::Declarations(v)) => {
                            for (d, t) in v {
                                let t = sync!(t);
                                let d = d.with_changed_node(|i| sync!(*i));
                                match rest {
                                    State::None => (),
                                    _ => panic!(),
                                }
                                match &d {
                                    Declarator::Variable(i) => match acc.solver.nodes[*i] {
                                        RefsEnum::Array(i) => {
                                            let i = Declarator::Variable(i);
                                            let t = acc.solver.intern(RefsEnum::Array(t));
                                            acc.solver.add_decl_simple(i, t)
                                        }
                                        _ => {
                                            let i = Declarator::Variable(*i);
                                            acc.solver.add_decl_simple(i, t)
                                        }
                                    },
                                    _ => todo!(),
                                };
                            }
                            // we do not need declarations apart of the map to solve local variable
                            // because a local variable declaration is never visible from outside
                            State::None
                        }
                        (
                            rest,
                            State::TypeDeclaration {
                                visibility,
                                identifier: d,
                                members: _,
                            },
                        ) => {
                            match d {
                                DeclType::Runtime(_) => panic!(),
                                DeclType::Compile(t, _, _) => {
                                    let t = sync!(t);
                                    let d = Declarator::Type(t);
                                    acc.solver.add_decl_simple(d, t);
                                }
                            };
                            match rest {
                                State::None => (),
                                _ => panic!(),
                            }
                            // we do not need declarations outside of the map to solve local variable
                            // because a local variable declaration is never visible from outside
                            State::None
                        }
                        (State::None, State::None) => State::None,
                        (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                    }
                } else if kind == &Type::SwitchBlockStatementGroup {
                    match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                        (rest, State::Declarations(v)) => {
                            for (d, t) in v {
                                let t = sync!(t);
                                let d = d.with_changed_node(|i| sync!(*i));
                                match rest {
                                    State::None => (),
                                    _ => panic!(),
                                }
                                match &d {
                                    Declarator::Variable(_) => acc.solver.add_decl_simple(d, t),
                                    _ => todo!(),
                                };
                            }
                            State::None
                        }
                        (
                            rest,
                            State::TypeDeclaration {
                                visibility,
                                identifier: d,
                                members: _,
                            },
                        ) => {
                            match d {
                                DeclType::Runtime(_) => panic!(),
                                DeclType::Compile(t, _, _) => {
                                    let t = sync!(t);
                                    let d = Declarator::Type(t);
                                    acc.solver.add_decl_simple(d, t);
                                }
                            };
                            match rest {
                                State::None => (),
                                _ => panic!(),
                            }
                            State::None
                        }
                        (State::None, State::None) => State::None,
                        (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                    }
                } else if kind == &Type::SwitchBlock {
                    // TODO retrieve decls not in Block from SwitchBlockStatementGroup
                    match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                        (rest, State::None) => rest, // TODO handle fall through declarations
                        (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                    }
                } else {
                    panic!("{:?}", kind)
                }
            } else if kind.is_structural_statement() {
                let mut remapper = acc.solver.local_solve_extend(&self.solver);
                macro_rules! sync {
                    ( $i:expr ) => {
                        remapper.intern_external(&mut acc.solver, $i.0)
                    };
                }
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::None) => State::None,

                    (State::None, State::LiteralType(t)) if kind == &Type::IfStatement => {
                        let t = sync!(t);
                        State::None
                    }
                    (State::None, State::LiteralType(_))
                        if kind == &Type::IfStatement
                            || kind == &Type::DoStatement
                            || kind == &Type::WhileStatement
                            || kind == &Type::SwitchStatement =>
                    {
                        State::None
                    }
                    (State::None, State::FieldIdentifier(_)) if kind == &Type::IfStatement => {
                        State::None
                    }
                    (State::None, State::ConstructorInvocation(i))
                        if kind == &Type::IfStatement =>
                    {
                        let i = sync!(i);
                        State::None
                    }
                    (State::None, State::ConstructorInvocation(i))
                        if kind == &Type::WhileStatement
                            || kind == &Type::DoStatement
                            || kind == &Type::SwitchStatement =>
                    {
                        let i = sync!(i);
                        State::None
                    }
                    (State::None, State::Invocation(i))
                        if kind == &Type::WhileStatement
                            || kind == &Type::DoStatement
                            || kind == &Type::SwitchStatement =>
                    {
                        let i = sync!(i);
                        State::None
                    }
                    (State::None, State::This(i))
                        if kind == &Type::WhileStatement
                            || kind == &Type::DoStatement
                            || kind == &Type::SwitchStatement =>
                    {
                        State::None
                    }
                    (State::None, State::ScopedIdentifier(i))
                        if kind == &Type::IfStatement
                            || kind == &Type::DoStatement
                            || kind == &Type::WhileStatement
                            || kind == &Type::SwitchStatement =>
                    {
                        State::None
                    }
                    (State::None, State::FieldIdentifier(i))
                        if kind == &Type::IfStatement
                            || kind == &Type::DoStatement
                            || kind == &Type::WhileStatement
                            || kind == &Type::SwitchStatement =>
                    {
                        State::None
                    }
                    (State::None, State::Invocation(i)) if kind == &Type::IfStatement => {
                        State::None
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind.is_simple_statement() {
                let mut remapper = acc.solver.extend(&self.solver);
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, rest) => {
                        match rest {
                            State::None => (),
                            State::FieldIdentifier(_) => (),
                            State::ScopedIdentifier(_) => (),
                            State::MethodReference(_) => (),
                            State::Invocation(_) => (),
                            State::ConstructorInvocation(_) => (),
                            State::LiteralType(_) => (),
                            State::This(_) => (),
                            State::LambdaExpression(_) => (),
                            State::SimpleIdentifier(i) => {
                                if kind == &Type::ExpressionStatement
                                    || kind == &Type::AssertStatement
                                    || kind == &Type::ReturnStatement
                                    || kind == &Type::SynchronizedStatement
                                    || kind == &Type::ThrowStatement
                                {
                                    scoped!(mm!(), i);
                                } else if kind == &Type::LabeledStatement
                                    || kind == &Type::BreakStatement
                                    || kind == &Type::ContinueStatement
                                {
                                } else {
                                    panic!()
                                }
                            }
                            State::SimpleIdentifier(i) => {}
                            x => panic!("{:?} {:?}", kind, x),
                        }
                        State::None
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else {
                panic!("{:?}", kind)
            }
        } else if kind.is_parameter() {
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }
            if kind == &Type::Resource {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleIdentifier(t)) if kind == &Type::Resource => {
                        let t = scoped!(mm!(), t);
                        State::ScopedIdentifier(t)
                    }
                    (State::None, State::ScopedIdentifier(t)) if kind == &Type::Resource => {
                        let t = sync!(t);
                        State::ScopedIdentifier(t)
                    }
                    (State::None, State::ScopedTypeIdentifier(t)) if kind == &Type::Resource => {
                        let t = sync!(t);
                        State::Declaration {
                            visibility: Visibility::None,
                            kind: t,
                            identifier: Declarator::None,
                        }
                    }
                    (State::None, State::SimpleTypeIdentifier(t)) if kind == &Type::Resource => {
                        let t = scoped!(mm!(), t);
                        State::Declaration {
                            visibility: Visibility::None,
                            kind: t,
                            identifier: Declarator::None,
                        }
                    }
                    (
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: _,
                        },
                        State::SimpleIdentifier(i),
                    ) if kind == &Type::Resource => {
                        let i = scoped!(mm!(), i);
                        let i = Declarator::Variable(i);
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: i,
                        }
                    }
                    (
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: Declarator::Variable(i),
                        },
                        rest,
                    ) if kind == &Type::Resource => {
                        match rest {
                            State::ConstructorInvocation(_) => (),
                            State::Invocation(_) => (),
                            State::ScopedIdentifier(_) => (), // not sure
                            x => todo!("{:?}", x),
                        };
                        let d = Declarator::Variable(i);
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: d,
                        }
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::FormalParameter {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleTypeIdentifier(t))
                        if kind == &Type::FormalParameter || kind == &Type::SpreadParameter =>
                    {
                        // TODO spread parameter is hard for invocation matching on check ? cannot use param ?
                        // TODO spread parameter is hard for decl matching on solve
                        // NOTE method ref resolution (overloading)
                        // 1)strict invocation: fixed arity method resolution, no boxing/unboxing )
                        // 2)loose invocation: fixed arity method resolution, boxing/unboxing
                        // 3)variable arity invocation: variable arity method resolution, boxing/unboxing
                        let t = scoped!(mm!(), t);
                        State::ScopedTypeIdentifier(t)
                    }
                    (State::None, State::ScopedTypeIdentifier(t))
                        if kind == &Type::FormalParameter || kind == &Type::SpreadParameter =>
                    {
                        let t = sync!(t);
                        State::ScopedTypeIdentifier(t)
                    }
                    (State::ScopedTypeIdentifier(t), State::SimpleIdentifier(i))
                        if kind == &Type::FormalParameter =>
                    {
                        let r = mm!();
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                        let i = Declarator::Variable(i);
                        State::Declaration {
                            visibility: Visibility::None,
                            kind: t,
                            identifier: i,
                        }
                    }
                    (State::SimpleTypeIdentifier(t), State::SimpleIdentifier(i))
                        if kind == &Type::FormalParameter =>
                    {
                        let t = scoped!(mm!(), t);
                        let r = mm!();
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                        let i = Declarator::Variable(i);
                        // no need because wont be used directly
                        // acc.solver.add_decl_simple(i.clone(), t);
                        State::Declaration {
                            visibility: Visibility::None,
                            kind: t,
                            identifier: i,
                        }
                    }
                    (
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: i,
                        },
                        State::Dimensions,
                    ) if kind == &Type::FormalParameter => State::Declaration {
                        visibility,
                        kind: acc.solver.intern_ref(RefsEnum::Array(t)),
                        identifier: i,
                    },
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::TypeParameter {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    // TypeParameter
                    (State::None, State::None) if kind == &Type::TypeParameter => State::None,
                    (State::None, State::SimpleIdentifier(i)) if kind == &Type::TypeParameter => {
                        State::SimpleIdentifier(i)
                    }
                    (State::SimpleIdentifier(i), State::TypeBound)
                        if kind == &Type::TypeParameter =>
                    {
                        // TODO use type bound
                        State::SimpleIdentifier(i)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::SpreadParameter {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::ScopedTypeIdentifier(t), State::Declarator(d))
                        if kind == &Type::SpreadParameter =>
                    {
                        let i = match d {
                            Declarator::Variable(t) => sync!(t),
                            _ => panic!(),
                        };
                        let i = Declarator::Variable(i);
                        State::Declaration {
                            visibility: Visibility::None,
                            kind: t,
                            identifier: i,
                        }
                        // State::ScopedTypeIdentifier(t)
                    }
                    (State::None, State::ScopedTypeIdentifier(t))
                        if kind == &Type::SpreadParameter =>
                    {
                        let t = sync!(t);
                        State::ScopedTypeIdentifier(t)
                    }
                    (State::None, State::SimpleTypeIdentifier(t))
                        if kind == &Type::SpreadParameter =>
                    {
                        let t = scoped!(mm!(), t);
                        State::ScopedTypeIdentifier(t)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::CatchFormalParameter {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::CatchTypes(v)) if kind == &Type::CatchFormalParameter => {
                        State::CatchTypes(v.iter().map(|x| sync!(x)).collect())
                    }
                    (State::CatchTypes(v), State::SimpleIdentifier(i))
                        if kind == &Type::CatchFormalParameter =>
                    {
                        State::CatchParameter {
                            kinds: v.into_boxed_slice(),
                            identifier: i,
                        }
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else {
                panic!("{:?}", kind)
            }
        } else if kind.is_expression() {
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }
            if kind == &Type::LambdaExpression {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleIdentifier(i))
                        if kind == &Type::LambdaExpression =>
                    {
                        let r = mm!();
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                        let i = Declarator::Variable(i);
                        acc.solver.add_decl_simple(i.clone(), r);
                        State::Declarations(vec![(i, r)])
                    }
                    (State::None, State::Declarations(v)) if kind == &Type::LambdaExpression => {
                        let v = v
                            .into_iter()
                            .map(|(i, t)| {
                                let i = i.with_changed_node(|i| sync!(*i));
                                let t = sync!(t);
                                acc.solver.add_decl_simple(i.clone(), t);
                                (i, t)
                            })
                            .collect();
                        State::Declarations(v)
                    }
                    (State::None, State::FormalParameters(v))
                        if kind == &Type::LambdaExpression =>
                    {
                        let v = v
                            .into_iter()
                            .map(|(i, t)| {
                                let i = sync!(i);
                                let t = sync!(t);
                                let i = Declarator::Variable(i);
                                acc.solver.add_decl_simple(i.clone(), t);
                                (i, t)
                            })
                            .collect();
                        State::Declarations(v)
                    }
                    (State::Declarations(p), State::None) if kind == &Type::LambdaExpression => {
                        let i = mm!();
                        State::LambdaExpression(i)
                    }
                    (State::Declarations(p), State::Invocation(i))
                        if kind == &Type::LambdaExpression =>
                    {
                        // TODO solve references to parameters
                        let i = sync!(i);
                        let i = mm!();
                        State::LambdaExpression(i)
                    }
                    (State::Declarations(p), State::FieldIdentifier(i))
                        if kind == &Type::LambdaExpression =>
                    {
                        // TODO solve references to parameters
                        let i = sync!(i);
                        let i = mm!();
                        State::LambdaExpression(i)
                    }
                    (State::Declarations(p), State::ConstructorInvocation(i))
                        if kind == &Type::LambdaExpression =>
                    {
                        // TODO solve references to parameters
                        let i = sync!(i);
                        let i = mm!();
                        State::LambdaExpression(i)
                    }
                    (State::Declarations(p), State::ScopedIdentifier(i))
                        if kind == &Type::LambdaExpression =>
                    {
                        // TODO solve references to parameters
                        let i = sync!(i);
                        let i = mm!();
                        State::LambdaExpression(i)
                    }
                    (State::Declarations(p), State::SimpleIdentifier(i))
                        if kind == &Type::LambdaExpression =>
                    {
                        let i = scoped!(mm!(), i);
                        // TODO solve references to parameters
                        let i = mm!();
                        State::LambdaExpression(i)
                    }
                    (State::Declarations(p), State::This(i)) if kind == &Type::LambdaExpression => {
                        let i = sync!(i);
                        // TODO solve references to parameters
                        let i = mm!();
                        State::LambdaExpression(i)
                    }
                    (State::Declarations(p), State::LiteralType(t))
                        if kind == &Type::LambdaExpression =>
                    {
                        // TODO solve references to parameters
                        let t = sync!(t);
                        State::LiteralType(t)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ArrayCreationExpression {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleTypeIdentifier(i))
                        if kind == &Type::ArrayCreationExpression =>
                    {
                        let r = mm!();
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                        let i = acc
                            .solver
                            .intern(RefsEnum::ConstructorInvocation(i, Arguments::Unknown));
                        State::ConstructorInvocation(i)
                    }
                    (State::None, State::ScopedTypeIdentifier(i))
                        if kind == &Type::ArrayCreationExpression =>
                    {
                        let i = sync!(i);
                        let i = acc
                            .solver
                            .intern(RefsEnum::ConstructorInvocation(i, Arguments::Unknown));
                        State::ConstructorInvocation(i)
                    }
                    (State::ConstructorInvocation(i), State::None)
                        if kind == &Type::ArrayCreationExpression =>
                    {
                        State::ConstructorInvocation(i)
                    }
                    (State::ConstructorInvocation(i), rest)
                        if kind == &Type::ArrayCreationExpression =>
                    {
                        match rest {
                            State::Dimensions => (),
                            State::ScopedIdentifier(_) => (),
                            State::LiteralType(_) => (),
                            State::SimpleIdentifier(i) => {
                                scoped!(mm!(), i);
                            }
                            x => todo!("{:?}", x),
                        };
                        let (o, p) = match &acc.solver.nodes[i] {
                            RefsEnum::ConstructorInvocation(o, p) => (*o, p.clone()),
                            x => todo!("{:?}", x),
                        };
                        let i = acc.solver.intern(RefsEnum::Array(o));
                        let i = acc.solver.intern(RefsEnum::ConstructorInvocation(i, p));
                        State::ConstructorInvocation(i)
                    }
                    // // (State::ConstructorInvocation(i), State::LiteralType(_))
                    // //     if kind == &Type::ArrayCreationExpression =>
                    // // {
                    // //     // TODO use the dimension expr
                    // //     State::ConstructorInvocation(i)
                    // // }
                    // (State::ScopedIdentifier(i), State::LiteralType(_))
                    //     if kind == &Type::ArrayCreationExpression =>
                    // {
                    //     let i = acc
                    //         .solver
                    //         .intern_ref(RefsEnum::ConstructorInvocation(i, Arguments::Unknown));
                    //     // TODO use dimension
                    //     State::ConstructorInvocation(i)
                    // }
                    // (
                    //     State::ScopedIdentifier(i),
                    //     State::FieldIdentifier(_),
                    // ) if kind == &Type::ArrayCreationExpression => {
                    //     let i = acc
                    //         .solver
                    //         .intern_ref(RefsEnum::ConstructorInvocation(i, Arguments::Unknown));
                    //     // TODO use dimension
                    //     State::ConstructorInvocation(i)
                    // }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ObjectCreationExpression {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleTypeIdentifier(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        let r = mm!();
                        State::InvocationId(r, i)
                    }
                    (State::None, State::SimpleIdentifier(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        State::SimpleIdentifier(i)
                    }
                    (State::None, State::ScopedIdentifier(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::SimpleIdentifier(o), State::SimpleTypeIdentifier(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        let o = scoped!(mm!(), o);
                        State::InvocationId(o, i)
                    }
                    (State::ScopedIdentifier(o), State::SimpleTypeIdentifier(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        State::InvocationId(o, i)
                    }
                    (State::SimpleTypeIdentifier(o), State::SimpleTypeIdentifier(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        let o = scoped!(mm!(), o);
                        State::InvocationId(o, i)
                    }
                    (State::ScopedTypeIdentifier(o), State::ScopedTypeIdentifier(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        let i = sync!(i);
                        let i = acc.solver.solve_node_with(i, o);
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::None, State::ScopedTypeIdentifier(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        let i = sync!(i);
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::ScopedTypeIdentifier(i), State::Arguments(p))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        let p = p.deref().iter().map(|i| sync!(*i)).collect();
                        let r = acc
                            .solver
                            .intern_ref(RefsEnum::ConstructorInvocation(i, Arguments::Given(p)));
                        State::ConstructorInvocation(r)
                    }
                    (State::InvocationId(r, i), State::Arguments(p))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        // TODO invocationId may not be the best way
                        let p = p.deref().iter().map(|i| sync!(*i)).collect();
                        let i = scoped!(r, i);
                        let r = acc
                            .solver
                            .intern_ref(RefsEnum::ConstructorInvocation(i, Arguments::Given(p)));
                        State::ConstructorInvocation(r)
                    }
                    (State::ConstructorInvocation(r), State::Declarations(v))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        State::ConstructorInvocation(r)
                    }
                    (State::ConstructorInvocation(r), State::None)
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        State::ConstructorInvocation(r)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }

            // }else if kind == &Type::TernaryExpression {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else if kind == &Type::BinaryExpression {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else if kind == &Type::AssignmentExpression {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else if kind == &Type::VariableDeclarator {
            // match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //     (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            // }
            // } else if kind == &Type::InstanceofExpression {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            //     }

            // } else if kind == &Type::CastExpression {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else if kind == &Type::UpdateExpression {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else if kind == &Type::ParenthesizedExpression {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            //     }
            } else if kind == &Type::MethodInvocation {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    // MethodInvocation f()
                    (State::None, State::SimpleIdentifier(t))
                        if kind == &Type::MethodInvocation =>
                    {
                        State::SimpleIdentifier(t)
                    }
                    (State::SimpleIdentifier(i), State::Arguments(p))
                        if kind == &Type::MethodInvocation =>
                    {
                        let p = p.deref().iter().map(|i| sync!(*i)).collect();
                        let r = mm!();
                        let r =
                            acc.solver
                                .intern_ref(RefsEnum::Invocation(r, i, Arguments::Given(p)));
                        State::ScopedIdentifier(r) // or should it be an invocation
                    }
                    (State::SimpleIdentifier(i), State::TypeArguments(p))
                        if kind == &Type::MethodInvocation =>
                    {
                        State::SimpleIdentifier(i)
                    }
                    (State::ScopedIdentifier(i), State::TypeArguments(p))
                        if kind == &Type::MethodInvocation =>
                    {
                        // TODO handle type argmuments
                        // todo!(
                        //     "{:?}",
                        //     ExplorableRef {
                        //         rf: i,
                        //         nodes: &acc.solver.nodes
                        //     }
                        // );
                        // let p = p.deref().iter().map(|i| sync!(*i)).collect();
                        // let r = mm!();
                        // let r =
                        //     acc.solver
                        //         .intern_ref(RefsEnum::Invocation(r, i, Arguments::Given(p)));
                        State::ScopedIdentifier(i) // or should it be an invocation
                    }
                    // MethodInvocation x.f()
                    (State::None, expr) if kind == &Type::MethodInvocation => {
                        let i = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::SimpleIdentifier(t) => panic!("should be handled specifically"),
                            State::This(i) => sync!(i),
                            State::Super(i) => sync!(i),
                            State::ScopedIdentifier(i) => sync!(i),
                            State::FieldIdentifier(i) => sync!(i),
                            State::Invocation(i) => sync!(i),
                            State::ConstructorInvocation(i) => sync!(i),
                            State::None => panic!(""),
                            x => panic!("{:?}", x),
                        };
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(i), State::Arguments(p))
                        if kind == &Type::MethodInvocation =>
                    {
                        // todo!("{:?}",ExplorableRef{rf:i,nodes:&acc.solver.nodes});
                        let p = p.deref().iter().map(|i| sync!(*i)).collect();
                        match &acc.solver.nodes[i] {
                            RefsEnum::ScopedIdentifier(o, i) => {
                                let r = acc.solver.intern_ref(RefsEnum::Invocation(
                                    *o,
                                    *i,
                                    Arguments::Given(p),
                                ));
                                State::ScopedIdentifier(r)
                            }
                            x => panic!(
                                "{:?} {:?}",
                                ExplorableRef {
                                    rf: i,
                                    nodes: &acc.solver.nodes
                                },
                                x
                            ),
                        }
                    }
                    (State::ScopedIdentifier(o), expr) if kind == &Type::MethodInvocation => {
                        match expr {
                            State::SimpleIdentifier(i) => State::InvocationId(o, i),
                            State::This(i) => State::ScopedIdentifier(spec!(o, sync!(i))),
                            State::Super(i) => State::ScopedIdentifier(spec!(o, sync!(i))),
                            x => panic!(
                                "{:?} {:?}",
                                ExplorableRef {
                                    rf: o,
                                    nodes: &acc.solver.nodes
                                },
                                x
                            ),
                        }
                    }
                    (State::SimpleIdentifier(o), expr) if kind == &Type::MethodInvocation => {
                        match expr {
                            State::SimpleIdentifier(i) => State::InvocationId(scoped!(mm!(), o), i),
                            State::This(i) => {
                                State::ScopedIdentifier(spec!(scoped!(mm!(), o), sync!(i)))
                            }
                            State::Super(i) => {
                                State::ScopedIdentifier(spec!(scoped!(mm!(), o), sync!(i)))
                            }
                            x => panic!("{:?}", x),
                        }
                    }
                    (State::InvocationId(o, i), State::Arguments(p))
                        if kind == &Type::MethodInvocation =>
                    {
                        let p = p.deref().iter().map(|i| sync!(*i)).collect();
                        let r =
                            acc.solver
                                .intern_ref(RefsEnum::Invocation(o, i, Arguments::Given(p)));
                        State::ScopedIdentifier(r) // or should it be an invocation
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::MethodReference {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, expr) if kind == &Type::MethodReference => {
                        let i = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::SimpleIdentifier(t) => scoped!(mm!(), t),
                            State::SimpleTypeIdentifier(t) => scoped!(mm!(), t),
                            State::This(t) => sync!(t),
                            State::ScopedTypeIdentifier(i) => sync!(i), // TODO fix related to getting type alias from tree-sitter API
                            State::ScopedIdentifier(i) => sync!(i),
                            State::FieldIdentifier(i) => panic!("not possible"),
                            State::Invocation(i) => panic!("not possible"),
                            State::ConstructorInvocation(i) => panic!("not possible"),
                            State::None => panic!("should handle before"),
                            x => panic!("{:?}", x),
                        };
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(o), State::SimpleIdentifier(i))
                        if kind == &Type::MethodReference =>
                    {
                        let r = acc.solver.intern_ref(RefsEnum::MethodReference(o, i));
                        State::MethodReference(r)
                    }
                    (State::ScopedIdentifier(o), State::None) if kind == &Type::MethodReference => {
                        let r = acc
                            .solver
                            .intern_ref(RefsEnum::ConstructorInvocation(o, Arguments::Unknown));
                        State::MethodReference(r)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ExplicitConstructorInvocation {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    // this() or super()
                    // TODO ExplicitConstructorInvocation try not to pollute ref resolution
                    (State::None, expr) if kind == &Type::ExplicitConstructorInvocation => {
                        match &expr {
                            State::SimpleIdentifier(i) => State::SimpleIdentifier(*i),
                            State::ScopedIdentifier(i) => State::ScopedIdentifier(sync!(*i)),
                            State::This(i) => State::This(sync!(*i)),
                            State::Super(i) => State::Super(sync!(*i)),
                            x => panic!("{:?}", x),
                        }
                    }
                    (State::ScopedIdentifier(o), State::SimpleIdentifier(i))
                        if kind == &Type::ExplicitConstructorInvocation =>
                    {
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(o, i));
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(o), State::Super(i))
                        if kind == &Type::ExplicitConstructorInvocation =>
                    {
                        let i = spec!(o, sync!(i));
                        State::ScopedIdentifier(i)
                    }
                    (State::SimpleIdentifier(o), State::Super(i))
                        if kind == &Type::ExplicitConstructorInvocation =>
                    {
                        let i = spec!(scoped!(mm!(), o), sync!(i));
                        State::ScopedIdentifier(i)
                    }
                    (expr, State::Arguments(p)) if kind == &Type::ExplicitConstructorInvocation => {
                        let i = match expr {
                            State::ScopedIdentifier(i) => i,
                            State::Super(i) => i,
                            State::This(i) => i,
                            _ => panic!(),
                        };
                        let p = p.deref().iter().map(|i| sync!(*i)).collect();
                        let i = acc
                            .solver
                            .intern_ref(RefsEnum::ConstructorInvocation(i, Arguments::Given(p)));
                        State::ConstructorInvocation(i)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }

            // } else if kind == &Type::ClassLiteral {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else if kind == &Type::FieldAccess {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else if kind == &Type::ArrayAccess {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            //     }
            } else {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleTypeIdentifier(t))
                        if kind == &Type::InstanceofExpression =>
                    {
                        scoped!(mm!(), t);
                        State::None
                    }
                    (State::Invocation(_), State::SimpleTypeIdentifier(i))
                        if kind == &Type::InstanceofExpression =>
                    {
                        let i = scoped!(mm!(), i);
                        // TODO intern boolean
                        State::ScopedIdentifier(mm!())
                    }
                    (State::ScopedIdentifier(_), State::SimpleTypeIdentifier(i))
                        if kind == &Type::InstanceofExpression =>
                    {
                        let i = scoped!(mm!(), i);
                        // TODO intern boolean
                        State::ScopedIdentifier(mm!())
                    }
                    (State::ScopedIdentifier(_), State::ScopedTypeIdentifier(i))
                        if kind == &Type::InstanceofExpression =>
                    {
                        // TODO intern boolean
                        State::ScopedIdentifier(mm!())
                    }

                    // array access
                    (State::None, expr) if kind == &Type::ArrayAccess => {
                        // TODO simp more FieldIdentifiers to ScopedIdentifier
                        let i = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::SimpleIdentifier(t) => scoped!(mm!(), t),
                            State::ScopedIdentifier(i) => sync!(i),
                            State::FieldIdentifier(i) => sync!(i),
                            State::Invocation(i) => sync!(i),
                            State::ConstructorInvocation(i) => sync!(i),
                            x => panic!("{:?}", x),
                        };
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(o), expr) if kind == &Type::ArrayAccess => {
                        // TODO create RefsEnum variant to use access expr and solve type of array
                        match expr {
                            State::LiteralType(t) => (),
                            State::SimpleIdentifier(t) => {
                                scoped!(mm!(), t);
                            }
                            State::This(t) => (),
                            State::ScopedIdentifier(_) => (),
                            State::FieldIdentifier(_) => (),
                            State::Invocation(_) => (),
                            State::ConstructorInvocation(_) => (),
                            // State::None => (), // TODO check
                            x => panic!("{:?}", x),
                        };
                        let o = acc.solver.intern_ref(RefsEnum::ArrayAccess(o));
                        State::ScopedIdentifier(o)
                    }
                    // field access
                    (State::None, expr) if kind == &Type::FieldAccess =>
                    //TODO get type
                    {
                        let i = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::SimpleIdentifier(t) => scoped!(mm!(), t),
                            State::This(i) => sync!(i),
                            State::Super(i) => sync!(i),
                            State::ScopedIdentifier(i) => sync!(i),
                            State::FieldIdentifier(i) => sync!(i),
                            State::Invocation(i) => sync!(i),
                            State::ConstructorInvocation(i) => sync!(i),
                            State::None => panic!("should handle super"),
                            x => panic!("{:?}", x),
                        };
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(o), State::SimpleIdentifier(i))
                        if kind == &Type::FieldAccess =>
                    {
                        let i = scoped!(o, i);
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(o), State::This(i)) if kind == &Type::FieldAccess => {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }

                    // literal
                    (State::None, State::ScopedTypeIdentifier(i))
                        if kind == &Type::ClassLiteral =>
                    {
                        // TODO should return Class<i>
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::None, State::SimpleTypeIdentifier(i))
                        if kind == &Type::ClassLiteral =>
                    {
                        // TODO should return Class<i>
                        let i = scoped!(mm!(), i);
                        State::ScopedIdentifier(i)
                    }

                    // CastExpression
                    (State::None, expr) if kind == &Type::CastExpression => {
                        let t = match expr {
                            State::SimpleTypeIdentifier(t) => scoped!(mm!(), t),
                            State::ScopedTypeIdentifier(i) => sync!(i),
                            State::None => panic!("should handle before"),
                            x => panic!("{:?}", x),
                        };
                        State::ScopedTypeIdentifier(t)
                    }
                    (State::ScopedTypeIdentifier(t), expr) if kind == &Type::CastExpression => {
                        let i = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::This(i) => sync!(i),
                            State::SimpleIdentifier(t) => scoped!(mm!(), t),
                            State::SimpleTypeIdentifier(t) => scoped!(mm!(), t), // should not append
                            State::ScopedIdentifier(i) => sync!(i),
                            State::LambdaExpression(i) => sync!(i),
                            State::FieldIdentifier(i) => sync!(i),
                            State::Invocation(i) => sync!(i),
                            State::ConstructorInvocation(i) => sync!(i),
                            State::MethodReference(i) => sync!(i),
                            State::ScopedTypeIdentifier(i) => panic!(),
                            State::None => panic!("should handle before"),
                            x => panic!("{:?}", x),
                        };
                        State::ScopedIdentifier(t)
                    }
                    (State::ScopedIdentifier(t), expr) if kind == &Type::CastExpression => {
                        // should be ScopedTypeIdentifier but cannot get alias from treesitter rust API cleanly
                        let i = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::This(i) => sync!(i),
                            State::SimpleIdentifier(t) => scoped!(mm!(), t),
                            State::SimpleTypeIdentifier(t) => scoped!(mm!(), t), // should not append
                            State::ScopedIdentifier(i) => sync!(i),
                            State::LambdaExpression(i) => sync!(i),
                            State::FieldIdentifier(i) => sync!(i),
                            State::Invocation(i) => sync!(i),
                            State::ConstructorInvocation(i) => sync!(i),
                            State::MethodReference(i) => sync!(i),
                            State::ScopedTypeIdentifier(i) => panic!(),
                            State::None => panic!("should handle before"),
                            x => panic!("{:?}", x),
                        };
                        State::ScopedIdentifier(t)
                    }
                    (State::None, State::SimpleIdentifier(i))
                        if kind == &Type::BinaryExpression
                            || kind == &Type::UnaryExpression
                            || kind == &Type::AssignmentExpression
                            || kind == &Type::InstanceofExpression
                            || kind == &Type::UpdateExpression
                            || kind == &Type::ParenthesizedExpression =>
                    {
                        let i = scoped!(mm!(), i);
                        State::ScopedIdentifier(i)
                    }
                    (State::None, State::ScopedIdentifier(i))
                        if kind == &Type::BinaryExpression
                            || kind == &Type::UnaryExpression
                            || kind == &Type::AssignmentExpression
                            || kind == &Type::InstanceofExpression
                            || kind == &Type::UpdateExpression
                            || kind == &Type::ParenthesizedExpression =>
                    {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::None, State::ConstructorInvocation(i))
                        if kind == &Type::InstanceofExpression
                            || kind == &Type::BinaryExpression
                            || kind == &Type::UnaryExpression
                            || kind == &Type::ParenthesizedExpression
                            || kind == &Type::UpdateExpression =>
                    {
                        let i = sync!(i);
                        State::ConstructorInvocation(i)
                    }
                    (State::None, State::Invocation(i))
                        if kind == &Type::InstanceofExpression
                            || kind == &Type::BinaryExpression
                            || kind == &Type::UnaryExpression
                            || kind == &Type::ParenthesizedExpression
                            || kind == &Type::UpdateExpression =>
                    {
                        // TODO TODO regroup right and match inside
                        let i = sync!(i);
                        State::Invocation(i)
                    }
                    (State::None, State::FieldIdentifier(i))
                        if kind == &Type::InstanceofExpression
                            || kind == &Type::UpdateExpression
                            || kind == &Type::ParenthesizedExpression =>
                    {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::None, State::LiteralType(t))
                        if kind == &Type::BinaryExpression
                            || kind == &Type::UnaryExpression
                            || kind == &Type::AssignmentExpression
                            || kind == &Type::ParenthesizedExpression =>
                    {
                        let t = sync!(t);
                        State::LiteralType(t)
                    }
                    (State::None, State::FieldIdentifier(i))
                        if kind == &Type::BinaryExpression
                            || kind == &Type::UnaryExpression
                            || kind == &Type::UpdateExpression
                            || kind == &Type::AssignmentExpression
                            || kind == &Type::ParenthesizedExpression =>
                    {
                        let i = sync!(i);
                        State::FieldIdentifier(i)
                    }
                    (State::None, State::This(i))
                        if kind == &Type::BinaryExpression
                            || kind == &Type::InstanceofExpression
                            || kind == &Type::ParenthesizedExpression =>
                    {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(_), State::LiteralType(t))
                        if kind == &Type::AssignmentExpression =>
                    {
                        let t = sync!(t);
                        State::LiteralType(t)
                    }
                    (State::ScopedIdentifier(i), State::This(_))
                        if kind == &Type::AssignmentExpression =>
                    {
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(i), State::ConstructorInvocation(_))
                        if kind == &Type::AssignmentExpression =>
                    {
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(i), State::LambdaExpression(_))
                        if kind == &Type::AssignmentExpression =>
                    {
                        State::ScopedIdentifier(i)
                    }
                    (State::FieldIdentifier(_), State::ScopedIdentifier(i))
                        if kind == &Type::AssignmentExpression =>
                    {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(i0), State::SimpleIdentifier(i))
                        if kind == &Type::AssignmentExpression =>
                    {
                        let i = scoped!(mm!(), i);
                        State::ScopedIdentifier(i0)
                    }
                    (State::ScopedIdentifier(i0), State::ScopedIdentifier(i))
                        if kind == &Type::AssignmentExpression =>
                    {
                        State::ScopedIdentifier(i0)
                    }
                    (State::ScopedIdentifier(i0), State::FieldIdentifier(i))
                        if kind == &Type::AssignmentExpression =>
                    {
                        State::ScopedIdentifier(i0)
                    }
                    (State::FieldIdentifier(i0), State::FieldIdentifier(i))
                        if kind == &Type::AssignmentExpression =>
                    {
                        State::FieldIdentifier(i0)
                    }
                    // TernaryExpression
                    // TernaryExpression (None,c)
                    (State::None, c) if kind == &Type::TernaryExpression => {
                        match c {
                            State::SimpleIdentifier(i) => {
                                let i = scoped!(mm!(), i);
                            }
                            State::LiteralType(t) => (),
                            State::ScopedIdentifier(i) => {
                                sync!(i);
                            }
                            State::Invocation(_) => (),
                            State::FieldIdentifier(_) => (),
                            x => todo!("{:?}", x),
                        };
                        State::Condition
                    }
                    // TernaryExpression (Cond,x)
                    (State::Condition, x) if kind == &Type::TernaryExpression => match x {
                        State::LiteralType(t) => {
                            let i = sync!(t);
                            State::ScopedIdentifier(i)
                        }
                        State::SimpleIdentifier(i) => {
                            let i = scoped!(mm!(), i);
                            State::ScopedIdentifier(i)
                        }
                        State::This(i) => {
                            let i = sync!(i);
                            State::ScopedIdentifier(i)
                        }
                        State::ConstructorInvocation(i) => State::ConstructorInvocation(sync!(i)),
                        State::Invocation(i) => State::Invocation(sync!(i)),
                        State::ScopedIdentifier(i) => State::ScopedIdentifier(sync!(i)),
                        State::FieldIdentifier(i) => State::FieldIdentifier(sync!(i)),
                        State::MethodReference(i) => State::MethodReference(sync!(i)),
                        State::None => panic!(),
                        x => todo!("{:?}", x),
                    },
                    // TernaryExpression (x,y)
                    (State::LiteralType(t), y) if kind == &Type::TernaryExpression => {
                        match y {
                            State::LiteralType(t) => (),
                            State::SimpleIdentifier(i) => {
                                scoped!(mm!(), i);
                            }
                            State::FieldIdentifier(i) => (),
                            State::ConstructorInvocation(i) => (),
                            State::Invocation(i) => (),
                            State::ScopedIdentifier(i) => (),
                            State::None => panic!(),
                            x => todo!("{:?}", x),
                        };
                        State::LiteralType(t)
                    }
                    (x, State::LiteralType(t)) if kind == &Type::TernaryExpression => {
                        match x {
                            State::LiteralType(t) => (),
                            State::SimpleIdentifier(i) => {
                                scoped!(mm!(), i);
                            }
                            State::ConstructorInvocation(i) => (),
                            State::Invocation(i) => (),
                            State::ScopedIdentifier(i) => (),
                            State::None => panic!(),
                            x => todo!("{:?}", x),
                        };
                        assert_ne!(x, State::Condition);
                        let t = sync!(t);
                        State::LiteralType(t)
                    }
                    (State::SimpleIdentifier(i), _) if kind == &Type::TernaryExpression => {
                        let i = scoped!(mm!(), i);
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(i), _) if kind == &Type::TernaryExpression => {
                        State::ScopedIdentifier(i)
                    }
                    (State::Invocation(_), State::ScopedIdentifier(i))
                        if kind == &Type::TernaryExpression =>
                    {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::ConstructorInvocation(i), State::ScopedIdentifier(_))
                        if kind == &Type::TernaryExpression =>
                    {
                        State::ConstructorInvocation(i)
                    }
                    (State::MethodReference(i), State::LambdaExpression(_))
                        if kind == &Type::TernaryExpression =>
                    {
                        State::ConstructorInvocation(i)
                    }
                    (State::FieldIdentifier(_), State::SimpleIdentifier(i))
                        if kind == &Type::TernaryExpression =>
                    {
                        let i = scoped!(mm!(), i);
                        State::ScopedIdentifier(i)
                    }
                    (State::FieldIdentifier(_), State::ScopedIdentifier(i))
                        if kind == &Type::TernaryExpression =>
                    {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::Invocation(_), State::SimpleIdentifier(i))
                        if kind == &Type::TernaryExpression =>
                    {
                        let i = scoped!(mm!(), i);
                        State::ScopedIdentifier(i)
                    }
                    (State::Invocation(i), State::Invocation(_))
                        if kind == &Type::TernaryExpression =>
                    {
                        State::Invocation(i)
                    }
                    (State::Invocation(i), State::FieldIdentifier(_))
                        if kind == &Type::TernaryExpression =>
                    {
                        State::Invocation(i)
                    }
                    (State::ConstructorInvocation(i), State::ConstructorInvocation(_))
                        if kind == &Type::TernaryExpression =>
                    {
                        State::ConstructorInvocation(i)
                    }
                    (State::FieldIdentifier(i), State::FieldIdentifier(_))
                        if kind == &Type::TernaryExpression =>
                    {
                        State::FieldIdentifier(i)
                    }
                    (State::FieldIdentifier(i), State::Invocation(_))
                        if kind == &Type::TernaryExpression =>
                    {
                        State::FieldIdentifier(i)
                    }
                    (State::Invocation(_), State::ConstructorInvocation(i))
                        if kind == &Type::TernaryExpression =>
                    {
                        let i = sync!(i);
                        State::ConstructorInvocation(i)
                    }
                    (State::ConstructorInvocation(i), State::Invocation(_))
                        if kind == &Type::TernaryExpression =>
                    {
                        State::ConstructorInvocation(i)
                    }
                    (State::ConstructorInvocation(t), State::SimpleIdentifier(i))
                        if kind == &Type::TernaryExpression =>
                    {
                        let i = scoped!(mm!(), i);
                        State::ConstructorInvocation(t)
                    }
                    (State::ConstructorInvocation(t), State::This(i))
                        if kind == &Type::TernaryExpression =>
                    {
                        let i = sync!(i);
                        State::This(i)
                    }

                    (State::None, State::ScopedIdentifier(i))
                        if kind == &Type::ParenthesizedExpression =>
                    {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::None, State::LambdaExpression(i))
                        if kind == &Type::ParenthesizedExpression =>
                    {
                        let i = sync!(i);
                        State::LambdaExpression(i)
                    }
                    (State::None, State::SimpleIdentifier(i))
                        if kind == &Type::ParenthesizedExpression =>
                    {
                        let i = scoped!(mm!(), i);
                        State::ScopedIdentifier(i)
                    }

                    (State::ScopedIdentifier(il), State::SimpleIdentifier(ir))
                        if kind == &Type::BinaryExpression =>
                    {
                        scoped!(mm!(), ir);
                        State::ScopedIdentifier(il)
                    }
                    (State::ScopedIdentifier(i), State::LiteralType(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(i), State::This(t))
                        if kind == &Type::BinaryExpression =>
                    {
                        let t = sync!(t);
                        State::ScopedIdentifier(i)
                    }
                    (State::FieldIdentifier(i), State::ScopedIdentifier(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::FieldIdentifier(i)
                    }
                    (State::LiteralType(t), _) if kind == &Type::BinaryExpression => {
                        // TODO not that obvious in general
                        State::LiteralType(t)
                    }
                    (State::ScopedIdentifier(i), State::ScopedIdentifier(_))
                        if kind == &Type::BinaryExpression
                            || kind == &Type::AssignmentExpression =>
                    {
                        State::ScopedIdentifier(i)
                    }
                    (State::LiteralType(t), State::SimpleTypeIdentifier(i))
                        if kind == &Type::BinaryExpression =>
                    {
                        let i = scoped!(mm!(), i);
                        State::LiteralType(t)
                    }
                    (State::LiteralType(t), _) if kind == &Type::BinaryExpression => {
                        State::LiteralType(t)
                    }
                    (State::Invocation(i), State::Invocation(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::Invocation(i)
                    }
                    (State::This(i), State::Invocation(_)) if kind == &Type::BinaryExpression => {
                        State::ScopedIdentifier(i)
                    }
                    (State::Invocation(i), State::LiteralType(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::Invocation(i)
                    }
                    (State::Invocation(i), State::This(t)) if kind == &Type::BinaryExpression => {
                        let t = sync!(t);
                        State::Invocation(i)
                    }
                    (State::Invocation(i), State::ScopedIdentifier(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::Invocation(i)
                    }
                    (State::Invocation(i), State::FieldIdentifier(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::Invocation(i)
                    }
                    (State::FieldIdentifier(i), State::LiteralType(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::FieldIdentifier(i)
                    }
                    (State::FieldIdentifier(i), State::Invocation(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::FieldIdentifier(i)
                    }
                    (State::Invocation(i0), State::SimpleIdentifier(i))
                        if kind == &Type::BinaryExpression =>
                    {
                        let i = scoped!(mm!(), i);
                        State::Invocation(i0)
                    }
                    (State::FieldIdentifier(i0), State::SimpleIdentifier(i))
                        if kind == &Type::BinaryExpression =>
                    {
                        let i = scoped!(mm!(), i);
                        State::FieldIdentifier(i0)
                    }
                    (State::FieldIdentifier(i), State::FieldIdentifier(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::FieldIdentifier(i)
                    }

                    (State::Invocation(_), State::LiteralType(t))
                        if kind == &Type::AssignmentExpression =>
                    {
                        let t = sync!(t);
                        State::LiteralType(t)
                    }
                    (State::FieldIdentifier(_), State::LiteralType(t))
                        if kind == &Type::AssignmentExpression =>
                    {
                        let t = sync!(t);
                        State::LiteralType(t)
                    }
                    (State::FieldIdentifier(i0), State::SimpleIdentifier(i))
                        if kind == &Type::AssignmentExpression =>
                    {
                        let i = scoped!(mm!(), i);
                        State::FieldIdentifier(i0)
                    }
                    (State::FieldIdentifier(_), State::ConstructorInvocation(i))
                        if kind == &Type::AssignmentExpression =>
                    {
                        let i = sync!(i);
                        State::ConstructorInvocation(i)
                    }
                    (State::FieldIdentifier(_), State::Invocation(i))
                        if kind == &Type::AssignmentExpression =>
                    {
                        let i = sync!(i);
                        State::ConstructorInvocation(i)
                    }
                    (State::None, State::ScopedIdentifier(i))
                        if kind == &Type::BinaryExpression
                            || kind == &Type::AssignmentExpression =>
                    {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(i), State::LiteralType(t))
                        if kind == &Type::AssignmentExpression =>
                    {
                        let t = sync!(t);
                        State::LiteralType(t)
                    }
                    (State::ScopedIdentifier(i), State::Invocation(_))
                        if kind == &Type::BinaryExpression
                            || kind == &Type::AssignmentExpression =>
                    {
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(i), State::ConstructorInvocation(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::ScopedIdentifier(i)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            }
        } else if kind == &Type::Directory {
            // let mut s: Solver = Default::default();
            // s.nodes = self.solver.nodes;
            // s.refs.resize(self.solver.refs.len(), false);
            // let mut remapper = acc.solver.extend(&s);
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }

            match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                (rest, State::None) => rest,
                (State::None, State::File{
                    package,
                    content,
                    top_level,
                    ..//TODO
                }) =>
                {
                    let package = package.map(|p|sync!(p));
                    State::Package{
                        package,
                        global_decls: vec![],
                        package_local: vec![],
                    }
                }
                (State::Directory{
                    ..//TODO
                }, State::File{
                    package,
                    ..//TODO
                }) =>
                {
                    let package = package.map(|p|sync!(p));
                    State::Package{
                        package,
                        global_decls: vec![],
                        package_local: vec![],
                    }
                }
                (State::Package{
                    package,
                    ..//TODO
                }, State::File{
                    package: p,
                    ..//TODO
                }) =>
                {
                    assert_eq!(package,p.map(|p|sync!(p)));
                    State::Package{
                        package,
                        global_decls: vec![],
                        package_local: vec![],
                    }
                }
                (State::None, State::Directory{
                    ..//TODO
                }) =>
                {
                    State::Directory{
                        global_decls: vec![],
                    }
                }
                (State::None, State::Package{
                    ..//TODO
                }) =>
                {
                    State::Directory{
                        global_decls: vec![],
                    }
                }
                (State::Directory{
                    ..//TODO
                }, State::Package{
                    ..//TODO
                }) =>
                {
                    State::Directory{
                        global_decls: vec![],
                    }
                }
                (State::Package{
                    ..//TODO
                }, State::Directory{
                    ..//TODO
                }) =>
                {
                    State::Directory{
                        global_decls: vec![],
                    }
                }
                (State::Package{
                    package,
                    ..//TODO
                }, State::Package{
                    ..//TODO
                }) =>
                {
                    State::Package{
                        package,
                        global_decls: vec![],
                        package_local: vec![],
                    }
                }
                (State::Directory{
                    ..//TODO
                }, State::Directory{
                    ..//TODO
                }) =>
                {
                    State::Directory{
                        global_decls: vec![],
                    }
                }
                (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
            }
        } else {
            // rest that is not easily categorized ie. used at multiple places
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }
            if kind == &Type::Annotation || kind == &Type::MarkerAnnotation {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleIdentifier(i))
                        if kind == &Type::MarkerAnnotation =>
                    {
                        let i = scoped!(mm!(), i);
                        // TODO handle annotations correctly
                        State::Annotation
                    }
                    (State::None, State::ScopedIdentifier(i))
                        if kind == &Type::MarkerAnnotation =>
                    {
                        let i = sync!(i);
                        // TODO handle annotations correctly
                        State::Annotation
                    }
                    (State::None, State::SimpleIdentifier(i)) if kind == &Type::Annotation => {
                        let i = scoped!(mm!(), i);
                        State::ScopedIdentifier(i)
                    }
                    (State::None, State::ScopedIdentifier(i)) if kind == &Type::Annotation => {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::SimpleIdentifier(i), State::Arguments(p))
                        if kind == &Type::Annotation =>
                    {
                        let i = scoped!(mm!(), i);
                        State::Annotation
                    }
                    (State::ScopedIdentifier(i), State::Arguments(p))
                        if kind == &Type::Annotation =>
                    {
                        State::Annotation
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::AnnotationArgumentList
                || kind == &Type::ElementValuePair
                || kind == &Type::ElementValueArrayInitializer
            {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (state, rest) if kind == &Type::AnnotationArgumentList => {
                        let mut v = match state {
                            State::Arguments(l) => l,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };

                        match rest {
                            State::ElementValuePair(p, i) => (),
                            State::LiteralType(_) => (),
                            State::FieldIdentifier(_) => (),
                            State::ScopedIdentifier(_) => (),
                            State::SimpleIdentifier(i) => {scoped!(mm!(),i);},
                            x => panic!("{:?}", x),
                        };

                        State::Arguments(v)
                    }
                    (State::None, State::SimpleIdentifier(i))
                        if kind == &Type::ElementValuePair =>
                    {
                        State::SimpleIdentifier(i)
                    }
                    (State::SimpleIdentifier(i), State::None)
                        if kind == &Type::ElementValuePair =>
                    {
                        let i = scoped!(mm!(), i);
                        State::ScopedIdentifier(i)
                    }
                    (State::SimpleIdentifier(p), expr) if kind == &Type::ElementValuePair => {
                        let t = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::SimpleIdentifier(t) => {
                                scoped!(mm!(), t)
                            }
                            State::ScopedIdentifier(t) => sync!(t),
                            State::FieldIdentifier(t) => sync!(t),
                            State::Invocation(t) => sync!(t),
                            State::ConstructorInvocation(t) => sync!(t),
                            x => panic!("{:?}", x),
                        };
                        State::ElementValuePair(p, t)
                    }
                    (rest, expr) if kind == &Type::ElementValueArrayInitializer => {
                        let i = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::SimpleIdentifier(t) => {
                                scoped!(mm!(), t)
                            }
                            State::ScopedIdentifier(t) => sync!(t),
                            State::FieldIdentifier(t) => sync!(t),
                            State::Invocation(t) => sync!(t),
                            State::ConstructorInvocation(t) => sync!(t),
                            x => panic!("{:?}", x),
                        };
                        match rest {
                            State::ScopedIdentifier(i) => State::ScopedIdentifier(i),
                            State::None => State::ScopedIdentifier(i),
                            _ => panic!(),
                        }
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ScopedIdentifier {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::ScopedIdentifier(i))
                        if kind == &Type::ScopedIdentifier =>
                    {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(o), State::SimpleIdentifier(i))
                        if kind == &Type::ScopedIdentifier =>
                    {
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(o, i));
                        State::ScopedIdentifier(i)
                    }
                    (State::None, State::SimpleIdentifier(i))
                        if kind == &Type::ScopedIdentifier =>
                    {
                        let o = acc.solver.intern(RefsEnum::Root);
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(o, i));
                        State::ScopedIdentifier(i)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ScopedTypeIdentifier {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleTypeIdentifier(i))
                        if kind == &Type::ScopedTypeIdentifier =>
                    {
                        let i = scoped!(mm!(), i);
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::None, State::ScopedTypeIdentifier(i))
                        if kind == &Type::ScopedTypeIdentifier =>
                    {
                        let i = sync!(i);
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::ScopedTypeIdentifier(o), State::SimpleTypeIdentifier(i))
                        if kind == &Type::ScopedTypeIdentifier =>
                    {
                        let i = scoped!(o, i);
                        State::ScopedTypeIdentifier(i)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::CatchType {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (rest, State::SimpleTypeIdentifier(i)) if kind == &Type::CatchType => {
                        let mut v = match rest {
                            State::CatchTypes(l) => l,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };
                        let i = scoped!(mm!(), i);
                        v.push(i);
                        State::CatchTypes(v)
                    }
                    (rest, State::ScopedTypeIdentifier(i)) if kind == &Type::CatchType => {
                        let mut v = match rest {
                            State::CatchTypes(l) => l,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };
                        let i = sync!(i);
                        v.push(i);
                        State::CatchTypes(v)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ArrayType {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleTypeIdentifier(i)) if kind == &Type::ArrayType => {
                        let i = scoped!(mm!(), i);
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::None, State::ScopedTypeIdentifier(i)) if kind == &Type::ArrayType => {
                        let i = sync!(i);
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::SimpleTypeIdentifier(i), State::Dimensions)
                        if kind == &Type::ArrayType =>
                    {
                        let i = scoped!(mm!(), i);
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::ScopedTypeIdentifier(i), State::Dimensions)
                        if kind == &Type::ArrayType =>
                    {
                        let i = acc.solver.intern(RefsEnum::Array(i));
                        State::ScopedTypeIdentifier(i)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ResourceSpecification || kind == &Type::FormalParameters {
                // TODO look like local decl
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (rest, State::ScopedIdentifier(i)) if kind == &Type::ResourceSpecification => {
                        rest
                    }
                    (
                        rest,
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: d,
                        },
                    ) if kind == &Type::FormalParameters
                        || kind == &Type::ResourceSpecification =>
                    {
                        // TODO do better than simple identifier
                        // TODO own State declaration (for parameters)
                        let mut v = match rest {
                            State::FormalParameters(l) => l,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };
                        let t = sync!(t);
                        let i = match d {
                            Declarator::Variable(i) => sync!(i),
                            _ => panic!(),
                        };
                        v.push((i, t));
                        State::FormalParameters(v)
                    }
                    (rest, State::None)
                        if kind == &Type::FormalParameters
                            || kind == &Type::ResourceSpecification =>
                    {
                        let mut v = match rest {
                            State::FormalParameters(l) => l,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };
                        State::FormalParameters(v)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::Wildcard
                || kind == &Type::WildcardSuper
                || kind == &Type::WildcardExtends
            {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, rest) if kind == &Type::Wildcard => {
                        match rest {
                            State::None => (),
                            State::WildcardExtends(_) => (),
                            State::WildcardSuper(_) => (),
                            x => panic!("{:?}", x),
                        }
                        // TODO solve correctly ie. DeclType::Runtime
                        let r = mm!();
                        State::ScopedTypeIdentifier(r)
                    }
                    (State::None, rest) if kind == &Type::WildcardExtends => {
                        let t = match rest {
                            State::SimpleTypeIdentifier(t) => scoped!(mm!(), t),
                            State::ScopedTypeIdentifier(t) => sync!(t),
                            x => panic!("{:?}", x),
                        };
                        State::WildcardExtends(t)
                    }
                    (State::None, rest) if kind == &Type::WildcardSuper => {
                        let t = match rest {
                            State::SimpleIdentifier(t) => scoped!(mm!(), t),
                            State::SimpleTypeIdentifier(t) => scoped!(mm!(), t),
                            State::ScopedTypeIdentifier(t) => sync!(t),
                            State::Super(t) => sync!(t),
                            x => panic!("{:?}", x),
                        };
                        State::WildcardSuper(t)
                    }
                    (State::WildcardSuper(i), State::SimpleTypeIdentifier(t))
                        if kind == &Type::WildcardSuper =>
                    {
                        let t = scoped!(mm!(), t);
                        State::WildcardSuper(i)
                    }
                    (State::WildcardSuper(i), State::ScopedTypeIdentifier(t))
                        if kind == &Type::WildcardSuper =>
                    {
                        State::WildcardSuper(i)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::SwitchLabel {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::LiteralType(_)) if kind == &Type::SwitchLabel => {
                        State::None
                    }
                    (State::None, State::FieldIdentifier(_)) if kind == &Type::SwitchLabel => {
                        State::None
                    }
                    (State::None, State::ScopedIdentifier(_)) if kind == &Type::SwitchLabel => {
                        State::None
                    }
                    (State::None, State::SimpleIdentifier(_)) if kind == &Type::SwitchLabel => {
                        State::None
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::Modifiers {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::Modifiers(v0, n0), State::Modifiers(v, n)) => State::Modifiers(
                        if v0 == Visibility::None {
                            v
                        } else {
                            assert_eq!(v, Visibility::None);
                            v0
                        },
                        n0.union(n),
                    ),

                    (State::None, State::Modifiers(v, n)) if kind == &Type::Modifiers => {
                        State::Modifiers(v, n)
                    }

                    (State::None, State::Annotation) if kind == &Type::Modifiers => {
                        State::Modifiers(Visibility::None, Default::default())
                    }

                    (State::Modifiers(v, n), State::Annotation) if kind == &Type::Modifiers => {
                        State::Modifiers(v, n)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ArgumentList {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (rest, State::MethodReference(i)) if kind == &Type::ArgumentList => {
                        let i = sync!(i);
                        let mut v = match rest {
                            State::Arguments(l) => l,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };
                        v.push(i);
                        State::Arguments(v)
                    }
                    (rest, State::LambdaExpression(i)) if kind == &Type::ArgumentList => {
                        let i = sync!(i);
                        let mut v = match rest {
                            State::Arguments(l) => l,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };
                        v.push(i);
                        State::Arguments(v)
                    }
                    (rest, expr) if kind == &Type::ArgumentList => {
                        // TODO do better than simple identifier
                        let mut v = match rest {
                            State::Arguments(l) => l,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };
                        let i = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::SimpleIdentifier(t) => scoped!(mm!(), t),
                            State::This(t) => sync!(t),
                            State::ScopedIdentifier(i) => sync!(i),
                            State::FieldIdentifier(i) => sync!(i),
                            State::Invocation(i) => sync!(i),
                            State::ConstructorInvocation(i) => sync!(i),
                            x => panic!("{:?}", x),
                        };
                        v.push(i);
                        State::Arguments(v)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::TypeArguments {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (rest, State::SimpleTypeIdentifier(t)) if kind == &Type::TypeArguments => {
                        let mut v = match rest {
                            State::TypeArguments(v) => v,
                            State::None => vec![],
                            _ => vec![],
                        };
                        let t = scoped!(mm!(), t);
                        v.push(t);
                        State::TypeArguments(v)
                    }
                    (rest, State::ScopedTypeIdentifier(i)) if kind == &Type::TypeArguments => {
                        let mut v = match rest {
                            State::TypeArguments(v) => v,
                            State::None => vec![],
                            _ => vec![],
                        };
                        let t = sync!(i);
                        v.push(t);
                        State::TypeArguments(v)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::InferredParameters {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (rest, State::SimpleIdentifier(i)) if kind == &Type::InferredParameters => {
                        let mut v = match rest {
                            State::Declarations(v) => v,
                            State::None => vec![],
                            _ => todo!(),
                        };
                        let r = mm!();
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                        let i = Declarator::Variable(i);
                        acc.solver.add_decl_simple(i.clone(), r);
                        v.push((i, r));
                        State::Declarations(v)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::TypeParameters {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (rest, State::SimpleIdentifier(i)) if kind == &Type::TypeParameters => {
                        let mut v = match rest {
                            State::TypeParameters(v) => v,
                            State::None => vec![],
                            _ => todo!(),
                        };
                        v.push((i, vec![].into_boxed_slice()));
                        State::TypeParameters(v)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ArrayInitializer {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    // ArrayInit
                    (State::None, expr) if kind == &Type::ArrayInitializer => {
                        match expr {
                            State::LiteralType(t) => (),
                            State::SimpleIdentifier(t) => {
                                scoped!(mm!(), t);
                            }
                            State::This(t) => (),
                            State::ScopedIdentifier(_) => (),
                            State::FieldIdentifier(_) => (),
                            State::Invocation(_) => (),
                            State::ConstructorInvocation(_) => (),
                            State::None => (), // TODO check
                            x => panic!("{:?}", x),
                        };
                        State::None
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::Throws {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (rest, State::SimpleTypeIdentifier(i)) if kind == &Type::Throws => {
                        let i = scoped!(mm!(), i);
                        State::Throws
                    }
                    (rest, State::ScopedTypeIdentifier(i)) if kind == &Type::Throws => {
                        State::Throws
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::VariableDeclarator {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleIdentifier(i))
                        if kind == &Type::VariableDeclarator =>
                    {
                        let r = mm!();
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                        State::Declarator(Declarator::Variable(i))
                    }
                    (State::Declarator(Declarator::Variable(v)), State::Dimensions)
                        if kind == &Type::VariableDeclarator =>
                    {
                        let v = acc.solver.intern(RefsEnum::Array(v));
                        State::Declarator(Declarator::Variable(v))
                    }
                    (State::Declarator(Declarator::Variable(v)), _)
                        if kind == &Type::VariableDeclarator =>
                    {
                        State::Declarator(Declarator::Variable(v))
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::DimensionsExpr {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, expr) if kind == &Type::DimensionsExpr => {
                        let i = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::SimpleIdentifier(t) => scoped!(mm!(), t),
                            State::ScopedIdentifier(i) => sync!(i),
                            State::FieldIdentifier(i) => sync!(i),
                            State::Invocation(i) => sync!(i),
                            State::ConstructorInvocation(i) => sync!(i),
                            x => panic!("{:?}", x),
                        };
                        State::ScopedIdentifier(i)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::GenericType {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleTypeIdentifier(t)) if kind == &Type::GenericType => {
                        State::SimpleTypeIdentifier(t)
                    }
                    (State::None, State::ScopedTypeIdentifier(t)) if kind == &Type::GenericType => {
                        let t = sync!(t);
                        State::ScopedTypeIdentifier(t)
                    }
                    (State::SimpleTypeIdentifier(t), State::TypeArguments(_))
                        if kind == &Type::GenericType =>
                    {
                        let t = scoped!(mm!(),t);
                        // TODO use arguments
                        State::ScopedTypeIdentifier(t)
                    }
                    (State::ScopedTypeIdentifier(t), State::TypeArguments(_))
                        if kind == &Type::GenericType =>
                    {
                        // TODO use arguments
                        State::ScopedTypeIdentifier(t)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::TypeBound {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (_, State::SimpleTypeIdentifier(t)) if kind == &Type::TypeBound => {
                        let t = scoped!(mm!(), t);
                        // TODO propag to use for solving refs
                        State::TypeBound
                    }
                    (_, State::ScopedTypeIdentifier(t)) if kind == &Type::TypeBound => {
                        // TODO propag to use for solving refs
                        State::TypeBound
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::Superclass {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleTypeIdentifier(i)) if kind == &Type::Superclass => {
                        let i = scoped!(mm!(), i);
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::None, State::ScopedTypeIdentifier(i)) if kind == &Type::Superclass => {
                        let i = sync!(i);
                        State::ScopedTypeIdentifier(i)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::SuperInterfaces || kind == &Type::ExtendsInterfaces {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (rest, State::ScopedTypeIdentifier(t))
                        if kind == &Type::SuperInterfaces || kind == &Type::ExtendsInterfaces =>
                    {
                        let mut v = match rest {
                            State::Interfaces(v) => v,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };
                        let t = sync!(t);
                        v.push(t);
                        State::Interfaces(v)
                    }
                    (rest, State::SimpleTypeIdentifier(t))
                        if kind == &Type::SuperInterfaces || kind == &Type::ExtendsInterfaces =>
                    {
                        let mut v = match rest {
                            State::Interfaces(v) => v,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };
                        let t = scoped!(mm!(), t);
                        v.push(t);
                        State::Interfaces(v)
                    }
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            } else {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::ScopedIdentifier(t))
                        if kind == &Type::ModuleDeclaration =>
                    {
                        State::None
                    },
                    (State::ScopedIdentifier(t), State::None)
                        if kind == &Type::ModuleDeclaration =>
                    {
                        State::None
                    },
                    (State::None, State::ScopedIdentifier(t))
                        if kind == &Type::ModuleDirective =>
                    {
                        State::None
                    },
                    (State::ScopedIdentifier(t), State::None)
                        if kind == &Type::ModuleDirective =>
                    {
                        State::None
                    },
                    (State::None, State::None)
                        if kind == &Type::ModuleBody =>
                    {
                        State::None
                    },
                    (State::None, State::None)
                        if kind == &Type::ModuleDeclaration =>
                    {
                        State::None
                    },
                    (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
                }
            }
        };

        println!("result for {:?} is {:?}", kind, acc.current_node);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Visibility {
    Public,
    Protected,
    Private,
    None,
}

#[derive(EnumSetType, Debug)]
pub enum NonVisibility {
    Static,
    Final,
    Abstract,
    Synchronized,
    Transient,
    Strictfp,
    Native,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum State<Node = LabelValue, Leaf = LabelValue>
where
    Leaf: std::cmp::Eq + std::hash::Hash,
    Node: std::cmp::Eq + std::hash::Hash,
{
    Todo,
    None,
    Asterisk,
    Super(Node),
    This(Node),
    Condition,
    Dimensions,
    Throws,
    Root,
    Annotation,
    Modifiers(Visibility, EnumSet<NonVisibility>),
    /// a
    SimpleIdentifier(Leaf),
    /// A or A.B
    SimpleTypeIdentifier(Leaf),
    ScopedTypeIdentifier(Node),
    WildcardExtends(Node),
    WildcardSuper(Node),
    TypeBound,
    TypeParameters(Vec<(Leaf, Box<[Node]>)>),
    GenericType(Node),
    CatchTypes(Vec<Node>),
    CatchParameter {
        kinds: Box<[Node]>,
        identifier: Leaf,
    },
    LiteralType(Node),
    ScopedIdentifier(Node),
    PackageDeclaration(Node),
    File {
        package: Option<Node>,
        asterisk_imports: Vec<Node>,
        top_level: Option<(Declarator<Node>, Node)>,
        content: Vec<(Declarator<Node>, Node)>,
    },
    Directory {
        global_decls: Vec<(Declarator<Node>, Node)>,
    },
    Package {
        package: Option<Node>,
        global_decls: Vec<(Declarator<Node>, Node)>,
        package_local: Vec<(Declarator<Node>, Node)>,
    },
    /// b.f() or A.f()
    Invocation(Node),
    InvocationId(Node, Leaf),
    MethodReference(Node),
    LambdaExpression(Node),
    TypeArguments(Vec<Node>),
    Arguments(Vec<Node>),
    /// A#constructor()
    ConstructorInvocation(Node),
    ImportDeclaration {
        sstatic: bool,
        identifier: Node,
        asterisk: bool,
    },
    /// a.b
    FieldIdentifier(Node),
    Interfaces(Vec<Node>),
    ElementValuePair(Leaf, Node),
    Declarator(Declarator<Node>),
    Declaration {
        visibility: Visibility,
        kind: Node,
        identifier: Declarator<Node>,
    },
    MethodImplementation {
        visibility: Visibility,
        kind: Option<Node>,
        identifier: Option<Leaf>,
        parameters: Box<[(Node, Node)]>,
    },
    ConstructorImplementation {
        visibility: Visibility,
        identifier: Option<Leaf>,
        parameters: Box<[(Node, Node)]>,
    },
    TypeDeclaration {
        visibility: Visibility,
        identifier: DeclType<Node>,
        members: Vec<(Declarator<Node>, Node)>,
    },
    Declarations(Vec<(Declarator<Node>, Node)>),
    FormalParameters(Vec<(Node, Node)>),

    ///TODO use this to make further flow type static analysis, most of the time replace None
    TypeOfValue(Box<[Leaf]>),
}
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Argument<Node = LabelValue>
where
    Node: Eq + Hash,
{
    Type(Node),
    Identifier(Node),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Declarator<Node = LabelValue>
where
    Node: Eq + Hash,
{
    None,
    Package(Node),
    Type(Node),
    Field(Node),
    Executable(Node),
    Variable(Node),
}

impl<Node> Declarator<Node>
where
    Node: Eq + Hash,
{
    fn node(&self) -> Option<&Node> {
        match self {
            Declarator::None => None,
            Declarator::Package(k) => Some(k),
            Declarator::Type(k) => Some(k),
            Declarator::Field(k) => Some(k),
            Declarator::Executable(k) => Some(k),
            Declarator::Variable(k) => Some(k),
            // Declarator::Parameter(k) => Some(k),
        }
    }

    fn with_changed_node<N, F: FnOnce(&Node) -> N>(&self, f: F) -> Declarator<N>
    where
        N: Eq + Hash,
    {
        match self {
            Declarator::None => Declarator::None,
            Declarator::Package(i) => Declarator::Package(f(i)),
            Declarator::Type(i) => Declarator::Type(f(i)),
            Declarator::Field(i) => Declarator::Field(f(i)),
            Declarator::Executable(i) => Declarator::Executable(f(i)),
            Declarator::Variable(i) => Declarator::Variable(f(i)),
            // Declarator::Parameter(i) => Declarator::Parameter(f(i)),
        }
    }
}

impl<Node, Leaf> State<Node, Leaf>
where
    Leaf: std::cmp::Eq + std::hash::Hash + Copy,
    Node: std::cmp::Eq + std::hash::Hash + Copy,
{
    pub fn take(&mut self) -> Self {
        std::mem::replace(self, State::None)
    }
    pub fn map<N, L, FN: Fn(Node) -> N, FL: Fn(Leaf) -> L>(&self, f: FN, g: FL) -> State<N, L>
    where
        L: std::cmp::Eq + std::hash::Hash,
        N: std::cmp::Eq + std::hash::Hash,
    {
        match self {
            State::Todo => State::Todo,
            State::None => State::None,
            State::Asterisk => State::Asterisk,
            State::Condition => State::Condition,
            State::Dimensions => State::Dimensions,
            State::Throws => State::Throws,
            State::Root => State::Root,
            State::Annotation => State::Annotation,
            State::TypeBound => State::TypeBound,
            State::SimpleIdentifier(l) => State::SimpleIdentifier(g(*l)),
            State::SimpleTypeIdentifier(l) => State::SimpleTypeIdentifier(g(*l)),

            State::Super(i) => State::Super(f(*i)),
            State::This(i) => State::This(f(*i)),
            State::ScopedTypeIdentifier(i) => State::ScopedTypeIdentifier(f(*i)),
            State::WildcardExtends(i) => State::WildcardExtends(f(*i)),
            State::WildcardSuper(i) => State::WildcardSuper(f(*i)),
            State::GenericType(i) => State::GenericType(f(*i)),
            State::LiteralType(i) => State::LiteralType(f(*i)),
            State::ScopedIdentifier(i) => State::ScopedIdentifier(f(*i)),
            State::PackageDeclaration(i) => State::PackageDeclaration(f(*i)),
            State::Invocation(i) => State::Invocation(f(*i)),
            State::MethodReference(i) => State::MethodReference(f(*i)),
            State::LambdaExpression(i) => State::LambdaExpression(f(*i)),
            State::ConstructorInvocation(i) => State::ConstructorInvocation(f(*i)),
            State::FieldIdentifier(i) => State::FieldIdentifier(f(*i)),
            State::Declarator(d) => State::Declarator(d.with_changed_node(|x| f(*x))),
            State::Interfaces(v) => State::Interfaces(v.iter().map(|x| f(*x)).collect()),
            State::Arguments(v) => State::Arguments(v.iter().map(|x| f(*x)).collect()),
            State::TypeArguments(v) => State::TypeArguments(v.iter().map(|x| f(*x)).collect()),
            State::CatchTypes(v) => State::CatchTypes(v.iter().map(|x| f(*x)).collect()),
            State::TypeParameters(v) => State::TypeParameters(
                v.iter()
                    .map(|(x, y)| ((g(*x), y.iter().map(|x| f(*x)).collect())))
                    .collect(),
            ),
            State::Declarations(v) => State::Declarations(
                v.iter()
                    .map(|(x, y)| (x.with_changed_node(|x| f(*x)), f(*y)))
                    .collect(),
            ),
            State::FormalParameters(v) => {
                State::FormalParameters(v.iter().map(|(x, y)| (f(*x), f(*y))).collect())
            }
            State::TypeOfValue(_) => todo!(),
            State::ElementValuePair(x, y) => State::ElementValuePair(g(*x), f(*y)),
            State::InvocationId(x, y) => State::InvocationId(f(*x), g(*y)),
            State::Modifiers(x, y) => State::Modifiers(x.clone(), y.clone()),
            State::ImportDeclaration {
                sstatic,
                identifier: i,
                asterisk,
            } => State::ImportDeclaration {
                sstatic: *sstatic,
                identifier: f(*i),
                asterisk: *asterisk,
            },
            State::CatchParameter {
                kinds: v,
                identifier: i,
            } => State::CatchParameter {
                kinds: v.iter().map(|x| f(*x)).collect(),
                identifier: g(*i),
            },

            State::File {
                package: p,
                asterisk_imports,
                top_level: t,
                content: v,
            } => State::File {
                package: p.map(|x| f(x)),
                asterisk_imports: asterisk_imports.iter().map(|x| f(*x)).collect(),
                top_level: t
                    .clone()
                    .map(|(x, y)| (x.with_changed_node(|x| f(*x)), f(y))),
                content: v
                    .iter()
                    .map(|(x, y)| (x.with_changed_node(|x| f(*x)), f(*y)))
                    .collect(),
            },

            State::Package {
                package: p,
                global_decls,
                package_local,
            } => State::Package {
                package: p.map(|x| f(x)),
                global_decls: global_decls
                    .iter()
                    .map(|(x, y)| (x.with_changed_node(|x| f(*x)), f(*y)))
                    .collect(),
                package_local: package_local
                    .iter()
                    .map(|(x, y)| (x.with_changed_node(|x| f(*x)), f(*y)))
                    .collect(),
            },
            State::Directory { global_decls } => State::Directory {
                global_decls: global_decls
                    .iter()
                    .map(|(x, y)| (x.with_changed_node(|x| f(*x)), f(*y)))
                    .collect(),
            },
            State::Declaration {
                visibility,
                kind: t,
                identifier: d,
            } => State::Declaration {
                visibility: visibility.clone(),
                kind: f(*t),
                identifier: d.with_changed_node(|x| f(*x)),
            },
            State::MethodImplementation {
                visibility,
                kind: t,
                identifier: i,
                parameters: p,
            } => State::MethodImplementation {
                visibility: visibility.clone(),
                kind: t.map(|x| f(x)),
                identifier: i.map(|x| g(x)),
                parameters: p.iter().map(|(x, y)| (f(*x), f(*y))).collect(),
            },
            State::ConstructorImplementation {
                visibility,
                identifier: i,
                parameters: p,
            } => State::ConstructorImplementation {
                visibility: visibility.clone(),
                identifier: i.map(|x| g(x)),
                parameters: p.iter().map(|(x, y)| (f(*x), f(*y))).collect(),
            },
            State::TypeDeclaration {
                visibility,
                identifier: d,
                members: v,
            } => State::TypeDeclaration {
                visibility: visibility.clone(),
                identifier: d.map(|x| f(x)),
                members: v
                    .iter()
                    .map(|(x, y)| (x.with_changed_node(|x| f(*x)), f(*y)))
                    .collect(),
            },
        }
    }
}

fn default_imports<F: FnMut(&str) -> LabelPtr>(solver: &mut Solver, mut intern_label: F) {
    macro_rules! scoped {
        ( $o:expr, $i:expr ) => {{
            let o = $o;
            let i = $i;
            solver.intern(RefsEnum::ScopedIdentifier(o, i))
        }};
    }
    macro_rules! import {
        ( $($p:expr),* ) => {
            {
                let t = solver.intern(RefsEnum::Root);
                $(
                    let i = intern_label($p);
                    let t = scoped!(t, i);
                )*
                let i = scoped!(solver.intern(RefsEnum::MaybeMissing), i);
                let d = Declarator::Type(i);
                solver.add_decl_simple(d, t);
            }
        }
    }
    // import!("java","lang","Appendable");
    // import!("java","lang","AutoCloseable");
    // import!("java","lang","CharSequence");
    // import!("java","lang","Cloneable");
    // import!("java","lang","Comparable");//<T>
    // import!("java","lang","Iterable");//<T>
    // import!("java","lang","Readable");
    // import!("java","lang","Runnable");
    // import!("java","lang","Thread","UncaughtExceptionHandler");
    // import!("java","lang","Byte");
    // import!("java","lang","Character");
    // import!("java","lang","Character","Subset");
    // import!("java","lang","Character","UnicodeBlock");
    // import!("java","lang","Class");//<T>
    // import!("java","lang","ClassLoader");
    // import!("java","lang","ClassValue");//<T>
    // import!("java","lang","Compiler");
    // import!("java","lang","Double");
    // import!("java","lang","Enum"); //<E extends Enum<E>>
    // import!("java","lang","Float");
    // import!("java","lang","InheritableThreadLocal");//<T>
    // import!("java", "lang", "Integer");
    // import!("java","lang","Long");
    // import!("java","lang","Math");
    // import!("java","lang","Number");
    // import!("java","lang","Object");
    // import!("java","lang","Package");
    // import!("java","lang","Process");
    // import!("java","lang","ProcessBuilder");
    // import!("java","lang","ProcessBuilder","Redirect");
    // import!("java","lang","Runtime");
    // import!("java","lang","RuntimePermission");
    // import!("java","lang","SecurityManager");
    // import!("java","lang","Short");
    // import!("java","lang","StackTraceElement");
    // import!("java","lang","StrictMath");
    // import!("java", "lang", "String");
    // import!("java","lang","StringBuffer");
    // import!("java","lang","StringBuilder");
    // import!("java","lang","System");
    // import!("java","lang","Thread");
    // import!("java","lang","ThreadGroup");
    // import!("java","lang","ThreadLocal");//<T>
    // import!("java","lang","Throwable");
    // import!("java","lang","Void");
    // import!("java","lang","ProcessBuilder","Redirect","Type");
    // import!("java","lang","Thread","State");
    // import!("java","lang","ArrayIndexOutOfBoundsException");
    // import!("java","lang","ArrayStoreException");
    // import!("java","lang","ClassCastException");
    // import!("java","lang","ClassNotFoundException");
    // import!("java","lang","CloneNotSupportedException");
    // import!("java","lang","EnumConstantNotPresentException");
    // import!("java","lang","Exception");
    // import!("java","lang","IllegalAccessException");
    // import!("java","lang","IllegalArgumentException");
    // import!("java","lang","IllegalMonitorStateException");
    // import!("java","lang","IllegalStateException");
    // import!("java","lang","IllegalThreadStateException");
    // import!("java","lang","IndexOutOfBoundsException");
    // import!("java","lang","InstantiationException");
    // import!("java","lang","InterruptedException");
    // import!("java","lang","NegativeArraySizeException");
    // import!("java","lang","NoSuchFieldException");
    // import!("java","lang","NoSuchMethodException");
    // import!("java","lang","NullPointerException");
    // import!("java","lang","NumberFormatException");
    // import!("java","lang","ReflectiveOperationException");
    // import!("java","lang","RuntimeException");
    // import!("java","lang","SecurityException");
    // import!("java","lang","StringIndexOutOfBoundsException");
    // import!("java","lang","TypeNotPresentException");
    // import!("java","lang","UnsupportedOperationException");
    // import!("java","lang","AssertionError");
    // import!("java","lang","BootstrapMethodError");
    // import!("java","lang","ClassCircularityError");
    // import!("java","lang","ClassFormatError");
    // import!("java","lang","Error");
    // import!("java","lang","ExceptionInInitializerError");
    // import!("java","lang","IllegalAccessError");
    // import!("java","lang","IncompatibleClassChangeError");
    // import!("java","lang","InstantiationError");
    // import!("java","lang","InternalError");
    // import!("java","lang","LinkageError");
    // import!("java","lang","NoClassDefFoundError");
    // import!("java","lang","NoSuchFieldError");
    // import!("java","lang","NoSuchMethodError");
    // import!("java","lang","OutOfMemoryError");
    // import!("java","lang","StackOverflowError");
    // import!("java","lang","ThreadDeath");
    // import!("java","lang","UnknownError");
    // import!("java","lang","UnsatisfiedLinkError");
    // import!("java","lang","UnsupportedClassVersionError");
    // import!("java","lang","VerifyError");
    // import!("java","lang","VirtualMachineError");
    // import!("java","lang","Override");
    // import!("java","lang","SafeVarargs");
    // import!("java","lang","SuppressWarnings");
}

trait RefMap {
    fn get<Q: Borrow<RefPtr>>(&self, k: Q) -> Option<&RefPtr>;
    fn insert(&mut self, k: RefPtr, v: RefPtr);
}

struct RefHashMap {
    map: HashMap<RefPtr, RefPtr>,
}

impl RefMap for RefHashMap {
    fn get<Q: Borrow<RefPtr>>(&self, k: Q) -> Option<&RefPtr> {
        self.map.get(k.borrow())
    }

    fn insert(&mut self, k: RefPtr, v: RefPtr) {
        self.map.insert(k, v);
    }
}

enum Simplify<T> {
    Continue(T),
    Terminate(T),
}

trait RefMapSimp {
    fn get<Q: Borrow<RefPtr>>(&self, k: Q) -> Option<Simplify<&RefPtr>>;
    fn insert(&mut self, k: RefPtr, v: Simplify<RefPtr>);
}

struct RefHashMapSimp {
    map: HashMap<RefPtr, RefPtr>,
    terminate: bitvec::vec::BitVec,
}

impl RefMapSimp for RefHashMapSimp {
    fn get<Q: Borrow<RefPtr>>(&self, k: Q) -> Option<Simplify<&RefPtr>> {
        let r = self.map.get(k.borrow());
        if let Some(r) = r {
            let r = if self.terminate.len() > *k.borrow() && self.terminate[*k.borrow()] {
                Simplify::Terminate(r)
            } else {
                Simplify::Continue(r)
            };
            Some(r)
        } else {
            None
        }
    }

    fn insert(&mut self, k: RefPtr, v: Simplify<RefPtr>) {
        let v = match v {
            Simplify::Continue(v) => v,
            Simplify::Terminate(v) => {
                if !(self.terminate.len() > k) {
                    self.terminate.resize(k + 1, false)
                }
                self.terminate.set(k, true);
                v
            }
        };
        self.map.insert(k, v);
    }
}
