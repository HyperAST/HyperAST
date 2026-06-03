use rhai::Dynamic;

use super::finalize::Finalize;

#[derive(Clone)]
pub(super) struct NamedContainer<T> {
    name: String,
    pub(super) content: T,
}

impl<T> NamedContainer<T> {
    pub(super) fn new(name: String, content: T) -> Self {
        Self { name, content }
    }
}

impl Finalize for NamedContainer<Dynamic> {
    type Output = rhai::Array;
    fn finalize(self) -> Self::Output {
        let mut r = rhai::Array::new();

        r.push(self.name.into());
        r.push(self.content.finalize());

        r
    }
}
