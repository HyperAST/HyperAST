use std::{
    cell::{Ref, RefCell},
    collections::{hash_map::DefaultHasher, HashSet},
    fmt::Debug,
    hash::{BuildHasher, Hash, Hasher},
    ops::DerefMut,
    rc::Rc,
};

use atomic_counter::{AtomicCounter, ConsistentCounter};

pub struct VecHasher<T: Hash> {
    state: u64,
    node_table: Rc<RefCell<Vec<T>>>,
    default: DefaultHasher,
}

impl<T: Hash> Hasher for VecHasher<T> {
    fn write_u8(&mut self, i: u8) {
        self.node_table.borrow()[i as usize].hash(&mut self.default);
        self.state = self.default.finish();
    }
    fn write_u16(&mut self, i: u16) {
        self.node_table.borrow()[i as usize].hash(&mut self.default);
        self.state = self.default.finish();
    }
    fn write_u32(&mut self, i: u32) {
        self.node_table.borrow()[i as usize].hash(&mut self.default);
        self.state = self.default.finish();
    }
    fn write_u64(&mut self, i: u64) {
        self.node_table.borrow()[i as usize].hash(&mut self.default);
        self.state = self.default.finish();
    }
    fn write_usize(&mut self, i: usize) {
        self.node_table.borrow()[i as usize].hash(&mut self.default);
        self.state = self.default.finish();
    }
    fn write(&mut self, _bytes: &[u8]) {
        // for &byte in bytes {
        //     self.state = self.state.rotate_left(8) ^ u64::from(byte);
        // }
        panic!("should not have been called")
    }

    fn finish(&self) -> u64 {
        self.state
    }
}

pub(crate) struct BuildVecHasher<T: Hash> {
    node_table: Rc<RefCell<Vec<T>>>,
}

impl<T: Hash> BuildHasher for BuildVecHasher<T> {
    type Hasher = VecHasher<T>;
    fn build_hasher(&self) -> VecHasher<T> {
        VecHasher {
            state: 0,
            node_table: self.node_table.clone(),
            default: DefaultHasher::new(),
        }
    }
}

pub trait Convertible: Copy + Debug {
    fn from(x: usize) -> Self;
    fn to(&self) -> usize;
}

impl Convertible for u8 {
    fn from(x: usize) -> Self {
        x as u8
    }
    fn to(&self) -> usize {
        *self as usize
    }
}

impl Convertible for u16 {
    fn from(x: usize) -> Self {
        x as u16
    }
    fn to(&self) -> usize {
        *self as usize
    }
}

impl Convertible for u32 {
    fn from(x: usize) -> Self {
        x as u32
    }
    fn to(&self) -> usize {
        *self as usize
    }
}

impl Convertible for u64 {
    fn from(x: usize) -> Self {
        x as u64
    }
    fn to(&self) -> usize {
        *self as usize
    }
}

impl Convertible for usize {
    fn from(x: usize) -> Self {
        x
    }
    fn to(&self) -> usize {
        *self as usize
    }
}

pub trait ArrayOffset: Convertible {
    fn offseted_hash<H: Hasher>(&self, state: &mut H);
}

impl ArrayOffset for u8 {
    fn offseted_hash<H: Hasher>(&self, state: &mut H) {
        state.write_u8(*self);
    }
}
impl ArrayOffset for u16 {
    fn offseted_hash<H: Hasher>(&self, state: &mut H) {
        state.write_u16(*self);
    }
}
impl ArrayOffset for u32 {
    fn offseted_hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(*self);
    }
}
impl ArrayOffset for u64 {
    fn offseted_hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(*self);
    }
}
impl ArrayOffset for usize {
    fn offseted_hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(*self);
    }
}

pub struct VecMapStore<T: Hash, I: ArrayOffset> {
    hash_table: HashSet<VecMapStoreEntry<I>, BuildVecHasher<T>>,
    node_table: Rc<RefCell<Vec<T>>>,
    counter: ConsistentCounter,
}

impl<T: Hash, I: ArrayOffset> VecMapStore<T, I> {
    pub fn new(filling_element: T) -> Self {
        let node_table: Rc<RefCell<Vec<T>>> = Rc::new(RefCell::new(vec![filling_element]));
        let nt = node_table.clone();
        Self {
            node_table,
            hash_table: std::collections::HashSet::with_hasher(BuildVecHasher { node_table: nt }),
            counter: ConsistentCounter::new(1),
        }
    }
}

impl<T: Hash + Debug, I: ArrayOffset> Debug for VecMapStore<T, I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VecMapStore")
            .field("counter", &self.counter)
            .field("node_table", &self.node_table)
            .field("hash_table", &self.hash_table)
            .finish()
    }
}

#[derive(Debug)]
struct VecMapStoreEntry<I: ArrayOffset> {
    node: I,
}

impl<I: ArrayOffset> Hash for VecMapStoreEntry<I> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node.offseted_hash(state);
    }
}

impl<I: ArrayOffset> PartialEq for VecMapStoreEntry<I> {
    fn eq(&self, other: &Self) -> bool {
        self.node.to() == other.node.to()
    }
}

impl<I: ArrayOffset> Eq for VecMapStoreEntry<I> {}

impl<T: Hash, I: ArrayOffset> VecMapStore<T, I> {
    pub fn get_or_insert(&mut self, node: T) -> I {
        let entry: VecMapStoreEntry<I> = VecMapStoreEntry {
            node: Convertible::from(0),
        };
        let (l, filling) = {
            let mut nt = self.node_table.borrow_mut();
            let l = nt.len();
            nt.push(node);
            let filling = nt.swap_remove(0);
            (l, filling)
        };
        if self.hash_table.contains(&entry) {
            self.hash_table.get(&entry).unwrap().node
        } else {
            let c = self.counter.get();
            self.counter.inc();
            let entry_to_insert = VecMapStoreEntry {
                node: Convertible::from(c),
            };
            {
                let mut nt = self.node_table.borrow_mut();
                nt.deref_mut().push(filling);
                nt.deref_mut().swap(0, l);

                assert_eq!(c + 1, nt.len());
            };
            self.hash_table.insert(entry_to_insert);
            Convertible::from(c)
        }
    }

    // pub fn get_node_at_id<'b>(&'b self, id: &I) -> &'b T {
    //     let a: &'b Ref<'b,Vec<T>> = &self.node_table.borrow();
    //     let b: &'b T = &a[id.to_usize()];
    //     b
    // }

    pub fn resolve<'b>(&'b self, id: &I) -> Ref<T> {
        //Ref<'b,Vec<T>> {
        Ref::map(self.node_table.borrow(), |x| &x[id.to()])
    }
}
