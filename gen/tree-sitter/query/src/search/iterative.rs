#![allow(unused)]
use crate::auto::tsq_ser_meta::Converter;

use super::*;
use hyperast::types::{
    Childrn, HyperType, NodeStore, TypeStore, Typed, TypedHyperAST, WithChildren,
};

pub(crate) struct MatchingIter<
    'a,
    'store,
    HAST: TypedHyperAST<TIdN>,
    TIdN: 'store + hyperast::types::TypedNodeId,
    C: Converter,
> {
    slf: &'a PreparedMatcher<TIdN::Ty, C>,
    code_store: &'store HAST,
    stack: Vec<State<HAST::IdN, HAST::Idx, TIdN::Ty>>,
}
struct State<IdN, Idx, Ty> {
    s: S<IdN, Idx, Ty>,
    p: Arc<Pattern<Ty>>,
}

#[derive(Clone)]
struct S<IdN, Idx, Ty> {
    path: Vec<Idx>,
    id: IdN,
    offset: u16,
    sup: Option<Ty>,
    is_immediate: bool,
    parent: usize,
    capture_ids: Vec<u32>,
    pred: Vec<Predicate>,
    neg: Vec<String>,
}

impl<
    'a,
    'store,
    HAST: TypedHyperAST<TIdN>,
    TIdN: 'store + hyperast::types::TypedNodeId,
    C: Converter,
> Iterator for MatchingIter<'a, 'store, HAST, TIdN, C>
{
    type Item = MatchingRes<HAST::IdN, HAST::Idx>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(state) = self.stack.pop() else {
                return None;
            };
            let s = state.s;
            let res = match state.p.as_ref() {
                Pattern::SupNamedNode { sup, ty, children } => todo!(),
                Pattern::NamedNode { ty, children } => self.is_matching_named_node(s, ty, children),
                Pattern::AnyNode { children } => self.is_matching_any_node(s, children),
                Pattern::AnonymousNode(ty) => self.is_matching_anonymous_node(s, ty),
                Pattern::Capture { name, pat } => self.is_matching_capture(s, name, pat),
                Pattern::Predicated { predicate, pat } => {
                    self.is_matching_predicated(s, predicate, pat)
                }
                Pattern::List(list) => self.is_matching_list(s, list),
                Pattern::FieldDefinition { name, pat } => self.is_matching_field(s, name, pat),
                Pattern::Dot => self.is_matching_dot(s),
                Pattern::Quantified { quantifier, pat } => {
                    self.is_matching_quantified(s, quantifier, pat)
                }
                Pattern::NegatedField(name) => self.is_matching_negated(s, name),
            };

            if res.matched == Quant::One {
                return Some(res);
            } else if res.matched == Quant::Zero {
                return Some(res);
            } else {
                todo!("{:?}", res.matched)
            }
        }
    }
}

impl<
    'a,
    'store,
    HAST: TypedHyperAST<TIdN>,
    TIdN: 'store + hyperast::types::TypedNodeId,
    C: Converter,
> MatchingIter<'a, 'store, HAST, TIdN, C>
where
// HAST::TS::Ty: TIdN::Ty,
{
    fn is_matching_anonymous_node(
        &mut self,
        s: S<HAST::IdN, HAST::Idx, TIdN::Ty>,
        ty: &TIdN::Ty,
    ) -> MatchingRes<HAST::IdN, HAST::Idx> {
        let t = self.code_store.try_resolve(&s.id).unwrap().0.get_type();
        MatchingRes {
            matched: quant_from_bool(*ty == t),
            captures: Default::default(),
        }
    }
    fn is_matching_named_node(
        &mut self,
        s: S<HAST::IdN, HAST::Idx, TIdN::Ty>,
        ty: &TIdN::Ty,
        children: &Arc<[Pattern<TIdN::Ty>]>,
    ) -> MatchingRes<HAST::IdN, HAST::Idx> {
        let n = self.code_store.node_store().resolve(&s.id);
        let t = self.code_store.try_resolve(&s.id).unwrap().0.get_type();
        if t.is_hidden() && *ty != t {
            dbg!(ty);
            return self.handle_hidden(n, *ty, t);
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
        let mut i: HAST::Idx = num::zero();
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
                        Pattern::FieldDefinition { pat, .. } | Pattern::Capture { pat, .. } => {
                            is_optional(pat)
                        }
                        Pattern::Predicated { predicate, pat } => todo!(),
                        Pattern::Quantified { quantifier: q, .. } => {
                            *q == Quant::Zero || *q == Quant::ZeroOrMore || *q == Quant::ZeroOrOne
                        }
                    }
                }
                if (&pats[i_pat..]).iter().any(|p| !is_optional(p)) {
                    return MatchingRes::zero();
                }
                let matched = Quant::One;
                return MatchingRes { matched, captures };
            };
            todo!("")
            // let Some((n, _)) = self.code_store.typed_node_store().try_resolve(&child) else {
            //     dbg!();
            //     return MatchingRes::zero();
            // };
            // let t = n.get_type();
            // if t.is_spaces() {
            //     continue;
            // }
            // match curr_p {
            //     Pattern::Dot => {
            //         immediate = true;
            //         continue;
            //     }
            //     _ => (),
            // }
            // dbg!(t);
            // todo!("call to self.next()?");
            // assert_eq!(immediate, false);
            // i += num::one();
        }

        todo!()
    }
    fn is_matching_any_node(
        &mut self,
        s: S<HAST::IdN, HAST::Idx, TIdN::Ty>,
        children: &Arc<[Pattern<TIdN::Ty>]>,
    ) -> MatchingRes<HAST::IdN, HAST::Idx> {
        todo!()
    }
    fn is_matching_capture(
        &mut self,
        mut s: S<HAST::IdN, HAST::Idx, TIdN::Ty>,
        name: &u32,
        pat: &Arc<Pattern<TIdN::Ty>>,
    ) -> MatchingRes<HAST::IdN, HAST::Idx> {
        s.capture_ids.push(*name);
        self.stack.push(State { s, p: pat.clone() });
        MatchingRes::zero()
    }
    fn is_matching_predicated(
        &mut self,
        mut s: S<HAST::IdN, HAST::Idx, TIdN::Ty>,
        predicate: &Predicate,
        pat: &Arc<Pattern<TIdN::Ty>>,
    ) -> MatchingRes<HAST::IdN, HAST::Idx> {
        s.pred.push(predicate.clone());
        self.stack.push(State { s, p: pat.clone() });
        MatchingRes::zero()
    }
    fn is_matching_field(
        &mut self,
        s: S<HAST::IdN, HAST::Idx, TIdN::Ty>,
        name: &str,
        pat: &Arc<Pattern<TIdN::Ty>>,
    ) -> MatchingRes<HAST::IdN, HAST::Idx> {
        todo!()
    }
    fn is_matching_quantified(
        &mut self,
        s: S<HAST::IdN, HAST::Idx, TIdN::Ty>,
        quantifier: &Quant,
        pat: &Arc<Pattern<TIdN::Ty>>,
    ) -> MatchingRes<HAST::IdN, HAST::Idx> {
        todo!()
    }
    fn is_matching_list(
        &mut self,
        s: S<HAST::IdN, HAST::Idx, TIdN::Ty>,
        list: &Arc<[Pattern<TIdN::Ty>]>,
    ) -> MatchingRes<HAST::IdN, HAST::Idx> {
        for pat in list.as_ref() {
            let s = s.clone();
            self.stack.push(State {
                s,
                p: Arc::new(pat.clone()),
            });
        }
        MatchingRes::zero()
    }
    fn is_matching_negated(
        &mut self,
        s: S<HAST::IdN, HAST::Idx, TIdN::Ty>,
        name: &str,
    ) -> MatchingRes<HAST::IdN, HAST::Idx> {
        todo!()
        // self.stack[s.parent].negated.push(name.to_string());
        // MatchingRes::zero()
    }
    fn is_matching_dot(
        &mut self,
        s: S<HAST::IdN, HAST::Idx, TIdN::Ty>,
    ) -> MatchingRes<HAST::IdN, HAST::Idx> {
        self.stack[s.parent].s.is_immediate = true;
        MatchingRes::zero()
    }

    fn handle_hidden(
        &mut self,
        n: <HAST as hyperast::types::AstLending<'_>>::RT,
        ty: TIdN::Ty,
        t: TIdN::Ty,
    ) -> MatchingRes<HAST::IdN, HAST::Idx> {
        let cs = n.children().unwrap().iter_children();
        for child in cs {
            let m_res: MatchingRes<HAST::IdN, HAST::Idx> = todo!();
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
        MatchingRes::zero()
    }
}

impl<
    'a,
    'store,
    HAST: TypedHyperAST<TIdN>,
    TIdN: 'store + hyperast::types::TypedNodeId,
    C: Converter,
> MatchingIter<'a, 'store, HAST, TIdN, C>
{
    fn new(slf: &'a PreparedMatcher<TIdN::Ty, C>, code_store: &'store HAST, id: HAST::IdN) -> Self {
        Self {
            slf,
            code_store,
            stack: vec![],
        }
    }
}

pub fn is_matching<'a, 'store, HAST, TIdN, Ty, C: Converter>(
    slf: &PreparedMatcher<Ty, C>,
    code_store: &'store HAST,
    id: HAST::IdN,
) -> bool
where
    HAST: TypedHyperAST<TIdN>,
    TIdN: hyperast::types::TypedNodeId,
    Ty: std::fmt::Debug + Eq + Copy,
    for<'b> <Ty as TryFrom<&'b str>>::Error: std::fmt::Debug,
    Ty: for<'b> TryFrom<&'b str> + HyperType,
{
    // let mut stack = vec![];
    todo!()
}

mod exp {

    // some brainstorming about optimization/simp/desugar rules:
    // (a)+        ==> (a) . (a)*
    // (a)? . (a)* ==> (a)*
    // (a)? . (a)+ ==> (a)+
    // (a/b) ==> {a (b)} // lets symbolise abstract nodes with curly braces
    // (a f: (b)) ==> (a (f: (b)))
    // (a (b (c@c) (d@d))) (x? @c @d) ==> (a (b (c@c) (d@d)) (x? @c @d))
    // (a (_)) ==> (a . (_)) || (a . (_){+1} .(_))

    // should also think about shortcircuiting and doing things without slice alloc, .ie using binary tree
    // m( (_), ..b,c.. ) ==> m((_), ..b) || m((_), c..)

    use std::sync::Arc;

    type W = legion::World;
    type I = legion::Entity;

    fn f<T>(w: &mut W, p: I) -> I {
        let pat = w.entry(p).unwrap();
        match pat {
            // Plus { pat } => match pat.as_ref() {
            //     pat => Plus { pat }
            // },
            x => p,
        }
    }
    enum P<T, I> {
        Named { ty: T, pat: Vec<I> },
        Hidden { ty: T, pat: Vec<I> },
        Choice { pat: Vec<I> },
        Fielded { field: T, pat: I },
        Supertyped { sty: T, ty: T, pat: Vec<I> },
        Plus { pat: I },
        QMark { pat: I },
        Star { pat: I },
        Pair { pat: I },
        Immediate { pat: I },
        End { pat: I },
    }

    type Idx = usize;

    struct S<T, I> {
        path: Vec<Idx>,
        offset: Idx,
        pat: Arc<P<T, I>>,
    }
}

fn quant_from_bool(b: bool) -> tree_sitter::CaptureQuantifier {
    if b { Quant::One } else { Quant::Zero }
}
