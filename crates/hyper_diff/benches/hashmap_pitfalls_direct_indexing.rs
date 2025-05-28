use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

trait Searchable<K, V> {
    fn search(&self, k: &K) -> Option<&V>;
}

// TODO try with different hashers
impl<K: Eq + std::hash::Hash, V, S: std::hash::BuildHasher> Searchable<K, V>
    for std::collections::HashMap<K, V, S>
{
    fn search(&self, k: &K) -> Option<&V> {
        self.get(k)
    }
}

impl<K: Eq + std::hash::Hash, V, S: std::hash::BuildHasher> Searchable<K, V>
    for hyperast::compat::HashMap<K, V, S>
{
    fn search(&self, k: &K) -> Option<&V> {
        self.get(k)
    }
}

impl<K: Ord, V> Searchable<K, V> for std::collections::BTreeMap<K, V> {
    fn search(&self, k: &K) -> Option<&V> {
        self.get(k)
    }
}

struct SortedVecSoA<K, V>(Vec<K>, Vec<V>);

impl<K: Clone + Ord, V: Clone> SortedVecSoA<K, V> {
    fn new(collec: &[(K, V)]) -> Self {
        let mut collec: Vec<(K, V)> = collec.into_iter().map(|x| x.clone()).collect();
        collec.sort_by_key(|x| x.0.clone());
        let (ks, vs) = collec.into_iter().unzip();
        Self(ks, vs)
    }
}

impl<K: Ord, V> Searchable<K, V> for SortedVecSoA<K, V> {
    fn search(&self, seek: &K) -> Option<&V> {
        let i = self.0.binary_search_by(|probe| probe.cmp(&seek)).ok()?;
        Some(&self.1[i])
    }
}

struct DirectIndexing<K, V>(Vec<Option<V>>, K);

impl<K: hyperast::PrimInt, V: Clone> DirectIndexing<K, V> {
    fn new(collec: &[(K, V)], max: K) -> Self {
        let mut vs = vec![None; max.to_usize().unwrap()];
        for (k, v) in collec {
            vs[k.to_usize().unwrap()] = Some(v.clone());
        }
        Self(vs, max)
    }
}

impl<K: hyperast::PrimInt, V> Searchable<K, V> for DirectIndexing<K, V> {
    fn search(&self, seek: &K) -> Option<&V> {
        self.0[seek.to_usize().unwrap()].as_ref()
    }
}

fn compare_hashmaps(c: &mut Criterion) {
    let max = 50_0000;
    let mut group = c.benchmark_group(format!("HashMap Pitfalls Direct Indexing {max}"));

    type K = u32;
    type V = u32;

    let mut curr: u64 = 42; // set seed
    macro_rules! h {
        () => {{
            curr = hyperast::utils::hash(&curr);
            curr
        }};
        ($i:expr) => {
            (0..$i)
                .map(|i| ((h!() as K % max, i as K)))
                .collect::<Vec<_>>()
        };
        ($i:expr,$e:expr) => {
            (0..$i).map(|_| $e).collect::<Vec<_>>()
        };
    }

    let mut simple = h!(150000);
    let k = &[
        simple[0].0,
        simple[1].0,
        simple[2].0,
        simple[3].0,
        simple[4].0,
        simple[5].0,
        simple[6].0,
        simple[7].0,
    ];
    simple.sort_by_key(|x| x.0);
    simple.dedup_by_key(|x| x.0);
    #[allow(non_snake_case)]
    let INPUTS: &[(&[(K, V)], &[K])] = &[
        (&simple[0..10], k),
        (&simple[0..100], k),
        (&simple[0..500], k),
        (&simple[0..1000], k),
        (&simple[0..2000], k),
        (&simple[0..3000], k),
        (&simple[0..4000], k),
    ];

    for (_i, (collec, keys)) in INPUTS.into_iter().enumerate() {
        let id = collec.len();
        group.throughput(Throughput::Elements(collec.len() as u64));
        let mut hashmap = None;
        let mut ahash = None;
        let mut sorted_vec_soa = None;
        let mut direct_indexing = None;
        for key in keys.into_iter().take(1) {
            group.bench_with_input(BenchmarkId::new("HashMap", id), key, |b, key| {
                let collec = hashmap.get_or_insert_with(|| {
                    collec
                        .into_iter()
                        .map(|x| x.clone())
                        .collect::<std::collections::HashMap<K, V>>()
                });
                b.iter(|| collec.search(key))
            });
            group.bench_with_input(BenchmarkId::new("AHash", id), key, |b, key| {
                let collec = ahash.get_or_insert_with(|| {
                    collec
                        .into_iter()
                        .map(|x| x.clone())
                        .collect::<hyperast::compat::HashMap<K, V>>()
                });
                b.iter(|| collec.search(key))
            });
            group.bench_with_input(BenchmarkId::new("SortedVecSoA", id), key, |b, key| {
                let collec = sorted_vec_soa.get_or_insert_with(|| SortedVecSoA::new(collec));
                b.iter(|| collec.search(key))
            });
            group.bench_with_input(BenchmarkId::new("DirectIndexing", id), key, |b, key| {
                let collec =
                    direct_indexing.get_or_insert_with(|| DirectIndexing::new(collec, max));
                b.iter(|| collec.search(key))
            });
        }
    }
    group.finish();
}

criterion_group!(hashmaps, compare_hashmaps);
criterion_main!(hashmaps);
