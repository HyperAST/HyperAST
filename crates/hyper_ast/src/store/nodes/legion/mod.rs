use std::{fmt::Debug, hash::Hash, ops::Deref};

use hashbrown::hash_map::DefaultHashBuilder;
use legion::{
    storage::{Component, IntoComponentSource},
    EntityStore, World,
};

use crate::{
    types::{
        Compo, CompoRegister, ErasedHolder, ErasedInserter, NodeId, Typed, TypedNodeId,
        TypedNodeStore,
    },
    utils::make_hash,
};

pub mod dyn_builder;

pub mod compo;

mod elem;

pub use elem::{EntryRef, HashedNode, HashedNodeRef, NodeIdentifier};

pub struct NodeStore {
    dedup: hashbrown::HashMap<NodeIdentifier, (), ()>,
    #[doc(hidden)]
    pub inner: NodeStoreInner,
}

pub struct NodeStoreInner {
    count: usize,
    errors: usize,
    #[cfg(feature = "subtree-stats")]
    pub height_counts: Vec<u32>,
    #[cfg(feature = "subtree-stats")]
    pub height_counts_non_dedup: Vec<u32>,
    #[cfg(feature = "subtree-stats")]
    pub height_counts_structural: Vec<u32>,
    #[cfg(feature = "subtree-stats")]
    pub structurals: std::collections::HashSet<u32>,
    #[cfg(feature = "subtree-stats")]
    pub height_counts_label: Vec<u32>,
    #[cfg(feature = "subtree-stats")]
    pub labels: std::collections::HashSet<u32>,
    // roots: HashMap<(u8, u8, u8), NodeIdentifier>,
    // dedup: hashbrown::HashMap<NodeIdentifier, (), ()>,
    internal: legion::World,
    // TODO intern lists of [`NodeIdentifier`]s, e.g. children, no space children, ...
    hasher: DefaultHashBuilder, //fasthash::city::Hash64,//fasthash::RandomState<fasthash::>,
                                // internal: VecMapStore<HashedNode, NodeIdentifier, legion::World>,
}

// * Node store impl

pub struct PendingInsert<'a>(
    crate::compat::hash_map::RawEntryMut<'a, legion::Entity, (), ()>,
    (u64, &'a mut NodeStoreInner),
);

impl<'a> PendingInsert<'a> {
    pub fn occupied_id(&self) -> Option<NodeIdentifier> {
        match &self.0 {
            hashbrown::hash_map::RawEntryMut::Occupied(occupied) => Some(occupied.key().clone()),
            _ => None,
        }
    }
    pub fn stash(
        self,
    ) -> (
        crate::compat::hash_map::RawVacantEntryMut<'a, legion::Entity, (), ()>,
        (u64,),
    ) {
        let vacant = self.vacant();
        (vacant.0, (vacant.1 .0,))
    }
    pub fn resolve<T>(&self, id: NodeIdentifier) -> HashedNodeRef<T> {
        self.1
             .1
            .internal
            .entry_ref(id)
            .map(|x| HashedNodeRef::new(x))
            .unwrap()
    }
    pub fn occupied(&'a self) -> Option<(NodeIdentifier, (u64, &'a NodeStoreInner))> {
        match &self.0 {
            hashbrown::hash_map::RawEntryMut::Occupied(occupied) => {
                Some((occupied.key().clone(), (self.1 .0, self.1 .1)))
            }
            _ => None,
        }
    }

    pub fn vacant(
        self,
    ) -> (
        crate::compat::hash_map::RawVacantEntryMut<'a, legion::Entity, (), ()>,
        (u64, &'a mut NodeStoreInner),
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
            let r = eq(backend.entry_ref(*symbol).unwrap());
            r
        });
        PendingInsert(entry, (hash, &mut self.inner))
    }

    pub fn insert_after_prepare<T>(
        (vacant, (hash, inner)): (
            crate::compat::hash_map::RawVacantEntryMut<legion::Entity, (), ()>,
            (u64, &mut NodeStoreInner),
        ),
        components: T,
    ) -> legion::Entity
    where
        Option<T>: IntoComponentSource,
    {
        let (&mut symbol, _) = {
            let symbol = inner.internal.push(components);
            vacant.insert_with_hasher(hash, symbol, (), |id| {
                let node: elem::HashedNodeRef<'_, NodeIdentifier> = inner
                    .internal
                    .entry_ref(*id)
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
            crate::compat::hash_map::RawVacantEntryMut<legion::Entity, (), ()>,
            (u64, &mut NodeStoreInner),
        ),
        components: dyn_builder::BuiltEntity,
    ) -> legion::Entity {
        let (&mut symbol, _) = {
            let symbol = inner.internal.extend(components)[0];
            vacant.insert_with_hasher(hash, symbol, (), |id| {
                let node: elem::HashedNodeRef<'_, NodeIdentifier> = inner
                    .internal
                    .entry_ref(*id)
                    .map(|x| HashedNodeRef::new(x))
                    .unwrap();

                make_hash(&inner.hasher, &node)
            })
        };
        symbol
    }

    pub fn resolve(&self, id: NodeIdentifier) -> HashedNodeRef<NodeIdentifier> {
        self.inner
            .internal
            .entry_ref(id)
            .map(|x| HashedNodeRef::new(x))
            .unwrap()
    }

    pub unsafe fn _resolve<T>(&self, id: &NodeIdentifier) -> HashedNodeRef<T> {
        self.inner
            .internal
            .entry_ref(*id)
            .map(|x| HashedNodeRef::new(x))
            .unwrap()
    }

    pub fn resolve_with_type<T: 'static + TypedNodeId<IdN = NodeIdentifier>>(
        &self,
        id: &T::IdN,
    ) -> (T::Ty, HashedNodeRef<T>) {
        let n = self
            .inner
            .internal
            .entry_ref(*id)
            .map(|x| HashedNodeRef::new(x))
            .unwrap();
        (n.get_type(), n)
    }

    pub fn try_resolve(&self, id: NodeIdentifier) -> Option<HashedNodeRef<NodeIdentifier>> {
        self.inner
            .internal
            .entry_ref(id)
            .map(|x| HashedNodeRef::new(x))
            .ok()
    }

    pub fn resolve_typed<TIdN: 'static + TypedNodeId<IdN = NodeIdentifier>>(
        &self,
        id: &TIdN,
    ) -> HashedNodeRef<TIdN> {
        let x = self.inner.internal.entry_ref(id.as_id().clone()).unwrap();
        HashedNodeRef::new(x)
    }

    pub fn try_resolve_typed<TIdN: 'static + TypedNodeId<IdN = NodeIdentifier>>(
        &self,
        id: &TIdN::IdN,
    ) -> Option<(HashedNodeRef<TIdN>, TIdN)> {
        let x = self.inner.internal.entry_ref(id.clone()).unwrap();
        x.get_component::<TIdN::Ty>().ok()?;
        Some((HashedNodeRef::new(x), unsafe { TIdN::from_id(id.clone()) }))
    }

    pub fn try_resolve_typed2<
        L: crate::types::LLang<crate::types::TypeU16<L>, I = u16> + 'static,
    >(
        &self,
        id: &NodeIdentifier,
    ) -> Option<(HashedNodeRef<NodeIdentifier>, L::E)> {
        let x = self.inner.internal.entry_ref(id.clone()).unwrap();
        let ty = x.get_component::<crate::types::TypeU16<L>>().ok()?;
        let ty = ty.e();
        Some((HashedNodeRef::new(x), ty))
    }

    pub fn try_resolve_typed3<
        L: crate::types::LLang<crate::types::TypeU16<L>, I = u16> + 'static,
    >(
        &self,
        id: &NodeIdentifier,
    ) -> Option<TypedNode<HashedNodeRef<NodeIdentifier>, L::E>> {
        let x = self.inner.internal.entry_ref(id.clone()).unwrap();
        let ty = x.get_component::<crate::types::TypeU16<L>>().ok()?;
        let ty = ty.e();
        Some(TypedNode(HashedNodeRef::new(x), ty))
    }
}

pub struct TypedNode<T, Ty>(T, Ty);

impl<T, Ty: Copy> TypedNode<T, Ty> {
    pub fn get_type(&self) -> Ty {
        self.1
    }
    pub fn into(self) -> T {
        self.0
    }
}
impl<T, Ty> Deref for TypedNode<T, Ty> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for NodeStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = f.debug_struct("NodeStore");
        r.field("count", &self.inner.count)
            .field("errors", &self.inner.errors)
            .field("internal_len", &self.inner.internal.len());
        // .field("internal", &self.internal)

        fn lim<T>(v: &[T]) -> &[T] {
            &v[..v.len().min(30)]
        }
        #[cfg(feature = "subtree-stats")]
        r.field("height_counts", &lim(&self.inner.height_counts_structural));
        #[cfg(feature = "subtree-stats")]
        r.field("height_counts", &lim(&self.inner.height_counts_label));
        #[cfg(feature = "subtree-stats")]
        r.field("height_counts", &lim(&self.inner.height_counts));
        #[cfg(feature = "subtree-stats")]
        r.field(
            "height_counts_non_dedup",
            &lim(&self.inner.height_counts_non_dedup),
        );

        r.finish()
    }
}

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
        crate::types::lending::NodeStore::resolve(&self.inner, id)
    }
}

impl crate::types::inner_ref::NodeStore<NodeIdentifier> for NodeStore {
    type Ref = HashedNodeRef<'static, NodeIdentifier>;
    fn scoped<R>(&self, id: &NodeIdentifier, f: impl Fn(&Self::Ref) -> R) -> R {
        self.inner.scoped(id, f)
    }
    fn scoped_mut<R>(&self, id: &NodeIdentifier, mut f: impl FnMut(&Self::Ref) -> R) -> R {
        self.inner.scoped_mut(id, f)
    }
    fn multi<R, const N: usize>(
        &self,
        id: &[NodeIdentifier; N],
        f: impl Fn(&[Self::Ref; N]) -> R,
    ) -> R {
        self.inner.multi(id, f)
    }
}

impl crate::types::inner_ref::NodeStore<NodeIdentifier> for &NodeStoreInner {
    type Ref = HashedNodeRef<'static, NodeIdentifier>;
    fn scoped<R>(&self, id: &NodeIdentifier, f: impl Fn(&Self::Ref) -> R) -> R {
        (*self).scoped(id, f)
    }
    fn scoped_mut<R>(&self, id: &NodeIdentifier, mut f: impl FnMut(&Self::Ref) -> R) -> R {
        (*self).scoped_mut(id, f)
    }
    fn multi<R, const N: usize>(
        &self,
        id: &[NodeIdentifier; N],
        f: impl Fn(&[Self::Ref; N]) -> R,
    ) -> R {
        (*self).multi(id, f)
    }
}

impl crate::types::inner_ref::NodeStore<NodeIdentifier> for NodeStoreInner {
    type Ref = HashedNodeRef<'static, NodeIdentifier>;
    fn scoped<R>(&self, id: &NodeIdentifier, f: impl Fn(&Self::Ref) -> R) -> R {
        let t = &self.resolve(id);
        // SAFETY: safe as long as Self::Ref does not exposes its fake &'static fields
        let t = unsafe { std::mem::transmute(t) };
        f(t)
    }
    fn scoped_mut<R>(&self, id: &NodeIdentifier, mut f: impl FnMut(&Self::Ref) -> R) -> R {
        let t = &self.resolve(id);
        // SAFETY: safe as long as Self::Ref does not exposes its fake &'static fields
        let t = unsafe { std::mem::transmute(t) };
        f(t)
    }
    fn multi<R, const N: usize>(
        &self,
        id: &[NodeIdentifier; N],
        f: impl Fn(&[Self::Ref; N]) -> R,
    ) -> R {
        todo!()
    }
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
            .entry_ref(id.clone())
            .map(|x| HashedNodeRef::new(x))
            .unwrap()
    }
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
            .entry_ref(id.clone())
            .map(|x| HashedNodeRef::new(x))
            .unwrap()
    }
}

// impl crate::types::NodStore<NodeIdentifier> for NodeStoreInner {
//     type R<'a> = HashedNodeRef<'a, NodeIdentifier>;
// }

// impl crate::types::NodeStore<NodeIdentifier> for NodeStoreInner {
//     fn resolve(&self, id: &NodeIdentifier) -> Self::R<'_> {
//         self.internal
//             .entry_ref(id.clone())
//             .map(|x| HashedNodeRef::new(x))
//             .unwrap()
//     }
// }

// impl crate::types::NodStore<NodeIdentifier> for &NodeStoreInner {
//     type R<'a> = HashedNodeRef<'a, NodeIdentifier>;
// }

// impl crate::types::NodeStore<NodeIdentifier> for &NodeStoreInner {
//     fn resolve(&self, id: &NodeIdentifier) -> Self::R<'_> {
//         self.internal
//             .entry_ref(id.clone())
//             .map(|x| HashedNodeRef::new(x))
//             .unwrap()
//     }
// }

pub fn _resolve<'a, T>(
    slf: &'a legion::World,
    id: &NodeIdentifier,
) -> Result<HashedNodeRef<'a, T>, legion::world::EntityAccessError> {
    slf.entry_ref(*id).map(|x| HashedNodeRef::new(x))
}

impl<'a> crate::types::NStore for &'a NodeStoreInner {
    type IdN = NodeIdentifier;

    type Idx = u16;
}

impl<'a> crate::types::NodeStoreLean<NodeIdentifier> for &'a NodeStoreInner {
    type R = HashedNodeRef<'a, NodeIdentifier>;
    fn resolve(&self, id: &NodeIdentifier) -> Self::R {
        self.internal
            .entry_ref(id.clone())
            .map(|x| HashedNodeRef::new(x))
            .unwrap()
    }
}

impl<'a> crate::types::NodeStoreLean<NodeIdentifier> for &'a NodeStore {
    type R = HashedNodeRef<'a, NodeIdentifier>;
    fn resolve(&self, id: &NodeIdentifier) -> Self::R {
        self.inner.resolve(id)
    }
}

impl<TIdN: 'static + TypedNodeId<IdN = NodeIdentifier>> crate::types::TyNodeStore<TIdN>
    for NodeStoreInner
{
    type R<'a> = HashedNodeRef<'a, TIdN>;
}

impl<TIdN: 'static + TypedNodeId<IdN = NodeIdentifier>> crate::types::TypedNodeStore<TIdN>
    for NodeStoreInner
{
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

impl<TIdN: 'static + TypedNodeId<IdN = NodeIdentifier>> crate::types::TyNodeStore<TIdN>
    for NodeStore
{
    type R<'a> = HashedNodeRef<'a, TIdN>;
}

impl<TIdN: 'static + TypedNodeId<IdN = NodeIdentifier>> crate::types::TypedNodeStore<TIdN>
    for NodeStore
{
    fn resolve(&self, id: &TIdN) -> Self::R<'_> {
        self.inner.resolve(id)
    }

    fn try_typed(&self, id: &<TIdN as NodeId>::IdN) -> Option<TIdN> {
        self.inner.try_typed(id)
    }
}

impl NodeStoreInner {
    pub fn len(&self) -> usize {
        self.internal.len()
    }

    #[cfg(feature = "subtree-stats")]
    pub fn add_height_non_dedup(&mut self, height: u32) {
        accumulate_height(&mut self.height_counts_non_dedup, height);
    }

    #[cfg(feature = "subtree-stats")]
    pub fn add_height_dedup(&mut self, height: u32, hashs: crate::hashed::SyntaxNodeHashs<u32>) {
        self.add_height(height);
        self.add_height_label(height, hashs.label);
        self.add_height_structural(height, hashs.structt);
    }

    #[cfg(feature = "subtree-stats")]
    fn add_height(&mut self, height: u32) {
        accumulate_height(&mut self.height_counts, height);
    }

    #[cfg(feature = "subtree-stats")]
    fn add_height_structural(&mut self, height: u32, hash: u32) {
        if not_there(&mut self.structurals, hash) {
            accumulate_height(&mut self.height_counts_structural, height);
        }
    }

    #[cfg(feature = "subtree-stats")]
    fn add_height_label(&mut self, height: u32, hash: u32) {
        if not_there(&mut self.labels, hash) {
            accumulate_height(&mut self.height_counts_label, height);
        }
    }
}

fn not_there(hash_set: &mut std::collections::HashSet<u32>, hash: u32) -> bool {
    if hash_set.contains(&hash) {
        return false;
    }
    hash_set.insert(hash);
    true
}

fn accumulate_height(counts: &mut Vec<u32>, height: u32) {
    if counts.len() <= height as usize {
        counts.resize(height as usize + 1, 0);
    }
    counts[height as usize] += 1;
}

impl NodeStore {
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl NodeStore {
    pub fn new() -> Self {
        Self {
            inner: NodeStoreInner {
                count: 0,
                errors: 0,
                #[cfg(feature = "subtree-stats")]
                height_counts: Vec::with_capacity(100),
                #[cfg(feature = "subtree-stats")]
                height_counts_non_dedup: Vec::with_capacity(100),
                #[cfg(feature = "subtree-stats")]
                height_counts_structural: Vec::with_capacity(100),
                #[cfg(feature = "subtree-stats")]
                structurals: std::collections::HashSet::with_capacity(100),
                #[cfg(feature = "subtree-stats")]
                height_counts_label: Vec::with_capacity(100),
                #[cfg(feature = "subtree-stats")]
                labels: std::collections::HashSet::with_capacity(100),
                // roots: Default::default(),
                internal: Default::default(),
                hasher: Default::default(),
            },
            dedup: hashbrown::HashMap::<_, (), ()>::with_capacity_and_hasher(
                1 << 10,
                Default::default(),
            ),
        }
    }
}

impl Default for NodeStore {
    fn default() -> Self {
        Self::new()
    }
}



impl<'a, TS: Copy + crate::types::TypeStore> crate::types::StoreLending<'a>
    for crate::store::SimpleStores<TS>
{
    type S = crate::store::SimpleStores<
        TS,
        &'a crate::store::nodes::legion::NodeStoreInner,
        &'a crate::store::labels::LabelStore,
    >;
}

impl<TS: Copy + crate::types::TypeStore> crate::types::StoreLending2
    for crate::store::SimpleStores<TS>
{
    type S<'a> = crate::store::SimpleStores<
        TS,
        &'a crate::store::nodes::legion::NodeStoreInner,
        &'a crate::store::labels::LabelStore,
    >;
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

pub struct TMarker<IdN>(std::marker::PhantomData<IdN>);

// impl<IdN> Default for TMarker<IdN> {
//     fn default() -> Self {
//         Self(Default::default())
//     }
// }

// impl<'a, IdN: 'a + crate::types::NodeId> crate::types::NLending<'a, IdN> for &TMarker<IdN> {
//     type N = HashedNodeRef<'a, IdN>;
// }

// impl<'a, IdN: 'a + crate::types::NodeId> crate::types::NLending<'a, IdN> for TMarker<IdN> {
//     type N = HashedNodeRef<'a, IdN>;
// }

// impl<IdN: crate::types::NodeId> crate::types::HyperASTShared for &TMarker<IdN> {
//     type IdN = IdN;
//     type Idx = u16;
//     type Label = crate::store::defaults::LabelIdentifier;
// }

// impl<IdN: crate::types::NodeId> crate::types::HyperASTShared for TMarker<IdN> {
//     type IdN = IdN;
//     type Idx = u16;
//     type Label = crate::store::defaults::LabelIdentifier;
// }

// impl<'a, IdN: TypedNodeId<IdN = NodeIdentifier>> crate::types::AstLending<'a> for &TMarker<IdN> {
//     type RT = HashedNodeRef<'a, IdN>;
// }

// impl<'a, IdN: TypedNodeId<IdN = NodeIdentifier>> crate::types::AstLending<'a> for TMarker<IdN> {
//     type RT = HashedNodeRef<'a, IdN>;
// }

// impl<IdN> crate::types::Node for TMarker<IdN> {}

// impl<IdN: crate::types::NodeId> crate::types::Stored for TMarker<IdN> {
//     type TreeId = IdN;
// }
// impl<IdN: TypedNodeId<IdN = NodeIdentifier>> crate::types::MarkedT for TMarker<IdN>
// where
//     IdN::IdN: crate::types::NodeId<IdN = NodeIdentifier>,
// {
//     type Label = crate::store::defaults::LabelIdentifier;

//     type ChildIdx = u16;
// }

mod stores_impl {
    use crate::{
        store::{labels, nodes, SimpleStores},
        types::{HyperAST, HyperASTShared, TypeStore, TypeTrait, TypedHyperAST, TypedNodeId},
    };

    use super::{NodeIdentifier, NodeStoreInner};

    impl<TS, NS, LS> HyperASTShared for SimpleStores<TS, NS, LS>
    where
        NS: crate::types::NStore,
        NS: crate::types::NodeStore<<NS as crate::types::NStore>::IdN>,
        LS: crate::types::LStore,
        <NS as crate::types::NStore>::IdN:
            crate::types::NodeId<IdN = <NS as crate::types::NStore>::IdN>,
        // for<'t> <NS::NMarker as crate::types::NLending<'t, <NS as crate::types::NStore>::IdN>>::N:
        // for<'t> crate::types::LendN<'t, NS::NMarker, <NS as crate::types::NStore>::IdN>:
        //     crate::types::Tree<
        //         Label = <LS as crate::types::LStore>::I,
        //         TreeId = <NS as crate::types::NStore>::IdN,
        //         ChildIdx = <NS as crate::types::NStore>::Idx,
        //     >,
    {
        type IdN = NS::IdN;

        type Idx = NS::Idx;
        type Label = LS::I;
    }

    impl<'a, TS, NS, LS> crate::types::NLending<'a, <NS as crate::types::NStore>::IdN>
        for SimpleStores<TS, NS, LS>
    where
        NS: crate::types::NStore,
        NS: crate::types::NodeStore<<NS as crate::types::NStore>::IdN>,
        LS: crate::types::LStore,
        <NS as crate::types::NStore>::IdN:
            crate::types::NodeId<IdN = <NS as crate::types::NStore>::IdN>,
        // for<'t> <NS as crate::types::NLending<'t, <NS as crate::types::NStore>::IdN>>::N:
        //     crate::types::Tree<
        //         Label = <LS as crate::types::LStore>::I,
        //         TreeId = <NS as crate::types::NStore>::IdN,
        //         ChildIdx = <NS as crate::types::NStore>::Idx,
        //     >,
    {
        type N = <NS as crate::types::NLending<'a, <NS as crate::types::NStore>::IdN>>::N;
    }

    impl<'a, TS, NS, LS> crate::types::AstLending<'a> for SimpleStores<TS, NS, LS>
    where
        NS: crate::types::NStore,
        NS: crate::types::NodeStore<<NS as crate::types::NStore>::IdN>,
        LS: crate::types::LStore,
        <NS as crate::types::NStore>::IdN:
            crate::types::NodeId<IdN = <NS as crate::types::NStore>::IdN>,
        // for<'t> crate::types::LendN<'t, NS::NMarker, <NS as crate::types::NStore>::IdN>:
        for<'t> crate::types::LendN<'t, NS, <NS as crate::types::NStore>::IdN>: crate::types::Tree<
            Label = <LS as crate::types::LStore>::I,
            TreeId = <NS as crate::types::NStore>::IdN,
            ChildIdx = <NS as crate::types::NStore>::Idx,
        >,
    {
        type RT = <NS as crate::types::NLending<'a, Self::IdN>>::N;
    }

    impl<TS, NS, LS> HyperASTShared for &SimpleStores<TS, NS, LS>
    where
        // <NS as crate::types::NStore>::IdN: 'static,
        NS: crate::types::NStore,
        NS: crate::types::NodeStore<<NS as crate::types::NStore>::IdN>,
        LS: crate::types::LStore,
        <NS as crate::types::NStore>::IdN:
            crate::types::NodeId<IdN = <NS as crate::types::NStore>::IdN>,
        // for<'t> crate::types::LendN<'t, NS::NMarker, <NS as crate::types::NStore>::IdN>:
        //     crate::types::Tree<
        //         Label = <LS as crate::types::LStore>::I,
        //         TreeId = <NS as crate::types::NStore>::IdN,
        //         ChildIdx = <NS as crate::types::NStore>::Idx,
        //     >,
    {
        type IdN = NS::IdN;

        type Idx = NS::Idx;
        type Label = LS::I;
    }

    impl<'a, TS, NS, LS> crate::types::NLending<'a, <NS as crate::types::NStore>::IdN>
        for &SimpleStores<TS, NS, LS>
    where
        NS: crate::types::NStore,
        NS: crate::types::NodeStore<<NS as crate::types::NStore>::IdN>,
        LS: crate::types::LStore,
        <NS as crate::types::NStore>::IdN:
            crate::types::NodeId<IdN = <NS as crate::types::NStore>::IdN>,
        for<'t> crate::types::LendN<'t, NS, <NS as crate::types::NStore>::IdN>: crate::types::Tree<
            Label = <LS as crate::types::LStore>::I,
            TreeId = <NS as crate::types::NStore>::IdN,
            ChildIdx = <NS as crate::types::NStore>::Idx,
        >,
    {
        type N = <NS as crate::types::NLending<'a, <NS as crate::types::NStore>::IdN>>::N;
    }

    impl<'a, TS, NS, LS> crate::types::AstLending<'a> for &SimpleStores<TS, NS, LS>
    where
        NS: crate::types::NStore,
        NS: crate::types::NodeStore<<NS as crate::types::NStore>::IdN>,
        LS: crate::types::LStore,
        <NS as crate::types::NStore>::IdN:
            crate::types::NodeId<IdN = <NS as crate::types::NStore>::IdN>,
        // for<'t> crate::types::LendN<'t, NS::NMarker, <NS as crate::types::NStore>::IdN>:
        for<'t> crate::types::LendN<'t, NS, <NS as crate::types::NStore>::IdN>: crate::types::Tree<
            Label = <LS as crate::types::LStore>::I,
            TreeId = <NS as crate::types::NStore>::IdN,
            ChildIdx = <NS as crate::types::NStore>::Idx,
        >,
    {
        type RT = <NS as crate::types::NLending<'a, <NS as crate::types::NStore>::IdN>>::N;
    }

    // impl<'store, TS> HyperASTLean for &'store SimpleStores<TS, nodes::DefaultNodeStore>
    // where
    //     TS: TypeStore,
    // {
    //     type T = self::nodes::legion::HashedNodeRef<'store, Self::IdN>;

    //     type NS = nodes::legion::NodeStore;

    //     fn node_store(&self) -> &Self::NS {
    //         &self.node_store
    //     }

    //     type LS = labels::LabelStore;

    //     fn label_store(&self) -> &Self::LS {
    //         &self.label_store
    //     }

    //     type TS = TS;
    // }

    // impl<'store, TS> HyperASTAsso for &'store SimpleStores<TS, nodes::DefaultNodeStore>
    // where
    //     TS: for<'s> TypeStore,
    // {
    //     type T<'s>
    //         = self::nodes::legion::HashedNodeRef<'s, Self::IdN>
    //     where
    //         Self: 's;

    //     type NS<'s>
    //         = nodes::legion::NodeStore
    //     where
    //         Self: 's;

    //     fn node_store(&self) -> &Self::NS<'_> {
    //         &self.node_store
    //     }

    //     type LS = labels::LabelStore;

    //     fn label_store(&self) -> &Self::LS {
    //         &self.label_store
    //     }

    //     type TS<'s>
    //         = TS
    //     where
    //         Self: 's;
    // }

    // pub struct SStores<'n, TS, NS = nodes::DefaultNodeStore, LS = labels::LabelStore> {
    //     pub label_store: LS,
    //     pub node_store: NS,
    //     pub type_store: std::marker::PhantomData<(&'n (), TS)>,
    // }

    // impl<'s, TS, NS, LS> HyperASTShared for SStores<'s, TS, NS, LS>
    // where
    //     NS: crate::types::NStore,
    //     LS: crate::types::LStore,
    // {
    //     type IdN = NS::IdN;

    //     type Idx = NS::Idx;
    //     type Label = LS::I;
    // }

    // impl<'store, 's, TS, NS, LS> HyperAST<'store> for SStores<'s, TS, NS, LS>
    // where
    //     TS: TypeStore,
    //     NS: 's,
    //     NS: crate::types::NStore,
    //     NS: crate::types::NodeStore<NS::IdN>,
    //     LS: crate::types::LStore,
    //     LS: crate::types::LabelStore<str, I = <LS as crate::types::LStore>::I>,
    //     NS::R<'s>: crate::types::Tree<
    //         Label = <LS as crate::types::LStore>::I,
    //         TreeId = Self::IdN,
    //         ChildIdx = Self::Idx,
    //     >,
    // {
    //     type T = NS::R<'s>;

    //     type NS = NS;

    //     fn node_store(&self) -> &Self::NS {
    //         &self.node_store
    //     }

    //     type LS = LS;

    //     fn label_store(&self) -> &Self::LS {
    //         &self.label_store
    //     }

    //     type TS = TS;
    // }

    impl<'store, TS, NS, LS> HyperAST for SimpleStores<TS, NS, LS>
    where
        TS: TypeStore,
        NS: crate::types::NStore,
        <NS as crate::types::NStore>::IdN:
            crate::types::NodeId<IdN = <NS as crate::types::NStore>::IdN>,
        NS: crate::types::NodeStore<NS::IdN>,
        LS: crate::types::LStore,
        LS: crate::types::LabelStore<str, I = <LS as crate::types::LStore>::I>,
        // for<'t> <NS as crate::types::NodeStore<<NS as crate::types::NStore>::IdN>>::NMarker:
        //     crate::types::AstLending<
        //         't,
        //         IdN = <NS as crate::types::NStore>::IdN,
        //         Idx = <NS as crate::types::NStore>::Idx,
        //         Label = <LS as crate::types::LStore>::I,
        //     >,
        for<'t> crate::types::LendN<'t, NS, <NS as crate::types::NStore>::IdN>: crate::types::Tree<
            Label = <LS as crate::types::LStore>::I,
            TreeId = <NS as crate::types::NStore>::IdN,
            ChildIdx = <NS as crate::types::NStore>::Idx,
        >,
    {
        type NS = NS;

        fn node_store(&self) -> &Self::NS {
            &self.node_store
        }

        type LS = LS;

        fn label_store(&self) -> &Self::LS {
            &self.label_store
        }

        type TS = TS;
    }
    impl<'a, Ty, TS, NS, LS> crate::types::TypedLending<'a, Ty> for SimpleStores<TS, NS, LS>
    where
        NS: crate::types::NStore,
        NS: crate::types::NodeStore<<NS as crate::types::NStore>::IdN>,
        LS: crate::types::LStore,
        <NS as crate::types::NStore>::IdN:
            crate::types::NodeId<IdN = <NS as crate::types::NStore>::IdN>,
        for<'t> crate::types::LendN<'t, NS, <NS as crate::types::NStore>::IdN>: crate::types::Tree<
            Label = <LS as crate::types::LStore>::I,
            TreeId = <NS as crate::types::NStore>::IdN,
            ChildIdx = <NS as crate::types::NStore>::Idx,
        >,
        Ty: TypeTrait,
        //     TS: TypeStore,
        //     NS: crate::types::NStore,
        //     <NS as crate::types::NStore>::IdN:
        //         crate::types::NodeId<IdN = <NS as crate::types::NStore>::IdN>,
        //     NS: crate::types::NodeStore<NS::IdN>,
        //     LS: crate::types::LStore,
        //     LS: crate::types::LabelStore<str, I = <LS as crate::types::LStore>::I>,

        //     for<'t> <NS as crate::types::NodeStore<<NS as crate::types::NStore>::IdN>>::NMarker:
        //         crate::types::AstLending<
        //             't,
        //             IdN = <NS as crate::types::NStore>::IdN,
        //             Idx = <NS as crate::types::NStore>::Idx,
        //             Label = <LS as crate::types::LStore>::I,
        //         >,

        //     TIdN: 'static + TypedNodeId<IdN = Self::IdN>,
        //     NS: crate::types::TypedNodeStore<TIdN>,
    {
        type TT = TypedHolder<<Self as crate::types::AstLending<'a>>::RT, Ty>;
    }

    pub struct TypedHolder<RT, Ty> {
        pub rt: RT,
        _p: std::marker::PhantomData<Ty>,
    }
    impl<RT, Ty> std::ops::Deref for TypedHolder<RT, Ty> {
        type Target = RT;
        fn deref(&self) -> &Self::Target {
            &self.rt
        }
    }

    impl<RT, Ty: TypeTrait> crate::types::Typed for TypedHolder<RT, Ty> {
        type Type = Ty;
        fn get_type(&self) -> Self::Type {
            todo!()
        }
    }

    // impl<RT: crate::types::Tree, Ty> crate::types::ErasedHolder for TypedHolder<RT, Ty> {
    //     fn unerase_ref<TT: 'static + Send + Sync>(&self, tid: std::any::TypeId) -> Option<&TT> {
    //         todo!()
    //     }
    // }

    // impl<RT: crate::types::Tree, Ty> crate::types::WithChildren for TypedHolder<RT, Ty> {
    // }

    // impl<RT: crate::types::Tree, Ty> crate::types::Tree for TypedHolder<RT, Ty> {
    //     fn has_children(&self) -> bool {
    //         self.rt.has_children()
    //     }

    //     fn has_label(&self) -> bool {
    //         self.rt.has_label()
    //     }
    // }
    // impl<RT: crate::types::Stored, Ty> crate::types::Node for TypedHolder<RT, Ty> {}
    // impl<RT: crate::types::Stored, Ty> crate::types::Stored for TypedHolder<RT, Ty> {
    //     type TreeId = RT::TreeId;
    // }

    // impl<'store, TIdN, TS, NS, LS> TypedHyperAST<TIdN> for SimpleStores<TS, NS, LS>
    // where
    //     NS: crate::types::NStore,
    //     <NS as crate::types::NStore>::IdN:
    //         crate::types::NodeId<IdN = <NS as crate::types::NStore>::IdN>,
    //     NS: crate::types::NodeStore<NS::IdN>,
    //     // for<'t> NS::R<'t>: crate::types::Tree<
    //     //     Label = <LS as crate::types::LStore>::I,
    //     //     TreeId = <NS as crate::types::NStore>::IdN,
    //     //     ChildIdx = <NS as crate::types::NStore>::Idx,
    //     // >,
    //     // NS: for<'t> crate::types::NLending<'t, NS::IdN, N = <NS::NMarker as crate::types::AstLending<'t>>::RT>,
    //     // for<'t> <NS as crate::types::NLending<'t, <NS as crate::types::NStore>::IdN>>::N:
    //     //     crate::types::Tree<
    //     //         Label = <LS as crate::types::LStore>::I,
    //     //         TreeId = <NS as crate::types::NStore>::IdN,
    //     //         ChildIdx = <NS as crate::types::NStore>::Idx,
    //     //     >,
    //     // NS: crate::types::inner_ref::NodeStore<
    //     //     NS::IdN,
    //     //     Ref = <NS as crate::types::NodStore<<NS as crate::types::NStore>::IdN>>::R<'static>,
    //     // >,
    //     // NS: crate::types::lending::NodeStore<NS::IdN, Ref = <NS as crate::types::NodStore<<NS as crate::types::NStore>::IdN>>::R<'static>>,
    //     LS: crate::types::LStore,
    //     LS: crate::types::LabelStore<str, I = <LS as crate::types::LStore>::I>,

    //     NS: crate::types::TypedNodeStore<TIdN>,
    //     for<'t> <<NS as crate::types::NodeStore<<NS as crate::types::NStore>::IdN>>::NMarker as crate::types::NLending<'t, <NS as crate::types::NStore>::IdN>>::N: crate::types::Tree<
    //             Label = <LS as crate::types::LStore>::I,
    //             TreeId = NS::IdN,
    //             ChildIdx = <NS as crate::types::NStore>::Idx,
    //         > + crate::types::Typed<Type = TIdN::Ty>,
    //     TIdN: 'static + TypedNodeId<IdN = Self::IdN>,
    //     // // TIdN::IdN:
    //     // //     crate::types::NodeId<IdN = TIdN::IdN>,
    //     // NS::IdN: crate::types::NodeId<IdN = NS::IdN>,
    //     for<'t> <NS as crate::types::NodeStore<<NS as crate::types::NStore>::IdN>>::NMarker:
    //         crate::types::TypedLending<
    //             't,
    //             TIdN::Ty,
    //             IdN = <NS as crate::types::NStore>::IdN,
    //             Idx = <NS as crate::types::NStore>::Idx,
    //             Label = <LS as crate::types::LStore>::I,
    //         >,
    // {
    //     type TNS = NS;

    //     fn typed_node_store(&self) -> &Self::TNS {
    //         todo!()
    //         // &self.node_store
    //     }
    // }

    impl<'store, TS, NS, LS> HyperAST for &SimpleStores<TS, NS, LS>
    where
        TS: TypeStore,
        NS: crate::types::NStore,
        <NS as crate::types::NStore>::IdN:
            crate::types::NodeId<IdN = <NS as crate::types::NStore>::IdN>,
        NS: crate::types::NodeStore<NS::IdN>,
        // NS: for<'t> crate::types::NLending<'t, NS::IdN, N = <NS::NMarker as crate::types::AstLending<'t>>::RT>,
        // for<'t> <NS as crate::types::NLending<'t, <NS as crate::types::NStore>::IdN>>::N:
        //     crate::types::Tree<
        //         Label = <LS as crate::types::LStore>::I,
        //         TreeId = <NS as crate::types::NStore>::IdN,
        //         ChildIdx = <NS as crate::types::NStore>::Idx,
        //     >,
        // NS: crate::types::inner_ref::NodeStore<NS::IdN, Ref = NS::R<'static>>,
        // NS: crate::types::lending::NodeStore<NS::IdN, N = NS::R<'static>>,
        LS: crate::types::LStore,
        LS: crate::types::LabelStore<str, I = <LS as crate::types::LStore>::I>,
        // for<'t> <NS as crate::types::NodeStore<<NS as crate::types::NStore>::IdN>>::NMarker:
        //     crate::types::AstLending<
        //         't,
        //         IdN = <NS as crate::types::NStore>::IdN,
        //         Idx = <NS as crate::types::NStore>::Idx,
        //         Label = <LS as crate::types::LStore>::I,
        //     >,
        for<'t> crate::types::LendN<'t, NS, <NS as crate::types::NStore>::IdN>: crate::types::Tree<
            Label = <LS as crate::types::LStore>::I,
            TreeId = <NS as crate::types::NStore>::IdN,
            ChildIdx = <NS as crate::types::NStore>::Idx,
        >,
    {
        type NS = NS;

        fn node_store(&self) -> &Self::NS {
            &self.node_store
        }

        type LS = LS;

        fn label_store(&self) -> &Self::LS {
            &self.label_store
        }

        type TS = TS;
    }

    // impl<'store, TS> HyperAST<'store> for SimpleStores<TS, nodes::DefaultNodeStore>
    // where
    //     TS: TypeStore,
    // {
    //     type T = self::nodes::legion::HashedNodeRef<'store, Self::IdN>;

    //     type NS = nodes::legion::NodeStore;

    //     fn node_store(&self) -> &Self::NS {
    //         &self.node_store
    //     }

    //     type LS = labels::LabelStore;

    //     fn label_store(&self) -> &Self::LS {
    //         &self.label_store
    //     }

    //     type TS = TS;
    // }

    // impl<'store, TS> HyperAST<'store> for &SimpleStores<TS, nodes::DefaultNodeStore>
    // where
    //     TS: TypeStore,
    // {
    //     type T = self::nodes::legion::HashedNodeRef<'store, Self::IdN>;

    //     type NS = nodes::legion::NodeStore;

    //     fn node_store(&self) -> &Self::NS {
    //         &self.node_store
    //     }

    //     type LS = labels::LabelStore;

    //     fn label_store(&self) -> &Self::LS {
    //         &self.label_store
    //     }

    //     type TS = TS;
    // }

    impl<'store, TIdN: TypedNodeId<IdN = Self::IdN>, TS, NS, LS> TypedHyperAST<TIdN>
        for SimpleStores<TS, NS, LS>
    where
        TIdN::Ty: TypeTrait,
        TS: TypeStore,
        NS: crate::types::NStore,
        <NS as crate::types::NStore>::IdN:
            crate::types::NodeId<IdN = <NS as crate::types::NStore>::IdN>,
        NS: crate::types::NodeStore<NS::IdN>,
        LS: crate::types::LStore,
        LS: crate::types::LabelStore<str, I = <LS as crate::types::LStore>::I>,

        for<'t> crate::types::LendN<'t, NS, <NS as crate::types::NStore>::IdN>: crate::types::Tree<
            Label = <LS as crate::types::LStore>::I,
            TreeId = <NS as crate::types::NStore>::IdN,
            ChildIdx = <NS as crate::types::NStore>::Idx,
        >,
        //     TIdN: 'static + TypedNodeId<IdN = Self::IdN>,
        //     TS: TypeStore,
    {
        //     type TT = self::nodes::legion::HashedNodeRef<'store, TIdN>;
        //     type TNS = nodes::legion::NodeStore;

        //     fn typed_node_store(&self) -> &Self::TNS {
        //         &self.node_store
        //     }

        fn try_resolve(
            &self,
            id: &Self::IdN,
        ) -> Option<(
            <Self as crate::types::TypedLending<'_, <TIdN as TypedNodeId>::Ty>>::TT,
            TIdN,
        )> {
            todo!()
        }

        fn try_typed(&self, id: &Self::IdN) -> Option<TIdN> {
            todo!()
        }

        fn resolve_typed(
            &self,
            id: &TIdN,
        ) -> <Self as crate::types::TypedLending<'_, <TIdN as TypedNodeId>::Ty>>::TT {
            todo!()
        }
    }

    // impl<'store, TS> HyperAST<'store> for SimpleStores<TS, &NodeStoreInner, &labels::LabelStore>
    // where
    //     TS: TypeStore,
    // {
    //     type T = self::nodes::legion::HashedNodeRef<'store, Self::IdN>;

    //     type NS = nodes::legion::NodeStoreInner;

    //     fn node_store(&self) -> &Self::NS {
    //         &self.node_store
    //     }

    //     type LS = labels::LabelStore;

    //     fn label_store(&self) -> &Self::LS {
    //         &self.label_store
    //     }

    //     type TS = TS;
    // }

    // impl<'store, TIdN, TS> TypedHyperAST<'store, TIdN>
    //     for SimpleStores<TS, &NodeStoreInner, &labels::LabelStore>
    // where
    //     TIdN: 'static + TypedNodeId<IdN = Self::IdN>,
    //     TS: TypeStore,
    // {
    //     type TT = self::nodes::legion::HashedNodeRef<'store, TIdN>;
    //     type TNS = nodes::legion::NodeStoreInner;

    //     fn typed_node_store(&self) -> &Self::TNS {
    //         &self.node_store
    //     }
    // }
}

pub fn eq_node<'a, K, L, I>(
    kind: &'a K,
    label_id: Option<&'a L>,
    children: &'a [I],
) -> impl Fn(EntryRef) -> bool + 'a
where
    K: 'static + Eq + Copy + std::marker::Send + std::marker::Sync,
    L: 'static + Eq + Copy + std::marker::Send + std::marker::Sync,
    I: 'static + Eq + Copy + std::marker::Send + std::marker::Sync,
{
    move |x: EntryRef| {
        let t = x.get_component::<K>();
        if t != Ok(kind) {
            return false;
        }
        let l = x.get_component::<L>().ok();
        if l != label_id {
            return false;
        } else {
            use crate::store::nodes::legion::compo::CS; // FIXME not
            let cs = x.get_component::<CS<I>>();
            let r = match cs {
                Ok(CS(cs)) => cs.as_ref() == children,
                Err(_) => children.is_empty(),
            };
            if !r {
                return false;
            }
        }
        true
    }
}

impl ErasedHolder for legion::world::Entry<'_> {
    fn unerase_ref<T: 'static + Send + Sync>(&self, tid: std::any::TypeId) -> Option<&T> {
        if tid == std::any::TypeId::of::<T>() {
            self.get_component().ok()
        } else {
            None
        }
    }
}

impl ErasedInserter for legion::world::Entry<'_> {
    fn insert<T: 'static + Compo>(&mut self, t: T) {
        self.add_component(t);
    }
}

impl CompoRegister for World {
    type Id = legion::storage::ComponentTypeId;

    fn register_compo<T: 'static + Compo>(&mut self) -> Self::Id {
        legion::storage::ComponentTypeId::of::<T>()
    }
}

pub type RawHAST<'hast, 'acc, TS> =
    crate::store::SimpleStores<TS, &'hast NodeStoreInner, &'acc crate::store::labels::LabelStore>;
