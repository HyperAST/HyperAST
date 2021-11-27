use crate::tree::tree::{NodeStore, Tree, WithHashs};

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::mem::size_of;

//     #[test]
//     fn test_size() {
//         println!("{}", size_of::<u64>());
//         println!("{}", size_of::<Option<u64>>());
//         println!("{}", size_of::<std::mem::ManuallyDrop<Box<[u8]>>>());
//         println!("{}", size_of::<std::mem::ManuallyDrop<u64>>());
//         println!("{}", size_of::<u16>());
//     }
// }

pub trait Matcher<'a, D, T: Tree + WithHashs, S: NodeStore<'a, T>> {
    type Store;
    type Ele;

    fn matchh(
        compressed_node_store: &'a S,
        src: &'a T::TreeId,
        dst: &'a T::TreeId,
        mappings: Self::Store,
    ) -> Self::Store;
}
