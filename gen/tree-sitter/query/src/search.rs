use std::io::{stdout, Write};

use crate::legion::TsQueryTreeGen;
use crate::types::TStore;

use hyper_ast::store::{defaults::LabelIdentifier, labels::LabelStore};
use hyper_ast::types::{
    self, HyperAST, IterableChildren, Labeled, NodeStore, Typed, TypedHyperAST, TypedNodeStore,
    WithChildren,
};

use hyper_ast::store::nodes::legion::NodeIdentifier;

// use hyper_ast_gen_ts_cpp::types::TStore;

// use crate::types::TStore;

use hyper_ast::store::SimpleStores;

use std::sync::Arc;

// for now just uses the root types
// TODO implement approaches based on probabilitic sets
pub(crate) struct QuickTrigger<T> {
    pub(crate) root_types: Arc<[T]>,
}

pub(crate) struct PreparedMatcher<'a, HAST, Ty> {
    pub(crate) query_store: &'a HAST,
    pub(crate) quick_trigger: QuickTrigger<Ty>,
    pub(crate) patterns: Arc<[Pattern<Ty>]>,
}

pub(crate) struct PatternMatcher<'a, 'b, Ty> {
    pub(crate) query_store: &'a SimpleStores<crate::types::TStore>,
    pub(crate) patterns: &'b Pattern<Ty>,
}

impl<'a, 'b, Ty> PatternMatcher<'a, 'b, Ty> {
    pub(crate) fn is_matching<'store, HAST, TS, TIdN>(
        &self,
        code_store: &'store HAST,
        id: TIdN,
    ) -> bool
    where
        HAST: TypedHyperAST<'store, TIdN>,
        TIdN: hyper_ast::types::NodeId<IdN = HAST::IdN>
            + hyper_ast::types::TypedNodeId<Ty = Ty>
            + 'static,
        Ty: std::fmt::Debug + Eq + Copy,
    {
        let n = code_store
            .typed_node_store()
            .resolve(&id);
        let t = n.get_type();
        dbg!(t);
        true
    }
}

impl<'a, Ty: for<'b> TryFrom<&'b str>> PreparedMatcher<'a, SimpleStores<TStore>, Ty>
where
    for<'b> <Ty as TryFrom<&'b str>>::Error: std::fmt::Debug,
{
    pub(crate) fn is_matching<'store, HAST, TIdN>(&self, code_store: &'store HAST, id: HAST::IdN) -> bool
    where
        HAST: TypedHyperAST<'store, TIdN>,
        TIdN: hyper_ast::types::NodeId<IdN = HAST::IdN>
            + hyper_ast::types::TypedNodeId<Ty = Ty>
            + 'static,
        Ty: std::fmt::Debug + Eq + Copy,
    {
        let Some((n, _)) = code_store.typed_node_store().try_resolve(&id) else {
            return false;
        };
        let t = n.get_type();
        dbg!(t);
        for i in 0..self.quick_trigger.root_types.len() {
            let tt = self.quick_trigger.root_types[i];
            let pat = &self.patterns[i];
            if t == tt {
                dbg!(tt);
                let b: bool = pat.is_matching(code_store, id.clone());
                dbg!(tt, b);
            }
        }
        false
    }
}

impl<'a, Ty> PreparedMatcher<'a, SimpleStores<TStore>, Ty>
where
    Ty: for<'b> TryFrom<&'b str>,
    for<'b> <Ty as TryFrom<&'b str>>::Error: std::fmt::Debug,
{
    pub(crate) fn new(
        query_store: &'a SimpleStores<crate::types::TStore>,
        query: NodeIdentifier,
    ) -> Self {
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
                let l = Ty::try_from(l).unwrap();
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
    pub(crate) fn process_named_node(
        query_store: &'a SimpleStores<crate::types::TStore>,
        rule: NodeIdentifier,
    ) -> Pattern<Ty> {
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
        let l = Ty::try_from(l).unwrap();
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
    pub(crate) fn process_anonymous_node(
        query_store: &SimpleStores<TStore>,
        rule: NodeIdentifier,
    ) -> Pattern<Ty> {
        use crate::types::TIdN;
        use crate::types::Type;
        use hyper_ast::types::{LabelStore, NodeStore};
        let n = query_store
            .node_store()
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
        let l = query_store.label_store().resolve(&l.unwrap());
        let l = &l[1..l.len() - 1];
        let l = Ty::try_from(l).unwrap();
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

#[derive(Debug)]
pub(crate) enum Pattern<Ty> {
    NamedNode { ty: Ty, children: Arc<[Pattern<Ty>]> },
    AnonymousNode(Ty),
}
impl<Ty> Pattern<Ty> {
    fn is_matching<'store, HAST, TIdN>(&self, code_store: &'store HAST, id: HAST::IdN) -> bool
    where
        HAST: TypedHyperAST<'store, TIdN>,
        TIdN: hyper_ast::types::NodeId<IdN = HAST::IdN>
            + hyper_ast::types::TypedNodeId<Ty = Ty>
            + 'static,
        Ty: std::fmt::Debug + Eq + Copy,
    {
        let Some((n, _)) = code_store.typed_node_store().try_resolve(&id) else {
            dbg!();
            return false;
        };
        let t = n.get_type();
        dbg!(t, self);
        
        match self {
            Pattern::NamedNode { ty, children } => {
                if *ty != t {
                    return false;
                }
                let Some(cs) = n.children() else {
                    return children.is_empty()
                };
                let mut cs = cs.iter_children();
                let mut pats = children.iter().peekable();
                loop {
                    let Some(curr_p) = pats.peek() else {
                        return true
                    };
                    let Some(child) = cs.next() else {
                        return false
                    };
                    if curr_p.is_matching(code_store, child.clone()) {
                        pats.next();
                    }
                }
            },
            Pattern::AnonymousNode(ty) => *ty == t,
        }

    }
}

pub(crate) struct QueryMatcher<'a, T, S> {
    pub(crate) quick_trigger: QuickTrigger<T>,
    pub(crate) query_store: &'a SimpleStores<crate::types::TStore>,
    pub(crate) state: S,
}

pub(crate) fn extract_root_type(
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

pub(crate) fn ts_query(text: &[u8]) -> (SimpleStores<crate::types::TStore>, legion::Entity) {
    use crate::types::TStore;
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TStore::default(),
        node_store: hyper_ast::store::nodes::legion::NodeStore::new(),
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
