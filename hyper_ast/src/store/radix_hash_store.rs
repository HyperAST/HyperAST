use std::{cell::Ref, mem::MaybeUninit, ops::Index};

use stack_graphs::arena::Handle;

// pub trait Store<'a,T:'a>: Index<Handle<T>,Output = Ref<'a,T>> {

//     fn get_or_insert(&mut self) -> Handle<T>;

// }

pub struct Store<T> {
    int: MaybeUninit<T>,
}

impl<T> Store<T> {
    fn get_or_insert(&mut self, v: T) -> Handle<T> {
        todo!()
    }
}

impl<T: 'static> Index<Handle<T>> for Store<T> {
    type Output = Ref<'static, T>;

    fn index(&self, index: Handle<T>) -> &Self::Output {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
