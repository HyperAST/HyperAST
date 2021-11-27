use rusted_gumtree_core::tree::tree::Type;

use crate::nodes::TypeIdentifier;

pub mod handle;
pub mod radix_hash_store;
pub mod vec_map_store;

pub struct TypeStore {}

impl TypeStore {
    pub fn get(&mut self, kind: &str) -> TypeIdentifier {
        Type::new(kind)
    }
}