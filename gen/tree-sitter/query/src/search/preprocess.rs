use std::collections::HashMap;
use std::sync::Arc;

use super::{Capture, Pattern, Predicate, PreparedMatcher, QuickTrigger};

use hyper_ast::store::nodes::legion::NodeIdentifier;
use hyper_ast::store::SimpleStores;
use hyper_ast::types::{HyperAST, IterableChildren, Labeled, TypeStore, Typed, WithChildren};

use tree_sitter::CaptureQuantifier as Quant;

use crate::types::TStore;

pub struct PreparingMatcher<Ty> {
    root_types: Vec<Ty>,
    patterns: Vec<Pattern<Ty>>,
    captures: Vec<Capture>,
    quantifiers: Vec<HashMap<usize, tree_sitter::CaptureQuantifier>>,
}

impl<Ty> PreparingMatcher<Ty> {
    fn add_or_insert_capture(&mut self, capture: &str) -> u32 {
        let p = self.captures.iter().position(|x| x.name == capture);
        let Some(p) = p else {
            let len = self.captures.len();
            self.captures.push(Capture {
                name: capture.to_string(),
            });
            return len as u32;
        };
        p as u32
    }
    fn insert_quantifier_for_capture_id(&mut self, capture_id: u32, quantifier: Quant) {
        self.quantifiers
            .last_mut()
            .unwrap()
            .insert(capture_id as usize, quantifier);
    }
}

impl<Ty> Default for PreparingMatcher<Ty> {
    fn default() -> Self {
        Self {
            root_types: Default::default(),
            patterns: Default::default(),
            captures: Default::default(),
            quantifiers: Default::default(),
        }
    }
}

impl<Ty> From<PreparingMatcher<Ty>> for PreparedMatcher<Ty> {
    fn from(value: PreparingMatcher<Ty>) -> Self {
        assert_eq!(value.patterns.len(), value.quantifiers.len());
        dbg!(&value.quantifiers);
        Self {
            quick_trigger: QuickTrigger {
                root_types: value.root_types.into(),
            },
            patterns: value.patterns.into(),
            captures: value.captures.into(),
            quantifiers: value.quantifiers.into(),
        }
    }
}

impl<'a, Ty> PreparedMatcher<Ty>
where
    Ty: for<'b> TryFrom<&'b str> + std::fmt::Debug,
    for<'b> <Ty as TryFrom<&'b str>>::Error: std::fmt::Debug,
{
    pub fn new(query_store: &'a SimpleStores<crate::types::TStore>, query: NodeIdentifier) -> Self {
        let preparing = Self::new_aux(query_store, query);

        preparing.into()
    }

    pub(crate) fn new_aux(
        query_store: &'a SimpleStores<TStore>,
        query: legion::Entity,
    ) -> PreparingMatcher<Ty>
    where
        Ty: for<'b> TryFrom<&'b str> + std::fmt::Debug,
    {
        use crate::types::TIdN;
        use crate::types::Type;
        use hyper_ast::types::LabelStore;
        let mut res = PreparingMatcher::default();
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
                res.quantifiers.push(Default::default());
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
                let patt = Self::process_named_node(query_store, *rule_id, &mut res).into();
                res.root_types.push(l);
                res.patterns.push(patt);
            } else if t == Type::AnonymousNode {
                todo!()
            } else if t == Type::Spaces {
                continue;
            } else if t == Type::List {
                res.quantifiers.push(Default::default());
                let mut patterns = vec![];
                let cs = rule.children().unwrap();
                let mut capture = vec![];
                for id in cs.iter_children() {
                    let i = query_store
                        .node_store
                        .try_resolve_typed::<TIdN<NodeIdentifier>>(&id)
                        .unwrap()
                        .0;
                    if i.get_type() == Type::Capture {
                        let mut cs = i.children().unwrap().iter_children();
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
                        dbg!(name);
                        dbg!(&res.captures);
                        capture.push(res.add_or_insert_capture(name));
                        continue;
                    }
                    if i.get_type() == Type::LBracket {
                        continue;
                    }
                    if i.get_type() == Type::Spaces {
                        continue;
                    }
                    if i.get_type() == Type::RBracket {
                        continue;
                    }
                    if i.get_type() == Type::Comment {
                        continue;
                    }
                    assert!(capture.is_empty(), "{}", i.get_type());
                    if i.get_type() == Type::NamedNode {
                        let ty = i.child(&1).unwrap();
                        let ty = query_store
                            .node_store
                            .try_resolve_typed::<TIdN<NodeIdentifier>>(&ty)
                            .unwrap()
                            .0;
                        assert_eq!(ty.get_type(), Type::Identifier);
                        let l = ty.try_get_label();
                        let l = query_store.label_store.resolve(&l.unwrap());
                        let l = Ty::try_from(l).expect("the node type does not exist");
                        let patt = Self::process_named_node(query_store, *id, &mut res);
                        res.root_types.push(l);
                        patterns.push(patt);
                    } else {
                        assert_eq!(i.get_type(), Type::Identifier);
                    }
                }
                for name in capture {
                    res.quantifiers
                        .last_mut()
                        .unwrap()
                        .insert(name as usize, Quant::One);
                    patterns.iter_mut().for_each(|res| {
                        let name = name.to_owned();
                        let r = std::mem::replace(res, Pattern::List(vec![].into()));
                        *res = Pattern::Capture {
                            name,
                            pat: Arc::new(r),
                        };
                    })
                }
                let patt = Pattern::List(patterns.into());
                res.patterns.push(patt);
            } else if t == Type::Predicate {
                let prev = res
                    .patterns
                    .pop()
                    .expect("a predicate should be preceded by a pattern");

                let predicate = Self::preprocess_predicate(query_store, *rule_id);
                let predicate = predicate.resolve_name(&res.captures);
                dbg!(&predicate);
                let predicated = Pattern::Predicated {
                    predicate,
                    pat: Arc::new(prev),
                };
                res.patterns.push(predicated);
            } else {
                todo!("{:?}", t)
            }
        }
        res
    }
    pub(crate) fn process_named_node(
        query_store: &'a SimpleStores<crate::types::TStore>,
        rule: NodeIdentifier,
        preparing: &mut PreparingMatcher<Ty>,
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
        let mut l = if ty.get_type() == Type::Inderscore {
            None
        } else {
            assert_eq!(Type::Identifier, ty.get_type());
            let l = ty.try_get_label();
            let l = query_store.label_store.resolve(&l.unwrap());
            let l = Ty::try_from(l).expect("the node type does not exist");
            Some(l)
        };
        loop {
            let Some(rule_id) = cs.peek() else { break };
            let rule = query_store
                .node_store
                .try_resolve_typed::<TIdN<NodeIdentifier>>(rule_id)
                .unwrap()
                .0;
            let t = rule.get_type();
            if t == Type::NamedNode {
                patterns.push(Self::process_named_node(query_store, **rule_id, preparing).into())
            } else if t == Type::Spaces {
            } else if t == Type::RParen {
            } else if t == Type::AnonymousNode {
                patterns
                    .push(Self::process_anonymous_node(query_store, **rule_id, preparing).into())
            } else if t == Type::Capture {
                break;
            } else if t == Type::Predicate {
                let prev = patterns.pop().expect("predicate must be preceded by node");
                let predicate = Self::preprocess_predicate(query_store, **rule_id);
                let predicate = predicate.resolve_name(&preparing.captures);
                dbg!(&predicate);
                patterns.push(Pattern::Predicated {
                    predicate,
                    pat: Arc::new(prev),
                });
            } else if t == Type::Quantifier {
                break;
            } else if t == Type::FieldDefinition {
                patterns
                    .push(Self::process_field_definition(query_store, **rule_id, preparing).into())
            } else if t == Type::Dot {
                patterns.push(Pattern::Dot);
            } else if t == Type::NegatedField {
                let mut cs = rule.children().unwrap().iter_children();
                let bang = cs.next().unwrap();
                let bang = query_store
                    .node_store
                    .try_resolve_typed::<TIdN<NodeIdentifier>>(&bang)
                    .unwrap()
                    .0;
                assert_eq!(Type::Bang, bang.get_type());
                let field_name = cs.next().unwrap();
                let field_name = query_store
                    .node_store
                    .try_resolve_typed::<TIdN<NodeIdentifier>>(&field_name)
                    .unwrap()
                    .0;
                assert_eq!(Type::Identifier, field_name.get_type());
                assert!(cs.next().is_none());
                let field_name = field_name.try_get_label();
                let field_name = query_store.label_store.resolve(&field_name.unwrap());
                let field_name = field_name.to_string();
                patterns.push(Pattern::NegatedField(field_name));
            } else if t == Type::Slash {
                cs.next();
                let Some(rule_id) = cs.peek() else { break };
                let ty = query_store
                    .node_store
                    .try_resolve_typed::<TIdN<NodeIdentifier>>(rule_id)
                    .unwrap()
                    .0;
                assert_eq!(Type::Identifier, ty.get_type());
                // TODO properly filter using the supertype
                // NOTE for now just ignore supertype
                l = {
                    let l = ty.try_get_label();
                    let l = query_store.label_store.resolve(&l.unwrap());
                    let l = Ty::try_from(l).expect("the node type does not exist");
                    Some(l)
                };
            } else {
                todo!("{}", t)
            }
            cs.next();
        }

        let mut res = Pattern::named(l, patterns);
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
                dbg!(name);
                let capture_id = preparing.add_or_insert_capture(name);
                dbg!(capture_id);
                dbg!(&res);
                match &res {
                    Pattern::NamedNode { .. } | Pattern::AnyNode { .. } => {
                        preparing.insert_quantifier_for_capture_id(capture_id, Quant::One)
                    }
                    Pattern::Predicated { .. } => panic!(),
                    Pattern::AnonymousNode { .. } => todo!("not sure if it works properly"),
                    Pattern::List(_) => todo!(),
                    Pattern::FieldDefinition { .. } => todo!(),
                    Pattern::Dot { .. } => todo!(),
                    Pattern::NegatedField { .. } => todo!(),
                    Pattern::Quantified { quantifier, .. } => {
                        preparing.insert_quantifier_for_capture_id(capture_id, *quantifier);
                    }
                    Pattern::Capture { pat, .. } => match pat.as_ref() {
                        Pattern::Capture { .. } => todo!(),
                        Pattern::Quantified { quantifier, .. } => {
                            preparing.insert_quantifier_for_capture_id(capture_id, *quantifier);
                        }
                        _ => {
                            preparing.insert_quantifier_for_capture_id(capture_id, Quant::One);
                        }
                    },
                }
                res = Pattern::Capture {
                    name: capture_id as u32,
                    pat: Arc::new(res),
                };
            } else if t == Type::Quantifier {
                let mut cs = n.children().unwrap().iter_children();
                let n = query_store
                    .node_store
                    .try_resolve_typed::<TIdN<NodeIdentifier>>(&cs.next().unwrap())
                    .unwrap()
                    .0;

                let quantifier = query_store.type_store().resolve_type(&n);
                assert!(cs.next().is_none());

                let quantifier = match quantifier {
                    Type::QMark => Quant::ZeroOrOne,
                    Type::Star => Quant::ZeroOrMore,
                    Type::Plus => Quant::OneOrMore,
                    x => todo!("{}", x),
                };

                res = Pattern::Quantified {
                    quantifier,
                    pat: Arc::new(res),
                };
            } else if t == Type::Spaces {
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
    pub(crate) fn process_field_definition(
        query_store: &'a SimpleStores<crate::types::TStore>,
        rule: NodeIdentifier,
        preparing: &mut PreparingMatcher<Ty>,
    ) -> Pattern<Ty> {
        use crate::types::TIdN;
        use crate::types::Type;
        use hyper_ast::types::LabelStore;
        let n = query_store
            .node_store
            .try_resolve_typed::<TIdN<NodeIdentifier>>(&rule)
            .unwrap()
            .0;
        let t = n.get_type();
        assert_eq!(t, Type::FieldDefinition);
        let mut cs = n.children().unwrap().iter_children().peekable();
        let field_name = cs.next().unwrap();
        let field_name = query_store
            .node_store
            .try_resolve_typed::<TIdN<NodeIdentifier>>(&field_name)
            .unwrap()
            .0;
        let field_name = field_name.try_get_label();
        let field_name = query_store.label_store.resolve(&field_name.unwrap());
        let colon = cs.next().unwrap();
        let colon = query_store
            .node_store
            .try_resolve_typed::<TIdN<NodeIdentifier>>(&colon)
            .unwrap()
            .0;
        assert_eq!(colon.get_type(), Type::Colon);
        let pat = loop {
            let Some(rule_id) = cs.next() else { panic!() };
            let rule = query_store
                .node_store
                .try_resolve_typed::<TIdN<NodeIdentifier>>(rule_id)
                .unwrap()
                .0;
            let t = rule.get_type();
            if t == Type::NamedNode {
                assert!(cs.next().is_none());
                break Self::process_named_node(query_store, *rule_id, preparing).into();
            } else if t == Type::Spaces {
            } else if t == Type::AnonymousNode {
                break Self::process_anonymous_node(query_store, *rule_id, preparing).into();
            } else {
                todo!("{}", t)
            }
        };
        loop {
            let Some(rule_id) = cs.next() else { break };
            let rule = query_store
                .node_store
                .try_resolve_typed::<TIdN<NodeIdentifier>>(rule_id)
                .unwrap()
                .0;
            let t = rule.get_type();
            if t == Type::Spaces {
            } else {
                todo!("{}", t)
            }
        }
        let name = field_name.to_string();
        Pattern::FieldDefinition { name, pat }
    }

    pub(crate) fn process_anonymous_node(
        query_store: &SimpleStores<TStore>,
        rule: NodeIdentifier,
        preparing: &mut PreparingMatcher<Ty>,
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
        if let Some(cs) = n.children() {
            let mut cs = cs.iter_children();
            let l = loop {
                let rule_id = cs.next().unwrap();
                let rule = query_store
                    .node_store
                    .try_resolve_typed::<TIdN<NodeIdentifier>>(&rule_id)
                    .unwrap()
                    .0;
                let t = rule.get_type();
                dbg!(t);
                if t == Type::NamedNode {
                    unreachable!()
                } else if t == Type::Spaces {
                } else if t == Type::Identifier {
                    let l = rule.try_get_label();
                    let l = query_store.label_store().resolve(&l.unwrap());
                    dbg!(l);
                    let l = &l[1..l.len() - 1];
                    dbg!(l);
                    let l = Ty::try_from(l).unwrap();
                    break l;
                } else {
                    todo!()
                }
            };
            let mut res = Pattern::AnonymousNode(l);
            for rule_id in cs {
                let rule = query_store
                    .node_store
                    .try_resolve_typed::<TIdN<NodeIdentifier>>(&rule_id)
                    .unwrap()
                    .0;
                let t = rule.get_type();
                dbg!(t);
                if t == Type::NamedNode {
                    unreachable!()
                } else if t == Type::Spaces {
                } else if t == Type::Identifier {
                    unreachable!()
                } else if t == Type::Quantifier {
                    let mut cs = rule.children().unwrap().iter_children();
                    let n = query_store
                        .node_store
                        .try_resolve_typed::<TIdN<NodeIdentifier>>(&cs.next().unwrap())
                        .unwrap()
                        .0;
                    let quantifier = query_store.type_store().resolve_type(&n);
                    assert!(cs.next().is_none());
                    let quantifier = match quantifier {
                        Type::QMark => Quant::ZeroOrOne,
                        Type::Star => Quant::ZeroOrMore,
                        Type::Plus => Quant::OneOrMore,
                        x => todo!("{}", x),
                    };
                    res = Pattern::Quantified {
                        quantifier,
                        pat: Arc::new(res),
                    };
                } else if t == Type::Capture {
                    let mut cs = rule.children().unwrap().iter_children();
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
                    dbg!(name);
                    let capture_id = preparing.add_or_insert_capture(name);
                    dbg!(capture_id);
                    match &res {
                        Pattern::NamedNode { .. } | Pattern::AnyNode { .. } => {
                            preparing.insert_quantifier_for_capture_id(capture_id, Quant::One);
                        }
                        Pattern::Predicated { .. } => panic!(),
                        Pattern::AnonymousNode { .. } => (),
                        Pattern::List(_) => todo!(),
                        Pattern::FieldDefinition { .. } => todo!(),
                        Pattern::Dot { .. } => todo!(),
                        Pattern::Quantified { quantifier, .. } => {
                            preparing.insert_quantifier_for_capture_id(capture_id, *quantifier);
                        }
                        Pattern::Capture { pat, .. } => match pat.as_ref() {
                            Pattern::Capture { .. } => todo!(),
                            Pattern::Quantified { quantifier, .. } => {
                                preparing.insert_quantifier_for_capture_id(capture_id, *quantifier);
                            }
                            _ => {
                                preparing.insert_quantifier_for_capture_id(capture_id, Quant::One);
                            }
                        },
                        Pattern::NegatedField { .. } => todo!(),
                    }
                    res = Pattern::Capture {
                        name: capture_id,
                        pat: Arc::new(res),
                    };
                } else {
                    todo!()
                }
            }
            res
        } else {
            panic!()
            // let l = n.try_get_label();
            // let l = query_store.label_store().resolve(&l.unwrap());
            // dbg!(l);
            // let l = &l[1..l.len() - 1];
            // dbg!(l);
            // let l = Ty::try_from(l).unwrap();
            // Pattern::AnonymousNode(l)
        }
    }

    pub(crate) fn preprocess_predicate(
        query_store: &SimpleStores<TStore>,
        rule: NodeIdentifier,
    ) -> Predicate<String> {
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

pub(crate) fn preprocess_capture_pred_arg(
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
