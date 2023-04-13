
#[cfg(feature="legion")]
pub mod legion;
mod simple;
pub mod fetched;



#[cfg(feature="legion")]
pub type DefaultNodeStore = legion::NodeStore;
#[cfg(not(feature="legion"))]
pub type DefaultNodeStore = simple::NodeStore;

#[cfg(feature="legion")]
pub type DefaultNodeIdentifier = legion::NodeIdentifier;
#[cfg(not(feature="legion"))]
pub type DefaultNodeIdentifier = simple::NodeIdentifier;


#[cfg(feature="legion")]
pub type HashedNodeRef<'store> = legion::HashedNodeRef<'store>;
#[cfg(not(feature="legion"))]
pub type HashedNodeRef<'store> = simple::HashedNodeRef<'store>;
