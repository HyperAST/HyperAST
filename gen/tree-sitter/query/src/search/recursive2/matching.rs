use std::collections::VecDeque;
use std::ops::Deref;

use crate::auto::tsq_ser_meta::Conv;

use super::{CaptureRes, Captured, MatchingRes, Pattern, Predicate, PreparedMatcher};

use hyperast::types::AstLending;
use hyperast::types::TypedLending;
use tree_sitter::CaptureQuantifier as Quant;

use hyperast::types::HyperType;
use hyperast::types::TypedHyperAST;
use hyperast::types::{Childrn, Typed, TypedNodeStore, WithChildren};

pub struct MatchingIter<
    'store,
    HAST: TypedHyperAST<TIdN>,
    TIdN: hyperast::types::TypedNodeId, //<IdN = HAST::IdN>,
    PM: Deref<Target = PreparedMatcher<TIdN::Ty, Conv<TIdN::Ty>>>,
> {
    slf: PM,
    code_store: &'store HAST,
    root: HAST::IdN,
    // stack: Vec<State<HAST::IdN, HAST::Idx, TIdN::Ty>>,
    res: Option<std::collections::VecDeque<Captured<HAST::IdN, HAST::Idx>>>,
    _phantom: std::marker::PhantomData<TIdN>,
}

impl<
        'store,
        HAST: TypedHyperAST<TIdN>,
        TIdN: hyperast::types::TypedNodeId, //<IdN = HAST::IdN>,
        PM: Deref<Target = PreparedMatcher<TIdN::Ty, Conv<TIdN::Ty>>>,
    > MatchingIter<'store, HAST, TIdN, PM>
{
    pub fn new(slf: PM, code_store: &'store HAST, root: HAST::IdN) -> Self {
        Self {
            slf,
            code_store,
            root,
            res: None,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<
        'store,
        HAST: TypedHyperAST<TIdN>,
        TIdN: hyperast::types::TypedNodeId, // <IdN = HAST::IdN>,
        PM: Deref<Target = PreparedMatcher<TIdN::Ty, Conv<TIdN::Ty>>>,
    > Iterator for MatchingIter<'store, HAST, TIdN, PM>
where
    HAST::IdN: std::fmt::Debug,
{
    type Item = Captured<HAST::IdN, HAST::Idx>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(res) = &mut self.res {
            return res.pop_front();
        }
        let mut res = std::collections::VecDeque::default();
        let arc = self.slf.patterns.clone();
        for (i, pat) in arc.as_ref().into_iter().enumerate() {
            let r = self.is_matching(&pat, self.root.clone());
            res.extend(r.into_iter().map(|res| Captured(res.captures, i)));
        }
        self.res = Some(res);
        self.next()
    }
}
impl<
        'store,
        HAST: TypedHyperAST<TIdN>,
        TIdN: 'store + hyperast::types::TypedNodeId, //<IdN = HAST::IdN>,
        PM: Deref<Target = PreparedMatcher<TIdN::Ty, Conv<TIdN::Ty>>>,
    > MatchingIter<'store, HAST, TIdN, PM>
where
    HAST::IdN: std::fmt::Debug,
{
    pub fn repurpose(&mut self, id: HAST::IdN) {
        self.res = None;
        self.root = id;
    }

    pub(crate) fn is_matching(
        &mut self,
        pattern: &Pattern<TIdN::Ty>,
        id: HAST::IdN,
    ) -> Vec<MatchingRes<HAST::IdN, HAST::Idx>> {
        let Some((n, tid)) = (&self.code_store).try_resolve(&id) else {
            dbg!();
            return vec![];
        };
        let t = n.get_type();
        dbg!(t);

        if t.is_spaces() {
            return vec![];
        }

        match pattern {
            Pattern::SupNamedNode { sup, ty, children } => {
                let ty = *ty;
                if t.is_hidden() && *sup == t {
                    dbg!(ty);
                    let cs = n.children().unwrap().iter_children();
                    let mut r = vec![];
                    let mut i = num::zero();
                    for child in cs {
                        let children = children.clone();
                        let m_res =
                            self.is_matching(&Pattern::NamedNode { ty, children }, child.clone());
                        for mut m_res in m_res {
                            if m_res.matched == Quant::One {
                                for v in &mut m_res.captures {
                                    v.path.push(i);
                                }
                                r.push(m_res);
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
                        i += num::one();
                    }
                    r
                } else if t.is_hidden() && ty != t && id != self.root {
                    dbg!(ty);
                    let cs = n.children().unwrap().iter_children();
                    let mut r = vec![];
                    let mut i = num::zero();
                    for child in cs {
                        let m_res = self.is_matching(pattern, child.clone());
                        for mut m_res in m_res {
                            if m_res.matched == Quant::One {
                                for v in &mut m_res.captures {
                                    v.path.push(i);
                                }
                                r.push(m_res);
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
                        i += num::one();
                    }
                    r
                } else {
                    vec![]
                }
            }
            Pattern::NamedNode { ty, children } => {
                if t.is_hidden() && *ty != t && id != self.root {
                    dbg!(ty);
                    let cs = n.children().unwrap().iter_children();
                    let mut r = vec![];
                    let mut i = num::zero();
                    for child in cs {
                        let m_res = self.is_matching(pattern, child.clone());
                        for mut m_res in m_res {
                            if m_res.matched == Quant::One {
                                for v in &mut m_res.captures {
                                    v.path.push(i);
                                }
                                r.push(m_res);
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
                        i += num::one();
                    }
                    return r;
                }
                if *ty != t {
                    return vec![];
                }
                let Some(cs) = n.children() else {
                    dbg!(ty, t);
                    let mut r = vec![];
                    if children.is_empty() {
                        r.push(MatchingRes {
                            matched: Quant::One,
                            captures: Default::default(),
                        });
                    }
                    return r;
                };
                dbg!(t);
                dbg!(n.child_count());
                let pats = &children[..];
                let captures = Default::default();
                if pats.is_empty() {
                    return vec![MatchingRes {
                        matched: Quant::One,
                        captures,
                    }];
                }
                let matched = Quant::Zero;
                // let i = num::zero();
                let i_pat = 0;
                let immediate = false;
                let cs = ChildIt::new(self.code_store, tid);
                self.children_aux(pats, i_pat, matched, captures, cs, immediate, t, &n)
            }
            Pattern::AnonymousNode(ty) if *ty == t => vec![MatchingRes {
                matched: quant_from_bool(*ty == t),
                captures: Default::default(),
            }],
            Pattern::AnonymousNode(ty) => vec![],
            Pattern::Capture { name, pat } => {
                if pat.unwrap_captures().is_any_node() && t.is_hidden() {
                    let cs = n.children().unwrap().iter_children();
                    let mut r = vec![];
                    let mut i = num::zero();
                    for child in cs {
                        let m_res = self.is_matching(pattern, child.clone());
                        for mut m_res in m_res {
                            if m_res.matched == Quant::One {
                                for v in &mut m_res.captures {
                                    v.path.push(i);
                                }
                                r.push(m_res);
                            } else if m_res.matched == Quant::Zero {
                            } else {
                                todo!("{:?} {:?} {}", t, m_res.matched, m_res.captures.len());
                            }
                        }
                        i += num::one();
                    }
                    return r;
                }
                let mut result = vec![];
                for m_res in self.is_matching(pat, id.clone()) {
                    match m_res {
                        MatchingRes {
                            matched: Quant::One,
                            mut captures,
                        } => {
                            let name = name.clone();
                            let n = (&self.code_store).try_resolve(&id).unwrap().0;
                            let v = CaptureRes {
                                id: name,
                                match_node: id.clone(),
                                path: vec![],
                            };
                            captures.push(v);
                            result.push(MatchingRes {
                                matched: Quant::One,
                                captures,
                            })
                        }
                        MatchingRes {
                            matched: Quant::Zero,
                            ..
                        } => (),
                        MatchingRes {
                            matched: Quant::ZeroOrOne,
                            mut captures,
                        } => {
                            let name = name.clone();
                            let v = CaptureRes {
                                id: name,
                                match_node: id.clone(),
                                path: vec![],
                            };
                            captures.push(v);
                            result.push(MatchingRes {
                                matched: Quant::ZeroOrOne,
                                captures,
                            })
                        }
                        MatchingRes { matched, .. } => todo!("{:?}", matched),
                    }
                }
                result
            }
            Pattern::Predicated { predicate, pat } => match predicate {
                Predicate::Eq { left, right } => self
                    .is_matching(pat, id)
                    .into_iter()
                    .filter_map(|matching_res| {
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
                            Some(MatchingRes { matched, captures })
                        } else {
                            None
                        }
                    })
                    .collect(),
                Predicate::EqString { left, right } => self
                    .is_matching(pat, id)
                    .into_iter()
                    .filter_map(|matching_res| {
                        if matching_res.matched == Quant::One {
                            let Some(capture) = matching_res.capture(*left) else {
                                return None;
                            };
                            let left = capture.try_label(self.code_store).unwrap();
                            let matched = left == right;
                            let captures = if matched {
                                matching_res.captures
                            } else {
                                Default::default()
                            };
                            let matched = quant_from_bool(matched);
                            Some(MatchingRes { matched, captures })
                        } else if matching_res.matched == Quant::Zero {
                            None
                        } else {
                            todo!("{:?}", matching_res.matched)
                        }
                    })
                    .collect(),
            },
            Pattern::AnyNode { children } => {
                if t.is_hidden() && id != self.root {
                    let cs = n.children().unwrap().iter_children();
                    let mut r = vec![];
                    let mut i = num::zero();
                    for child in cs {
                        let m_res = self.is_matching(pattern, child.clone());
                        for mut m_res in m_res {
                            if m_res.matched == Quant::One {
                                for v in &mut m_res.captures {
                                    v.path.push(i);
                                }
                                r.push(m_res);
                            } else if m_res.matched == Quant::Zero {
                            } else {
                                todo!("{:?} {:?} {}", t, m_res.matched, m_res.captures.len());
                            }
                        }
                        i += num::one();
                    }
                    return r;
                }
                let Some(cs) = n.children() else {
                    return if children.is_empty() && !t.is_spaces() && !t.is_syntax() {
                        vec![MatchingRes {
                            matched: Quant::One,
                            captures: Default::default(),
                        }]
                    } else {
                        vec![]
                    };
                };
                dbg!(t);
                dbg!(n.child_count());
                let pats = &children[..];
                let captures = Default::default();
                if pats.is_empty() {
                    return vec![MatchingRes {
                        matched: Quant::One,
                        captures,
                    }];
                }
                let matched = Quant::Zero;
                // let i = num::zero();
                let i_pat = 0;
                let immediate = false;
                let cs = ChildIt::new(self.code_store, tid);
                self.children_aux(pats, i_pat, matched, captures, cs, immediate, t, &n)
            }
            Pattern::List(patts) => {
                let mut result = vec![];
                for pat in patts.iter() {
                    match pat {
                        Pattern::AnyNode { .. } => todo!(),
                        _ => (),
                    }
                    for res in self.is_matching(&pat, id.clone()) {
                        if res.matched == Quant::One {
                            result.push(res);
                        } else if res.matched == Quant::Zero {
                        } else {
                            todo!("{:?}", res.matched)
                        }
                    }
                }
                result
            }
            Pattern::FieldDefinition { name, pat } => {
                unreachable!()
            }
            Pattern::Dot { .. } => todo!(),
            Pattern::Quantified { quantifier, pat } => match quantifier {
                Quant::ZeroOrOne => self
                    .is_matching(pat, id)
                    .into_iter()
                    .map(|MatchingRes { matched, captures }| MatchingRes {
                        matched: if matched == Quant::One {
                            Quant::ZeroOrOne
                        } else if matched == Quant::Zero {
                            Quant::ZeroOrOne
                        } else if matched == Quant::ZeroOrOne {
                            Quant::ZeroOrOne
                        } else {
                            todo!()
                        },
                        captures,
                    })
                    .collect(),
                Quant::ZeroOrMore => todo!("{:?}", pat),
                Quant::OneOrMore => self
                    .is_matching(pat, id)
                    .into_iter()
                    .map(|MatchingRes { matched, captures }| MatchingRes {
                        matched: if matched == Quant::One {
                            Quant::OneOrMore
                        } else if matched == Quant::Zero {
                            Quant::Zero
                        } else {
                            todo!()
                        },
                        captures,
                    })
                    .collect(),
                Quant::One => todo!("{:?}", pat),
                Quant::Zero => todo!("{:?}", pat),
            },
            Pattern::NegatedField(_) => unreachable!(),
        }
    }

    fn children_aux(
        &mut self,
        pats: &[Pattern<TIdN::Ty>],
        mut i_pat: usize,
        mut matched: Quant,
        mut captures: Vec<CaptureRes<HAST::IdN, HAST::Idx>>,
        mut cs: ChildIt<'store, HAST, TIdN>,
        // cs: &(impl hyperast::types::Children<HAST::Idx, HAST::IdN> + ?Sized),
        mut immediate: bool,
        // mut i: HAST::Idx,
        p_t: TIdN::Ty,
        parent_node: &<HAST as TypedLending<'_, TIdN::Ty>>::TT,
    ) -> Vec<MatchingRes<HAST::IdN, HAST::Idx>> {
        // dbg!(i);
        let mut result = vec![];
        loop {
            let Some(curr_p) = pats.get(i_pat) else {
                if immediate {
                    let Some(child) = cs.id() else {
                        if matched == Quant::ZeroOrOne {
                            matched = Quant::One;
                        }
                        result.push(MatchingRes { matched, captures });
                        break;
                    };
                    let n = cs.node();
                    let t = n.get_type();
                    if t.is_spaces() {
                        drop(n);
                        cs.adv();
                        continue;
                    }
                    // if t.as_shared() == hyperast::types::Shared::Comment {
                    //     i += num::one();
                    //     continue;
                    // }
                    if t.is_syntax() {
                        dbg!(t);
                        drop(n);
                        cs.adv();
                        continue;
                    }
                    break;
                }
                if matched == Quant::ZeroOrOne {
                    matched = Quant::One;
                }
                result.push(MatchingRes { matched, captures });
                break;
            };
            let Some(child) = cs.id() else {
                if pats[i_pat..].iter().any(|p| !p.is_optional_match()) {
                    break;
                }
                let matched = Quant::One;
                result.push(MatchingRes { matched, captures });
                break;
            };
            let n = cs.node();
            let t = n.get_type();
            if t.is_spaces() {
                drop(n);
                cs.adv();
                continue;
            }
            let curr_p = match curr_p {
                Pattern::Dot => {
                    if immediate {
                        panic!();
                    }
                    i_pat += 1;
                    immediate = true;
                    if i_pat == pats.len() {
                        // if cs.child_count() != i {
                        //     break;
                        // }
                        continue;
                    }
                    continue;
                }
                Pattern::NegatedField(field) => {
                    dbg!(field);
                    drop(n);
                    cs.adv();
                    continue;
                }
                Pattern::FieldDefinition { name, pat } => {
                    let Ok(role) = name.as_str().try_into() else {
                        todo!("missing role for: {}", name)
                    };
                    let r = cs.role();
                    if r == Some(role) {
                        pat.as_ref()
                    } else if r.is_some() {
                        if immediate {
                            break;
                        }
                        drop(n);
                        cs.adv();
                        continue;
                    } else {
                        if p_t.is_hidden() || p_t.is_supertype() {
                            dbg!(name);
                            dbg!(t);
                            dbg!(p_t);
                            panic!("should have skipped those before accessing children");
                            // let captures = Default::default();
                            // if pats.is_empty() {
                            //     return vec![MatchingRes {
                            //         matched: Quant::One,
                            //         captures,
                            //     }];
                            // }
                            // let Some(cs) = n.children() else {
                            //     break;
                            // };
                            // let mut r = self.children_aux(
                            //     pats,
                            //     i_pat,
                            //     matched,
                            //     captures,
                            //     cs,
                            //     immediate,
                            //     num::zero(),
                            //     t,
                            //     parent_node,
                            // );

                            // for v in &mut r {
                            //     for c in &mut v.captures {
                            //         c.path.push(i);
                            //     }
                            // }
                            // result.extend(r);
                        } else if t.is_supertype() {
                            todo!("should not contain field")
                        } else if t.is_hidden() {
                            dbg!(name);
                            dbg!(t);
                            dbg!(p_t);
                            let captures = Default::default();
                            if pats.is_empty() {
                                return vec![MatchingRes {
                                    matched: Quant::One,
                                    captures,
                                }];
                            }
                            todo!("go inside to try to find field")

                            // let Some(cs) = n.children() else {
                            //     break;
                            // };
                            // let mut r = self.children_aux(
                            //     pats,
                            //     i_pat,
                            //     matched,
                            //     captures,
                            //     cs,
                            //     immediate,
                            //     num::zero(),
                            //     t,
                            //     parent_node,
                            // );

                            // for v in &mut r {
                            //     for c in &mut v.captures {
                            //         c.path.push(i);
                            //     }
                            // }
                            // result.extend(r);
                        }
                        if immediate {
                            break;
                        }
                        drop(n);
                        cs.adv();
                        continue;
                    }
                }
                x => x,
            };
            let aaa = format!("{:?}", curr_p);
            if !curr_p.unwrap_captures().is_anonymous() {
                if !curr_p.unwrap_captures().is_any_node()
                    && t.as_shared() == hyperast::types::Shared::Comment
                {
                    drop(n);
                    cs.adv();
                    continue;
                }
                if t.is_syntax() {
                    drop(n);
                    cs.adv();
                    continue;
                }
            }
            dbg!(t);
            let mut m_res = self.is_matching(&curr_p, child.clone());
            let last = m_res.pop();
            for res in m_res {
                match res {
                    MatchingRes {
                        matched: Quant::One,
                        captures: mut capt,
                    } => {
                        for v in &mut capt {
                            v.path.extend(cs.short_path());
                        }
                        let mut captures = captures.clone();
                        captures.extend(capt);
                        let n = cs.node();
                        let t = n.get_type();
                        dbg!(t);
                        result.extend(self.children_aux(
                            pats,
                            i_pat + 1,
                            Quant::One,
                            captures.clone(),
                            cs.clone_adv(),
                            false,
                            p_t,
                            parent_node,
                        ));
                    }
                    MatchingRes {
                        matched: Quant::Zero,
                        ..
                    } => {
                        if immediate {
                            continue;
                        }
                        let n = cs.node();
                        let t = n.get_type();
                        dbg!(t);
                        result.extend(self.children_aux(
                            pats,
                            i_pat,
                            matched,
                            captures.clone(),
                            cs.clone_adv(),
                            immediate,
                            p_t,
                            parent_node,
                        ));
                    }
                    MatchingRes {
                        matched: Quant::ZeroOrOne,
                        ..
                    } => {
                        let n = cs.node();
                        let t = n.get_type();
                        dbg!(t);
                        result.extend(self.children_aux(
                            pats,
                            i_pat,
                            Quant::ZeroOrOne,
                            captures.clone(),
                            cs.clone_adv(),
                            false,
                            p_t,
                            parent_node,
                        ));
                    }
                    MatchingRes { matched, .. } => todo!("{:?}", matched),
                }
            }
            if let Some(res) = last {
                match res {
                    MatchingRes {
                        matched: Quant::One,
                        captures: mut capt,
                    } => {
                        if !immediate {
                            result.extend(self.children_aux(
                                pats,
                                i_pat,
                                Quant::Zero,
                                captures.clone(),
                                cs.clone_adv(),
                                false,
                                p_t,
                                parent_node,
                            ));
                        }
                        immediate = false;
                        i_pat += 1;
                        matched = Quant::One;
                        for v in &mut capt {
                            v.path.extend(cs.short_path());
                        }
                        captures.extend(capt);
                        let n = cs.node();
                        dbg!(n.get_type());
                    }
                    MatchingRes {
                        matched: Quant::OneOrMore,
                        captures: mut capt,
                    } => {
                        if !immediate {
                            result.extend(self.children_aux(
                                pats,
                                i_pat,
                                Quant::Zero,
                                captures.clone(),
                                cs.clone_adv(),
                                false,
                                p_t,
                                parent_node,
                            ));
                        }
                        for v in &mut capt {
                            v.path.extend(cs.short_path());
                        }
                        captures.extend(capt);
                        result.extend(self.children_aux(
                            pats,
                            i_pat,
                            Quant::Zero,
                            captures.clone(),
                            cs.clone_adv(),
                            true,
                            p_t,
                            parent_node,
                        ));
                        immediate = false;
                        i_pat += 1;
                        matched = Quant::One;
                        let n = cs.node();
                        dbg!(n.get_type());
                    }
                    MatchingRes {
                        matched: Quant::Zero,
                        ..
                    } => {
                        if immediate {
                            break;
                        }
                        let n = cs.node();
                        dbg!(n.get_type());
                    }
                    MatchingRes {
                        matched: Quant::ZeroOrOne,
                        ..
                    } => {
                        result.extend(self.children_aux(
                            pats,
                            i_pat + 1,
                            Quant::Zero,
                            captures.clone(),
                            cs.clone(),
                            false,
                            p_t,
                            parent_node,
                        ));
                        immediate = false;
                        matched = Quant::ZeroOrOne;
                        let n = cs.node();
                        dbg!(n.get_type());
                    }
                    MatchingRes { matched, .. } => todo!("{:?}", matched),
                }
            }
            if immediate {
                break;
            }
            // assert_eq!(immediate, false);
            drop(n);
            cs.adv();
        }
        result
    }
}

fn quant_from_bool(b: bool) -> tree_sitter::CaptureQuantifier {
    if b {
        Quant::One
    } else {
        Quant::Zero
    }
}

struct ChildIt<'store, HAST, IdN> {
    stores: &'store HAST,
    id: IdN,
    waiting: VecDeque<IdN>,
    role: Option<hyperast::types::Role>,
}
impl<'store, HAST, IdN: Clone> Clone for ChildIt<'store, HAST, IdN> {
    fn clone(&self) -> Self {
        Self {
            stores: self.stores,
            id: self.id.clone(),
            waiting: self.waiting.clone(),
            role: self.role.clone(),
        }
    }
}
impl<'store, TIdN, HAST> ChildIt<'store, HAST, TIdN>
where
    HAST: TypedHyperAST<TIdN>,
    TIdN: 'store + hyperast::types::TypedNodeId, //<IdN = HAST::IdN>,
{
    fn new(stores: &'store HAST, id: TIdN) -> Self {
        Self {
            stores,
            id: todo!(),
            waiting: todo!(),
            role: todo!(),
        }
    }
    /// peek current node
    fn id(&self) -> Option<HAST::IdN> {
        todo!()
    }
    /// panics if nothing there, use peek before calling it
    fn node(&self) -> <HAST as TypedLending<'_, TIdN::Ty>>::TT {
        self.stores.try_resolve(todo!()).unwrap().0
    }
    /// advance to next named node
    fn adv(&mut self) {
        todo!()
    }
    fn clone_adv(&self) -> Self {
        todo!()
    }
    fn role(&self) -> Option<hyperast::types::Role> {
        todo!()
    }
    fn short_path(&self) -> impl Iterator<Item = HAST::Idx> {
        vec![todo!()].into_iter()
    }
}
