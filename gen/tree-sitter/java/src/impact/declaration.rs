use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
};

use super::{
    element::{ExplorableRef, Nodes, RawLabelPtr, RefPtr},
    label_value::LabelValue,
    reference::DisplayRef,
};

use hyper_ast::types::LabelStore;

pub struct ExplorableDecl<'a> {
    decl: (&'a Declarator<RefPtr>, &'a DeclType<RefPtr>),
    nodes: &'a Nodes,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclType<Node> {
    /// Typically erased types but also difficult compile time resolutions
    Runtime(Box<[Node]>),
    Compile(Node, Box<[Node]>, Box<[Node]>),
}

impl<Node> Declarator<Node>
where
    Node: Eq + Hash,
{
    pub fn node(&self) -> Option<&Node> {
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

    pub fn with_changed_node<N, F: FnOnce(&Node) -> N>(&self, f: F) -> Declarator<N>
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

impl<Node: Clone> DeclType<Node> {
    pub fn map<N, FN: FnMut(&Node) -> N>(&self, mut f: FN) -> DeclType<N>
where {
        match self {
            DeclType::Runtime(b) => DeclType::Runtime(b.iter().map(f).collect()),
            DeclType::Compile(t, s, i) => DeclType::Compile(
                f(t),
                s.iter().map(&mut f).collect(),
                i.iter().map(&mut f).collect(),
            ),
        }
    }
}

pub struct DeclsIter<'a> {
    pub(crate) decls: std::collections::hash_map::Iter<'a, Declarator<usize>, DeclType<usize>>,
    pub(crate) nodes: &'a Nodes,
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

pub struct DebugDecls<'a> {
    pub(crate) decls: &'a HashMap<Declarator<usize>, DeclType<usize>>,
    pub(crate) nodes: &'a Nodes,
}

impl<'a> Debug for DebugDecls<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, v) in self.decls {
            let kr = match k.node() {
                None => ExplorableRef {
                    rf: 0,
                    nodes: self.nodes,
                },
                Some(rf) => ExplorableRef {
                    rf: *rf,
                    nodes: self.nodes,
                },
            };
            match v {
                DeclType::Runtime(b) => {
                    // TODO print more things
                    writeln!(f, "   {:?}: {:?} =>", k, kr)?;
                    for v in b.iter() {
                        let r = ExplorableRef {
                            rf: *v,
                            nodes: self.nodes,
                        };
                        write!(f, " ({:?}) {:?}", *v, r)?;
                    }
                    writeln!(f)?;
                }
                DeclType::Compile(v, s, b) => {
                    // TODO print more things
                    let r = ExplorableRef {
                        rf: *v,
                        nodes: self.nodes,
                    };
                    write!(f, "   {:?}: {:?} => {:?}", k, kr, r)?;
                    if s.len() > 0 {
                        write!(f, " extends")?;
                    }
                    for v in s.iter() {
                        let v = ExplorableRef {
                            rf: *v,
                            nodes: self.nodes,
                        };
                        write!(f, " {:?},", v)?;
                    }
                    if b.len() > 0 {
                        write!(f, " implements")?;
                    }
                    for v in b.iter() {
                        let v = ExplorableRef {
                            rf: *v,
                            nodes: self.nodes,
                        };
                        write!(f, " {:?},", v)?;
                    }
                    writeln!(f)?;
                }
            }
        }
        Ok(())
    }
}

pub struct DisplayDecl<'a, 'b, LS: LabelStore<str>> {
    decl: (&'a Declarator<RefPtr>, &'a DeclType<RefPtr>),
    nodes: &'a Nodes,
    leafs: &'b LS,
}

impl<'a, 'b, LS: LabelStore<str>> DisplayDecl<'a, 'b, LS> {
    pub fn with(&self, decl: (&'a Declarator<RefPtr>, &'a DeclType<RefPtr>)) -> Self {
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

impl<'a, 'b, LS: LabelStore<str, I = RawLabelPtr>> Display for DisplayDecl<'a, 'b, LS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (k, v) = &self.decl;
        let kr = match k.node() {
            None => 0,
            Some(rf) => *rf,
        };
        let kr: DisplayRef<LS> = (self.nodes.with(kr), self.leafs).into();
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
                if s.len() > 0 {
                    write!(f, " extends")?;
                }
                for v in s.iter() {
                    let v: DisplayRef<LS> = (self.nodes.with(*v), self.leafs).into();
                    write!(f, " {},", v)?;
                }
                if b.len() > 0 {
                    write!(f, " implements")?;
                }
                for v in b.iter() {
                    let v: DisplayRef<LS> = (self.nodes.with(*v), self.leafs).into();
                    write!(f, " {},", v)?;
                }
                Ok(())
            }
        }
    }
}
