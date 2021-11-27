use rusted_gumtree_core::tree::tree::{NodeStore, Stored};

pub struct Arena<T>(stack_graphs::arena::Arena<T>);

impl<'a, T> NodeStore<'a, T> for Arena<T>
where
    T: 'a + Stored<TreeId = stack_graphs::arena::Handle<T>>,
{
    type D = &'a T;

    fn get_or_insert(&mut self, node: T) -> T::TreeId {
        self.0.add(node)
    }

    fn resolve(&'a self, id: &T::TreeId) -> Self::D {
        self.0.get(*id)
    }
}

impl<'a, T> Arena<T> {
    pub fn new() -> Self {
        Self(stack_graphs::arena::Arena::new())
    }
}
