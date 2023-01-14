use crate::types::{HyperAST, SimpleHyperAST, Type};

use crate::nodes::TypeIdentifier;

pub mod handle;
pub mod labels;
pub mod mapped_world;
pub mod nodes;
// pub mod ecs; // TODO try a custom ecs ?
// pub mod radix_hash_store; // TODO yet another WIP store
// pub mod vec_map_store; // TODO yet another WIP store

pub struct TypeStore {}

impl TypeStore {
    pub fn get(&mut self, kind: &str) -> TypeIdentifier {
        Type::new(kind)
    }
    pub fn get_xml(&mut self, kind: &str) -> TypeIdentifier {
        Type::parse_xml(kind)
    }
}

pub struct SimpleStores<NS = nodes::DefaultNodeStore> {
    pub label_store: labels::LabelStore,
    pub type_store: TypeStore,
    pub node_store: NS,
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

impl<'store> From<&'store SimpleStores<nodes::DefaultNodeStore>>
    for SimpleHyperAST<
        self::nodes::legion::HashedNodeRef<'store>,
        &'store nodes::DefaultNodeStore,
        &'store labels::LabelStore,
    >
{
    fn from(value: &'store SimpleStores<nodes::DefaultNodeStore>) -> Self {
        Self {
            node_store: &value.node_store,
            label_store: &value.label_store,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'store> HyperAST<'store> for SimpleStores<nodes::DefaultNodeStore> {
    type IdN = nodes::DefaultNodeIdentifier;

    type Label = labels::DefaultLabelIdentifier;

    type T = self::nodes::legion::HashedNodeRef<'store>;

    type NS = nodes::DefaultNodeStore;

    fn node_store(&self) -> &Self::NS {
        &self.node_store
    }

    type LS = labels::LabelStore;

    fn label_store(&self) -> &Self::LS {
        &self.label_store
    }
}
