use super::estimate::Estimate;

#[derive(Clone)]
pub(super) struct Max {
    max: i64,
}

impl Default for Max {
    fn default() -> Self {
        Max { max: i64::MIN }
    }
}

impl std::fmt::Display for Max {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.estimate())
    }
}

impl average::Merge for Max {
    fn merge(&mut self, other: &Self) {
        if other.max > self.max {
            self.max = other.max;
        }
    }
}

impl Max {
    pub fn add_i64(&mut self, x: i64) {
        if x > self.max {
            self.max = x;
        }
    }
}

impl Estimate for Max {
    type Output = i64;
    fn estimate(&self) -> Self::Output {
        self.max
    }
}

impl<'de> serde::Serialize for Max {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let x = self.estimate();
        serializer.serialize_i64(x)
    }
}
