#![allow(unused)]
use crate::auto::tsq_ser_meta::Converter;
use crate::auto::tsq_transform;
use hyperast::position::position_accessors::{SolvedPosition, WithPreOrderOffsets};
use hyperast::store::defaults::NodeIdentifier;
use hyperast::types::{
    self, HashKind, HyperAST, RoleStore, TypeStore, TypedNodeId, WithHashs, WithSerialization,
    WithStats,
};
use hyperast_tsquery::{Cursor, Node as _};
use num::ToPrimitive;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

// use crate::legion as qgen; // includes identation, such as spaces and new lines
use crate::no_fmt_legion as qgen; // ignores spaces, new lines,...

type QStore = hyperast::store::SimpleStores<crate::types::TStore>;

type IdN = NodeIdentifier;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TR<E = IdN, I = IdNQ> {
    // WARN different NodeIdentifier, this one is referring to the provided examples
    Init(E),
    RMs(I),
    SimpEQ(I),
}

impl<E: PartialEq, I: PartialEq> PartialOrd for TR<E, I> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (TR::Init(_), TR::Init(_)) => Some(Ordering::Equal),
            (TR::Init(_), _) => Some(Ordering::Less),
            (TR::RMs(_), TR::RMs(_)) => Some(Ordering::Equal),
            (TR::RMs(_), _) => Some(Ordering::Less),
            (TR::SimpEQ(_), TR::SimpEQ(_)) => Some(Ordering::Equal),
            (TR::SimpEQ(_), _) => Some(Ordering::Less),
            // _ => self.eq(other).then(|| Ordering::Equal),
        }
    }
}

pub struct QueryLattice<E> {
    pub query_store: QStore,
    leaf_queries: Vec<IdNQ>,
    pub raw_rels: std::collections::HashMap<IdNQ, Vec<TR<E>>>,
    pub queries: Vec<(IdNQ, Vec<IdQ>)>,
    sort_cache: Vec<u32>,
}

/// make the deduplication through raw entries, probably slower, is it marginal ?
#[derive(Default)]
pub struct DedupRawEntry(hashbrown::HashMap<IdNQ, Vec<(IdN, TR)>>);
// type DedupRawEntry = hashbrown::HashMap<IdN, Vec<(IdN, TR)>>;
// TODO directly handle this kind of dedup referentially in the HyperAST
// i.e. putting all the spaces in a wrapper root.
// It requires a special TreeGen that uses the global context, where it accumulates a topologically sorted list of spaces.
impl Deref for DedupRawEntry {
    type Target = hashbrown::HashMap<IdN, Vec<(IdN, TR)>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for DedupRawEntry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub trait Ded {
    fn queries(&self) -> Vec<IdNQ>;
    fn sorted_queries<TS: TypeStore>(
        &self,
        stores: &hyperast::store::SimpleStores<TS>,
    ) -> Vec<IdNQ> {
        let mut v = self.queries();
        v.sort_by_cached_key(|x| WithHashs::hash(&stores.resolve(x), &HashKind::label()));
        v.dedup();
        v
    }
}
impl Ded for DedupRawEntry {
    fn queries(&self) -> Vec<IdNQ> {
        self.0.keys().copied().collect()
    }
}

type IdNQ = IdN;

impl QueryLattice<IdN> {
    pub fn with_examples<TS, TIdN>(
        stores: &hyperast::store::SimpleStores<TS>,
        from: impl Iterator<Item = IdN>,
        meta_gen: &hyperast_tsquery::Query,
        meta_simp: &hyperast_tsquery::Query,
    ) -> Self
    where
        TS: TypeStore + RoleStore,
        TIdN: TypedNodeId<IdN = IdN>,
        TIdN::Ty: types::TypeTrait,
        TS::IdF: From<u16> + Into<u16>,
    {
        let b = Self::builder::<TS, TIdN, _>(stores, from, meta_gen, meta_simp, &|x| {
            x.local.metrics.hashs.label
        });
        let mut b = b.dedup_leaf_queries(|from| {
            // from.into_iter().fold(DedupSimp::new(), |mut acc, x| {
            //     let (from, (query, label_h)) = x;
            //     let v = &mut acc.entry(label_h).or_default();
            //     let x = (query, TR::Init(from));
            //     if !v.contains(&x) {
            //         v.push(x);
            //         // v.sort_by(cmp_lat_entry(&s.query_store))
            //     }
            //     acc
            // })
            from.into_iter()
                .fold(DedupRawEntry::default(), |mut acc, x| {
                    let (from, (query, label_h)) = x;
                    let v = acc.raw_entry_mut().from_hash(label_h as u64, |x| true);
                    let v = match v {
                        hashbrown::hash_map::RawEntryMut::Occupied(occ) => occ.into_key_value().1,
                        hashbrown::hash_map::RawEntryMut::Vacant(vacant) => {
                            vacant
                                .insert_with_hasher(label_h as u64, query, vec![], |query| {
                                    WithHashs::hash(&stores.resolve(query), &HashKind::label())
                                        as u64
                                })
                                .1
                        }
                    };
                    let x = (query, TR::Init(from));
                    if !v.contains(&x) {
                        v.push(x);
                        // v.sort_by(cmp_lat_entry(&s.query_store))
                    }
                    acc
                })
        });
        b.rest0();

        let dedup = &mut b.dedup;
        dedup
            .values_mut()
            .for_each(|x| x.sort_by(cmp_lat_entry(&b.lattice.query_store)));
        dbg!(dedup.len());
        for v in dedup.values() {
            b.lattice.add_raw_rels(v);
        }
        for v in b.dedup.values() {
            b.lattice.queries.push(b.extract(v));
        }
        b.build()
    }

    pub fn count(&self) -> usize {
        self.queries.len()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (IdNQ, &'a [IdQ])> {
        self.sort_cache.iter().filter_map(|i| {
            let (q, e) = &self.queries[*i as usize];
            Some((*q, &e[..]))
        })
    }

    pub fn iter_pretty<'a>(&'a self) -> impl Iterator<Item = (String, &'a [IdQ])> {
        self.sort_cache.iter().filter_map(|i| {
            let (q, e) = &self.queries[*i as usize];
            let q = self.pretty(q);
            if q.is_empty() {
                return None;
            }
            Some((q, &e[..]))
        })
    }

    pub fn pretty(&self, q: &IdNQ) -> String {
        let q = qgen::PP::<_, _>::new(&self.query_store, *q)
            // let q = hyperast::nodes::TextSerializer::<_, _>::new(&self.query_store, *q)
            // .to_string()
            // .trim()
            // .lines()
            // .filter(|x| !x.trim().is_empty())
            // .map(|x| x.to_string() + "\n---\n")
            // .collect::<String>()
        ;
        format!("{}", q)
    }

    pub fn get_query(&self, index: usize) -> Option<(String, &[IdQ])> {
        self.queries
            .get(self.sort_cache[index] as usize)
            .and_then(|(q, e)| {
                let q = hyperast::nodes::TextSerializer::<_, _>::new(&self.query_store, *q)
                    .to_string()
                    .trim()
                    .lines()
                    .filter(|x| !x.trim().is_empty())
                    .map(|x| x.to_string() + "\n")
                    .collect::<String>();
                if q.is_empty() {
                    None
                } else {
                    Some((q, &e[..]))
                }
            })
    }

    pub fn sort_by_size(&mut self) {
        if self.sort_cache.is_empty() && !self.queries.is_empty() {
            self.sort_cache = (0..self.queries.len())
                .map(|x| x.to_u32().unwrap())
                .collect();
        }
        self.sort_cache.sort_by(|a, b| {
            (self.queries[*a as usize].1.len()).cmp(&self.queries[*b as usize].1.len())
        });
    }

    pub fn builder<'q, TS, TIdN, T>(
        stores: &hyperast::store::SimpleStores<TS>,
        from: impl Iterator<Item = IdN>,
        meta_gen: &'q hyperast_tsquery::Query,
        meta_simp: &'q hyperast_tsquery::Query,
        f: &impl Fn(qgen::FNode) -> T,
    ) -> Builder<'q, IdN, Vec<(IdN, (IdNQ, T))>>
    where
        TS: TypeStore + RoleStore,
        TIdN: TypedNodeId<IdN = IdN>,
        TIdN::Ty: types::TypeTrait,
        TS::IdF: From<u16> + Into<u16>,
    {
        let mut s = Self::new();
        // TODO do not use u32 but the entry_raw and compute the hash on the fly
        let mut md_cache = Default::default();
        let dedup = from
            .filter_map(|from| {
                // TODO add variant with immediates
                let x = generate_query_aux::<TS, TIdN, _>(
                    &mut s.query_store,
                    &mut md_cache,
                    stores,
                    from,
                    meta_gen,
                    f,
                )?;
                // TODO generate multiple initial variants, by adding common meta rules
                if !simp_search_need(&s.query_store, x.0, meta_simp) {
                    return None;
                }
                Some((from, x))
            })
            .collect();
        Builder {
            lattice: s,
            dedup,
            meta_simp,
        }
    }

    /// Similar to with_examples,
    /// but processes queries from the biggest to the smallest,
    /// thus we can use a kind of vec of maps and paralelize inserts in the maps.
    #[cfg(feature = "synth_par")]
    pub fn with_examples_by_size<TS, TIdN>(
        stores: &hyperast::store::SimpleStores<TS>,
        from: impl Iterator<Item = IdN>,
        meta_gen: &hyperast_tsquery::Query,
        meta_simp: &hyperast_tsquery::Query,
    ) -> Self
    where
        TS: TypeStore + RoleStore,
        TIdN: TypedNodeId<IdN = IdN>,
        TIdN::Ty: types::TypeTrait,
        TS::IdF: From<u16> + Into<u16>,
    {
        let b = Self::builder::<TS, TIdN, _>(stores, from, meta_gen, meta_simp, &|x| {
            // TODO use size ignoring spaces
            (x.local.metrics.size, x.local.metrics.hashs.label)
        });
        let mut b = b.dedup_leaf_queries(|from: Vec<(_, (_, (u32, u32)))>| group_by_size(from));
        b.loop_par();
        b.post();
        b.lattice.sort_by_size();
        b.build()
    }

    /// Similar to with_examples_by_size,
    /// but tries to merge simplifications in parallel through a shared ref to query store
    /// then merges requiring additional subtrees are merged sequentially.
    #[cfg(feature = "synth_par")]
    pub fn with_examples_by_size_try<TS, TIdN>(
        stores: &hyperast::store::SimpleStores<TS>,
        from: impl Iterator<Item = IdN>,
        meta_gen: &hyperast_tsquery::Query,
        meta_simp: &hyperast_tsquery::Query,
    ) -> Self
    where
        TS: TypeStore + RoleStore,
        TIdN: TypedNodeId<IdN = IdN>,
        TIdN::Ty: types::TypeTrait,
        TS::IdF: From<u16> + Into<u16>,
    {
        let b = Self::builder::<TS, TIdN, _>(stores, from, meta_gen, meta_simp, &|x| {
            // TODO use size ignoring spaces
            (x.local.metrics.size, x.local.metrics.hashs.label)
        });
        let mut b = b.dedup_leaf_queries(|from: Vec<(_, (_, (u32, u32)))>| group_by_size(from));
        b.loop_par_par();
        b.post();
        b.lattice.sort_by_size();
        b.build()
    }
}

#[derive(Default)]
pub struct DedupBySize(Vec<std::collections::HashMap<u32, Vec<(IdNQ, TR)>>>);

impl Deref for DedupBySize {
    type Target = Vec<std::collections::HashMap<u32, Vec<(IdNQ, TR)>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for DedupBySize {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Ded for DedupBySize {
    fn queries(&self) -> Vec<IdNQ> {
        self.0
            .iter()
            .flat_map(|x| x.values().flat_map(|v| v.iter().map(|x| x.0)))
            .collect()
    }
}

#[cfg(feature = "synth_par")]
pub fn group_by_size_old(from: Vec<(IdN, (IdNQ, (u32, u32)))>) -> DedupBySize {
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    DedupBySize(
        from.into_iter()
            // grouping on size of queries (i.e. number of nodes)
            .fold(Vec::<Vec<(IdN, (IdNQ, u32))>>::new(), |mut acc, x| {
                let (fr, (query, (size, label_h))) = x;
                let size = size as usize;
                if size >= acc.len() {
                    acc.resize(size + 1, Default::default());
                }
                acc[size].push((fr, (query, label_h)));
                acc
            })
            .into_par_iter()
            .map(|acc| {
                // grouping by hash ignoring indentation and spaces (i.e., label hash)
                acc.into_iter().fold(
                    std::collections::HashMap::<u32, Vec<(IdNQ, TR)>>::new(),
                    |mut acc, x| {
                        let (from, (query, label_h)) = x;
                        let v = &mut acc.entry(label_h).or_default();
                        let x = (query, TR::Init(from));
                        if !v.contains(&x) {
                            v.push(x);
                            // v.sort_by(cmp_lat_entry(&s.query_store))
                        }
                        acc
                    },
                )
            })
            .collect::<Vec<_>>(),
    )
}

#[derive(Default)]
pub struct DedupBySize2(Vec<std::collections::HashMap<IdNQ, Vec<TR>>>);

impl Deref for DedupBySize2 {
    type Target = Vec<std::collections::HashMap<IdNQ, Vec<TR>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for DedupBySize2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Ded for DedupBySize2 {
    fn queries(&self) -> Vec<IdNQ> {
        self.0
            .iter()
            // .flat_map(|x| x.values().flat_map(|v| v.iter().map(|x| x)))
            .flat_map(|x| x.keys().copied())
            .collect()
    }
}

#[cfg(feature = "synth_par")]
pub fn group_by_size(from: Vec<(IdN, (IdNQ, (u32, u32)))>) -> DedupBySize2 {
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    DedupBySize2(
        from.into_iter()
            // grouping on size of queries (i.e. number of nodes)
            .fold(Vec::<Vec<(IdN, (IdNQ, u32))>>::new(), |mut acc, x| {
                let (fr, (query, (size, label_h))) = x;
                let size = size as usize;
                if size >= acc.len() {
                    acc.resize(size + 1, Default::default());
                }
                acc[size].push((fr, (query, label_h)));
                acc
            })
            .into_par_iter()
            .map(|acc| {
                // grouping by hash ignoring indentation and spaces (i.e., label hash)
                acc.into_iter().fold(
                    std::collections::HashMap::<IdNQ, Vec<TR>>::new(),
                    |mut acc, x| {
                        let (from, (query, label_h)) = x;
                        let v = &mut acc.entry(query).or_default();
                        dbg!(query);
                        let x = (query, TR::Init(from));
                        if !v.contains(&x.1) {
                            v.push(x.1);
                            // v.sort_by(cmp_lat_entry(&s.query_store))
                        }
                        acc
                    },
                )
            })
            .collect::<Vec<_>>(),
    )
}

type IdQ = u32;

pub struct Builder<'q, E, D = DedupBySize2> {
    pub lattice: QueryLattice<E>,
    pub dedup: D,
    pub meta_simp: &'q hyperast_tsquery::Query,
}

impl<'q, E, D> Builder<'q, E, D> {
    pub fn build(self) -> QueryLattice<E> {
        self.lattice
    }
    pub fn dedup_leaf_queries<D2: Ded>(self, f: impl Fn(D) -> D2) -> Builder<'q, E, D2> {
        let mut b = Builder {
            lattice: self.lattice,
            dedup: f(self.dedup),
            meta_simp: self.meta_simp,
        };
        b.lattice.leaf_queries = b.dedup.queries();
        b.lattice.leaf_queries.sort_by_cached_key(|x| {
            WithHashs::hash(&b.lattice.query_store.resolve(x), &HashKind::label())
        });
        b.lattice.leaf_queries.dedup();
        b
    }
}

impl Builder<'_, IdN, DedupRawEntry> {
    fn rest0(&mut self) {
        let s = &mut self.lattice;
        let dedup = &mut self.dedup;
        let meta_simp = self.meta_simp;
        let mut active: Vec<IdNQ> = dedup
            .keys()
            .copied()
            .filter(|x| {
                simp_search_need(
                    &s.query_store,
                    dedup
                        .raw_entry()
                        .from_hash(
                            WithHashs::hash(&s.query_store.resolve(x), &HashKind::label()) as u64,
                            |y| true,
                        )
                        .unwrap()
                        .1[0]
                        .0,
                    meta_simp,
                )
            })
            .collect();

        for _ in 0..4 {
            dbg!(active.len());
            let rms = std::mem::take(&mut active)
                .into_iter()
                .flat_map(|x| {
                    let Some((_, x)) = dedup.raw_entry().from_hash(
                        WithHashs::hash(&s.query_store.resolve(&x), &HashKind::label()) as u64,
                        |x| true,
                    ) else {
                        return vec![];
                    };
                    let query = x[0].0;
                    simp_rms2(&mut s.query_store, query, meta_simp)
                        .map(|(new_q, label_h)| (label_h, (new_q, TR::RMs(query))))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            dbg!(rms.len());
            for (label_h, x) in rms {
                let v = dedup.raw_entry_mut().from_hash(label_h as u64, |x| true);
                let v = match v {
                    hashbrown::hash_map::RawEntryMut::Occupied(occ) => occ.into_key_value().1,
                    hashbrown::hash_map::RawEntryMut::Vacant(vacant) => {
                        active.push(x.0);
                        vacant
                            .insert_with_hasher(label_h as u64, x.0, vec![], |query| {
                                WithHashs::hash(&s.query_store.resolve(query), &HashKind::label())
                                    as u64
                            })
                            .1
                    }
                };
                if !v.contains(&x) {
                    v.push(x);
                } else {
                    dbg!()
                }
            }
            // TODO add pass to replace some symbols with a wildcard
        }
        dbg!(dedup.0.len());
        let simp_eq = dedup
            .0
            .values()
            .filter_map(|x| {
                let query = x[0].0;
                let (new_q, label_h) = simp_imm_eq(&mut s.query_store, query, meta_simp)?;
                Some((label_h, (new_q, TR::RMs(query))))
            })
            .collect::<Vec<_>>();

        for (label_h, x) in simp_eq {
            let v = dedup.0.raw_entry_mut().from_hash(label_h as u64, |x| true);
            let v = match v {
                hashbrown::hash_map::RawEntryMut::Occupied(occ) => occ.into_key_value().1,
                hashbrown::hash_map::RawEntryMut::Vacant(vacant) => {
                    active.push(x.0);
                    vacant
                        .insert_with_hasher(label_h as u64, x.0, vec![], |query| {
                            WithHashs::hash(&s.query_store.resolve(query), &HashKind::label())
                                as u64
                        })
                        .1
                }
            };
            if !v.contains(&x) {
                v.push(x);
            }
        }
        dbg!(dedup.0.len());
    }
}

// the parallel implementations
impl Builder<'_, IdN> {
    #[cfg(feature = "synth_par")]
    fn loop_par(&mut self) {
        let mut active_size = self.dedup.0.len() - 1;
        // eprintln!("{active_size}: {}", pp_dedup(&self.dedup.0, active_size));
        let mut active: Vec<_> = self.actives(active_size);

        loop {
            dbg!(active_size);
            dbg!(active.len());
            // eprintln!("{}", pp_dedup(&self.dedup.0, active_size));
            // TODO add pass to replace some symbols with a wildcard
            let rms = self.removes_par(active_size, &mut active);
            dbg!(rms.len());
            self.dedup_removes_par(active_size, &mut active, rms);
            if self.between(&mut active_size, &mut active) {
                break;
            }
        }
    }

    /// Must use dedup_removes_par on output to properly progress
    #[must_use]
    #[cfg(feature = "synth_par")]
    fn removes_par(&mut self, active_size: usize, active: &mut Vec<IdNQ>) -> Vec<(u32, (IdN, TR))> {
        let s = &mut self.lattice;
        let dedup = &mut self.dedup;
        let meta_simp = self.meta_simp;
        let rms = std::mem::take(active).into_iter();
        let rms = rms
            .flat_map(|x| {
                let Some(y) = dedup.0[active_size].get(&x) else {
                    return vec![];
                };
                let query = x;
                // let query = y[0].0;
                simp_rms2(&mut s.query_store, query, meta_simp)
                    .map(|(new_q, label_h)| (label_h, (new_q, TR::RMs(query))))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        rms
    }

    #[cfg(feature = "synth_par")]
    pub fn dedup_removes_par(
        &mut self,
        active_size: usize,
        active: &mut Vec<IdNQ>,
        rms: Vec<(u32, (legion::Entity, TR))>,
    ) {
        use rayon::iter::{
            IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator,
            ParallelIterator,
        };
        let s = &mut self.lattice;
        let dedup = &mut self.dedup;
        let fold = ParallelIterator::fold(
            rms.into_par_iter(),
            || BTreeMap::default(),
            |mut acc: BTreeMap<usize, Vec<_>>, (label_h, x): (u32, (IdN, TR))| {
                let size = s.query_store.resolve(&x.0).size();
                acc.entry(size).or_default().push((label_h, x));
                acc
            },
        );
        let aaa: BTreeMap<usize, Vec<(LabelH, (IdN, TR))>> = ParallelIterator::reduce(
            fold,
            || BTreeMap::<usize, Vec<(LabelH, (IdN, TR))>>::default(),
            |mut acc, b| {
                for (size, v) in b {
                    acc.entry(size).or_default().extend(v);
                }
                acc
            },
        );
        let aaa = aaa
            .into_iter()
            .fold(vec![vec![]; active_size], |mut acc, x| {
                acc[x.0] = x.1;
                acc
            });
        let act: Vec<_> = ParallelIterator::flat_map(
            dedup.0[..active_size].par_iter_mut().enumerate(),
            |(i, dedup)| {
                let mut r = vec![];
                for (label_h, x) in &aaa[i] {
                    let v = dedup.entry(x.0);
                    let v = match v {
                        std::collections::hash_map::Entry::Occupied(x) => x.into_mut(),
                        std::collections::hash_map::Entry::Vacant(x) => {
                            r.push(*x.key());
                            x.insert(vec![])
                        }
                    };
                    if !v.contains(&x.1) {
                        v.push(x.1);
                    } else {
                    }
                }
                r
            },
        )
        .collect();
        active.extend(act);
    }

    #[cfg(feature = "synth_par")]
    pub fn post(&mut self) {
        use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
        self.dedup.0.par_iter_mut().for_each(|x| {
            dbg!(x.len());
            let qstores = &self.lattice.query_store;
            for y in x.values_mut() {
                // y.sort_by(cmp_lat_entry(&qstores))
            }
        });
        for v in self.dedup.iter().flat_map(|x| x.iter()) {
            self.lattice.add_raw_rels2(*v.0, &v.1);
        }
        // eprintln!("final: {}", pp_dedup(&self.dedup, self.dedup.len() - 1));
        for v in self.dedup.iter().flat_map(|x| x.iter()) {
            let value = self.extract2(*v.0, &v.1);
            dbg!(&value);
            self.lattice.queries.push(value);
        }
    }

    pub fn actives(&mut self, active_size: usize) -> Vec<IdN> {
        self.dedup[active_size]
            .keys()
            .copied()
            .filter(|x| {
                // simp_search_need(
                //     &self.lattice.query_store,
                //     self.dedup[active_size].get(&x).unwrap()[0].0,
                //     self.meta_simp,
                // )
                simp_search_need(&self.lattice.query_store, *x, self.meta_simp)
            })
            .collect()
    }

    #[cfg(feature = "synth_par")]
    pub fn loop_par_par(&mut self) {
        let mut active_size = self.dedup.len() - 1;
        // eprintln!("{active_size}: {}", pp_dedup(&self.dedup, active_size));
        let mut active: Vec<_> = self.actives(active_size);
        loop {
            dbg!(active_size);
            dbg!(active.len());
            // eprintln!("{}", pp_dedup(&self.dedup, active_size));
            let rms = self.removes_par_par(active_size, &mut active);
            dbg!(rms.len());
            self.dedup_removes_par(active_size, &mut active, rms);
            if self.between(&mut active_size, &mut active) {
                break;
            }
        }
    }

    pub fn between(&mut self, active_size: &mut usize, active: &mut Vec<IdNQ>) -> bool {
        if !active.is_empty() {
            return false;
        }
        loop {
            if *active_size == 0 {
                return true;
            }
            *active_size -= 1;
            if !self.dedup[*active_size].is_empty() {
                break;
            }
        }
        *active = self.actives(*active_size);
        false
    }

    /// Must use dedup_removes_par on output to properly progress
    #[must_use]
    #[cfg(feature = "synth_par")]
    pub fn removes_par_par(
        &mut self,
        active_size: usize,
        active: &mut Vec<IdNQ>,
    ) -> Vec<(u32, (IdN, TR))> {
        use rayon::iter::{
            IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator,
            ParallelIterator,
        };
        let s = &mut self.lattice;
        let dedup = &mut self.dedup;
        let meta_simp = self.meta_simp;
        let rms = std::mem::take(active).into_par_iter();
        let rms = ParallelIterator::flat_map(rms, |x| {
            let Some(y) = dedup[active_size].get(&x) else {
                return vec![];
            };
            let query = x;
            // let query = y[0].0;
            try_simp_rms2(&s.query_store, query, meta_simp)
                .map(|x| match x {
                    Ok((new_q, label_h)) => Ok((label_h, (new_q, TR::RMs(query)))),
                    Err(e) => Err(e),
                })
                .collect::<Vec<_>>()
        });
        let (remains, mut rms): (Vec<(IdN, Vec<u16>)>, Vec<(u32, (IdN, TR))>) =
            ParallelIterator::partition_map(rms, |x| x.into());
        log::info!("remains: {}", remains.len());
        log::info!("rms: {}", rms.len());

        rms.extend(remains.into_iter().filter_map(|(query, path)| {
            let label_h;
            let new_q = {
                let query = apply_rms_aux(&mut s.query_store, query, &path)?;
                if !simp_search_need(&s.query_store, query, meta_simp) {
                    return None;
                }
                label_h = s.query_store.resolve(&query).hash(&HashKind::label());
                query
            };
            Some((label_h, (new_q, TR::RMs(query))))
        }));
        rms
    }

    /// WIP there are limitations related to combinatorial cases (i.e., multiple EQ checking for the same value)
    fn simp_eq() {
        // let simp_eq = dedup
        //     .iter()
        //     .enumerate()
        //     .flat_map(|(i, x)| x.values().map(move |v| (i, v)))
        //     .filter_map(|(i, x)| {
        //         let query = x[0].0;
        //         let (new_q, label_h) = simp_imm_eq(&mut s.query_store, query, meta_simp)?;
        //         Some((i, label_h, (new_q, TR::RMs(query))))
        //     })
        //     .collect::<Vec<_>>();
        // for (i, label_h, x) in simp_eq {
        //     let v = dedup[i].entry(label_h).or_default();
        //     if !v.contains(&x) {
        //         v.push(x);
        //         sort!(v);
        //     }
        // }
        // dbg!(dedup.len());
        // for v in dedup.iter().flat_map(|x| x.values()) {
        //     for v in v {
        //         let w = s.raw_rels.entry(v.0).or_default();
        //         w.push(v.1);
        //     }
        // }
    }
}

impl<E: Copy> QueryLattice<E> {
    fn add_raw_rels(&mut self, v: &Vec<(IdN, TR<E>)>) {
        for v in v {
            self.raw_rels.entry(v.0).or_default().push(v.1);
        }
    }

    fn add_raw_rels2(&mut self, k: IdN, v: &Vec<TR<E>>) {
        for v in v {
            self.raw_rels.entry(k).or_default().push(*v);
        }
    }

    pub fn leaf(&self, id: IdQ) -> IdNQ {
        self.leaf_queries[id as usize]
    }
}

impl<'q, D> Builder<'q, IdN, D> {
    fn extract(&self, v: &[(IdNQ, TR)]) -> (IdNQ, Vec<IdQ>) {
        fn extract<'a>(
            map: &'a std::collections::HashMap<IdNQ, Vec<TR>>,
            curr: IdNQ,
            downs: impl Iterator<Item = &'a TR>,
            already: &mut HashSet<IdNQ>,
            r: &mut Vec<IdNQ>,
            leafs: &[IdNQ],
        ) {
            for s in downs {
                match s {
                    TR::Init(_) if !r.contains(&curr) => {
                        assert!(leafs.contains(&curr), "{curr:?}");
                        r.push(curr)
                    }
                    TR::RMs(v) | TR::SimpEQ(v) if !already.contains(v) => {
                        already.insert(*v);
                        extract(map, *v, map.get(v).unwrap().iter(), already, r, leafs)
                    }
                    _ => (),
                }
            }
        }
        let mut already = HashSet::default();
        let mut r = vec![];
        extract(
            &self.lattice.raw_rels,
            v[0].0,
            v.into_iter().map(|x| &x.1),
            &mut already,
            &mut r,
            &self.lattice.leaf_queries,
        );
        let r = r
            .into_iter()
            .map(|x| {
                dbg!(x);
                dbg!(&self.lattice.leaf_queries);
                dbg!(
                    (self.lattice.raw_rels.get(&x).unwrap().iter())
                        .position(|x| matches!(x, TR::Init(_)))
                );
                self.lattice
                    .leaf_queries
                    .iter()
                    .position(|y| x == *y)
                    .unwrap() as u32
            })
            .collect();
        (v[0].0, r)
    }
    fn extract2(&self, k: IdNQ, v: &[TR]) -> (IdNQ, Vec<IdQ>) {
        fn extract<'a>(
            map: &'a std::collections::HashMap<IdNQ, Vec<TR>>,
            curr: IdNQ,
            downs: impl Iterator<Item = &'a TR>,
            already: &mut HashSet<IdNQ>,
            r: &mut Vec<IdNQ>,
            leafs: &[IdNQ],
        ) {
            for s in downs {
                match s {
                    TR::Init(_) if !r.contains(&curr) => {
                        assert!(leafs.contains(&curr), "{curr:?}");
                        r.push(curr)
                    }
                    TR::RMs(v) | TR::SimpEQ(v) if !already.contains(v) => {
                        already.insert(*v);
                        extract(map, *v, map.get(v).unwrap().iter(), already, r, leafs)
                    }
                    _ => (),
                }
            }
        }
        let mut already = HashSet::default();
        let mut r = vec![];
        extract(
            &self.lattice.raw_rels,
            // v[0].0,
            k,
            v.into_iter().map(|x| x),
            &mut already,
            &mut r,
            &self.lattice.leaf_queries,
        );
        let r = r
            .into_iter()
            .map(|x| {
                self.lattice
                    .leaf_queries
                    .iter()
                    .position(|y| x == *y)
                    .unwrap() as u32
            })
            .collect();
        (k, r)
    }
}

fn cmp_lat_entry<TS: TypeStore + RoleStore, T: PartialOrd>(
    stores: &hyperast::store::SimpleStores<TS>,
) -> impl Fn(&(IdN, T), &(IdN, T)) -> Ordering {
    |a, b| {
        let tr = a.1.partial_cmp(&b.1);
        if tr != Some(Ordering::Equal) {
            return tr.unwrap();
        }
        let a_l = stores
            .node_store()
            .resolve(a.0)
            .try_bytes_len()
            .unwrap_or_default();
        let b_l = stores
            .node_store()
            .resolve(b.0)
            .try_bytes_len()
            .unwrap_or_default();
        let l_ord = a_l.cmp(&b_l);
        l_ord
    }
}

pub fn pp_dedup<E, E2>(
    dedup: &Vec<std::collections::HashMap<u32, Vec<(E, TR<E2>)>>>,
    active_size: usize,
) -> String {
    dedup[..active_size + 1]
        .iter()
        .fold(Vec::<Result<usize, usize>>::new(), |mut acc, x| {
            if x.is_empty() {
                if acc.last().map_or(false, |x| x.is_err()) {
                    if let Err(x) = acc.last_mut().unwrap() {
                        *x += 1;
                    }
                } else {
                    acc.push(Err(1));
                }
            } else {
                acc.push(Ok(x.len()));
            }
            acc
        })
        .into_iter()
        .map(|x| match x {
            Ok(x) => format!("{x}"),
            Err(x) => format!("{x}x0"),
        })
        .collect::<Vec<_>>()
        .join(",")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QueryId(
    // even if I use just a NodeIdentifier, queries are dedup early
    NodeIdentifier,
);

impl<E> QueryLattice<E> {
    pub fn new() -> Self {
        Self {
            query_store: crate::search::ts_query_store(),
            leaf_queries: vec![],
            queries: vec![],
            raw_rels: Default::default(),
            sort_cache: Default::default(),
        }
    }
}

impl<E> Default for QueryLattice<E> {
    fn default() -> Self {
        Self::new()
    }
}

fn generate_query<TS: TypeStore + RoleStore, TIdN: TypedNodeId<IdN = NodeIdentifier>>(
    query_store: &mut QStore,
    // stores: &JStore,
    stores: &hyperast::store::SimpleStores<TS>,
    from: NodeIdentifier,
) -> NodeIdentifier
where
    for<'t> TIdN::Ty: From<&'t str>,
    TIdN::Ty: types::TypeTrait,
    TS::IdF: From<u16> + Into<u16>,
{
    struct Conv<Ty>(PhantomData<Ty>);

    impl<Ty> Default for Conv<Ty> {
        fn default() -> Self {
            Self(Default::default())
        }
    }
    impl<Ty: for<'t> From<&'t str>> Converter for Conv<Ty> {
        type Ty = Ty;

        fn conv(s: &str) -> Option<Self::Ty> {
            Some(Ty::from(s))
        }
    }
    let _query = crate::auto::tsq_ser_meta::TreeToQuery::<_, TIdN, Conv<TIdN::Ty>>::with_pred(
        stores,
        from,
        "(identifier) (type_identifier)",
    );
    let _query = _query.to_string();
    let (mut query_store, query) = crate::search::ts_query(_query.as_bytes());
    const M0: &str = r#"(predicate (identifier) @op (#eq? @op "eq") (parameters (capture (identifier) @id ) (string) @label ))"#;
    println!("");
    println!("\nThe meta query:\n{}", M0);
    let (query_store1, query1) = crate::search::ts_query(M0.as_bytes());
    let path = hyperast::position::structural_pos::StructuralPosition::new(query);
    let prepared_matcher =
        crate::search::PreparedMatcher::<crate::types::Type>::new(&query_store1, query1);
    let mut per_label = std::collections::HashMap::<
        String,
        Vec<(
            String,
            hyperast::position::structural_pos::StructuralPosition<NodeIdentifier, u16>,
        )>,
    >::default();
    for e in crate::iter::IterAll::new(&query_store, path, query) {
        if let Some(capts) = prepared_matcher
            .is_matching_and_capture::<_, crate::types::TIdN<NodeIdentifier>>(
                &query_store,
                e.node(),
            )
        {
            dbg!(&capts);
            let l_l = prepared_matcher
                .captures
                .iter()
                .position(|x| &x.name == "label")
                .unwrap() as u32;
            let l_i = prepared_matcher
                .captures
                .iter()
                .position(|x| &x.name == "id")
                .unwrap() as u32;
            let k = capts
                .by_capture_id(l_l)
                .unwrap()
                .clone()
                .try_label(&query_store)
                .unwrap();
            let v = capts
                .by_capture_id(l_i)
                .unwrap()
                .clone()
                .try_label(&query_store)
                .unwrap();
            let p = e;
            per_label
                .entry(k.to_string())
                .or_insert(vec![])
                .push((v.to_string(), p));
        }
    }
    dbg!(&per_label);
    let query_bis = tsq_transform::regen_query(
        &mut query_store,
        query,
        per_label
            .values()
            .filter(|l| l.len() == 2)
            .flatten()
            .map(|x| tsq_transform::Action::Delete {
                path: x.1.iter_offsets().collect(),
            })
            .collect(),
    );
    let query =
        hyperast::nodes::TextSerializer::<_, _>::new(&query_store, query_bis.unwrap()).to_string();
    let query = format!("{} {}", query, PerLabel(per_label.clone()));
    println!("\nThe generified query:\n{}", query);
    let query = crate::search::ts_query2(&mut query_store, query.as_bytes());
    query
}

type LabelH = u32;

fn simp_imm_eq(
    query_store: &mut hyperast::store::SimpleStores<crate::types::TStore>,
    query: NodeIdentifier,
    meta_simp: &hyperast_tsquery::Query,
) -> Option<(NodeIdentifier, LabelH)> {
    // merge immediate predicates with identical labels
    let mut per_label = simp_search_imm_preds(&query_store, query, meta_simp);
    // dbg!(&per_label);
    let query = replace_preds_with_caps(query_store, query, &mut per_label);
    let query = hyperast::nodes::TextSerializer::<_, _>::new(&*query_store, query?).to_string();
    // TODO pretty print
    // NOTE hyperast::nodes::PrettyPrinter is not specifica enough to do a proper pp
    // print issue after removing something is due to having consecutive space nodes,
    // best would be to keep the one with a newline or else the first.

    let query = format!("{} {}", query, PerLabel(per_label));
    println!("\nThe generified query:\n{}", query);
    crate::search::ts_query2_with_label_hash(query_store, query.as_bytes())
}

/// remove a matched thing from query
fn simp_rms2<'a>(
    query_store: &'a mut hyperast::store::SimpleStores<crate::types::TStore>,
    query: NodeIdentifier,
    meta_simp: &'a hyperast_tsquery::Query,
) -> impl Iterator<Item = (NodeIdentifier, LabelH)> + 'a {
    let rms = simp_search_rm(&query_store, query, meta_simp);
    rms.into_iter().filter_map(move |path| {
        let query = apply_rms_aux(query_store, query, &path)?;
        if !simp_search_need(&query_store, query, meta_simp) {
            return None;
        }
        // let query = hyperast::nodes::TextSerializer::<_, _>::new(&*query_store, query).to_string();
        // crate::search::ts_query2_with_label_hash(query_store, query.as_bytes())
        Some((query, query_store.resolve(&query).hash(&HashKind::label())))
    })
}

fn try_simp_rms2<'a>(
    query_store: &'a hyperast::store::SimpleStores<crate::types::TStore>,
    query: NodeIdentifier,
    meta_simp: &'a hyperast_tsquery::Query,
) -> impl Iterator<Item = Result<(NodeIdentifier, LabelH), (NodeIdentifier, Vec<u16>)>> + 'a {
    let rms = simp_search_rm(&query_store, query, meta_simp);
    rms.into_iter().filter_map(move |path| {
        let Some(query) = try_apply_rms_aux(query_store, query, &path) else {
            return Some(Err((query, path)));
        };
        if !simp_search_need(&query_store, query, meta_simp) {
            return None;
        }
        Some(Ok((
            query,
            query_store.resolve(&query).hash(&HashKind::label()),
        )))
    })
}

fn generate_query_aux<TS: TypeStore + RoleStore, TIdN: TypedNodeId<IdN = NodeIdentifier>, T>(
    query_store: &mut QStore,
    md_cache: &mut qgen::MDCache,
    stores: &hyperast::store::SimpleStores<TS>,
    from: NodeIdentifier,
    meta_gen: &hyperast_tsquery::Query,
    f: &impl Fn(qgen::FNode) -> T,
) -> Option<(NodeIdentifier, T)>
where
    TIdN::Ty: types::TypeTrait,
    TS::IdF: From<u16> + Into<u16>,
{
    use crate::auto::tsq_ser_meta2::TreeToQuery;
    let query = TreeToQuery::<_, TIdN>::new(stores, from, meta_gen.clone());
    let query = format!("{} @_root", query);
    let text = query.as_bytes();
    let mut query_tree_gen = qgen::TsQueryTreeGen::new(query_store, md_cache);
    let tree = match crate::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => {
            log::warn!("Error parsing query: {}", t.root_node().to_sexp());
            return None;
        }
    };
    let full_node = query_tree_gen.generate_file(b"", text, tree.walk());
    let r = (full_node.local.compressed_node, f(full_node));
    Some(r)
}

fn simp_search_atleast(
    query_store: &Store,
    query: NodeIdentifier,
    meta_simp: &hyperast_tsquery::Query,
) -> bool {
    let Some(cid) = meta_simp.capture_index_for_name("atleast") else {
        return true;
    };
    let pos = hyperast::position::structural_pos::CursorWithPersistance::new(query);
    let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(query_store, pos);
    let mut matches = meta_simp.matches(cursor);
    // at least one match
    loop {
        let Some(m) = matches.next() else {
            return false;
        };
        if m.nodes_for_capture_index(cid).next().is_some() {
            return true;
        }
    }
}

fn simp_search_need(
    query_store: &Store,
    query: NodeIdentifier,
    meta_simp: &hyperast_tsquery::Query,
) -> bool {
    let Some(cid) = meta_simp.capture_index_for_name("need") else {
        return true;
    };
    let pos = hyperast::position::structural_pos::CursorWithPersistance::new(query);
    let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(query_store, pos);
    let mut matches = meta_simp.matches(cursor);
    let mut bs = bitvec::bitvec!(0;meta_simp.pattern_count());
    meta_simp
        .quants(cid)
        .for_each(|i| bs.set(i.to_usize(), true));
    loop {
        let Some(m) = matches.next() else {
            return bs.count_ones() == 0;
        };
        if m.nodes_for_capture_index(cid).next().is_some() {
            bs.set(m.pattern_index.to_usize(), false);
        }
    }
}

fn simp_search_uniq(
    query_store: &Store,
    query: NodeIdentifier,
    meta_simp: &hyperast_tsquery::Query,
) -> bool {
    let cid = meta_simp.capture_index_for_name("uniq");
    let Some(cid) = cid else {
        return true;
    };
    let pos = hyperast::position::structural_pos::CursorWithPersistance::new(query);
    let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(query_store, pos);
    let mut matches = meta_simp.matches(cursor);
    // exactly one match, unique
    let mut found = false;
    loop {
        let Some(m) = matches.next() else {
            return found;
        };
        if m.nodes_for_capture_index(cid).next().is_some() {
            if found {
                return false;
            }
            found = true;
        }
    }
}

fn simp_search_need2(
    query_store: &Store,
    query: NodeIdentifier,
    meta_simp: &hyperast_tsquery::Query,
) -> bool {
    let need = meta_simp.capture_index_for_name("need");
    let uniq = meta_simp.capture_index_for_name("uniq");
    if need.is_none() && uniq.is_none() {
        return true;
    };
    let pos = hyperast::position::structural_pos::CursorWithPersistance::new(query);
    let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(query_store, pos);
    let mut matches = meta_simp.matches(cursor);
    // // at least one match
    // loop {
    //     let Some(m) = matches.next() else {
    //         return false;
    //     };
    //     if m.nodes_for_capture_index(cid).next().is_some() {
    //         return true;
    //     }
    // }
    // // exactly one match, unique
    // let mut found = false;
    // loop {
    //     let Some(m) = matches.next() else {
    //         return found;
    //     };
    //     if m.nodes_for_capture_index(cid).next().is_some() {
    //         if found {
    //             return false;
    //         }
    //         found = true;
    //     }
    // }
    // both
    let mut found = false;
    let mut found_need = false;
    loop {
        let Some(m) = matches.next() else {
            return found;
        };
        if let Some(_cid) = uniq {}
        if let Some(cid) = need {
            let q = meta_simp.quant(m.pattern_index, cid);
            if matches!(
                q,
                hyperast_tsquery::CaptureQuantifier::One
                    | hyperast_tsquery::CaptureQuantifier::OneOrMore
            ) {}
        }

        let mut need = need.iter().flat_map(|cid| m.nodes_for_capture_index(*cid));
        let mut uniq = uniq.iter().flat_map(|cid| m.nodes_for_capture_index(*cid));
        loop {
            if uniq.next().is_some() {
                found = true;
            }
            if need.next().is_some() {
                if found {
                    panic!("cannot match @uniq and @need in the same pattern")
                    // TODO think about sem. of this case
                }
                return true;
            }
        }
        // if m.nodes_for_capture_index(cid).next().is_some() {
        //     if found {
        //         return false;
        //     }
        //     found = true;
        // }
    }
}

fn simp_search_rm(
    query_store: &Store,
    query: NodeIdentifier,
    meta_simp: &hyperast_tsquery::Query,
) -> Vec<P> {
    let Some(cid_p) = meta_simp.capture_index_for_name("rm") else {
        return vec![];
    };
    let mut result = vec![];
    let pos = hyperast::position::structural_pos::CursorWithPersistance::new(query);
    let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(query_store, pos);
    let mut matches = meta_simp.matches(cursor);
    loop {
        let Some(m) = matches.next() else {
            break;
        };
        let Some(p) = m.nodes_for_capture_index(cid_p).next() else {
            continue;
        };
        let p = p.pos.clone().offsets();
        result.push(p);
    }
    result
}

fn apply_rms_aux(
    query_store: &mut Store,
    query: NodeIdentifier,
    path: &Vec<u16>,
) -> Option<NodeIdentifier> {
    let mut path = path.clone();
    path.pop();
    path.reverse();
    let action = tsq_transform::Action::Delete { path };
    let actions = vec![action];
    let query_bis = tsq_transform::regen_query(query_store, query, actions);
    query_bis
}

fn try_apply_rms_aux(
    query_store: &Store,
    query: NodeIdentifier,
    path: &Vec<u16>,
) -> Option<NodeIdentifier> {
    let mut path = path.clone();
    path.pop();
    path.reverse();
    let action = tsq_transform::Action::Delete { path };
    let actions = vec![action];
    let query_bis = tsq_transform::try_regen_query(query_store, query, actions);
    query_bis
}

fn replace_preds_with_caps(
    query_store: &mut Store,
    query: NodeIdentifier,
    per_label: &mut std::collections::HashMap<Lab, Vec<(Cap, P)>>,
) -> Option<NodeIdentifier> {
    let mut count = 0;
    println!(
        "{}",
        hyperast::nodes::TextSerializer::new(query_store, query)
    );
    let actions = per_label
        .values_mut()
        // .filter(|l| l.len() == 2)
        .flatten()
        .filter_map(|x| {
            assert!(x.0.is_empty()); // for now lets not consider other cases than imm. eq
            x.0 = format!("p{}", count);
            count += 1;
            let new = make_cap(query_store, &x.0);
            let mut path = x.1.clone();
            path.pop();
            path.reverse();
            // dbg!(&path);
            Some(tsq_transform::Action::Replace { path, new })
        })
        .collect();
    dbg!(&actions);
    let query_bis = tsq_transform::regen_query(query_store, query, actions);
    query_bis
}

type Store = hyperast::store::SimpleStores<crate::types::TStore>;

fn make_cap(query_store: &mut Store, name: &str) -> NodeIdentifier {
    let q = format!("_ @{}", name);
    let q = crate::search::ts_query2(query_store, q.as_bytes());
    use hyperast::types::WithChildren;
    let q = query_store.node_store.resolve(q).child(&0).unwrap();
    let q = query_store.node_store.resolve(q).child(&2).unwrap();
    q
}

type P = Vec<u16>;
type Lab = String;
type Cap = String;

fn simp_search_imm_preds(
    query_store: &Store,
    query: NodeIdentifier,
    meta_simp: &hyperast_tsquery::Query,
) -> std::collections::HashMap<Lab, Vec<(Cap, P)>> {
    let mut per_label = std::collections::HashMap::default();
    let pos = hyperast::position::structural_pos::CursorWithPersistance::new(query);
    let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(query_store, pos);
    let mut matches = meta_simp.matches(cursor);
    let Some(cid_p) = meta_simp.capture_index_for_name("pred") else {
        return Default::default();
    };
    let Some(cid_l) = meta_simp.capture_index_for_name("label") else {
        return Default::default();
    };
    // let cid_i = meta_simp.capture_index_for_name("id").unwrap();
    loop {
        let Some(capts) = matches.next() else { break };
        let Some(p) = capts.nodes_for_capture_index(cid_p).next() else {
            continue;
        };
        let k = capts.nodes_for_capture_index(cid_l).next().unwrap();
        let k = k.text(matches.cursor().text_provider());
        // let v = capts.nodes_for_capture_index(cid_i).next().unwrap();
        // let v = v.text(());
        let v = "";
        let p = p.pos.clone().offsets();
        per_label
            .entry(k.to_string())
            .or_insert(vec![])
            .push((v.to_string(), p));
    }
    per_label
}

struct PerLabel<P>(std::collections::HashMap<String, Vec<(String, P)>>);
impl<P> std::fmt::Display for PerLabel<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for x in self.0.values() {
            if x.len() == 2 {
                writeln!(f, "(#eq? @{} @{})", x[0].0, x[1].0)?;
            } else if x.len() == 1 {
                // noop
            } else {
                for y in &x[1..] {
                    writeln!(f, "(#eq? @{} @{})", x[0].0, y.0)?;
                }
                // todo!("need to do combination")
            }
        }
        Ok(())
    }
}
