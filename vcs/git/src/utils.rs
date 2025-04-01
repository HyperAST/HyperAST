pub struct TypeMap<V: std::any::Any = Box<dyn std::any::Any>>(
    std::collections::HashMap<std::any::TypeId, V>,
);
impl<V: std::any::Any> Default for TypeMap<V> {
    fn default() -> Self {
        Self(Default::default())
    }
}

unsafe impl Send for TypeMap {}
unsafe impl Sync for TypeMap {}

impl TypeMap {
    pub fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.0
            .get(&std::any::TypeId::of::<T>())
            .and_then(|x| x.downcast_ref())
    }
    pub fn get_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.0
            .get_mut(&std::any::TypeId::of::<T>())
            .and_then(|x| x.downcast_mut())
    }
    pub fn mut_or_default<T: 'static + Default + Send + Sync>(&mut self) -> &mut T {
        self.0
            .entry(std::any::TypeId::of::<T>())
            .or_insert_with(|| Box::new(T::default()))
            .downcast_mut()
            .unwrap()
    }
    pub fn insert<T: 'static + Send + Sync>(&mut self, x: T) -> Option<T> {
        self.0
            .insert(std::any::TypeId::of::<T>(), Box::new(x))
            .and_then(|x| x.downcast().ok().map(|x| *x))
    }
    pub fn clear(&mut self) {
        self.0.clear()
    }
}
impl<V: std::any::Any> TypeMap<V> {
    // pub fn by_id(&mut self, id: std::any::TypeId) -> Option<&mut dyn Any> {
    //     self.0
    //         .get_mut(&id)
    // }
    // pub fn mut_or_default2<T: 'static + Any + Default + Send + Sync>(&mut self) -> &mut T {
    //     let r = self
    //         .0
    //         .entry(std::any::TypeId::of::<T>())
    //         .or_insert_with(|| {
    //             let d: Box<dyn Any> = Box::new(T::default());
    //             d
    //         });
    //     // .downcast_mut()
    //     // .unwrap()
    //     todo!()
    // }
}

pub struct TypeMap2(std::collections::HashMap<std::any::TypeId, Box<dyn std::any::Any>>);
impl Default for TypeMap2 {
    fn default() -> Self {
        Self(Default::default())
    }
}

unsafe impl Send for TypeMap2 {}
unsafe impl Sync for TypeMap2 {}

impl TypeMap2 {
    pub fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.0
            .get(&std::any::TypeId::of::<T>())
            .and_then(|x| x.downcast_ref())
    }
    pub fn get_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.0
            .get_mut(&std::any::TypeId::of::<T>())
            .and_then(|x| x.downcast_mut())
    }
    pub fn mut_or_default<T: 'static + Default + Send + Sync>(&mut self) -> &mut T {
        self.0
            .entry(std::any::TypeId::of::<T>())
            .or_insert_with(|| Box::new(T::default()))
            .downcast_mut()
            .unwrap()
    }
    pub fn insert<T: 'static + Send + Sync>(&mut self, x: T) -> Option<T> {
        self.0
            .insert(std::any::TypeId::of::<T>(), Box::new(x))
            .and_then(|x| x.downcast().ok().map(|x| *x))
    }
    pub fn clear(&mut self) {
        self.0.clear()
    }
}

pub struct TypeMap3<V>(std::collections::HashMap<std::any::TypeId, V>);
impl<V> Default for TypeMap3<V> {
    fn default() -> Self {
        Self(Default::default())
    }
}

unsafe impl<V: Send> Send for TypeMap3<V> {}
unsafe impl<V: Sync> Sync for TypeMap3<V> {}

pub trait GG: ToGG {
    fn compute(&self);
}
pub trait ToGG {
    fn to_gg(self: Box<Self>) -> Box<dyn GG>;
}

impl TypeMap3<Box<dyn GG>> {
    pub fn by_id<T: 'static + Send + Sync>(&mut self) -> Option<&mut (dyn GG + 'static)> {
        self.0
            .get_mut(&std::any::TypeId::of::<T>())
            .map(|x| x.as_mut())
    }
    pub fn mut_or_default<T: 'static + ToGG + Default + Send + Sync>(&mut self) -> &mut dyn GG {
        self.0
            .entry(std::any::TypeId::of::<T>())
            .or_insert_with(|| Box::new(T::default()).to_gg())
            .as_mut()
    }
}
