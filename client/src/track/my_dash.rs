use std::{
    cell::UnsafeCell,
    collections::hash_map::RandomState,
    fmt::Debug,
    hash::{BuildHasher, Hash},
};

use dashmap::{DashMap, RwLockWriteGuard, SharedValue};
use hashbrown::HashMap;

// pub fn entries<'a, K: 'a + Eq + Hash, V: 'a, S: BuildHasher + Clone>(
//     map: DashMap<K, V, S>,
//     key1: K,
//     key2: K,
// ) -> Entry<'a, K, V, S> {
//     let hash = map.hash_usize(&key1);

//     let idx = map.determine_shard(hash);

//     let shard: RwLockWriteGuard<HashMap<K, SharedValue<V>, S>> = unsafe {
//         debug_assert!(idx < map.shards().len());

//         map.shards().get_unchecked(idx).write()
//     };

//     #[repr(transparent)]
//     struct MySharedValue<T> {
//         value: UnsafeCell<T>,
//     }

//     impl<T> MySharedValue<T> {
//         /// Get a mutable raw pointer to the underlying value
//         fn as_ptr(&self) -> *mut T {
//             self.value.get()
//         }
//     }
//     // SAFETY: Sharded and UnsafeCell are transparent wrappers of V
//     let shard: RwLockWriteGuard<HashMap<K, V, S>> = unsafe { std::mem::transmute(shard) };
//     if let Some((kptr, vptr)) = shard.get_key_value(&key1) {
//         unsafe {
//             let kptr: *const K = kptr;
//             // SAFETY: same memory layout because transparent and same fields
//             let vptr: &MySharedValue<V> = std::mem::transmute(&vptr);
//             let vptr: *mut V = vptr.as_ptr();
//             Entry::Occupied(OccupiedEntry::new(shard, key1, (kptr, vptr)))
//         }
//     } else {
//         unsafe {
//             // SAFETY: same memory layout because transparent and same fields
//             let shard: RwLockWriteGuard<HashMap<K, V, S>> = std::mem::transmute(shard);
//             Entry::Vacant(VacantEntry::new(shard, key1))
//         }
//     }
pub fn entries<'a, K: 'a + Eq + Hash, V: 'a, S: BuildHasher + Clone>(
    map: &'a DashMap<K, V, S>,
    key1: K,
    key2: K,
) -> Entry<'a, K, V, S> {
    assert!(key1 != key2, "keys should be different");
    let hash1 = map.hash_usize(&key1);
    let idx1 = map.determine_shard(hash1);
    let hash2 = map.hash_usize(&key2);
    let idx2 = map.determine_shard(hash2);

    if idx1 == idx2 {
        let shard = unsafe {
            debug_assert!(idx1 < map.shards().len());
            debug_assert!(idx2 < map.shards().len());
            map.shards().get_unchecked(idx1).write()
        };
        // SAFETY: Sharded and UnsafeCell are transparent wrappers of V
        let shard: RwLockWriteGuard<HashMap<K, V, S>> = unsafe { std::mem::transmute(shard) };
        let elem1 = shard
            .get_key_value(&key1)
            .map(|(kptr, vptr)| unsafe { as_ptr(kptr, vptr) });
        let elem2 = shard
            .get_key_value(&key2)
            .map(|(kptr, vptr)| unsafe { as_ptr(kptr, vptr) });
        Entry {
            shard1: shard,
            shard2: None,
            key1,
            key2,
            elem1,
            elem2,
        }
    } else {
        let (shard1, shard2) = unsafe {
            debug_assert!(idx1 < map.shards().len());
            debug_assert!(idx2 < map.shards().len());
            (
                map.shards().get_unchecked(idx1).write(),
                map.shards().get_unchecked(idx2).write(),
            )
        };
        let shard1: RwLockWriteGuard<HashMap<K, V, S>> = unsafe { std::mem::transmute(shard1) };
        let shard2: RwLockWriteGuard<HashMap<K, V, S>> = unsafe { std::mem::transmute(shard2) };
        let elem1 = shard1
            .get_key_value(&key1)
            .map(|(kptr, vptr)| unsafe { as_ptr(kptr, vptr) });
        let elem2 = shard2
            .get_key_value(&key2)
            .map(|(kptr, vptr)| unsafe { as_ptr(kptr, vptr) });
        Entry {
            shard1: shard1,
            shard2: Some(shard2),
            key1,
            key2,
            elem1,
            elem2,
        }
    }
}

unsafe fn as_ptr<'a, K: 'a + Eq + Hash, V: 'a>(kptr1: &K, vptr1: &V) -> (*const K, *mut V) {
    let kptr1: *const K = kptr1;
    // SAFETY: same memory layout because transparent and same fields
    let vptr1: &MySharedValue<V> = std::mem::transmute(&vptr1);
    let vptr1: *mut V = vptr1.as_ptr();
    (kptr1, vptr1)
}

pub(super) unsafe fn shard_as_ptr<'a, V: 'a>(vptr1: &SharedValue<V>) -> *mut V {
    // SAFETY: same memory layout because transparent and same fields
    let vptr1: &MySharedValue<V> = std::mem::transmute(&vptr1);
    let vptr1: *mut V = vptr1.as_ptr();
    vptr1
}

pub(super) unsafe fn shard_as_ptr2<'a, V: 'a>(vptr1: &V) -> *mut V {
    // SAFETY: same memory layout because transparent and same fields
    let vptr1: &MySharedValue<V> = std::mem::transmute(&vptr1);
    let vptr1: *mut V = vptr1.as_ptr();
    vptr1
}

#[repr(transparent)]
struct MySharedValue<T> {
    value: UnsafeCell<T>,
}

impl<T> MySharedValue<T> {
    /// Get a mutable raw pointer to the underlying value
    fn as_ptr(&self) -> *mut T {
        self.value.get()
    }
}
pub struct Entry<'a, K, V, S = RandomState> {
    shard1: RwLockWriteGuard<'a, HashMap<K, V, S>>,
    shard2: Option<RwLockWriteGuard<'a, HashMap<K, V, S>>>,
    elem1: Option<(*const K, *mut V)>,
    elem2: Option<(*const K, *mut V)>,
    key1: K,
    key2: K,
}
unsafe impl<'a, K: Eq + Hash + Sync, V: Sync, S: BuildHasher> Send for Entry<'a, K, V, S> {}
unsafe impl<'a, K: Eq + Hash + Sync, V: Sync, S: BuildHasher> Sync for Entry<'a, K, V, S> {}

impl<'a, K: Clone + Eq + Hash + Debug, V: Debug, S: BuildHasher> Entry<'a, K, V, S> {
    pub fn or_insert_with(
        self,
        value: impl FnOnce((Option<()>, Option<()>)) -> (Option<V>, Option<V>),
    ) -> RefMut<'a, K, V, S> {
        match self {
            Entry {
                shard1,
                shard2,
                elem1: Some((k1, v1)),
                elem2: Some((k2, v2)),
                ..
            } => {
                dbg!(v1);
                dbg!(v2);
                RefMut {
                    guard1: shard1,
                    guard2: shard2,
                    k1,
                    k2,
                    v1,
                    v2,
                }
            }
            Entry {
                mut shard1,
                shard2: None,
                elem1,
                elem2,
                key1,
                key2,
            } => {
                let (r1, r2) = value((elem1.as_ref().map(|_| ()), elem2.as_ref().map(|_| ())));
                let k1 = key1.clone();
                let k2 = key2.clone();
                if elem1.is_none() {
                    let value = r1.expect("some value");
                    let key = key1;
                    let shard = &mut shard1;
                    insert2_p1(key, shard, value)
                }
                if elem2.is_none() {
                    let value = r2.expect("some value");
                    let key = key2;
                    let shard = &mut shard1;
                    insert2_p1(key, shard, value)
                }
                let (k1, v1) = elem1.unwrap_or_else(|| {
                    let shard = &mut shard1;
                    insert2_p2(&k1, shard)
                });
                let (k2, v2) = elem2.unwrap_or_else(|| {
                    let shard = &mut shard1;
                    insert2_p2(&k2, shard)
                });
                dbg!(v1);
                dbg!(v2);
                RefMut {
                    guard1: shard1,
                    guard2: None,
                    k1,
                    k2,
                    v1,
                    v2,
                }
            }
            Entry {
                mut shard1,
                shard2: Some(mut shard2),
                elem1,
                elem2,
                key1,
                key2,
            } => {
                let (r1, r2) = value((elem1.as_ref().map(|_| ()), elem2.as_ref().map(|_| ())));
                // let (k1, v1) = elem1.unwrap_or_else(|| {
                //     let value = r1.expect("some value");
                //     let key = key1;
                //     let shard = &mut shard1;
                //     println!("{:p}", shard);
                //     println!("{:p}", &key);
                //     println!("{}", shard.hasher().hash_one(&key));
                //     insert2(key, shard, value)
                // });
                // let (k2, v2) = elem2.unwrap_or_else(|| {
                //     let value = r2.expect("some value");
                //     let key = key2;
                //     let shard = &mut shard2;
                //     insert2(key, shard, value)
                // });
                let k1 = key1.clone();
                let k2 = key2.clone();
                dbg!(&k1);
                dbg!(&k2);
                println!("{:p}", &k1);
                println!("{:p}", &k2);
                println!("{:p}", &r1);
                println!("{:p}", &r2);
                if elem1.is_none() {
                    let value = r1.expect("some value");
                    dbg!(&value);
                    println!("{:p}", &value);
                    let key = key1;
                    let shard = &mut shard1;
                    insert2_p1_shard(key, shard, value)
                }
                if elem2.is_none() {
                    let value = r2.expect("some value");
                    dbg!(&value);
                    println!("{:p}", &value);
                    let key = key2;
                    let shard = &mut shard2;
                    insert2_p1(key, shard, value)
                }
                let (k1, v1) = elem1.unwrap_or_else(|| {
                    let shard = &mut shard1;
                    dbg!(shard.hasher().hash_one(&k1));
                    let shard: &mut RwLockWriteGuard<HashMap<K, SharedValue<V>>> =
                        unsafe { std::mem::transmute(shard) };
                    insert2_p2_shard(&k1, shard)
                });
                let (k2, v2) = elem2.unwrap_or_else(|| {
                    let shard = &mut shard2;
                    insert2_p2(&k2, shard)
                });
                println!("{:p}", &shard1);
                dbg!(shard1.len());
                println!("{:p}", &shard2);
                dbg!(shard2.len());
                dbg!(v1);
                dbg!(v2);
                RefMut {
                    guard1: shard1,
                    guard2: Some(shard2),
                    k1,
                    k2,
                    v1,
                    v2,
                }
            }
        }
    }
}

fn insert2<'a, K: Eq + Hash, V, S: BuildHasher>(
    key: K,
    shard: &mut RwLockWriteGuard<'a, HashMap<K, V, S>>,
    value: V,
) -> (*const K, *mut V) {
    let c = unsafe { std::ptr::read(&key) };
    shard.insert(key, value);
    // let shard: &mut RwLockWriteGuard<HashMap<K, SharedValue<V>>> =
    //     unsafe { std::mem::transmute(shard) };
    {
        // let shard: &'a mut RwLockWriteGuard<'a, HashMap<K, SharedValue<V>>> = shard;
        unsafe {
            use std::mem;
            dbg!();
            let (k, v) = shard.get_key_value(&c).unwrap();
            dbg!();
            let k = change_lifetime_const(k);
            dbg!();
            let v = &mut *shard_as_ptr2(v);
            dbg!();
            mem::forget(c);
            dbg!();
            (k, v)
        }
    }
}

fn insert2_p1<'a, K: Eq + Hash, V, S: BuildHasher>(
    key: K,
    shard: &mut RwLockWriteGuard<'a, HashMap<K, V, S>>,
    value: V,
) {
    println!("{:p}", &key);
    println!("{}", shard.hasher().hash_one(&key));
    println!("{:p}", &value);
    shard.insert(key, value);
}

fn insert2_p1_shard<'a, K: Eq + Hash, V, S: BuildHasher>(
    key: K,
    shard: &mut RwLockWriteGuard<'a, HashMap<K, V, S>>,
    value: V,
) {
    println!("{:p}", &key);
    println!("{}", shard.hasher().hash_one(&key));
    println!("{:p}", &value);
    // let shard: &mut RwLockWriteGuard<HashMap<K, SharedValue<V>>> =
    //             unsafe { std::mem::transmute(shard) };
    // let value: SharedValue<V> = SharedValue::new(value);
    println!("{:p}", &key);
    println!("{}", shard.hasher().hash_one(&key));
    println!("{:p}", &value);
    // todo!()
    shard.insert(key, value);
}

fn insert2_p2<'a, K: Eq + Hash, V, S: BuildHasher>(
    key: &K,
    shard: &mut RwLockWriteGuard<'a, HashMap<K, V, S>>,
) -> (*const K, *mut V) {
    unsafe {
        use std::mem;
        dbg!();
        println!("{:p}", &key);
        println!("{}", shard.hasher().hash_one(&key));
        let (k, v) = shard.get_key_value(key).unwrap();
        dbg!();
        let k = change_lifetime_const(k);
        dbg!();
        let v = &mut *shard_as_ptr2(v);
        dbg!();
        (k, v)
    }
}

fn insert2_p2_shard<'a, K: Eq + Hash, V, S: BuildHasher>(
    key: &K,
    shard: &mut RwLockWriteGuard<'a, HashMap<K, SharedValue<V>, S>>,
) -> (*const K, *mut V) {
    unsafe {
        dbg!();
        println!("{:p}", &key);
        println!("{}", shard.hasher().hash_one(&key));
        todo!();

        let (k, v) = shard.get_key_value(key).unwrap();
        dbg!();
        let k = change_lifetime_const(k);
        dbg!();
        let v = &mut *shard_as_ptr(v);
        dbg!();
        (k, v)
    }
}

fn insert<'a, K: Eq + Hash, V>(
    key: K,
    shard: &'a mut RwLockWriteGuard<'a, HashMap<K, SharedValue<V>>>,
    value: SharedValue<V>,
) -> (*const K, *mut V) {
    unsafe {
        use std::mem;
        use std::ptr;
        let c: K = ptr::read(&key);
        dbg!();
        println!("{:p}", &key);
        println!("{}", shard.hasher().hash_one(&key));
        println!("{:p}", &value);
        {
            // let shard: &mut RwLockWriteGuard<HashMap<K, V>> =
            //     unsafe { std::mem::transmute(shard) };
            // let value: V =
            //     unsafe { std::mem::transmute(value) };
            shard.insert(key, value);
        }
        dbg!();
        let (k, v) = shard.get_key_value(&c).unwrap();
        dbg!();
        let k = change_lifetime_const(k);
        dbg!();
        let v = &mut *shard_as_ptr(v);
        dbg!();
        mem::forget(c);
        dbg!();
        (k, v)
    }
}

/// # Safety
///
/// Requires that you ensure the reference does not become invalid.
/// The object has to outlive the reference.
unsafe fn change_lifetime_const<'a, 'b, T>(x: &'a T) -> &'b T {
    &*(x as *const T)
}

pub struct RefMut<'a, K, V, S = RandomState> {
    guard1: RwLockWriteGuard<'a, HashMap<K, V, S>>,
    guard2: Option<RwLockWriteGuard<'a, HashMap<K, V, S>>>,
    k1: *const K,
    k2: *const K,
    v1: *mut V,
    v2: *mut V,
}

unsafe impl<'a, K: Eq + Hash + Sync, V: Sync, S: BuildHasher> Send for RefMut<'a, K, V, S> {}
unsafe impl<'a, K: Eq + Hash + Sync, V: Sync, S: BuildHasher> Sync for RefMut<'a, K, V, S> {}

impl<'a, K: Eq + Hash, V, S: BuildHasher> RefMut<'a, K, V, S> {
    pub fn value_mut(&mut self) -> (&mut V, &mut V) {
        unsafe { (&mut *self.v1, &mut *self.v2) }
    }
}
