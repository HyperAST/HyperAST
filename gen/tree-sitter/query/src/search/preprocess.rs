use std::collections::HashMap;
use std::sync::Arc;

use super::{Capture, Pattern, Predicate, PreparedMatcher, QuickTrigger};

use hyperast::store::SimpleStores;
use hyperast::store::nodes::legion::NodeIdentifier;
use hyperast::types::{Childrn, HyperAST, Labeled, Typed, WithChildren};

use tree_sitter::CaptureQuantifier as Quant;

use crate::auto::tsq_ser_meta::Converter;
use crate::types::{TStore, TsQuery};

pub struct PreparingMatcher<Ty, C> {
    root_types: Vec<Ty>,
    patterns: Vec<Pattern<Ty>>,
    captures: Vec<Capture>,
    quantifiers: Vec<HashMap<usize, tree_sitter::CaptureQuantifier>>,
    converter: C,
}

impl<Ty, C> PreparingMatcher<Ty, C> {
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

impl<Ty, C: Default> Default for PreparingMatcher<Ty, C> {
    fn default() -> Self {
        Self {
            root_types: Default::default(),
            patterns: Default::default(),
            captures: Default::default(),
            quantifiers: Default::default(),
            converter: Default::default(),
        }
    }
}

impl<Ty, C> From<PreparingMatcher<Ty, C>> for PreparedMatcher<Ty, C> {
    fn from(value: PreparingMatcher<Ty, C>) -> Self {
        assert_eq!(value.patterns.len(), value.quantifiers.len());
        dbg!(&value.quantifiers);
        Self {
            quick_trigger: QuickTrigger {
                root_types: value.root_types.into(),
            },
            patterns: value.patterns.into(),
            captures: value.captures.into(),
            quantifiers: value.quantifiers.into(),
            converter: value.converter,
        }
    }
}

impl<'a, Ty, C: Converter<Ty = Ty>> PreparedMatcher<Ty, C> {
    pub fn new(query_store: &'a SimpleStores<TStore>, query: NodeIdentifier) -> Self {
        let preparing = Self::new_aux(query_store, query);

        preparing.into()
    }

    pub(crate) fn new_aux(
        query_store: &'a SimpleStores<TStore>,
        query: legion::Entity,
    ) -> PreparingMatcher<Ty, C> {
        use crate::types::Type;
        use hyperast::types::LabelStore;
        let mut res = PreparingMatcher::default();
        let n = query_store.node_store.resolve(query);
        let t: Type = *query_store.resolve_type(&query);
        assert_eq!(t, Type::Program);
        let Some(cs) = n.children() else { return res };
        for rule_id in cs.iter_children() {
            let rule = query_store.node_store.resolve(rule_id);
            let t = *query_store.resolve_type(&rule_id);
            if t == Type::NamedNode {
                res.quantifiers.push(Default::default());
                let ty = rule.child(&1).unwrap();
                assert_eq!(*query_store.resolve_type(&ty), Type::Identifier);
                let ty = query_store.node_store.resolve(ty);
                let l = ty.try_get_label();
                let l = query_store.label_store.resolve(&l.unwrap());
                let l = C::conv(l).expect("the node type does not exist");
                let patt = Self::process_named_node(query_store, rule_id, &mut res).into();
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
                    let i = query_store.node_store.resolve(id);
                    let i_ty = *query_store.resolve_type(&id);
                    if i_ty == Type::Capture {
                        let mut cs = i.children().unwrap().iter_children();
                        let n_id = cs.next().unwrap();
                        let t = *query_store.resolve_type(&n_id);
                        assert_eq!(t, Type::At);
                        let name = cs.next().unwrap();
                        let ty = query_store.node_store.resolve(name);
                        assert_eq!(*query_store.resolve_type(&name), Type::Identifier);
                        let name = ty.try_get_label().unwrap();
                        let name = query_store.label_store.resolve(&name);
                        dbg!(name);
                        dbg!(&res.captures);
                        capture.push(res.add_or_insert_capture(name));
                        continue;
                    }
                    if i_ty == Type::LBracket {
                        continue;
                    }
                    if i_ty == Type::Spaces {
                        continue;
                    }
                    if i_ty == Type::RBracket {
                        continue;
                    }
                    if i_ty == Type::Comment {
                        continue;
                    }
                    assert!(capture.is_empty(), "{}", i.get_type());
                    if i_ty == Type::NamedNode {
                        let ty = i.child(&1).unwrap();
                        let ty = query_store
                            .node_store
                            .try_resolve_typed3::<TsQuery>(&ty)
                            .unwrap();
                        assert_eq!(ty.get_type(), Type::Identifier);
                        let l = ty.try_get_label();
                        let l = query_store.label_store.resolve(&l.unwrap());
                        let l = C::conv(l).expect("the node type does not exist");
                        let patt = Self::process_named_node(query_store, id, &mut res);
                        res.root_types.push(l);
                        patterns.push(patt);
                    } else {
                        assert_eq!(i_ty, Type::Identifier);
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

                let predicate = Self::preprocess_predicate(query_store, rule_id);
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
        query_store: &'a SimpleStores<TStore>,
        rule: NodeIdentifier,
        preparing: &mut PreparingMatcher<Ty, C>,
    ) -> Pattern<Ty> {
        use crate::types::Type;
        use hyperast::types::LabelStore;
        let mut patterns = vec![];
        let n = query_store
            .node_store
            .try_resolve_typed3::<TsQuery>(&rule)
            .unwrap();
        let t = n.get_type();
        assert_eq!(t, Type::NamedNode);
        let mut cs = n.children().unwrap().iter_children().peekable();
        cs.next().unwrap();
        let ty = cs.next().unwrap();
        let ty = query_store
            .node_store
            .try_resolve_typed3::<TsQuery>(&ty)
            .unwrap();
        let mut supertype = None;
        let mut l = if ty.get_type() == Type::Inderscore {
            None
        } else {
            assert_eq!(Type::Identifier, ty.get_type());
            let l = ty.try_get_label();
            let l = query_store.label_store.resolve(&l.unwrap());
            let l = C::conv(l).expect("the node type does not exist");
            Some(l)
        };
        loop {
            let Some(rule_id) = cs.peek() else { break };
            let rule = query_store
                .node_store
                .try_resolve_typed3::<TsQuery>(rule_id)
                .unwrap();
            let t = rule.get_type();
            if t == Type::NamedNode {
                patterns.push(Self::process_named_node(query_store, *rule_id, preparing).into())
            } else if t == Type::Spaces {
            } else if t == Type::RParen {
            } else if t == Type::AnonymousNode {
                patterns.push(Self::process_anonymous_node(query_store, *rule_id, preparing).into())
            } else if t == Type::Capture {
                break;
            } else if t == Type::Predicate {
                let prev = patterns.pop().expect("predicate must be preceded by node");
                let predicate = Self::preprocess_predicate(query_store, *rule_id);
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
                    .push(Self::process_field_definition(query_store, *rule_id, preparing).into())
            } else if t == Type::Dot {
                patterns.push(Pattern::Dot);
            } else if t == Type::NegatedField {
                let mut cs = rule.children().unwrap().iter_children();
                let bang = cs.next().unwrap();
                let bang = query_store
                    .node_store
                    .try_resolve_typed3::<TsQuery>(&bang)
                    .unwrap();
                assert_eq!(Type::Bang, bang.get_type());
                let field_name = cs.next().unwrap();
                let field_name = query_store
                    .node_store
                    .try_resolve_typed3::<TsQuery>(&field_name)
                    .unwrap();
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
                    .try_resolve_typed3::<TsQuery>(rule_id)
                    .unwrap();
                assert_eq!(Type::Identifier, ty.get_type());
                // TODO properly filter using the supertype
                // NOTE for now just ignore supertype
                supertype = l;
                l = {
                    let l = ty.try_get_label();
                    let l = query_store.label_store.resolve(&l.unwrap());
                    let l = C::conv(l).expect("the node type does not exist");
                    Some(l)
                };
            } else {
                todo!("{}", t)
            }
            cs.next();
        }

        let mut res = if let Some(sup) = supertype {
            if let Some(ty) = l {
                Pattern::SupNamedNode {
                    sup,
                    ty,
                    children: patterns.into(),
                }
            } else {
                Pattern::named(Some(sup), patterns)
            }
        } else {
            Pattern::named(l, patterns)
        };
        loop {
            let Some(rule_id) = cs.peek() else {
                return res;
            };
            let n = query_store
                .node_store
                .try_resolve_typed3::<TsQuery>(rule_id)
                .unwrap();
            let t = n.get_type();
            if t == Type::Capture {
                let mut cs = n.children().unwrap().iter_children();
                let n = query_store
                    .node_store
                    .try_resolve_typed3::<TsQuery>(&cs.next().unwrap())
                    .unwrap();
                let t = n.get_type();
                assert_eq!(t, Type::At);
                let name = cs.next().unwrap();
                let ty = query_store
                    .node_store
                    .try_resolve_typed3::<TsQuery>(&name)
                    .unwrap();
                assert_eq!(ty.get_type(), Type::Identifier);
                let name = ty.try_get_label().unwrap();
                let name = query_store.label_store.resolve(&name);
                dbg!(name);
                let capture_id = preparing.add_or_insert_capture(name);
                dbg!(capture_id);
                // dbg!(&res);
                match &res {
                    Pattern::SupNamedNode { .. }
                    | Pattern::NamedNode { .. }
                    | Pattern::AnyNode { .. } => {
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
                    .try_resolve_typed3::<TsQuery>(&cs.next().unwrap())
                    .unwrap();

                let quantifier = n.get_type(); //query_store.resolve_type(&n);
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
                .try_resolve_typed3::<TsQuery>(&rule_id)
                .unwrap();
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
        query_store: &'a SimpleStores<TStore>,
        rule: NodeIdentifier,
        preparing: &mut PreparingMatcher<Ty, C>,
    ) -> Pattern<Ty> {
        use crate::types::Type;
        use hyperast::types::LabelStore;
        let n = query_store
            .node_store
            .try_resolve_typed3::<TsQuery>(&rule)
            .unwrap();
        let t = n.get_type();
        assert_eq!(t, Type::FieldDefinition);
        let mut cs = n.children().unwrap().iter_children().peekable();
        let field_name = cs.next().unwrap();
        let field_name = query_store
            .node_store
            .try_resolve_typed3::<TsQuery>(&field_name)
            .unwrap();
        let field_name = field_name.try_get_label();
        let field_name = query_store.label_store.resolve(&field_name.unwrap());
        let colon = cs.next().unwrap();
        let colon = query_store
            .node_store
            .try_resolve_typed3::<TsQuery>(&colon)
            .unwrap();
        assert_eq!(colon.get_type(), Type::Colon);
        let pat = loop {
            let Some(rule_id) = cs.next() else { panic!() };
            let rule = query_store
                .node_store
                .try_resolve_typed3::<TsQuery>(&rule_id)
                .unwrap();
            let t = rule.get_type();
            if t == Type::NamedNode {
                assert!(cs.next().is_none());
                break Self::process_named_node(query_store, rule_id, preparing).into();
            } else if t == Type::Spaces {
            } else if t == Type::AnonymousNode {
                break Self::process_anonymous_node(query_store, rule_id, preparing).into();
            } else {
                todo!("{}", t)
            }
        };
        loop {
            let Some(rule_id) = cs.next() else { break };
            let rule = query_store
                .node_store
                .try_resolve_typed3::<TsQuery>(&rule_id)
                .unwrap();
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
        preparing: &mut PreparingMatcher<Ty, C>,
    ) -> Pattern<Ty> {
        use crate::types::Type;
        use hyperast::types::LabelStore;
        let n = query_store
            .node_store()
            .try_resolve_typed3::<TsQuery>(&rule)
            .unwrap();
        let t = n.get_type();
        assert_eq!(t, Type::AnonymousNode);
        if let Some(cs) = n.children() {
            let mut cs = cs.iter_children();
            let l = loop {
                let rule_id = cs.next().unwrap();
                let rule = query_store
                    .node_store
                    .try_resolve_typed3::<TsQuery>(&rule_id)
                    .unwrap();
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
                    let l = C::conv(l).unwrap();
                    break l;
                } else {
                    todo!()
                }
            };
            let mut res = Pattern::AnonymousNode(l);
            for rule_id in cs {
                let rule = query_store
                    .node_store
                    .try_resolve_typed3::<TsQuery>(&rule_id)
                    .unwrap();
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
                        .try_resolve_typed3::<TsQuery>(&cs.next().unwrap())
                        .unwrap();
                    let quantifier = n.get_type(); //query_store.resolve_type(&n);
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
                        .try_resolve_typed3::<TsQuery>(&cs.next().unwrap())
                        .unwrap();
                    let t = n.get_type();
                    assert_eq!(t, Type::At);
                    let name = cs.next().unwrap();
                    let ty = query_store
                        .node_store
                        .try_resolve_typed3::<TsQuery>(&name)
                        .unwrap();
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
                        Pattern::SupNamedNode { .. } => todo!(),
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
        use crate::types::Type;
        use hyperast::types::LabelStore;
        let n = query_store
            .node_store()
            .try_resolve_typed3::<TsQuery>(&rule)
            .unwrap();
        let t = n.get_type();
        assert_eq!(t, Type::Predicate);
        let mut cs = n.children().unwrap().iter_children();
        cs.next().unwrap();
        let sharp = cs.next().unwrap();
        let sharp = query_store
            .node_store
            .try_resolve_typed3::<TsQuery>(&sharp)
            .unwrap();
        let t = sharp.get_type();
        assert_eq!(t, Type::Sharp);

        let pred = cs.next().unwrap();
        let pred = query_store
            .node_store
            .try_resolve_typed3::<TsQuery>(&pred)
            .unwrap();
        let t = pred.get_type();
        assert_eq!(t, Type::Identifier);

        let l = pred.try_get_label();
        let l = query_store.label_store().resolve(&l.unwrap());
        match l {
            "eq" => {
                let pred = cs.next().unwrap();
                let pred = query_store
                    .node_store
                    .try_resolve_typed3::<TsQuery>(&pred)
                    .unwrap();
                let t = pred.get_type();
                assert_eq!(t, Type::PredicateType);
                let l = pred.try_get_label();
                let l = query_store.label_store().resolve(&l.unwrap());
                assert_eq!(l, "?");
                for rule_id in cs {
                    let rule = query_store
                        .node_store
                        .try_resolve_typed3::<TsQuery>(&rule_id)
                        .unwrap();
                    let t = rule.get_type();
                    if t == Type::Parameters {
                        let mut cs = rule.children().unwrap().iter_children();
                        let left = cs.next().unwrap();
                        let left = query_store
                            .node_store
                            .try_resolve_typed3::<TsQuery>(&left)
                            .unwrap();
                        let left = match left.get_type() {
                            Type::Capture => preprocess_capture_pred_arg(left, query_store),
                            t => todo!("{}", t),
                        };
                        {
                            let center = cs.next().unwrap();
                            let center = query_store
                                .node_store
                                .try_resolve_typed3::<TsQuery>(&center)
                                .unwrap();
                            let t = center.get_type();
                            assert_eq!(t, Type::Spaces);
                        }
                        let right = cs.next().unwrap();
                        let right = query_store
                            .node_store
                            .try_resolve_typed3::<TsQuery>(&right)
                            .unwrap();
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
    arg: hyperast::store::nodes::legion::TypedNode<
        hyperast::store::nodes::legion::HashedNodeRef<'_, NodeIdentifier>,
        crate::types::Type,
    >,
    query_store: &SimpleStores<TStore>,
) -> String {
    use crate::types::Type;
    use hyperast::types::LabelStore;
    if let Type::Capture = arg.get_type() {
        let mut cs = arg.children().unwrap().iter_children();
        assert_eq!(2, cs.len());
        let at = cs.next().unwrap();
        let at = query_store
            .node_store
            .try_resolve_typed3::<TsQuery>(&at)
            .unwrap();
        let t = at.get_type();
        assert_eq!(t, Type::At);
        let id = cs.next().unwrap();
        let id = query_store
            .node_store
            .try_resolve_typed3::<TsQuery>(&id)
            .unwrap();
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
