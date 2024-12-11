#[cfg(feature = "impl")]
pub mod legion;

pub mod types;

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
    tree_sitter::Language::new(tree_sitter_xml::LANGUAGE_XML)
}

#[cfg(feature = "impl")]
pub fn node_types() -> &'static str {
    tree_sitter_xml::XML_NODE_TYPES
}
