use std::ops::Deref;

use std::fmt::Debug;

#[derive(PartialEq, Eq, Clone, Hash)]
#[repr(transparent)]
pub struct LabelValue(Box<[u8]>);

impl Debug for LabelValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", std::str::from_utf8(&self.0).unwrap())
    }
}

impl Deref for LabelValue {
    type Target = Box<[u8]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<[u8]> for LabelValue {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl From<Box<[u8]>> for LabelValue {
    fn from(x: Box<[u8]>) -> Self {
        Self(x)
    }
}

impl Into<Box<[u8]>> for LabelValue {
    fn into(self) -> Box<[u8]> {
        self.0
    }
}

impl From<Vec<u8>> for LabelValue {
    fn from(x: Vec<u8>) -> Self {
        x.into_boxed_slice().into()
    }
}

impl From<&[u8]> for LabelValue {
    fn from(x: &[u8]) -> Self {
        x.to_owned().into()
    }
}

impl From<&String> for LabelValue {
    fn from(x: &String) -> Self {
        x.as_bytes().into()
    }
}
