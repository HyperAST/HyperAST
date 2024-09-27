pub mod compressed_tree;
pub mod decompressed_tree;
pub mod simple_tree;
// pub mod tree;
pub mod tree_path;

pub(crate) struct TStore;

use self::simple_tree::{Tree, TreeRef};

impl hyper_ast::types::TypeStore<Tree> for TStore {
    type Ty = u8;

    fn resolve_type(&self, n: &Tree) -> Self::Ty {
        use hyper_ast::types::Typed;
        n.get_type()
    }

    fn resolve_lang(&self, _n: &Tree) -> hyper_ast::types::LangWrapper<Self::Ty> {
        todo!()
    }

    fn type_eq(&self, _n: &Tree, _m: &Tree) -> bool {
        todo!()
    }
}

impl<'a> hyper_ast::types::TypeStore<TreeRef<'a, Tree>> for TStore {
    type Ty = u8;

    fn resolve_type(&self, n: &TreeRef<'a, Tree>) -> Self::Ty {
        use hyper_ast::types::Typed;
        n.get_type()
    }

    fn resolve_lang(&self, _n: &TreeRef<'a, Tree>) -> hyper_ast::types::LangWrapper<Self::Ty> {
        todo!()
    }

    fn type_eq(&self, _n: &TreeRef<'a, Tree>, _m: &TreeRef<'a, Tree>) -> bool {
        todo!()
    }
}
