pub use hyperast::store::nodes::fetched::{FetchedLabels, NodeIdentifier, NodeStore};
use hyperast::{
    store::nodes::fetched::{HashedNodeRef, LabelIdentifier},
    types::AnyType,
};
use std::{
    borrow::Borrow,
    collections::{HashSet, VecDeque},
    hash::Hash,
};

#[derive(Default)]
pub(crate) struct TStore;

impl<'a> hyperast::types::TypeStore for TStore {
    type Ty = AnyType;
}

#[derive(Default)]
pub struct FetchedHyperAST {
    pub(crate) label_store: std::sync::RwLock<FetchedLabels>,
    pub(crate) node_store: std::sync::RwLock<NodeStore>,
    pub(crate) type_store: TStore,
    // /// each set is fetched sequentially, non blocking
    // /// pushed ids are tested against all pending sets because they might not have entered the store
    // /// new set every 100 elements, due to id serialized size in url
    // /// TODO split by arch
    // /// TODO maybe use a crossbeam queue while putting a dummy value in nodestore or use dashmap
    // nodes_waiting: std::sync::Mutex<VecDeque<HashSet<NodeIdentifier>>>,
    // /// each set is fetched sequentially, non blocking
    // /// pushed ids are tested against all pending sets because they might not have entered the store
    // /// new set every 200 elements, due to id serialized size in url
    // labels_waiting: std::sync::Mutex<VecDeque<HashSet<LabelIdentifier>>>,
    /// pending ie. nodes in flight
    pub(crate) nodes_pending: std::sync::Mutex<VecDeque<HashSet<NodeIdentifier>>>,
    pub(crate) nodes_waiting: std::sync::Mutex<Option<HashSet<NodeIdentifier>>>,
    pub(crate) labels_pending: std::sync::Mutex<VecDeque<HashSet<LabelIdentifier>>>,
    pub(crate) labels_waiting: std::sync::Mutex<Option<HashSet<LabelIdentifier>>>,
    /// timer to avoid flooding
    pub(crate) timer: std::sync::Mutex<Option<f32>>,
}

struct Fetchable<'a, I, S> {
    pub(crate) store: &'a std::sync::RwLock<S>,
    pub(crate) pending: &'a std::sync::Mutex<VecDeque<HashSet<I>>>,
    pub(crate) waiting: &'a std::sync::Mutex<Option<HashSet<I>>>,
}

impl FetchedHyperAST {
    fn read(&self) -> AcessibleFetchedHyperAST<'_> {
        AcessibleFetchedHyperAST {
            label_store: self.label_store.read().unwrap(),
            node_store: self.node_store.read().unwrap(),
            type_store: &self.type_store,
            nodes_pending: self.nodes_pending.lock().unwrap(),
            nodes_waiting: std::cell::RefCell::new(self.nodes_waiting.lock().unwrap()),
            labels_pending: self.labels_pending.lock().unwrap(),
            labels_waiting: std::cell::RefCell::new(self.labels_waiting.lock().unwrap()),
        }
    }
}

struct AcessibleFetchedHyperAST<'a> {
    pub(crate) label_store: std::sync::RwLockReadGuard<'a, FetchedLabels>,
    pub(crate) node_store: std::sync::RwLockReadGuard<'a, NodeStore>,
    pub(crate) type_store: &'a TStore,
    pub(crate) nodes_pending: std::sync::MutexGuard<'a, VecDeque<HashSet<NodeIdentifier>>>,
    pub(crate) nodes_waiting:
        std::cell::RefCell<std::sync::MutexGuard<'a, Option<HashSet<NodeIdentifier>>>>,
    pub(crate) labels_pending: std::sync::MutexGuard<'a, VecDeque<HashSet<LabelIdentifier>>>,
    pub(crate) labels_waiting:
        std::cell::RefCell<std::sync::MutexGuard<'a, Option<HashSet<LabelIdentifier>>>>,
}

impl<'b> hyperast::types::NodStore<NodeIdentifier> for AcessibleFetchedHyperAST<'b> {
    type R<'a> = HashedNodeRef<'a, NodeIdentifier>;
}

impl<'b> hyperast::types::NodeStore<NodeIdentifier> for AcessibleFetchedHyperAST<'b> {
    fn resolve(&self, id: &NodeIdentifier) -> Self::R<'_> {
        if let Some(r) = self.node_store.try_resolve(*id) {
            r
        } else {
            // TODO use a recursive fetch
            // TODO need an additional queue for such recursive fetch
            // TODO use additional nodes that are not fetched but where fetched to avoid transfering more than necessary
            if !self.nodes_pending.iter().any(|x| x.contains(id)) {
                self.nodes_waiting
                    .borrow_mut()
                    .get_or_insert(Default::default())
                    .insert(*id);
            }
            // unimplemented!()
            self.node_store.unavailable_node()
        }
    }
}

impl<'b> hyperast::types::LabelStore<str> for AcessibleFetchedHyperAST<'b> {
    type I = LabelIdentifier;

    fn get_or_insert<U: Borrow<str>>(&mut self, _node: U) -> Self::I {
        todo!()
    }

    fn get<U: Borrow<str>>(&self, _node: U) -> Option<Self::I> {
        todo!()
    }

    fn resolve(&self, id: &Self::I) -> &str {
        if let Some(get) = self.label_store.try_resolve(id) {
            get
        } else {
            if !self.labels_pending.iter().any(|x| x.contains(id)) {
                self.labels_waiting
                    .borrow_mut()
                    .get_or_insert(Default::default())
                    .insert(*id);
            }
            "."
        }
    }
}

impl<'a, 'b> hyperast::types::TypeStore for AcessibleFetchedHyperAST<'b> {
    type Ty = AnyType;
}

impl Hash for FetchedHyperAST {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.label_store.read().unwrap().len().hash(state);
        self.node_store.read().unwrap().len().hash(state);
    }
}
