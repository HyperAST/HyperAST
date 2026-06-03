//! Some common component definitions to be used with ECSs.
//!
//! Not an exhaustive list.
//! These components enable to leverage the type system to efficiently register and access components of entities.
//! In the context of the HyperAST, these components are used to identify the data of nodes within the AST.
//! Certain components hold identifying data, while other hold derived data.
//!
//! See also [`TypeU16`] and [`LabelIdentifier`] for more examples of components
//!
//! [`TypeU16`]: crate::types::TypeU16
//! [`LabelIdentifier`]: crate::store::defaults::LabelIdentifier

#[repr(transparent)]
pub struct More<T>(pub T);
pub struct Size(pub u32);
pub struct SizeNoSpaces(pub u32);
pub struct Height(pub u32);
pub struct BytesLen(pub u32);
pub struct LineCount(pub u16);
pub struct VizCsCount(pub u32);

pub struct HStruct(pub u32);
pub struct HLabel(pub u32);

pub struct CS0<T, const N: usize>(pub [T; N]);
pub struct NoSpacesCS0<T, const N: usize>(pub [T; N]);
#[derive(PartialEq, Eq, Debug)]
pub struct CS<T>(pub Box<[T]>);
pub struct NoSpacesCS<T>(pub Box<[T]>);

impl<'a, T: Eq> From<&'a CS<T>> for &'a [T] {
    fn from(cs: &'a CS<T>) -> Self {
        &cs.0
    }
}
impl<'a, T: Eq, const N: usize> From<&'a CS0<T, N>> for &'a [T] {
    fn from(cs: &'a CS0<T, N>) -> Self {
        &cs.0
    }
}

pub struct Roles<R>(pub Box<[R]>);
pub struct RoleOffsets(pub Box<[u8]>);
pub struct Precomp<T>(pub T);
pub struct PrecompFlag;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct Flags<F>(pub F);

// TODO unify with `Precomp` so we directly use the code patterns, way more flexible
// To be done cleanly it would require to preregister them in priority to custom ones
pub struct StmtCount(pub u8);
pub struct MemberImportCount(pub u8);
pub struct LeafCount(pub u8);

macro_rules! impls {
    ($Ty:ident) => {
        #[cfg(feature = "bevy_ecs")]
        impl bevy_ecs::component::Component for $Ty {
            const STORAGE_TYPE: bevy_ecs::component::StorageType =
                bevy_ecs::component::StorageType::Table;
        }
    };
    ($Ty:ident < $T:ident >) => {
        #[cfg(feature = "bevy_ecs")]
        impl<$T: 'static + Send + Sync> bevy_ecs::component::Component for $Ty<$T> {
            const STORAGE_TYPE: bevy_ecs::component::StorageType =
                bevy_ecs::component::StorageType::Table;
        }
    };
    ($Ty:ident < $T:ident, const $N:ident: $U:ident >) => {
        #[cfg(feature = "bevy_ecs")]
        impl<$T: 'static + Send + Sync, const $N: $U> bevy_ecs::component::Component
            for $Ty<$T, $N>
        {
            const STORAGE_TYPE: bevy_ecs::component::StorageType =
                bevy_ecs::component::StorageType::Table;
        }
    };
}

impls! { More<T> }
impls! { Size }
impls! { SizeNoSpaces }
impls! { Height }
impls! { BytesLen }
impls! { LineCount }
impls! { VizCsCount }
impls! { HStruct }
impls! { HLabel }
impls! { Roles<R> }
impls! { RoleOffsets }
impls! { Precomp<T> }
impls! { PrecompFlag }
impls! { CS<T> }
impls! { NoSpacesCS<T> }
impls! { CS0<T, const N: usize> }
impls! { NoSpacesCS0<T, const N: usize> }
impls! { Flags<F> }

macro_rules! impl_deref {
    ($Ty:ident < $T:ident >) => {
        impl<$T> std::ops::Deref for $Ty<$T> {
            type Target = $T;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

impl_deref! { Flags<F> }
impl_deref! { More<T> }
