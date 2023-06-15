#![feature(generic_associated_types)]

#[cfg(feature = "impl")]
pub mod legion;

pub mod types;

#[cfg(feature = "impl")]
#[cfg(test)]
mod tests;

#[cfg(feature = "legion")]
mod tnode {
    use super::*;

    #[repr(transparent)]
    pub struct TNode<'a>(pub(crate) tree_sitter::Node<'a>);

    impl<'a> hyper_ast::tree_gen::parser::Node<'a> for TNode<'a> {
        fn kind(&self) -> &str {
            self.0.kind()
        }

        fn start_byte(&self) -> usize {
            self.0.start_byte()
        }

        fn end_byte(&self) -> usize {
            self.0.end_byte()
        }

        fn child_count(&self) -> usize {
            self.0.child_count()
        }

        fn child(&self, i: usize) -> Option<Self> {
            self.0.child(i).map(TNode)
        }

        fn is_named(&self) -> bool {
            self.0.is_named()
        }
    }
    impl<'a> hyper_ast::tree_gen::parser::NodeWithU16TypeId<'a> for TNode<'a> {
        fn kind_id(&self) -> u16 {
            self.0.kind_id()
        }
    }
}

#[cfg(feature = "legion")]
pub use tnode::TNode;
