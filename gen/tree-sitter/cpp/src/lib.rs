#[cfg(all(feature = "impl", feature = "legion"))]
pub mod legion;

#[cfg(test)]
#[cfg(all(feature = "impl", feature = "legion"))]
mod legion_ts_simp;



pub mod types;
#[allow(unused)]
#[cfg(feature = "impl")]
pub mod types_exp;

#[cfg(feature = "impl")]
#[cfg(test)]
mod tests;

#[cfg(feature = "legion")]
mod tnode {
    pub use hyper_ast::tree_gen::utils_ts::TNode;
}

#[cfg(feature = "legion")]
pub use tnode::TNode;

#[cfg(feature = "legion")]
pub mod iter;

#[cfg(feature = "impl")]
pub fn language() -> tree_sitter::Language {
    tree_sitter::Language::new(tree_sitter_cpp::LANGUAGE)
}

#[cfg(feature = "impl")]
pub fn node_types() -> &'static str {
    tree_sitter_cpp::NODE_TYPES
}
