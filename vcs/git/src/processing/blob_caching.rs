use super::CacheHolding;
use git2::{Oid, Repository};

impl crate::processing::erased::ProcessorMap {
    pub(crate) fn caching_blob_handler<C>(&mut self) -> CachingBlobWrapper2<'_, C> {
        CachingBlobWrapper2 {
            processors: self,
            phantom: std::marker::PhantomData,
        }
    }
}

pub(crate) struct CachingBlobWrapper2<'cache, C> {
    pub(crate) processors: &'cache mut crate::processing::erased::ProcessorMap,
    pub(crate) phantom: std::marker::PhantomData<C>,
}

impl<'cache, Sys> CachingBlobWrapper2<'cache, Sys> {
    pub fn handle<
        T: crate::processing::erased::CommitProcExt,
        N,
        E: From<std::str::Utf8Error>,
        F: FnOnce(
            &mut crate::processing::erased::ProcessorMap,
            &N,
            &[u8],
        ) -> Result<<Sys::Caches as crate::processing::ObjectMapper>::V, E>,
    >(
        &mut self,
        oid: Oid,
        repository: &Repository,
        name: &N,
        parameters: crate::processing::erased::ParametrizedCommitProcessor2Handle<T>,
        wrapped: F,
    ) -> Result<<Sys::Caches as crate::processing::ObjectMapper>::V, E>
    where
        for<'a> &'a N: TryInto<&'a str>,
        Sys: crate::processing::CachesHolding,
        Sys::Caches: 'static + crate::processing::ObjectMapper<K = Oid> + Send + Sync + Default,
        <Sys::Caches as crate::processing::ObjectMapper>::V: Clone,
        T::Holder: 'static + crate::processing::erased::ErasableProcessor + Send + Sync,
        T::Holder: crate::processing::erased::ParametrizedCommitProc2,
        <T::Holder as crate::processing::erased::ParametrizedCommitProc2>::Proc:
            crate::processing::CacheHolding<Sys::Caches>,
        T::Holder: std::default::Default
    {
        use crate::processing::erased::ParametrizedCommitProc2;
        let caches = self.processors.mut_or_default::<T::Holder>();
        let caches = caches.with_parameters_mut(parameters.0);
        let caches = caches.get_caches_mut();
        use crate::processing::ObjectMapper;
        if let Some(already) = caches.get(&oid) {
            // TODO reinit already computed node for post order
            let full_node = already.clone();
            return Ok(full_node);
        }
        log::info!(target: "blob",
            "blob {:?} {:?}",
            name.try_into().unwrap_or("'non utf8 name'"),
            oid
        );
        let blob = repository.find_blob(oid).unwrap();
        std::str::from_utf8(blob.content())?;
        let text = blob.content();
        let full_node = wrapped(self.processors, &name, text);
        if let Ok(x) = &full_node {
            self.processors
                // .mut_or_default::<Sys::Holder>().get_caches_mut()
                .mut_or_default::<T::Holder>()
                .with_parameters_mut(parameters.0)
                .get_caches_mut()
                .insert(oid, x.clone());
        }
        full_node
    }
    
    pub fn handle2<
        T: crate::processing::erased::CommitProcExt,
        N: Clone,
        E: From<std::str::Utf8Error>,
        F: FnOnce(
            &mut crate::processing::erased::ProcessorMap,
            &N,
            &[u8],
        ) -> Result<<Sys::Caches as crate::processing::ObjectMapper>::V, E>,
    >(
        &mut self,
        oid: Oid,
        repository: &Repository,
        name: &N,
        parameters: crate::processing::erased::ParametrizedCommitProcessor2Handle<T>,
        wrapped: F,
    ) -> Result<<Sys::Caches as crate::processing::ObjectMapper>::V, E>
    where
        for<'a> &'a N: TryInto<&'a str>,
        Sys: crate::processing::CachesHolding,
        Sys::Caches:
            'static + crate::processing::ObjectMapper<K = (Oid, N)> + Send + Sync + Default,
        <Sys::Caches as crate::processing::ObjectMapper>::V: Clone,
        T::Holder: 'static + crate::processing::erased::ErasableProcessor + Default + Send + Sync,
        T::Holder: crate::processing::erased::ParametrizedCommitProc2,
        <T::Holder as crate::processing::erased::ParametrizedCommitProc2>::Proc:
            crate::processing::CacheHolding<Sys::Caches>,
    {
        use crate::processing::erased::ParametrizedCommitProc2;
        let caches = self.processors.mut_or_default::<T::Holder>();
        let caches = caches.with_parameters_mut(parameters.0);
        let caches = caches.get_caches_mut();
        use crate::processing::ObjectMapper;
        if let Some(already) = caches.get(&(oid, name.clone())) {
            // TODO reinit already computed node for post order
            let full_node = already.clone();
            return Ok(full_node);
        }
        log::info!(target: "blob",
            "blob {:?} {:?}",
            name.try_into().unwrap_or("'non utf8 name'"),
            oid
        );
        let blob = repository.find_blob(oid).unwrap();
        if let Err(err) = std::str::from_utf8(blob.content()) {
            log::warn!("non utf8 char in blob content {}", err);
        }

        // TODO make a proper test later when attempting to be more resilient to ill-formating, ...
        // for ref on dubbo because of chinese chars vscode detects GB 2312
        // [2024-08-27T11:22:37Z INFO  hyperast_vcs_git::preprocessed] handle commit: f8cb608c1eb242544be640f3ae994391729b2175
        // [2024-08-27T11:22:37Z INFO  hyperast_vcs_git::java_processor] tree Ok("com")
        // [2024-08-27T11:22:37Z INFO  hyperast_vcs_git::java_processor] tree Ok("alibaba")
        // [2024-08-27T11:22:37Z INFO  hyperast_vcs_git::java_processor] tree Ok("dubbo")
        // [2024-08-27T11:22:37Z INFO  hyperast_vcs_git::java_processor] tree Ok("config")
        // [2024-08-27T11:22:37Z INFO  hyperast_vcs_git::java_processor] tree Ok("utils")
        // [2024-08-27T11:22:37Z INFO  hyperast_vcs_git::preprocessed] blob "ReferenceConfigCacheTest.java" 3452c2ebc75e77ccf23a5c6cd7c1823f77096aa3
        // thread 'tokio-runtime-worker' panicked at cvs/git/src/java_processor.rs:112:26:

        let text = blob.content();
        let full_node = wrapped(self.processors, &name, text);
        if let Ok(x) = &full_node {
            self.processors
                .mut_or_default::<T::Holder>()
                .with_parameters_mut(parameters.0)
                .get_caches_mut()
                .insert((oid, name.clone()), x.clone());
        }
        full_node
    }
}
