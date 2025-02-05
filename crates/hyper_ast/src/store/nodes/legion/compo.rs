pub struct More<T>(pub T);
pub struct Size(pub u32);
pub struct SizeNoSpaces(pub u32);
pub struct Height(pub u32);
pub struct BytesLen(pub u32);
pub struct LineCount(pub u16);

pub struct HStruct(pub u32);
pub struct HLabel(pub u32);

#[derive(PartialEq, Eq)]
pub struct CSStaticCount(pub u8);
pub struct CS0<T: Eq, const N: usize>(pub [T; N]);
pub struct CSE<const N: usize>([legion::Entity; N]);
pub struct NoSpacesCS0<T: Eq, const N: usize>(pub [T; N]);
#[derive(PartialEq, Eq, Debug)]
pub struct CS<T>(pub Box<[T]>);
pub struct NoSpacesCS<T: Eq>(pub Box<[T]>);
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

pub struct RoleOffsets(pub Box<[u8]>);
pub struct Precomp<T>(pub T);
pub struct PrecompFlag;
