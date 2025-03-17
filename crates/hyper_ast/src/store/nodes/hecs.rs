mod compo;
mod elem;
pub use elem::{HashedNodeRef, NodeIdentifier};

pub struct NodeStore {
    count: usize,
    errors: usize,
    dedup: hashbrown::HashMap<NodeIdentifier, (), ()>,
    internal: hecs::World,
    hasher: hashbrown::hash_map::DefaultHashBuilder,
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

mod store_read;
mod store_write;
pub use store_write::PendingInsert;

#[test]
fn simple_vanilla_hecs() {
    use hecs::World;
    let mut world = World::new();
    let entity = world.spawn((0u32, "coucou"));
    let entry = world.entity(entity).unwrap();
    let s: &str = *entry.get::<&&str>().unwrap();
    assert_eq!("coucou", s);
    let i = *entry.get::<&u32>().unwrap();
    assert_eq!(0u32, i);
}

#[test]
fn builder_vanilla_hecs() {
    use hecs::{EntityBuilder, World};
    let mut world = World::new();
    let mut builder = EntityBuilder::new();
    builder.add(0u32).add("coucou");
    let built = builder.build();
    let entity = world.spawn(built);
    let entry = world.entity(entity).unwrap();
    let s: &str = *entry.get::<&&str>().unwrap();
    assert_eq!("coucou", s);
    let i = *entry.get::<&u32>().unwrap();
    assert_eq!(0u32, i);
}

#[test]
fn simple() {
    let mut node_store = NodeStore::new();
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Ty(&'static str);
    let ty = Ty("public");
    let pending = node_store.prepare_insertion(&ty.0, |x| {
        use std::ops::Deref;
        if x.get::<&Ty>().unwrap().deref() != &ty {
            return false;
        }
        // check other components
        true
    });
    if let Some(_) = pending.occupied() {
        // pass through some data
        unreachable!()
    }
    let vacant = pending.vacant();
    let mut builder = hecs::EntityBuilder::new();
    builder.add(ty);
    // add additional components,
    // while also possibly computing stuff
    let built = builder.build();
    let entity = NodeStore::insert_after_prepare(vacant, built);
    {
        // check if everything is there
        use crate::types::NodeStore;
        todo!()
        // let entry = node_store.resolve(&entity);
        // use std::ops::Deref;
        // if let Some(t) = entry.get_component::<&Ty>() {
        //     assert_eq!(&ty, t.deref());
        // } else {
        //     unreachable!()
        // };
    }
}
