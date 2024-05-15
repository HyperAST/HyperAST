use crate::legion::TsQueryTreeGen;
use crate::types::TStore;

use hyper_ast::store::labels::LabelStore;
use hyper_ast::types::{
    HyperAST, HyperType, IterableChildren, Labeled, TypeStore, Typed, TypedHyperAST, TypedNodeStore, WithChildren
};

use hyper_ast::store::nodes::legion::NodeIdentifier;

// use hyper_ast_gen_ts_cpp::types::TStore;

// use crate::types::TStore;

use hyper_ast::store::SimpleStores;
use tree_sitter::CaptureQuantifier as Quant;

use std::collections::HashMap;
use std::sync::Arc;

// for now just uses the root types
// TODO implement approaches based on probabilitic sets
pub(crate) struct QuickTrigger<T> {
    pub(crate) root_types: Arc<[T]>,
}

pub struct PreparedMatcher<Ty> {
    pub(crate) quick_trigger: QuickTrigger<Ty>,
    pub(crate) patterns: Arc<[Pattern<Ty>]>,
    pub captures: Arc<[Capture]>,
    pub(crate) quantifiers: Arc<[HashMap<usize, tree_sitter::CaptureQuantifier>]>,
}

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

#[derive(Debug)]
pub struct Capture {
    pub name: String,
}

impl<Ty: std::fmt::Debug> std::fmt::Debug for PreparedMatcher<Ty> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedMatcher")
            .field("quick_trigger", &self.quick_trigger.root_types)
            .field("patterns", &self.patterns)
            .finish()
    }
}
impl<Ty> PreparedMatcher<Ty> {
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    pub fn capture_index_for_name(&self, name: &str) -> Option<u32> {
        dbg!(self.captures.len());
        dbg!(&self.captures[..10.min(self.captures.len())]);
        dbg!(name);
        self.captures
            .iter()
            .position(|x| x.name == name)
            .map(|x| x as u32)
    }

    pub fn capture_quantifiers(
        &self,
        index: usize,
    ) -> (impl std::ops::Index<usize, Output = tree_sitter::CaptureQuantifier> + '_) {
        // struct A([tree_sitter::CaptureQuantifier]);
        // impl std::ops::Index<usize> for &A {
        //     type Output = tree_sitter::CaptureQuantifier;

        //     fn index(&self, index: usize) -> &Self::Output {
        //         self.0.get(index).unwrap_or(&Quant::Zero)
        //     }
        // }
        // let left = self.quantifiers_skips[index];
        // let right = self.quantifiers_skips.get(index + 1).copied().unwrap();
        // let s = &self.quantifiers[left..right];
        // let s: &A = unsafe { std::mem::transmute(s) };
        // s
        struct A(HashMap<usize, tree_sitter::CaptureQuantifier>);
        impl std::ops::Index<usize> for &A {
            type Output = tree_sitter::CaptureQuantifier;

            fn index(&self, index: usize) -> &Self::Output {
                self.0.get(&index).unwrap_or(&Quant::Zero)
            }
        }
        let s = &self.quantifiers[index];
        let s: &A = unsafe { std::mem::transmute(s) };
        s
    }
}

impl<'a, Ty: for<'b> TryFrom<&'b str> + HyperType> PreparedMatcher<Ty>
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
                let res: MatchingRes<_, _> = pat.is_matching(code_store, id.clone());
                if res.matched == Quant::One {
                    return true;
                } else if res.matched == Quant::Zero {
                } else {
                    todo!("{:?}", res.matched)
                }
            }
        }
        false
    }
    pub fn is_matching_and_capture<'store, HAST, TIdN>(
        &self,
        code_store: &'store HAST,
        id: HAST::IdN,
    ) -> Option<Captured<HAST::IdN, HAST::Idx>>
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
        let mut i = 0;
        let mut j = 0;
        // 0..self.quick_trigger.root_types.len()
        let mut tt = self.quick_trigger.root_types[j];
        let mut pat: &Pattern<_> = &self.patterns[i];
        loop {
            match pat {
                Pattern::List(patts) => {
                    for pat in patts.iter() {
                        match pat {
                            Pattern::AnyNode { .. } => todo!(),
                            _ => (),
                        }
                        if t == tt {
                            let res = pat.is_matching(code_store, id.clone());
                            if res.matched == Quant::One {
                                return Some(Captured(res.captures, i));
                            } else if res.matched == Quant::Zero {
                            } else {
                                todo!("{:?}", res.matched)
                            }
                        }
                        j += 1;
                        if j < self.quick_trigger.root_types.len() {
                            tt = self.quick_trigger.root_types[j];
                        }
                    }
                    i += 1;
                    if i < self.patterns.len() {
                        pat = &self.patterns[i];
                    } else {
                        break;
                    }
                }
                Pattern::AnyNode { .. } => {
                    let res = pat.is_matching(code_store, id.clone());
                    if res.matched == Quant::One {
                        return Some(Captured(res.captures, i));
                    } else if res.matched == Quant::Zero {
                    } else {
                        todo!("{:?}", res.matched)
                    }
                    i += 1;
                    if i < self.patterns.len() {
                        pat = &self.patterns[i];
                    } else {
                        break;
                    }
                }
                _ => {
                    if t == tt {
                        let res = pat.is_matching(code_store, id.clone());
                        if res.matched == Quant::One {
                            return Some(Captured(res.captures, i));
                        } else if res.matched == Quant::Zero {
                        } else {
                            todo!("{:?}", res.matched)
                        }
                    }
                    i += 1;
                    if i < self.patterns.len() {
                        pat = &self.patterns[i];
                    } else {
                        break;
                    }
                    j += 1;
                    if j < self.quick_trigger.root_types.len() {
                        tt = self.quick_trigger.root_types[i];
                    }
                }
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct Captured<IdN, Idx>(pub Vec<CaptureRes<IdN, Idx>>, usize);
impl<IdN, Idx> Captured<IdN, Idx> {
    pub fn by_capture_id(&self, id: CaptureId) -> Option<&CaptureRes<IdN, Idx>> {
        captures(&self.0, id).next()
    }
    pub fn pattern_index(&self) -> usize {
        self.1
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
                    res.quantifiers.last_mut().unwrap().insert(name as usize,Quant::One);
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
                if let Some(prev) = patterns.pop() {
                    dbg!(&prev);
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

                    patterns.push(Pattern::Quantified {
                        quantifier,
                        pat: Arc::new(prev),
                    });
                } else {
                    break;
                }
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
                let c_id = capture_id as usize;
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
        name: CaptureId,
        pat: Arc<Pattern<Ty>>,
    },
    Predicated {
        predicate: Predicate,
        pat: Arc<Pattern<Ty>>,
    },
    AnyNode {
        children: Arc<[Pattern<Ty>]>,
    },
    List(Arc<[Pattern<Ty>]>),
    FieldDefinition {
        name: Field,
        pat: Arc<Pattern<Ty>>,
    },
    Dot,
    Quantified {
        quantifier: tree_sitter::CaptureQuantifier,
        pat: Arc<Pattern<Ty>>,
    },
    NegatedField(Field),
}

type Field = String;

type CaptureId = u32;

#[derive(Debug)]
pub(crate) enum Predicate<I = CaptureId> {
    Eq { left: I, right: I },
    EqString { left: I, right: String },
}
impl Predicate<String> {
    fn resolve_name(self, captures: &[Capture]) -> Predicate<CaptureId> {
        match self {
            Predicate::Eq { left, right } => {
                for i in 0..captures.len() {
                    if captures[i].name == left {
                        let left = i as u32;
                        for i in i..captures.len() {
                            if captures[i].name == right {
                                let right = i as u32;
                                return Predicate::Eq { left, right };
                            }
                        }
                    } else if captures[i].name == right {
                        let right = i as u32;
                        for i in i..captures.len() {
                            if captures[i].name == left {
                                let left = i as u32;
                                return Predicate::Eq { left, right };
                            }
                        }
                    }
                }
                panic!(
                    "{} and {} cannot be resolved in {:?}",
                    left, right, captures
                );
            }
            Predicate::EqString { left, right } => {
                for i in 0..captures.len() {
                    if captures[i].name == left {
                        let left = i as u32;
                        return Predicate::EqString { left, right };
                    }
                }
                panic!("{} cannot be resolved in {:?}", left, captures);
            }
        }
    }
}
#[derive(Debug)]
pub(crate) struct MatchingRes<IdN = NodeIdentifier, Idx = u16> {
    matched: tree_sitter::CaptureQuantifier,
    captures: Vec<CaptureRes<IdN, Idx>>,
}

impl<IdN, Idx> MatchingRes<IdN, Idx> {
    fn zero() -> Self {
        Self {
            matched: Quant::Zero,
            captures: Default::default(),
        }
    }

    fn capture(&self, id: CaptureId) -> Option<&CaptureRes<IdN, Idx>> {
        captures(&self.captures, id).next()
    }
}

fn captures<IdN, Idx>(
    c: &[CaptureRes<IdN, Idx>],
    id: CaptureId,
) -> impl Iterator<Item = &CaptureRes<IdN, Idx>> {
    c.iter().filter(move |x| x.id == id)
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct CaptureRes<IdN = NodeIdentifier, Idx = u16> {
    pub id: CaptureId,
    pub match_node: IdN,
    pub path: Vec<Idx>,
}

impl CaptureRes {
    #[deprecated]
    pub fn try_label_old(self) -> Option<String> {
        unimplemented!("refactor code using that")
    }
}

impl<IdN, Idx> CaptureRes<IdN, Idx> {
    pub fn try_label<'store, HAST>(&self, store: &'store HAST) -> Option<&'store str>
    where
        HAST: HyperAST<'store, IdN = IdN, Idx = Idx>,
    {
        use hyper_ast::types::LabelStore;
        use hyper_ast::types::NodeStore;
        let n = store.node_store().resolve(&self.match_node);
        let l = n.try_get_label()?;
        let l = store.label_store().resolve(l);
        Some(l)
    }
}

impl<Ty> Pattern<Ty> {
    fn is_matching<'store, HAST, TIdN>(
        &self,
        code_store: &'store HAST,
        id: HAST::IdN,
    ) -> MatchingRes<HAST::IdN, HAST::Idx>
    where
        HAST: TypedHyperAST<'store, TIdN>,
        TIdN: hyper_ast::types::NodeId<IdN = HAST::IdN>
            + hyper_ast::types::TypedNodeId<Ty = Ty>
            + 'static,
        Ty: std::fmt::Debug + Eq + Copy + HyperType,
    {
        let Some((n, _)) = code_store.typed_node_store().try_resolve(&id) else {
            dbg!();
            return MatchingRes::zero();
        };
        let t = n.get_type();
        dbg!(t);

        match self {
            Pattern::NamedNode { ty, children } => {
                if *ty != t {
                    return MatchingRes::zero();
                }
                let Some(cs) = n.children() else {
                    return MatchingRes {
                        matched: quant_from_bool(children.is_empty()),
                        captures: Default::default(),
                    };
                };
                let mut cs = cs.iter_children();
                dbg!(t);
                dbg!(n.child_count());
                let pats = &children[..];
                let mut captures = Default::default();
                if pats.is_empty() {
                    return MatchingRes {
                        matched: Quant::One,
                        captures,
                    };
                }
                let mut matched = Quant::Zero;
                let mut i = num::zero();
                let mut i_pat = 0;
                loop {
                    let Some(curr_p) = pats.get(i_pat) else {
                        if matched == Quant::ZeroOrOne {
                            matched = Quant::One;
                        }
                        return MatchingRes { matched, captures };
                    };
                    let Some(child) = cs.next() else {
                        fn is_optional<Ty>(p:&Pattern<Ty>) -> bool {
                            match p {
                                Pattern::NamedNode { .. } |
                                Pattern::AnyNode { .. } |
                                Pattern::Dot |
                                Pattern::NegatedField(_) |
                                Pattern::List(_) |
                                Pattern::AnonymousNode(_) => false,
                                Pattern::FieldDefinition { pat, .. } |
                                Pattern::Capture { pat , ..} => is_optional(pat),
                                Pattern::Predicated { predicate, pat } => todo!(),
                                Pattern::Quantified { quantifier: q, .. } => {
                                    *q == Quant::Zero ||
                                    *q == Quant::ZeroOrMore ||
                                    *q == Quant::ZeroOrOne
                                },
                            }
                        }
                        if (&pats[i_pat..]).iter().any(|p|!is_optional(p)) {
                            return MatchingRes::zero();
                        }
                        let matched = Quant::One;
                        return MatchingRes { matched, captures };
                    };
                    let Some((n, _)) = code_store.typed_node_store().try_resolve(&child) else {
                        dbg!();
                        return MatchingRes::zero();
                    };
                    let t = n.get_type();
                    dbg!(t);
                    match curr_p.is_matching(code_store, child.clone()) {
                        MatchingRes {
                            matched: Quant::One,
                            captures: mut capt,
                        } => {
                            i_pat +=1;;
                            matched = Quant::One;
                            for v in &mut capt {
                                v.path.push(i);
                            }
                            captures.extend(capt);
                            let Some((n, _)) = code_store.typed_node_store().try_resolve(&child) else {
                                dbg!();
                                return MatchingRes::zero();
                            };
                            let t = n.get_type();
                            dbg!(t);
                        }
                        MatchingRes {
                            matched: Quant::Zero,
                            ..
                        } => {
                            let Some((n, _)) = code_store.typed_node_store().try_resolve(&child) else {
                                dbg!();
                                return MatchingRes::zero();
                            };
                            let t = n.get_type();
                            dbg!(t);
                        }
                        MatchingRes {
                            matched: Quant::ZeroOrOne,
                            ..
                        } => {
                            matched = Quant::ZeroOrOne;
                            let Some((n, _)) = code_store.typed_node_store().try_resolve(&child) else {
                                dbg!();
                                return MatchingRes::zero();
                            };
                            let t = n.get_type();
                            dbg!(t);
                        }
                        MatchingRes { matched, .. } => todo!("{:?}", matched),
                    }
                    i += num::one();
                }
            }
            Pattern::AnonymousNode(ty) => MatchingRes {
                matched: quant_from_bool(*ty == t),
                captures: Default::default(),
            },
            Pattern::Capture { name, pat } => match pat.is_matching(code_store, id.clone()) {
                MatchingRes {
                    matched: Quant::One,
                    mut captures,
                } => {
                    let name = name.clone();
                    let n = code_store.typed_node_store().try_resolve(&id).unwrap().0;
                    use hyper_ast::types::Tree;
                    // let v = if !n.has_children() {
                    //     let l = n.try_get_label().unwrap();
                    //     use hyper_ast::types::LabelStore;
                    //     let l = code_store.label_store().resolve(l);
                    //     CaptureRes::Label(l.to_owned())
                    // } else {
                    //     CaptureRes::Node
                    // };
                    let v = CaptureRes {
                        id: name,
                        match_node: id,
                        path: vec![],
                    };
                    captures.push(v);
                    MatchingRes {
                        matched: Quant::One,
                        captures,
                    }
                }
                MatchingRes {
                    matched: Quant::Zero,
                    ..
                } => MatchingRes::zero(),
                MatchingRes {
                    matched: Quant::ZeroOrOne,
                    mut captures,
                } => {
                    let name = name.clone();
                    let v = CaptureRes {
                        id: name,
                        match_node: id,
                        path: vec![],
                    };
                    captures.push(v);
                    MatchingRes {
                        matched: Quant::ZeroOrOne,
                        captures,
                    }
                }
                MatchingRes { matched, .. } => todo!("{:?}", matched),
            },
            Pattern::Predicated { predicate, pat } => match predicate {
                Predicate::Eq { left, right } => {
                    let matching_res = pat.is_matching(code_store, id);
                    if matching_res.matched == Quant::One {
                        let matched = matching_res.capture(*left).map_or(false, |x| {
                            Some(&x.path) == matching_res.capture(*right).map(|x| &x.path)
                        });
                        let captures = if matched {
                            matching_res.captures
                        } else {
                            Default::default()
                        };
                        let matched = quant_from_bool(matched);
                        MatchingRes { matched, captures }
                    } else {
                        MatchingRes::zero()
                    }
                }
                Predicate::EqString { left, right } => {
                    let matching_res = pat.is_matching(code_store, id);
                    if matching_res.matched == Quant::One {
                        let Some(capture) = matching_res.capture(*left) else {
                            return MatchingRes::zero();
                        };
                        let left = capture.try_label(code_store).unwrap();
                        let matched = left == right;
                        let captures = if matched {
                            matching_res.captures
                        } else {
                            Default::default()
                        };
                        let matched = quant_from_bool(matched);
                        MatchingRes { matched, captures }
                    } else if matching_res.matched == Quant::Zero {
                        MatchingRes::zero()
                    } else {
                        todo!("{:?}", matching_res.matched)
                    }
                }
            },
            Pattern::AnyNode { children } => {
                let Some(cs) = n.children() else {
                    return MatchingRes {
                        matched: quant_from_bool(children.is_empty() && !t.is_spaces() && !t.is_syntax()),
                        captures: Default::default(),
                    };
                };
                let mut cs = cs.iter_children();
                let mut pats = children.iter().peekable();
                let mut captures = Default::default();
                if pats.peek().is_none() {
                    return MatchingRes {
                        matched: Quant::One,
                        captures,
                    };
                }
                let mut matched = Quant::Zero;
                let mut i = num::zero();
                loop {
                    let Some(curr_p) = pats.peek() else {
                        return MatchingRes { matched, captures };
                    };
                    let Some(child) = cs.next() else {
                        return MatchingRes::zero();
                    };
                    match curr_p.is_matching(code_store, child.clone()) {
                        MatchingRes {
                            matched: Quant::One,
                            captures: mut capt,
                        } => {
                            pats.next();
                            matched = Quant::One;

                            for v in &mut capt {
                                v.path.push(i);
                            }
                            captures.extend(capt);
                        }
                        MatchingRes {
                            matched: Quant::Zero,
                            ..
                        } => {}
                        MatchingRes { matched, .. } => todo!("{:?}", matched),
                    }
                    i += num::one();
                }
            }
            Pattern::List(_) => todo!(),
            Pattern::FieldDefinition { name, pat } => {
                // TODO check field name
                // TODO need to add them to hyperast
                pat.is_matching(code_store, id)
            }
            Pattern::Dot { .. } => todo!(),
            Pattern::Quantified { quantifier, pat } => match quantifier {
                Quant::ZeroOrOne => {
                    let MatchingRes { matched, captures } = pat.is_matching(code_store, id);
                    let matched = if matched == Quant::One {
                        Quant::One
                    } else {
                        Quant::ZeroOrOne
                    };
                    MatchingRes { matched, captures }
                }
                Quant::ZeroOrMore => todo!("{:?}", pat),
                Quant::OneOrMore => todo!("{:?}", pat),
                Quant::One => todo!("{:?}", pat),
                Quant::Zero => todo!("{:?}", pat),
            },
            Pattern::NegatedField(_) => todo!(),
        }
    }

    fn named(ty: Option<Ty>, patterns: Vec<Pattern<Ty>>) -> Pattern<Ty>
    where
        Ty: for<'b> TryFrom<&'b str> + std::fmt::Debug,
        for<'b> <Ty as TryFrom<&'b str>>::Error: std::fmt::Debug,
    {
        if let Some(ty) = ty {
            Self::NamedNode {
                ty,
                children: patterns.into(),
            }
        } else {
            Self::AnyNode {
                children: patterns.into(),
            }
        }
    }
}

fn quant_from_bool(b: bool) -> tree_sitter::CaptureQuantifier {
    if b {
        Quant::One
    } else {
        Quant::Zero
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
        stores,
        md_cache: &mut md_cache,
    };

    let tree = match crate::legion::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => {
            eprintln!("{}", t.root_node().to_sexp());
            t
        }
    };
    dbg!(tree.root_node().to_sexp());
    let full_node = query_tree_gen.generate_file(b"", text, tree.walk());
    eprintln!(
        "{}",
        hyper_ast::nodes::SyntaxSerializer::new(stores, full_node.local.compressed_node)
    );
    full_node.local.compressed_node
}
