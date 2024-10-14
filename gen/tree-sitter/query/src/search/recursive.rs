//! This protoype query matcher is implemented totally differently from the original treesitter one.

use super::{CaptureRes, Captured, MatchingRes, Pattern, Predicate, PreparedMatcher};

mod matching;
