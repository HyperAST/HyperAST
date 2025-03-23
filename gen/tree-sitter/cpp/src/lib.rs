#[cfg(all(feature = "impl_intern", feature = "legion"))]
pub mod legion;

#[cfg(test)]
#[cfg(all(feature = "impl_intern", feature = "legion"))]
mod legion_ts_simp;

#[cfg(not(feature = "alt_grammar"))]
pub mod types;

#[cfg(feature = "alt_grammar")]
pub mod types_alt;
#[cfg(feature = "alt_grammar")]
pub use types_alt as types;

#[allow(unused)]
#[cfg(feature = "impl_intern")]
pub mod types_exp;

#[cfg(feature = "impl_intern")]
#[cfg(test)]
mod tests;

#[cfg(feature = "legion")]
mod tnode {
    pub use hyperast::tree_gen::utils_ts::TNode;
}

#[cfg(feature = "legion")]
pub use tnode::TNode;

#[cfg(feature = "legion")]
pub mod iter;

#[cfg(feature = "impl_intern")]
pub fn language() -> tree_sitter::Language {
    #[cfg(feature = "alt_grammar")]
    {
        tree_sitter::Language::new(tree_sitter_cpp_alt::LANGUAGE)
    }
    #[cfg(not(feature = "alt_grammar"))]
    tree_sitter::Language::new(tree_sitter_cpp::LANGUAGE)
}

#[cfg(feature = "impl_intern")]
pub fn node_types() -> &'static str {
    #[cfg(feature = "alt_grammar")]
    {
        tree_sitter_cpp_alt::NODE_TYPES
    }
    #[cfg(not(feature = "alt_grammar"))]
    tree_sitter_cpp::NODE_TYPES
}
