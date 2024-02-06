use hyper_ast::{
    position::TreePath,
    store::{
        defaults::{LabelIdentifier, NodeIdentifier},
        labels::LabelStore,
        nodes::legion::NodeStore,
        SimpleStores,
    },
    types::{IterableChildren, Labeled, Typed, WithChildren},
};
use hyper_ast_gen_ts_cpp::legion::CppTreeGen;
use std::{
    io::{stdout, Write},
    ops::Deref,
    sync::Arc,
};

use crate::legion::TsQueryTreeGen;

const Q0: &str = r#"(binary_expression (number_literal) "+" (number_literal))"#;
const C0: &str = r#"int f() {
    21 + 21;
}"#;

// Possible useful stuff:
// - test if subtree is conforming to ts query
//   - initially for each node in subtree, do the test
//     - terminate on wrong root type as fast as possible
//   - after that find different oracles
//     - type oracle
//     - structure hash oracle
//     - filtered structure hash oracle
//     - other convolutions (including prev hashes)
//     - labels through bags of words and defered bloom filters computing
// - edit distance between query and subtree
// - acceleration related to extracting entropy from basic constructs

#[test]
fn simple() {
    use hyper_ast::types::{LabelStore, NodeStore};
    let (code_store, code) = cpp_tree(C0.as_bytes());
    let (query_store, query) = ts_query(Q0.as_bytes());
    let path = hyper_ast::position::StructuralPosition::new(code);
    let prepared_matcher =
        PreparedMatcher::<hyper_ast_gen_ts_cpp::types::Type>::new(&query_store, query);
    for e in iter::IterAll::new(&code_store, path, code) {
        if prepared_matcher.is_matching::<hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>(&code_store, *e.node().unwrap()) {
            type T = hyper_ast_gen_ts_cpp::types::TIdN<hyper_ast::store::defaults::NodeIdentifier>;
            let n = code_store
                .node_store
                .try_resolve_typed::<T>(e.node().unwrap())
                .unwrap()
                .0;
            let t = n.get_type();
            dbg!(t);
        }
        // for i in 0..prepared_matcher.quick_trigger.root_types.deref().len() {
        //     let tt = prepared_matcher.quick_trigger.root_types[i];
        //     let pat = &prepared_matcher.patterns[i];
        //     if t == tt {
        //         dbg!("", pat.is_matching(&code_store, e.node().unwrap()));

        //     }
    }
}

// for now just uses the root types
// TODO implement approaches based on probabilitic sets
struct QuickTrigger<T> {
    root_types: Arc<[T]>,
}

struct PreparedMatcher<'a, T> {
    query_store: &'a SimpleStores<crate::types::TStore>,
    quick_trigger: QuickTrigger<T>,
    patterns: Arc<[Pattern<T>]>,
}

struct PatternMatcher<'a, 'b, T> {
    query_store: &'a SimpleStores<crate::types::TStore>,
    patterns: &'b Pattern<T>,
}

impl<'a, 'b, T> PatternMatcher<'a, 'b, T>
{
    fn is_matching<TIdN>(
        &self,
        code_store: &SimpleStores<hyper_ast_gen_ts_cpp::types::TStore>,
        id: NodeIdentifier,
    ) -> bool
    where
        TIdN: hyper_ast::types::NodeId<IdN = NodeIdentifier> + hyper_ast::types::TypedNodeId<Ty=T> + 'static,
        T: std::fmt::Debug + Eq + Copy,
    {
        let n = code_store
            .node_store
            .try_resolve_typed::<TIdN>(&id)
            .unwrap()
            .0;
        let t = n.get_type();
        dbg!(t);
        // for i in 0..self.quick_trigger.root_types.deref().len() {
        //     let tt = self.quick_trigger.root_types[i];
        //     let pat = &self.patterns[i];
        //     if t == tt {
        //         // dbg!("", pat.is_matching(&code_store, e.node().unwrap()));
        //     }
        // }
        true
    }
}

impl<'a, T: for<'b> TryFrom<&'b str>> PreparedMatcher<'a, T>
where
    for<'b> <T as TryFrom<&'b str>>::Error: std::fmt::Debug,
{
    fn is_matching<TIdN>(
        &self,
        code_store: &SimpleStores<hyper_ast_gen_ts_cpp::types::TStore>,
        id: NodeIdentifier,
    ) -> bool
    where
        TIdN: hyper_ast::types::NodeId<IdN = NodeIdentifier> + hyper_ast::types::TypedNodeId<Ty=T> + 'static,
        T: std::fmt::Debug + Eq + Copy,
    {
        let n = code_store
            .node_store
            .try_resolve_typed::<TIdN>(&id)
            .unwrap()
            .0;
        let t = n.get_type();
        dbg!(t);
        for i in 0..self.quick_trigger.root_types.deref().len() {
            let tt = self.quick_trigger.root_types[i];
            let pat = &self.patterns[i];
            if t == tt {
                // dbg!("", pat.is_matching(&code_store, e.node().unwrap()));
            }
        }
        false
    }
}
impl<'a, T: for<'b> TryFrom<&'b str>> PreparedMatcher<'a, T>
where
    for<'b> <T as TryFrom<&'b str>>::Error: std::fmt::Debug,
{
    fn new(query_store: &'a SimpleStores<crate::types::TStore>, query: NodeIdentifier) -> Self {
        use crate::types::TIdN;
        use crate::types::Type;
        use hyper_ast::store::nodes::legion::NodeIdentifier;
        use hyper_ast::types::{LabelStore, NodeStore};
        let mut root_types = vec![];
        let mut patterns = vec![];
        let n = query_store
            .node_store
            .try_resolve_typed::<TIdN<NodeIdentifier>>(&query)
            .unwrap()
            .0;
        let t = n.get_type();
        dbg!(t);
        assert_eq!(t, Type::Program);
        for rule_id in n.children().unwrap().iter_children() {
            let rule = query_store
                .node_store
                .try_resolve_typed::<TIdN<NodeIdentifier>>(&rule_id)
                .unwrap()
                .0;
            let t = rule.get_type();
            dbg!(t);
            if t == Type::NamedNode {
                let ty = rule.child(&1).unwrap();
                let ty = query_store
                    .node_store
                    .try_resolve_typed::<TIdN<NodeIdentifier>>(&ty)
                    .unwrap()
                    .0;
                let t = ty.get_type();
                dbg!(t);
                let l = ty.try_get_label();
                dbg!(l);
                let l = query_store.label_store.resolve(&l.unwrap());
                let l = T::try_from(l).unwrap();
                root_types.push(l);
                patterns.push(Self::process_named_node(query_store, *rule_id).into())
            } else if t == Type::AnonymousNode {
                todo!()
            } else {
                todo!()
            }
        }

        Self {
            query_store,
            quick_trigger: QuickTrigger {
                root_types: root_types.into(),
            },
            patterns: patterns.into(),
        }
    }
    fn process_named_node(
        query_store: &'a SimpleStores<crate::types::TStore>,
        rule: NodeIdentifier,
    ) -> Pattern<T> {
        use crate::types::TIdN;
        use crate::types::Type;
        use hyper_ast::store::nodes::legion::NodeIdentifier;
        use hyper_ast::types::{LabelStore, NodeStore};
        let mut patterns = vec![];
        let n = query_store
            .node_store
            .try_resolve_typed::<TIdN<NodeIdentifier>>(&rule)
            .unwrap()
            .0;
        let t = n.get_type();
        assert_eq!(t, Type::NamedNode);
        let mut cs = n.children().unwrap().iter_children();
        cs.next().unwrap();
        let ty = cs.next().unwrap();
        let ty = query_store
            .node_store
            .try_resolve_typed::<TIdN<NodeIdentifier>>(&ty)
            .unwrap()
            .0;
        let t = ty.get_type();
        dbg!(t);
        let l = ty.try_get_label();
        dbg!(l);
        let l = query_store.label_store.resolve(&l.unwrap());
        let l = T::try_from(l).unwrap();
        for rule_id in cs {
            let rule = query_store
                .node_store
                .try_resolve_typed::<TIdN<NodeIdentifier>>(&rule_id)
                .unwrap()
                .0;
            let t = rule.get_type();
            dbg!(t);
            if t == Type::NamedNode {
                patterns.push(Self::process_named_node(query_store, *rule_id).into())
            } else if t == Type::Spaces {
            } else if t == Type::RParen {
            } else if t == Type::AnonymousNode {
                patterns.push(Self::process_anonymous_node(query_store, *rule_id).into())
            } else {
                todo!()
            }
        }

        Pattern::NamedNode {
            ty: l,
            children: patterns.into(),
        }
    }
    fn process_anonymous_node(
        query_store: &'a SimpleStores<crate::types::TStore>,
        rule: NodeIdentifier,
    ) -> Pattern<T> {
        use crate::types::TIdN;
        use crate::types::Type;
        use hyper_ast::store::nodes::legion::NodeIdentifier;
        use hyper_ast::types::{LabelStore, NodeStore};
        let n = query_store
            .node_store
            .try_resolve_typed::<TIdN<NodeIdentifier>>(&rule)
            .unwrap()
            .0;
        let t = n.get_type();
        assert_eq!(t, Type::AnonymousNode);
        // let mut cs = n.children().unwrap().iter_children();
        // cs.next().unwrap();
        // let ty = cs.next().unwrap();
        // let ty = query_store
        //     .node_store
        //     .try_resolve_typed::<TIdN<NodeIdentifier>>(&ty)
        //     .unwrap()
        //     .0;
        // let t = ty.get_type();
        // dbg!(t);
        let l = n.try_get_label();
        dbg!(l);
        let l = query_store.label_store.resolve(&l.unwrap());
        let l = &l[0..l.len() - 2];
        let l = T::try_from(l).unwrap();
        // for rule_id in cs {
        //     let rule = query_store
        //         .node_store
        //         .try_resolve_typed::<TIdN<NodeIdentifier>>(&rule_id)
        //         .unwrap()
        //         .0;
        //     let t = rule.get_type();
        //     dbg!(t);
        //     if t == Type::NamedNode {
        //         unreachable!()
        //     } else if t == Type::Spaces {
        //     } else {
        //         todo!()
        //     }
        // }
        Pattern::AnonymousNode(l)
    }
}

enum Pattern<T> {
    NamedNode { ty: T, children: Arc<[Pattern<T>]> },
    AnonymousNode(T),
}

struct QueryMatcher<'a, T, S> {
    quick_trigger: QuickTrigger<T>,
    query_store: &'a SimpleStores<crate::types::TStore>,
    state: S,
}

fn extract_root_type(
    query_store: &SimpleStores<crate::types::TStore>,
    query: NodeIdentifier,
) -> LabelIdentifier {
    use crate::types::TIdN;
    use crate::types::Type;
    use hyper_ast::store::nodes::legion::NodeIdentifier;
    let n = query_store
        .node_store
        .try_resolve_typed::<TIdN<NodeIdentifier>>(&query)
        .unwrap()
        .0;
    let t = n.get_type();
    dbg!(t);
    assert_eq!(t, Type::Program);
    if n.child_count() == 1 {
        let rule = n.child(&0).unwrap();
        let rule = query_store
            .node_store
            .try_resolve_typed::<TIdN<NodeIdentifier>>(&rule)
            .unwrap()
            .0;
        let t = rule.get_type();
        dbg!(t);
        if t == Type::NamedNode {
            let ty = rule.child(&1).unwrap();
            let ty = query_store
                .node_store
                .try_resolve_typed::<TIdN<NodeIdentifier>>(&ty)
                .unwrap()
                .0;
            let t = ty.get_type();
            dbg!(t);
            let l = ty.try_get_label();
            dbg!(l);
            l.unwrap().clone()
        } else if t == Type::Identifier {
            todo!()
        } else {
            todo!()
        }
    } else {
        todo!()
    }
}

mod iter {
    use std::fmt::{self, Debug};

    use hyper_ast::types::HyperType;
    use hyper_ast::types::NodeStore;
    use hyper_ast::types::TypeTrait;
    use hyper_ast::{
        position::{TreePath, TreePathMut},
        store::nodes::legion::NodeIdentifier,
        types::{
            AnyType, HyperAST, IterableChildren, NodeId, Tree, Typed, TypedNodeStore, WithChildren,
        },
    };
    use num::ToPrimitive;

    use hyper_ast_gen_ts_cpp::types::TIdN;
    use hyper_ast_gen_ts_cpp::types::Type;

    pub struct IterAll<'a, T, HAST> {
        stores: &'a HAST,
        path: T,
        stack: Vec<(Id<NodeIdentifier>, u16, Option<Vec<NodeIdentifier>>)>,
    }

    enum Id<IdN> {
        Cpp(TIdN<IdN>),
        Other(IdN),
    }

    impl<IdN: Clone + Eq + NodeId> Id<IdN> {
        fn id(&self) -> &IdN {
            match self {
                Id::Cpp(node) => node.as_id(),
                Id::Other(node) => node,
            }
        }
    }

    impl<'a, T: TreePath<NodeIdentifier, u16>, HAST> Debug for IterAll<'a, T, HAST> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("IterDeclarations2")
                // .field("parents", &self.parents())
                .finish()
        }
    }

    impl<
            'a,
            T: TreePathMut<NodeIdentifier, u16> + Clone + Debug,
            HAST: HyperAST<'a, IdN = NodeIdentifier, Idx = u16>,
        > Iterator for IterAll<'a, T, HAST>
    where
        <HAST::T as Typed>::Type: Copy + Send + Sync,
        HAST::NS: TypedNodeStore<TIdN<NodeIdentifier>>,
        for<'b> <HAST::NS as TypedNodeStore<TIdN<HAST::IdN>>>::R<'b>:
            Tree<Type = Type, TreeId = HAST::IdN, Label = HAST::Label, ChildIdx = u16>,
        <HAST::NS as NodeStore<HAST::IdN>>::R<'a>:
            Tree<Type = AnyType, TreeId = HAST::IdN, Label = HAST::Label, ChildIdx = u16>,
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
                            let b = hyper_ast::types::NodeStore::resolve(
                                self.stores.node_store(),
                                node.id(),
                            );
                            if b.has_children() {
                                assert!(offset < b.child_count());
                                let cs = b.children();
                                // println!("children: {:?} {} {:?}", node,cs.len(),cs);
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
                            // self.scout.check_size(&self.stores).expect(&format!(
                            //     "{:?} {} {:?} {:?} {:?}",
                            //     node,
                            //     offset,
                            //     child,
                            //     children.len(),
                            //     self.scout
                            // ));
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
                        let child = if let Some(tid) =
                            TypedNodeStore::try_typed(self.stores.node_store(), &child)
                        {
                            Id::Cpp(tid)
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
                        Id::Cpp(node) => TypedNodeStore::resolve(self.stores.node_store(), node),
                        Id::Other(node) => {
                            let b = hyper_ast::types::NodeStore::resolve(
                                self.stores.node_store(),
                                node,
                            );
                            if b.has_children() {
                                let children = b.children();
                                let children = children.unwrap();
                                self.stack.push((
                                    Id::Other(*node),
                                    0,
                                    Some(children.iter_children().cloned().collect()),
                                ));
                            }
                            continue;
                        }
                    };
                    let t = b.get_type();
                    // let t = self.stores.type_store().resolve(t);

                    // if t.is_spaces() {
                    //     continue;
                    // } else if t.is_comment() {
                    //     continue;
                    // // } else if t == Type::PackageDeclaration {
                    // //     continue;
                    // // } else if t == Type::ImportDeclaration {
                    // //     continue;
                    // } else if t == Type::Identifier {
                    //     let mut p = self.path.clone();
                    //     p.pop();
                    //     let p = p.node().unwrap();
                    //     let Id::Cpp(x) = &self.stack.last().unwrap().0 else {
                    //         continue;
                    //     };
                    //     assert_eq!(p, x.as_id());
                    //     let b = TypedNodeStore::resolve(self.stores.node_store(), x);
                    //     let tt = b.get_type();
                    //     // let tt = self.stores.type_store().resolve(tt);
                    //     if self.path.offset() == Some(&1) && tt == Type::LambdaExpression {
                    //         self.path.check(self.stores).unwrap();
                    //         return Some(self.path.clone());
                    //     // } else if tt == Type::InferredParameters {
                    //     //     self.path.check(self.stores).unwrap();
                    //     //     return Some(self.path.clone());
                    //     }
                    //     continue;
                    // }

                    if b.has_children() {
                        let children = b.children();
                        let children = children.unwrap();
                        self.stack.push((
                            node,
                            0,
                            Some(children.iter_children().cloned().collect()),
                        ));
                    }
                    return Some(self.path.clone());

                    // if t.is_type_declaration() || t.is_parameter() {
                    //     //     assert!(b.has_children(), "{:?}", t);
                    //     //     self.path.check(self.stores).unwrap();
                    //     //     return Some(self.path.clone());
                    //     // } else if t == Type::LocalVariableDeclaration
                    //     //     || t == Type::EnhancedForVariable
                    //     //     || t == Type::CatchFormalParameter
                    //     // {
                    //     //     assert!(b.has_children(), "{:?}", t);
                    //     //     self.path.check(self.stores).unwrap();
                    //     //     return Some(self.path.clone());
                    //     // } else if t == Type::TypeParameter {
                    //     //     assert!(b.has_children(), "{:?}", t);
                    //     //     self.path.check(self.stores).unwrap();
                    //     //     return Some(self.path.clone());
                    //     // } else if t == Type::ClassBody {
                    //     //     let mut p = self.path.clone();
                    //     //     p.pop();
                    //     //     let p = p.node().unwrap();
                    //     //     let Id::Java(x) = &self.stack.last().unwrap().0 else {
                    //     //         continue;
                    //     //     };
                    //     //     assert_eq!(p, x.as_id());
                    //     //     let b = TypedNodeStore::resolve(self.stores.node_store(), x);
                    //     //     let tt = b.get_type();
                    //     //     if tt == Type::ObjectCreationExpression {
                    //     //         self.path.check(self.stores).unwrap();
                    //     //         return Some(self.path.clone());
                    //     //     } else if tt == Type::EnumDeclaration {
                    //     //         self.path.check(self.stores).unwrap();
                    //     //         return Some(self.path.clone());
                    //     //     }
                    //     // } else if t == Type::Resource {
                    //     //     assert!(b.has_children(), "{:?}", t);
                    //     //     self.path.check(self.stores).unwrap();
                    //     //     // TODO also need to find an "=" and find the name just before
                    //     //     let cs = b.children().unwrap();
                    //     //     for xx in cs.iter_children() {
                    //     //         let bb = TypedNodeStore::try_resolve(self.stores.node_store(), xx);
                    //     //         let Some((bb, _)) = bb else {
                    //     //             continue;
                    //     //         };
                    //     //         // let bb = self.stores.node_store().resolve(xx);
                    //     //         if bb.get_type() == Type::GT {
                    //     //             return Some(self.path.clone());
                    //     //         }
                    //     //     }
                    //     // } else if t.is_value_member()
                    //     // {
                    //     //     assert!(b.has_children(), "{:?}", t);
                    //     //     self.path.check(&self.stores).unwrap();
                    //     //     return Some(self.path.clone());
                    //     // } else if t.is_executable_member()
                    //     // {
                    //     //     assert!(b.has_children(), "{:?}", t);
                    //     //     self.path.check(&self.stores).unwrap();
                    //     //     return Some(self.path.clone());
                    // } else {
                    // }
                }
            }
        }
    }

    impl<
            'a,
            T: TreePathMut<NodeIdentifier, u16> + Clone + Debug,
            HAST: HyperAST<'a, IdN = NodeIdentifier, Idx = u16>,
        > IterAll<'a, T, HAST>
    where
        <HAST::T as Typed>::Type: Copy + Send + Sync,
        HAST::NS: TypedNodeStore<TIdN<NodeIdentifier>>,
        for<'b> <HAST::NS as TypedNodeStore<TIdN<HAST::IdN>>>::R<'b>:
            Tree<Type = Type, TreeId = HAST::IdN, Label = HAST::Label, ChildIdx = u16>,
        <HAST::NS as NodeStore<HAST::IdN>>::R<'a>:
            Tree<Type = AnyType, TreeId = HAST::IdN, Label = HAST::Label, ChildIdx = u16>,
    {
        fn nextOld(&mut self) -> Option<T> {
            loop {
                let (node, offset, children) = self.stack.pop()?;
                if let Some(children) = children {
                    if offset.to_usize().unwrap() < children.len() {
                        let child = children[offset.to_usize().unwrap()];
                        self.path.check(self.stores).unwrap();
                        {
                            let b = hyper_ast::types::NodeStore::resolve(
                                self.stores.node_store(),
                                node.id(),
                            );
                            if b.has_children() {
                                assert!(offset < b.child_count());
                                let cs = b.children();
                                // println!("children: {:?} {} {:?}", node,cs.len(),cs);
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
                            // self.scout.check_size(&self.stores).expect(&format!(
                            //     "{:?} {} {:?} {:?} {:?}",
                            //     node,
                            //     offset,
                            //     child,
                            //     children.len(),
                            //     self.scout
                            // ));
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
                        let child = if let Some(tid) =
                            TypedNodeStore::try_typed(self.stores.node_store(), &child)
                        {
                            Id::Cpp(tid)
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
                        Id::Cpp(node) => TypedNodeStore::resolve(self.stores.node_store(), node),
                        Id::Other(node) => {
                            let b = hyper_ast::types::NodeStore::resolve(
                                self.stores.node_store(),
                                node,
                            );
                            if b.has_children() {
                                let children = b.children();
                                let children = children.unwrap();
                                self.stack.push((
                                    Id::Other(*node),
                                    0,
                                    Some(children.iter_children().cloned().collect()),
                                ));
                            }
                            continue;
                        }
                    };
                    let t = b.get_type();
                    // let t = self.stores.type_store().resolve(t);

                    if t.is_spaces() {
                        continue;
                    } else if t.is_comment() {
                        continue;
                    // } else if t == Type::PackageDeclaration {
                    //     continue;
                    // } else if t == Type::ImportDeclaration {
                    //     continue;
                    } else if t == Type::Identifier {
                        let mut p = self.path.clone();
                        p.pop();
                        let p = p.node().unwrap();
                        let Id::Cpp(x) = &self.stack.last().unwrap().0 else {
                            continue;
                        };
                        assert_eq!(p, x.as_id());
                        let b = TypedNodeStore::resolve(self.stores.node_store(), x);
                        let tt = b.get_type();
                        // let tt = self.stores.type_store().resolve(tt);
                        if self.path.offset() == Some(&1) && tt == Type::LambdaExpression {
                            self.path.check(self.stores).unwrap();
                            return Some(self.path.clone());
                            // } else if tt == Type::InferredParameters {
                            //     self.path.check(self.stores).unwrap();
                            //     return Some(self.path.clone());
                        }
                        continue;
                    }

                    if b.has_children() {
                        let children = b.children();
                        let children = children.unwrap();
                        self.stack.push((
                            node,
                            0,
                            Some(children.iter_children().cloned().collect()),
                        ));
                    }

                    if t.is_type_declaration() || t.is_parameter() {
                        //     assert!(b.has_children(), "{:?}", t);
                        //     self.path.check(self.stores).unwrap();
                        //     return Some(self.path.clone());
                        // } else if t == Type::LocalVariableDeclaration
                        //     || t == Type::EnhancedForVariable
                        //     || t == Type::CatchFormalParameter
                        // {
                        //     assert!(b.has_children(), "{:?}", t);
                        //     self.path.check(self.stores).unwrap();
                        //     return Some(self.path.clone());
                        // } else if t == Type::TypeParameter {
                        //     assert!(b.has_children(), "{:?}", t);
                        //     self.path.check(self.stores).unwrap();
                        //     return Some(self.path.clone());
                        // } else if t == Type::ClassBody {
                        //     let mut p = self.path.clone();
                        //     p.pop();
                        //     let p = p.node().unwrap();
                        //     let Id::Java(x) = &self.stack.last().unwrap().0 else {
                        //         continue;
                        //     };
                        //     assert_eq!(p, x.as_id());
                        //     let b = TypedNodeStore::resolve(self.stores.node_store(), x);
                        //     let tt = b.get_type();
                        //     if tt == Type::ObjectCreationExpression {
                        //         self.path.check(self.stores).unwrap();
                        //         return Some(self.path.clone());
                        //     } else if tt == Type::EnumDeclaration {
                        //         self.path.check(self.stores).unwrap();
                        //         return Some(self.path.clone());
                        //     }
                        // } else if t == Type::Resource {
                        //     assert!(b.has_children(), "{:?}", t);
                        //     self.path.check(self.stores).unwrap();
                        //     // TODO also need to find an "=" and find the name just before
                        //     let cs = b.children().unwrap();
                        //     for xx in cs.iter_children() {
                        //         let bb = TypedNodeStore::try_resolve(self.stores.node_store(), xx);
                        //         let Some((bb, _)) = bb else {
                        //             continue;
                        //         };
                        //         // let bb = self.stores.node_store().resolve(xx);
                        //         if bb.get_type() == Type::GT {
                        //             return Some(self.path.clone());
                        //         }
                        //     }
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

    impl<'a, T: TreePath<NodeIdentifier, u16>, HAST: HyperAST<'a, IdN = NodeIdentifier>>
        IterAll<'a, T, HAST>
    where
        HAST::NS: TypedNodeStore<TIdN<HAST::IdN>>,
    {
        pub fn new(stores: &'a HAST, path: T, root: NodeIdentifier) -> Self {
            let root = if let Some(tid) = TypedNodeStore::try_typed(stores.node_store(), &root) {
                Id::Cpp(tid)
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
}

fn ts_query(text: &[u8]) -> (SimpleStores<crate::types::TStore>, legion::Entity) {
    use crate::types::TStore;
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TStore::default(),
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = TsQueryTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };

    let tree = match crate::legion::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    println!("{}", tree.root_node().to_sexp());
    let full_node = java_tree_gen.generate_file(b"", text, tree.walk());

    println!();
    println!(
        "{}",
        hyper_ast::nodes::SyntaxSerializer::<_, _, true>::new(
            &*java_tree_gen.stores,
            full_node.local.compressed_node
        )
    );
    stdout().flush().unwrap();
    println!(
        "{}",
        hyper_ast::nodes::JsonSerializer::<_, _, false>::new(
            &*java_tree_gen.stores,
            full_node.local.compressed_node
        )
    );
    stdout().flush().unwrap();
    println!(
        "{}",
        hyper_ast::nodes::TextSerializer::new(
            &*java_tree_gen.stores,
            full_node.local.compressed_node
        )
    );
    stdout().flush().unwrap();
    (stores, full_node.local.compressed_node)
}

fn cpp_tree(
    text: &[u8],
) -> (
    SimpleStores<hyper_ast_gen_ts_cpp::types::TStore>,
    legion::Entity,
) {
    use hyper_ast_gen_ts_cpp::types::TStore;
    let tree = match CppTreeGen::<TStore>::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    println!("{:#?}", tree.root_node().to_sexp());
    let mut stores: SimpleStores<TStore> = SimpleStores::default();
    let mut md_cache = Default::default();
    let mut tree_gen = CppTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    let x = tree_gen.generate_file(b"", text, tree.walk()).local;
    let entity = x.compressed_node;
    println!(
        "{}",
        hyper_ast::nodes::SyntaxSerializer::<_, _, true>::new(&stores, entity)
    );
    (stores, entity)
}
