use crate::{
    types::{
        Compo, CompoRegister, ErasedHolder, ErasedInserter, NodeId, Typed, TypedNodeId,
        TypedNodeStore,
    },
    utils::make_hash,
};
use hashbrown::hash_map::DefaultHashBuilder;
use legion::{
    storage::{Component, IntoComponentSource},
    EntityStore, World,
};
use std::{fmt::Debug, hash::Hash, ops::Deref};

pub mod compo;
pub mod dyn_builder;
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

    #[cfg(feature = "subtree-stats")]
    fn not_there(hash_set: &mut std::collections::HashSet<u32>, hash: u32) -> bool {
        if hash_set.contains(&hash) {
            return false;
        }
        hash_set.insert(hash);
        true
    }

    #[cfg(feature = "subtree-stats")]
    fn accumulate_height(counts: &mut Vec<u32>, height: u32) {
        if counts.len() <= height as usize {
            counts.resize(height as usize + 1, 0);
        }
        counts[height as usize] += 1;
    }
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

mod stores_impl {
    use crate::{
        store::SimpleStores,
        types::{
            self, HyperAST, HyperASTShared, LStore, LendN, NStore, NodeId, NodeStore, TypeStore,
            TypeTrait, TypedHyperAST, TypedNodeId,
        },
    };

    impl<TS, NS, LS> HyperASTShared for SimpleStores<TS, NS, LS>
    where
        NS: NStore,
        NS: NodeStore<<NS as NStore>::IdN>,
        LS: LStore,
        <NS as NStore>::IdN: NodeId<IdN = <NS as NStore>::IdN>,
    {
        type IdN = NS::IdN;
        type Idx = NS::Idx;
        type Label = LS::I;
    }

    impl<'a, TS, NS, LS> types::NLending<'a, <NS as NStore>::IdN> for SimpleStores<TS, NS, LS>
    where
        NS: NStore,
        NS: NodeStore<<NS as NStore>::IdN>,
        LS: LStore,
        <NS as NStore>::IdN: NodeId<IdN = <NS as NStore>::IdN>,
    {
        type N = <NS as types::NLending<'a, <NS as NStore>::IdN>>::N;
    }

    impl<'a, TS, NS, LS> types::AstLending<'a> for SimpleStores<TS, NS, LS>
    where
        NS: NStore,
        NS: NodeStore<<NS as NStore>::IdN>,
        LS: LStore,
        <NS as NStore>::IdN: NodeId<IdN = <NS as NStore>::IdN>,
        for<'t> LendN<'t, NS, <NS as NStore>::IdN>: types::Tree<
            Label = <LS as LStore>::I,
            TreeId = <NS as NStore>::IdN,
            ChildIdx = <NS as NStore>::Idx,
        >,
    {
        type RT = <NS as types::NLending<'a, Self::IdN>>::N;
    }

    impl<TS, NS, LS> HyperASTShared for &SimpleStores<TS, NS, LS>
    where
        NS: NStore,
        NS: NodeStore<<NS as NStore>::IdN>,
        LS: LStore,
        <NS as NStore>::IdN: NodeId<IdN = <NS as NStore>::IdN>,
    {
        type IdN = NS::IdN;
        type Idx = NS::Idx;
        type Label = LS::I;
    }

    impl<'a, TS, NS, LS> types::NLending<'a, <NS as NStore>::IdN> for &SimpleStores<TS, NS, LS>
    where
        NS: NStore,
        NS: NodeStore<<NS as NStore>::IdN>,
        LS: LStore,
        <NS as NStore>::IdN: NodeId<IdN = <NS as NStore>::IdN>,
        for<'t> LendN<'t, NS, <NS as NStore>::IdN>: types::Tree<
            Label = <LS as LStore>::I,
            TreeId = <NS as NStore>::IdN,
            ChildIdx = <NS as NStore>::Idx,
        >,
    {
        type N = <NS as types::NLending<'a, <NS as NStore>::IdN>>::N;
    }

    impl<'a, TS, NS, LS> types::AstLending<'a> for &SimpleStores<TS, NS, LS>
    where
        NS: NStore,
        NS: NodeStore<<NS as NStore>::IdN>,
        LS: LStore,
        <NS as NStore>::IdN: NodeId<IdN = <NS as NStore>::IdN>,
        for<'t> LendN<'t, NS, <NS as NStore>::IdN>: types::Tree<
            Label = <LS as LStore>::I,
            TreeId = <NS as NStore>::IdN,
            ChildIdx = <NS as NStore>::Idx,
        >,
    {
        type RT = <NS as types::NLending<'a, <NS as NStore>::IdN>>::N;
    }

    impl<'store, TS, NS, LS> HyperAST for SimpleStores<TS, NS, LS>
    where
        TS: TypeStore,
        NS: NStore,
        <NS as NStore>::IdN: NodeId<IdN = <NS as NStore>::IdN>,
        NS: NodeStore<NS::IdN>,
        LS: LStore,
        LS: types::LabelStore<str, I = <LS as LStore>::I>,
        for<'t> LendN<'t, NS, <NS as NStore>::IdN>: types::Tree<
            Label = <LS as LStore>::I,
            TreeId = <NS as NStore>::IdN,
            ChildIdx = <NS as NStore>::Idx,
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
    impl<'a, Ty, TS, NS, LS> types::TypedLending<'a, Ty> for SimpleStores<TS, NS, LS>
    where
        NS: NStore,
        NS: NodeStore<<NS as NStore>::IdN>,
        LS: LStore,
        <NS as NStore>::IdN: NodeId<IdN = <NS as NStore>::IdN>,
        for<'t> LendN<'t, NS, <NS as NStore>::IdN>: types::Tree<
            Label = <LS as LStore>::I,
            TreeId = <NS as NStore>::IdN,
            ChildIdx = <NS as NStore>::Idx,
        >,
        Ty: TypeTrait,
    {
        type TT = TypedHolder<<Self as types::AstLending<'a>>::RT, Ty>;
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

    impl<RT, Ty: TypeTrait> types::Typed for TypedHolder<RT, Ty> {
        type Type = Ty;
        fn get_type(&self) -> Self::Type {
            todo!()
        }
    }

    impl<'store, TS, NS, LS> HyperAST for &SimpleStores<TS, NS, LS>
    where
        TS: TypeStore,
        NS: NStore,
        <NS as NStore>::IdN: NodeId<IdN = <NS as NStore>::IdN>,
        NS: NodeStore<NS::IdN>,
        LS: LStore,
        LS: types::LabelStore<str, I = <LS as LStore>::I>,
        for<'t> LendN<'t, NS, <NS as NStore>::IdN>: types::Tree<
            Label = <LS as LStore>::I,
            TreeId = <NS as NStore>::IdN,
            ChildIdx = <NS as NStore>::Idx,
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

    impl<'store, TIdN: TypedNodeId<IdN = Self::IdN>, TS, NS, LS> TypedHyperAST<TIdN>
        for SimpleStores<TS, NS, LS>
    where
        TIdN::Ty: TypeTrait,
        TS: TypeStore,
        NS: NStore,
        <NS as NStore>::IdN: NodeId<IdN = <NS as NStore>::IdN>,
        NS: NodeStore<NS::IdN>,
        LS: LStore,
        LS: types::LabelStore<str, I = <LS as LStore>::I>,

        for<'t> LendN<'t, NS, <NS as NStore>::IdN>: types::Tree<
            Label = <LS as LStore>::I,
            TreeId = <NS as NStore>::IdN,
            ChildIdx = <NS as NStore>::Idx,
        >,
    {
        fn try_resolve(
            &self,
            id: &Self::IdN,
        ) -> Option<(
            <Self as types::TypedLending<'_, <TIdN as TypedNodeId>::Ty>>::TT,
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
        ) -> <Self as types::TypedLending<'_, <TIdN as TypedNodeId>::Ty>>::TT {
            todo!()
        }
    }

    impl<'a, TS: Copy + types::TypeStore> types::StoreLending<'a> for crate::store::SimpleStores<TS> {
        type S = crate::store::SimpleStores<
            TS,
            &'a super::NodeStoreInner,
            &'a crate::store::labels::LabelStore,
        >;
    }

    impl<TS: Copy + types::TypeStore> types::StoreRefAssoc for crate::store::SimpleStores<TS> {
        type S<'a> = crate::store::SimpleStores<
            TS,
            &'a super::NodeStoreInner,
            &'a crate::store::labels::LabelStore,
        >;
    }
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
            use compo::CS; // FIXME not
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
