use std::{any::Any, marker::PhantomData, ops::Deref};

#[derive(Clone)]
#[allow(unused)]
pub struct ConfigParameters(std::rc::Rc<dyn std::any::Any>);
pub trait Parametrized: ParametrizedCommitProc {
    type T: 'static;
    // Register a parameter to later be used by the processor,
    // each identical (structurally) parameter should identify exactly one processor
    fn register_param(&mut self, t: Self::T) -> ParametrizedCommitProcessorHandle;
}
#[derive(Clone, Copy, Debug)]
pub struct ConfigParametersHandle(pub usize);
#[derive(Clone, Copy, Debug)]
pub struct ParametrizedCommitProcessorHandle(pub CommitProcessorHandle, pub ConfigParametersHandle);
#[derive(Clone, Copy, Debug)]
pub struct CommitProcessorHandle(pub(crate) std::any::TypeId);
#[derive(Debug)]
pub struct ParametrizedCommitProcessor2Handle<T: CommitProcExt>(
    pub ConfigParametersHandle,
    pub(crate) PhantomData<T>,
);

impl<T: CommitProcExt> Eq for ParametrizedCommitProcessor2Handle<T> {}

impl<T: CommitProcExt> PartialEq for ParametrizedCommitProcessor2Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.0 == other.0.0 && self.1 == other.1
    }
}

impl<T: CommitProcExt> Clone for ParametrizedCommitProcessor2Handle<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}
impl<T: CommitProcExt> Copy for ParametrizedCommitProcessor2Handle<T> {}
impl<T: CommitProcExt> ParametrizedCommitProcessor2Handle<T> {
    #[allow(unused)]
    fn recover_handle(&self) -> ParametrizedCommitProcessorHandle {
        ParametrizedCommitProcessorHandle(
            CommitProcessorHandle(std::any::TypeId::of::<T::Holder>()),
            self.0,
        )
    }
}
impl<T: CommitProcExt> Deref for ParametrizedCommitProcessor2Handle<T> {
    type Target = ConfigParametersHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
pub trait CommitProc {
    // TODO remove
    fn p(&self) {
        dbg!()
    }
    fn prepare_processing<'repo>(
        &self,
        repository: &'repo git2::Repository,
        commit_builder: crate::preprocessed::CommitBuilder,
        param_handle: ParametrizedCommitProcessorHandle,
    ) -> Box<dyn PreparedCommitProc + 'repo>;

    fn commit_count(&self) -> usize;
    fn get_commit(&self, commit_oid: git2::Oid) -> Option<&crate::Commit>;
    fn get_precomp_query(&self) -> Option<hyperast_tsquery::ZeroSepArrayStr> {
        None
    }
    fn get_lang_handle(&self, _lang: &str) -> Option<ParametrizedCommitProcessorHandle> {
        None
    }
}
pub trait PreparedCommitProc {
    fn process(
        self: Box<Self>,
        prepro: &mut crate::preprocessed::RepositoryProcessor,
    ) -> NodeIdentifier;
}
pub trait CommitProcExt: CommitProc {
    type Holder: ParametrizedCommitProc + Parametrized;
    fn register_param(
        h: &mut Self::Holder,
        t: <Self::Holder as Parametrized>::T,
    ) -> ParametrizedCommitProcessor2Handle<Self>
    where
        Self: Sized,
    {
        ParametrizedCommitProcessor2Handle(h.register_param(t).1, PhantomData)
    }
}
pub trait ParametrizedCommitProc: std::any::Any {
    fn erased_handle(&self) -> CommitProcessorHandle
    where
        Self: 'static,
    {
        CommitProcessorHandle(std::any::TypeId::of::<Self>())
    }

    fn get_mut(&mut self, parameters: ConfigParametersHandle) -> &mut dyn CommitProc;
    fn get(&self, parameters: ConfigParametersHandle) -> &dyn CommitProc;
}

pub trait ParametrizedCommitProc2: ParametrizedCommitProc {
    type Proc: CommitProcExt;
    fn with_parameters(&self, parameters: ConfigParametersHandle) -> &Self::Proc;
    fn with_parameters_mut(&mut self, parameters: ConfigParametersHandle) -> &mut Self::Proc;
}

impl<T: ParametrizedCommitProc2> ParametrizedCommitProc for T {
    fn get_mut(&mut self, parameters: ConfigParametersHandle) -> &mut dyn CommitProc {
        ParametrizedCommitProc2::with_parameters_mut(self, parameters)
    }

    fn get(&self, parameters: ConfigParametersHandle) -> &dyn CommitProc {
        ParametrizedCommitProc2::with_parameters(self, parameters)
    }
}

#[test]
#[allow(unused)]
fn t() {
    #[derive(Clone, PartialEq, Eq)]
    struct S(u8);
    #[derive(Clone, PartialEq, Eq)]
    struct S0(u8);
    #[derive(Default)]
    struct P0(Vec<P>);
    struct P(S);
    impl Parametrized for P0 {
        type T = S;
        fn register_param(&mut self, t: Self::T) -> ParametrizedCommitProcessorHandle {
            let l = self.0.iter().position(|x| &x.0 == &t).unwrap_or_else(|| {
                let l = self.0.len();
                self.0.push(P(t));
                l
            });
            ParametrizedCommitProcessorHandle(self.erased_handle(), ConfigParametersHandle(l))
        }
    }
    impl CommitProc for P {
        fn prepare_processing(
            &self,
            repository: &git2::Repository,
            tree_oid: crate::preprocessed::CommitBuilder,
            param_handle: ParametrizedCommitProcessorHandle,
        ) -> Box<dyn PreparedCommitProc> {
            unimplemented!("required for processing at the root of a project")
        }

        fn get_commit(&self, commit_oid: git2::Oid) -> Option<&crate::Commit> {
            unimplemented!("required for processing at the root of a project")
        }

        fn commit_count(&self) -> usize {
            unimplemented!()
        }
    }
    impl ParametrizedCommitProc for P0 {
        fn get_mut(&mut self, parameters: ConfigParametersHandle) -> &mut dyn CommitProc {
            &mut self.0[parameters.0]
        }
        fn get(&self, parameters: ConfigParametersHandle) -> &dyn CommitProc {
            &self.0[parameters.0]
        }
    }

    pub struct ProcessorMap<V>(std::collections::HashMap<std::any::TypeId, V>);
    impl<V> Default for ProcessorMap<V> {
        fn default() -> Self {
            Self(Default::default())
        }
    }

    unsafe impl<V: Send> Send for ProcessorMap<V> {}
    unsafe impl<V: Sync> Sync for ProcessorMap<V> {}

    // Should not need to be public
    pub trait ErasableProcessor: Any + ToErasedProc + ParametrizedCommitProc {}
    pub trait ToErasedProc {
        fn to_erasable_processor(self: Box<Self>) -> Box<dyn ErasableProcessor>;
        fn as_mut_any(&mut self) -> &mut dyn Any;
        fn as_any(&self) -> &dyn Any;
    }

    impl<T: ErasableProcessor> ToErasedProc for T {
        fn to_erasable_processor(self: Box<Self>) -> Box<dyn ErasableProcessor> {
            self
        }
        fn as_mut_any(&mut self) -> &mut dyn Any {
            self
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    impl<T> ErasableProcessor for T where T: Any + ParametrizedCommitProc {}

    // NOTE crazy good stuff
    impl ProcessorMap<Box<dyn ErasableProcessor>> {
        pub fn by_id(
            &mut self,
            id: &std::any::TypeId,
        ) -> Option<&mut (dyn ErasableProcessor + 'static)> {
            self.0.get_mut(id).map(|x| x.as_mut())
        }
        pub fn mut_or_default<T: 'static + ToErasedProc + Default + Send + Sync>(
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
    }

    let mut h = ProcessorMap::<Box<dyn ErasableProcessor>>::default();
    // The registered parameter is type checked
    let hh = h.mut_or_default::<P0>().register_param(S(42));
    // You can easily store hh in any collection.
    // You can easily add a method to CommitProc.
    h.by_id(&hh.0.0).unwrap().get_mut(hh.1).p();
}

pub type ProcessorMap = spreaded::ProcessorMap<Box<dyn spreaded::ErasableProcessor>>;
use hyperast::store::defaults::NodeIdentifier;
pub use spreaded::ErasableProcessor;

mod spreaded {
    use super::*;

    pub struct ProcessorMap<V>(std::collections::HashMap<std::any::TypeId, V>);
    impl<V> Default for ProcessorMap<V> {
        fn default() -> Self {
            Self(Default::default())
        }
    }
    impl<V> ProcessorMap<V> {
        pub(crate) fn clear(&mut self) {
            self.0.clear()
        }
    }

    unsafe impl<V> Send for ProcessorMap<V> {}
    unsafe impl<V> Sync for ProcessorMap<V> {}

    // Should not need to be public
    pub trait ErasableProcessor: Any + ToErasedProc + ParametrizedCommitProc {}
    pub trait ToErasedProc {
        fn to_erasable_processor(self: Box<Self>) -> Box<dyn ErasableProcessor>;
        fn as_mut_any(&mut self) -> &mut dyn Any;
        fn as_any(&self) -> &dyn Any;
    }

    impl<T: ErasableProcessor> ToErasedProc for T {
        fn to_erasable_processor(self: Box<Self>) -> Box<dyn ErasableProcessor> {
            self
        }
        fn as_mut_any(&mut self) -> &mut dyn Any {
            self
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    impl<T> ErasableProcessor for T where T: Any + ParametrizedCommitProc {}
    // NOTE crazy good stuff
    impl ProcessorMap<Box<dyn ErasableProcessor>> {
        pub fn by_id_mut(
            &mut self,
            id: &CommitProcessorHandle,
        ) -> Option<&mut (dyn ErasableProcessor + 'static)> {
            self.0.get_mut(&id.0).map(|x| x.as_mut())
        }
        pub fn by_id(
            &self,
            id: &CommitProcessorHandle,
        ) -> Option<&(dyn ErasableProcessor + 'static)> {
            self.0.get(&id.0).map(|x| x.as_ref())
        }
        pub fn mut_or_default<T: 'static + ToErasedProc + Default + Send + Sync>(
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
        pub fn get<T: 'static + ToErasedProc + Default + Send + Sync>(&self) -> Option<&T> {
            let r = self.0.get(&std::any::TypeId::of::<T>())?;
            <dyn Any>::downcast_ref(r.as_any())
        }
        // pub fn mut_or_default_with_param<T: 'static + CommitProcExt>(
        //     &mut self,
        //     handle: ParametrizedCommitProcessor2Handle<T>,
        // ) -> &mut T {
        //     let r = self
        //         .0
        //         .entry(std::any::TypeId::of::<T>())
        //         .or_insert_with(|| Box::new(T::default()).to_erasable_processor());
        //     let r = r.as_mut();
        //     let r = <dyn Any>::downcast_mut(r.as_mut_any());
        //     r.unwrap()
        // }
    }
    #[allow(unused)]
    #[test]
    fn a() {
        #[derive(Clone, PartialEq, Eq)]
        struct S(u8);
        #[derive(Clone, PartialEq, Eq)]
        struct S0(u8);
        #[derive(Default)]
        struct P0(Vec<P>);
        struct P(S);
        impl Parametrized for P0 {
            type T = S;
            fn register_param(&mut self, t: Self::T) -> ParametrizedCommitProcessorHandle {
                let l = self.0.iter().position(|x| &x.0 == &t).unwrap_or_else(|| {
                    let l = self.0.len();
                    self.0.push(P(t));
                    l
                });
                ParametrizedCommitProcessorHandle(self.erased_handle(), ConfigParametersHandle(l))
            }
        }
        impl CommitProc for P {
            fn prepare_processing(
                &self,
                repository: &git2::Repository,
                tree_oid: crate::preprocessed::CommitBuilder,
                param_handle: ParametrizedCommitProcessorHandle,
            ) -> Box<dyn PreparedCommitProc> {
                unimplemented!()
            }

            fn get_commit(&self, commit_oid: git2::Oid) -> Option<&crate::Commit> {
                unimplemented!()
            }

            fn commit_count(&self) -> usize {
                unimplemented!()
            }
        }
        impl CommitProcExt for P {
            type Holder = P0;
        }
        // impl ParametrizedCommitProc for P0 {
        //     fn get(&mut self, parameters: ConfigParametersHandle) -> &mut dyn CommitProc {
        //         &mut self.0[parameters.0]
        //     }
        // }

        impl ParametrizedCommitProc2 for P0 {
            type Proc = P;
            fn with_parameters(&self, parameters: ConfigParametersHandle) -> &Self::Proc {
                &self.0[parameters.0]
            }
            fn with_parameters_mut(
                &mut self,
                parameters: ConfigParametersHandle,
            ) -> &mut Self::Proc {
                &mut self.0[parameters.0]
            }
        }

        let mut h = ProcessorMap::<Box<dyn ErasableProcessor>>::default();
        // The registered parameter is type checked
        let hh = h.mut_or_default::<P0>().register_param(S(42));
        // You can easily store hh in any collection.
        // You can easily add a method to CommitProc.
        h.by_id_mut(&hh.0).unwrap().get_mut(hh.1).p();
    }
}
