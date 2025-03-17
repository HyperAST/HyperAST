pub mod boxed_components;
#[cfg(feature = "fetched")]
pub mod fetched;
#[cfg(feature = "hecs")]
pub mod hecs;
#[cfg(feature = "bevy_ecs")]
pub mod bevy_ecs;
#[cfg(feature = "legion")]
pub mod legion;

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

#[cfg(feature = "legion")]
pub trait Metadata: ::legion::storage::Component {}
#[cfg(not(feature = "legion"))]
pub trait Metadata {}

#[cfg(feature = "legion")]
impl<T> Metadata for T where T: ::legion::storage::Component {}
#[cfg(not(feature = "legion"))]
impl<T> Metadata for T {}

pub trait EntityBuilder {
    fn add<T: Metadata>(&mut self, component: T) -> &mut Self;
}
