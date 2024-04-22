use crate::legion::TsQueryTreeGen;
use crate::types::TStore;

use hyper_ast::store::{defaults::LabelIdentifier, labels::LabelStore};
use hyper_ast::types::{
    HyperAST, IterableChildren, Labeled, Typed, TypedHyperAST, TypedNodeStore, WithChildren,
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

pub struct PreparedMatcher<HAST, Ty> {
    pub(crate) query_store: HAST,
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
        let n = code_store.typed_node_store().resolve(&id);
        let t = n.get_type();
        dbg!(t);
        true
    }
}

impl<'a, Ty: for<'b> TryFrom<&'b str>, QHAST: std::ops::Deref<Target = SimpleStores<TStore>>>
    PreparedMatcher<QHAST, Ty>
where
    for<'b> <Ty as TryFrom<&'b str>>::Error: std::fmt::Debug,
{
    pub fn is_matching<'store, HAST, TIdN>(&self, code_store: &'store HAST, id: HAST::IdN) -> bool
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
        for i in 0..self.quick_trigger.root_types.len() {
            let tt = self.quick_trigger.root_types[i];
            let pat = &self.patterns[i];
            if t == tt {
                let res: MatchingRes = pat.is_matching(code_store, id.clone());
                if res.matched {
                    return true;
                }
            }
        }
        false
    }
    pub fn is_matching_and_capture<'store, HAST, TIdN>(
        &self,
        code_store: &'store HAST,
        id: HAST::IdN,
    ) -> Option<std::collections::HashMap<String, CaptureRes>>
    where
        HAST: TypedHyperAST<'store, TIdN>,
        TIdN: hyper_ast::types::NodeId<IdN = HAST::IdN>
            + hyper_ast::types::TypedNodeId<Ty = Ty>
            + 'static,
        Ty: std::fmt::Debug + Eq + Copy,
    {
        let Some((n, _)) = code_store.typed_node_store().try_resolve(&id) else {
            return None;
        };
        let t = n.get_type();
        for i in 0..self.quick_trigger.root_types.len() {
            let tt = self.quick_trigger.root_types[i];
            let pat = &self.patterns[i];
            if t == tt {
                let res: MatchingRes = pat.is_matching(code_store, id.clone());
                if res.matched {
                    return Some(res.captures);
                }
            }
        }
        None
    }
}

impl<HAST, Ty> PreparedMatcher<HAST, Ty>
where
    Ty: for<'b> TryFrom<&'b str> + std::fmt::Debug,
    for<'b> <Ty as TryFrom<&'b str>>::Error: std::fmt::Debug,
{
    pub(crate) fn with_patterns(
        query_store: HAST,
        root_types: Vec<Ty>,
        patterns: Vec<Pattern<Ty>>,
    ) -> PreparedMatcher<HAST, Ty> {
        Self {
            query_store,
            quick_trigger: QuickTrigger {
                root_types: root_types.into(),
            },
            patterns: patterns.into(),
        }
    }
}

impl<'a, Ty> PreparedMatcher<&'a SimpleStores<TStore>, Ty>
where
    Ty: for<'b> TryFrom<&'b str> + std::fmt::Debug,
    for<'b> <Ty as TryFrom<&'b str>>::Error: std::fmt::Debug,
{
    pub fn new(query_store: &'a SimpleStores<crate::types::TStore>, query: NodeIdentifier) -> Self {
        let (root_types, patterns) = Self::new_aux(query_store, query);

        Self::with_patterns(query_store, root_types, patterns)
    }

    pub(crate) fn new_aux(
        query_store: &'a SimpleStores<TStore>,
        query: legion::Entity,
    ) -> (Vec<Ty>, Vec<Pattern<Ty>>)
    where
        Ty: for<'b> TryFrom<&'b str> + std::fmt::Debug,
    {
        use crate::types::TIdN;
        use crate::types::Type;
        use hyper_ast::types::LabelStore;
        let mut root_types = vec![];
        let mut patterns = vec![];
        let n = query_store
            .node_store
            .try_resolve_typed::<TIdN<NodeIdentifier>>(&query)
            .unwrap()
            .0;
        let t = n.get_type();
        assert_eq!(t, Type::Program);
        for rule_id in n.children().unwrap().iter_children() {
            let rule = query_store
                .node_store
                .try_resolve_typed::<TIdN<NodeIdentifier>>(&rule_id)
                .unwrap()
                .0;
            let t = rule.get_type();
            if t == Type::NamedNode {
                let ty = rule.child(&1).unwrap();
                let ty = query_store
                    .node_store
                    .try_resolve_typed::<TIdN<NodeIdentifier>>(&ty)
                    .unwrap()
                    .0;
                assert_eq!(ty.get_type(), Type::Identifier);
                let l = ty.try_get_label();
                let l = query_store.label_store.resolve(&l.unwrap());
                let l = Ty::try_from(l).expect("the node type does not exist");
                root_types.push(l);
                patterns.push(Self::process_named_node(query_store, *rule_id).into());
            } else if t == Type::AnonymousNode {
                todo!()
            } else if t == Type::Spaces {
            } else if t == Type::Predicate {
                let prev = patterns
                    .pop()
                    .expect("a predicate should be preceded by a pattern");

                let predicate = Self::preprocess_predicate(query_store, *rule_id);

                let predicated = Pattern::Predicated {
                    predicate,
                    pat: Arc::new(prev),
                };
                patterns.push(predicated);
            } else {
                todo!("{}", t)
            }
        }
        (root_types, patterns)
    }
    pub(crate) fn process_named_node(
        query_store: &'a SimpleStores<crate::types::TStore>,
        rule: NodeIdentifier,
    ) -> Pattern<Ty> {
        use crate::types::TIdN;
        use crate::types::Type;
        use hyper_ast::types::LabelStore;
        let mut patterns = vec![];
        let n = query_store
            .node_store
            .try_resolve_typed::<TIdN<NodeIdentifier>>(&rule)
            .unwrap()
            .0;
        let t = n.get_type();
        assert_eq!(t, Type::NamedNode);
        let mut cs = n.children().unwrap().iter_children().peekable();
        cs.next().unwrap();
        let ty = cs.next().unwrap();
        let ty = query_store
            .node_store
            .try_resolve_typed::<TIdN<NodeIdentifier>>(&ty)
            .unwrap()
            .0;
        assert_eq!(ty.get_type(), Type::Identifier);
        let l = ty.try_get_label();
        let l = query_store.label_store.resolve(&l.unwrap());
        let l = Ty::try_from(l).expect("the node type does not exist");
        loop {
            let Some(rule_id) = cs.peek() else { break };
            let rule = query_store
                .node_store
                .try_resolve_typed::<TIdN<NodeIdentifier>>(rule_id)
                .unwrap()
                .0;
            let t = rule.get_type();
            if t == Type::NamedNode {
                patterns.push(Self::process_named_node(query_store, **rule_id).into())
            } else if t == Type::Spaces {
            } else if t == Type::RParen {
            } else if t == Type::AnonymousNode {
                patterns.push(Self::process_anonymous_node(query_store, **rule_id).into())
            } else if t == Type::Capture {
                break;
            } else if t == Type::Predicate {
                let prev = patterns.pop().expect("predicate must be preceded by node");
                let predicate = Self::preprocess_predicate(query_store, **rule_id);
                patterns.push(Pattern::Predicated {
                    predicate,
                    pat: Arc::new(prev),
                });
            } else {
                todo!("{}", t)
            }
            cs.next();
        }
        let mut res = Pattern::NamedNode {
            ty: l,
            children: patterns.into(),
        };
        loop {
            let Some(rule_id) = cs.peek() else {
                return res;
            };
            let n = query_store
                .node_store
                .try_resolve_typed::<TIdN<NodeIdentifier>>(rule_id)
                .unwrap()
                .0;
            let t = n.get_type();
            if t == Type::Capture {
                let mut cs = n.children().unwrap().iter_children();
                let n = query_store
                    .node_store
                    .try_resolve_typed::<TIdN<NodeIdentifier>>(cs.next().unwrap())
                    .unwrap()
                    .0;
                let t = n.get_type();
                assert_eq!(t, Type::At);
                let name = cs.next().unwrap();
                let ty = query_store
                    .node_store
                    .try_resolve_typed::<TIdN<NodeIdentifier>>(name)
                    .unwrap()
                    .0;
                assert_eq!(ty.get_type(), Type::Identifier);
                let name = ty.try_get_label().unwrap();
                let name = query_store.label_store.resolve(&name);
                let name = name.to_string();
                match &res {
                    Pattern::NamedNode { .. } | Pattern::Capture { .. } => (),
                    Pattern::Predicated { .. } => panic!(),
                    Pattern::AnonymousNode { .. } => todo!("not sure if it works properly"),
                    p => todo!("{:?} still not implemented to be captured", p), // TODO quantificators will need more works
                }
                res = Pattern::Capture {
                    name,
                    pat: Arc::new(res),
                };
            } else if t == Type::Quantifier {
                break;
            } else {
                todo!("{}", t)
            }
            cs.next().unwrap();
        }
        loop {
            let Some(rule_id) = cs.next() else {
                break res;
            };
            let n = query_store
                .node_store
                .try_resolve_typed::<TIdN<NodeIdentifier>>(rule_id)
                .unwrap()
                .0;
            let t = n.get_type();
            if t == Type::Capture {
                panic!("captures after predicated are not allowed");
            } else if t == Type::Quantifier {
                todo!()
            } else {
                todo!("{}", t)
            }
        }
    }

    pub(crate) fn process_anonymous_node(
        query_store: &SimpleStores<TStore>,
        rule: NodeIdentifier,
    ) -> Pattern<Ty> {
        use crate::types::TIdN;
        use crate::types::Type;
        use hyper_ast::types::LabelStore;
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
        let l = n.try_get_label();
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

    pub(crate) fn preprocess_predicate(
        query_store: &SimpleStores<TStore>,
        rule: NodeIdentifier,
    ) -> Predicate {
        use crate::types::TIdN;
        use crate::types::Type;
        use hyper_ast::types::LabelStore;
        let n = query_store
            .node_store()
            .try_resolve_typed::<TIdN<NodeIdentifier>>(&rule)
            .unwrap()
            .0;
        let t = n.get_type();
        assert_eq!(t, Type::Predicate);
        let mut cs = n.children().unwrap().iter_children();
        cs.next().unwrap();
        let sharp = cs.next().unwrap();
        let sharp = query_store
            .node_store
            .try_resolve_typed::<TIdN<NodeIdentifier>>(&sharp)
            .unwrap()
            .0;
        let t = sharp.get_type();
        assert_eq!(t, Type::Sharp);

        let pred = cs.next().unwrap();
        let pred = query_store
            .node_store
            .try_resolve_typed::<TIdN<NodeIdentifier>>(&pred)
            .unwrap()
            .0;
        let t = pred.get_type();
        assert_eq!(t, Type::Identifier);

        let l = pred.try_get_label();
        let l = query_store.label_store().resolve(&l.unwrap());
        match l {
            "eq" => {
                let pred = cs.next().unwrap();
                let pred = query_store
                    .node_store
                    .try_resolve_typed::<TIdN<NodeIdentifier>>(&pred)
                    .unwrap()
                    .0;
                let t = pred.get_type();
                assert_eq!(t, Type::PredicateType);
                let l = pred.try_get_label();
                let l = query_store.label_store().resolve(&l.unwrap());
                assert_eq!(l, "?");
                for rule_id in cs {
                    let rule = query_store
                        .node_store
                        .try_resolve_typed::<TIdN<NodeIdentifier>>(&rule_id)
                        .unwrap()
                        .0;
                    let t = rule.get_type();
                    if t == Type::Parameters {
                        let mut cs = rule.children().unwrap().iter_children();
                        let left = cs.next().unwrap();
                        let left = query_store
                            .node_store
                            .try_resolve_typed::<TIdN<NodeIdentifier>>(left)
                            .unwrap()
                            .0;
                        let left = match left.get_type() {
                            Type::Capture => preprocess_capture_pred_arg(left, query_store),
                            t => todo!("{}", t),
                        };
                        {
                            let center = cs.next().unwrap();
                            let center = query_store
                                .node_store
                                .try_resolve_typed::<TIdN<NodeIdentifier>>(center)
                                .unwrap()
                                .0;
                            let t = center.get_type();
                            assert_eq!(t, Type::Spaces);
                        }
                        let right = cs.next().unwrap();
                        let right = query_store
                            .node_store
                            .try_resolve_typed::<TIdN<NodeIdentifier>>(right)
                            .unwrap()
                            .0;
                        return match right.get_type() {
                            Type::Capture => {
                                let right = preprocess_capture_pred_arg(right, query_store);
                                Predicate::Eq { left, right }
                            }
                            Type::String => {
                                let right = preprocess_capture_pred_arg(right, query_store);
                                Predicate::EqString { left, right }
                            }
                            t => todo!("{}", t),
                        };
                    } else if t == Type::Spaces {
                    } else {
                        todo!()
                    }
                }
                panic!()
            }
            l => todo!("{}", l),
        }
    }
}

fn preprocess_capture_pred_arg(
    arg: hyper_ast::store::nodes::legion::HashedNodeRef<'_, crate::types::TIdN<legion::Entity>>,
    query_store: &SimpleStores<TStore>,
) -> String {
    use crate::types::TIdN;
    use crate::types::Type;
    use hyper_ast::types::LabelStore;
    if let Type::Capture = arg.get_type() {
        let mut cs = arg.children().unwrap().iter_children();
        assert_eq!(2, cs.len());
        let at = cs.next().unwrap();
        let at = query_store
            .node_store
            .try_resolve_typed::<TIdN<NodeIdentifier>>(&at)
            .unwrap()
            .0;
        let t = at.get_type();
        assert_eq!(t, Type::At);
        let id = cs.next().unwrap();
        let id = query_store
            .node_store
            .try_resolve_typed::<TIdN<NodeIdentifier>>(&id)
            .unwrap()
            .0;
        let t = id.get_type();
        assert_eq!(t, Type::Identifier);
        let l = id.try_get_label();
        let l = query_store.label_store().resolve(&l.unwrap());
        l.to_string()
    } else if let Type::String = arg.get_type() {
        let l = arg.try_get_label();
        let l = query_store.label_store().resolve(&l.unwrap());
        l[1..l.len() - 1].to_string()
    } else {
        unreachable!()
    }
}

#[derive(Debug)]
pub(crate) enum Pattern<Ty> {
    NamedNode {
        ty: Ty,
        children: Arc<[Pattern<Ty>]>,
    },
    AnonymousNode(Ty),
    Capture {
        name: String,
        pat: Arc<Pattern<Ty>>,
    },
    Predicated {
        predicate: Predicate,
        pat: Arc<Pattern<Ty>>,
    },
}
#[derive(Debug)]
pub(crate) enum Predicate {
    Eq { left: String, right: String },
    EqString { left: String, right: String },
}
#[derive(Debug)]
pub(crate) struct MatchingRes {
    matched: bool,
    captures: std::collections::HashMap<String, CaptureRes>,
}

impl MatchingRes {
    fn fals() -> Self {
        Self {
            matched: false,
            captures: Default::default(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum CaptureRes {
    Label(String),
    Node,
}

impl CaptureRes {
    pub(crate) fn label(self) -> Option<String> {
        match self {
            CaptureRes::Label(l) => Some(l),
            CaptureRes::Node => None,
        }
    }
}

impl<Ty> Pattern<Ty> {
    fn is_matching<'store, HAST, TIdN>(
        &self,
        code_store: &'store HAST,
        id: HAST::IdN,
    ) -> MatchingRes
    where
        HAST: TypedHyperAST<'store, TIdN>,
        TIdN: hyper_ast::types::NodeId<IdN = HAST::IdN>
            + hyper_ast::types::TypedNodeId<Ty = Ty>
            + 'static,
        Ty: std::fmt::Debug + Eq + Copy,
    {
        let Some((n, _)) = code_store.typed_node_store().try_resolve(&id) else {
            dbg!();
            return MatchingRes::fals();
        };
        let t = n.get_type();

        match self {
            Pattern::NamedNode { ty, children } => {
                if *ty != t {
                    return MatchingRes::fals();
                }
                let Some(cs) = n.children() else {
                    return MatchingRes {
                        matched: children.is_empty(),
                        captures: Default::default(),
                    };
                };
                let mut cs = cs.iter_children();
                let mut pats = children.iter().peekable();
                let mut captures = Default::default();
                if pats.peek().is_none() {
                    return MatchingRes {
                        matched: true,
                        captures,
                    };
                }
                let mut matched = false;
                loop {
                    let Some(curr_p) = pats.peek() else {
                        return MatchingRes { matched, captures };
                    };
                    let Some(child) = cs.next() else {
                        return MatchingRes::fals();
                    };
                    match curr_p.is_matching(code_store, child.clone()) {
                        MatchingRes {
                            matched: true,
                            captures: capt,
                        } => {
                            pats.next();
                            matched = true;
                            captures.extend(capt);
                        }
                        MatchingRes { matched: false, .. } => {}
                    }
                }
            }
            Pattern::AnonymousNode(ty) => MatchingRes {
                matched: *ty == t,
                captures: Default::default(),
            },
            Pattern::Capture { name, pat } => match pat.is_matching(code_store, id.clone()) {
                MatchingRes {
                    matched: true,
                    mut captures,
                } => {
                    let name = name.clone();
                    let n = code_store.typed_node_store().try_resolve(&id).unwrap().0;
                    let v = if n.children().map_or(true, |x| x.is_empty()) {
                        let l = n.try_get_label().unwrap();
                        use hyper_ast::types::LabelStore;
                        let l = code_store.label_store().resolve(l);
                        CaptureRes::Label(l.to_owned())
                    } else {
                        todo!()
                    };
                    captures.insert(name, v);
                    MatchingRes {
                        matched: true,
                        captures,
                    }
                }
                MatchingRes { matched: false, .. } => MatchingRes::fals(),
            },
            Pattern::Predicated { predicate, pat } => match predicate {
                Predicate::Eq { left, right } => {
                    let MatchingRes { matched, captures } = pat.is_matching(code_store, id);
                    if matched {
                        let matched = captures
                            .get(left)
                            .map_or(false, |x| Some(x) == captures.get(right));
                        let captures = if matched {
                            captures
                        } else {
                            Default::default()
                        };
                        MatchingRes { matched, captures }
                    } else {
                        MatchingRes::fals()
                    }
                }
                Predicate::EqString { left, right } => {
                    let MatchingRes { matched, captures } = pat.is_matching(code_store, id);
                    if matched {
                        let Some(CaptureRes::Label(left)) = captures.get(left) else {
                            return MatchingRes::fals();
                        };
                        let matched = left == right;
                        let captures = if matched {
                            captures
                        } else {
                            Default::default()
                        };
                        MatchingRes { matched, captures }
                    } else {
                        MatchingRes::fals()
                    }
                }
                p => todo!("{:?}", p),
            },
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
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TStore::default(),
        node_store: hyper_ast::store::nodes::legion::NodeStore::new(),
    };
    let query = ts_query2(&mut stores, text);
    (stores, query)
}

pub(crate) fn ts_query2(stores: &mut SimpleStores<TStore>, text: &[u8]) -> legion::Entity {
    let mut md_cache = Default::default();
    let mut query_tree_gen = TsQueryTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: stores,
        md_cache: &mut md_cache,
    };

    let tree = match crate::legion::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => {
            eprintln!("{}", t.root_node().to_sexp());
            t
        }
    };
    let full_node = query_tree_gen.generate_file(b"", text, tree.walk());
    full_node.local.compressed_node
}
