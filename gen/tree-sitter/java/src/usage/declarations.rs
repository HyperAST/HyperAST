use core::fmt;
use std::fmt::Debug;

use hyperast::{
    position::{TreePath, TreePathMut},
    store::defaults::NodeIdentifier,
    types::{
        AnyType, Children, HyperAST, HyperType, NodeId, NodeStore, Tree, TypeTrait, Typed,
        TypedHyperAST, TypedNodeStore, TypedTree, WithChildren, AAAA,
    },
};
use num::ToPrimitive;

use crate::types::Type;

pub struct IterDeclarations<'a, T, HAST> {
    stores: &'a HAST,
    path: T,
    stack: Vec<(Id<NodeIdentifier>, u16, Option<Vec<NodeIdentifier>>)>,
}

enum Id<IdN> {
    Java(crate::types::TIdN<IdN>),
    Other(IdN),
}

impl<IdN: Clone + Eq + AAAA> Id<IdN> {
    fn id(&self) -> &IdN {
        match self {
            Id::Java(node) => node.as_id(),
            Id::Other(node) => node,
        }
    }
}

impl<'a, T: TreePath<NodeIdentifier, u16>, HAST> Debug for IterDeclarations<'a, T, HAST> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IterDeclarations2")
            // .field("parents", &self.parents())
            .finish()
    }
}

impl<'a, T, HAST> Iterator for IterDeclarations<'a, T, HAST>
where
    // T: TreePathMut<NodeIdentifier, u16> + Clone + Debug,
    // HAST: TypedHyperAST<crate::types::TIdN<NodeIdentifier>, IdN = NodeIdentifier, Idx = u16>,
    // HAST::TS: JavaEnabledTypeStore<HAST::T>,
    // for<'t> <HAST::TT<'t> as Typed>::Type: Copy + Send + Sync,
    // HAST::NS: TypedNodeStore<crate::types::TIdN<NodeIdentifier>>,
    // HAST::NS: TypedNodeStore<crate::types::TIdN<HAST::IdN>>,
    // for<'b> <HAST::NS as hyperast::types::TyNodeStore<crate::types::TIdN<HAST::IdN>>>::R<'b>:
    //     TypedTree<Type = Type, TreeId = HAST::IdN, Label = HAST::Label, ChildIdx = u16>,
    // // for<'t> <HAST as hyperast::types::AstLending<'t, HAST::IdN>>::RT:
    // // // <HAST::NS as hyperast::types::NodStore<HAST::IdN>>::R<'a>:
    // //     TypedTree<Type = AnyType, TreeId = HAST::IdN, Label = HAST::Label, ChildIdx = u16>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
        // loop {
        //     let (node, offset, children) = self.stack.pop()?;
        //     if let Some(children) = children {
        //         if offset.to_usize().unwrap() < children.len() {
        //             let child = children[offset.to_usize().unwrap()];
        //             self.path.check(self.stores).unwrap();
        //             {
        //                 let b = hyperast::types::NodeStore::resolve(
        //                     self.stores.node_store(),
        //                     node.id(),
        //                 );
        //                 if b.has_children() {
        //                     assert!(offset < b.child_count());
        //                     let cs = b.children();
        //                     // println!("children: {:?} {} {:?}", node,cs.len(),cs);
        //                     assert_eq!(child, cs.unwrap()[num::cast(offset).unwrap()]);
        //                 } else {
        //                     panic!()
        //                 }
        //             }
        //             if offset == 0 {
        //                 match self.path.node() {
        //                     Some(x) => assert_eq!(x, node.id()),
        //                     None => {}
        //                 }
        //                 self.path.goto(child, offset);
        //                 self.path.check(self.stores).unwrap();
        //             } else {
        //                 match self.path.node() {
        //                     Some(x) => assert_eq!(*x, children[offset.to_usize().unwrap() - 1]),
        //                     None => {}
        //                 }
        //                 self.path.inc(child);
        //                 assert_eq!(*self.path.offset().unwrap(), offset + 1);
        //                 // self.scout.check_size(&self.stores).expect(&format!(
        //                 //     "{:?} {} {:?} {:?} {:?}",
        //                 //     node,
        //                 //     offset,
        //                 //     child,
        //                 //     children.len(),
        //                 //     self.scout
        //                 // ));
        //                 self.path.check(self.stores).expect(&format!(
        //                     "{:?} {} {:?} {:?} {:?}",
        //                     node.id(),
        //                     offset,
        //                     child,
        //                     children,
        //                     self.path
        //                 ));
        //             }
        //             self.stack.push((node, offset + 1, Some(children)));
        //             let child = if let Some(tid) =
        //                 TypedNodeStore::try_typed(self.stores.node_store(), &child)
        //             {
        //                 Id::Java(tid)
        //             } else {
        //                 Id::Other(child)
        //             };
        //             self.stack.push((child, 0, None));
        //             continue;
        //         } else {
        //             self.path.check(self.stores).unwrap();
        //             self.path.pop().expect("should not go higher than root");
        //             self.path.check(self.stores).unwrap();
        //             continue;
        //         }
        //     } else {
        //         let b = match &node {
        //             Id::Java(node) => TypedNodeStore::resolve(self.stores.node_store(), node),
        //             Id::Other(node) => {
        //                 let b = hyperast::types::NodeStore::resolve(self.stores.node_store(), node);
        //                 if b.has_children() {
        //                     let children = b.children();
        //                     let children = children.unwrap();
        //                     self.stack.push((
        //                         Id::Other(*node),
        //                         0,
        //                         Some(children.iter_children().cloned().collect()),
        //                     ));
        //                 }
        //                 continue;
        //             }
        //         };
        //         let t = b.get_type();
        //         // let t = self.stores.type_store().resolve(t);

        //         if t.is_spaces() {
        //             continue;
        //         } else if t.is_comment() {
        //             continue;
        //         } else if t == Type::PackageDeclaration {
        //             continue;
        //         } else if t == Type::ImportDeclaration {
        //             continue;
        //         } else if t == Type::Identifier {
        //             let mut p = self.path.clone();
        //             p.pop();
        //             let p = p.node().unwrap();
        //             let Id::Java(x) = &self.stack.last().unwrap().0 else {
        //                 continue;
        //             };
        //             assert_eq!(p, x.as_id());
        //             let b = TypedNodeStore::resolve(self.stores.node_store(), x);
        //             let tt = b.get_type();
        //             // let tt = self.stores.type_store().resolve(tt);
        //             if self.path.offset() == Some(&1) && tt == Type::LambdaExpression {
        //                 self.path.check(self.stores).unwrap();
        //                 return Some(self.path.clone());
        //             } else if tt == Type::InferredParameters {
        //                 self.path.check(self.stores).unwrap();
        //                 return Some(self.path.clone());
        //             }
        //             continue;
        //         }

        //         if b.has_children() {
        //             let children = b.children();
        //             let children = children.unwrap();
        //             self.stack
        //                 .push((node, 0, Some(children.iter_children().cloned().collect())));
        //         }

        //         if t.is_type_declaration() || t.is_parameter() {
        //             assert!(b.has_children(), "{:?}", t);
        //             self.path.check(self.stores).unwrap();
        //             return Some(self.path.clone());
        //         } else if t == Type::LocalVariableDeclaration
        //             // || t == Type::EnhancedForVariable // TODO trick to group nodes semantically
        //             || t == Type::CatchFormalParameter
        //         {
        //             assert!(b.has_children(), "{:?}", t);
        //             self.path.check(self.stores).unwrap();
        //             return Some(self.path.clone());
        //         } else if t == Type::TypeParameter {
        //             assert!(b.has_children(), "{:?}", t);
        //             self.path.check(self.stores).unwrap();
        //             return Some(self.path.clone());
        //         } else if t == Type::ClassBody {
        //             let mut p = self.path.clone();
        //             p.pop();
        //             let p = p.node().unwrap();
        //             let Id::Java(x) = &self.stack.last().unwrap().0 else {
        //                 continue;
        //             };
        //             assert_eq!(p, x.as_id());
        //             let b = TypedNodeStore::resolve(self.stores.node_store(), x);
        //             let tt = b.get_type();
        //             if tt == Type::ObjectCreationExpression {
        //                 self.path.check(self.stores).unwrap();
        //                 return Some(self.path.clone());
        //             } else if tt == Type::EnumDeclaration {
        //                 self.path.check(self.stores).unwrap();
        //                 return Some(self.path.clone());
        //             }
        //         } else if t == Type::Resource {
        //             assert!(b.has_children(), "{:?}", t);
        //             self.path.check(self.stores).unwrap();
        //             // TODO also need to find an "=" and find the name just before
        //             let cs = b.children().unwrap();
        //             for xx in cs.iter_children() {
        //                 let bb = TypedNodeStore::try_resolve(self.stores.node_store(), xx);
        //                 let Some((bb, _)) = bb else {
        //                     continue;
        //                 };
        //                 // let bb = self.stores.node_store().resolve(xx);
        //                 if bb.get_type() == Type::GT {
        //                     return Some(self.path.clone());
        //                 }
        //             }
        //         // } else if t.is_value_member()
        //         // {
        //         //     assert!(b.has_children(), "{:?}", t);
        //         //     self.path.check(&self.stores).unwrap();
        //         //     return Some(self.path.clone());
        //         // } else if t.is_executable_member()
        //         // {
        //         //     assert!(b.has_children(), "{:?}", t);
        //         //     self.path.check(&self.stores).unwrap();
        //         //     return Some(self.path.clone());
        //         } else {
        //         }
        //     }
        // }
    }
}

impl<'a, T: TreePath<NodeIdentifier, u16>, HAST: HyperAST<IdN = NodeIdentifier>>
    IterDeclarations<'a, T, HAST>
where
    HAST::NS: TypedNodeStore<crate::types::TIdN<HAST::IdN>>,
{
    pub fn new(stores: &'a HAST, path: T, root: NodeIdentifier) -> Self {
        let root = if let Some(tid) = TypedNodeStore::try_typed(stores.node_store(), &root) {
            Id::Java(tid)
        } else {
            Id::Other(root)
        };
        let stack = vec![(root, 0, None)];
        Self {
            stores,
            path,
            stack,
        }
    }
}

#[cfg(test)]
mod experiment {
    use super::*;
    use hyperast::position::StructuralPosition;
    use std::ops::AddAssign;

    pub struct IterDeclarationsUnstableOpti<'a, HAST> {
        stores: &'a HAST,
        parents: Vec<NodeIdentifier>,
        offsets: Vec<u16>,
        /// to tell that we need to pop a parent, we could also use a bitvec instead of Option::None
        remaining: Vec<Option<NodeIdentifier>>,
    }

    impl<'a, HAST> Debug for IterDeclarationsUnstableOpti<'a, HAST> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("IterDeclarations")
                .field("parents", &self.parents())
                .field("offsets", &self.offsets())
                .field("remaining", &self.remaining)
                .finish()
        }
    }

    impl<'a, HAST: HyperAST> Iterator for IterDeclarationsUnstableOpti<'a, HAST> {
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
            todo!()

            // let b = self.stores.node_store().resolve(&x);
            // let t = b.get_type();

            // if t == Type::Spaces {
            //     return self.next();
            // } else if t == Type::Comment {
            //     return self.next();
            // } else if t == Type::PackageDeclaration {
            //     return self.next();
            // } else if t == Type::ImportDeclaration {
            //     return self.next();
            // }

            // self.parents.push(x);
            // self.offsets.push(0);
            // self.remaining.push(None);
            // if let Some(cs) = b.children() {
            //     self.remaining
            //         .extend(cs.iter_children().rev().map(|x| Some(*x)));
            // }

            // if t.is_type_declaration() {
            //     Some(x)
            // } else if t == Type::LocalVariableDeclaration {
            //     Some(x)
            // } else if t == Type::EnhancedForStatement {
            //     Some(x)
            // } else if t == Type::Resource {
            //     // TODO also need to find an "=" and find the name just before
            //     Some(x)
            // } else if t.is_value_member() {
            //     Some(x)
            // } else if t.is_parameter() {
            //     Some(x)
            // } else if t.is_executable_member() {
            //     Some(x)
            // } else {
            //     while !self.remaining.is_empty() {
            //         if let Some(x) = self.next() {
            //             return Some(x);
            //         }
            //     }
            //     None
            // }
        }
    }

    impl<'a, HAST> IterDeclarationsUnstableOpti<'a, HAST> {
        pub fn new(stores: &'a HAST, root: NodeIdentifier) -> Self {
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
        pub fn offsets(&self) -> &[u16] {
            &self.offsets[..self.offsets.len() - 1]
        }
        pub fn position(&self, x: NodeIdentifier) -> StructuralPosition<NodeIdentifier, u16> {
            (self.parents().to_vec(), self.offsets().to_vec(), x).into()
        }
    }
}
