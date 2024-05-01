use super::*;
use hashbrown::hash_map::DefaultHashBuilder;
use hecs::{Entity, EntityRef as EntryRef, World};

pub struct PendingInsert<'a>(
    crate::compat::hash_map::RawEntryMut<'a, Entity, (), ()>,
    (u64, &'a mut World, &'a DefaultHashBuilder),
);

impl<'a> PendingInsert<'a> {
    pub fn occupied_id(&self) -> Option<NodeIdentifier> {
        match &self.0 {
            hashbrown::hash_map::RawEntryMut::Occupied(occupied) => Some(occupied.key().clone()),
            _ => None,
        }
    }
    pub fn resolve<T>(&self, id: NodeIdentifier) -> HashedNodeRef<T> {
        self.1 .1.entity(id).map(|x| HashedNodeRef::new(x)).unwrap()
    }
    pub fn occupied(
        &'a self,
    ) -> Option<(NodeIdentifier, (u64, &'a World, &'a DefaultHashBuilder))> {
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
        crate::compat::hash_map::RawVacantEntryMut<'a, Entity, (), ()>,
        (u64, &'a mut World, &'a DefaultHashBuilder),
    ) {
        match self.0 {
            hashbrown::hash_map::RawEntryMut::Vacant(occupied) => (occupied, self.1),
            _ => panic!(),
        }
    }
}

impl NodeStore {
    pub fn prepare_insertion<'a, Eq: Fn(EntryRef) -> bool, V: std::hash::Hash>(
        &'a mut self,
        hashable: &'a V,
        eq: Eq,
    ) -> PendingInsert {
        let Self {
            dedup,
            internal: backend,
            ..
        } = self;
        let hash = crate::utils::make_hash(&self.hasher, hashable);
        let entry = dedup.raw_entry_mut().from_hash(hash, |symbol| {
            let r = eq(backend.entity(*symbol).unwrap());
            r
        });
        PendingInsert(entry, (hash, &mut self.internal, &self.hasher))
    }

    pub fn insert_after_prepare<T>(
        (vacant, (hash, internal, hasher)): (
            crate::compat::hash_map::RawVacantEntryMut<Entity, (), ()>,
            (u64, &mut World, &DefaultHashBuilder),
        ),
        components: T,
    ) -> Entity
    where
        T: hecs::DynamicBundle,
    {
        let (&mut symbol, _) = {
            let symbol = internal.spawn(components);
            vacant.insert_with_hasher(hash, symbol, (), |id| {
                let node: elem::HashedNodeRef<'_, NodeIdentifier> =
                    internal.entity(*id).map(|x| HashedNodeRef::new(x)).unwrap();

                crate::utils::make_hash(hasher, &node)
            })
        };
        symbol
    }
}
