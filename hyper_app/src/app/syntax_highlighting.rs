pub mod syntax_highlighting_async;
pub mod syntax_highlighting_ts;

// #[cfg(feature = "syntect")]
pub(crate) mod syntect;

pub(crate) mod simple;

// ----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(enum_map::Enum)]
pub(crate) enum TokenType {
    Comment,
    Keyword,
    Literal,
    StringLiteral,
    Punctuation,
    Whitespace,
}
