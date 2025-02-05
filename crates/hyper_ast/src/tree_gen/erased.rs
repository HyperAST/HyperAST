use std::{marker::PhantomData, ops::Deref};

#[derive(Clone)]
pub struct ConfigParameters(std::rc::Rc<dyn std::any::Any>);
pub trait Parametrized<S: Source, V: Version>: ParametrizedCommitProc<S, V> {
    type T: 'static;
    // Register a parameter to later be used by the processor,
    // each identical (structurally) parameter should identify exactly one processor
    fn register_param(&mut self, t: Self::T) -> ParametrizedCommitProcessorHandle;
}
#[derive(Clone, Copy, Debug)]
pub struct ConfigParametersHandle(pub usize);
#[derive(Clone, Copy, Debug)]
pub struct ParametrizedCommitProcessorHandle(
    pub VersionProcessorHandle,
    pub ConfigParametersHandle,
);
#[derive(Clone, Copy, Debug)]
pub struct VersionProcessorHandle(std::any::TypeId);
#[derive(Debug)]
pub struct ParametrizedCommitProcessor2Handle<T>(
    pub ConfigParametersHandle,
    pub(crate) PhantomData<T>,
);
impl<T> Clone for ParametrizedCommitProcessor2Handle<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}
impl<T> Copy for ParametrizedCommitProcessor2Handle<T> {}
impl<T: VersionProcExt> ParametrizedCommitProcessor2Handle<T> {
    fn recover_handle(&self) -> ParametrizedCommitProcessorHandle {
        ParametrizedCommitProcessorHandle(
            VersionProcessorHandle(std::any::TypeId::of::<T::Holder>()),
            self.0,
        )
    }
}

impl<T> Deref for ParametrizedCommitProcessor2Handle<T> {
    type Target = ConfigParametersHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait Version {
    type Builder;
    type Id;
}

pub trait Source {
    /// e.g. Git's object id + name
    /// or file path + timestamp
    type Id;
}

pub trait VersionProc<S: Source, V: Version>: std::any::Any + std::fmt::Debug {
    fn get_version(&self, id: S::Id) -> Option<&V>;
    fn as_mut_any(&mut self) -> &mut dyn std::any::Any;
}

struct SourceProcessor<HAST, S, V> {
    pub main_stores: HAST,
    pub processing_systems: ProcessorMap<(S, V)>,
}

pub trait PreparedVersionProc<HAST, S: Source, V: Version> {
    fn process(self: Box<Self>, prepro: &mut SourceProcessor<HAST, S, V>) -> V::Id;
}

pub trait VersionProcExt: VersionProc<Self::S, Self::V> {
    type S: Source;
    type V: Version;
    type Holder: ParametrizedCommitProc<Self::S, Self::V> + Parametrized<Self::S, Self::V>;
    fn register_param(
        h: &mut Self::Holder,
        t: <Self::Holder as Parametrized<Self::S, Self::V>>::T,
    ) -> ParametrizedCommitProcessor2Handle<Self>
    where
        Self: Sized,
    {
        ParametrizedCommitProcessor2Handle(h.register_param(t).1, PhantomData)
    }
}

pub trait ParametrizedCommitProc<S: Source, V: Version>: std::any::Any {
    fn erased_handle(&self) -> VersionProcessorHandle
    where
        Self: 'static,
    {
        VersionProcessorHandle(std::any::TypeId::of::<Self>())
    }

    fn get_mut(&mut self, parameters: ConfigParametersHandle) -> &mut impl VersionProc<S, V>;
    fn get(&self, parameters: ConfigParametersHandle) -> &impl VersionProc<S, V>;
}

pub trait ParametrizedCommitProc2<S: Source, V: Version>: ParametrizedCommitProc<S, V> {
    type Proc: VersionProcExt<S = S, V = V>;
    fn with_parameters(&self, parameters: ConfigParametersHandle) -> &Self::Proc;
    fn with_parameters_mut(&mut self, parameters: ConfigParametersHandle) -> &mut Self::Proc;
}

// impl<T: ParametrizedCommitProc2<S, V>, S: Source, V: Version> ParametrizedCommitProc<S, V> for T {
//     fn get_mut(&mut self, parameters: ConfigParametersHandle) -> &mut impl VersionProc<S, V> {
//         ParametrizedCommitProc2::with_parameters_mut(self, parameters)
//     }

//     fn get(&self, parameters: ConfigParametersHandle) -> &impl VersionProc<S, V> {
//         ParametrizedCommitProc2::with_parameters(self, parameters)
//     }
// }

pub struct ProcessorMap<V>(std::collections::HashMap<std::any::TypeId, V>);

#[test]
fn t() {
    use std::any::Any;
    #[derive(Clone, PartialEq, Eq, Debug)]
    struct S(u8);
    #[derive(Clone, PartialEq, Eq)]
    struct S0(u8);
    #[derive(Default)]
    struct P0(Vec<P>);
    #[derive(Debug)]
    struct P(S);
    struct Oid;
    struct Repo;
    impl Source for Repo {
        type Id = Oid;
    }
    struct IdN;
    struct Bld;
    struct Commit;
    impl Version for Commit {
        type Builder = Bld;

        type Id = IdN;
    }
    // impl Parametrized<Repo, Commit> for P0 {
    //     type T = S;
    //     fn register_param(&mut self, t: Self::T) -> ParametrizedCommitProcessorHandle {
    //         let l = self.0.iter().position(|x| &x.0 == &t).unwrap_or_else(|| {
    //             let l = self.0.len();
    //             self.0.push(P(t));
    //             l
    //         });
    //         ParametrizedCommitProcessorHandle(self.erased_handle(), ConfigParametersHandle(l))
    //     }
    // }
    impl VersionProc<Repo, Commit> for P {
        // fn prepare_processing(
        //     &self,
        //     repository: &Repo,
        //     tree_oid: Bld,
        // ) -> Box<dyn PreparedVersionProc<IdN>> {
        //     unimplemented!("required for processing at the root of a project")
        // }

        fn get_version(&self, commit_oid: Oid) -> Option<&Commit> {
            unimplemented!("required for processing at the root of a project")
        }

        fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }
    
    impl ProcessorWithParam<Repo, Commit> for P0 {
        type P = S;
        fn register_param(&mut self, param: Self::P) -> ParametrizedCommitProcessorHandle {
            todo!()
        }
        
        fn by_param(&mut self, handle: ParametrizedCommitProcessorHandle) -> () {
            todo!()
        }
    }

    // impl ParametrizedCommitProc<Repo, Commit> for P0 {
    //     fn get_mut(
    //         &mut self,
    //         parameters: ConfigParametersHandle,
    //     ) -> &mut dyn VersionProc<Repo, Commit> {
    //         &mut self.0[parameters.0]
    //     }
    //     fn get(&self, parameters: ConfigParametersHandle) -> &dyn VersionProc<Repo, Commit> {
    //         &self.0[parameters.0]
    //     }
    // }

    impl<V> Default for ProcessorMap<V> {
        fn default() -> Self {
            Self(Default::default())
        }
    }

    unsafe impl<V: Send> Send for ProcessorMap<V> {}
    unsafe impl<V: Sync> Sync for ProcessorMap<V> {}

    // Should not need to be public

    // pub trait ErasableProcessor<S: Source, V: Version>: Any + VersionProc<S, V> {}
    // pub trait ToErasedProc {
    //     fn as_mut_any(&mut self) -> &mut dyn Any;
    //     fn as_any(&self) -> &dyn Any;
    // }

    // impl<T: ErasableProcessor> ToErasedProc for T {
    //     fn as_mut_any(&mut self) -> &mut dyn Any {
    //         self
    //     }
    //     fn as_any(&self) -> &dyn Any {
    //         self
    //     }
    // }
    // impl<T> ErasableProcessor for T where T: Any + ParametrizedCommitProc<Repo, Commit> {}

    // NOTE crazy good stuff
    impl<S: Source + 'static, V: Version + 'static> ProcessorMap<Box<dyn Processor<S, V>>> {
        pub fn by_id(
            &mut self,
            id: &std::any::TypeId,
        ) -> Option<&mut (dyn Processor<S, V> + 'static)> {
            self.0.get_mut(id).map(|x| x.as_mut())
        }
        pub fn by_id_mut(
            &mut self,
            id: &std::any::TypeId,
        ) -> Option<&mut (dyn Processor<S, V> + 'static)> {
            self.0.get_mut(&id).map(|x| x.as_mut())
        }
        pub fn mut_or_default<T: 'static + Processor<S, V> + Default + Send + Sync>(
            &mut self,
        ) -> &mut T {
            let r = self
                .0
                .entry(std::any::TypeId::of::<T>())
                .or_insert_with(|| Box::new(T::default()));
            let r = r.as_mut();

            let r = <dyn Any>::downcast_mut(r.as_mut_any());
            r.unwrap()
        }
    }

    let mut h = ProcessorMap::<Box<dyn Processor<Repo, Commit>>>::default();
    // The registered parameter is type checked
    let hh = h.mut_or_default::<P0>();
    let hhh = hh.register_param(S(42));

    // You can easily store hh in any collection.
    // You can easily add a method to CommitProc.
    // h.by_id(&hh.0 .0).unwrap().get_mut(hh.1).p();
}

trait Processor<S, V>: std::any::Any {
    #[doc(hidden)]
    fn as_mut_any(&mut self) -> &mut dyn std::any::Any;
}

trait ProcessorWithParam<S, V> {
    type P;
    fn by_param(&mut self, handle: ParametrizedCommitProcessorHandle) -> ();
    /// avoid expensive comparison of param,
    /// NOTE use a different start point for handles per Processor to help detect handles used on wrong 
    fn register_param(&mut self, param: Self::P) -> ParametrizedCommitProcessorHandle;
}

impl<T, S, V> Processor<S, V> for T where T: ProcessorWithParam<S, V> + 'static {
    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// pub type ProcessorMap<S, V> = spreaded::ProcessorMap<Box<dyn spreaded::ErasableProcessor<S, V>>>;
// pub use spreaded::ErasableProcessor;

use crate::tree_gen;

// mod spreaded {
//     use std::any::Any;

//     use super::*;

//     // TODO benchmark vs using a Vec and comparing each TypeId ? dichotomy
//     pub struct ProcessorMap<V>(std::collections::HashMap<std::any::TypeId, V>);
//     impl<V> Default for ProcessorMap<V> {
//         fn default() -> Self {
//             Self(Default::default())
//         }
//     }
//     impl<V> ProcessorMap<V> {
//         pub(crate) fn clear(&mut self) {
//             self.0.clear()
//         }
//     }

//     unsafe impl<V> Send for ProcessorMap<V> {}
//     unsafe impl<V> Sync for ProcessorMap<V> {}

//     // Should not need to be public
//     pub trait ErasableProcessor<S: Source, V: Version>:
//         Any + ToErasedProc<S, V> + ParametrizedCommitProc<S, V>
//     {
//     }
//     pub trait ToErasedProc<S, V> {
//         fn to_erasable_processor(self: Box<Self>) -> Box<dyn ErasableProcessor<S, V>>;
//         fn as_mut_any(&mut self) -> &mut dyn Any;
//         fn as_any(&self) -> &dyn Any;
//     }

//     impl<T: ErasableProcessor<S, V>, S: Source, V: Version> ToErasedProc<S, V> for T {
//         fn to_erasable_processor(self: Box<Self>) -> Box<dyn ErasableProcessor<S, V>> {
//             self
//         }
//         fn as_mut_any(&mut self) -> &mut dyn Any {
//             self
//         }
//         fn as_any(&self) -> &dyn Any {
//             self
//         }
//     }
//     impl<T, S: Source, V: Version> ErasableProcessor<S, V> for T where
//         T: Any + ParametrizedCommitProc<S, V>
//     {
//     }
//     // NOTE crazy good stuff
//     impl<S, V> ProcessorMap<Box<dyn ErasableProcessor<S, V>>> {
//         pub fn by_id_mut(
//             &mut self,
//             id: &VersionProcessorHandle,
//         ) -> Option<&mut (dyn ErasableProcessor<S, V> + 'static)> {
//             self.0.get_mut(&id.0).map(|x| x.as_mut())
//         }
//         pub fn by_id(
//             &self,
//             id: &VersionProcessorHandle,
//         ) -> Option<&(dyn ErasableProcessor<S, V> + 'static)> {
//             self.0.get(&id.0).map(|x| x.as_ref())
//         }
//         pub fn mut_or_default<T: 'static + ToErasedProc<S, V> + Default + Send + Sync>(
//             &mut self,
//         ) -> &mut T {
//             let r = self
//                 .0
//                 .entry(std::any::TypeId::of::<T>())
//                 .or_insert_with(|| Box::new(T::default()).to_erasable_processor());
//             let r = r.as_mut();
//             let r = <dyn Any>::downcast_mut(r.as_mut_any());
//             r.unwrap()
//         }
//         pub fn get<T: 'static + ToErasedProc<S, V> + Default + Send + Sync>(&self) -> Option<&T> {
//             let r = self.0.get(&std::any::TypeId::of::<T>())?;
//             <dyn Any>::downcast_ref(r.as_any())
//         }
//     }
//     #[test]
//     fn a() {
//         #[derive(Clone, PartialEq, Eq, Debug)]
//         struct S(u8);
//         #[derive(Clone, PartialEq, Eq)]
//         struct S0(u8);
//         #[derive(Default)]
//         struct P0(Vec<P>);
//         #[derive(Debug)]
//         struct P(S);
//         struct Oid;
//         struct Repo;
//         impl Source for Repo {
//             type Id = Oid;
//         }
//         struct IdN;
//         struct Bld;
//         struct Commit;
//         impl Version for Commit {
//             type Builder = Bld;

//             type Id = IdN;
//         }
//         impl Parametrized<Repo, Commit> for P0 {
//             type T = S;
//             fn register_param(&mut self, t: Self::T) -> ParametrizedCommitProcessorHandle {
//                 let l = self.0.iter().position(|x| &x.0 == &t).unwrap_or_else(|| {
//                     let l = self.0.len();
//                     self.0.push(P(t));
//                     l
//                 });
//                 ParametrizedCommitProcessorHandle(self.erased_handle(), ConfigParametersHandle(l))
//             }
//         }
//         impl VersionProc<Repo, Commit> for P {
//             fn prepare_processing(
//                 &self,
//                 _repository: &Repo,
//                 _builder: Bld,
//             ) -> Box<dyn PreparedVersionProc<IdN>> {
//                 unimplemented!("required for processing at the root of a project")
//             }

//             fn get_version(&self, _id: Oid) -> Option<&Commit> {
//                 unimplemented!("required for processing at the root of a project")
//             }
//         }
//         impl VersionProcExt for P {
//             type S = Repo;

//             type V = Commit;

//             type Holder = P0;
//         }

//         impl ParametrizedCommitProc2<Repo, Commit> for P0 {
//             type Proc = P;
//             fn with_parameters(&self, parameters: ConfigParametersHandle) -> &Self::Proc {
//                 &self.0[parameters.0]
//             }
//             fn with_parameters_mut(
//                 &mut self,
//                 parameters: ConfigParametersHandle,
//             ) -> &mut Self::Proc {
//                 &mut self.0[parameters.0]
//             }
//         }

//         let mut h = ProcessorMap::<Box<dyn ErasableProcessor<Repo, Commit>>>::default();
//         // The registered parameter is type checked
//         let hh = h.mut_or_default::<P0>().register_param(S(42));
//         // You can easily store hh in any collection.
//         // You can easily add a method to CommitProc.
//         h.by_id_mut(&hh.0).unwrap().get_mut(hh.1).p();
//     }
// }
