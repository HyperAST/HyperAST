#![allow(unused)]
use crate::auto::tsq_ser_meta::Converter;

use super::{CaptureRes, Captured, MatchingRes, Pattern, Predicate, PreparedMatcher};

use hyperast::types::TypeStore;
use hyperast::types::TypeTrait;
use tree_sitter::CaptureQuantifier as Quant;

use hyperast::types::HyperType;
use hyperast::types::TypedHyperAST;
use hyperast::types::{Childrn, Typed, TypedNodeStore, WithChildren};

impl<'a, Ty: TypeTrait, C: Converter<Ty = Ty>> PreparedMatcher<Ty, C> {
    pub fn is_matching<'store, HAST, TIdN>(&self, code_store: &'store HAST, id: HAST::IdN) -> bool
    where
        HAST: TypedHyperAST<TIdN>,
        TIdN: hyperast::types::TypedNodeId<Ty = Ty> + 'static,
        Ty: std::fmt::Debug + Eq + Copy,
    {
        let Some((n, t)) = code_store.try_resolve(&id) else {
            return false;
        };
        let t = n.get_type();
        // let t = code_store.resolve_type(&id);
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
        HAST: TypedHyperAST<TIdN>,
        HAST::TS: TypeStore,
        TIdN: hyperast::types::TypedNodeId<Ty = Ty> + 'static,
        Ty: std::fmt::Debug + Eq + Copy,
    {
        let Some((n, _)) = code_store.try_resolve(&id) else {
            return None;
        };
        let t = n.get_type();
        // let t = code_store.resolve_type(&id);
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

impl<Ty> Pattern<Ty> {
    pub(crate) fn is_matching<'store, HAST, TIdN>(
        &self,
        code_store: &'store HAST,
        id: HAST::IdN,
    ) -> MatchingRes<HAST::IdN, HAST::Idx>
    where
        HAST: TypedHyperAST<TIdN>,
        TIdN: hyperast::types::TypedNodeId<Ty = Ty> + 'static,
        Ty: std::fmt::Debug + TypeTrait,
    {
        let Some((n, _)) = code_store.try_resolve(&id) else {
            dbg!();
            return MatchingRes::zero();
        };
        let t = n.get_type();
        dbg!(t);

        match self {
            Pattern::SupNamedNode { sup, ty, children } => {
                todo!()
            }
            Pattern::NamedNode { ty, children } => {
                if t.is_hidden() && *ty != t {
                    dbg!(ty);
                    let cs = n.children().unwrap().iter_children();
                    for child in cs {
                        let m_res = self.is_matching(code_store, child.clone());
                        if m_res.matched == Quant::One {
                            return m_res;
                        } else if m_res.matched == Quant::Zero {
                        } else {
                            todo!(
                                "{:?} {:?} {:?} {}",
                                ty,
                                t,
                                m_res.matched,
                                m_res.captures.len()
                            );
                        }
                    }
                    return MatchingRes::zero();
                }
                if *ty != t {
                    return MatchingRes::zero();
                }
                let Some(cs) = n.children() else {
                    dbg!(ty, t);
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
                let mut immediate = false;
                loop {
                    let Some(curr_p) = pats.get(i_pat) else {
                        if matched == Quant::ZeroOrOne {
                            matched = Quant::One;
                        }
                        return MatchingRes { matched, captures };
                    };
                    let Some(child) = cs.next() else {
                        pub(crate) fn is_optional<Ty>(p: &Pattern<Ty>) -> bool {
                            match p {
                                Pattern::NamedNode { .. }
                                | Pattern::SupNamedNode { .. }
                                | Pattern::AnyNode { .. }
                                | Pattern::Dot
                                | Pattern::NegatedField(_)
                                | Pattern::List(_)
                                | Pattern::AnonymousNode(_) => false,
                                Pattern::FieldDefinition { pat, .. }
                                | Pattern::Capture { pat, .. } => is_optional(pat),
                                Pattern::Predicated { predicate, pat } => todo!(),
                                Pattern::Quantified { quantifier: q, .. } => {
                                    *q == Quant::Zero
                                        || *q == Quant::ZeroOrMore
                                        || *q == Quant::ZeroOrOne
                                }
                            }
                        }
                        if (&pats[i_pat..]).iter().any(|p| !is_optional(p)) {
                            return MatchingRes::zero();
                        }
                        let matched = Quant::One;
                        return MatchingRes { matched, captures };
                    };
                    let Some((n, _)) = code_store.try_resolve(&child) else {
                        dbg!();
                        return MatchingRes::zero();
                    };
                    let t = n.get_type();
                    if t.is_spaces() {
                        continue;
                    }
                    match curr_p {
                        Pattern::Dot => {
                            immediate = true;
                            continue;
                        }
                        _ => (),
                    }
                    dbg!(t);
                    match curr_p.is_matching(code_store, child.clone()) {
                        MatchingRes {
                            matched: Quant::One,
                            captures: mut capt,
                        } => {
                            immediate = false;
                            i_pat += 1;
                            matched = Quant::One;
                            for v in &mut capt {
                                v.path.push(i);
                            }
                            captures.extend(capt);
                            let Some((n, _)) = code_store.try_resolve(&child) else {
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
                            if immediate {
                                return MatchingRes::zero();
                            }
                            let Some((n, _)) = code_store.try_resolve(&child) else {
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
                            immediate = false;
                            matched = Quant::ZeroOrOne;
                            let Some((n, _)) = code_store.try_resolve(&child) else {
                                dbg!();
                                return MatchingRes::zero();
                            };
                            let t = n.get_type();
                            dbg!(t);
                        }
                        MatchingRes { matched, .. } => todo!("{:?}", matched),
                    }
                    assert_eq!(immediate, false);
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
                    let n = code_store.try_resolve(&id).unwrap().0;
                    use hyperast::types::Tree;
                    // let v = if !n.has_children() {
                    //     let l = n.try_get_label().unwrap();
                    //     use hyperast::types::LabelStore;
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
                        matched: quant_from_bool(
                            children.is_empty() && !t.is_spaces() && !t.is_syntax(),
                        ),
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

    pub(crate) fn named(ty: Option<Ty>, patterns: Vec<Pattern<Ty>>) -> Pattern<Ty> {
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
    if b { Quant::One } else { Quant::Zero }
}
