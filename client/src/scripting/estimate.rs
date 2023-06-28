pub(crate) trait Estimate {
    type Output;
    fn estimate(&self) -> Self::Output;
}