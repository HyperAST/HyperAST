use super::estimate::Estimate;
use serde::Serialize;
use std::fmt::Display;

#[derive(Default, Clone)]
pub(super) struct Mean {
    sum: i64,
    n: u64,
}

impl Display for Mean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.estimate())
    }
}

impl average::Merge for Mean {
    fn merge(&mut self, other: &Self) {
        self.sum += other.sum;
        self.n += other.n;
    }
}

impl Estimate for Mean {
    type Output = f64;
    fn estimate(&self) -> Self::Output {
        match self.sum {
            i64::MAX => f64::INFINITY,
            i64::MIN => f64::NEG_INFINITY,
            _ => self.sum as f64 / self.n as f64,
        }
    }
}

impl Mean {
    pub fn add_i64(&mut self, x: i64) {
        self.n += 1;
        self.sum += x;
    }
}
impl<'de> Serialize for Mean {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let x = self.estimate();
        serializer.serialize_f64(x)
    }
}
