#![recursion_limit = "4096"]

#[cfg(all(feature = "impl", feature = "bevy"))]
pub mod bevy;
#[cfg(feature = "impl")]
pub mod compat;
#[cfg(feature = "impl")]
pub mod legion_with_refs; // TODO rename and move to a module for construction

pub mod types;
#[allow(unused)]
#[cfg(feature = "impl")]
pub mod types_exp;

#[cfg(all(feature = "impl", feature = "impact"))]
pub mod impact;
#[cfg(all(feature = "impl", feature = "tsg"))]
#[cfg(test)]
pub mod tsg;
#[cfg(feature = "impl")]
pub mod usage;

#[cfg(feature = "impl")]
#[cfg(test)]
mod tests;

pub use hyperast::utils;

#[cfg(feature = "legion")]
mod tnode {
    pub use hyperast::tree_gen::utils_ts::TNode;
}

#[cfg(feature = "legion")]
pub use tnode::TNode;

#[cfg(feature = "legion")]
pub mod iter;

#[cfg(feature = "impl")]
pub fn language() -> tree_sitter::Language {
    tree_sitter::Language::new(tree_sitter_java::LANGUAGE)
}

#[cfg(feature = "impl")]
pub fn node_types() -> &'static str {
    tree_sitter_java::NODE_TYPES
}
