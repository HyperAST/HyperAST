pub mod fetched;
#[cfg(feature = "hecs")]
pub mod hecs;
#[cfg(feature = "legion")]
pub mod legion;
pub mod boxed_components;

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
