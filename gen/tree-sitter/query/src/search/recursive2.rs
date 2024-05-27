//! Attempt to handle multiple matches for a single root pattern.

use super::{CaptureRes, Captured, MatchingRes, Pattern, Predicate, PreparedMatcher};

mod matching;
pub use matching::*;