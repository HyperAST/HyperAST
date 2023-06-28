use rhai::EvalAltResult;

use super::estimate::Estimate;
use super::finalize::Finalize;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
pub(super) struct Stats(Arc<Mutex<StatsInner>>);

#[derive(Clone)]
struct StatsInner {
    values: BTreeMap<i64, u64>,
    n: u64,
}

impl average::Merge for Stats {
    fn merge(&mut self, other: &Self) {
        if Arc::ptr_eq(&self.0, &other.0) {
            return;
        } else {
            let mut inner = self.0.lock().unwrap();
            let other_inner = other.0.lock().unwrap();
            inner.n += other_inner.n;
            for (value, count) in other_inner.values.iter() {
                inner
                    .values
                    .entry(*value)
                    .and_modify(|curr| *curr += count)
                    .or_insert(*count);
            }
        }
    }
}

impl Stats {
    // const EPSILON: i64 = 5;

    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(StatsInner {
            values: BTreeMap::new(),
            n: 0,
        })))
    }

    pub fn add_i64(&mut self, x: i64) {
        let mut inner = self.0.lock().unwrap();
        inner
            .values
            .entry(x)
            .and_modify(|curr| *curr += 1)
            .or_insert(1);
        inner.n += 1;
    }
}

impl Estimate for Stats {
    type Output = FlatStats;
    fn estimate(&self) -> Self::Output {
        let inner = &self.0.lock().unwrap();
        FlatStats {
            max: quartile(1.0, inner).unwrap(),
            min: quartile(0.5, inner).unwrap(),
            first_quartile: quartile(0.25, inner).unwrap(),
            third_quartile: quartile(0.75, inner).unwrap(),
            median: quartile(0.5, inner).unwrap(),
            mean: inner
                .values
                .iter()
                .map(|(value, count)| value * *count as i64)
                .sum::<i64>() as f64
                / inner.n as f64,
            count: inner.n,
        }
    }
}

fn quartile(quartile: f64, inner: &std::sync::MutexGuard<'_, StatsInner>) -> Result<i64, String> {
    let mut n = 0;
    let index = quartile * inner.n as f64;
    if index < 0.0 {
        return Err("Quartile less than 0.0, quartile must be between 0.0 and 1.0".into());
    }
    let index = index.floor() as u64;
    dbg!(index, inner.n, quartile);
    let mut current_value;
    for (value, count) in inner.values.iter() {
        current_value = *value;
        n += count;
        if n >= index {
            return Ok(current_value);
        }
    }
    // TODO : handle error
    Err("Quartile greater than 1.0, quartile must be between 0.0 and 1.0".into())
}

#[derive(Debug, Clone)]
pub struct FlatStats {
    max: i64,
    min: i64,
    mean: f64,
    median: i64,
    first_quartile: i64,
    third_quartile: i64,
    count: u64,
}

impl Estimate for FlatStats {
    type Output = rhai::Dynamic;
    fn estimate(&self) -> Self::Output {
        let vec: Vec<(_, rhai::Dynamic)> = vec![
            ("max".into(), self.max.into()),
            ("min".into(), self.min.into()),
            ("mean".into(), self.mean.into()),
            ("median".into(), self.median.into()),
            ("first_quartile".into(), self.first_quartile.into()),
            ("third_quartile".into(), self.third_quartile.into()),
            ("count".into(), rhai::Dynamic::from_int(self.count as i64)),
        ];
        rhai::Dynamic::from_map(vec.into_iter().collect())
    }
}
