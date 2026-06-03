use rhai::EvalAltResult;

use super::estimate::Estimate;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
pub(super) struct Quantile(Arc<Mutex<QuantileInner>>);

#[derive(Clone)]
struct QuantileInner {
    quartile: f64,
    values: BTreeMap<i64, u64>,
    n: u64,
}

impl average::Merge for Quantile {
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

impl Quantile {
    // const EPSILON: i64 = 5;

    pub fn new(quantile: f64) -> Result<Self, Box<EvalAltResult>> {
        if quantile > 1.0 || quantile < 0.0 {
            Err("Quartile must be between 0.0 and 1.0".into())
        } else {
            Ok(Self(Arc::new(Mutex::new(QuantileInner {
                quartile: quantile,
                values: BTreeMap::new(),
                n: 0,
            }))))
        }
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

impl Estimate for Quantile {
    type Output = Result<i64, String>;
    fn estimate(&self) -> Self::Output {
        let inner = self.0.lock().unwrap();
        let mut n = 0;
        let index = inner.quartile * inner.n as f64;
        if index < 0.0 {
            return Err("Quartile less than 0.0, quartile must be between 0.0 and 1.0".into());
        }
        let index = index.floor() as u64;
        dbg!(index, inner.n, inner.quartile);
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
}
