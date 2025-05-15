use std::ops::Deref;

use bevy_ecs::component::Component;
use hashbrown::hash_map::DefaultHashBuilder;
use num::ToPrimitive as _;

use crate::{
    store::{
        defaults::LabelIdentifier,
        nodes::{self, compo},
    },
    utils::make_hash,
};

#[derive(Default)]
pub struct NodeStore {
    dedup: hashbrown::HashMap<NodeIdentifier, (), ()>,
    #[doc(hidden)]
    pub inner: NodeStoreInner,
}

#[derive(Default)]
pub struct NodeStoreInner<W = bevy_ecs::world::World, S = nodes::NodeStoreStats> {
    stats: S,
    internal: W,
    hasher: DefaultHashBuilder, // TODO try https://github.com/ogxd/gxhash
}

impl crate::types::NStore for NodeStoreInner {
    type IdN = NodeIdentifier;
    type Idx = u16;
}

impl<'a> crate::types::lending::NLending<'a, NodeIdentifier> for NodeStoreInner {
    type N = HashedNodeRef<'a, NodeIdentifier>;
}

impl crate::types::lending::NodeStore<NodeIdentifier> for NodeStoreInner {
    fn resolve(
        &self,
        id: &NodeIdentifier,
    ) -> <Self as crate::types::lending::NLending<'_, NodeIdentifier>>::N {
        self.internal
            .get_entity(id.clone())
            .map(|x| HashedNodeRef::new(x))
            .unwrap()
    }
}

impl crate::types::NStore for &NodeStoreInner {
    type IdN = NodeIdentifier;
    type Idx = u16;
}

impl<'a> crate::types::lending::NLending<'a, NodeIdentifier> for &NodeStoreInner {
    type N = HashedNodeRef<'a, NodeIdentifier>;
}

impl crate::types::lending::NodeStore<NodeIdentifier> for &NodeStoreInner {
    fn resolve(
        &self,
        id: &NodeIdentifier,
    ) -> <Self as crate::types::lending::NLending<'_, NodeIdentifier>>::N {
        self.internal
            .get_entity(id.clone())
            .map(|x| HashedNodeRef::new(x))
            .unwrap()
    }
}

impl NodeStoreInner {
    pub fn len(&self) -> usize {
        self.internal.entities().len() as usize
    }
    pub fn stats(&mut self) -> &mut nodes::NodeStoreStats {
        &mut self.stats
    }
}

// ------ NodeStore Impls

impl crate::types::NStore for NodeStore {
    type IdN = NodeIdentifier;
    type Idx = u16;
}

impl<'a> crate::types::lending::NLending<'a, NodeIdentifier> for NodeStore {
    type N = HashedNodeRef<'a, NodeIdentifier>;
}

impl crate::types::lending::NodeStore<NodeIdentifier> for NodeStore {
    fn resolve(
        &self,
        id: &NodeIdentifier,
    ) -> <Self as crate::types::lending::NLending<'_, NodeIdentifier>>::N {
        self.inner.resolve(id)
    }
}

impl crate::types::NStoreRefAssoc for NodeStore {
    type S = NodeStoreInner;
}

// ------ Construction supports

impl NodeStore {
    pub fn prepare_insertion<'a, Eq: Fn(EntryRef) -> bool, V: std::hash::Hash>(
        &'a mut self,
        hashable: &V,
        eq: Eq,
    ) -> PendingInsert<'a> {
        let Self {
            dedup,
            inner: NodeStoreInner {
                internal: backend, ..
            },
        } = self;
        let hash = make_hash(&self.inner.hasher, hashable);
        let entry = dedup.raw_entry_mut().from_hash(hash, |symbol| {
            let r = eq(backend.get_entity(*symbol).unwrap());
            r
        });
        PendingInsert(entry, (hash, &mut self.inner))
    }

    pub fn insert_after_prepare<T>(
        (vacant, (hash, inner)): (
            crate::compat::hash_map::RawVacantEntryMut<bevy_ecs::entity::Entity, (), ()>,
            (u64, &mut NodeStoreInner),
        ),
        components: T,
    ) -> bevy_ecs::entity::Entity
    where
        T: bevy_ecs::bundle::Bundle,
    {
        let (&mut symbol, _) = {
            let symbol = inner.internal.spawn(components).id();
            vacant.insert_with_hasher(hash, symbol, (), |id| {
                let node: HashedNodeRef<'_, NodeIdentifier> = inner
                    .internal
                    .get_entity(*id)
                    .map(|x| HashedNodeRef::new(x))
                    .unwrap();

                make_hash(&inner.hasher, &node)
            })
        };
        symbol
    }

    /// uses the dyn builder see dyn_builder::EntityBuilder
    pub fn insert_built_after_prepare(
        (vacant, (hash, inner)): (
            crate::compat::hash_map::RawVacantEntryMut<bevy_ecs::entity::Entity, (), ()>,
            (u64, &mut NodeStoreInner),
        ),
        components: super::dyn_builder::BuiltEntity,
    ) -> bevy_ecs::entity::Entity {
        let (&mut symbol, _) = {
            let symbol = inner
                .internal
                .spawn({
                    todo!("need to find how to insert{:?}", components);
                    ()
                })
                .id();
            vacant.insert_with_hasher(hash, symbol, (), |id| {
                let node: HashedNodeRef<'_, NodeIdentifier> = inner
                    .internal
                    .get_entity(*id)
                    .map(|x| HashedNodeRef::new(x))
                    .unwrap();

                make_hash(&inner.hasher, &node)
            })
        };
        symbol
    }
}

pub struct PendingInsert<'a>(
    crate::compat::hash_map::RawEntryMut<'a, bevy_ecs::entity::Entity, (), ()>,
    (u64, &'a mut NodeStoreInner),
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
            .internal
            .get_entity(id)
            .map(|x| HashedNodeRef::new(x))
            .unwrap()
    }

    pub fn occupied(&'a self) -> Option<(NodeIdentifier, (u64, &'a NodeStoreInner))> {
        match &self.0 {
            hashbrown::hash_map::RawEntryMut::Occupied(occupied) => {
                Some((occupied.key().clone(), (self.1.0, self.1.1)))
            }
            _ => None,
        }
    }

    pub fn vacant(
        self,
    ) -> (
        crate::compat::hash_map::RawVacantEntryMut<'a, bevy_ecs::entity::Entity, (), ()>,
        (u64, &'a mut NodeStoreInner),
    ) {
        match self.0 {
            hashbrown::hash_map::RawEntryMut::Vacant(occupied) => (occupied, self.1),
            _ => panic!(),
        }
    }
}

impl<'a> PendingInsert<'a> {
    pub fn vacant2(self) -> DynBuilder<'a> {
        match self.0 {
            hashbrown::hash_map::RawEntryMut::Vacant(vac) => DynBuilder(
                vac,
                (
                    self.1.0,
                    NodeStoreInner {
                        internal: self.1.1.internal.spawn_empty(),
                        stats: &mut self.1.1.stats,
                        hasher: self.1.1.hasher.clone(),
                    },
                ),
            ),
            _ => panic!(),
        }
    }
    // vacant3 where we directly insert into the map
    // TODO bench against vacant2
    pub fn vacant3(self) -> AltInner<'a> {
        match self.0 {
            hashbrown::hash_map::RawEntryMut::Vacant(vacant) => {
                let internal = self.1.1.internal.spawn_empty();
                let hash = self.1.0;
                let symbol = internal.id();
                vacant.insert_with_hasher(hash, symbol, (), |id| {
                    let node: HashedNodeRef<'_, NodeIdentifier> = internal
                        .world()
                        .get_entity(*id)
                        .map(|x| HashedNodeRef::new(x))
                        .unwrap();

                    make_hash(&self.1.1.hasher, &node)
                });
                NodeStoreInner {
                    internal,
                    stats: &mut self.1.1.stats,
                    hasher: self.1.1.hasher.clone(),
                }
            }
            _ => panic!(),
        }
    }
    pub fn vacant4(self) -> bevy_ecs::world::EntityWorldMut<'a> {
        self.vacant3().internal
    }
}

pub struct DynBuilder<'a>(
    crate::compat::hash_map::RawVacantEntryMut<'a, bevy_ecs::entity::Entity, (), ()>,
    (u64, AltInner<'a>),
);

impl NodeStore {
    // /// uses the dyn builder see dyn_builder::EntityBuilder
    // pub fn insert_built_after_prepare2(
    //     (vacant, (hash, inner)): (
    //         crate::compat::hash_map::RawVacantEntryMut<bevy_ecs::entity::Entity, (), ()>,
    //         (u64, &mut AltInner<'_>),
    //     ),
    //     components: super::dyn_builder::BuiltEntity,
    // ) -> bevy_ecs::entity::Entity {
    //     let (&mut symbol, _) = {
    //         let symbol = inner
    //             .internal
    //             .spawn({
    //                 todo!("need to find how to insert{:?}", components);
    //                 ()
    //             })
    //             .id();
    //         vacant.insert_with_hasher(hash, symbol, (), |id| {
    //             let node: HashedNodeRef<'_, NodeIdentifier> = inner
    //                 .internal
    //                 .get_entity(*id)
    //                 .map(|x| HashedNodeRef::new(x))
    //                 .unwrap();

    //             make_hash(&inner.hasher, &node)
    //         })
    //     };
    //     symbol
    // }
}

type AltInner<'a> =
    NodeStoreInner<bevy_ecs::world::EntityWorldMut<'a>, &'a mut nodes::NodeStoreStats>;

impl<'a> AltInner<'a> {
    pub fn world(&mut self) -> &mut bevy_ecs::world::EntityWorldMut<'a> {
        &mut self.internal
    }
}

pub struct PendingInsert2<'a>(
    crate::compat::hash_map::RawEntryMut<'a, bevy_ecs::entity::Entity, (), ()>,
    (u64, &'a mut AltInner<'a>),
);

impl<'a> PendingInsert2<'a> {
    pub fn occupied_id(&self) -> Option<NodeIdentifier> {
        match &self.0 {
            hashbrown::hash_map::RawEntryMut::Occupied(occupied) => Some(occupied.key().clone()),
            _ => None,
        }
    }

    pub fn resolve<T>(&self, id: NodeIdentifier) -> HashedNodeRef<T> {
        self.1
            .1
            .internal
            .world()
            .get_entity(id)
            .map(|x| HashedNodeRef::new(x))
            .unwrap()
    }

    pub fn occupied(&'a self) -> Option<(NodeIdentifier, (u64, &'a AltInner<'a>))> {
        match &self.0 {
            hashbrown::hash_map::RawEntryMut::Occupied(occupied) => {
                Some((occupied.key().clone(), (self.1.0, self.1.1)))
            }
            _ => None,
        }
    }

    pub fn vacant(
        self,
    ) -> (
        crate::compat::hash_map::RawVacantEntryMut<'a, bevy_ecs::entity::Entity, (), ()>,
        (u64, &'a mut AltInner<'a>),
    ) {
        match self.0 {
            hashbrown::hash_map::RawEntryMut::Vacant(occupied) => (occupied, self.1),
            _ => panic!(),
        }
    }
}

#[inline]
pub fn eq_node<'a, K, L, I>(
    kind: &'a K,
    label_id: Option<&'a L>,
    children: &'a [I],
) -> impl Fn(EntryRef) -> bool + 'a
where
    K: Eq + Copy + Component,
    L: Eq + Copy + Component,
    compo::CS<I>: Component,
    [I]: Eq,
{
    move |x: EntryRef| {
        let t = x.get::<K>();
        if t != Some(kind) {
            return false;
        }
        let l = x.get::<L>();
        if l != label_id {
            return false;
        } else {
            use compo::CS; // FIXME not
            let cs = x.get::<CS<I>>();
            let r = match cs {
                Some(CS(cs)) => cs.as_ref() == children,
                None => children.is_empty(),
            };
            if !r {
                return false;
            }
        }
        true
    }
}

#[inline]
pub fn eq_space<'a, K, L>(kind: &'a K, label_id: &'a L) -> impl Fn(EntryRef) -> bool + 'a
where
    K: Eq + Copy + Component,
    L: Eq + Copy + Component,
{
    move |x: EntryRef| {
        let t = x.get::<K>();
        if t != Some(&kind) {
            return false;
        }
        let l = x.get::<L>();
        if l != Some(&label_id) {
            return false;
        }
        true
    }
}

// ------------dyn_builder---------

impl super::super::EntityBuilder for bevy_ecs::world::EntityWorldMut<'_> {
    fn add<T: nodes::Compo>(&mut self, component: T) -> &mut Self {
        self.insert(component)
    }
}

impl super::super::EntityBuilder for &mut bevy_ecs::world::EntityWorldMut<'_> {
    fn add<T: nodes::Compo>(&mut self, component: T) -> &mut Self {
        self.insert(component);
        self
    }
}

// ------------elem.rs------------

pub type NodeIdentifier = bevy_ecs::entity::Entity;

pub type EntryRef<'a> = bevy_ecs::world::EntityRef<'a>;

#[derive(ref_cast::RefCast)]
#[repr(transparent)]
pub struct HashedNodeRef<'a, T = NodeIdentifier>(
    pub(super) EntryRef<'a>,
    std::marker::PhantomData<T>,
);

impl crate::types::AAAA for NodeIdentifier {}

impl crate::types::NodeId for NodeIdentifier {
    type IdN = Self;
    fn as_id(&self) -> &Self::IdN {
        self
    }
    unsafe fn from_id(id: Self::IdN) -> Self {
        id
    }

    unsafe fn from_ref_id(id: &Self::IdN) -> &Self {
        id
    }
}

impl<'a, T> HashedNodeRef<'a, T> {
    pub fn new(entry: EntryRef<'a>) -> Self {
        HashedNodeRef(entry, std::marker::PhantomData)
    }
}

impl<Id> PartialEq for HashedNodeRef<'_, Id> {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl<Id> Eq for HashedNodeRef<'_, Id> {}

impl<Id> std::hash::Hash for HashedNodeRef<'_, Id> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use crate::hashed::SyntaxNodeHashsKinds;
        crate::types::WithHashs::hash(self, SyntaxNodeHashsKinds::default()).hash(state)
    }
}

impl<Id> std::fmt::Debug for HashedNodeRef<'_, Id> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("bevy::HashedNodeRef")
            .field(&self.0.location())
            .finish()
    }
}

impl<'a, T> Deref for HashedNodeRef<'a, T> {
    type Target = EntryRef<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> crate::types::Node for HashedNodeRef<'a, T> {}

impl<'a, T: crate::types::NodeId> crate::types::Stored for HashedNodeRef<'a, T> {
    type TreeId = T;
}

impl<'a, T: crate::types::NodeId> crate::types::CLending<'a, u16, T::IdN> for HashedNodeRef<'_, T> {
    type Children = crate::types::ChildrenSlice<'a, T::IdN>;
}

impl<'a, T: crate::types::NodeId<IdN = NodeIdentifier>> HashedNodeRef<'a, T> {
    pub fn cs(&self) -> Option<crate::types::LendC<'_, Self, u16, NodeIdentifier>> {
        self.0
            .get::<compo::CS<NodeIdentifier>>()
            .map(|x| (*x.0).into())
            .or_else(|| {
                self.0
                    .get::<compo::CS0<NodeIdentifier, 1>>()
                    .map(|x| (&x.0).into())
            })
            .or_else(|| {
                self.0
                    .get::<compo::CS0<NodeIdentifier, 2>>()
                    .map(|x| (&x.0).into())
            })
    }
}

impl<'a, T: crate::types::NodeId<IdN = NodeIdentifier>> crate::types::WithChildren
    for HashedNodeRef<'a, T>
{
    type ChildIdx = u16;

    fn child_count(&self) -> u16 {
        self.cs()
            .map_or(0, |x| {
                use crate::types::Children;
                let c: u16 = x.child_count();
                c
            })
            .to_u16()
            .expect("too much children")
    }

    fn child(&self, idx: &Self::ChildIdx) -> Option<NodeIdentifier> {
        self.cs()?.0.get(idx.to_usize().unwrap()).map(|x| *x)
    }

    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<NodeIdentifier> {
        let v = self.cs()?;
        use crate::types::Children;
        let c: Self::ChildIdx = v.child_count();
        let c = c.checked_sub(idx.checked_add(1)?)?;
        v.get(c).cloned()
    }

    fn children(
        &self,
    ) -> Option<
        crate::types::LendC<'_, Self, Self::ChildIdx, <Self::TreeId as crate::types::NodeId>::IdN>,
    > {
        self.cs()
    }
}

impl<'a, Id> crate::store::nodes::ErasedHolder for HashedNodeRef<'a, Id> {
    unsafe fn unerase_ref_unchecked<T: 'static + crate::store::nodes::Compo>(
        &self,
        tid: std::any::TypeId,
    ) -> Option<&T> {
        if tid == std::any::TypeId::of::<T>() {
            self.get()
        } else {
            None
        }
    }
    fn unerase_ref<T: 'static + Send + Sync>(&self, _tid: std::any::TypeId) -> Option<&T> {
        unreachable!()
    }
}

impl<'a, T> crate::types::Labeled for HashedNodeRef<'a, T> {
    type Label = LabelIdentifier;

    fn get_label_unchecked(&self) -> &LabelIdentifier {
        self.0
            .get::<LabelIdentifier>()
            .expect("check with self.has_label()")
    }

    fn try_get_label(&self) -> Option<&Self::Label> {
        self.0.get::<LabelIdentifier>()
    }
}

impl<'a, Id: crate::types::NodeId<IdN = NodeIdentifier>> crate::types::Tree
    for HashedNodeRef<'a, Id>
{
    fn has_children(&self) -> bool {
        todo!()
        // self.cs()
        //     .map(|x| !crate::types::Childrn::is_empty(&x))
        //     .unwrap_or(false)
    }

    fn has_label(&self) -> bool {
        self.0.get::<LabelIdentifier>().is_some()
    }
}

// ------------more.rs------------

impl<'a, T> crate::types::WithHashs for HashedNodeRef<'a, T> {
    type HK = crate::hashed::SyntaxNodeHashsKinds;
    type HP = crate::nodes::HashSize;

    fn hash(&self, kind: impl std::ops::Deref<Target = Self::HK>) -> Self::HP {
        use crate::hashed::SyntaxNodeHashs;
        let compo = self.0.get_ref::<SyntaxNodeHashs<Self::HP>>().unwrap();
        crate::hashed::NodeHashs::hash(&*compo, &kind)
    }
}
