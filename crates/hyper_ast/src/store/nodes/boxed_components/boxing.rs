use std::any::Any;
#[derive(Clone, Copy, Debug)]
pub struct CommitProcessorHandle(std::any::TypeId);
pub struct ErasedMap<V = Box<dyn ErasableComponent>>(
    std::collections::HashMap<std::any::TypeId, V>,
);
impl<V> Default for ErasedMap<V> {
    fn default() -> Self {
        Self(Default::default())
    }
}

unsafe impl<V> Send for ErasedMap<V> {}
unsafe impl<V> Sync for ErasedMap<V> {}

// Should not need to be public
pub trait ErasableComponent: Any + ToErasedComponent {}
pub trait ToErasedComponent {
    fn to_erasable_processor(self: Box<Self>) -> Box<dyn ErasableComponent>;
    fn as_mut_any(&mut self) -> &mut dyn Any;
    fn as_any(&self) -> &dyn Any;
}

// todo use downcast-rs
impl<T: ErasableComponent> ToErasedComponent for T {
    fn to_erasable_processor(self: Box<Self>) -> Box<dyn ErasableComponent> {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl<T> ErasableComponent for T where T: Any {}

#[allow(unused)]
// NOTE crazy good stuff for complex component handling
// a column oriented layout would allow this scheme to truly shine
impl ErasedMap<Box<dyn ErasableComponent>> {
    pub fn by_id_mut(
        &mut self,
        id: &CommitProcessorHandle,
    ) -> Option<&mut (dyn ErasableComponent + 'static)> {
        self.0.get_mut(&id.0).map(|x| x.as_mut())
    }
    pub fn by_id(&self, id: &CommitProcessorHandle) -> Option<&(dyn ErasableComponent + 'static)> {
        self.0.get(&id.0).map(|x| x.as_ref())
    }
    pub fn mut_or_default<T: 'static + ToErasedComponent + Default + Send + Sync>(
        &mut self,
    ) -> &mut T {
        let r = self
            .0
            .entry(std::any::TypeId::of::<T>())
            .or_insert_with(|| Box::new(T::default()).to_erasable_processor());
        let r = r.as_mut();
        let r = <dyn Any>::downcast_mut(r.as_mut_any());
        r.unwrap()
    }
    pub fn get<T: 'static + ToErasedComponent + Send + Sync>(&self) -> Option<&T> {
        let r = self.0.get(&std::any::TypeId::of::<T>())?;
        <dyn Any>::downcast_ref(r.as_any())
    }
}
