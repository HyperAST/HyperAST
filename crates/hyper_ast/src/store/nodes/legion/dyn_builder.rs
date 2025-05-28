//! Mixing hecs entity builder with legion intoComponentSource
//! ```
//! let mut world = legion::World::new(Default::default());
//! let mut components = EntityBuilder::new();
//! components.add(42i32);
//! components.add(true);
//! components.add(vec![0, 1, 2, 3]);
//! components.add("hello");
//! components.add(0u64);
//! let components = components.build();
//! let entity = world.extend(components)[0];
//! assert_eq!(Ok(&42), world.entry(entity).unwrap().get_component::<i32>());
//! assert_eq!(Ok(&vec![0, 1, 2, 3]), world.entry(entity).unwrap().get_component::<Vec<i32>>());
//!
//! ```

// WIP
// TODO try to use it in generators to facilitate adding new metadata
//! # Possible build facilities
//! ```
//! struct Builder<T, S> {
//!     inner: BuiltEntity,
//!     phantom: PhantomData<T,S>
//! }
//! fn new<T:TypeTrait, H:NodeHashs>(t:T, h:H) -> Builder<T,Typed> {
//!     let mut inner = BuiltEntity::default();
//!     inner.add(t);
//!     inner.add(h);
//!     Builder {
//!         inner,
//!         phantom: PhantomData
//!     }
//! }
//!
//! trait Final {}
//! struct Typed;
//! struct Keyword;
//! impl Final for Keyword {}
//! struct Labeled;
//! impl Final for Labeled {}
//! struct WithChildren;
//! impl Final for WithChildren {}
//!
//! // use a bound on T to know if it can have a label ?
//! impl<T> Builder<T,Typed> {
//!     pub fn label(self, l: LabelIdentifier) -> Builder<T, Labeled> {
//!         let mut inner = self.inner;
//!         inner.add(l);
//!         Builder {
//!             inner,
//!             phantom: PhantomData
//!         }
//!     }
//!     pub fn children(self, cs: Children) -> Builder<T, WithChildren> {
//!         let mut inner = self.inner;
//!         inner.add(cs);
//!         Builder {
//!             inner,
//!             phantom: PhantomData
//!         }
//!     }
//!     pub fn add_metadata(self, md: MD) -> Builder<T, Keyword> {
//!         let mut inner = self.inner;
//!         inner.add(md);
//!         Builder {
//!             inner,
//!             phantom: PhantomData
//!         }
//!     }
//! }
//!
//! impl<T, S:Final> Builder<T,S> {
//!     pub fn add_metadata(self, md: MD) -> Builder<T, S> {
//!         let mut inner = self.inner;
//!         inner.add(md);
//!         Builder {
//!             inner,
//!             phantom: PhantomData
//!         }
//!     }
//!     pub fn build(self) -> BuiltEntity {
//!         self.inner.build()
//!     }
//! }
//!
//! ```

use std::{
    alloc::{Layout, alloc, dealloc},
    any::TypeId,
    collections::HashMap,
    hash::{BuildHasher, BuildHasherDefault, Hasher},
    ptr::NonNull,
};

use legion::{
    Entity,
    query::{FilterResult, LayoutFilter},
    storage::{
        ArchetypeSource, ArchetypeWriter, ComponentSource, ComponentTypeId, EntityLayout,
        UnknownComponentStorage,
    },
};

use super::*;

/// A builder of entities for a archetypal store, here legion.
pub struct BuiltEntity {
    inner: Common<fn() -> Box<dyn UnknownComponentStorage>>,
}

impl Debug for BuiltEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuiltEntity").finish()
    }
}

#[derive(Default)]
pub struct EntityBuilder {
    inner: Common<fn() -> Box<dyn UnknownComponentStorage>>,
}

impl EntityBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn build(self) -> BuiltEntity {
        BuiltEntity { inner: self.inner }
    }
    /// Add `component` to the entity.
    ///
    /// If the bundle already contains a component of type `T`, it will be dropped and replaced with
    /// the most recently added one.
    pub(crate) fn _add<T: legion::storage::Component>(&mut self, mut component: T) -> &mut Self {
        unsafe {
            self.inner.add(
                (&mut component as *mut T).cast(),
                TypeInfo::of::<T>(),
                || Box::new(T::Storage::default()), //DynamicClone::new::<T>(),
            );
        }
        core::mem::forget(component);
        self
    }
}

impl super::super::EntityBuilder for EntityBuilder {
    /// Add `component` to the entity.
    ///
    /// If the bundle already contains a component of type `T`, it will be dropped and replaced with
    /// the most recently added one.
    fn add<T: Component>(&mut self, component: T) -> &mut Self {
        self._add(component)
    }
}

impl IntoComponentSource for BuiltEntity {
    type Source = BuiltEntity;

    fn into(self) -> Self::Source {
        self
    }
}

impl IntoComponentSource for EntityBuilder {
    type Source = BuiltEntity;

    fn into(self) -> Self::Source {
        self.build()
    }
}

/// A layout filter used to select the appropriate archetype for inserting
/// entities from a component source into a world.
pub struct ComponentSourceFilter(Vec<ComponentTypeId>);

// impl Default for ComponentSourceFilter {
//     fn default() -> Self {
//         ComponentSourceFilter(PhantomData)
//     }
// }

impl LayoutFilter for ComponentSourceFilter {
    fn matches_layout(&self, components: &[ComponentTypeId]) -> FilterResult {
        // FilterResult::Match(components.is_empty())
        // TODO check if inverted
        FilterResult::Match(
            components.len() == self.0.len() && components.iter().all(|x| self.0.contains(x)),
        )
    }
}

impl ArchetypeSource for BuiltEntity {
    type Filter = ComponentSourceFilter;

    fn filter(&self) -> Self::Filter {
        let v = self.inner.inner.info.iter().map(|x| x.0.id()).collect();
        ComponentSourceFilter(v)
    }

    fn layout(&mut self) -> EntityLayout {
        let mut layout = EntityLayout::default();

        for (tid, _offset, meta) in &self.inner.inner.info {
            unsafe {
                layout.register_component_raw(tid.id(), meta.clone());
            }
        }

        layout
    }
}

impl ComponentSource for BuiltEntity {
    fn push_components<'a>(
        &mut self,
        writer: &mut ArchetypeWriter<'a>,
        mut entities: impl Iterator<Item = Entity>,
    ) {
        let entity = entities.next().unwrap();
        writer.push(entity);

        // let v = unsafe { Vec::from_raw_parts(self.inner.storage.as_ptr(), self.inner.cursor, 4) };
        // dbg!(&v);
        // std::mem::forget(v);

        for (ty, offset, _) in &mut self.inner.inner.info {
            let mut target = writer.claim_components_unknown(ty.id());
            let ptr = unsafe { self.inner.inner.storage.as_ptr().add(*offset) };
            // let len = ty.layout().size();

            // eprintln!();
            // eprintln!("store:  {:?}", self.inner.storage.as_ptr());
            // eprintln!("ptr:    {:p}", ptr);
            // eprintln!("off:    {:?}", offset);
            // eprintln!("cursor: {:?}", self.inner.cursor);
            // eprintln!("len:    {:?}", len);
            // if ty.id().type_id() == TypeId::of::<(Vec<usize>,)>() {
            //     let aaa = ptr as *mut (Vec<usize>,);
            //     // dbg!(unsafe { aaa.as_ref() });
            // } else if ty.id().type_id() == TypeId::of::<(Box<[u32]>,)>() {
            //     let aaa = ptr as *mut (Box<[u32]>,);
            //     // dbg!(unsafe { aaa.as_ref() });
            // } else if ty.id().type_id() == TypeId::of::<Vec<u64>>() {
            //     let aaa = ptr as *mut Vec<u64>;
            //     // dbg!(unsafe { aaa.as_ref() });
            // }
            let len = 1; // actually the len of slice, not the len of the component
            unsafe { target.extend_memcopy_raw(ptr, len) };
            // if ty.id().type_id() == TypeId::of::<(Vec<usize>,)>() {
            //     let aaa = ptr as *mut (Vec<usize>,);
            //     // dbg!(unsafe { aaa.as_ref() });
            // } else if ty.id().type_id() == TypeId::of::<(Box<[u32]>,)>() {
            //     let aaa = ptr as *mut (Box<[u32]>,);
            //     // dbg!(unsafe { aaa.as_ref() });
            // } else if ty.id().type_id() == TypeId::of::<Vec<u64>>() {
            //     let aaa = ptr as *mut Vec<u64>;
            //     // dbg!(unsafe { aaa.as_ref() });
            // }
        }
    }
}

// impl legion::internals::insert::KnownLength for DynBuiltEntity<(), Iter>
// where
//     Iter: ExactSizeIterator,
// {
//     fn len(&self) -> usize {
//         self.iter.len()
//     }
// }

/// A hasher optimized for hashing a single TypeId.
///
/// TypeId is already thoroughly hashed, so there's no reason to hash it again.
/// Just leave the bits unchanged.
#[derive(Default)]
pub(crate) struct TypeIdHasher {
    hash: u64,
}

impl Hasher for TypeIdHasher {
    fn write_u64(&mut self, n: u64) {
        // Only a single value can be hashed, so the old hash should be zero.
        debug_assert_eq!(self.hash, 0);
        self.hash = n;
    }

    // Tolerate TypeId being either u64 or u128.
    fn write_u128(&mut self, n: u128) {
        debug_assert_eq!(self.hash, 0);
        self.hash = n as u64;
    }

    fn write(&mut self, bytes: &[u8]) {
        debug_assert_eq!(self.hash, 0);

        // This will only be called if TypeId is neither u64 nor u128, which is not anticipated.
        // In that case we'll just fall back to using a different hash implementation.
        let mut hasher = <DefaultHashBuilder as BuildHasher>::Hasher::default();
        hasher.write(bytes);
        self.hash = hasher.finish();
    }

    fn finish(&self) -> u64 {
        self.hash
    }
}

/// A HashMap with TypeId keys
///
/// Because TypeId is already a fully-hashed u64 (including data in the high seven bits,
/// which hashbrown needs), there is no need to hash it again. Instead, this uses the much
/// faster no-op hash.
pub(crate) type TypeIdMap<V> = HashMap<TypeId, V, BuildHasherDefault<TypeIdHasher>>;

/// Metadata required to store a component.
///
/// All told, this means a [`TypeId`], to be able to dynamically name/check the component type; a
/// [`Layout`], so that we know how to allocate memory for this component type; and a drop function
/// which internally calls [`core::ptr::drop_in_place`] with the correct type parameter.
#[derive(Debug, Copy, Clone)]
pub struct TypeInfo {
    id: ComponentTypeId,
    layout: Layout,
    drop: unsafe fn(*mut u8),
    // #[cfg(debug_assertions)]
    // type_name: &'static str,
}

impl TypeInfo {
    /// Construct a `TypeInfo` directly from the static type.
    pub fn of<T: 'static + Send + Sync>() -> Self {
        unsafe fn drop_ptr<T>(x: *mut u8) {
            unsafe { x.cast::<T>().drop_in_place() }
        }

        Self {
            id: ComponentTypeId::of::<T>(),
            layout: Layout::new::<T>(),
            drop: drop_ptr::<T>,
            // #[cfg(debug_assertions)]
            // type_name: core::any::type_name::<T>(),
        }
    }

    // /// Construct a `TypeInfo` from its components. This is useful in the rare case that you have
    // /// some kind of pointer to raw bytes/erased memory holding a component type, coming from a
    // /// source unrelated to hecs, and you want to treat it as an insertable component by
    // /// implementing the `DynamicBundle` API.
    // pub fn from_parts(id: ComponentTypeId, layout: Layout, drop: unsafe fn(*mut u8)) -> Self {
    //     Self {
    //         id,
    //         layout,
    //         drop,
    //         // #[cfg(debug_assertions)]
    //         // type_name: "<unknown> (TypeInfo constructed from parts)",
    //     }
    // }

    /// Access the `TypeId` for this component type.
    pub fn id(&self) -> ComponentTypeId {
        self.id
    }

    /// Access the `Layout` of this component type.
    pub fn layout(&self) -> Layout {
        self.layout
    }

    /// Directly call the destructor on a pointer to data of this component type.
    ///
    /// # Safety
    ///
    /// All of the caveats of [`core::ptr::drop_in_place`] apply, with the additional requirement
    /// that this method is being called on a pointer to an object of the correct component type.
    pub unsafe fn drop(&self, data: *mut u8) {
        unsafe { (self.drop)(data) }
    }

    /// Get the function pointer encoding the destructor for the component type this `TypeInfo`
    /// represents.
    pub fn drop_shim(&self) -> unsafe fn(*mut u8) {
        self.drop
    }
}

pub struct Common<M> {
    inner: CommonInner<M>,
    indices: TypeIdMap<usize>,
}

pub struct CommonInner<M> {
    storage: NonNull<u8>,
    layout: Layout,
    cursor: usize,
    info: Vec<(TypeInfo, usize, M)>,
    ids: Vec<TypeId>,
}

#[allow(unused)] // I might use it later
impl<M> Common<M> {
    fn has<T: Component>(&self) -> bool {
        self.indices.contains_key(&TypeId::of::<T>())
    }

    fn get_by_tid<'a, T>(&'a self, tid: &TypeId) -> Option<T> {
        let index = self.indices.get(tid)?;
        let (_, offset, _) = self.inner.info[*index];
        unsafe {
            let storage = self.inner.storage.as_ptr().add(offset).cast::<T>();
            // Some(T::from_raw(storage))
            Some(todo!())
        }
    }

    // fn get<'a, T: ComponentRefShared<'a>>(&'a self) -> Option<T> {
    //     let index = self.indices.get(&TypeId::of::<T::Component>())?;
    //     let (_, offset, _) = self.info[*index];
    //     unsafe {
    //         let storage = self.storage.as_ptr().add(offset).cast::<T::Component>();
    //         Some(T::from_raw(storage))
    //     }
    // }

    // fn get_mut<'a, T: ComponentRef<'a>>(&'a self) -> Option<T> {
    //     let index = self.indices.get(&TypeId::of::<T::Component>())?;
    //     let (_, offset, _) = self.info[*index];
    //     unsafe {
    //         let storage = self.storage.as_ptr().add(offset).cast::<T::Component>();
    //         Some(T::from_raw(storage))
    //     }
    // }

    fn component_types(&self) -> impl Iterator<Item = ComponentTypeId> + '_ {
        self.inner.info.iter().map(|(info, _, _)| info.id())
    }

    fn clear(&mut self) {
        self.inner.ids.clear();
        self.indices.clear();
        self.inner.cursor = 0;
        // NOTE we do not clone stuff and use everything, thus we do not need to drop things pointed by structures in storage
        // unsafe {
        //     for (ty, offset, _) in self.info.drain(..) {
        //         ty.drop(self.storage.as_ptr().add(offset));
        //     }
        // }
    }

    unsafe fn add(&mut self, ptr: *mut u8, ty: TypeInfo, meta: M) {
        use std::collections::hash_map::Entry;
        match self.indices.entry(ty.id().type_id()) {
            Entry::Occupied(occupied) => {
                let index = *occupied.get();
                let (ty, offset, _) = self.inner.info[index];
                unsafe {
                    let storage = self.inner.storage.as_ptr().add(offset);

                    // Drop the existing value
                    ty.drop(storage);

                    // Overwrite the old value with our new one.
                    std::ptr::copy_nonoverlapping(ptr, storage, ty.layout().size());
                }
            }
            Entry::Vacant(vacant) => {
                unsafe { self.inner.fun_name(ty, ptr, vacant, meta) };
            }
        }
    }
}

impl<M> CommonInner<M> {
    unsafe fn grow(
        min_size: usize,
        cursor: usize,
        align: usize,
        storage: NonNull<u8>,
    ) -> (NonNull<u8>, Layout) {
        let layout = Layout::from_size_align(min_size.next_power_of_two().max(64), align).unwrap();
        let new_storage = unsafe { NonNull::new_unchecked(alloc(layout)) };
        unsafe { std::ptr::copy_nonoverlapping(storage.as_ptr(), new_storage.as_ptr(), cursor) };
        (new_storage, layout)
    }

    unsafe fn fun_name(
        &mut self,
        ty: TypeInfo,
        ptr: *mut u8,
        vacant: std::collections::hash_map::VacantEntry<'_, TypeId, usize>,
        meta: M,
    ) {
        let offset = align(self.cursor, ty.layout().align());
        let end = offset + ty.layout().size();
        if end > self.layout.size() || ty.layout().align() > self.layout.align() {
            let new_align = self.layout.align().max(ty.layout().align());
            let (new_storage, new_layout) =
                unsafe { Self::grow(end, self.cursor, new_align, self.storage) };
            if self.layout.size() != 0 {
                unsafe { dealloc(self.storage.as_ptr(), self.layout) };
            }
            self.storage = new_storage;
            self.layout = new_layout;
        }

        if ty.id().type_id() == TypeId::of::<(Vec<usize>,)>() {
            let aaa = ptr as *mut (Vec<usize>,);
            dbg!(unsafe { aaa.as_ref() });
            // let v = unsafe { Vec::<usize>::from_raw_parts(ptr as *mut usize, 4, 4) };
            // dbg!(&v);
            // std::mem::forget(v);
        }

        if ty.id().type_id() == TypeId::of::<(Box<[u32]>,)>() {
            let aaa = ptr as *mut (Box<[u32]>,);
            dbg!(unsafe { aaa.as_ref() });
            // let v = unsafe { Vec::<usize>::from_raw_parts(ptr as *mut usize, 4, 4) };
            // dbg!(&v);
            // std::mem::forget(v);
        }

        let addr = unsafe { self.storage.as_ptr().add(offset) };
        unsafe { std::ptr::copy_nonoverlapping(ptr, addr, ty.layout().size()) };

        vacant.insert(self.info.len());
        self.info.push((ty, offset, meta));
        self.cursor = end;

        if ty.id().type_id() == TypeId::of::<(Box<[u32]>,)>() {
            let aaa = ptr as *mut (Box<[u32]>,);
            dbg!(unsafe { aaa.as_ref() });
            // let v = unsafe { Vec::<usize>::from_raw_parts(ptr as *mut usize, 4, 4) };
            // dbg!(&v);
            // std::mem::forget(v);
        }
    }
}
fn align(x: usize, alignment: usize) -> usize {
    debug_assert!(alignment.is_power_of_two());
    (x + alignment - 1) & (!alignment + 1)
}
unsafe impl<M> Send for Common<M> {}
unsafe impl<M> Sync for Common<M> {}

impl<M> Drop for Common<M> {
    fn drop(&mut self) {
        // Ensure buffered components aren't leaked
        self.clear();
        if self.inner.layout.size() != 0 {
            unsafe {
                dealloc(self.inner.storage.as_ptr(), self.inner.layout);
            }
        }
    }
}

impl<M> Default for Common<M> {
    /// Create a builder representing an entity with no components
    fn default() -> Self {
        Self {
            inner: CommonInner {
                storage: NonNull::dangling(),
                layout: Layout::from_size_align(0, 8).unwrap(),
                cursor: 0,
                info: Vec::new(),
                ids: Vec::new(),
            },
            indices: Default::default(),
        }
    }
}

#[test]
fn example() {
    let mut world = legion::World::new(Default::default());
    let mut components = EntityBuilder::new();
    components._add(42i32);
    components._add(true);
    components._add(vec![0, 1, 2, 3]);
    components._add("hello");
    components._add(0u64);
    let components = components.build();
    let entity = world.extend(components)[0];
    assert_eq!(Ok(&42), world.entry(entity).unwrap().get_component::<i32>());
    assert_eq!(
        Ok(&vec![0, 1, 2, 3]),
        world.entry(entity).unwrap().get_component::<Vec<i32>>()
    );
}

#[test]
fn simple() {
    let mut world = legion::World::new(Default::default());
    let mut components = EntityBuilder::new();
    let mut comp0: (Box<[u32]>,) = (vec![0, 0, 0, 0, 0, 1, 4100177920].into_boxed_slice(),); //0, 14, 43, 10, 876, 7, 1065, 35
    let mut comp0_saved = comp0.clone();
    let comp0_ptr = (&mut comp0) as *mut (Box<[u32]>,);
    components._add(comp0);
    unsafe { (*comp0_ptr).0[4] = 42 };
    comp0_saved.0[4] = 42;
    let comp1: i32 = 0;
    components._add(comp1);
    let comp2: bool = true;
    components._add(comp2);
    let mut comp3: Vec<u64> = vec![0, 1, 2, 3];
    let comp3_saved = comp3.clone();
    let comp3_ptr = (&mut comp3) as *mut Vec<u64>;
    components._add(comp3);
    let comp4: String = "ewgwgwsegwesf".into();
    components._add(comp4.clone());
    let comp5: u64 = 0;
    components._add(comp5);
    let components = components.build();
    dbg!(unsafe { comp0_ptr.as_ref() });
    let entity = world.extend(components)[0];
    assert_eq!(
        Some(&comp0_saved),
        unsafe { comp0_ptr.as_ref() },
        "slice should not have changed"
    );
    assert_eq!(
        Some(&comp3_saved),
        unsafe { comp3_ptr.as_ref() },
        "vec should not have changed"
    );

    if let Some(entry) = world.entry(entity) {
        unsafe { (*comp0_ptr).0[5] += 1 };
        comp0_saved.0[5] += 1;
        dbg!(unsafe { comp0_ptr.as_ref() });
        assert_eq!(Ok(&comp0_saved), entry.get_component::<(Box<[u32]>,)>());
        assert_eq!(Ok(&comp1), entry.get_component::<i32>());
        assert_eq!(Ok(&comp2), entry.get_component::<bool>());
        assert_eq!(Ok(&comp3_saved), entry.get_component::<Vec<u64>>());
        assert_eq!(Ok(&comp4), entry.get_component::<String>());
    }
}
