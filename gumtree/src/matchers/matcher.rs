// use hyper_ast::types::{NodeStore, Tree, WithHashs};

// pub trait Matcher<'a, Dsrc, Ddst, T: 'a + Tree + WithHashs, S>
// where
//     //S:'a+NodeStore2<T::TreeId,R<'a>=T>,//
//     S: 'a + NodeStore<T::TreeId>,
//     S::R<'a>: Tree<TreeId = T::TreeId>,
// {
//     type Store;
//     type Ele;

//     fn matchh(
//         compressed_node_store: &'a S,
//         src: &T::TreeId,
//         dst: &T::TreeId,
//         mappings: Self::Store,
//     ) -> Self::Store;
// }
