use std::fmt::Debug;

use super::TextPredicateCapture;

/// [`PerPattern`] Builder
pub struct PerPatternBuilder<P> {
    curr: Option<Vec<P>>,
    acc: Vec<Box<[P]>>,
}

impl<P> PerPatternBuilder<P> {
    pub fn with_patt_count(pat_count: usize) -> Self {
        Self {
            curr: None,
            acc: Vec::with_capacity(pat_count),
        }
    }
    pub fn prep(&mut self) {
        if let Some(curr) = self.curr.take() {
            self.acc.push(curr.into());
        }
        self.curr = Some(vec![])
    }
    pub fn push(&mut self, value: P) {
        self.curr.as_mut().unwrap().push(value)
    }
    pub fn build(mut self) -> PerPattern<P> {
        if let Some(curr) = self.curr.take() {
            self.acc.push(curr.into());
        }
        PerPattern(self.acc.into())
    }

    pub(crate) fn curr_len(&self) -> usize {
        self.curr.as_ref().unwrap().len()
    }
}
/// Efficiently packs/indexes elements P per PatternId
#[derive(Debug)]
pub struct PerPattern<P>(Box<[Box<[P]>]>);
pub type TextPredicateCaptures = PerPattern<TextPredicateCapture>;
pub type GeneralPredicates = PerPattern<QueryPredicate>;
pub type PropertyPredicates = PerPattern<(tree_sitter::QueryProperty, IsPositive)>;
pub type PropertySettings = PerPattern<tree_sitter::QueryProperty>;
type IsPositive = bool;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum QueryPredicateArg {
    Capture(u32),
    String(Box<str>),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct QueryPredicate {
    pub operator: Box<str>,
    pub args: Box<[QueryPredicateArg]>,
}

impl<P: Debug> PerPattern<P> {
    pub fn preds_for_patern_id<'a>(
        &'a self,
        id: crate::indexed::PatternId,
    ) -> impl Iterator<Item = &'a P> {
        self.0[id.to_usize()].iter()
    }

    pub(crate) fn extend(&mut self, preds: PerPattern<P>) {
        let mut r = std::mem::take(&mut self.0).into_vec();
        r.extend(preds.0.into_vec());
        self.0 = r.into();
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = std::slice::IterMut<P>> + '_ {
        self.0.iter_mut().map(|x|x.iter_mut())
    }

    pub(crate) fn check_empty(&mut self) {
        self.0.iter_mut().for_each(|x|assert!(x.is_empty(), "{:?}", x))
    }
}

impl Clone for PerPattern<TextPredicateCapture> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Clone for PerPattern<QueryPredicate> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Clone for PerPattern<(tree_sitter::QueryProperty, IsPositive)> {
    fn clone(&self) -> Self {
        Self(
            self.0
                .iter()
                .map(|x| {
                    x.iter()
                        .map(|(x, b)| {
                            (
                                tree_sitter::QueryProperty {
                                    key: x.key.clone(),
                                    value: x.value.clone(),
                                    capture_id: x.capture_id,
                                },
                                *b,
                            )
                        })
                        .collect()
                })
                .collect(),
        )
    }
}

impl Clone for PerPattern<tree_sitter::QueryProperty> {
    fn clone(&self) -> Self {
        Self(
            self.0
                .iter()
                .map(|x| {
                    x.iter()
                        .map(|x| tree_sitter::QueryProperty {
                            key: x.key.clone(),
                            value: x.value.clone(),
                            capture_id: x.capture_id,
                        })
                        .collect()
                })
                .collect(),
        )
    }
}

impl Clone for PerPattern<ImmediateTextPredicate> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ImmediateTextPredicate<T = Box<str>> {
    EqString {
        str: T,
        is_named: bool,
        is_positive: bool,
    },
    MatchString {
        re: regex::bytes::Regex,
    },
    MatchStringUnamed {
        re: regex::bytes::Regex,
    },
    AnyString(Box<[T]>),
}

impl PartialEq for ImmediateTextPredicate {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::EqString {
                    str: l_str,
                    is_named: l_is_named,
                    is_positive: l_is_positive,
                },
                Self::EqString {
                    str: r_str,
                    is_named: r_is_named,
                    is_positive: r_is_positive,
                },
            ) => l_str == r_str && l_is_named == r_is_named && l_is_positive == r_is_positive,
            (Self::MatchString { re: l_re }, Self::MatchString { re: r_re }) => {
                l_re.as_str() == r_re.as_str()
            }
            (Self::MatchStringUnamed { re: l_re }, Self::MatchStringUnamed { re: r_re }) => {
                l_re.as_str() == r_re.as_str()
            }
            (Self::AnyString(l0), Self::AnyString(r0)) => l0 == r0,
            _ => false,
        }
    }
}
