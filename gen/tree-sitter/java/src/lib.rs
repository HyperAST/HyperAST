#![feature(min_specialization)]
#![feature(let_chains)]
// #![feature(generic_const_exprs)]
#![feature(variant_count)]
#![recursion_limit = "4096"]

#[cfg(feature = "impl")]
pub mod compat;
#[cfg(feature = "impl")]
pub mod legion_with_refs;

pub mod types;
#[allow(unused)]
#[cfg(feature = "impl")]
pub mod types_exp;

#[cfg(feature = "impl")]
pub mod impact;
#[cfg(feature = "impl")]
pub mod usage;

#[cfg(feature = "impl")]
#[cfg(test)]
mod tests;

pub use hyper_ast::utils;

#[cfg(feature = "legion")]
mod tnode {

    #[repr(transparent)]
    pub struct TNode<'a>(pub(super) tree_sitter::Node<'a>);

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
