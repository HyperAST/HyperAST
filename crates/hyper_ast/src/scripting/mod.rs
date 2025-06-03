mod preprocessing;
mod querying;

#[cfg(feature = "scripting")]
pub mod lua_scripting;
#[cfg(feature = "scripting")]
mod rhai_scripting;
// mod native_impl {}

#[cfg(feature = "scripting")]
pub trait ScriptingHyperAST:
    for<'t> crate::types::StoreRefAssoc<S<'t> = <Self as ScriptingHyperAST>::_S<'t>>
{
    type _S<'t>: mlua::UserData;
}

// #[cfg(feature = "scripting")]
// impl<T> ScriptingHyperAST for T
// where
//     T: crate::types::StoreRefAssoc,
//     for<'a> Self::S<'a>: mlua::UserData,
// {
//     type _S<'t> = T::S<'t>;
// }

#[cfg(feature = "scripting")]
mod exp_mlua;
#[cfg(feature = "scripting")]
mod metrics; // TODO migrate to rhai_impl and preprocessing // TODO migrate to lua_impl and preprocessing

#[cfg(feature = "scripting")]
#[allow(unused)]
mod exp_lisp {
    pub static PREPRO_MCC: &'static str = r#"
(defun init ()
    "Initialize the cyclimatic complexity."
    (list mcc: 0)
)

(defun acc (x c)
    "Accumulate the value of the 'mcc' field on x from the child record c."
    (setf *mcc* (+ (getf *mcc* x) (getf *mcc* child)) x)
   ;(over *mcc* (+ (view-child *mcc* c)) x)
)

(define acc-lens
    (over *mcc*
        (+ (view-child *mcc*))
    )
    ; (accu *mcc* +)
)

(define accu (lens op)
    (over
        lens
        (op (view-child lens))
    )
)

(defun finish (a)
    "Finalize the computation based on the type of the node."
    (let ((mcc-value (if (ty-is-branch)
                        (+ a 1)
                        a)))
    (list :mcc mcc-value)))
(def_metric "mcc"
    (init (dict
        mcc: (if (is_branch) 1 0)
    ))
    (acc (c)
        (fold mcc c)
    )
)
(def_metric "mcc2"
    (init (dict
        mcc: 0
    ))
    (acc (c)
        (fold mcc c)
    )
    (finish
    )
)
    "#;
}

#[cfg(feature = "scripting")]
#[derive(Default)]
#[cfg_attr(feature = "bevy_ecs", derive(bevy_ecs::prelude::Component))]
pub struct DerivedData(
    // good way to improve compatibility and reusability
    pub rhai::Map,
);

#[derive(PartialEq, Eq)]
pub struct Prepro<HAST, Acc> {
    txt: std::sync::Arc<str>,
    _ph: std::marker::PhantomData<(HAST, Acc)>,
}

impl<HAST, Acc> Clone for Prepro<HAST, &Acc> {
    fn clone(&self) -> Self {
        Self {
            txt: self.txt.clone(),
            _ph: self._ph.clone(),
        }
    }
}

impl<HAST, Acc> From<&str> for Prepro<HAST, &Acc> {
    fn from(txt: &str) -> Self {
        Self {
            txt: txt.into(),
            _ph: Default::default(),
        }
    }
}

impl<HAST, Acc> From<std::sync::Arc<str>> for Prepro<HAST, Acc> {
    fn from(txt: std::sync::Arc<str>) -> Self {
        Self {
            txt: txt,
            _ph: Default::default(),
        }
    }
}

use crate::{store::defaults::NodeIdentifier, types::HyperType};

pub trait Scriptable {
    type Error: std::error::Error;

    type Scripts;
}

#[cfg(feature = "scripting")]
pub trait Finishable: Scriptable {
    fn finish<T: HyperType>(
        self,
        scripts: &Self::Scripts,
        subtree: &Subtr<T>,
    ) -> Result<DerivedData, Self::Error>;

    fn finish_with_label<T: HyperType>(
        self,
        scripts: &Self::Scripts,
        subtree: &Subtr<T>,
        label: &str,
    ) -> Result<DerivedData, Self::Error>;
}

pub struct Subtr<'a, T>(
    pub T,
    #[cfg(feature = "scripting")] pub &'a crate::store::nodes::legion::dyn_builder::EntityBuilder,
    #[cfg(not(feature = "scripting"))] pub &'a (),
);

#[cfg(feature = "scripting")]
pub trait Accumulable: Scriptable {
    fn acc<
        'a,
        T: HyperType + 'static,
        T2: HyperType + Send + Sync + 'static,
        HAST: mlua::UserData + 'static,
    >(
        &mut self,
        scripts: &Self::Scripts,
        store: &'a HAST,
        ty: T,
        child: SubtreeHandle<T2>,
    ) -> Result<(), Self::Error>;

    fn acc2<
        'a,
        T: HyperType + 'static,
        T2: HyperType + Send + Sync + 'static,
        HAST: ScriptingHyperAST + 'static,
    >(
        &mut self,
        scripts: &Self::Scripts,
        store: &'a HAST::S<'_>,
        ty: T,
        child: SubtreeHandle<T2>,
    ) -> Result<(), Self::Error>;
}

pub struct SubtreeHandle<T>(NodeIdentifier, std::marker::PhantomData<T>);

impl<T> Clone for SubtreeHandle<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

pub struct Acc {
    id: u16,
}

impl Scriptable for Acc {
    #[cfg(feature = "scripting")]
    type Error = mlua::Error;
    #[cfg(not(feature = "scripting"))]
    type Error = std::fmt::Error;
    type Scripts = ();
}

#[cfg(feature = "scripting")]
impl Finishable for Acc {
    fn finish<T: HyperType>(
        self,
        _scripts: &Self::Scripts,
        subtree: &Subtr<T>,
    ) -> Result<DerivedData, Self::Error> {
        Self::finish(self, subtree)
    }

    fn finish_with_label<T: HyperType>(
        self,
        _scripts: &Self::Scripts,
        subtree: &Subtr<T>,
        label: &str,
    ) -> Result<DerivedData, Self::Error> {
        Self::finish_with_label(self, subtree, label)
    }
}

#[cfg(feature = "scripting")]
impl Accumulable for Acc {
    fn acc<
        'a,
        T: HyperType + 'static,
        T2: HyperType + Send + Sync + 'static,
        HAST: mlua::UserData + 'static,
    >(
        &mut self,
        _scripts: &Self::Scripts,
        store: &'a HAST,
        ty: T,
        child: SubtreeHandle<T2>,
    ) -> Result<(), Self::Error> {
        Acc::acc(self, store, ty, child)
    }
    fn acc2<
        'a,
        T: HyperType + 'static,
        T2: HyperType + Send + Sync + 'static,
        HAST: ScriptingHyperAST,
    >(
        &mut self,
        _scripts: &Self::Scripts,
        _store: &'a HAST::S<'_>,
        _ty: T,
        _child: SubtreeHandle<T2>,
    ) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

impl<T> From<NodeIdentifier> for SubtreeHandle<T> {
    fn from(value: NodeIdentifier) -> Self {
        Self(value, Default::default())
    }
}
