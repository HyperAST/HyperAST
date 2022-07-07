use crate::tree::tree::{NodeStore, Tree, WithHashs, Stored};

pub trait Matcher<'a, D, T: 'a + Stored + Tree + WithHashs, S: NodeStore<'a, T::TreeId, &'a T>> {
    type Store;
    type Ele;

    fn matchh(
        compressed_node_store: &'a S,
        src: &'a T::TreeId,
        dst: &'a T::TreeId,
        mappings: Self::Store,
    ) -> Self::Store;
}
