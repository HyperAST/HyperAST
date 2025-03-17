use std::{borrow::Borrow, marker::PhantomData};

use crate::types::{SimpleHyperAST, TypeStore};

pub mod handle;
pub mod labels;
// pub mod mapped_world;
pub mod nodes;
// pub mod ecs; // TODO try a custom ecs ?
// pub mod radix_hash_store; // TODO yet another WIP store
// pub mod vec_map_store; // TODO yet another WIP store

pub struct SimpleStores<TS, NS = nodes::DefaultNodeStore, LS = labels::LabelStore> {
    pub label_store: LS,
    pub node_store: NS,
    pub type_store: PhantomData<TS>,
}

#[cfg(feature = "scripting")]
impl<TS> mlua::UserData for SimpleStores<TS> {}

impl<TS, NS, LS> SimpleStores<TS, NS, LS> {
    pub fn change_type_store<TS2>(self) -> SimpleStores<TS2, NS, LS> {
        SimpleStores {
            type_store: PhantomData,
            node_store: self.node_store,
            label_store: self.label_store,
        }
    }
}

/// Declare we can convert from Self to T,
/// e.g. from the git::types::TStore to the Java one, but not the contrary
pub trait TyDown<T> {}

impl<TS, NS, LS> SimpleStores<TS, NS, LS> {
    pub fn mut_with_ts<TS2>(&mut self) -> &mut SimpleStores<TS2, NS, LS>
    where
        TS: TyDown<TS2>,
    {
        unsafe { std::mem::transmute(self) }
    }
    pub fn with_ts<TS2>(&self) -> &SimpleStores<TS2, NS, LS>
    where
        TS: TyDown<TS2>,
    {
        unsafe { std::mem::transmute(self) }
    }
    pub unsafe fn erase_ts_unchecked(&self) -> &SimpleStores<(), NS, LS> {
        unsafe { std::mem::transmute(self) }
    }
}

impl<TS: Default, NS: Default, LS: Default> Default for SimpleStores<TS, NS, LS> {
    fn default() -> Self {
        Self {
            label_store: Default::default(),
            type_store: Default::default(),
            node_store: Default::default(),
        }
    }
}

impl<TS: Copy, NS: Copy, LS: Copy> Copy for SimpleStores<TS, NS, LS> {}
impl<TS: Clone, NS: Clone, LS: Clone> Clone for SimpleStores<TS, NS, LS> {
    fn clone(&self) -> Self {
        Self {
            label_store: self.label_store.clone(),
            node_store: self.node_store.clone(),
            type_store: self.type_store.clone(),
        }
    }
}

impl<'store, TS, NS, LS> crate::types::RoleStore for SimpleStores<TS, NS, LS>
where
    TS: crate::types::RoleStore,
{
    type IdF = TS::IdF;

    type Role = TS::Role;

    fn resolve_field(lang: crate::types::LangWrapper<Self::Ty>, field_id: Self::IdF) -> Self::Role {
        TS::resolve_field(lang, field_id)
    }
    fn intern_role(lang: crate::types::LangWrapper<Self::Ty>, role: Self::Role) -> Self::IdF {
        TS::intern_role(lang, role)
    }
}

// impl<'a, IdN, TS, NS, LS> crate::types::NLending<'a, IdN> for SimpleStores<TS, NS, LS>
// where
//     <NS as crate::types::NLending<'a, IdN>>::N: crate::types::Tree<TreeId = IdN>,
//     IdN: crate::types::NodeId<IdN = IdN>,
//     NS: crate::types::NodeStore<IdN>,
// {
//     type N = <NS as crate::types::NLending<'a, IdN>>::N;
// }

// impl<IdN, TS, NS, LS> crate::types::NodeStore<IdN> for SimpleStores<TS, NS, LS>
// where
//     for<'a> <NS as crate::types::NLending<'a, IdN>>::N: crate::types::Tree<TreeId = IdN>,
//     IdN: crate::types::NodeId<IdN = IdN>,
//     NS: crate::types::NodeStore<IdN>,
//     NS: crate::types::NStore<IdN = IdN>,
//     LS: crate::types::LStore,
// {
//     fn resolve(&self, id: &IdN) -> <Self as crate::types::NLending<'_, IdN>>::N {
//         self.node_store.resolve(id)
//     }
//     type NMarker = NS::NMarker;
// }

// impl<IdN, TS, NS, LS> crate::types::NodStore<IdN> for SimpleStores<TS, NS, LS>
// where
//     for<'a> NS::R<'a>: crate::types::Tree<TreeId = IdN>,
//     IdN: crate::types::NodeId<IdN = IdN>,
//     NS: crate::types::NodeStore<IdN>,
// {
//     type R<'a> = NS::R<'a>;
// }

// impl<IdN, TS, NS, LS> crate::types::NodeStore<IdN> for SimpleStores<TS, NS, LS>
// where
//     for<'a> NS::R<'a>: crate::types::Tree<TreeId = IdN>,
//     IdN: crate::types::NodeId<IdN = IdN>,
//     NS: crate::types::NodeStore<IdN>,
// {
//     fn resolve(&self, id: &IdN) -> Self::R<'_> {
//         self.node_store.resolve(id)
//     }
// }

impl<IdN, TS, NS, LS> crate::types::NodeStoreLean<IdN> for SimpleStores<TS, NS, LS>
where
    NS::R: crate::types::Tree<TreeId = IdN>,
    IdN: crate::types::NodeId<IdN = IdN>,
    NS: crate::types::NodeStoreLean<IdN>,
{
    type R = NS::R;

    fn resolve(&self, id: &IdN) -> Self::R {
        self.node_store.resolve(id)
    }
}

impl<'store, TS, NS, LS> crate::types::LabelStore<str> for SimpleStores<TS, NS, LS>
where
    LS: crate::types::LabelStore<str>,
{
    type I = LS::I;

    fn get_or_insert<U: Borrow<str>>(&mut self, node: U) -> Self::I {
        self.label_store.get_or_insert(node)
    }

    fn get<U: Borrow<str>>(&self, node: U) -> Option<Self::I> {
        self.label_store.get(node)
    }

    fn resolve(&self, id: &Self::I) -> &str {
        self.label_store.resolve(id)
    }
}

impl<'store, TS, NS, LS> crate::types::TypeStore for SimpleStores<TS, NS, LS>
where
    TS::Ty: 'static + std::hash::Hash,
    TS: TypeStore,
{
    type Ty = TS::Ty;
}

pub mod defaults {
    pub type LabelIdentifier = super::labels::DefaultLabelIdentifier;
    pub type LabelValue = super::labels::DefaultLabelValue;
    pub type NodeIdentifier = super::nodes::DefaultNodeIdentifier;
}

impl<'store, T, TS, NS, LS> From<&'store SimpleStores<TS, NS, LS>>
    for SimpleHyperAST<T, &'store TS, &'store NS, &'store LS>
{
    fn from(value: &'store SimpleStores<TS, NS, LS>) -> Self {
        Self {
            node_store: &value.node_store,
            label_store: &value.label_store,
            _phantom: std::marker::PhantomData,
        }
    }
}
