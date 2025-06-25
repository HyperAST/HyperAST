use crate::indexed::CaptureId;

use super::QueryError;
use super::QueryErrorKind;

#[must_use]
pub(crate) const fn predicate_error(row: usize, message: String) -> QueryError {
    QueryError {
        kind: QueryErrorKind::Predicate,
        row,
        column: 0,
        offset: 0,
        message,
    }
}

type IsPositive = bool;
type MatchAllNodes = bool;

#[derive(Clone, Debug)]
pub enum TextPredicateCapture<T = Box<str>> {
    EqString(CaptureId, T, IsPositive, MatchAllNodes),
    EqCapture(CaptureId, CaptureId, IsPositive, MatchAllNodes),
    MatchString(CaptureId, regex::bytes::Regex, IsPositive, MatchAllNodes),
    AnyString(CaptureId, Box<[T]>, MatchAllNodes),
}

impl TextPredicateCapture {
    pub(crate) fn remap(&mut self, capture_map: &[CaptureId]) {
        match self {
            TextPredicateCapture::EqString(a, _, _, _)
            | TextPredicateCapture::EqCapture(a, _, _, _)
            | TextPredicateCapture::MatchString(a, _, _, _)
            | TextPredicateCapture::AnyString(a, _, _) => {
                *a = capture_map[a.to_usize()];
            }
        };
        match self {
            TextPredicateCapture::EqCapture(_, a, _, _) => {
                *a = capture_map[a.to_usize()];
            }
            _ => (),
        };
    }
}
