pub mod syntax_highlighting_async;
#[cfg(feature = "ts_highlight")]
pub mod syntax_highlighting_ts;

// #[cfg(feature = "syntect")]
pub mod simple;
pub mod syntect;

// ----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(enum_map::Enum)]
pub enum TokenType {
    Comment,
    Keyword,
    Literal,
    StringLiteral,
    Punctuation,
    Whitespace,
}
