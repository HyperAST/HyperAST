use super::estimate::Estimate;

#[derive(Clone)]
pub(super) struct Min {
    min: i64,
}

impl Default for Min {
    fn default() -> Self {
        Self { min: i64::MAX }
    }
}

impl std::fmt::Display for Min {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.estimate())
    }
}

impl average::Merge for Min {
    fn merge(&mut self, other: &Self) {
        if other.min < self.min {
            self.min = other.min;
        }
    }
}

impl Min {
    pub fn add_i64(&mut self, x: i64) {
        if x < self.min {
            self.min = x;
        }
    }
}

impl Estimate for Min {
    type Output = i64;
    fn estimate(&self) -> Self::Output {
        self.min
    }
}

impl<'de> serde::Serialize for Min {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let x = self.estimate();
        serializer.serialize_i64(x)
    }
}
