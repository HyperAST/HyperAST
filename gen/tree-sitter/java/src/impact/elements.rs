use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Deref, Index},
};

use bitvec::order::Lsb0;
use enumset::{enum_set, EnumSet, EnumSetType};
use rusted_gumtree_core::tree::tree::{LabelStore, Type};
use string_interner::{DefaultSymbol, StringInterner, Symbol};

use crate::nodes::LabelIdentifier;

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

struct ExplorableRef<'a> {
    rf: RefPtr,
    nodes: &'a Nodes,
}

impl<'a> AsRef<RefsEnum<RefPtr, LabelPtr>> for ExplorableRef<'a> {
    fn as_ref(&self) -> &RefsEnum<RefPtr, LabelPtr> {
        &self.nodes[self.rf]
    }
}

impl<'a> ExplorableRef<'a> {
    fn with(&self, rf: RefPtr) -> Self {
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
                out.extend(i.to_string().as_bytes())},
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
                    Arguments::Unknown => out.push(b"."[0]),
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
                    Arguments::Unknown => out.push(0),
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
            RefsEnum::Primitive(i) => Display::fmt(i,f),
            RefsEnum::ScopedIdentifier(o, i) => {
                write!(f, "{}", self.with(*o))?;
                write!(f, ".{}", self.leafs.resolve(i))
            }
            RefsEnum::MethodReference(o, i) => {
                write!(f, "{}", self.with(*o))?;
                write!(f, "::{}", self.leafs.resolve(i))
            }
            RefsEnum::ConstructorReference(o) => {
                write!(f, "{}", self.with(*o))?;
                write!(f, "::new")
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
    MaybeMissing,
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

impl Primitive {
    fn new(s:&Type) -> Self {
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
            Type::StringLiteral => panic!("{:?}",s),
            Type::CharacterLiteral => Self::Char,
            Type::NullLiteral => Self::Null,
            _ => panic!("{:?}",s),
        }
    }
}

impl Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
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
        })
    }
}

trait SubTyping : PartialOrd {
}

impl PartialOrd for Primitive {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        let r = match (self,other) {
            (x,y) if x==y => Some(Ordering::Equal),
            (Primitive::Double,Primitive::Double) => Some(Ordering::Equal),
            // double >1 float
            (Primitive::Double,Primitive::Float) => Some(Ordering::Greater),
            (Primitive::Float,Primitive::Float) => Some(Ordering::Equal),
            // float >1 long
            (Primitive::Float,Primitive::Long) => Some(Ordering::Greater),
            (Primitive::Long,Primitive::Long) => Some(Ordering::Equal),
            // long >1 int
            (Primitive::Long,Primitive::Int) => Some(Ordering::Greater),
            (Primitive::Int,Primitive::Int) => Some(Ordering::Equal),
            // int >1 char
            (Primitive::Int,Primitive::Char) => Some(Ordering::Greater),
            // int >1 short
            (Primitive::Int,Primitive::Short) => Some(Ordering::Greater),
            (Primitive::Char,Primitive::Char) => Some(Ordering::Equal),
            (Primitive::Short,Primitive::Short) => Some(Ordering::Equal),
            // short >1 byte
            (Primitive::Short,Primitive::Byte) => Some(Ordering::Greater),
            (Primitive::Byte,Primitive::Byte) => Some(Ordering::Equal),
            (Primitive::Boolean,Primitive::Boolean) => Some(Ordering::Equal),
            (Primitive::Null,Primitive::Null) => Some(Ordering::Equal),
            (Primitive::Void,Primitive::Void) => Some(Ordering::Equal),
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
            (RefsEnum::ScopedIdentifier(_, i), RefsEnum::ScopedIdentifier(_, j)) => i == j,
            (RefsEnum::MethodReference(_, i), RefsEnum::MethodReference(_, j)) => i == j,
            (RefsEnum::ConstructorReference(i), RefsEnum::ConstructorReference(j)) => i == j,
            (RefsEnum::Invocation(_, i, _), RefsEnum::Invocation(_, j, _)) => i == j,
            (RefsEnum::ConstructorInvocation(_, _), RefsEnum::ConstructorInvocation(_, _)) => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Arguments<Node = LabelValue>
where
    Node: Eq + Hash,
{
    Unknown,
    Given(Box<[Node]>),
}

pub fn leaf_state(t: &Type, label: Option<LabelPtr>) -> State<RefPtr, LabelPtr> {
    let r = if t == &Type::Comment {
        State::None
    } else if t.is_primitive() {
        // State::SimpleTypeIdentifier(label.unwrap())
        panic!("{:?} {:?}",t,label);
    } else if t.is_literal() {
        // State::LiteralType(label.unwrap())
        panic!("{:?} {:?}",t,label);
    } else if t == &Type::ScopedIdentifier {
        panic!();
    } else if t == &Type::ScopedTypeIdentifier {
        panic!();
    } else if t == &Type::ArgumentList {
        State::Arguments(vec![])
    } else if t == &Type::FormalParameters {
        State::FormalParameters(vec![])
    } else if t == &Type::Super {
        State::Super(label.unwrap())
    } else if t == &Type::This {//t.is_instance_ref() {
        State::This(label.unwrap())
    } else if t == &Type::TypeIdentifier {
        State::SimpleTypeIdentifier(label.unwrap())
    } else if t.is_identifier() {
        State::SimpleIdentifier(label.unwrap())
    } else if t == &Type::Spaces {
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
    println!("init: {:?} {:?}", t, r);
    r
}

#[derive(Debug)]
pub struct PartialAnalysis {
    current_node: State<RefPtr, LabelPtr>,
    solver: Solver,
}

#[derive(Debug, Clone)]
struct Nodes(Vec<RefsEnum<RefPtr, LabelPtr>>);

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
    nodes: Nodes,
    refs: bitvec::vec::BitVec,
    decls: HashMap<Declarator<RefPtr>, DeclType<RefPtr>>,
}

impl Default for Solver {
    fn default() -> Self {
        Self {
            // leafs: Default::default(),
            nodes: Nodes(vec![RefsEnum::Root, RefsEnum::MaybeMissing]),
            refs: Default::default(),
            decls: Default::default(),
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
impl Solver {
    fn intern(&mut self, other: RefsEnum<RefPtr, LabelPtr>) -> RefPtr {
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

    fn intern_ref(&mut self, other: RefsEnum<RefPtr, LabelPtr>) -> RefPtr {
        let r = self.intern(other);
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
        match self.nodes[t].clone() {
            RefsEnum::Root => panic!("fully qualified node cannot be qualified further"),
            RefsEnum::MaybeMissing => p,
            RefsEnum::ScopedIdentifier(i, y) => {
                let x = self.solve_node_with(i, p);
                let tmp = RefsEnum::ScopedIdentifier(x, y);
                if t < self.refs.len() && self.refs[t] {
                    self.intern_ref(tmp)
                } else {
                    self.intern(tmp)
                }
            }
            RefsEnum::Invocation(o,i, args) => {
                let x = self.solve_node_with(o, p);
                let tmp = RefsEnum::Invocation(x,i, args);
                if t < self.refs.len() && self.refs[t] {
                    self.intern_ref(tmp)
                } else {
                    self.intern(tmp)
                }

            },
            RefsEnum::ConstructorInvocation(o, args) => {
                let x = self.solve_node_with(o, p);
                let tmp = RefsEnum::ConstructorInvocation(x, args);
                if t < self.refs.len() && self.refs[t] {
                    self.intern_ref(tmp)
                } else {
                    self.intern(tmp)
                }

            },
            x => todo!("not sure how {:?} should be handled",x),
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
        println!("other: {:?}", other);
        let r = match other.as_ref() {
            RefsEnum::Root => self.intern(RefsEnum::Root),
            RefsEnum::MaybeMissing => self.intern(RefsEnum::MaybeMissing),
            RefsEnum::Primitive(i) => self.intern(RefsEnum::Primitive(*i)),
            RefsEnum::ScopedIdentifier(o, i) => {
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
                assert_ne!(i,r);
                r
            }
        };
        let r = match self.decls.get(&Declarator::Variable(r)) {
            Some(DeclType::Runtime(b)) => {
                panic!()
            }
            Some(DeclType::Compile(r, s, i)) => {
                println!("solved local variable: {:?}", r);
                // self.solve_intern_external(cache, other.with(r))
                *r
            }
            None => r,
        };
        // TODO handle class statements
        cache.insert(other.rf, r);
        r
    }

    /// no internalization needed
    /// not used on blocks, only bodies, declarations and whole programs
    fn solve_aux(
        &mut self,
        cache: &mut HashMap<RefPtr, Option<RefPtr>>,
        other: RefPtr,
    ) -> Option<RefPtr> {
        if let Some(x) = cache.get(&other) {
            return *x;
        }
        println!("other: {:?}", other);
        let r = match self.nodes[other].clone() {
            RefsEnum::Root => Some(other),
            RefsEnum::MaybeMissing => Some(other),
            RefsEnum::Primitive(i) => Some(self.intern(RefsEnum::Primitive(i))),
            RefsEnum::ScopedIdentifier(o, i) => {
                let r = if let Some(o) = self.solve_aux(cache, o) {
                    Some(self.intern(RefsEnum::ScopedIdentifier(o, i)))
                } else {
                    None
                };
                let r = if let Some(r) = r {
                    r
                } else {
                    cache.insert(other, r);
                    return None;
                };
                let r = if let Some(r) = (&self.decls).get(&Declarator::Variable(r)).cloned() {
                    // TODO should not need
                    match r {
                        DeclType::Compile(r, _, _) => {
                            println!("solved local variable: {:?}", r);
                            self.solve_aux(cache, r)
                        }
                        _ => todo!(),
                    }
                } else if let Some(r) = (&self.decls).get(&Declarator::Field(r)).cloned() {
                    match r {
                        DeclType::Compile(r, _, _) => {
                            println!("solved field: {:?}", r);
                            self.solve_aux(cache, r)
                        }
                        _ => todo!(),
                    }
                } else if let Some(r) = (&self.decls).get(&Declarator::Type(r)).cloned() {
                    println!("solved class: {:?}", r);
                    // None // TODO not 100% sure Some of None
                    match r {
                        DeclType::Compile(r, _, _) => {
                            println!("solved class: {:?}", r);
                            Some(r)
                        }
                        DeclType::Runtime(b) => {
                            println!("solved runtime: {:?}", b);
                            None
                        }
                        x => todo!("{:?}",x),
                    }
                } else if r != other {
                    self.solve_aux(cache, r)
                } else {
                    Some(r)
                };

                r
            }
            RefsEnum::MethodReference(o, i) => {
                let r = if let Some(o) = self.solve_aux(cache, o) {
                    Some(self.intern(RefsEnum::MethodReference(o, i)))
                } else {
                    None
                };
                let r = if let Some(r) = r {
                    r
                } else {
                    cache.insert(other, r);
                    return None;
                };
                let r = if let Some(r) = (&self.decls).get(&Declarator::Executable(r)).cloned() {
                    match r {
                        DeclType::Compile(r, _, _) => {
                            println!("solved method ref: {:?}", r);
                            self.solve_aux(cache, r)
                        }
                        _ => todo!(),
                    }
                } else if r != other {
                    self.solve_aux(cache, r)
                } else {
                    Some(r)
                };
                r
            }
            RefsEnum::ConstructorReference(o) => {
                let r = if let Some(o) = self.solve_aux(cache, o) {
                    Some(self.intern(RefsEnum::ConstructorReference(o)))
                } else {
                    None
                };
                let r = if let Some(r) = r {
                    r
                } else {
                    cache.insert(other, r);
                    return None;
                };
                let r = if let Some(r) = (&self.decls).get(&Declarator::Executable(r)).cloned() {
                    match r {
                        DeclType::Compile(r, _, _) => {
                            println!("solved constructor ref: {:?}", r);
                            self.solve_aux(cache, r)
                        }
                        _ => todo!(),
                    }
                } else if r != other {
                    self.solve_aux(cache, r)
                } else {
                    Some(r)
                };
                r
            }
            RefsEnum::Invocation(o, i, p) => {
                let r = if let Some(o) = self.solve_aux(cache, o) {
                    let mut b = false;
                    let p = match p {
                        Arguments::Unknown => Arguments::Unknown,
                        Arguments::Given(p) => {
                            b = false;
                            let mut v = vec![];
                            for x in p.deref() {
                                if let Some(r) = self.solve_aux(cache, *x) {
                                    b = true;
                                    v.push(r);
                                }
                            }
                            if v.is_empty() {
                                b = true
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
                } else {
                    None
                };

                let r = if let Some(r) = r {
                    r
                } else {
                    cache.insert(other, r);
                    return None;
                };
                let r = if let Some(r) = (&self.decls).get(&Declarator::Executable(r)).cloned() {
                    match r {
                        DeclType::Compile(r, _, _) => {
                            println!("solved method: {:?}", r);
                            self.solve_aux(cache, r)
                        }
                        _ => todo!(),
                    }
                } else if r != other {
                    self.solve_aux(cache, r)
                } else {
                    Some(r)
                };
                r
            }
            RefsEnum::ConstructorInvocation(o, p) => {
                let r = if let Some(i) = self.solve_aux(cache, o) {
                    let mut b = true;
                    let p = match p {
                        Arguments::Unknown => Arguments::Unknown,
                        Arguments::Given(p) => {
                            b = false;
                            let mut v = vec![];
                            for x in p.deref() {
                                if let Some(r) = self.solve_aux(cache, *x) {
                                    b = true;
                                    v.push(r);
                                }
                            }
                            if v.is_empty() {
                                b = true
                            }
                            let p = v.into_boxed_slice();
                            Arguments::Given(p)
                        }
                    };
                    if b {
                        let r = self.intern(RefsEnum::ConstructorInvocation(i, p));
                        assert_ne!(r,i);
                        Some(r)
                    } else {
                        None
                    }
                } else {
                    None
                };

                let r = if let Some(r) = r {
                    r
                } else {
                    cache.insert(other, r);
                    return None;
                };
                let r = if let Some(r) = (&self.decls).get(&Declarator::Executable(r)).cloned() {
                    match r {
                        DeclType::Compile(r, s, i) => {
                            println!("solved constructor: {:?} {:?} {:?}", r, s, i);
                            self.solve_aux(cache, r)
                        }
                        _ => todo!(),
                    }
                } else if r != other {
                    self.solve_aux(cache, r)
                } else {
                    Some(r)
                };
                r
            }
        };
        if r != Some(other) {
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
                    println!("   {:?}: {:?}", k, kr);
                    for v in b.iter() {
                        let r = ExplorableRef {
                            rf: *v,
                            nodes: &self.nodes,
                        };
                        print!(" {:?}", r);
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
        cache: &mut HashMap<RefPtr, RefPtr>,
        other: ExplorableRef,
    ) -> RefPtr {
        if let Some(x) = cache.get(&other.rf) {
            assert!(
                self.nodes[*x].similar(other.as_ref()),
                "{:?} ~ {:?}",
                ExplorableRef {
                    nodes: &self.nodes,
                    rf: *x
                },
                other
            );
            return *x;
        }
        let r = match other.as_ref() {
            RefsEnum::Root => self.intern(RefsEnum::Root),
            RefsEnum::MaybeMissing => self.intern(RefsEnum::MaybeMissing),
            RefsEnum::Primitive(i) => self.intern(RefsEnum::Primitive(*i)),
            RefsEnum::ScopedIdentifier(o, i) => {
                let tmp = o;
                let o = self.intern_external(cache, other.with(*o));
                assert!(self.nodes[o].similar(other.with(*tmp).as_ref()));
                self.intern(RefsEnum::ScopedIdentifier(o, *i))
            }
            RefsEnum::MethodReference(o, i) => {
                let tmp = o;
                let o = self.intern_external(cache, other.with(*o));
                assert!(self.nodes[o].similar(other.with(*tmp).as_ref()));
                println!("{:?}", o);
                println!("{:?}", self.nodes);
                self.intern(RefsEnum::MethodReference(o, *i))
            }
            RefsEnum::ConstructorReference(o) => {
                let tmp = o;
                let o = self.intern_external(cache, other.with(*o));
                assert!(self.nodes[o].similar(other.with(*tmp).as_ref()));
                println!("{:?}", o);
                println!("{:?}", self.nodes);
                self.intern(RefsEnum::ConstructorReference(o))
            }
            RefsEnum::Invocation(o, i, p) => {
                let o = self.intern_external(cache, other.with(*o));
                let p = match p {
                    Arguments::Unknown => Arguments::Unknown,
                    Arguments::Given(p) => {
                        let mut v = vec![];
                        for x in p.deref() {
                            let r = self.intern_external(cache, other.with(*x));
                            assert!(self.nodes[r].similar(other.with(*x).as_ref()));
                            v.push(r);
                        }
                        let p = v.into_boxed_slice();
                        Arguments::Given(p)
                    }
                };
                self.intern(RefsEnum::Invocation(o, *i, p))
            }
            RefsEnum::ConstructorInvocation(o, p) => {
                let i = self.intern_external(cache, other.with(*o));
                let p = match p {
                    Arguments::Unknown => Arguments::Unknown,
                    Arguments::Given(p) => {
                        let p = p
                            .deref()
                            .iter()
                            .map(|x| self.intern_external(cache, other.with(*x)))
                            .collect();
                        Arguments::Given(p)
                    }
                };
                let r = self.intern(RefsEnum::ConstructorInvocation(i, p));
                assert_ne!(r,i);
                r
            }
        };
        assert!(
            self.nodes[r].similar(other.as_ref()),
            "{:?} ~ {:?}",
            self.nodes[r],
            other.as_ref()
        );
        println!(
            "{:?}",
            ExplorableRef {
                rf:r,
                nodes: &self.nodes,
            }
        );
        cache.insert(other.rf, r);
        if let Some(x) = cache.get(&other.rf) {
            assert!(
                self.nodes[*x].similar(other.as_ref()),
                "{:?} ~ {:?}",
                ExplorableRef {
                    nodes: &self.nodes,
                    rf: *x
                },
                other
            );

            println!(
                "{:?}",
                ExplorableRef {
                    rf:*x,
                    nodes: &self.nodes,
                }
            );
            return *x;
        }
        r
    }

    pub(crate) fn extend<'a>(&mut self, solver: &'a Solver) -> Internalizer<'a> {
        let mut cached = Internalizer {
            solve: false,
            cache: Default::default(),
            solver,
        };
        for r in solver.iter_refs() {
            // TODO make it imperative ?
            let r = self.intern_external(&mut cached.cache, r);
            if r >= self.refs.len() {
                self.refs.resize(r + 1, false);
            }
            self.refs.set(r, true);
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
        self.print_decls();
        for r in solver.iter_refs() {
            // TODO make it imperative ?
            let r = self.local_solve_intern_external(&mut cached.cache, r);
            if r >= self.refs.len() {
                self.refs.resize(r + 1, false);
            }
            self.refs.set(r, true);
        }
        // TODO extend decls ?
        // for r in solver.iter_decls() {
        //     {

        //     };
        //     let r = self.intern_external(&mut cached.cache, r);
        //     if r >= self.refs.len() {
        //         self.refs.resize(r + 1, false);
        //     }
        //     self.refs.set(r, true);
        // }
        cached
    }

    fn resolve(self) -> Solver {
        // let mut r = self.clone();
        let mut r = Solver {
            nodes: self.nodes.clone(),
            refs: Default::default(),
            decls: self.decls.clone(),
        };
        self.print_decls();
        let mut cache = Default::default();
        for s in self.iter_refs() {
            // TODO make it imperative ?
            if let Some(s) = r.solve_aux(&mut cache, s.rf) {
                if s >= r.refs.len() {
                    r.refs.resize(s + 1, false);
                }
                r.refs.set(s, true);
            }
        }
        // TODO try better
        r
    }
}

pub(crate) struct Internalizer<'a> {
    solve: bool,
    cache: HashMap<RefPtr, RefPtr>,
    solver: &'a Solver,
}

impl<'a> Internalizer<'a> {
    fn intern_external(&mut self, solver: &mut Solver, other: RefPtr) -> RefPtr {
        let r = if self.solve {
            solver.local_solve_intern_external(
                &mut self.cache,
                ExplorableRef {
                    rf: other,
                    nodes: &self.solver.nodes,
                },
            )
        } else {
            solver.intern_external(
                &mut self.cache,
                ExplorableRef {
                    rf: other,
                    nodes: &self.solver.nodes,
                },
            )
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

#[derive(Debug, Clone,PartialEq, Eq)]
pub enum DeclType<Node> {
    Runtime(Box<[Node]>), // Typically erased types
    Compile(Node, Option<Node>, Box<[Node]>),
}

impl PartialAnalysis {
    // apply before commiting/saving subtree
    pub fn resolve(self) -> Self {
        Self {
            current_node: self.current_node,
            solver: self.solver.resolve(),
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
        self.solver.refs.len()
    }

    // pub fn decls<'a>(&'a self) -> impl Iterator<Item = (&'a Declarator, &'a LabelValue)> {
    //     // self.solver.decls.iter()
    //     todo!()
    // }

    pub fn decls_count(&self) -> usize {
        self.solver.decls.len()
    }

    pub fn init<F:FnMut(&str)-> LabelPtr>(kind: &Type, label: Option<LabelPtr>, mut intern_label: F) -> Self {
        
        let mut solver: Solver = Default::default();
        if kind == &Type::Program {
            macro_rules! scoped {
                ( $o:expr, $i:expr ) => {
                   {
                       let o = $o;
                       let i = $i;
                       solver.intern(RefsEnum::ScopedIdentifier(o, i))
                    }
                }
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

            import!("java","lang","String");
            import!("java","lang","Object");

            PartialAnalysis {
                current_node: State::None,
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
                let p = Primitive::new(kind);
                let i = solver.intern(RefsEnum::Primitive(p));
                i
            };
            PartialAnalysis {
                current_node: State::LiteralType(i),
                solver,
            }
        } else if kind.is_primitive() {
            let p = Primitive::new(kind);
            let i = solver.intern(RefsEnum::Primitive(p));
            // let i = label.unwrap();
            // let t = solver.intern(RefsEnum::MaybeMissing);
            // let i = solver.intern(RefsEnum::ScopedIdentifier(t, i));
            PartialAnalysis {
                current_node: State::ScopedTypeIdentifier(i),
                solver,
            }
            // panic!("{:?} {:?}",kind,label);
        } else if kind == &Type::ClassBody {
            {
                let i = intern_label("this");
                let t = solver.intern(RefsEnum::MaybeMissing);
                let i = solver.intern(RefsEnum::ScopedIdentifier(t, i));
                let t = solver.intern(RefsEnum::Root);
                let t = solver.intern(RefsEnum::ConstructorInvocation(t,Arguments::Given(vec![].into_boxed_slice())));
                let d = Declarator::Variable(i);
                solver.add_decl_simple(d, t);
            }
            {
                let t = solver.intern(RefsEnum::MaybeMissing);
                let i = solver.intern(RefsEnum::ConstructorInvocation(t,Arguments::Given(vec![].into_boxed_slice())));
                let t = solver.intern(RefsEnum::Root);
                let t = solver.intern(RefsEnum::ConstructorInvocation(t,Arguments::Given(vec![].into_boxed_slice())));
                let d = Declarator::Executable(i);
                solver.add_decl_simple(d, t);
            }

            PartialAnalysis {
                current_node: State::None,
                solver,
            }
        } else {
            PartialAnalysis {
                current_node: leaf_state(kind, label),
                solver,
            }

        }
    }

    pub fn acc(self, kind: &Type, acc: &mut Self) {
        let mut remapper =
            if kind == &Type::Block 
            || kind == &Type::ConstructorBody 
            || kind == &Type::SwitchBlock {
                acc.solver.local_solve_extend(&self.solver)
            } else {
                acc.solver.extend(&self.solver)
            };

        let current_node = self.current_node;
        println!(
            "{:?} {:?} {:?}\n**{:?}",
            &kind,
            &acc.current_node,
            &current_node,
            acc.refs().collect::<Vec<_>>()
        );
        macro_rules! mm {
            () => {acc.solver.intern(RefsEnum::MaybeMissing)}
        }
        macro_rules! symbol {
            ( $i:expr ) => {
                {
                    let o = mm!();
                    acc.solver.intern(RefsEnum::ScopedIdentifier(o, $i))
                }
            }
        }
        macro_rules! scoped {
            ( $o:expr, $i:expr ) => {
                {
                    let o = $o;
                    acc.solver.intern_ref(RefsEnum::ScopedIdentifier(o, $i))
                }
            };
        }
        macro_rules! sync {
            ( $i:expr ) => {
                remapper.intern_external(&mut acc.solver, $i)
            };
        }
        acc.current_node = match (acc.current_node.take(), current_node) {
            (rest, State::Annotation)if kind == &Type::Modifiers => {
                State::Modifiers(Visibility::None,enum_set!())
            }
            (rest, State::Annotation) => {
                rest
            }
            (x, State::None) if kind == &Type::ArgumentList => {
                assert_eq!(x, State::None);
                State::Arguments(vec![])
            }
            (rest, State::None) if kind == &Type::Block => {
                println!("dlen: {:?}", acc.solver.decls.len());
                println!("{:?}", acc.solver.decls);
                rest
            }


            //program
            (
                State::None,
                State::Declaration {
                    visibility,
                    kind: t,
                    identifier: d,
                },
            ) if kind == &Type::Program => {
                // no package declaration at start of java file
                State::Declaration {
                    visibility,
                    kind: t,
                    identifier: d,
                }
            }
            (State::None, State::PackageDeclaration(p))
                if kind == &Type::Program =>
            {
                State::File {
                    package: Some(sync!(p)),
                    content: vec![],
                }
            }
            (
                State::None,
                State::TypeDeclaration {
                    visibility,
                    identifier: d,
                    members: _,
                },
            ) if kind == &Type::Program => {
                // TODO check for file's class? visibility ? etc
                // TODO maybe bubleup members
                let mut content = vec![];
                match d {
                    DeclType::Compile(d,_,_) => {
                        let d = sync!(d);
                        let i = Declarator::Type(d);
                        content.push((i.clone(), d));
                        acc.solver.add_decl_simple(i.clone(), d);
                    }
                    _ => panic!(),
                }

                State::File {
                    package: None,
                    content,
                }
            }
            (
                State::File {
                    package: p,
                    mut content,
                },
                State::ImportDeclaration(i),
            ) if kind == &Type::Program => {
                let i = sync!(i);
                let (o, i) = match &acc.solver.nodes[i] {
                    RefsEnum::ScopedIdentifier(o, i) => (*o, *i),
                    _ => panic!("must be a scoped id in an import"),
                };
                let c = scoped!(o,i);
                let r = mm!();
                let d = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                acc.solver.add_decl_simple(Declarator::Type(d), c);
                State::File {
                    package: p,
                    content,
                }
            }
            (
                State::File {
                    package: p,
                    mut content,
                },
                State::TypeDeclaration {
                    visibility,
                    identifier,
                    members,
                },
            ) if kind == &Type::Program => {
                // TODO check for file's class? visibility? etc
                // TODO maybe bubleup members
                let identifier = match (identifier, p) {
                    (DeclType::Compile(d,_,_), Some(p)) => {
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
                    (DeclType::Compile(d,_,_), None) => {
                        let d = sync!(d);
                        let i = Declarator::Type(d);
                        content.push((i.clone(), d));
                        acc.solver.add_decl_simple(i.clone(), d);
                        d
                    }
                    _ => panic!(),
                };
                for (d, t) in members {
                    let d = d.with_changed_node(|i| sync!(*i));
                    let t = sync!(t);

                    match &d {
                        Declarator::Executable(d)=> 
                        {
                            {
                                let d = Declarator::Executable(*d);
                                acc.solver.add_decl_simple(d, t);
                            }
                            if let Some(p) = p{
                                let solved = acc.solver.solve_node_with(*d, p);
                                let d = Declarator::Executable(solved);
                                acc.solver.add_decl_simple(d.clone(), t);
                                content.push((d, t));
                            }
                        }
                        Declarator::Field(d)=> 
                        {
                            {
                                let d = Declarator::Field(*d);
                                acc.solver.add_decl_simple(d, t);
                            }
                            if let Some(p) = p{
                                let solved = acc.solver.solve_node_with(*d, p);
                                let d = Declarator::Field(solved);
                                acc.solver.add_decl_simple(d.clone(), t);
                                content.push((d, t));
                            }
                        }
                        Declarator::Type(d)=> 
                        {
                            {
                                let d = Declarator::Type(*d);
                                acc.solver.add_decl_simple(d, t);
                            }
                            if let Some(p) = p{
                                let solved = acc.solver.solve_node_with(*d, p);
                                let d = Declarator::Type(solved);
                                acc.solver.add_decl_simple(d.clone(), t);
                                content.push((d, t));
                            }
                        }
                        x => panic!("{:?}",x)
                    }
                }
                State::File {
                    package: p,
                    content,
                }
            }
            // package
            (State::None, State::ScopedIdentifier(i))
                if kind == &Type::PackageDeclaration =>
            {
                // TODO complete refs
                let i = sync!(i);
                State::PackageDeclaration(i)
            }
            (State::None, State::SimpleIdentifier(i))
                if kind == &Type::PackageDeclaration =>
            {
                // TODO complete refs
                let o = acc.solver.intern(RefsEnum::Root);
                let i = acc.solver.intern(RefsEnum::ScopedIdentifier(o, i));
                State::PackageDeclaration(i)
            }
            // scoped id
            (State::None, State::ScopedIdentifier(i))
                if kind == &Type::ScopedIdentifier =>
            {
                let i = sync!(i);
                State::ScopedIdentifier(i)
            }
            (
                State::ScopedIdentifier(o),
                State::SimpleIdentifier(i),
            ) if kind == &Type::ScopedIdentifier => {
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
            // imports
            (State::None, State::ScopedIdentifier(i))
                if kind == &Type::ImportDeclaration =>
            {
                let i = sync!(i);
                State::ImportDeclaration(i)
            }
            (State::None, State::Modifiers(v, n))
                if kind == &Type::ImportDeclaration =>
            {
                State::Modifiers(v, n)
            }
            (State::Modifiers(v, n), State::ScopedIdentifier(i))
                if kind == &Type::ImportDeclaration =>
            {
                let i = sync!(i);
                State::ImportDeclaration(i) // TODO use static
            }
            (State::Declarations(p), State::None)
                if kind == &Type::LambdaExpression =>
            {
                let i = mm!();
                State::LambdaExpression(i)
            }
            (State::None, State::SimpleTypeIdentifier(t))
                if kind == &Type::MethodDeclaration =>
            {
                let t = scoped!(mm!(),t);
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
                let t = scoped!(mm!(),t);
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
            (
                State::ScopedTypeIdentifier(t),
                State::SimpleIdentifier(i),
            ) if kind == &Type::MethodDeclaration => State::MethodImplementation {
                visibility: Visibility::None,
                kind: Some(t),
                identifier: Some(i),
                parameters: vec![].into_boxed_slice(),
            },
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
                if kind == &Type::MethodDeclaration || kind == &Type::ConstructorDeclaration =>
            {
                State::Modifiers(v, n)
            }
            (State::Modifiers(v0, n0), State::Modifiers(v, n)) => {
                State::Modifiers(
                    if v0 == Visibility::None {
                        v
                    } else {
                        assert_eq!(v, Visibility::None);
                        v0
                    },
                    n0.union(n),
                )
            }
            (
                State::Modifiers(v, n),
                State::SimpleTypeIdentifier(t),
            ) if kind == &Type::MethodDeclaration => {
                let t = scoped!(mm!(),t);
                State::MethodImplementation {
                    visibility: v,
                    kind: Some(t),
                    identifier: None,
                    parameters: Default::default(),
                }
            }
            (
                State::Modifiers(v, n),
                State::ScopedTypeIdentifier(t),
            ) if kind == &Type::MethodDeclaration => {
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
                let t = scoped!(mm!(),t);
                State::MethodImplementation {
                    visibility: v,
                    kind: Some(t),
                    identifier: None,
                    parameters: Default::default(),
                }
            }
            (State::None, State::TypeParameters(t))
                if kind == &Type::MethodDeclaration =>
            {
                for (t, b) in t {
                    let t = scoped!(mm!(),t);
                    let b = b
                        .into_iter()
                        .map(|t| sync!(*t))
                        .collect();
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
                    let t = scoped!(mm!(),t);
                    let b = b
                        .into_iter()
                        .map(|t| sync!(*t))
                        .collect();
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
            (State::None, State::FormalParameters(p))
                if kind == &Type::TryWithResourcesStatement =>
            {
                // TODO it implicitly calls close on resource so need to materialize it
                p.into_iter().for_each(|(i, t)| {
                    let i = sync!(i);
                    let t = sync!(t);
                    acc.solver.add_decl_simple(Declarator::Variable(i), t); // TODO use variable or parameter ?
                });
                State::None
            }
            (
                State::ConstructorImplementation {
                    visibility,
                    identifier: i,
                    parameters: p,
                },
                State::Throws,
            ) if kind == &Type::ConstructorDeclaration => {
                State::ConstructorImplementation {
                    visibility,
                    identifier: i,
                    parameters: p,
                }
            }
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
                State::ConstructorImplementation {
                    visibility,
                    identifier,
                    parameters,
                },
                State::None,
            ) if kind == &Type::ConstructorDeclaration => {
                State::ConstructorImplementation {
                    visibility,
                    identifier,
                    parameters,
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
                    identifier: i,
                    parameters: p,
                },
                State::None,
            ) if kind == &Type::MethodDeclaration => {
                let p = p.into_iter().map(|(i, t)| *t).collect();
                let r = mm!();
                let i = acc
                    .solver
                    .intern(RefsEnum::Invocation(r, i.unwrap(), Arguments::Given(p)));
                State::Declaration {
                    visibility,
                    kind: t.unwrap(),
                    identifier: Declarator::Executable(i),
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
            (_, State::None) if kind == &Type::MethodInvocation => todo!(),
            // (x, State::None) => x,
            (x, y) if kind == &Type::Error => panic!("{:?} {:?} {:?}", kind, x, y),
            (State::None, State::SimpleIdentifier(i))
                if kind == &Type::ClassDeclaration
                    || kind == &Type::EnumDeclaration
                    || kind == &Type::AnnotationTypeDeclaration
                    || kind == &Type::InterfaceDeclaration =>
            {
                let i = scoped!(mm!(),i);
                let d = Declarator::Type(i);
                acc.solver.add_decl_simple(d.clone(), i);
                {
                    let t = acc.solver.intern(RefsEnum::Root);
                    let t = acc.solver.intern(RefsEnum::ConstructorInvocation(t,Arguments::Given(vec![].into_boxed_slice())));
                    let t = Declarator::Executable(t);
                    acc.solver.add_decl_simple(t, i);
                }
                State::TypeDeclaration {
                    visibility: Visibility::None,
                    identifier: DeclType::Compile(i,None,vec![].into_boxed_slice()),
                    members: vec![],
                }
            }
            (State::Modifiers(v, n), State::SimpleIdentifier(i))
                if kind == &Type::ClassDeclaration
                    || kind == &Type::EnumDeclaration
                    || kind == &Type::AnnotationTypeDeclaration
                    || kind == &Type::InterfaceDeclaration =>
            {
                let i = scoped!(mm!(),i);
                let d = Declarator::Type(i);
                acc.solver.add_decl_simple(d.clone(), i);
                {
                    let t = acc.solver.intern(RefsEnum::Root);
                    let t = acc.solver.intern(RefsEnum::ConstructorInvocation(t,Arguments::Given(vec![].into_boxed_slice())));
                    let t = Declarator::Executable(t);
                    acc.solver.add_decl_simple(t, i);
                }
                State::TypeDeclaration {
                    visibility: v,
                    identifier: DeclType::Compile(i,None,vec![].into_boxed_slice()),
                    members: vec![],
                }
            }
            (State::None, State::SimpleIdentifier(i))
                if kind == &Type::EnumConstant =>
            {
                let i = scoped!(mm!(),i);
                let i = Declarator::Field(i);
                State::Declaration {
                    visibility: Visibility::None,
                    kind: mm!(),
                    identifier: i,
                }
            }
            (State::Declaration {
                visibility,
                kind:t,
                identifier
            }, State::Arguments(_))
                if kind == &Type::EnumConstant =>
            {
                // TODO use arguments ie they are calls to enum constructor
                State::Declaration {
                    visibility,
                    kind:t,
                    identifier,
                }
            }
            (State::None, State::SimpleTypeIdentifier(i))
                if kind == &Type::Superclass =>
            {
                let i = scoped!(mm!(),i);
                State::ScopedTypeIdentifier(i)
            }
            (State::None, State::ScopedTypeIdentifier(i))
                if kind == &Type::Superclass =>
            {
                let i = sync!(i);
                State::ScopedTypeIdentifier(i)
            }
            (rest, State::SimpleTypeIdentifier(t))
                if kind == &Type::SuperInterfaces || kind == &Type::ExtendsInterfaces =>
            {
                let mut v = match rest {
                    State::Interfaces(v) => v,
                    State::None => vec![],
                    x => panic!("{:?}", x),
                };
                let t = scoped!(mm!(),t);
                v.push(t);
                State::Interfaces(v)
            }
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
            (
                State::TypeDeclaration {
                    visibility,
                    identifier,
                    mut members,
                },
                State::Declarations(ds),
            ) if kind == &Type::ClassDeclaration
                || kind == &Type::AnnotationTypeDeclaration
                || kind == &Type::EnumDeclaration
                || kind == &Type::InterfaceDeclaration =>
            {
                let i = match &identifier {
                    DeclType::Compile(i,_,_)=> *i,
                    _=> panic!(),
                };
                for (d, t) in ds {
                    let d = d.with_changed_node(|i| sync!(*i));
                    let t = sync!(t);

                    match &d {
                        Declarator::Executable(d)=> 
                        {
                            match &acc.solver.nodes[*d] {
                                RefsEnum::ConstructorInvocation(_,_) => {
                                    {let d = Declarator::Executable(*d);
                                    acc.solver.add_decl(d, identifier.clone());}
                                    // let solved = acc.solver.solve_node_with(*d, i);
                                    // let d = Declarator::Executable(solved);
                                    // acc.solver.add_decl(d.clone(), identifier.clone());
                                    // members.push((d, i));
                                },
                                RefsEnum::Invocation(_, _,_) => {
                                    {let d = Declarator::Executable(*d);
                                    acc.solver.add_decl_simple(d, t);}
                                    let solved = acc.solver.solve_node_with(*d, i);
                                    let d = Declarator::Executable(solved);
                                    acc.solver.add_decl_simple(d.clone(), t);
                                    // members.push((d, t));
                                },
                                x => todo!("{:?}",x),
                            }
                        }
                        Declarator::Field(d)=> 
                        {
                            {
                                let d = Declarator::Field(*d);
                                acc.solver.add_decl_simple(d, t);
                            }
                            {
                                let solved = acc.solver.solve_node_with(*d, i);
                                let d = Declarator::Field(solved);
                                acc.solver.add_decl_simple(d.clone(), t);
                                members.push((d, t));
                            }
                        }
                        Declarator::Type(d)=> 
                        {
                            {
                                let d = Declarator::Type(*d);
                                acc.solver.add_decl_simple(d, t);
                            }
                            {
                                let solved = acc.solver.solve_node_with(*d, i);
                                let d = Declarator::Type(solved);
                                acc.solver.add_decl_simple(d.clone(), t);
                                members.push((d, t));
                            }
                        }
                        x => panic!("{:?}",x)
                    }
                }
                // let r = scoped!(mm!(),i);
                // acc.solver.add_decl_simple(i.clone(), r);
                State::TypeDeclaration {
                    visibility,
                    identifier,
                    members,
                }
            }
            (
                State::TypeDeclaration {
                    visibility,
                    identifier: i,
                    mut members,
                },
                State::TypeParameters(ps),
            ) if kind == &Type::ClassDeclaration
                // || kind == &Type::EnumDeclaration
                || kind == &Type::InterfaceDeclaration =>
            {
                for (d, t) in ps {
                    let d = scoped!(mm!(),d);
                    let d = Declarator::Type(d);
                    let t = t
                        .into_iter()
                        .map(|t| sync!(*t))
                        .collect();
                    acc.solver.add_decl(d.clone(), DeclType::Runtime(t));
                }
                // let r = scoped!(mm!(),i);
                // acc.solver.add_decl_simple(i.clone(), r);
                State::TypeDeclaration {
                    visibility,
                    identifier: i,
                    members,
                }
            }
            (
                State::TypeDeclaration {
                    visibility,
                    identifier: i,
                    mut members,
                },
                State::ScopedTypeIdentifier(t),
            ) if kind == &Type::ClassDeclaration => {
                // TODO use superclass value
                State::TypeDeclaration {
                    visibility,
                    identifier: i,
                    members,
                }
            }
            (
                State::TypeDeclaration {
                    visibility,
                    identifier: i,
                    mut members,
                },
                State::Interfaces(t),
            ) if kind == &Type::ClassDeclaration || kind == &Type::InterfaceDeclaration => {
                // TODO use interfaces
                State::TypeDeclaration {
                    visibility,
                    identifier: i,
                    members,
                }
            }
            (
                State::TypeDeclaration {
                    visibility,
                    identifier: i,
                    mut members,
                },
                State::None,
            ) if kind == &Type::ClassDeclaration || kind == &Type::InterfaceDeclaration || kind == &Type::EnumDeclaration => {
                // TODO use interfaces
                State::TypeDeclaration {
                    visibility,
                    identifier: i,
                    members,
                }
            }
            (State::None, State::Modifiers(v, n))
                if kind == &Type::ClassDeclaration
                    || kind == &Type::EnumDeclaration
                    || kind == &Type::LocalVariableDeclaration
                    || kind == &Type::AnnotationTypeDeclaration
                    || kind == &Type::InterfaceDeclaration =>
            {
                State::Modifiers(v, n)
            }
            (State::None, State::SimpleIdentifier(i))
                if kind == &Type::VariableDeclarator =>
            {
                let r = mm!();
                let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                State::Declarator(Declarator::Variable(i))
            }
            (State::None, State::ScopedTypeIdentifier(t))
                if kind == &Type::LocalVariableDeclaration =>
            {
                let t = sync!(t);
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
                State::Declarator(Declarator::Variable(i)),
            ) if kind == &Type::LocalVariableDeclaration => {
                let i = sync!(i);
                let i = Declarator::Variable(i);
                State::Declaration {
                    visibility,
                    kind: t,
                    identifier: i,
                }
            }
            (State::None, State::SimpleTypeIdentifier(t))
                if kind == &Type::LocalVariableDeclaration =>
            { 
                let t = scoped!(mm!(),t);
                State::Declaration {
                    visibility: Visibility::None,
                    kind: t,
                    identifier: Declarator::None,
                }
            }
            (State::Modifiers(v,n), State::SimpleTypeIdentifier(t))
                if kind == &Type::LocalVariableDeclaration =>
            { 
                let t = scoped!(mm!(),t);
                State::Declaration {
                    visibility: v,
                    kind: t,
                    identifier: Declarator::None,
                }
            }
            (State::Modifiers(v,n), State::ScopedTypeIdentifier(t))
                if kind == &Type::LocalVariableDeclaration =>
            {
                let t = sync!(t);
                State::Declaration {
                    visibility: v,
                    kind: t,
                    identifier: Declarator::None,
                }
            }
            (State::Declarator(Declarator::Variable(v)), _)
                if kind == &Type::VariableDeclarator =>
            {
                State::Declarator(Declarator::Variable(v))
            }
            (State::None, State::ScopedTypeIdentifier(t))
                if kind == &Type::Resource =>
            {
                let t = sync!(t);
                State::Declaration {
                    visibility: Visibility::None,
                    kind: t,
                    identifier: Declarator::None,
                }
            }
            (State::None, State::SimpleTypeIdentifier(t))
                if kind == &Type::Resource =>
            {
                let t = scoped!(mm!(),t);
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
                let i = scoped!(mm!(),i);
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
                    x => todo!("{:?}",x),
                };
                let d = Declarator::Variable(i);
                State::Declaration {
                    visibility,
                    kind: t,
                    identifier: d,
                }
            }
            (
                State::SimpleTypeIdentifier(t),
                State::Declarator(Declarator::Variable(i)),
            ) if kind == &Type::FieldDeclaration => {
                // TODO simple type identifier should be a type identifier ie. already scoped
                let t = scoped!(mm!(),t);
                let i = sync!(i);
                let i = Declarator::Field(i);
                // not used directly
                // acc.solver.add_decl_simple(i.clone(), t);
                State::Declaration {
                    visibility: Visibility::None,
                    kind: t,
                    identifier: i,
                }
            }
            (
                State::Modifiers(v, n),
                State::SimpleTypeIdentifier(t),
            ) if kind == &Type::FieldDeclaration || kind == &Type::ConstantDeclaration => {
                // TODO simple type identifier should be a type identifier ie. already scoped
                let t = scoped!(mm!(),t);
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
                State::Modifiers(v, n),
                State::ScopedTypeIdentifier(t),
            ) if kind == &Type::FieldDeclaration || kind == &Type::ConstantDeclaration => {
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
            (
                State::Modifiers(v, n),
                State::SimpleTypeIdentifier(t),
            ) if kind == &Type::AnnotationTypeElementDeclaration => {
                // TODO simple type identifier should be a type identifier ie. already scoped
                let t = scoped!(mm!(),t);
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
                State::Modifiers(v, n),
                State::ScopedTypeIdentifier(t),
            ) if kind == &Type::FieldDeclaration => {
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
            (
                State::Declaration {
                    visibility,
                    kind: t,
                    identifier: _,
                },
                State::SimpleIdentifier(i),
            ) if kind == &Type::AnnotationTypeElementDeclaration => {
                // TODO simple type identifier should be a type identifier ie. already scoped
                let i = scoped!(mm!(),i);
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
                    kind:t,
                    identifier: i,
                },
                State::LiteralType(_),
            ) if kind == &Type::AnnotationTypeElementDeclaration && i!= Declarator::None => {
                // TODO do something with default value
                State::Declaration {
                    visibility,
                    kind:t,
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
                // TODO simple type identifier should be a type identifier ie. already scoped
                let i = sync!(i);
                let i = Declarator::Field(i);
                // not used directly
                // acc.solver.add_decl_simple(i.clone(), t);
                State::Declaration {
                    visibility,
                    kind: t,
                    identifier: i,
                }
            }
            (State::None, State::SimpleTypeIdentifier(t))
                if kind == &Type::FieldDeclaration || kind == &Type::AnnotationTypeElementDeclaration =>
            {
                let t = scoped!(mm!(),t);
                State::Declaration {
                    visibility: Visibility::None,
                    kind: t,
                    identifier: Declarator::None,
                }
            }
            (State::None, State::ScopedTypeIdentifier(t))
                if kind == &Type::FieldDeclaration || kind == &Type::AnnotationTypeElementDeclaration =>
            {
                let t = sync!(t);
                State::Declaration {
                    visibility: Visibility::None,
                    kind: t,
                    identifier: Declarator::None,
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
                if kind == &Type::FormalParameter || kind == &Type::SpreadParameter =>
            {
                // TODO spread parameter is hard for invocation matching on check ? cannot use param ?
                // TODO spread parameter is hard for decl matching on solve
                // NOTE method ref resolution (overloading)
                // 1)strict invocation: fixed arity method resolution, no boxing/unboxing )
                // 2)loose invocation: fixed arity method resolution, boxing/unboxing
                // 3)variable arity invocation: variable arity method resolution, boxing/unboxing
                let t = scoped!(mm!(),t);
                State::ScopedTypeIdentifier(t)
            }
            (State::None, State::ScopedTypeIdentifier(t))
                if kind == &Type::FormalParameter || kind == &Type::SpreadParameter =>
            {
                let t = sync!(t);
                State::ScopedTypeIdentifier(t)
            }
            (
                State::ScopedTypeIdentifier(t),
                State::Declarator(d),
            ) if kind == &Type::SpreadParameter => {
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
            (State::None, State::CatchTypes(v))
                if kind == &Type::CatchFormalParameter =>
            {
                State::CatchTypes(v)
            }
            (State::CatchTypes(v), State::SimpleIdentifier(i))
                if kind == &Type::CatchFormalParameter =>
            {
                State::CatchParameter {
                    kinds: v.into_boxed_slice(),
                    identifier: i,
                }
            }
            (
                State::None,
                State::CatchParameter {
                    kinds: b,
                    identifier: i,
                },
            ) if kind == &Type::CatchClause => {
                let i = scoped!(mm!(),i);
                let d = Declarator::Variable(i);
                // TODO send whole intersection
                // let b = b.into_iter().map(|t|
                //     sync!(*t)
                // ).collect();
                let b = sync!(b[0]);
                acc.solver.add_decl_simple(d.clone(), b);
                State::None
            }
            (rest, State::SimpleTypeIdentifier(i)) if kind == &Type::CatchType => {
                let mut v = match rest {
                    State::CatchTypes(l) => l,
                    State::None => vec![],
                    x => panic!("{:?}", x),
                };
                let i = scoped!(mm!(),i);
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
            (
                State::ScopedTypeIdentifier(t),
                State::SimpleIdentifier(i),
            ) if kind == &Type::FormalParameter => {
                let i = scoped!(mm!(),i);
                let i = Declarator::Variable(i);
                State::Declaration {
                    visibility: Visibility::None,
                    kind: t,
                    identifier: i,
                }
            }
            (
                State::SimpleTypeIdentifier(t),
                State::SimpleIdentifier(i),
            ) if kind == &Type::FormalParameter => {
                let t = scoped!(mm!(),t);
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
                kind: t,
                identifier: i,
            },
            (State::None, expr)
                if kind == &Type::DimensionsExpr =>
            {   
                let i = match expr {
                        State::LiteralType(t) => sync!(t),
                        State::SimpleIdentifier(t) => scoped!(mm!(),t),
                        State::ScopedIdentifier(i) => sync!(i),
                        State::FieldIdentifier(i) => sync!(i),
                        State::Invocation(i) => sync!(i),
                        State::ConstructorInvocation(i) => sync!(i),
                        x => panic!("{:?}",x),
                };
                State::ScopedIdentifier(i)
            }
            //ArrayCreationExpression
            (State::None, State::SimpleTypeIdentifier(i))
            if kind == &Type::ArrayCreationExpression =>
            {
                let i = scoped!(mm!(),i);
                State::ScopedIdentifier(i)
            }
            (State::None, State::ScopedTypeIdentifier(i))
                if kind == &Type::ArrayCreationExpression =>
            {
                let i = sync!(i);
                State::ScopedIdentifier(i)
            }
            (State::ScopedIdentifier(i), State::Dimensions)
                if kind == &Type::ArrayCreationExpression =>
            {
                let i = acc
                    .solver
                    .intern_ref(RefsEnum::ConstructorInvocation(i, Arguments::Unknown));
                // TODO use dimension
                State::ConstructorInvocation(i)
            }
            (State::ConstructorInvocation(i), State::None)
                if kind == &Type::ArrayCreationExpression =>
            {
                State::ConstructorInvocation(i)
            }
            (State::ConstructorInvocation(i), State::ScopedIdentifier(_))
                if kind == &Type::ArrayCreationExpression =>
            {
                // TODO use the dimension expr
                State::ConstructorInvocation(i)
            }
            (State::ConstructorInvocation(i), State::LiteralType(_))
                if kind == &Type::ArrayCreationExpression =>
            {
                // TODO use the dimension expr
                State::ConstructorInvocation(i)
            }
            (State::ConstructorInvocation(i), State::Dimensions)
                if kind == &Type::ArrayCreationExpression =>
            {
                // TODO use the dimension expr
                State::ConstructorInvocation(i)
            }
            (State::ScopedIdentifier(i), State::LiteralType(_))
                if kind == &Type::ArrayCreationExpression =>
            {
                let i = acc
                    .solver
                    .intern_ref(RefsEnum::ConstructorInvocation(i, Arguments::Unknown));
                // TODO use dimension
                State::ConstructorInvocation(i)
            }
            (
                State::ScopedIdentifier(i),
                State::FieldIdentifier(_),
            ) if kind == &Type::ArrayCreationExpression => {
                let i = acc
                    .solver
                    .intern_ref(RefsEnum::ConstructorInvocation(i, Arguments::Unknown));
                // TODO use dimension
                State::ConstructorInvocation(i)
            }
            (
                State::ScopedIdentifier(i),
                State::ScopedIdentifier(_),
            ) if kind == &Type::ArrayCreationExpression => {
                let i = acc
                    .solver
                    .intern_ref(RefsEnum::ConstructorInvocation(i, Arguments::Unknown));
                // TODO use dimension
                State::ConstructorInvocation(i)
            }
            (State::None, expr)
                if kind == &Type::ElementValueArrayInitializer =>
            {
                // TODO ElementValueArrayInit return something else than None for AnnotationArgumentList
                match expr {
                    State::LiteralType(t) => (),
                    State::SimpleIdentifier(t) => {scoped!(mm!(),t);},
                    State::ScopedIdentifier(_) => (),
                    State::FieldIdentifier(_) => (),
                    State::Invocation(_) => (),
                    State::ConstructorInvocation(_) => (),
                    x => panic!("{:?}",x),
                };
                State::None
            }
            // ArrayInit
            (State::None, expr)
                if kind == &Type::ArrayInitializer =>
            {
                match expr {
                    State::LiteralType(t) => (),
                    State::SimpleIdentifier(t) => {scoped!(mm!(),t);},
                    State::This(t) => {scoped!(mm!(),t);},
                    State::ScopedIdentifier(_) => (),
                    State::FieldIdentifier(_) => (),
                    State::Invocation(_) => (),
                    State::ConstructorInvocation(_) => (),
                    State::None => (), // TODO check
                    x => panic!("{:?}",x),
                };
                State::None
            }
            //ObjectCreationExpression
            (State::None, State::SimpleTypeIdentifier(i))
                if kind == &Type::ObjectCreationExpression =>
            {
                let r = mm!();
                State::InvocationId(r, i)
            }
            (
                State::SimpleTypeIdentifier(o),
                State::SimpleTypeIdentifier(i),
            ) if kind == &Type::ObjectCreationExpression => {
                let o = scoped!(mm!(),o);
                State::InvocationId(o, i)
            }
            (
                State::ScopedTypeIdentifier(o),
                State::ScopedTypeIdentifier(i),
            ) if kind == &Type::ObjectCreationExpression => {
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
                let p = p
                    .deref()
                    .iter()
                    .map(|i| sync!(*i))
                    .collect();
                let r = acc
                    .solver
                    .intern_ref(RefsEnum::ConstructorInvocation(i, Arguments::Given(p)));
                State::ConstructorInvocation(r)
            }
            (State::InvocationId(r, i), State::Arguments(p))
                if kind == &Type::ObjectCreationExpression =>
            {
                // TODO invocationId may not be the best way
                let p = p
                    .deref()
                    .iter()
                    .map(|i| sync!(*i))
                    .collect();
                let i = scoped!(r,i);
                let r = acc
                    .solver
                    .intern_ref(RefsEnum::ConstructorInvocation(i, Arguments::Given(p)));
                State::ConstructorInvocation(r)
            }
            (
                State::ConstructorInvocation(r),
                State::Declarations(v),
            ) if kind == &Type::ObjectCreationExpression => {
                State::ConstructorInvocation(r)
            }
            (State::None, State::None)
                if kind == &Type::Modifiers
                 =>
            {
                State::None
            }
            (State::None, State::None)
                if kind == &Type::AnnotationTypeDeclaration
                || kind == &Type::FieldDeclaration // TODO not sure
                 =>
            {
                State::None
            }
            (State::None, State::Modifiers(v, n))
                if kind == &Type::Modifiers =>
            {
                State::Modifiers(v, n)
            }
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
                let i = scoped!(mm!(),i);
                State::ScopedIdentifier(i)
            }
            (State::None, expr)
                if kind == &Type::ArrayAccess =>
            {
                // TODO simp more FieldIdentifiers to ScopedIdentifier
                let i = match expr {
                    State::LiteralType(t) => sync!(t),
                    State::SimpleIdentifier(t) => scoped!(mm!(),t),
                    State::ScopedIdentifier(i) => sync!(i),
                    State::FieldIdentifier(i) => sync!(i),
                    State::Invocation(i) => sync!(i),
                    State::ConstructorInvocation(i) => sync!(i),
                    x => panic!("{:?}",x),
                };
                State::ScopedIdentifier(i)
            }
            (
                State::ScopedIdentifier(o),
                expr,
            ) if kind == &Type::ArrayAccess => {
                // TODO create RefsEnum variant to use access expr and solve type of array
                match expr {
                    State::LiteralType(t) => (),
                    State::SimpleIdentifier(t) => {scoped!(mm!(),t);},
                    State::This(t) => {scoped!(mm!(),t);},
                    State::ScopedIdentifier(_) => (),
                    State::FieldIdentifier(_) => (),
                    State::Invocation(_) => (),
                    State::ConstructorInvocation(_) => (),
                    State::None => (), // TODO check
                    x => panic!("{:?}",x),
                };
                State::ScopedIdentifier(o)
            }
            (State::None, expr)
                if kind == &Type::FieldAccess =>
            {
                let i = match expr {
                    State::LiteralType(t) => sync!(t),
                    State::SimpleIdentifier(t) => scoped!(mm!(),t),
                    State::This(t) => scoped!(mm!(),t),
                    State::ScopedIdentifier(i) => sync!(i),
                    State::FieldIdentifier(i) => sync!(i),
                    State::Invocation(i) => sync!(i),
                    State::ConstructorInvocation(i) => sync!(i),
                    State::None => panic!("should handle super"),
                    x => panic!("{:?}",x),
                };
                State::ScopedIdentifier(i)
            }
            (State::ScopedIdentifier(o), State::SimpleIdentifier(i))
                if kind == &Type::FieldAccess =>
            {
                let i = scoped!(o, i);
                State::ScopedIdentifier(i)
            }
            (State::ScopedIdentifier(o), State::This(i))
                if kind == &Type::FieldAccess =>
            {
                let i = scoped!(o, i);
                State::ScopedIdentifier(i)
            }
            // MethodInvocation f()
            (State::None, State::SimpleIdentifier(t))
                if kind == &Type::MethodInvocation =>
            {
                State::SimpleIdentifier(t)
            }
            (State::SimpleIdentifier(i), State::Arguments(p))
                if kind == &Type::MethodInvocation =>
            {
                let p = p
                    .deref()
                    .iter()
                    .map(|i| sync!(*i))
                    .collect();
                let r = mm!();
                let r = acc
                    .solver
                    .intern_ref(RefsEnum::Invocation(r, i, Arguments::Given(p)));
                State::ScopedIdentifier(r)  // or should it be an invocation
            }
            // MethodInvocation x.f()
            (State::None, expr)
                if kind == &Type::MethodInvocation =>
            {
                let i = match expr {
                    State::LiteralType(t) => sync!(t),
                    State::SimpleIdentifier(t) => panic!("should be handled specifically"),
                    State::This(t) => scoped!(mm!(),t),
                    State::Super(t) => scoped!(mm!(),t),
                    State::ScopedIdentifier(i) => sync!(i),
                    State::FieldIdentifier(i) => sync!(i),
                    State::Invocation(i) => sync!(i),
                    State::ConstructorInvocation(i) => sync!(i),
                    State::None => panic!("should handle super"),
                    x => panic!("{:?}",x),
                };
                State::ScopedIdentifier(i)
            }
            (
                State::ScopedIdentifier(o),
                expr,
            ) if kind == &Type::MethodInvocation => {
                match expr {
                    State::SimpleIdentifier(i) => State::InvocationId(o, i),
                    State::This(i) => State::ScopedIdentifier(scoped!(o,i)),
                    State::Super(i) => State::ScopedIdentifier(scoped!(o,i)),
                    x => panic!("{:?}",x),
                }
            }
            (
                State::SimpleIdentifier(o),
                expr,
            ) if kind == &Type::MethodInvocation => {
                match expr {
                    State::SimpleIdentifier(i) => State::InvocationId(scoped!(mm!(),o), i),
                    State::This(i) => State::ScopedIdentifier(scoped!(scoped!(mm!(),o),i)),
                    State::Super(i) => State::ScopedIdentifier(scoped!(scoped!(mm!(),o),i)),
                    x => panic!("{:?}",x),
                }
            }
            (State::InvocationId(o, i), State::Arguments(p))
                if kind == &Type::MethodInvocation =>
            {
                let p = p
                    .deref()
                    .iter()
                    .map(|i| sync!(*i))
                    .collect();
                let r = acc
                    .solver
                    .intern_ref(RefsEnum::Invocation(o, i, Arguments::Given(p)));
                State::ScopedIdentifier(r) // or should it be an invocation
            }
            (State::None, expr)
                if kind == &Type::MethodReference =>
            {
                let i = match expr {
                    State::LiteralType(t) => sync!(t),
                    State::SimpleIdentifier(t) => scoped!(mm!(),t),
                    State::This(t) => scoped!(mm!(),t),
                    State::ScopedTypeIdentifier(i) => sync!(i), // TODO fix related to getting type alias from tree-sitter API
                    State::ScopedIdentifier(i) => sync!(i),
                    State::FieldIdentifier(i) => panic!("not possible"),
                    State::Invocation(i) => panic!("not possible"),
                    State::ConstructorInvocation(i) => panic!("not possible"),
                    State::None => panic!("should handle before"),
                    x => panic!("{:?}",x),
                };
                State::ScopedIdentifier(i)
            }
            (
                State::ScopedIdentifier(o),
                State::SimpleIdentifier(i),
            ) if kind == &Type::MethodReference => {
                let r = acc.solver.intern_ref(RefsEnum::MethodReference(o, i));
                State::MethodReference(r)
            }
            (State::ScopedIdentifier(o), State::None)
                if kind == &Type::MethodReference =>
            {
                let r = acc
                    .solver
                    .intern_ref(RefsEnum::ConstructorInvocation(o, Arguments::Unknown));
                State::MethodReference(r)
            }
            // this() or super()
            // TODO ExplicitConstructorInvocation try not to pollute ref resolution
            (State::None, expr)
            if kind == &Type::ExplicitConstructorInvocation =>
            {
                match &expr {
                    State::SimpleIdentifier(_) => expr,
                    State::This(_) => expr,
                    State::Super(_) => expr,
                    x => panic!("{:?}",x)
                }
            }
            (State::ScopedIdentifier(o), State::SimpleIdentifier(i))
                if kind == &Type::ExplicitConstructorInvocation =>
            {
                panic!("used?");
                let i = acc.solver.intern(RefsEnum::ScopedIdentifier(o, i));
                State::ScopedIdentifier(i)
            }
            (State::ScopedIdentifier(o), State::Super(i))
                if kind == &Type::ExplicitConstructorInvocation =>
            {
                panic!("used?");
                let i = scoped!(o,i);
                State::ScopedIdentifier(i)
            }
            (State::SimpleIdentifier(o), State::Super(i))
                if kind == &Type::ExplicitConstructorInvocation =>
            {
                let i = scoped!(scoped!(mm!(),o),i);
                State::ScopedIdentifier(i)
            }
            (expr, State::Arguments(p))
                if kind == &Type::ExplicitConstructorInvocation =>
            {
                let i = match expr {
                    State::ScopedIdentifier(i) => i,
                    State::Super(i) => scoped!(mm!(),i),
                    State::This(i) => scoped!(mm!(),i),
                    _=> panic!()
                };
                let p = p
                    .deref()
                    .iter()
                    .map(|i| sync!(*i))
                    .collect();
                let i = acc
                    .solver
                    .intern_ref(RefsEnum::ConstructorInvocation(i, Arguments::Given(p)));
                State::ConstructorInvocation(i)
            }
            (
                rest,
                State::TypeDeclaration {
                    visibility,
                    identifier: d,
                    members: _,
                },
            ) if kind == &Type::ClassBody || kind == &Type::InterfaceBody => {
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
                    },
                };
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
            ) if kind == &Type::ClassBody || kind == &Type::InterfaceBody || kind == &Type::AnnotationTypeBody => {
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
            (
                rest,
                State::Declaration {
                    visibility,
                    kind: t,
                    identifier: d,
                },
            ) if kind == &Type::EnumBody => {
                let mut v = match rest {
                    State::Declarations(u) => u,
                    State::None => vec![],
                    _ => panic!(),
                };
                let t = sync!(t);
                let d = d.with_changed_node(|i| sync!(*i));
                v.push((d, t));
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
            (
                rest,
                State::Declaration {
                    visibility,
                    kind: t,
                    identifier: d,
                },
            ) if kind == &Type::EnumBodyDeclarations => {
                let mut v = match rest {
                    State::Declarations(u) => u,
                    State::None => vec![],
                    _ => panic!(),
                };
                let t = sync!(t);
                let d = d.with_changed_node(|i| sync!(*i));
                v.push((d, t));
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
            ) if kind == &Type::ClassBody || kind == &Type::InterfaceBody => {
                let t = sync!(t.unwrap());
                let r = mm!();
                let p = p.into_iter().map(|(_, t)| sync!(*t)).collect();
                let d = acc.solver.intern_ref(RefsEnum::Invocation(
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
                let r = mm!();
                let t = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i.unwrap()));
                let p = p.into_iter().map(|(_, t)| sync!(*t)).collect();
                let i = acc.solver.intern(RefsEnum::ConstructorInvocation(
                    t,
                    Arguments::Given(p),
                ));
                let r = acc.solver.intern(RefsEnum::Root);
                let t = acc.solver.intern(RefsEnum::ConstructorInvocation(r,Arguments::Given(vec![].into_boxed_slice())));
                let d = Declarator::Executable(i);
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
                State::Declaration {
                    visibility,
                    kind: t,
                    identifier: d,
                },
            ) if kind == &Type::Block || kind == &Type::SwitchBlockStatementGroup => {
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
                State::None
            }
            (
                rest,
                State::TypeDeclaration {
                    visibility,
                    identifier: d,
                    members: _,
                },
            ) if kind == &Type::Block => {
                match d {
                    DeclType::Runtime(_) => panic!(),
                    DeclType::Compile(t, _, _) => {
                        let t = sync!(t);
                        let d = Declarator::Type(t);
                        acc.solver.add_decl_simple(d, t);
                    },
                };
                match rest {
                    State::None => (),
                    _ => panic!(),
                }
                // we do not need declarations outside of the map to solve local variable
                // because a local variable declaration is never visible from outside
                // TODO declarations needed in MethodDeclaration
                State::None
            }
            (State::None, rest) if kind == &Type::SynchronizedStatement => {
                match rest {
                    State::None => (),
                    State::FieldIdentifier(_) => (),
                    State::ScopedIdentifier(_) => (),
                    State::Invocation(_) => (),
                    State::ConstructorInvocation(_) => (),
                    State::SimpleIdentifier(i) => {
                        let r = mm!();
                        acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                    }
                    x => panic!("{:?}", x),
                }
                State::None
            }
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
            (State::None, State::Modifiers(_, _))
                if kind == &Type::StaticInitializer =>
            {
                State::None
            }
            (rest, State::LiteralType(_))
                if kind == &Type::Block
                    || kind == &Type::WhileStatement
                    || kind == &Type::ConstructorBody =>
            {
                match rest {
                    State::None => (),
                    _ => panic!(),
                }
                State::None
            }
            (State::None, State::None)
                if kind == &Type::SwitchBlockStatementGroup
                    || kind == &Type::StaticInitializer
                    || kind == &Type::MethodDeclaration
                    || kind == &Type::Block
                    || kind == &Type::ConstructorBody
                    || kind == &Type::WhileStatement
                    || kind == &Type::DoStatement
                    || kind == &Type::IfStatement
                    || kind == &Type::LocalVariableDeclaration
                    || kind == &Type::SwitchBlock
                    || kind == &Type::SwitchStatement
                    || kind == &Type::TryStatement
                    || kind == &Type::TryWithResourcesStatement
                    || kind == &Type::TryWithResourcesExtendedStatement
                    || kind == &Type::SynchronizedStatement
                    || kind == &Type::FinallyClause
                    || kind == &Type::CatchClause =>
            {
                State::None
            }
            (
                State::None,
                State::Declaration {
                    visibility,
                    kind: t,
                    identifier: d,
                },
            ) if kind == &Type::ForStatement => {
                let t = sync!(t);
                let d = d.with_changed_node(|i| sync!(*i));
                State::Declaration {
                    visibility,
                    kind: t,
                    identifier: d,
                }
            }
            (State::None, State::ScopedIdentifier(i))
                if kind == &Type::ForStatement || kind == &Type::DoStatement =>
            {
                State::None
            }
            (State::None, State::None)
                if kind == &Type::ForStatement =>
            {
                State::None
            }
            (State::None, State::LiteralType(_))
                if kind == &Type::ForStatement || kind == &Type::DoStatement =>
            {
                State::None
            }
            (
                State::Declaration {
                    visibility,
                    kind: t,
                    identifier: d,
                },
                State::ScopedIdentifier(_),
            ) if kind == &Type::ForStatement => State::Declaration {
                visibility,
                kind: t,
                identifier: d,
            },
            (
                State::Declaration {
                    visibility,
                    kind: t,
                    identifier: d,
                },
                State::Invocation(_),
            ) if kind == &Type::ForStatement => State::Declaration {
                visibility,
                kind: t,
                identifier: d,
            },
            (
                State::Declaration {
                    visibility,
                    kind: t,
                    identifier: d,
                },
                State::LiteralType(i),
            ) if kind == &Type::ForStatement => {
                    let i = sync!(i);
                    State::Declaration {
                    visibility,
                    kind: t,
                    identifier: d,
                }
            },
            (
                State::Declaration {
                    visibility,
                    kind: t,
                    identifier: d,
                },
                State::None,
            ) if kind == &Type::ForStatement => State::None,



            //EnhancedFor
            //  enhanced for var decl
            (State::None, State::SimpleTypeIdentifier(t))
                if kind == &Type::EnhancedForStatement =>
            {
                let t = scoped!(mm!(),t);
                State::ScopedTypeIdentifier(t)
            }
            (State::None, State::ScopedTypeIdentifier(t))
                if kind == &Type::EnhancedForStatement =>
            {
                let t = sync!(t);
                State::ScopedTypeIdentifier(t)
            }
            (
                State::ScopedTypeIdentifier(t),
                State::SimpleIdentifier(i),
            ) if kind == &Type::EnhancedForStatement => {
                let i = scoped!(mm!(),i);
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
                let i = scoped!(mm!(),i);
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
                let i = scoped!(mm!(),i);
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
            ) if kind == &Type::EnhancedForStatement => {
                State::Declaration {
                    visibility,
                    kind: t,
                    identifier: d,
                }
            }
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





            (rest, State::SimpleIdentifier(i))
                if kind == &Type::InferredParameters =>
            {
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
            (State::None, State::SimpleIdentifier(i))
                if kind == &Type::LambdaExpression =>
            {
                let r = mm!();
                let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                let i = Declarator::Variable(i);
                acc.solver.add_decl_simple(i.clone(), r);
                State::Declarations(vec![(i, r)])
            }
            (State::None, State::Declarations(v))
                if kind == &Type::LambdaExpression =>
            {
                let v = v
                    .into_iter()
                    .map(|(i, t)| {
                        let i =
                            i.with_changed_node(|i| sync!(*i));
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
                let i = scoped!(mm!(),i);
                // TODO solve references to parameters
                let i = mm!();
                State::LambdaExpression(i)
            }
            (State::Declarations(p), State::This(i))
                if kind == &Type::LambdaExpression =>
            {
                let i = scoped!(mm!(),i);
                // TODO solve references to parameters
                let i = mm!();
                State::LambdaExpression(i)
            }
            (State::Declarations(p), State::LiteralType(t))
                if kind == &Type::LambdaExpression =>
            {
                // TODO solve references to parameters
                let i = sync!(t);
                State::LiteralType(t)
            }
            (rest, State::SimpleTypeIdentifier(i)) if kind == &Type::Throws => {
                let i = scoped!(mm!(),i);
                State::Throws
            }
            (rest, State::ScopedTypeIdentifier(i)) if kind == &Type::Throws => {
                State::Throws
            }
            (State::None, State::SimpleTypeIdentifier(t))
                if kind == &Type::GenericType =>
            {
                State::SimpleTypeIdentifier(t)
            }
            (State::None, State::ScopedTypeIdentifier(t))
                if kind == &Type::GenericType =>
            {
                let t = sync!(t);
                State::ScopedTypeIdentifier(t)
            }
            (State::SimpleTypeIdentifier(t), State::Arguments(_))
                if kind == &Type::GenericType =>
            {
                // TODO use arguments
                State::SimpleTypeIdentifier(t)
            }
            (State::ScopedTypeIdentifier(t), State::Arguments(_))
                if kind == &Type::GenericType =>
            {
                // TODO use arguments
                State::ScopedTypeIdentifier(t)
            }
            // TypeParameter
            (State::None, State::None)
                if kind == &Type::TypeParameter =>
            {
                State::None
            }
            (State::None, State::SimpleIdentifier(i))
                if kind == &Type::TypeParameter =>
            {
                State::SimpleIdentifier(i)
            }
            (State::SimpleIdentifier(i), State::TypeBound)
                if kind == &Type::TypeParameter =>
            {
                // TODO use type bound
                State::SimpleIdentifier(i)
            }
            (rest, State::SimpleIdentifier(i)) if kind == &Type::TypeParameters => {
                let mut v = match rest {
                    State::TypeParameters(v) => v,
                    State::None => vec![],
                    _ => todo!(),
                };
                v.push((i, vec![].into_boxed_slice()));
                State::TypeParameters(v)
            }
            (_, State::SimpleTypeIdentifier(t))
                if kind == &Type::TypeBound =>
            {
                let t = scoped!(mm!(),t);
                // TODO propag to use for solving refs
                State::TypeBound
            }
            (_, State::ScopedTypeIdentifier(t))
                if kind == &Type::TypeBound =>
            {
                // TODO propag to use for solving refs
                State::TypeBound
            }
            (State::None, State::None) if kind == &Type::Wildcard => {
                let r = mm!();
                State::ScopedTypeIdentifier(r)
            }
            (State::None, State::WildcardExtends(t))
                if kind == &Type::Wildcard =>
            {
                // TODO solve correctly ie. DeclType::Runtime
                let r = mm!();
                State::ScopedTypeIdentifier(r)
            }
            (State::None, State::WildcardSuper(t))
                if kind == &Type::Wildcard =>
            {
                // TODO solve correctly ie. DeclType::Runtime
                let r = mm!();
                State::ScopedTypeIdentifier(r)
            }
            (State::None, State::WildcardSuper(t))
                if kind == &Type::Wildcard =>
            {
                let r = mm!();
                State::ScopedTypeIdentifier(r)
            }
            (State::None, State::SimpleTypeIdentifier(t))
                if kind == &Type::WildcardExtends =>
            {
                let t = scoped!(mm!(),t);
                State::WildcardExtends(t)
            }
            (State::None, State::ScopedTypeIdentifier(t))
                if kind == &Type::WildcardExtends =>
            {
                let t = sync!(t);
                State::WildcardExtends(t)
            }
            (State::None, State::Super(t))
                if kind == &Type::WildcardSuper =>
            {
                let r = mm!();
                State::WildcardSuper(r)
            }
            (State::None, State::SimpleTypeIdentifier(t))
                if kind == &Type::WildcardSuper =>
            {
                let r = mm!();
                State::WildcardSuper(r)
            }
            (
                State::WildcardSuper(_),
                State::SimpleTypeIdentifier(t),
            ) if kind == &Type::WildcardSuper => {
                let t = scoped!(mm!(),t);
                State::WildcardSuper(t)
            }
            (State::None, State::SimpleIdentifier(i))
                if kind == &Type::WildcardSuper =>
            {
                let t = scoped!(mm!(),i);
                State::WildcardSuper(t)
            }
            (State::None, State::ScopedTypeIdentifier(t))
                if kind == &Type::WildcardSuper =>
            {
                let t = sync!(t);
                State::WildcardSuper(t)
            }
            (rest, State::SimpleTypeIdentifier(t))
                if kind == &Type::TypeArguments =>
            {
                let mut v = match rest {
                    State::Arguments(v) => v,
                    State::None => vec![],
                    _ => vec![],
                };
                let t = scoped!(mm!(),t);
                v.push(t);
                State::Arguments(v)
            }
            (rest, State::ScopedTypeIdentifier(i))
                if kind == &Type::TypeArguments =>
            {
                let mut v = match rest {
                    State::Arguments(v) => v,
                    State::None => vec![],
                    _ => vec![],
                };
                let t = sync!(i);
                v.push(t);
                State::Arguments(v)
            }
            (
                rest,
                State::Declaration {
                    visibility,
                    kind: t,
                    identifier: d,
                },
            ) if kind == &Type::FormalParameters || kind == &Type::ResourceSpecification => {
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
                if kind == &Type::FormalParameters || kind == &Type::ResourceSpecification =>
            {
                let mut v = match rest {
                    State::FormalParameters(l) => l,
                    State::None => vec![],
                    x => panic!("{:?}", x),
                };
                State::FormalParameters(v)
            }
            // ArgumentList
            (rest, State::MethodReference(i)) if kind == &Type::ArgumentList => {
                let i = sync!(i);
                let mut v = match rest {
                    State::Arguments(l) => l,
                    State::None => vec![],
                    x => panic!("{:?}", x),
                };
                v.push(i);
                println!("{:?}", acc.solver.nodes);
                println!("{:?}", acc.solver.nodes[i]);
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
                    State::SimpleIdentifier(t) => scoped!(mm!(),t),
                    State::This(t) => scoped!(mm!(),t),
                    State::ScopedIdentifier(i) => sync!(i),
                    State::FieldIdentifier(i) => sync!(i),
                    State::Invocation(i) => sync!(i),
                    State::ConstructorInvocation(i) => sync!(i),
                    x => panic!("{:?}",x),
                };
                v.push(i);
                State::Arguments(v)
            }



            // STATEMENTS
            (State::None, State::SimpleIdentifier(i))
                if kind == &Type::ThrowStatement =>
            {
                let i = scoped!(mm!(),i);
                State::None
            }
            (State::None, State::LiteralType(_))
                if kind == &Type::IfStatement =>
            {
                State::None
            }
            (State::None, State::FieldIdentifier(_))
                if kind == &Type::IfStatement =>
            {
                State::None
            }
            (State::None, State::LiteralType(t))
                if kind == &Type::ExpressionStatement =>
            {
                let t = sync!(t);
                State::None
            }
            (State::None, State::LiteralType(t))
                if kind == &Type::IfStatement =>
            {
                let t = sync!(t);
                State::None
            }
            (State::None, State::ConstructorInvocation(i))
                if kind == &Type::IfStatement =>
            {
                let i = sync!(i);
                State::None
            }
            (State::None, State::ConstructorInvocation(i))
                if kind == &Type::ExpressionStatement
                    || kind == &Type::ReturnStatement
                    || kind == &Type::AssertStatement
                    || kind == &Type::WhileStatement
                    || kind == &Type::SwitchStatement
                    || kind == &Type::ThrowStatement =>
            {
                let i = sync!(i);
                State::None
            }
            (State::None, State::Invocation(i))
                if kind == &Type::ExpressionStatement
                    || kind == &Type::ReturnStatement
                    || kind == &Type::AssertStatement
                    || kind == &Type::WhileStatement
                    || kind == &Type::DoStatement
                    || kind == &Type::SwitchStatement
                    || kind == &Type::ThrowStatement =>
            {
                let i = sync!(i);
                State::None
            }
            (State::None, State::This(i))
                if kind == &Type::ExpressionStatement
                    || kind == &Type::ReturnStatement
                    || kind == &Type::AssertStatement
                    || kind == &Type::WhileStatement
                    || kind == &Type::DoStatement
                    || kind == &Type::SwitchStatement
                    || kind == &Type::ThrowStatement =>
            {
                State::None
            }
            (State::None, State::ScopedIdentifier(i))
                if kind == &Type::IfStatement
                    || kind == &Type::AssertStatement
                    || kind == &Type::ThrowStatement
                    || kind == &Type::WhileStatement
                    || kind == &Type::SwitchStatement =>
            {
                State::None
            }
            (State::None, State::FieldIdentifier(i))
                if kind == &Type::IfStatement
                    || kind == &Type::AssertStatement
                    || kind == &Type::ThrowStatement
                    || kind == &Type::WhileStatement
                    || kind == &Type::SwitchStatement =>
            {
                State::None
            }
            (State::None, State::SimpleIdentifier(i))
                if kind == &Type::LabeledStatement =>
            {
                // TODO label decl that can be ref by break
                State::None
            }
            (State::None, State::None)
                if kind == &Type::LabeledStatement =>
            {
                // TODO label decl that can be ref by break
                State::None
            }
            (State::None, State::SimpleIdentifier(i))
                if kind == &Type::ContinueStatement =>
            {
                // TODO should ref label, but not very usefull relation for impact analysis :)
                State::None
            }
            (State::None, State::SimpleIdentifier(i))
                if kind == &Type::AssertStatement =>
            {
                let i = scoped!(mm!(),i);
                State::None
            }
            (State::None, State::LiteralType(t))
                if kind == &Type::AssertStatement =>
            {
                let t = sync!(t);
                State::None
            }
            (State::None, State::Invocation(i))
                if kind == &Type::IfStatement =>
            {
                State::None
            }
            (State::None, State::SimpleIdentifier(i))
                if kind == &Type::ExpressionStatement || kind == &Type::ReturnStatement =>
            {
                scoped!(mm!(),i);
                State::None
            }







            // EXPRESSIONS
            (State::None, State::SimpleTypeIdentifier(t))
                if kind == &Type::InstanceofExpression =>
            {
                scoped!(mm!(),t);
                State::None
            }
            // CastExpression
            (State::None, expr)
                if kind == &Type::CastExpression =>
            {
                let t = match expr {
                    State::SimpleTypeIdentifier(t) => scoped!(mm!(),t),
                    State::ScopedTypeIdentifier(i) => sync!(i),
                    State::None => panic!("should handle before"),
                    x => panic!("{:?}",x),
                };
                State::ScopedTypeIdentifier(t)
            }
            (State::ScopedTypeIdentifier(t), expr)
                if kind == &Type::CastExpression =>
            {
                let i = match expr {
                    State::LiteralType(t) => sync!(t),
                    State::This(i) => scoped!(mm!(),i),
                    State::SimpleIdentifier(t) => scoped!(mm!(),t),
                    State::SimpleTypeIdentifier(t) => scoped!(mm!(),t), // should not append
                    State::ScopedIdentifier(i) => sync!(i),
                    State::LambdaExpression(i) => sync!(i),
                    State::FieldIdentifier(i) => sync!(i),
                    State::Invocation(i) => sync!(i),
                    State::ConstructorInvocation(i) => sync!(i),
                    State::ScopedTypeIdentifier(i) => panic!(),
                    State::None => panic!("should handle before"),
                    x => panic!("{:?}",x),
                };
                State::ScopedIdentifier(t)
            }
            (State::ScopedIdentifier(t), expr)
                if kind == &Type::CastExpression =>
            {
                // should be ScopedTypeIdentifier but cannot get alias from treesitter rust API cleanly
                let i = match expr {
                    State::LiteralType(t) => sync!(t),
                    State::This(i) => scoped!(mm!(),i),
                    State::SimpleIdentifier(t) => scoped!(mm!(),t),
                    State::SimpleTypeIdentifier(t) => scoped!(mm!(),t), // should not append
                    State::ScopedIdentifier(i) => sync!(i),
                    State::LambdaExpression(i) => sync!(i),
                    State::FieldIdentifier(i) => sync!(i),
                    State::Invocation(i) => sync!(i),
                    State::ConstructorInvocation(i) => sync!(i),
                    State::ScopedTypeIdentifier(i) => panic!(),
                    State::None => panic!("should handle before"),
                    x => panic!("{:?}",x),
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
                let i = scoped!(mm!(),i);
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
                let i = scoped!(mm!(),i);
                State::ScopedIdentifier(i)
            }
            (State::ScopedIdentifier(_), State::LiteralType(t))
                if kind == &Type::AssignmentExpression =>
            {
                let t = sync!(t);
                State::LiteralType(t)
            }
            (
                State::ScopedIdentifier(i),
                State::This(_),
            ) if kind == &Type::AssignmentExpression => State::ScopedIdentifier(i),
            (
                State::ScopedIdentifier(i),
                State::ConstructorInvocation(_),
            ) if kind == &Type::AssignmentExpression => State::ScopedIdentifier(i),
            (
                State::FieldIdentifier(_),
                State::ScopedIdentifier(i),
            ) if kind == &Type::AssignmentExpression => {
                let i = sync!(i);
                State::ScopedIdentifier(i)
            }
            (
                State::ScopedIdentifier(i0),
                State::SimpleIdentifier(i),
            ) if kind == &Type::AssignmentExpression => {
                let i = scoped!(mm!(),i);
                State::ScopedIdentifier(i0)
            }
            (
                State::ScopedIdentifier(i0),
                State::ScopedIdentifier(i),
            ) if kind == &Type::AssignmentExpression => State::ScopedIdentifier(i0),
            (
                State::ScopedIdentifier(i0),
                State::FieldIdentifier(i),
            ) if kind == &Type::AssignmentExpression => State::ScopedIdentifier(i0),
            (
                State::FieldIdentifier(i0),
                State::FieldIdentifier(i),
            ) if kind == &Type::AssignmentExpression => State::FieldIdentifier(i0),
            // TernaryExpression
            // TernaryExpression (None,c)
            (State::None, c) if kind == &Type::TernaryExpression => {
                match c {
                    State::SimpleIdentifier(i) => {
                        let i = scoped!(mm!(),i);
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
                    let i = scoped!(mm!(),i);
                    State::ScopedIdentifier(i)
                }
                State::This(i) => {
                    let i = scoped!(mm!(),i);
                    State::ScopedIdentifier(i)
                }
                State::ConstructorInvocation(i) => {
                    State::ConstructorInvocation(i)
                }
                State::Invocation(i) => State::Invocation(i),
                State::ScopedIdentifier(i) => {
                    State::ScopedIdentifier(i)
                }
                State::FieldIdentifier(i) => {
                    State::FieldIdentifier(i)
                }
                State::None => panic!(),
                _ => todo!(),
            },
            // TernaryExpression (x,y)
            (State::LiteralType(t), y) if kind == &Type::TernaryExpression => {
                match y {
                    State::LiteralType(t) => (),
                    State::SimpleIdentifier(i) => {
                        scoped!(mm!(),i);
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
                        scoped!(mm!(),i);
                    }
                    State::ConstructorInvocation(i) => (),
                    State::Invocation(i) => (),
                    State::ScopedIdentifier(i) => (),
                    State::None => panic!(),
                    _ => todo!(),
                };
                assert_ne!(x, State::Condition);
                let t = sync!(t);
                State::LiteralType(t)
            }
            (State::SimpleIdentifier(i), _) if kind == &Type::TernaryExpression => {
                let i = scoped!(mm!(),i);
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
            (State::FieldIdentifier(_), State::SimpleIdentifier(i))
                if kind == &Type::TernaryExpression =>
            {
                let i = scoped!(mm!(),i);
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
                let i = scoped!(mm!(),i);
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
            (
                State::ConstructorInvocation(i),
                State::ConstructorInvocation(_),
            ) if kind == &Type::TernaryExpression => State::ConstructorInvocation(i),
            (
                State::FieldIdentifier(i),
                State::FieldIdentifier(_),
            ) if kind == &Type::TernaryExpression => State::FieldIdentifier(i),
            (
                State::FieldIdentifier(i),
                State::Invocation(_),
            ) if kind == &Type::TernaryExpression => State::FieldIdentifier(i),
            (
                State::Invocation(_),
                State::ConstructorInvocation(i),
            ) if kind == &Type::TernaryExpression => {
                let i = sync!(i);
                State::ConstructorInvocation(i)
            }
            (
                State::ConstructorInvocation(i),
                State::Invocation(_),
            ) if kind == &Type::TernaryExpression => {
                State::ConstructorInvocation(i)
            }
            (
                State::ConstructorInvocation(t),
                State::SimpleIdentifier(i),
            ) if kind == &Type::TernaryExpression => {
                let i = scoped!(mm!(),i);
                State::ConstructorInvocation(t)
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
                let i = scoped!(mm!(),i);
                State::ScopedIdentifier(i)
            }
            (State::None, State::ScopedIdentifier(i))
                if kind == &Type::ExpressionStatement =>
            {
                State::None
            }
            (State::None, State::SimpleIdentifier(i))
                if kind == &Type::ReturnStatement =>
            {
                scoped!(mm!(),i);
                State::None
            }
            (State::None, State::ScopedIdentifier(i))
                if kind == &Type::ReturnStatement =>
            {
                let i = sync!(i); // not really needed
                State::None
            }
            (State::None, State::LiteralType(_))
                if kind == &Type::ReturnStatement =>
            {
                State::None
            }
            (State::None, State::FieldIdentifier(_))
                if kind == &Type::ReturnStatement =>
            {
                State::None
            }


            (
                State::ScopedIdentifier(il),
                State::SimpleIdentifier(ir),
            ) if kind == &Type::BinaryExpression => {
                scoped!(mm!(),ir);
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
                let t = scoped!(mm!(),t);
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
            (
                State::ScopedIdentifier(i),
                State::ScopedIdentifier(_),
            ) if kind == &Type::BinaryExpression || kind == &Type::AssignmentExpression => {
                State::ScopedIdentifier(i)
            }
            (
                State::LiteralType(t),
                State::SimpleTypeIdentifier(i),
            ) if kind == &Type::BinaryExpression => {
                let i = scoped!(mm!(),i);
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
            (State::This(i), State::Invocation(_))
                if kind == &Type::BinaryExpression =>
            {
                let i = scoped!(mm!(),i);
                State::ScopedIdentifier(i)
            }
            (State::Invocation(i), State::LiteralType(_))
                if kind == &Type::BinaryExpression =>
            {
                State::Invocation(i)
            }
            (State::Invocation(i), State::This(t))
                if kind == &Type::BinaryExpression =>
            {
                let t = scoped!(mm!(),t);
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
                let i = scoped!(mm!(),i);
                State::Invocation(i0)
            }
            (
                State::FieldIdentifier(i0),
                State::SimpleIdentifier(i),
            ) if kind == &Type::BinaryExpression => {
                let i = scoped!(mm!(),i);
                State::FieldIdentifier(i0)
            }
            (
                State::FieldIdentifier(i),
                State::FieldIdentifier(_),
            ) if kind == &Type::BinaryExpression => State::FieldIdentifier(i),





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
            (
                State::FieldIdentifier(i0),
                State::SimpleIdentifier(i),
            ) if kind == &Type::AssignmentExpression => {
                let i = scoped!(mm!(),i);
                State::FieldIdentifier(i0)
            }
            (
                State::FieldIdentifier(_),
                State::ConstructorInvocation(i),
            ) if kind == &Type::AssignmentExpression => {
                let i = sync!(i);
                State::ConstructorInvocation(i)
            }
            (State::FieldIdentifier(_), State::Invocation(i))
                if kind == &Type::AssignmentExpression =>
            {
                let i = sync!(i);
                State::ConstructorInvocation(i)
            }
            (State::None, State::FieldIdentifier(i))
                if kind == &Type::ExpressionStatement =>
            {
                State::None
            }
            (
                State::Invocation(_),
                State::SimpleTypeIdentifier(i),
            ) if kind == &Type::InstanceofExpression => {
                let i = scoped!(mm!(),i);
                // TODO intern boolean
                State::ScopedIdentifier(mm!())
            }
            (
                State::ScopedIdentifier(_),
                State::SimpleTypeIdentifier(i),
            ) if kind == &Type::InstanceofExpression => {
                let i = scoped!(mm!(),i);
                // TODO intern boolean
                State::ScopedIdentifier(mm!())
            }
            (
                State::ScopedIdentifier(_),
                State::ScopedTypeIdentifier(i),
            ) if kind == &Type::InstanceofExpression => {
                // TODO intern boolean
                State::ScopedIdentifier(mm!())
            }
            (State::None, State::ScopedIdentifier(i))
                if kind == &Type::BinaryExpression || kind == &Type::AssignmentExpression =>
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
                if kind == &Type::BinaryExpression || kind == &Type::AssignmentExpression =>
            {
                State::ScopedIdentifier(i)
            }
            (
                State::ScopedIdentifier(i),
                State::FieldIdentifier(_),
            ) if kind == &Type::BinaryExpression => State::ScopedIdentifier(i),
            (State::None, State::ScopedIdentifier(i))
                if kind == &Type::ExpressionStatement =>
            {
                State::None
            }
            (State::None, State::SimpleIdentifier(i))
                if kind == &Type::SwitchLabel =>
            {
                // TODO link label to switch expr
                State::None
            }
            (State::None, State::LiteralType(_))
                if kind == &Type::SwitchLabel =>
            {
                State::None
            }
            (State::None, State::FieldIdentifier(_))
                if kind == &Type::SwitchLabel =>
            {
                State::None
            }
            (State::None, State::ScopedIdentifier(_))
                if kind == &Type::SwitchLabel =>
            {
                State::None
            }
            (State::None, State::SimpleIdentifier(i))
                if kind == &Type::MarkerAnnotation =>
            {
                let i = scoped!(mm!(),i);
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
            (State::None, State::SimpleIdentifier(i))
                if kind == &Type::Annotation =>
            {
                let i = scoped!(mm!(),i);
                State::ScopedIdentifier(i)
            }
            (State::None, State::ScopedIdentifier(i))
                if kind == &Type::Annotation =>
            {
                let i = sync!(i);
                State::ScopedIdentifier(i)
            }
            (State::SimpleIdentifier(i), State::Arguments(p))
                if kind == &Type::Annotation =>
            {
                let i = scoped!(mm!(),i);
                State::Annotation
            }
            (State::ScopedIdentifier(i), State::Arguments(p))
                if kind == &Type::Annotation =>
            {
                State::Annotation
            }
            (State::None, State::SimpleIdentifier(i))
                if kind == &Type::ElementValuePair =>
            {
                State::SimpleIdentifier(i)
            }
            (State::SimpleIdentifier(i), State::None)
                if kind == &Type::ElementValuePair =>
            {
                let i = scoped!(mm!(),i);
                State::ScopedIdentifier(i)
            }
            (State::SimpleIdentifier(p), State::LiteralType(t))
                if kind == &Type::ElementValuePair =>
            {
                let t = sync!(t);
                State::ElementValuePair(p, t)
            }
            (
                State::SimpleIdentifier(p),
                State::FieldIdentifier(i),
            ) if kind == &Type::ElementValuePair => {
                let i = sync!(i);
                State::ElementValuePair(p, i)
            }
            // TODO fusion AnnotationArgumentList cases
            (rest, State::ElementValuePair(p, i))
                if kind == &Type::AnnotationArgumentList =>
            {
                let mut v = match rest {
                    State::Arguments(l) => l,
                    State::None => vec![],
                    x => panic!("{:?}", x),
                };
                let i = sync!(i);
                State::Arguments(v)
            }
            (rest, State::LiteralType(_))
                if kind == &Type::AnnotationArgumentList =>
            {
                let mut v = match rest {
                    State::Arguments(l) => l,
                    State::None => vec![],
                    x => panic!("{:?}", x),
                };
                State::Arguments(v)
            }
            (rest, State::FieldIdentifier(i))
                if kind == &Type::AnnotationArgumentList =>
            {
                let mut v = match rest {
                    State::Arguments(l) => l,
                    State::None => vec![],
                    x => panic!("{:?}", x),
                };
                State::Arguments(v)
            }
            (rest, State::ScopedIdentifier(i))
                if kind == &Type::AnnotationArgumentList =>
            {
                let mut v = match rest {
                    State::Arguments(l) => l,
                    State::None => vec![],
                    x => panic!("{:?}", x),
                };
                State::Arguments(v)
            }
            (rest, State::None)
                if kind == &Type::AnnotationArgumentList =>
            {
                // TODO should not be None but the value of ElementValueArrayInitializer
                let mut v = match rest {
                    State::Arguments(l) => l,
                    State::None => vec![],
                    x => panic!("{:?}", x),
                };
                State::Arguments(v)
            }
            (State::None, State::SimpleTypeIdentifier(i))
                if kind == &Type::ScopedTypeIdentifier =>
            {
                let i = scoped!(mm!(),i);
                State::ScopedTypeIdentifier(i)
            }
            (State::None, State::ScopedTypeIdentifier(i))
                if kind == &Type::ScopedTypeIdentifier =>
            {
                let i = sync!(i);
                State::ScopedTypeIdentifier(i)
            }
            (
                State::ScopedTypeIdentifier(o),
                State::SimpleTypeIdentifier(i),
            ) if kind == &Type::ScopedTypeIdentifier => {
                let i = scoped!(o,i);
                State::ScopedTypeIdentifier(i)
            }
            (State::None, State::SimpleTypeIdentifier(i))
                if kind == &Type::ArrayType =>
            {
                let i = scoped!(mm!(),i);
                State::ScopedTypeIdentifier(i)
            }
            (State::None, State::ScopedTypeIdentifier(i))
                if kind == &Type::ArrayType =>
            {
                let i = sync!(i);
                State::ScopedTypeIdentifier(i)
            }
            (State::SimpleTypeIdentifier(i), State::Dimensions)
                if kind == &Type::ArrayType =>
            {
                let i = scoped!(mm!(),i);
                State::ScopedTypeIdentifier(i)
            }
            (State::ScopedTypeIdentifier(i), State::Dimensions)
                if kind == &Type::ArrayType =>
            {
                State::ScopedTypeIdentifier(i)
            }
            (x, y) => todo!("{:?} {:?} {:?}", kind, x, y),
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
    Super(Leaf),
    This(Leaf),
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
        content: Vec<(Declarator<Node>, Node)>,
    },
    /// b.f() or A.f()
    Invocation(Node),
    InvocationId(Node, Leaf),
    MethodReference(Node),
    LambdaExpression(Node),
    Arguments(Vec<Node>),
    /// A#constructor()
    ConstructorInvocation(Node),
    ImportDeclaration(Node),
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

    fn with_changed_node<F: FnOnce(&Node) -> Node>(&self, f: F) -> Self {
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
    Leaf: std::cmp::Eq + std::hash::Hash,
    Node: std::cmp::Eq + std::hash::Hash,
{
    pub fn take(&mut self) -> Self {
        std::mem::replace(self, State::None)
    }
}
