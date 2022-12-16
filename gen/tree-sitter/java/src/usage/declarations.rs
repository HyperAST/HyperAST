use core::fmt;
use std::{fmt::Debug, ops::AddAssign};

use hyper_ast::{
    position::{StructuralPosition, TreePath},
    store::{defaults::NodeIdentifier, SimpleStores},
    types::{Tree, Type, Typed, WithChildren, IterableChildren},
};

pub struct IterDeclarations<'a, T: TreePath<NodeIdentifier>> {
    stores: &'a SimpleStores,
    path: T,
    stack: Vec<(NodeIdentifier, usize, Option<Vec<NodeIdentifier>>)>,
}

impl<'a, T: TreePath<NodeIdentifier>> Debug for IterDeclarations<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IterDeclarations2")
            // .field("parents", &self.parents())
            .finish()
    }
}

impl<'a, T: TreePath<NodeIdentifier> + Clone + Debug> Iterator for IterDeclarations<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (node, offset, children) = self.stack.pop()?;
            if let Some(children) = children {
                if offset < children.len() {
                    let child = children[offset];
                    self.path.check(&self.stores).unwrap();
                    {
                        let b = self.stores.node_store.resolve(node);
                        if b.has_children() {
                            assert!(offset < b.child_count().into());
                            let cs = b.children();
                            // println!("children: {:?} {} {:?}", node,cs.len(),cs);
                            assert_eq!(child, cs.unwrap()[offset]);
                        } else {
                            panic!()
                        }
                    }
                    if offset == 0 {
                        match self.path.node() {
                            Some(x) => assert_eq!(*x, node),
                            None => {}
                        }
                        self.path.goto(child, offset);
                        self.path.check(&self.stores).unwrap();
                    } else {
                        match self.path.node() {
                            Some(x) => assert_eq!(*x, children[offset - 1]),
                            None => {}
                        }
                        self.path.inc(child);
                        assert_eq!(*self.path.offset().unwrap(), offset + 1);
                        // self.scout.check_size(&self.stores).expect(&format!(
                        //     "{:?} {} {:?} {:?} {:?}",
                        //     node,
                        //     offset,
                        //     child,
                        //     children.len(),
                        //     self.scout
                        // ));
                        self.path.check(&self.stores).expect(&format!(
                            "{:?} {} {:?} {:?} {:?}",
                            node, offset, child, children, self.path
                        ));
                    }
                    self.stack.push((node, offset + 1, Some(children)));
                    self.stack.push((child, 0, None));
                    continue;
                } else {
                    self.path.check(&self.stores).unwrap();
                    self.path.pop().expect("should not go higher than root");
                    self.path.check(&self.stores).unwrap();
                    continue;
                }
            } else {
                let b = self.stores.node_store.resolve(node);
                let t = b.get_type();

                if t == Type::Spaces {
                    continue;
                } else if t == Type::Comment {
                    continue;
                } else if t == Type::PackageDeclaration {
                    continue;
                } else if t == Type::ImportDeclaration {
                    continue;
                } else if t == Type::Identifier {
                    let mut p = self.path.clone();
                    p.pop();
                    let x = p.node().unwrap();
                    let b = self.stores.node_store.resolve(*x);
                    let tt = b.get_type();
                    if self.path.offset() == Some(&1) && tt == Type::LambdaExpression {
                        self.path.check(&self.stores).unwrap();
                        return Some(self.path.clone());
                    } else if tt == Type::InferredParameters {
                        self.path.check(&self.stores).unwrap();
                        return Some(self.path.clone());
                    }
                    continue;
                }

                if b.has_children() {
                    let children = b.children();
                    let children = children.unwrap();
                    self.stack.push((node, 0, Some(children.iter_children().cloned().collect())));
                }

                if t.is_type_declaration() || t.is_parameter() {
                    assert!(b.has_children(), "{:?}", t);
                    self.path.check(&self.stores).unwrap();
                    return Some(self.path.clone());
                } else if t == Type::LocalVariableDeclaration
                    || t == Type::EnhancedForVariable
                    || t == Type::CatchFormalParameter
                {
                    assert!(b.has_children(), "{:?}", t);
                    self.path.check(&self.stores).unwrap();
                    return Some(self.path.clone());
                } else if t == Type::TypeParameter {
                    assert!(b.has_children(), "{:?}", t);
                    self.path.check(&self.stores).unwrap();
                    return Some(self.path.clone());
                } else if t == Type::ClassBody {
                    let mut p = self.path.clone();
                    p.pop();
                    let x = p.node().unwrap();
                    let b = self.stores.node_store.resolve(*x);
                    let tt = b.get_type();
                    if tt == Type::ObjectCreationExpression {
                        self.path.check(&self.stores).unwrap();
                        return Some(self.path.clone());
                    } else if tt == Type::EnumDeclaration {
                        self.path.check(&self.stores).unwrap();
                        return Some(self.path.clone());
                    }
                } else if t == Type::Resource {
                    assert!(b.has_children(), "{:?}", t);
                    self.path.check(&self.stores).unwrap();
                    // TODO also need to find an "=" and find the name just before
                    let cs = b.children().unwrap();
                    for xx in cs.iter_children() {
                        let bb = self.stores.node_store.resolve(*xx);
                        if bb.get_type() == Type::TS30 {
                            return Some(self.path.clone());
                        }
                    }
                // } else if t.is_value_member()
                // {
                //     assert!(b.has_children(), "{:?}", t);
                //     self.path.check(&self.stores).unwrap();
                //     return Some(self.path.clone());
                // } else if t.is_executable_member()
                // {
                //     assert!(b.has_children(), "{:?}", t);
                //     self.path.check(&self.stores).unwrap();
                //     return Some(self.path.clone());
                } else {
                }
            }
        }
    }
}

impl<'a, T: TreePath<NodeIdentifier>> IterDeclarations<'a, T> {
    pub fn new(stores: &'a SimpleStores, path: T, root: NodeIdentifier) -> Self {
        let stack = vec![(root, 0, None)];
        Self {
            stores,
            path,
            stack,
        }
    }
}

pub struct IterDeclarationsUnstableOpti<'a> {
    stores: &'a SimpleStores,
    parents: Vec<NodeIdentifier>,
    offsets: Vec<usize>,
    /// to tell that we need to pop a parent, we could also use a bitvec instead of Option::None
    remaining: Vec<Option<NodeIdentifier>>,
}

impl<'a> Debug for IterDeclarationsUnstableOpti<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IterDeclarations")
            .field("parents", &self.parents())
            .field("offsets", &self.offsets())
            .field("remaining", &self.remaining)
            .finish()
    }
}

impl<'a> Iterator for IterDeclarationsUnstableOpti<'a> {
    type Item = NodeIdentifier;

    fn next(&mut self) -> Option<Self::Item> {
        let x;
        loop {
            if let Some(c) = self.remaining.pop()? {
                self.offsets.last_mut().unwrap().add_assign(1);
                x = c;
                break;
            } else {
                self.offsets.pop();
                self.parents.pop();
            }
        }

        let b = self.stores.node_store.resolve(x);
        let t = b.get_type();

        if t == Type::Spaces {
            return self.next();
        } else if t == Type::Comment {
            return self.next();
        } else if t == Type::PackageDeclaration {
            return self.next();
        } else if t == Type::ImportDeclaration {
            return self.next();
        }

        self.parents.push(x);
        self.offsets.push(0);
        self.remaining.push(None);
        if let Some(cs) = b.children() {
            self.remaining
                .extend(cs.iter_children().rev().map(|x| Some(*x)));
        }

        if t.is_type_declaration() {
            Some(x)
        } else if t == Type::LocalVariableDeclaration {
            Some(x)
        } else if t == Type::EnhancedForStatement {
            Some(x)
        } else if t == Type::Resource {
            // TODO also need to find an "=" and find the name just before
            Some(x)
        } else if t.is_value_member() {
            Some(x)
        } else if t.is_parameter() {
            Some(x)
        } else if t.is_executable_member() {
            Some(x)
        } else {
            while !self.remaining.is_empty() {
                if let Some(x) = self.next() {
                    return Some(x);
                }
            }
            None
        }
    }
}

impl<'a> IterDeclarationsUnstableOpti<'a> {
    pub fn new(stores: &'a SimpleStores, root: NodeIdentifier) -> Self {
        Self {
            stores,
            parents: vec![],
            offsets: vec![0],
            remaining: vec![Some(root)],
        }
    }
    pub fn parents(&self) -> &[NodeIdentifier] {
        &self.parents[..self.parents.len() - 1]
    }
    pub fn offsets(&self) -> &[usize] {
        &self.offsets[..self.offsets.len() - 1]
    }
    pub fn position(&self, x: NodeIdentifier) -> StructuralPosition {
        (self.parents().to_vec(), self.offsets().to_vec(), x).into()
    }
}
