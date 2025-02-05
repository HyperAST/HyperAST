use std::num::{NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8};

/// Small entity
/// Index+Id for nodes representing small subtrees
/// actual stored data:
/// - do not directly contain labels
/// - should not need to be structurally hashed as (meta,id) is uniq
/// - nor directly contain spaces, do same thing done for labels
/// * kind:NonZeroU16, size:u8, height: u8, children:[EntityU16;N] with 0<N<?
// /// * kind:NonZeroU16, children:Box<[EntityU16]> with children.len() > N
#[derive(Debug, Clone, Copy)]
pub struct EntityU16 {
    /// if [0] == 1
    ///     [1..] used for kind of node
    /// [1..5] for N the number of children .ie 16+1 children as leaf are handled
    /// thus, size > N but for now we will limit to 16+1 total size (later attempt 255)
    /// because we need to be cautious about the fact that we cannot ref Medium from Small
    /// [5..16] for id .ie 2048 id to increment per #children
    content: NonZeroU16,
}

const LEAF_FLAG: u16 = 14;
const CHILDREN_COUNT: u16 = 9;

impl EntityU16 {
    pub fn new_leaf(kind: u16) -> Self {
        let content = kind;
        if content & 1 << LEAF_FLAG != 0 {
            panic!("too much types (first bit internally used)");
        }
        let content = content | 1 << LEAF_FLAG;
        let content = NonZeroU16::new(content).unwrap();
        Self { content }
    }

    pub fn new_node(children_count: u8, id: u16) -> Self {
        if children_count >= 16 {
            panic!("too much children");
        }
        assert_ne!(children_count, 0);
        if id >= 1024 {
            panic!("not enough ids");
        }

        let content = children_count - 1;
        let content = content as u16;
        let content = content << CHILDREN_COUNT;
        let content = content | id;

        let content = NonZeroU16::new(content).unwrap();
        Self { content }
    }

    fn is_leaf(content: u16) -> bool {
        content & 1 << LEAF_FLAG != 0
    }

    pub fn try_type(&self) -> Option<u16> {
        let content = self.content.get();
        if Self::is_leaf(content) {
            Some(content & !(1 << LEAF_FLAG))
        } else {
            None
        }
    }

    pub fn try_len(&self) -> Option<u8> {
        let content = self.content.get();
        if Self::is_leaf(content) {
            None
        } else {
            let content = content & !(1 << LEAF_FLAG);
            let content = (content >> CHILDREN_COUNT) as u8;
            Some(content)
        }
    }

    pub fn try_id(&self) -> Option<u16> {
        let content = self.content.get();
        if Self::is_leaf(content) {
            None
        } else {
            let content = content & 0b111111111; // 11 last bits
            Some(content)
        }
    }

    fn as_u16(&self) -> u16 {
        self.content.get()
    }
}

enum LabelPlaceHolder<T> {
    Label,
    Other(T),
}

struct SmallData<const N: usize> {
    kind: u16,
    size: NonZeroU8,
    height: u8,
    children: [LabelPlaceHolder<EntityU16>; N],
}

/// Medium entity
/// actual stored data:
/// - keep track of labels and spaces maps
/// - and also other optimizations
#[derive(Debug, Clone, Copy)]
pub struct EntityU32 {
    /// [0..16] == 0 flag EntityU16 wrapper
    /// if [0] == 1
    ///     [1..8] for children.len()
    ///     [8..16] for storeIDperLength
    /// else
    ///     [1..16] for storeID
    /// [16..32] incremented id 60k elements
    content: NonZeroU32,
}
const WRAP32_FLAG: u32 = 0xffff0000;
const MEDIUM_CHILD_FLAG: u32 = 31;
const MEDIUM_CHILDREN_COUNT: u32 = 24;
const STORE_OFFSET: u32 = 16;

impl EntityU32 {
    fn wrap(small: EntityU16) -> Self {
        Self {
            content: NonZeroU32::new(small.as_u16() as u32).unwrap(),
        }
    }

    pub fn new_node(children_count: u16, id: u16) -> Self {
        assert_ne!(children_count, 0);

        let content = children_count - 1;
        let content = content as u32;
        // let content = content << MEDIUM_CHILDREN_COUNT;
        // let content = content | (id as u32);
        // let content = content & !(1 << WRAP32_FLAG);

        let content = NonZeroU32::new(content).unwrap();
        Self { content }
    }

    fn is_wrap(content: u32) -> bool {
        content & WRAP32_FLAG == 0
    }

    fn is_inline_length(content: u32) -> bool {
        content & (1 << MEDIUM_CHILD_FLAG) != 0
    }

    pub fn try_unwrap(&self) -> Option<EntityU16> {
        let content = self.content.get();
        if Self::is_wrap(content) {
            let content = (content & !WRAP32_FLAG) as u16;
            let content = NonZeroU16::new(content).unwrap();
            Some(EntityU16 { content })
        } else {
            None
        }
    }

    pub fn try_len(&self) -> Option<u16> {
        let content = self.content.get();
        if Self::is_wrap(content) {
            self.try_unwrap()
                .unwrap()
                .try_len()
                .and_then(|x| Some(x as u16))
        } else if Self::is_inline_length(content) {
            let content = content & !(1 << MEDIUM_CHILD_FLAG);
            let content = (content >> CHILDREN_COUNT) as u16;
            Some(content)
        } else {
            None
        }
    }

    pub fn try_store(&self) -> Option<u16> {
        let content = self.content.get();
        if Self::is_wrap(content) {
            None
        } else if Self::is_inline_length(content) {
            let content = content & !(1 << MEDIUM_CHILD_FLAG);
            let content = (content >> CHILDREN_COUNT) as u16;
            let content = content as u16 & 0x00ff;
            Some(content)
        } else {
            let content = content & !WRAP32_FLAG;
            Some(content as u16)
        }
    }

    pub fn try_id(&self) -> Option<u16> {
        let content = self.content.get();
        if Self::is_wrap(content) {
            None
        } else {
            let content = content & !WRAP32_FLAG;
            Some(content as u16)
        }
    }

    fn as_u32(&self) -> u32 {
        self.content.get()
    }
}

struct MediumData<CS, H, LS, SS, RS> {
    kind: u16,
    size: NonZeroU16,
    height: u16,
    hash: H,
    labels: LS,
    spaces: SS,
    children: CS,
    refs: RS,
}

mod medium {
    use super::*;
    use crate::filter::Bloom;
    use std::{collections::HashSet, marker::PhantomData};

    struct Kind(u16);
    struct Size(NonZeroU16);
    struct Height(u16);

    struct HStructLabel(u64);
    struct HSyntax(u32);

    struct CS0<const N: usize> {
        children: [EntityU32; N],
    }
    struct CS {
        children: [EntityU32],
    }
    struct LS {
        // small: StoreSmall,
        // big: StoreBig,
        starts: Box<[usize]>,
        length: Box<[Option<NonZeroU16>]>,
    }
    struct RS0<T, const N: usize> {
        bloom: Bloom<T, [u64; N]>,
        _phantom: PhantomData<T>,
    }
    struct RS<T> {
        set: HashSet<T>,
    }
}

/// Large entity should be compatible with legion ?
#[derive(Debug, Clone, Copy)]
pub struct EntityU64 {
    /// [0..32] == 0 flag EntityU16 wrapper
    /// if [0] == 1
    // ///     [1..8] for children.len()
    // ///     [8..32+16] for storeIDperLength
    /// else
    ///     [1..32+16] for storeID
    /// [32+16..64] incremented id 60k elements
    content: NonZeroU64,
}

const WRAP64_FLAG: u64 = 0xffffffff00000000;
impl EntityU64 {
    fn wrap(small: EntityU32) -> Self {
        Self {
            content: NonZeroU64::new(small.as_u32() as u64).unwrap(),
        }
    }

    fn is_wrap(content: u64) -> bool {
        content & WRAP64_FLAG == 0
    }

    pub fn try_unwrap(&self) -> Option<EntityU32> {
        let content = self.content.get();
        if Self::is_wrap(content) {
            let content = (content & !WRAP64_FLAG) as u32;
            let content = NonZeroU32::new(content).unwrap();
            Some(EntityU32 { content })
        } else {
            None
        }
    }
}

mod large {
    use std::marker::PhantomData;

    use crate::filter::Bloom;

    pub use super::medium::*;
    use super::*;

    struct CS0<const N: usize> {
        children: [EntityU64; N],
    }
    struct CS {
        children: [EntityU64],
    }
    struct RS0<T, const N: usize> {
        bloom: Bloom<T, [u64; N]>,
        _phantom: PhantomData<T>,
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;

    #[test]
    fn test() {
        println!("{}", size_of::<EntityU16>());
        println!("{}", size_of::<Option<EntityU16>>());
        println!("{}", size_of::<EntityU32>());
        println!("{}", size_of::<Option<EntityU32>>());
        println!("{}", size_of::<EntityU64>());
        println!("{}", size_of::<Option<EntityU64>>());
        println!("{}", size_of::<Box<()>>());
        println!("{}", size_of::<Box<[u8]>>());
        println!("{}", size_of::<[u8; 1]>());
        println!("{}", size_of::<&[u8]>());
    }
}

const BLOCK_SIZE: u32 = 16;
const BLOCK_SIZE_USIZE: usize = BLOCK_SIZE as usize;

// Always divisible by BLOCK_SIZE.
// Safety: This must never be 0, so skip the first block
static NEXT_ENTITY: AtomicU32 = AtomicU32::new(BLOCK_SIZE);

/// An iterator which yields new entity IDs.
#[derive(Debug)]
pub struct Allocate {
    next: u32,
}

impl Allocate {
    /// Constructs a new enity ID allocator iterator.
    pub fn new() -> Self {
        // This is still safe because the allocator grabs a new block immediately
        Self { next: 0 }
    }
}

impl Default for Allocate {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Iterator for Allocate {
    type Item = Entity;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.next % BLOCK_SIZE == 0 {
            // This is either the first block, or we overflowed to the next block.
            self.next = NEXT_ENTITY.fetch_add(BLOCK_SIZE, Ordering::Relaxed);
            debug_assert_eq!(self.next % BLOCK_SIZE, 0);
        }

        // Safety: self.next can't be 0 as long as the first block is skipped,
        // and no overflow occurs in NEXT_ENTITY
        let entity = unsafe {
            debug_assert_ne!(self.next, 0);
            Entity {
                content: NonZeroU32::new_unchecked(self.next),
            }
        };
        self.next += 1;
        Some(entity)
    }
}

type Entity = EntityU32;

use std::sync::atomic::AtomicU32;
use std::{
    collections::HashMap,
    mem,
    ops::Range,
    sync::atomic::{AtomicU64, Ordering},
};

// use bit_set::BitSet;
// use itertools::Itertools;

use legion::query::LayoutFilter;
use legion::storage::{
    Archetype, ArchetypeIndex, ArchetypeSource, ArchetypeWriter, Component, ComponentStorage,
    ComponentTypeId,
};
use legion::storage::{Components, EntityLayout, SearchIndex};
use legion::world::{ComponentAccess, ComponentError, EntityAccessError};

/// Describes a type which can write entity components into a world.
pub trait ComponentSource: ArchetypeSource {
    /// Writes components for new entities into an archetype.
    fn push_components<'a>(
        &mut self,
        writer: &mut ArchetypeWriter<'a>,
        entities: impl Iterator<Item = Entity>,
    );
}

/// A collection with a known length.
pub trait KnownLength {
    fn len(&self) -> usize;
}

/// Converts a type into a [`ComponentSource`].
pub trait IntoComponentSource {
    /// The output component source.
    type Source: ComponentSource;

    /// Converts this structure into a component source.
    fn into(self) -> Self::Source;
}

// use super::{
//     entity::{Allocate, /*Entity*/, EntityHasher, EntityLocation, LocationMap, ID_CLONE_MAPPINGS},
//     entry::{Entry, EntryMut, EntryRef},
//     event::{EventSender, Subscriber, Subscribers},
//     insert::{ArchetypeSource, ArchetypeWriter, ComponentSource, IntoComponentSource},
//     query::{
//         filter::{EntityFilter, LayoutFilter},
//         view::{IntoView, View},
//         Query,
//     },
//     storage::{
//         archetype::{Archetype, ArchetypeIndex, EntityLayout},
//         component::{Component, ComponentTypeId},
//         group::{Group, GroupDef},
//         index::SearchIndex,
//         ComponentIndex, Components, PackOptions, UnknownComponentStorage,
//     },
//     subworld::{ComponentAccess, SubWorld},
// };

// type MapEntry<'a, K, V> = std::collections::hash_map::Entry<'a, K, V>;

// /// Error type representing a failure to access entity data.
// #[derive(thiserror::Error, Debug, Eq, PartialEq, Hash)]
// pub enum EntityAccessError {
//     /// Attempted to access an entity which lies outside of the subworld.
//     #[error("this world does not have permission to access the entity")]
//     AccessDenied,
//     /// Attempted to access an entity which does not exist.
//     #[error("the entity does not exist")]
//     EntityNotFound,
// }

// /// The `EntityStore` trait abstracts access to entity data as required by queries for
// /// both [`World`] and [`SubWorld`]
pub trait EntityStore {
    /// Returns the world's unique ID.
    fn id(&self) -> WorldId;

    /// Returns an entity entry which can be used to access entity metadata and components.
    fn entry_ref(&self, entity: Entity) -> Result<EntryRef, EntityAccessError>;

    //     /// Returns a mutable entity entry which can be used to access entity metadata and components.
    //     fn entry_mut(&mut self, entity: Entity) -> Result<EntryMut, EntityAccessError>;

    //     /// Returns a component storage accessor for component types declared in the specified [`View`].
    //     fn get_component_storage<V: for<'b> View<'b>>(
    //         &self,
    //     ) -> Result<StorageAccessor, EntityAccessError>;
}

/// Unique identifier for a [`World`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldId(u64);
static WORLD_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

impl WorldId {
    fn next() -> Self {
        WorldId(WORLD_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for WorldId {
    fn default() -> Self {
        Self::next()
    }
}

// /// Describes configuration options for the creation of a new [`World`].
// #[derive(Default)]
// pub struct WorldOptions {
//     /// A vector of component [`GroupDef`]s to provide layout hints for query optimization.
//     pub groups: Vec<GroupDef>,
// }

/// A container of entities.
///
/// Each entity stored inside a world is uniquely identified by an [`Entity`] ID and may have an
/// arbitrary collection of [`Component`]s attached.
///
/// The entities in a world may be efficiently searched and iterated via [queries](crate::query).
#[derive(Debug)]
pub struct World {
    small: SmallWorld,
    id: WorldId,
    index: SearchIndex,
    components: Components,
    // groups: Vec<Group>,
    // group_members: HashMap<ComponentTypeId, usize>,
    archetypes: Vec<Archetype>,
    allocation_buffer: Vec<Entity>,
    // subscribers: Subscribers,
}

// impl Default for World {
//     fn default() -> Self {
//         Self::new(WorldOptions::default())
//     }
// }

impl World {
    //     /// Creates a new world with the given options,
    //     pub fn new(options: WorldOptions) -> Self {
    //         let groups: Vec<Group> = options.groups.into_iter().map(|def| def.into()).collect();
    //         let mut group_members = HashMap::default();
    //         for (i, group) in groups.iter().enumerate() {
    //             for comp in group.components() {
    //                 match group_members.entry(comp) {
    //                     MapEntry::Vacant(entry) => {
    //                         entry.insert(i);
    //                     }
    //                     MapEntry::Occupied(_) => {
    //                         panic!("components can only belong to a single group");
    //                     }
    //                 }
    //             }
    //         }

    //         Self {
    //             id: WorldId::next(),
    //             index: SearchIndex::default(),
    //             components: Components::default(),
    //             groups,
    //             group_members,
    //             archetypes: Vec::default(),
    //             entities: LocationMap::default(),
    //             allocation_buffer: Vec::default(),
    //             subscribers: Subscribers::default(),
    //         }
    //     }

    //     /// Returns the world's unique ID.
    //     pub fn id(&self) -> WorldId {
    //         self.id
    //     }

    //     /// Returns the number of entities in the world.
    //     pub fn len(&self) -> usize {
    //         self.entities.len()
    //     }

    //     /// Returns `true` if the world contains no entities.
    //     pub fn is_empty(&self) -> bool {
    //         self.len() == 0
    //     }

    //     /// Returns `true` if the world contains an entity with the given ID.
    //     pub fn contains(&self, entity: Entity) -> bool {
    //         self.entities.contains(entity)
    //     }

    //     /// Appends a named entity to the word, replacing any existing entity with the given ID.
    //     pub fn push_with_id<T>(&mut self, entity_id: Entity, components: T)
    //     where
    //         Option<T>: IntoComponentSource,
    //     {
    //         self.remove(entity_id);

    //         let mut components = <Option<T> as IntoComponentSource>::into(Some(components));

    //         let arch_index = self.get_archetype_for_components(&mut components);
    //         let archetype = &mut self.archetypes[arch_index.0 as usize];
    //         let mut writer =
    //             ArchetypeWriter::new(arch_index, archetype, self.components.get_multi_mut());
    //         components.push_components(&mut writer, std::iter::once(entity_id));

    //         let (base, entities) = writer.inserted();
    //         self.entities.insert(entities, arch_index, base);
    //     }

    //     /// Appends a new entity to the world. Returns the ID of the new entity.
    //     /// `components` should be a tuple of components to attach to the entity.
    //     ///
    //     /// # Examples
    //     ///
    //     /// Pushing an entity with three components:
    //     /// ```
    //     /// # use legion::*;
    //     /// let mut world = World::default();
    //     /// let _entity = world.push((1usize, false, 5.3f32));
    //     /// ```
    //     ///
    //     /// Pushing an entity with one component (note the tuple syntax):
    //     /// ```
    //     /// # use legion::*;
    //     /// let mut world = World::default();
    //     /// let _entity = world.push((1usize,));
    //     /// ```
    //     pub fn push<T>(&mut self, components: T) -> Entity
    //     where
    //         Option<T>: IntoComponentSource,
    //     {
    //         struct One(Option<Entity>);

    //         impl<'a> Extend<&'a Entity> for One {
    //             fn extend<I: IntoIterator<Item = &'a Entity>>(&mut self, iter: I) {
    //                 debug_assert!(self.0.is_none());
    //                 let mut iter = iter.into_iter();
    //                 self.0 = iter.next().copied();
    //                 debug_assert!(iter.next().is_none());
    //             }
    //         }

    //         let mut o = One(None);
    //         self.extend_out(Some(components), &mut o);
    //         o.0.unwrap()
    //     }

    /// Appends a collection of entities to the world. Returns the IDs of the new entities.
    ///
    /// # Examples
    ///
    /// Inserting a vector of component tuples:
    /// ```
    /// # use legion::*;
    /// let mut world = World::default();
    /// let _entities = world.extend(vec![
    ///     (1usize, false, 5.3f32),
    ///     (2usize, true, 5.3f32),
    ///     (3usize, false, 5.3f32),
    /// ]);
    /// ```
    ///
    /// Inserting a tuple of component vectors:
    /// ```
    /// # use legion::*;
    /// let mut world = World::default();
    /// let _entities = world.extend(
    ///     (
    ///         vec![1usize, 2usize, 3usize],
    ///         vec![false, true, false],
    ///         vec![5.3f32, 5.3f32, 5.2f32],
    ///     )
    ///         .into_soa(),
    /// );
    /// ```
    /// SoA inserts require all vectors to have the same length. These inserts are faster than inserting via an iterator of tuples.
    pub fn extend(&mut self, components: impl IntoComponentSource) -> &[Entity] {
        let mut self_alloc_buf = mem::take(&mut self.allocation_buffer);
        self_alloc_buf.clear();
        self.extend_out(components, &mut self_alloc_buf);
        self.allocation_buffer = self_alloc_buf;

        &self.allocation_buffer
    }

    /// Appends a collection of entities to the world.
    /// Extends the given `out` collection with the IDs of the new entities.
    ///
    /// # Examples
    ///
    /// Inserting a vector of component tuples:
    ///
    /// ```
    /// # use legion::*;
    /// let mut world = World::default();
    /// let mut entities = Vec::new();
    /// world.extend_out(
    ///     vec![
    ///         (1usize, false, 5.3f32),
    ///         (2usize, true, 5.3f32),
    ///         (3usize, false, 5.3f32),
    ///     ],
    ///     &mut entities,
    /// );
    /// ```
    ///
    /// Inserting a tuple of component vectors:
    ///
    /// ```
    /// # use legion::*;
    /// let mut world = World::default();
    /// let mut entities = Vec::new();
    /// // SoA inserts require all vectors to have the same length.
    /// // These inserts are faster than inserting via an iterator of tuples.
    /// world.extend_out(
    ///     (
    ///         vec![1usize, 2usize, 3usize],
    ///         vec![false, true, false],
    ///         vec![5.3f32, 5.3f32, 5.2f32],
    ///     )
    ///         .into_soa(),
    ///     &mut entities,
    /// );
    /// ```
    ///
    /// The collection type is generic over [`Extend`], thus any collection could be used:
    ///
    /// ```
    /// # use legion::*;
    /// let mut world = World::default();
    /// let mut entities = std::collections::VecDeque::new();
    /// world.extend_out(
    ///     vec![
    ///         (1usize, false, 5.3f32),
    ///         (2usize, true, 5.3f32),
    ///         (3usize, false, 5.3f32),
    ///     ],
    ///     &mut entities,
    /// );
    /// ```
    ///
    /// [`Extend`]: std::iter::Extend
    pub fn extend_out<S, E>(&mut self, components: S, out: &mut E)
    where
        S: IntoComponentSource,
        E: for<'a> Extend<&'a Entity>,
    {
        let replaced = {
            let mut components = components.into();

            let arch_index = self.get_archetype_for_components(&mut components);
            let archetype = &mut self.archetypes[arch_index.0 as usize];
            let mut writer =
                ArchetypeWriter::new(arch_index, archetype, self.components.get_multi_mut());
            todo!();
            components.push_components(&mut writer, Allocate::new());

            let (base, entities) = writer.inserted();
            // let r = self.entities.insert(entities, arch_index, base);
            // Extend the given collection with inserted entities.
            // if !r.is_empty() {
            //     todo!(); // out.extend(entities.iter());
            // }

            // r
        };

        // for location in replaced {
        //     self.remove_at_location(location);
        // }
    }

    /// Returns the raw component storage.
    pub fn components(&self) -> &Components {
        &self.components
    }

    //     pub(crate) fn components_mut(&mut self) -> &mut Components {
    //         &mut self.components
    //     }

    pub(crate) fn archetypes(&self) -> &[Archetype] {
        &self.archetypes
    }

    pub(crate) fn get_archetype_for_components<T: ArchetypeSource>(
        &mut self,
        components: &mut T,
    ) -> ArchetypeIndex {
        let index = self.index.search(&components.filter()).next();
        if let Some(index) = index {
            index
        } else {
            self.insert_archetype(components.layout())
        }
    }

    fn insert_archetype(&mut self, layout: EntityLayout) -> ArchetypeIndex {
        // create and insert new archetype
        // self.index.push(&layout);
        let arch_index = ArchetypeIndex(self.archetypes.len() as u32);
        todo!();
        // let subscribers = self.subscribers.matches_layout(layout.component_types());
        // self.archetypes
        //     .push(Archetype::new(arch_index, layout, subscribers));
        // let archetype = &self.archetypes[self.archetypes.len() - 1];

        // // find all groups which contain each component
        // let groups = &mut self.groups;
        // let group_members = &mut self.group_members;
        // let types_by_group = archetype
        //     .layout()
        //     .component_types()
        //     .iter()
        //     .map(|type_id| {
        //         (
        //             match group_members.entry(*type_id) {
        //                 MapEntry::Occupied(entry) => *entry.get(),
        //                 MapEntry::Vacant(entry) => {
        //                     // create a group for the component (by itself) if it does not already have one
        //                     let mut group = GroupDef::new();
        //                     group.add(*type_id);
        //                     groups.push(group.into());
        //                     *entry.insert(groups.len() - 1)
        //                 }
        //             },
        //             *type_id,
        //         )
        //     })
        //     .into_group_map();

        // // insert the archetype into each component storage
        // for (group_index, component_types) in types_by_group.iter() {
        //     let group = &mut self.groups[*group_index];
        //     let index = group.try_insert(arch_index, archetype);
        //     for type_id in component_types {
        //         let storage = self.components.get_or_insert_with(*type_id, || {
        //             let index = archetype
        //                 .layout()
        //                 .component_types()
        //                 .iter()
        //                 .position(|t| t == type_id)
        //                 .unwrap();
        //             archetype.layout().component_constructors()[index]()
        //         });
        //         storage.insert_archetype(arch_index, index);
        //     }
        // }

        arch_index
    }
}

/// Provides safe read-only access to an entity's components.
pub struct EntryRef<'a> {
    pub(crate) location: EntityLocation,
    pub(crate) components: &'a Components,
    pub(crate) archetype: &'a Archetype,
    pub(crate) allowed_components: ComponentAccess<'a>,
}

#[derive(Clone, Copy)]
pub struct EntityLocation {
    index: u16,
}

pub struct ComponentIndex(pub(crate) usize);

impl EntityLocation {
    /// Constructs a new entity location.
    pub fn new(index: u16) -> Self {
        EntityLocation { index }
    }

    // /// Returns the entity's archetype index.
    // pub fn archetype(&self) -> ArchetypeIndex {
    //     todo!()
    // }

    /// Returns the entity's component index within its archetype.
    pub fn component(&self) -> ComponentIndex {
        ComponentIndex(self.index as usize)
    }
}

impl<'a> EntryRef<'a> {
    pub(crate) fn new(
        location: EntityLocation,
        components: &'a Components,
        archetype: &'a Archetype,
        allowed_components: ComponentAccess<'a>,
    ) -> Self {
        Self {
            location,
            components,
            archetype,
            allowed_components,
        }
    }

    /// Returns the entity's archetype.
    pub fn archetype(&self) -> &Archetype {
        &self.archetype
    }

    /// Returns the entity's location.
    pub fn location(&self) -> EntityLocation {
        self.location
    }

    /// Returns a reference to one of the entity's components.
    pub fn into_component<T: Component>(self) -> Result<&'a T, ComponentError> {
        let component_type = ComponentTypeId::of::<T>();
        if !self.allowed_components.allows_read(component_type) {
            return Err(ComponentError::Denied {
                component_type,
                component_name: std::any::type_name::<T>(),
            });
        }

        let component = self.location.component();
        let archetype = self.archetype();
        todo!();
        // self.components
        //     .get_downcast::<T>()
        //     .and_then(move |storage| storage.get(archetype))
        //     .and_then(move |slice| slice.into_slice().get(component.0))
        //     .ok_or_else(|| {
        //         ComponentError::NotFound {
        //             component_type,
        //             component_name: std::any::type_name::<T>(),
        //         }
        //     })
    }

    /// Returns a reference to one of the entity's components.
    pub fn get_component<T: Component>(&self) -> Result<&T, ComponentError> {
        todo!()
        // let component_type = ComponentTypeId::of::<T>();
        // if !self.allowed_components.allows_read(component_type) {
        //     return Err(ComponentError::Denied {
        //         component_type,
        //         component_name: std::any::type_name::<T>(),
        //     });
        // }

        // let component = self.location.component();
        // let archetype = self.archetype();
        // self.components
        //     .get_downcast::<T>()
        //     .and_then(move |storage| storage.get(archetype))
        //     .and_then(move |slice| slice.into_slice().get(component.0))
        //     .ok_or_else(|| {
        //         ComponentError::NotFound {
        //             component_type,
        //             component_name: std::any::type_name::<T>(),
        //         }
        //     })
    }
}
#[derive(Debug)]
struct SmallWorld {}

impl SmallWorld {
    fn archetype(&self, len: u8) -> &Archetype {
        todo!()
    }

    fn leaf_archetype(&self) -> &Archetype {
        todo!()
    }
}
impl EntityStore for World {
    fn entry_ref(&self, entity: Entity) -> Result<EntryRef, EntityAccessError> {
        entity
            .try_store()
            .map(|x| {
                EntryRef::new(
                    EntityLocation::new(entity.try_id().unwrap()),
                    &self.components,
                    &self.archetypes[x as usize],
                    ComponentAccess::All,
                )
            })
            .or_else(|| {
                let unwrapped = entity.try_unwrap().unwrap();
                unwrapped
                    .try_len()
                    .map(|len| {
                        let small = self.small.archetype(len);
                        EntryRef::new(
                            EntityLocation::new(unwrapped.try_id().unwrap()),
                            &self.components,
                            small,
                            ComponentAccess::All,
                        )
                    })
                    .or_else(|| {
                        let small = self.small.leaf_archetype();
                        unwrapped.try_type().map(|x| {
                            EntryRef::new(
                                EntityLocation::new(x),
                                &self.components,
                                small,
                                ComponentAccess::All,
                            )
                        })
                    })
            })
            .ok_or(EntityAccessError::EntityNotFound)
    }

    //     fn entry_mut(&mut self, entity: Entity) -> Result<EntryMut, EntityAccessError> {
    //         // safety: we have exclusive access to the world
    //         unsafe { self.entry_unchecked(entity) }
    //     }

    //     fn get_component_storage<V: for<'b> View<'b>>(
    //         &self,
    //     ) -> Result<StorageAccessor, EntityAccessError> {
    //         Ok(StorageAccessor::new(
    //             self.id,
    //             &self.index,
    //             &self.components,
    //             &self.archetypes,
    //             &self.groups,
    //             &self.group_members,
    //             None,
    //         ))
    //     }

    fn id(&self) -> WorldId {
        self.id
    }
}
