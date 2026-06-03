use std::{
    borrow::Borrow,
    fmt::{Debug, Display},
};

use string_interner::{DefaultSymbol, StringInterner, Symbol};

use crate::types::LabelStore as _;

#[derive(Default)]
pub struct LabelStore {
    count: usize,
    internal: StringInterner<string_interner::DefaultBackend>, //VecMapStore<OwnedLabel, LabelIdentifier>,
}

impl Debug for LabelStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LabelStore")
            .field("count", &self.count)
            .field("internal_len", &self.internal.len())
            .field("internal", &self.internal)
            .finish()
    }
}

impl Display for LabelStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, x) in self.internal.clone().into_iter() {
            writeln!(f, "{:?}:{:?}", i.to_usize(), x)?
        }
        Ok(())
    }
}

pub type DefaultLabelValue = str;
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::component::Component))]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefaultLabelIdentifier(pub(crate) DefaultSymbol);

pub fn label_id_from_usize(x: usize) -> Option<DefaultLabelIdentifier> {
    DefaultSymbol::try_from_usize(x).map(DefaultLabelIdentifier)
}

impl crate::types::LStore for LabelStore {
    type I = DefaultLabelIdentifier;
}

impl crate::types::LStore for &LabelStore {
    type I = DefaultLabelIdentifier;
}

impl crate::types::LabelStore<DefaultLabelValue> for LabelStore {
    type I = DefaultLabelIdentifier;
    fn get_or_insert<T: Borrow<DefaultLabelValue>>(&mut self, node: T) -> Self::I {
        self.count += 1;
        DefaultLabelIdentifier(self.internal.get_or_intern(node.borrow()))
    }
    fn get<T: Borrow<DefaultLabelValue>>(&self, node: T) -> Option<Self::I> {
        self.internal.get(node.borrow()).map(DefaultLabelIdentifier)
    }

    fn resolve(&self, id: &Self::I) -> &DefaultLabelValue {
        self.internal.resolve(id.0).unwrap()
    }
}

impl crate::types::LabelStore<DefaultLabelValue> for &LabelStore {
    type I = DefaultLabelIdentifier;
    fn get_or_insert<T: Borrow<DefaultLabelValue>>(&mut self, _node: T) -> Self::I {
        unimplemented!("&mut & does not allow to mutate in place :/")
    }
    fn get<T: Borrow<DefaultLabelValue>>(&self, node: T) -> Option<Self::I> {
        self.internal.get(node.borrow()).map(DefaultLabelIdentifier)
    }

    fn resolve(&self, id: &Self::I) -> &DefaultLabelValue {
        self.internal.resolve(id.0).unwrap()
    }
}

impl LabelStore {
    pub fn new() -> Self {
        let mut r = Self {
            count: 1,
            internal: Default::default(),
        };
        r.get_or_insert("length"); // TODO verify/model statically
        r
    }
}
