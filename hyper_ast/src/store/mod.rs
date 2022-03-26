use crate::types::Type;

use crate::nodes::TypeIdentifier;

pub mod handle;
pub mod radix_hash_store;
pub mod vec_map_store;
pub mod ecs;
pub mod mapped_world;
pub mod labels;
pub mod nodes;

pub struct TypeStore {}

impl TypeStore {
    pub fn get(&mut self, kind: &str) -> TypeIdentifier {
        Type::new(kind)
    }
    pub fn get_xml(&mut self, kind: &str) -> TypeIdentifier {
        Type::parse_xml(kind)
    }
}



pub struct SimpleStores {
    pub label_store: labels::LabelStore,
    pub type_store: TypeStore,
    pub node_store: nodes::DefaultNodeStore,
}

impl Default for SimpleStores {
    fn default() -> Self {
        Self {
            label_store: labels::LabelStore::new(),
            type_store: TypeStore {},
            node_store: nodes::DefaultNodeStore::new(),
        }
    }
}

pub mod defaults {
    pub type LabelIdentifier = super::labels::DefaultLabelIdentifier;
    pub type LabelValue = super::labels::DefaultLabelValue;
    pub type NodeIdentifier = super::nodes::DefaultNodeIdentifier;
}
