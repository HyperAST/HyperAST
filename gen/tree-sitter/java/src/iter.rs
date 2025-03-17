use std::fmt::{self, Debug};

use hyperast::types::{TypedHyperAST, AAAA};
use hyperast::{
    position::{TreePath, TreePathMut},
    store::nodes::legion::NodeIdentifier,
    types::{HyperAST, NodeId, Tree, TypedNodeStore, WithChildren, Childrn},
};
use num::ToPrimitive;

use crate::types::TIdN;

pub struct IterAll<'a, T, HAST> {
    stores: &'a HAST,
    path: T,
    stack: Vec<(Id<NodeIdentifier>, u16, Option<Vec<NodeIdentifier>>)>,
}

enum Id<IdN> {
    Java(TIdN<IdN>),
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

impl<'a, T: TreePath<NodeIdentifier, u16>, HAST> Debug for IterAll<'a, T, HAST> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IterAllNodes")
            // .field("parents", &self.parents())
            .finish()
    }
}

impl<'a, T: TreePath<NodeIdentifier, u16>, HAST: HyperAST<IdN = NodeIdentifier>> From<(&'a HAST, T)>
    for IterAll<'a, T, HAST>
where
    HAST::NS: TypedNodeStore<TIdN<HAST::IdN>>,
{
    fn from((stores, path): (&'a HAST, T)) -> Self {
        let root = *path.node().unwrap();
        Self::new(stores, path, root)
    }
}

impl<'a, T: TreePath<NodeIdentifier, u16>, HAST: HyperAST<IdN = NodeIdentifier>>
    IterAll<'a, T, HAST>
where
    HAST::NS: TypedNodeStore<TIdN<HAST::IdN>>,
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

impl<
        'a,
        T: TreePathMut<NodeIdentifier, u16> + Clone + Debug,
        HAST: TypedHyperAST<TIdN<NodeIdentifier>, Idx = u16>,
    > Iterator for IterAll<'a, T, HAST>
where
// HAST::NS: TypedNodeStore<TIdN<NodeIdentifier>>,
// HAST::TS: TypeStore<HAST::T, Ty = Type>,
// HAST::TT: TypedTree<Type = Type>,
// <HAST::T as Typed>::Type: Copy + Send + Sync,
// for<'b> <HAST::NS as TypedNodeStore<TIdN<HAST::IdN>>>::R<'b>:
//     TypedTree<Type = Type, TreeId = HAST::IdN, Label = HAST::Label, ChildIdx = u16>,
// <HAST::NS as NodeStore<HAST::IdN>>::R<'a>:
//     TypedTree<Type = AnyType, TreeId = HAST::IdN, Label = HAST::Label, ChildIdx = u16>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (node, offset, children) = self.stack.pop()?;
            if let Some(children) = children {
                if offset.to_usize().unwrap() < children.len() {
                    let child = children[offset.to_usize().unwrap()];
                    self.path.check(self.stores).unwrap();
                    {
                        let b = hyperast::types::NodeStore::resolve(
                            self.stores.node_store(),
                            node.id(),
                        );
                        if b.has_children() {
                            assert!(offset < b.child_count());
                            let cs = b.children();
                            assert_eq!(child, cs.unwrap()[num::cast(offset).unwrap()]);
                        } else {
                            panic!()
                        }
                    }
                    if offset == 0 {
                        match self.path.node() {
                            Some(x) => assert_eq!(x, node.id()),
                            None => {}
                        }
                        self.path.goto(child, offset);
                        self.path.check(self.stores).unwrap();
                    } else {
                        match self.path.node() {
                            Some(x) => assert_eq!(*x, children[offset.to_usize().unwrap() - 1]),
                            None => {}
                        }
                        self.path.inc(child);
                        assert_eq!(*self.path.offset().unwrap(), offset + 1);
                        self.path.check(self.stores).expect(&format!(
                            "{:?} {} {:?} {:?} {:?}",
                            node.id(),
                            offset,
                            child,
                            children,
                            self.path
                        ));
                    }
                    self.stack.push((node, offset + 1, Some(children)));
                    let child = if let Some(tid) = self.stores.try_typed(&child)
                    {
                        Id::Java(tid)
                    } else {
                        Id::Other(child)
                    };
                    self.stack.push((child, 0, None));
                    continue;
                } else {
                    self.path.check(self.stores).unwrap();
                    self.path.pop().expect("should not go higher than root");
                    self.path.check(self.stores).unwrap();
                    continue;
                }
            } else {
                let b = match &node {
                    Id::Java(node) => self.stores.resolve_typed(node),
                    Id::Other(node) => {
                        let b = hyperast::types::NodeStore::resolve(self.stores.node_store(), node);
                        if b.has_children() {
                            let children = b.children();
                            let children = children.unwrap();
                            self.stack.push((
                                Id::Other(*node),
                                0,
                                Some(children.iter_children().collect()),
                            ));
                        }
                        continue;
                    }
                };

                if b.has_children() {
                    let children = b.children();
                    let children = children.unwrap();
                    self.stack
                        .push((node, 0, Some(children.iter_children().collect())));
                }
                return Some(self.path.clone());
            }
        }
    }
}
