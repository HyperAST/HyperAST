use hyper_ast::types::Stored;

pub struct Arena<T>(stack_graphs::arena::Arena<T>);

// impl<'a, T> NodeStore<'a, T::TreeId, &'a T> for Arena<T>
// where
//     T: 'a + Stored<TreeId = stack_graphs::arena::Handle<T>>,
// {
//     fn resolve(&'a self, id: &T::TreeId) -> &'a T {
//         self.0.get(*id)
//     }
// }
// impl<'a, T> NodeStoreMut<'a, T, &'a T> for Arena<T>
// where
//     T: 'a + Stored<TreeId = stack_graphs::arena::Handle<T>>,
// {
//     fn get_or_insert(&mut self, node: T) -> <T as Stored>::TreeId {
//         todo!()
//     }
// }

impl<'a, T> Arena<T>
where
    T: 'a + Stored<TreeId = stack_graphs::arena::Handle<T>>,
{
    pub fn get_or_insert(&mut self, node: T) -> T::TreeId {
        self.0.add(node)
    }
}

impl<'a, T> Arena<T> {
    pub fn new() -> Self {
        Self(stack_graphs::arena::Arena::new())
    }
}

impl<'a, T> Into<stack_graphs::arena::Arena<T>> for Arena<T> {
    fn into(self) -> stack_graphs::arena::Arena<T> {
        self.0
    }
}

impl<'a, T> From<stack_graphs::arena::Arena<T>> for Arena<T> {
    fn from(a: stack_graphs::arena::Arena<T>) -> Self {
        Self(a)
    }
}
