use std::{fmt::Debug, hash::Hash, num::NonZeroU64};

use hashbrown::hash_map::DefaultHashBuilder;
use legion::{
    storage::{Component, IntoComponentSource},
    EntityStore,
};

use crate::{
    types::{NodeId, Typed, TypedNodeId},
    utils::make_hash,
};

pub mod dyn_builder;

pub mod compo;

mod elem;

pub use elem::{EntryRef, HashedNode, HashedNodeRef, NodeIdentifier};

pub struct NodeStore {
    count: usize,
    errors: usize,
    // roots: HashMap<(u8, u8, u8), NodeIdentifier>,
    dedup: hashbrown::HashMap<NodeIdentifier, (), ()>,
    internal: legion::World,
    hasher: DefaultHashBuilder, //fasthash::city::Hash64,//fasthash::RandomState<fasthash::>,
                                // internal: VecMapStore<HashedNode, NodeIdentifier, legion::World>,
}

// * Node store impl

pub struct PendingInsert<'a>(
    crate::compat::hash_map::RawEntryMut<'a, legion::Entity, (), ()>,
    (u64, &'a mut legion::World, &'a DefaultHashBuilder),
);

impl<'a> PendingInsert<'a> {
    pub fn occupied_id(&self) -> Option<NodeIdentifier> {
        match &self.0 {
            hashbrown::hash_map::RawEntryMut::Occupied(occupied) => Some(occupied.key().clone()),
            _ => None,
        }
    }
    pub fn resolve<T>(&self, id: NodeIdentifier) -> HashedNodeRef<T> {
        self.1
             .1
            .entry_ref(id)
            .map(|x| HashedNodeRef::new(x))
            .unwrap()
    }
    pub fn occupied(
        &'a self,
    ) -> Option<(
        NodeIdentifier,
        (u64, &'a legion::World, &'a DefaultHashBuilder),
    )> {
        match &self.0 {
            hashbrown::hash_map::RawEntryMut::Occupied(occupied) => {
                Some((occupied.key().clone(), (self.1 .0, self.1 .1, self.1 .2)))
            }
            _ => None,
        }
    }

    pub fn vacant(
        self,
    ) -> (
        crate::compat::hash_map::RawVacantEntryMut<'a, legion::Entity, (), ()>,
        (u64, &'a mut legion::World, &'a DefaultHashBuilder),
    ) {
        match self.0 {
            hashbrown::hash_map::RawEntryMut::Vacant(occupied) => (occupied, self.1),
            _ => panic!(),
        }
    }
    // pub fn occupied(&self) -> Option<(
    //     crate::compat::hash_map::RawVacantEntryMut<legion::Entity, (), ()>,
    //     (u64, &mut legion::World, &DefaultHashBuilder),
    // )> {
    //     match self.0 {
    //         hashbrown::hash_map::RawEntryMut::Occupied(occupied) => {
    //             Some(occupied.into_key_value().0.clone())
    //         }
    //         _ => None
    //     }
    // }
}

impl NodeStore {
    pub fn prepare_insertion<'a, Eq: Fn(EntryRef) -> bool, V: Hash>(
        &'a mut self,
        hashable: &'a V,
        eq: Eq,
    ) -> PendingInsert {
        let Self {
            dedup,
            internal: backend,
            ..
        } = self;
        let hash = make_hash(&self.hasher, hashable);
        let entry = dedup.raw_entry_mut().from_hash(hash, |symbol| {
            let r = eq(backend.entry_ref(*symbol).unwrap());
            r
        });
        PendingInsert(entry, (hash, &mut self.internal, &self.hasher))
    }

    pub fn insert_after_prepare<T>(
        (vacant, (hash, internal, hasher)): (
            crate::compat::hash_map::RawVacantEntryMut<legion::Entity, (), ()>,
            (u64, &mut legion::World, &DefaultHashBuilder),
        ),
        components: T,
    ) -> legion::Entity
    where
        Option<T>: IntoComponentSource,
    {
        let (&mut symbol, _) = {
            let symbol = internal.push(components);
            vacant.insert_with_hasher(hash, symbol, (), |id| {
                let node: elem::HashedNodeRef<'_, NodeIdentifier> = internal
                    .entry_ref(*id)
                    .map(|x| HashedNodeRef::new(x))
                    .unwrap();

                make_hash(hasher, &node)
            })
        };
        symbol
    }

    /// uses the dyn builder see dyn_builder::EntityBuilder
    pub fn insert_built_after_prepare(
        (vacant, (hash, internal, hasher)): (
            crate::compat::hash_map::RawVacantEntryMut<legion::Entity, (), ()>,
            (u64, &mut legion::World, &DefaultHashBuilder),
        ),
        components: dyn_builder::BuiltEntity,
    ) -> legion::Entity {
        let (&mut symbol, _) = {
            let symbol = internal.extend(components)[0];
            vacant.insert_with_hasher(hash, symbol, (), |id| {
                let node: elem::HashedNodeRef<'_, NodeIdentifier> = internal
                    .entry_ref(*id)
                    .map(|x| HashedNodeRef::new(x))
                    .unwrap();

                make_hash(hasher, &node)
            })
        };
        symbol
    }

    pub fn resolve(&self, id: NodeIdentifier) -> HashedNodeRef<NodeIdentifier> {
        self.internal
            .entry_ref(id)
            .map(|x| HashedNodeRef::new(x))
            .unwrap()
    }

    pub unsafe fn _resolve<T>(&self, id: &NodeIdentifier) -> HashedNodeRef<T> {
        self.internal
            .entry_ref(*id)
            .map(|x| HashedNodeRef::new(x))
            .unwrap()
    }

    pub fn resolve_with_type<T: 'static + TypedNodeId<IdN = NodeIdentifier>>(
        &self,
        id: &T::IdN,
    ) -> (T::Ty, HashedNodeRef<T>) {
        let n = self
            .internal
            .entry_ref(*id)
            .map(|x| HashedNodeRef::new(x))
            .unwrap();
        (n.get_type(), n)
    }

    pub fn try_resolve(&self, id: NodeIdentifier) -> Option<HashedNodeRef<NodeIdentifier>> {
        self.internal
            .entry_ref(id)
            .map(|x| HashedNodeRef::new(x))
            .ok()
    }

    pub fn resolve_typed<TIdN: 'static + TypedNodeId<IdN = NodeIdentifier>>(
        &self,
        id: &TIdN,
    ) -> HashedNodeRef<TIdN> {
        let x = self.internal.entry_ref(id.as_id().clone()).unwrap();
        HashedNodeRef::new(x)
    }

    pub fn try_resolve_typed<TIdN: 'static + TypedNodeId<IdN = NodeIdentifier>>(
        &self,
        id: &TIdN::IdN,
    ) -> Option<(HashedNodeRef<TIdN>, TIdN)> {
        let x = self.internal.entry_ref(id.clone()).unwrap();
        x.get_component::<TIdN::Ty>().ok()?;
        Some((HashedNodeRef::new(x), unsafe { TIdN::from_id(id.clone()) }))
    }
}

impl Debug for NodeStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeStore")
            .field("count", &self.count)
            .field("errors", &self.errors)
            .field("internal_len", &self.internal.len())
            // .field("internal", &self.internal)
            .finish()
    }
}

impl crate::types::NodeStore<NodeIdentifier> for NodeStore {
    type R<'a> = HashedNodeRef<'a, NodeIdentifier>;
    fn resolve(&self, id: &NodeIdentifier) -> Self::R<'_> {
        self.internal
            .entry_ref(id.clone())
            .map(|x| HashedNodeRef::new(x))
            .unwrap()
    }
}

impl<'a> crate::types::NodeStoreLean<NodeIdentifier> for &'a NodeStore {
    type R = HashedNodeRef<'a, NodeIdentifier>;
    fn resolve(&self, id: &NodeIdentifier) -> Self::R {
        self.internal
            .entry_ref(id.clone())
            .map(|x| HashedNodeRef::new(x))
            .unwrap()
    }
}

impl<TIdN: 'static + TypedNodeId<IdN = NodeIdentifier>> crate::types::TypedNodeStore<TIdN>
    for NodeStore
{
    type R<'a> = HashedNodeRef<'a, TIdN>;
    fn resolve(&self, id: &TIdN) -> Self::R<'_> {
        let x = self.internal.entry_ref(id.as_id().clone()).unwrap();
        HashedNodeRef::new(x)
    }

    fn try_typed(&self, id: &<TIdN as NodeId>::IdN) -> Option<TIdN> {
        let x = self.internal.entry_ref(id.clone()).unwrap();
        x.get_component::<TIdN::Ty>()
            .is_ok()
            .then(|| unsafe { TIdN::from_id(id.clone()) })
    }
}

impl NodeStore {
    pub fn len(&self) -> usize {
        self.internal.len()
    }
}

impl NodeStore {
    pub fn new() -> Self {
        Self {
            count: 0,
            errors: 0,
            // roots: Default::default(),
            internal: Default::default(),
            dedup: hashbrown::HashMap::<_, (), ()>::with_capacity_and_hasher(
                1 << 10,
                Default::default(),
            ),
            hasher: Default::default(),
        }
    }
}
impl Default for NodeStore {
    fn default() -> Self {
        Self::new()
    }
}

// // impl<'a> crate::types::NodeStore<'a, NodeIdentifier, HashedNodeRef<'a>> for NodeStore {
// //     fn resolve(&'a self, id: &NodeIdentifier) -> HashedNodeRef<'a> {
// //         self.internal
// //             .entry_ref(id.clone())
// //             .map(|x| HashedNodeRef(x))
// //             .unwrap()
// //     }
// // }

// // impl crate::types::NodeStore3<NodeIdentifier> for NodeStore {
// //     type R = dyn for<'any> GenericItem<'any, Item = HashedNodeRef<'any>>;
// //     fn resolve(&self, id: &NodeIdentifier) -> HashedNodeRef<'_> {
// //         self.internal
// //             .entry_ref(id.clone())
// //             .map(|x| HashedNodeRef(x))
// //             .unwrap()
// //     }
// // }

// // impl crate::types::NodeStore4<NodeIdentifier> for NodeStore {
// //     type R<'a> = HashedNodeRef<'a>;
// //     fn resolve(&self, id: &NodeIdentifier) -> HashedNodeRef<'_> {
// //         self.internal
// //             .entry_ref(id.clone())
// //             .map(|x| HashedNodeRef(x))
// //             .unwrap()
// //     }
// // }

// // impl crate::types::NodeStore2<NodeIdentifier> for NodeStore{
// //     type R<'a> = HashedNodeRef<'a>;
// //     fn resolve(&self, id: &NodeIdentifier) -> HashedNodeRef<'_> {
// //         self.internal
// //             .entry_ref(id.clone())
// //             .map(|x| HashedNodeRef(x))
// //             .unwrap()
// //     }
// // }

// // impl<'a> crate::types::NodeStoreMut<'a, HashedNode, HashedNodeRef<'a>> for NodeStore {
// //     fn get_or_insert(
// //         &mut self,
// //         node: HashedNode,
// //     ) -> <HashedNodeRef<'a> as crate::types::Stored>::TreeId {
// //         todo!()
// //     }
// // }
// impl<'a> crate::types::NodeStoreMut<HashedNode> for NodeStore {
//     fn get_or_insert(
//         &mut self,
//         node: HashedNode,
//     ) -> <HashedNodeRef<'a> as crate::types::Stored>::TreeId {
//         todo!()
//     }
// }

// // impl<'a> crate::types::NodeStoreExt<'a, HashedNode, HashedNodeRef<'a>> for NodeStore {
// //     fn build_then_insert(
// //         &mut self,
// //         t: <HashedNodeRef<'a> as crate::types::Typed>::Type,
// //         l: <HashedNodeRef<'a> as crate::types::Labeled>::Label,
// //         cs: Vec<<HashedNodeRef<'a> as crate::types::Stored>::TreeId>,
// //     ) -> <HashedNodeRef<'a> as crate::types::Stored>::TreeId {
// //         todo!()
// //     }
// // }

// /// WARN this is polyglote related
// /// for now I only implemented for java.
// /// In the future you should use the Type of the node
// /// and maybe an additional context might be necessary depending on choices to materialize polyglot nodes
// impl crate::types::NodeStoreExt<HashedNode> for NodeStore {
//     fn build_then_insert(
//         &mut self,
//         i: <HashedNode as crate::types::Stored>::TreeId,
//         t: <HashedNode as crate::types::Typed>::Type,
//         l: Option<<HashedNode as crate::types::Labeled>::Label>,
//         cs: Vec<<HashedNode as crate::types::Stored>::TreeId>,
//     ) -> <HashedNode as crate::types::Stored>::TreeId {
//         // self.internal.
//         todo!()
//     }
// }

mod stores_impl {
    use crate::{
        store::{labels, nodes, SimpleStores},
        types::{
            AnyType, HyperAST, HyperASTAsso, HyperASTLean, HyperASTShared, TypeStore,
            TypedHyperAST, TypedNodeId,
        },
    };

    impl<TS> HyperASTShared for SimpleStores<TS, nodes::DefaultNodeStore> {
        type IdN = nodes::DefaultNodeIdentifier;

        type Idx = u16;
        type Label = labels::DefaultLabelIdentifier;
    }

    impl<'store, TS> HyperASTLean for &'store SimpleStores<TS, nodes::DefaultNodeStore>
    where
        TS: TypeStore<self::nodes::legion::HashedNodeRef<'store>>,
    {
        type T = self::nodes::legion::HashedNodeRef<'store, Self::IdN>;

        type NS = nodes::legion::NodeStore;

        fn node_store(&self) -> &Self::NS {
            &self.node_store
        }

        type LS = labels::LabelStore;

        fn label_store(&self) -> &Self::LS {
            &self.label_store
        }

        type TS = TS;

        fn type_store(&self) -> &Self::TS {
            &self.type_store
        }
    }

    impl<'store, TS> HyperASTAsso for &'store SimpleStores<TS, nodes::DefaultNodeStore>
    where
        TS: for<'s> TypeStore<self::nodes::legion::HashedNodeRef<'s>>,
    {
        type T<'s> = self::nodes::legion::HashedNodeRef<'s, Self::IdN> where Self: 's;

        type NS<'s> = nodes::legion::NodeStore where Self: 's;

        fn node_store(&self) -> &Self::NS<'_> {
            &self.node_store
        }

        type LS = labels::LabelStore;

        fn label_store(&self) -> &Self::LS {
            &self.label_store
        }

        type TS<'s> = TS where Self: 's;

        fn type_store(&self) -> &Self::TS<'_> {
            &self.type_store
        }
    }

    // impl<TS> HyperASTAsso for SimpleStores<TS, nodes::DefaultNodeStore>
    // where
    //     TS: for<'s> TypeStore<self::nodes::legion::elem::HashedNodeRef<'s, Self::IdN>>,
    // {
    //     type T<'s> = self::nodes::legion::HashedNodeRef<'s, Self::IdN> where TS: 's;

    //     type NS<'s> = nodes::legion::NodeStore where Self: 's;

    //     fn node_store(&self) -> &Self::NS<'_> {
    //         &self.node_store
    //     }

    //     type LS = labels::LabelStore;

    //     fn label_store(&self) -> &Self::LS {
    //         &self.label_store
    //     }

    //     type TS<'s> = TS where TS: 's;

    //     fn type_store(&self) -> &Self::TS<'_> {
    //         &self.type_store
    //     }
    // }

    impl<'store, TS> HyperAST<'store> for SimpleStores<TS, nodes::DefaultNodeStore>
    where
        // <TS as TypeStore>::Ty: HyperType + Send + Sync,
        TS: TypeStore<
            self::nodes::legion::HashedNodeRef<'store, nodes::DefaultNodeIdentifier>,
            Ty = AnyType,
        >,
    {
        type IdN = nodes::DefaultNodeIdentifier;

        type Idx = u16;
        type Label = labels::DefaultLabelIdentifier;

        type T = self::nodes::legion::HashedNodeRef<'store, Self::IdN>;

        type NS = nodes::legion::NodeStore;

        fn node_store(&self) -> &Self::NS {
            &self.node_store
        }

        type LS = labels::LabelStore;

        fn label_store(&self) -> &Self::LS {
            &self.label_store
        }

        type TS = TS;

        fn type_store(&self) -> &Self::TS {
            &self.type_store
        }
    }

    impl<'store, TIdN, TS> TypedHyperAST<'store, TIdN> for SimpleStores<TS, nodes::DefaultNodeStore>
    where
        TIdN: 'static + TypedNodeId<IdN = Self::IdN>,
        TS: TypeStore<
            self::nodes::legion::HashedNodeRef<'store, nodes::DefaultNodeIdentifier>,
            Ty = AnyType,
        >,
    {
        type TT = self::nodes::legion::HashedNodeRef<'store, TIdN>;
        type TNS = nodes::legion::NodeStore;

        fn typed_node_store(&self) -> &Self::TNS {
            &self.node_store
        }
    }
}
