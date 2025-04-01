#[cfg(all(feature = "impl", feature = "legion"))]
pub mod legion;

#[cfg(test)]
#[cfg(all(feature = "impl", feature = "legion"))]
mod legion_ts_simp;

#[cfg(test)]
mod tests;

pub mod types;
#[allow(unused)]
#[cfg(feature = "impl")]
pub mod types_exp;

#[cfg(feature = "legion")]
mod tnode {
    pub use hyperast::tree_gen::utils_ts::TNode;
}

#[cfg(feature = "legion")]
pub use tnode::TNode;

#[cfg(feature = "impl")]
pub fn language() -> tree_sitter::Language {
    tree_sitter::Language::new(tree_sitter_c::LANGUAGE)
}

#[cfg(feature = "impl")]
pub fn node_types() -> &'static str {
    tree_sitter_c::NODE_TYPES
}
