use std::{fmt::{Debug, Display}, borrow::Borrow};

use string_interner::{StringInterner, DefaultSymbol, Symbol};

use crate::types::LabelStore as _;

pub struct LabelStore {
    count: usize,
    internal: StringInterner, //VecMapStore<OwnedLabel, LabelIdentifier>,
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
pub type DefaultLabelIdentifier = DefaultSymbol;

impl crate::types::LabelStore<DefaultLabelValue> for LabelStore {
    type I = DefaultLabelIdentifier;
    fn get_or_insert<T: Borrow<DefaultLabelValue>>(&mut self, node: T) -> Self::I {
        self.count += 1;
        self.internal.get_or_intern(node.borrow())
    }
    fn get<T: Borrow<DefaultLabelValue>>(&self, node: T) -> Option<Self::I> {
        self.internal.get(node.borrow())
    }

    fn resolve(&self, id: &Self::I) -> &DefaultLabelValue {
        self.internal.resolve(*id).unwrap()
    }
}

impl crate::types::LabelStore<DefaultLabelValue> for &LabelStore {
    type I = DefaultLabelIdentifier;
    fn get_or_insert<T: Borrow<DefaultLabelValue>>(&mut self, node: T) -> Self::I {
        unimplemented!()
    }
    fn get<T: Borrow<DefaultLabelValue>>(&self, node: T) -> Option<Self::I> {
        self.internal.get(node.borrow())
    }

    fn resolve(&self, id: &Self::I) -> &DefaultLabelValue {
        self.internal.resolve(*id).unwrap()
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