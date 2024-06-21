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

#[derive(Clone)]
pub(crate) enum TextPredicateCapture<T = Box<str>> {
    EqString(CaptureId, T, IsPositive, MatchAllNodes),
    EqCapture(CaptureId, CaptureId, IsPositive, MatchAllNodes),
    MatchString(CaptureId, regex::bytes::Regex, IsPositive, MatchAllNodes),
    AnyString(CaptureId, Box<[T]>, MatchAllNodes),
}
