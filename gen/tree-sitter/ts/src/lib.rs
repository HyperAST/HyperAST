#[cfg(feature = "impl")]
pub mod legion;

pub mod types;

#[cfg(feature = "legion")]
mod tnode {
    pub use hyperast::tree_gen::utils_ts::TNode;
}

#[cfg(feature = "legion")]
pub use tnode::TNode;



#[cfg(feature = "impl")]
pub fn language() -> tree_sitter::Language {
    tree_sitter::Language::new(tree_sitter_typescript::LANGUAGE_TYPESCRIPT)
}

#[cfg(feature = "impl")]
pub fn node_types() -> &'static str {
    tree_sitter_typescript::TYPESCRIPT_NODE_TYPES
}
