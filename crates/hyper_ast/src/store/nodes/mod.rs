//! Stores handling nodes
//! following the deduplication approach of the HyperAST these stores are actually Direct Acyclic Graphs,
//! thus nodes are subtrees.
pub mod compo;

#[cfg(feature = "bevy_ecs")]
pub mod bevy_ecs;
pub mod boxed_components;
#[cfg(feature = "fetched")]
pub mod fetched;
#[cfg(feature = "hecs")]
pub mod hecs;
#[cfg(feature = "legion")]
pub mod legion;

#[cfg(feature = "subtree-stats")]
pub mod stats;
#[cfg(feature = "subtree-stats")]
pub use stats::*;

#[cfg(feature = "legion")]
pub type DefaultNodeStore = legion::NodeStore;
#[cfg(not(feature = "legion"))]
pub type DefaultNodeStore = boxed_components::NodeStore;

#[cfg(feature = "legion")]
pub type DefaultNodeIdentifier = legion::NodeIdentifier;
#[cfg(not(feature = "legion"))]
pub type DefaultNodeIdentifier = boxed_components::NodeIdentifier;

#[cfg(feature = "legion")]
pub type HashedNodeRef<'store> = legion::HashedNodeRef<'store, DefaultNodeIdentifier>;
#[cfg(not(feature = "legion"))]
pub type HashedNodeRef<'store> = boxed_components::HashedNodeRef<'store, DefaultNodeIdentifier>;

/// Creates trait `$name` as the composition of `$l` and `$r`, while also generating the blanket impls
macro_rules! traits_compose {
    ($v:vis $name:ident: #[$($lc:tt)+] $l:path,  #[$($rc:tt)+] $r:path $({$($d:tt)*})?) => {
        #[cfg(all($($lc)*, $($rc)*))]
        $v trait $name: $l + $r {}
        #[cfg(all($($lc)*, $($rc)*))]
        impl<T> $name for T where T: $l + $r {}

        #[cfg(all($($lc)*, not($($rc)*)))]
        $v trait $name: $l {}
        #[cfg(all($($lc)*, not($($rc)*)))]
        impl<T> $name for T where T: $l {}

        #[cfg(all(not($($lc)*), $($rc)*))]
        $v trait $name: $r {}
        #[cfg(all(not($($lc)*), $($rc)*))]
        impl<T> $name for T where T: $r {}

        #[cfg(all(not($($lc)*), not($($rc)*)))]
        $v trait $name: $($($d)*)? {}
        #[cfg(all(not($($lc)*), not($($rc)*)))]
        impl< T > $name for T where T: $($($d)*)? {}
    };
}

traits_compose! { pub Compo:
    #[feature = "legion"] ::legion::storage::Component, 
    #[feature = "bevy_ecs"] ::bevy_ecs::component::Component
    { 'static + Send + Sync }
}

pub trait EntityBuilder {
    fn add<T: Compo>(&mut self, component: T) -> &mut Self;
}

pub trait DerivedData<EB: EntityBuilder>: Sized {
    fn persist(self, builder: &mut EB);
}


pub trait CompressedCompo {
    fn decomp(ptr: impl ErasedHolder, tid: std::any::TypeId) -> Self
    where
        Self: Sized;

    // fn compressed_insert(self, e: &mut EntityWorldMut<'_>);
    // fn components(world: &mut World) -> Vec<ComponentId>;
}

pub trait ErasedHolder {
    /// made unsafe because mixed-up args could return corrupted memory for certain impls
    unsafe fn unerase_ref_unchecked<T: 'static + Compo>(
        &self,
        tid: std::any::TypeId,
    ) -> Option<&T> {
        self.unerase_ref(tid)
    }
    fn unerase_ref<T: 'static + Send + Sync>(&self, tid: std::any::TypeId) -> Option<&T>;
}

impl ErasedHolder for &dyn std::any::Any {
    fn unerase_ref<T: 'static + Send + Sync>(&self, tid: std::any::TypeId) -> Option<&T> {
        if tid == std::any::TypeId::of::<T>() {
            self.downcast_ref()
        } else {
            None
        }
    }
}

pub trait ErasedInserter {
    fn insert<T: 'static + Compo>(&mut self, t: T);
}

pub trait CompoRegister {
    type Id;
    fn register_compo<T: 'static + Compo>(&mut self) -> Self::Id;
}

