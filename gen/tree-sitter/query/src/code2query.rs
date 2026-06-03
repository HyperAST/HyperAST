#![allow(unused)]
use crate::auto::tsq_ser_meta::Converter;
use crate::auto::tsq_transform;
use hyperast::position;
use hyperast::position::offsets_and_nodes::SolvedStructuralPosition;
use hyperast::position::position_accessors::{SolvedPosition, WithPreOrderOffsets};
use hyperast::store::defaults::NodeIdentifier;
use hyperast::types::{
    self, Children, HashKind, HyperAST, RoleStore, TypeStore, TypedNodeId, WithHashs,
    WithSerialization, WithStats,
};
use hyperast_tsquery::{Cursor, Node as _};
use legion::query;
use num::ToPrimitive;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

// use crate::legion as qgen; // includes identation, such as spaces and new lines
use crate::no_fmt_legion as qgen; // ignores spaces, new lines,...

type QStore = hyperast::store::SimpleStores<crate::types::TStore>;

type IdN = NodeIdentifier;
type Idx = u16;
type IdInit = SolvedStructuralPosition<IdN, Idx>;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TR<E = IdInit, I = IdNQ> {
    // WARN different NodeIdentifier, this one is referring to the provided examples
    Init(E),
    Uniqs(I),
    RMs(I),
    RMall(I),
    SimpEQ(I),
}

impl<E: PartialEq, I: PartialEq> PartialOrd for TR<E, I> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (TR::Init(_), TR::Init(_)) => Some(Ordering::Equal),
            (TR::Init(_), _) => Some(Ordering::Less),
            (TR::Uniqs(_), TR::Uniqs(_)) => Some(Ordering::Equal),
            (TR::Uniqs(_), _) => Some(Ordering::Less),
            (TR::RMall(_), TR::RMall(_)) => Some(Ordering::Equal),
            (TR::RMall(_), _) => Some(Ordering::Less),
            (TR::RMs(_), TR::RMs(_)) => Some(Ordering::Equal),
            (TR::RMs(_), _) => Some(Ordering::Less),
            (TR::SimpEQ(_), TR::SimpEQ(_)) => Some(Ordering::Equal),
            (TR::SimpEQ(_), _) => Some(Ordering::Less),
            // _ => self.eq(other).then(|| Ordering::Equal),
        }
    }
}

impl<E: PartialEq, I: PartialEq> TR<E, I> {
    pub fn each(&self, mut f: impl FnMut(&'static str, &E), mut g: impl FnMut(&'static str, &I)) {
        match self {
            TR::Init(t) => f("Init", t),
            TR::Uniqs(t) => g("Uniqs", t),
            TR::RMall(t) => g("RMall", t),
            TR::RMs(t) => g("RMs", t),
            TR::SimpEQ(t) => g("SimpEQ", t),
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
pub struct DedupRawEntry<TR>(hashbrown::HashMap<IdNQ, Vec<(IdNQ, TR)>>);

impl<TR> Default for DedupRawEntry<TR> {
    fn default() -> Self {
        Self(Default::default())
    }
}
// type DedupRawEntry = hashbrown::HashMap<IdN, Vec<(IdN, TR)>>;
// TODO directly handle this kind of dedup referentially in the HyperAST
// i.e. putting all the spaces in a wrapper root.
// It requires a special TreeGen that uses the global context, where it accumulates a topologically sorted list of spaces.
impl<TR> Deref for DedupRawEntry<TR> {
    type Target = hashbrown::HashMap<IdNQ, Vec<(IdNQ, TR)>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<TR> DerefMut for DedupRawEntry<TR> {
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
impl<TR> Ded for DedupRawEntry<TR> {
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
        let mut b: Builder<'_, IdN, DedupRawEntry<TR<IdN>>> = b.dedup_leaf_queries(|from| {
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
                .fold(DedupRawEntry::<TR<IdN>>::default(), |mut acc, x| {
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
}
impl<Init> QueryLattice<Init> {
    pub fn count(&self) -> usize {
        self.queries.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (IdNQ, &[IdQ])> {
        self.sort_cache.iter().filter_map(|i| {
            let (q, e) = &self.queries[*i as usize];
            Some((*q, &e[..]))
        })
    }

    pub fn iter_pretty(&self) -> impl Iterator<Item = (String, &[IdQ])> {
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
}

impl<Init: Clone + SolvedPosition<IdN>> QueryLattice<Init> {
    pub fn get_query(&self, index: usize) -> Option<(String, &[IdQ])> {
        self.queries
            .get(self.sort_cache[index] as usize)
            .and_then(|(q, e)| {
                let q = qgen::PP::<_, _>::new(&self.query_store, *q)
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
        from: impl Iterator<Item = Init>,
        meta_gen: &'q hyperast_tsquery::Query,
        meta_simp: &'q hyperast_tsquery::Query,
        f: &impl Fn(qgen::FNode) -> T,
    ) -> Builder<'q, Init, Vec<(Init, (IdNQ, T))>>
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
                let x = generate_query_aux::<TS, TIdN, _, Init>(
                    &mut s.query_store,
                    &mut md_cache,
                    stores,
                    from.clone(),
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
        from: impl Iterator<Item = Init>,
        meta_gen: &hyperast_tsquery::Query,
        meta_simp: &hyperast_tsquery::Query,
    ) -> Self
    where
        TS: TypeStore + RoleStore,
        TIdN: TypedNodeId<IdN = IdN>,
        TIdN::Ty: types::TypeTrait,
        TS::IdF: From<u16> + Into<u16>,
        Init: Sync + Send + Eq,
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
        from: impl Iterator<Item = Init>,
        meta_gen: &hyperast_tsquery::Query,
        meta_simp: &hyperast_tsquery::Query,
    ) -> Self
    where
        TS: TypeStore + RoleStore,
        TIdN: TypedNodeId<IdN = IdN>,
        TIdN::Ty: types::TypeTrait,
        TS::IdF: From<u16> + Into<u16>,
        Init: Sync + Send + Eq,
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

#[derive(Default)]
pub struct DedupBySize2<TR = self::TR>(Vec<hashbrown::HashMap<IdNQ, Vec<TR>>>);

impl<TR> Deref for DedupBySize2<TR> {
    type Target = Vec<hashbrown::HashMap<IdNQ, Vec<TR>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<TR> DerefMut for DedupBySize2<TR> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<TR> Ded for DedupBySize2<TR> {
    fn queries(&self) -> Vec<IdNQ> {
        self.0
            .iter()
            // .flat_map(|x| x.values().flat_map(|v| v.iter().map(|x| x)))
            .flat_map(|x| x.keys().copied())
            .collect()
    }
}

#[cfg(feature = "synth_par")]
pub fn group_by_size<Init: Clone + SolvedPosition<IdN> + Eq + Sync + Send>(
    from: Vec<(Init, (IdNQ, (u32, u32)))>,
) -> DedupBySize2<TR<Init>> {
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    DedupBySize2(
        from.into_iter()
            // grouping on size of queries (i.e. number of nodes)
            .fold(Vec::<Vec<(Init, (IdNQ, u32))>>::new(), |mut acc, x| {
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
                    hashbrown::HashMap::<IdNQ, Vec<TR<Init>>>::new(),
                    |mut acc, x| {
                        let (from, (query, label_h)) = x;
                        let v = &mut acc.entry(query).or_default();
                        // dbg!(query);
                        let x = (query, TR::Init(from));
                        if !v.contains(&x.1) {
                            v.push(x.1.clone());
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

pub struct Builder<'q, E, D = DedupBySize2<TR<E>>> {
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

impl Builder<'_, IdN, DedupRawEntry<TR<IdN>>> {
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
                    simp_rms(&mut s.query_store, query, meta_simp)
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
#[cfg(feature = "synth_par")]
impl<Init: Clone + SolvedPosition<IdN> + Sync + Send> Builder<'_, Init, DedupBySize2<TR<Init>>>
where
    Init: Sync,
{
    fn loop_par(&mut self)
    where
        Init: Eq,
    {
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
    fn removes_par(
        &mut self,
        active_size: usize,
        active: &mut Vec<IdNQ>,
    ) -> Vec<(u32, (IdNQ, TR<Init>))> {
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
                simp_rms(&mut s.query_store, query, meta_simp)
                    .map(|(new_q, label_h)| (label_h, (new_q, TR::RMs(query))))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        rms
    }

    pub fn dedup_removes_par(
        &mut self,
        active_size: usize,
        active: &mut Vec<IdNQ>,
        rms: Vec<(u32, (IdNQ, TR<Init>))>,
    ) where
        Init: Eq,
    {
        use rayon::iter::{
            IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator,
            ParallelIterator,
        };
        let s = &mut self.lattice;
        let dedup = &mut self.dedup;
        let fold = ParallelIterator::fold(
            rms.into_par_iter(),
            BTreeMap::default,
            |mut acc: BTreeMap<usize, Vec<_>>, (label_h, x): (u32, (IdN, TR<Init>))| {
                let size = s.query_store.resolve(&x.0).size();
                acc.entry(size).or_default().push((label_h, x));
                acc
            },
        );
        let aaa: BTreeMap<usize, Vec<(LabelH, (IdN, TR<Init>))>> = ParallelIterator::reduce(
            fold,
            BTreeMap::<usize, Vec<(LabelH, (IdN, TR<Init>))>>::default,
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
                    use hashbrown::hash_map::Entry;
                    let v = match v {
                        Entry::Occupied(x) => x.into_mut(),
                        Entry::Vacant(x) => {
                            r.push(*x.key());
                            x.insert(vec![])
                        }
                    };
                    if !v.contains(&x.1) {
                        v.push(x.1.clone());
                    }
                }
                r
            },
        )
        .collect();
        active.extend(act);
    }

    pub fn dedup_uniques_par(
        &mut self,
        active_size: usize,
        // size, origin, curr, tr
        uniques: Vec<(u32, (IdNQ, TR<Init>))>,
    ) -> Vec<IdNQ>
    where
        Init: Eq,
    {
        use rayon::iter::{
            IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator,
            ParallelIterator,
        };
        let s = &mut self.lattice;
        let dedup = &mut self.dedup;
        let fold = ParallelIterator::fold(
            uniques.into_par_iter(),
            BTreeMap::default,
            |mut acc: BTreeMap<usize, Vec<_>>, (label_h, x): (u32, (IdNQ, TR<Init>))| {
                let size = s.query_store.resolve(&x.0).size();
                acc.entry(size).or_default().push((label_h, x));
                acc
            },
        );
        let aaa: BTreeMap<usize, Vec<(LabelH, (IdNQ, TR<Init>))>> = ParallelIterator::reduce(
            fold,
            BTreeMap::<usize, Vec<(LabelH, (IdNQ, TR<Init>))>>::default,
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
                debug_assert!(x.0 < active_size);
                acc[x.0] = x.1;
                acc
            });
        ParallelIterator::flat_map(
            dedup.0[..active_size].par_iter_mut().enumerate(),
            |(i, dedup)| {
                let mut r = vec![];
                for (label_h, x) in &aaa[i] {
                    let v = dedup.entry(x.0);
                    use hashbrown::hash_map::Entry;
                    let v = match v {
                        Entry::Occupied(x) => x.into_mut(),
                        Entry::Vacant(x) => {
                            r.push(*x.key());
                            x.insert(vec![])
                        }
                    };
                    if !v.contains(&x.1) {
                        v.push(x.1.clone());
                    }
                }
                r
            },
        )
        .collect()
    }

    pub fn dedup_uniques_par2(
        &mut self,
        active_size: usize,
        // size, origin, curr, tr
        uniques: Vec<(u32, (IdNQ, TR<Init>))>,
    ) -> Vec<IdNQ>
    where
        Init: Eq,
    {
        use rayon::iter::{
            IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator,
            ParallelIterator,
        };
        let s = &mut self.lattice;
        let dedup = &mut self.dedup;
        let fold = ParallelIterator::fold(
            uniques.into_par_iter(),
            BTreeMap::default,
            |mut acc: BTreeMap<usize, Vec<_>>, (label_h, x): (u32, (IdNQ, TR<Init>))| {
                let size = s.query_store.resolve(&x.0).size();
                acc.entry(size).or_default().push((label_h, x));
                acc
            },
        );
        let aaa: BTreeMap<usize, Vec<(LabelH, (IdNQ, TR<Init>))>> = ParallelIterator::reduce(
            fold,
            BTreeMap::<usize, Vec<(LabelH, (IdNQ, TR<Init>))>>::default,
            |mut acc, b| {
                for (size, v) in b {
                    acc.entry(size).or_default().extend(v);
                }
                acc
            },
        );
        let max_active = aaa.last_key_value().map_or(active_size, |x| *x.0);
        let len = max_active.max(dedup.len());
        dedup.resize(len, Default::default());
        ParallelIterator::flat_map(dedup.0[..].par_iter_mut().enumerate(), |(i, dedup)| {
            let mut r = vec![];
            for (label_h, x) in aaa.get(&i).map_or(&vec![], |x| x) {
                let v = dedup.entry(x.0);
                use hashbrown::hash_map::Entry;
                let v = match v {
                    Entry::Occupied(x) => x.into_mut(),
                    Entry::Vacant(x) => {
                        r.push(*x.key());
                        x.insert(vec![])
                    }
                };
                if !v.contains(&x.1) {
                    v.push(x.1.clone());
                }
            }
            r
        })
        .collect()
    }

    pub fn post(&mut self)
    where
        Init: Send,
    {
        use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
        self.dedup.0.par_iter_mut().for_each(|x| {
            let qstores = &self.lattice.query_store;
            for y in x.values_mut() {
                // y.sort_by(cmp_lat_entry(&qstores))
            }
        });
        for v in self.dedup.iter().flat_map(|x| x.iter()) {
            self.lattice.add_raw_rels2(*v.0, v.1);
        }
        // eprintln!("final: {}", pp_dedup(&self.dedup, self.dedup.len() - 1));
        for v in self.dedup.iter().flat_map(|x| x.iter()) {
            let value = self.extract2(*v.0, v.1);
            self.lattice.queries.push(value);
        }
    }

    pub fn loop_par_par(&mut self)
    where
        Init: Eq,
    {
        let mut active_size = self.dedup.len() - 1;
        // eprintln!("{active_size}: {}", pp_dedup(&self.dedup, active_size));
        let mut active: Vec<_> = self.actives(active_size);
        loop {
            dbg!(active_size);
            dbg!(active.len());
            // eprintln!("{}", pp_dedup(&self.dedup, active_size));
            let (uniqs, already) = self.uniques_par_par(active_size, &mut active);
            dbg!(uniqs.len());
            self.dedup_uniques_par(active_size, uniqs);
            active = already;
            let rms = self.removes_par_par(active_size, &mut active);
            dbg!(rms.len());
            self.dedup_removes_par(active_size, &mut active, rms);
            if self.between(&mut active_size, &mut active) {
                break;
            }
        }
    }

    /// Must use dedup_removes_par on output to properly progress
    #[must_use]
    pub fn removes_par_par(
        &mut self,
        active_size: usize,
        active: &mut Vec<IdNQ>,
    ) -> Vec<(u32, (IdNQ, TR<Init>))> {
        use rayon::iter::{
            IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator,
            ParallelIterator,
        };
        let s = &mut self.lattice;
        let dedup = &mut self.dedup;
        let meta_simp = self.meta_simp;
        let rms = std::mem::take(active).into_par_iter();
        let rms = ParallelIterator::flat_map(rms, |x| {
            let query = x;
            try_simp_rms(&s.query_store, query, meta_simp)
                .map(|x| match x {
                    Ok((new_q, label_h)) => Ok((label_h, (new_q, TR::RMs(query)))),
                    Err(e) => Err(e),
                })
                .collect::<Vec<_>>()
        });
        let (remains, mut rms): (Vec<(IdN, Vec<u16>)>, Vec<(u32, (IdN, TR<Init>))>) =
            ParallelIterator::partition_map(rms, |x| x.into());
        log::info!("remains: {}", remains.len());
        log::info!("rms: {}", rms.len());

        let rem_count = remains.len();
        remains.chunks(1000).enumerate().for_each(|(i, x)| {
            let i = i * 1000;
            log::info!("remains removes {i:4}/{rem_count}");
            rms.extend(x.into_iter().filter_map(|(query, path)| {
                let query = *query;
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
            }))
        });
        rms
    }
    #[must_use]
    pub fn removesall_par_par(
        &mut self,
        active_size: usize,
        active: &mut Vec<IdNQ>,
        cid: hyperast_tsquery::CaptureId,
    ) -> Vec<(u32, (IdNQ, TR<Init>))> {
        use rayon::iter::{
            IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator,
            ParallelIterator,
        };
        let s = &mut self.lattice;
        let dedup = &mut self.dedup;
        let meta_simp = self.meta_simp;
        let rms = std::mem::take(active).into_par_iter();
        let rms = ParallelIterator::filter_map(rms, |x| {
            let query = x;
            try_simp_rmalls(&s.query_store, query, meta_simp, cid).map(|x| match x {
                Ok((new_q, label_h)) => Ok((label_h, (new_q, TR::RMall(query)))),
                Err(e) => Err(e),
            })
        });
        let (remains, mut rms): (
            Vec<(IdNQ, IdNQ, Vec<Vec<u16>>)>,
            Vec<(u32, (IdNQ, TR<Init>))>,
        ) = ParallelIterator::partition_map(rms, |x| x.into());
        log::info!("remains: {}", remains.len());
        log::info!("rms: {}", rms.len());

        let rem_count = remains.len();
        remains.chunks(1000).enumerate().for_each(|(i, x)| {
            let i = i * 1000;
            log::info!("remains removesall {i:4}/{rem_count}");

            rms.extend(x.into_iter().filter_map(|(query, curr, paths)| {
                let query = *query;
                let mut curr = *curr;
                // dbg!(&paths);
                for path in paths.into_iter() {
                    curr = apply_rms_aux2(&mut s.query_store, curr, &path)?;
                }
                if !simp_search_need(&s.query_store, curr, meta_simp) {
                    return None;
                }
                let new_q = curr;
                let label_h = s.query_store.resolve(&new_q).hash(&HashKind::label());
                if new_q == query {
                    todo!()
                    // return None;
                }
                assert_ne!(new_q, query);
                Some((label_h, (new_q, TR::RMall(query))))
            }))
        });
        rms
    }

    /// Must use dedup_uniques_par on output to properly progress
    #[must_use]
    pub fn uniques_par_par(
        &mut self,
        active_size: usize,
        active: &mut Vec<IdNQ>,
    ) -> (Vec<(u32, (IdNQ, TR<Init>))>, Vec<IdNQ>) {
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
            use rayon::iter::Either;
            try_simp_uniq(&s.query_store, query, meta_simp)
                .map(|x| match x {
                    ResSimpUniq::Deduplicated(new_q, label_h) => {
                        Either::Right((label_h, (new_q, TR::Uniqs(query))))
                    }
                    ResSimpUniq::AlreadyUniq(i) => Either::Left(Either::Right(i)),
                    ResSimpUniq::NeedMut(a, b, c) => Either::Left(Either::Left((a, b, c))),
                })
                .collect::<Vec<_>>()
        });

        let ((remains, already), mut rms): (
            (Vec<(IdNQ, IdNQ, Vec<Vec<u16>>)>, Vec<IdNQ>),
            Vec<(u32, (IdNQ, TR<Init>))>,
        ) = ParallelIterator::partition_map(rms, |x| x.into());
        log::info!("remains: {}", remains.len());
        log::info!("uniqs: {}", rms.len());

        rms.extend(remains.into_iter().filter_map(|(query, mut curr, paths)| {
            // dbg!(&paths);
            for path in paths.into_iter() {
                curr = apply_rms_aux(&mut s.query_store, curr, &path)?;
            }
            if !simp_search_need(&s.query_store, curr, meta_simp) {
                return None;
            }
            let new_q = curr;
            let label_h = s.query_store.resolve(&new_q).hash(&HashKind::label());
            if new_q == query {
                todo!()
                // return None;
            }
            assert_ne!(new_q, query);
            Some((label_h, (new_q, TR::Uniqs(query))))
        }));
        (rms, already)
    }
}
#[cfg(feature = "synth_par")]
impl<Init: Clone + SolvedPosition<IdN> + Send + Sync> Builder<'_, Init> {
    pub fn actives(&mut self, active_size: usize) -> Vec<IdN> {
        use rayon::iter::ParallelIterator;

        self.dedup[active_size]
            .par_keys()
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
}

#[cfg(not(feature = "synth_par"))]
impl<Init: Clone + SolvedPosition<IdN>> Builder<'_, Init> {
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
}

impl<Init: Clone + SolvedPosition<IdN>> Builder<'_, Init> {
    pub fn simp_eq(
        &mut self,
        active_size: usize,
        active: &mut Vec<IdNQ>,
    ) -> (Vec<(u32, (IdNQ, TR<Init>))>, Vec<IdNQ>) {
        let s = &mut self.lattice;
        let dedup = &mut self.dedup;
        let meta_simp = self.meta_simp;
        let act = std::mem::take(active).into_iter();
        let mut already = vec![];
        let simp_eq = act
            .flat_map(|x| {
                let query = x;
                let Some((new_q, label_h)) = simp_imm_eq(&mut s.query_store, query, meta_simp)
                else {
                    already.push(query);
                    return vec![];
                };
                assert_ne!(new_q, query);
                vec![(label_h, (new_q, TR::<Init>::SimpEQ(query)))]
            })
            .collect::<Vec<_>>();
        dbg!(dedup.len());
        (simp_eq, already)
    }
}

impl<E: Clone> QueryLattice<E> {
    fn add_raw_rels(&mut self, v: &Vec<(IdN, TR<E>)>) {
        for v in v {
            self.raw_rels.entry(v.0).or_default().push(v.1.clone());
        }
    }

    fn add_raw_rels2(&mut self, k: IdN, v: &Vec<TR<E>>) {
        for v in v {
            self.raw_rels.entry(k).or_default().push(v.clone());
        }
    }

    pub fn leaf(&self, id: IdQ) -> IdNQ {
        self.leaf_queries[id as usize]
    }
}

impl<Init> Builder<'_, Init, DedupRawEntry<TR<Init>>> {
    fn extract(&self, v: &[(IdNQ, TR<Init>)]) -> (IdNQ, Vec<IdQ>) {
        fn extract<'a, Init: 'a>(
            map: &'a std::collections::HashMap<IdNQ, Vec<TR<Init>>>,
            curr: IdNQ,
            downs: impl Iterator<Item = &'a TR<Init>>,
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
                    TR::RMs(v) | TR::Uniqs(v) | TR::SimpEQ(v) if !already.contains(v) => {
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
            v.iter().map(|x| &x.1),
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
}

impl<Init, D> Builder<'_, Init, D> {
    fn extract2(&self, k: IdNQ, v: &[TR<Init>]) -> (IdNQ, Vec<IdQ>) {
        fn extract<'a, Init: 'a>(
            map: &'a std::collections::HashMap<IdNQ, Vec<TR<Init>>>,
            curr: IdNQ,
            downs: impl Iterator<Item = &'a TR<Init>>,
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
                    TR::RMs(v) | TR::Uniqs(v) | TR::SimpEQ(v) if !already.contains(v) => {
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
            v.iter(),
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

        a_l.cmp(&b_l)
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
                if acc.last().is_some_and(|x| x.is_err()) {
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
    println!();
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
    let query = qgen::PP::<_, _>::new(&query_store, query_bis.unwrap()).to_string();
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
    let mut per_label = simp_search_imm_preds(query_store, query, meta_simp);
    let query = replace_preds_with_caps(query_store, query, per_label.values_mut().collect())?;
    let preds = format!("(_) {}", PerLabel(per_label));
    let mut md_cache = Default::default();
    let mut query_tree_gen = qgen::TsQueryTreeGen::new(query_store, &mut md_cache);
    let tree = match qgen::tree_sitter_parse(preds.as_bytes()) {
        Ok(t) => t,
        Err(t) => {
            eprintln!("{}", t.root_node().to_sexp());
            t
        }
    };
    let preds = query_tree_gen.generate_file(b"", preds.as_bytes(), tree.walk());
    let preds = preds.local.compressed_node;
    use hyperast::types::WithChildren;
    eprintln!(
        "{}",
        hyperast::nodes::SyntaxSerializer::new(query_store, preds)
    );

    // dbg!(qgen::PP::<_, _>::new(&*query_store, preds).to_string());
    let main_query = query_store.node_store.resolve(query).child(&0).unwrap();

    let mut new_q = vec![main_query];
    let preds = query_store.node_store.resolve(preds);
    let preds = Children::<Idx, _>::after(&(preds).children().unwrap(), 1);
    new_q.extend(preds);

    let mut query_tree_gen = qgen::TsQueryTreeGen::new(query_store, &mut md_cache);
    let new_q = query_tree_gen.build_then_insert(query, crate::types::Type::Program, None, new_q);

    // dbg!();
    // eprintln!(
    //     "{}",
    //     hyperast::nodes::SyntaxSerializer::new(query_store, new_q)
    // );

    let query = new_q;

    // let query = format!("{} {}", query, PerLabel(per_label));
    // println!("\nThe generified query:\n{}", query);
    // crate::search::ts_query2_with_label_hash(query_store, query.as_bytes())
    Some((query, query_store.resolve(&query).hash(&HashKind::label())))
}

/// remove a matched thing from query
fn simp_rms<'a>(
    query_store: &'a mut hyperast::store::SimpleStores<crate::types::TStore>,
    query: NodeIdentifier,
    meta_simp: &'a hyperast_tsquery::Query,
) -> impl Iterator<Item = (NodeIdentifier, LabelH)> + 'a {
    let rms = if let Some(cid) = meta_simp.capture_index_for_name("rm") {
        find_matches(query_store, query, meta_simp, cid)
    } else {
        vec![]
    };
    rms.into_iter().filter_map(move |path| {
        let query = apply_rms_aux(query_store, query, &path)?;
        if !simp_search_need(query_store, query, meta_simp) {
            return None;
        }
        // let query = hyperast::nodes::TextSerializer::<_, _>::new(&*query_store, query).to_string();
        // crate::search::ts_query2_with_label_hash(query_store, query.as_bytes())
        Some((query, query_store.resolve(&query).hash(&HashKind::label())))
    })
}

/// remove all matched thing from query
fn simp_rmalls<'a>(
    query_store: &'a mut hyperast::store::SimpleStores<crate::types::TStore>,
    query: NodeIdentifier,
    meta_simp: &'a hyperast_tsquery::Query,
    cid: hyperast_tsquery::CaptureId,
) -> impl Iterator<Item = (NodeIdentifier, LabelH)> + 'a {
    let mut rms = find_matches(query_store, query, meta_simp, cid);
    let mut curr = query;
    for path in rms {
        curr = apply_rms_aux(query_store, curr, &path).unwrap();
    }
    if !simp_search_need(query_store, query, meta_simp) {
        return vec![].into_iter();
    }
    if curr == query {
        return vec![].into_iter();
    }
    vec![(query, query_store.resolve(&query).hash(&HashKind::label()))].into_iter()
}

fn try_simp_rms<'a>(
    query_store: &'a hyperast::store::SimpleStores<crate::types::TStore>,
    query: NodeIdentifier,
    meta_simp: &'a hyperast_tsquery::Query,
) -> impl Iterator<Item = Result<(NodeIdentifier, LabelH), (NodeIdentifier, Vec<u16>)>> + 'a {
    let rms = if let Some(cid) = meta_simp.capture_index_for_name("rm") {
        find_matches(query_store, query, meta_simp, cid)
    } else {
        vec![]
    };
    rms.into_iter().filter_map(move |path| {
        let Some(query) = try_apply_rms_aux(query_store, query, &path) else {
            return Some(Err((query, path)));
        };
        if !simp_search_need(query_store, query, meta_simp) {
            return None;
        }
        Some(Ok((
            query,
            query_store.resolve(&query).hash(&HashKind::label()),
        )))
    })
}

fn try_simp_rmalls<'a>(
    query_store: &'a hyperast::store::SimpleStores<crate::types::TStore>,
    query: NodeIdentifier,
    meta_simp: &'a hyperast_tsquery::Query,
    cid: hyperast_tsquery::CaptureId,
) -> Option<Result<(NodeIdentifier, LabelH), (NodeIdentifier, NodeIdentifier, Vec<Vec<u16>>)>> {
    let mut rms = find_matches(query_store, query, meta_simp, cid);
    for rm in &mut rms {
        rm.pop();
        rm.reverse();
    }
    rms.sort();
    rms.reverse();
    let mut curr = query;
    for i in 0..rms.len() {
        let Some(query) = try_apply_rms_aux2(query_store, curr, &rms[i]) else {
            return Some(Err((query, curr, rms[i..].into())));
        };
        if !simp_search_need(query_store, query, meta_simp) {
            return None;
        }
        curr = query;
    }
    if curr == query {
        return None;
    }
    let query = curr;
    Some(Ok((
        query,
        query_store.resolve(&query).hash(&HashKind::label()),
    )))
}

fn simp_uniq<'a>(
    query_store: &'a mut hyperast::store::SimpleStores<crate::types::TStore>,
    query: NodeIdentifier,
    meta_simp: &'a hyperast_tsquery::Query,
) -> impl Iterator<Item = (NodeIdentifier, LabelH)> + 'a {
    let m = if let Some(cid) = meta_simp.capture_index_for_name("uniq") {
        find_matches(query_store, query, meta_simp, cid)
    } else {
        vec![]
    };
    (0..m.len()).filter_map(move |i| {
        let paths = m
            .iter()
            .enumerate()
            .filter_map(|(j, x)| (i != j).then_some(x))
            .rev();
        let mut curr = query;

        // TODO perf: do all the removes at once
        dbg!(&paths);
        for path in paths {
            curr = apply_rms_aux(query_store, curr, &path)?;
        }
        if !simp_search_need(query_store, query, meta_simp) {
            return None;
        }
        Some((query, query_store.resolve(&query).hash(&HashKind::label())))
    })
}

enum ResSimpUniq {
    Deduplicated(NodeIdentifier, LabelH), // Some Ok
    NeedMut(NodeIdentifier, NodeIdentifier, Vec<Vec<u16>>), // Some Err
    AlreadyUniq(NodeIdentifier),          // None
}

fn try_simp_uniq<'a>(
    query_store: &'a hyperast::store::SimpleStores<crate::types::TStore>,
    query: IdNQ,
    meta_simp: &'a hyperast_tsquery::Query,
) -> impl Iterator<
    Item = ResSimpUniq, // Result<(NodeIdentifier, LabelH), (NodeIdentifier, NodeIdentifier, Vec<Vec<u16>>)>,
> + 'a {
    let m = if let Some(cid) = meta_simp.capture_index_for_name("uniq") {
        find_matches(query_store, query, meta_simp, cid)
    } else {
        vec![]
    };
    let already = if m.len() == 1 { vec![query] } else { vec![] };
    let candidates = if m.len() == 1 { 0..0 } else { 0..m.len() };
    candidates
        .map(move |i| {
            assert_ne!(m.len(), 1);
            let mut curr = query;
            // TODO perf: do all then removes at once
            let mut it = m.iter().enumerate().rev();
            // while let Some((j, path)) = it.next() {
            // if i == j {
            //     continue;
            // }
            // let Some(q) = try_apply_rms_aux(query_store, curr, path) else {
            return ResSimpUniq::NeedMut(
                query,
                curr,
                it
                    // .chain([(j, path)])
                    .filter_map(|(j, x)| (i != j).then_some(x).cloned())
                    .collect(),
            );
            // };
            // curr = q
            // }
            if !simp_search_need(query_store, curr, meta_simp) {
                unimplemented!("don't know what to do there");
            }
            assert_ne!(curr, query);
            ResSimpUniq::Deduplicated(curr, query_store.resolve(&curr).hash(&HashKind::label()))
        })
        .chain(already.into_iter().map(|x| ResSimpUniq::AlreadyUniq(x)))
}

fn generate_query_aux<
    TS: TypeStore + RoleStore,
    TIdN: TypedNodeId<IdN = NodeIdentifier>,
    T,
    Init: SolvedPosition<IdN>,
>(
    query_store: &mut QStore,
    md_cache: &mut qgen::MDCache,
    stores: &hyperast::store::SimpleStores<TS>,
    from: Init,
    meta_gen: &hyperast_tsquery::Query,
    f: &impl Fn(qgen::FNode) -> T,
) -> Option<(IdNQ, T)>
where
    TIdN::Ty: types::TypeTrait,
    TS::IdF: From<u16> + Into<u16>,
{
    use crate::auto::tsq_ser_meta2::TreeToQuery;
    let query = TreeToQuery::<_, TIdN>::new(stores, from.node(), meta_gen.clone());
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

pub fn pred_uniq(
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
    let Some(cid) = meta_simp.capture_index_for_name("rm") else {
        return vec![];
    };
    find_matches(query_store, query, meta_simp, cid)
}

fn simp_search_uniq(
    query_store: &Store,
    query: NodeIdentifier,
    meta_simp: &hyperast_tsquery::Query,
) -> Vec<P> {
    let Some(cid) = meta_simp.capture_index_for_name("uniq") else {
        return vec![];
    };
    find_matches(query_store, query, meta_simp, cid)
}

pub fn find_matches(
    query_store: &Store,
    query: NodeIdentifier,
    meta_simp: &hyperast_tsquery::Query,
    cid: hyperast_tsquery::CaptureId,
) -> Vec<P> {
    let set = find_matches_aux(query_store, query, meta_simp, cid);
    set.collect_vec(|p| p.offsets())
}

pub fn find_matches_aux(
    query_store: &Store,
    query: NodeIdentifier,
    meta_simp: &hyperast_tsquery::Query,
    cid: hyperast_tsquery::CaptureId,
) -> position::structural_pos::CursorWithPersistanceOrderedSet<NodeIdentifier> {
    // let mut result = vec![];
    let mut pos = hyperast::position::structural_pos::CursorWithPersistance::new(query);
    let mut set = pos.build_empty_set();
    let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(query_store, pos);
    let mut matches = meta_simp.matches(cursor);
    loop {
        let Some(m) = matches.next() else {
            break;
        };
        for p in m.nodes_for_capture_index(cid) {
            // let p = p.pos.clone().offsets();
            // result.push(p);
            set.register(&p.pos);
        }
    }
    set
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

    tsq_transform::regen_query(query_store, query, actions)
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

    tsq_transform::try_regen_query(query_store, query, actions)
}

fn apply_rms_aux2(
    query_store: &mut Store,
    query: NodeIdentifier,
    path: &Vec<u16>,
) -> Option<NodeIdentifier> {
    let mut path = path.clone();
    let action = tsq_transform::Action::Delete { path };
    let actions = vec![action];

    tsq_transform::regen_query(query_store, query, actions)
}

fn try_apply_rms_aux2(
    query_store: &Store,
    query: NodeIdentifier,
    path: &Vec<u16>,
) -> Option<NodeIdentifier> {
    let mut path = path.clone();
    let action = tsq_transform::Action::Delete { path };
    let actions = vec![action];

    tsq_transform::try_regen_query(query_store, query, actions)
}

pub fn replace_preds_with_caps(
    query_store: &mut Store,
    query: NodeIdentifier,
    per_label_values: Vec<&mut Vec<(String, Vec<u16>)>>,
) -> Option<NodeIdentifier> {
    let mut count = 0;
    let mut values: Vec<_> = per_label_values;
    values.sort_by_key(|x| x.iter().map(|x| &x.1).max().unwrap_or(&vec![]).clone());
    let mut actions: Vec<_> = values
        .into_iter()
        .filter(|l| l.len() >= 2)
        // .filter(|l| l.len() == 2)
        .flatten()
        .filter_map(|x| {
            let new = if x.0.is_empty() {
                x.0 = format!("p{}", count);
                count += 1;
                make_cap(query_store, &x.0)
            } else {
                make_cap(query_store, &x.0)
            };
            // assert!(x.0.is_empty()); // for now lets not consider other cases than imm. eq
            // x.0 = format!("p{}", count);
            // count += 1;
            // let new = make_cap(query_store, &x.0);
            let mut path = x.1.clone();
            path.pop();
            path.reverse();
            // dbg!(&path);
            Some((path, new))
        })
        .collect();
    if actions.is_empty() {
        return None;
    }
    actions.sort_by(|a, b| a.0.cmp(&b.0));
    let actions: Vec<_> = actions
        .into_iter()
        .map(|(path, new)| tsq_transform::Action::Replace { path, new })
        .collect();
    eprintln!("[{}:{}]", file!(), line!());
    for a in actions.iter() {
        match a {
            tsq_transform::Action::Replace { path, new } => {
                eprintln!("replace {path:?}");
                println!("\t`{}`", qgen::PP::<_, _>::new(&*query_store, *new));
            }
            x => {
                dbg!(x);
            }
        }
    }

    tsq_transform::regen_query(query_store, query, actions)
}

type Store = hyperast::store::SimpleStores<crate::types::TStore>;

pub fn make_cap(query_store: &mut Store, name: &str) -> NodeIdentifier {
    let q = format!("_ @{}", name);
    let q = crate::search::ts_query2(query_store, q.as_bytes());
    use hyperast::types::WithChildren;
    let q = query_store.node_store.resolve(q).child(&0).unwrap();
    let q = query_store.node_store.resolve(q).child(&1).unwrap(); // NOTE: no spaces now
    // let q = query_store.node_store.resolve(q).child(&2).unwrap();
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
impl<P: Ord> std::fmt::Display for PerLabel<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut values: Vec<_> = self.0.values().collect();
        values.sort_by_key(|x| x.iter().map(|x| &x.1).max().unwrap());
        for x in values {
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
