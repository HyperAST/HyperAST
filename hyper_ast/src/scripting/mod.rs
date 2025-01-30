mod preprocessing;
mod querying;

#[cfg(feature = "scripting")]
pub mod lua_scripting;
// mod rhai_impl {}
// mod native_impl {}

#[cfg(feature = "scripting")]
mod exp_mlua;
#[cfg(feature = "scripting")]
mod metrics; // TODO migrate to rhai_impl and preprocessing // TODO migrate to lua_impl and preprocessing

#[derive(PartialEq, Eq)]
pub struct Prepro<HAST, Acc> {
    txt: std::sync::Arc<str>,
    _ph: std::marker::PhantomData<(HAST, Acc)>,
}

impl<HAST, Acc> Clone for Prepro<HAST, &Acc> {
    fn clone(&self) -> Self {
        Self { txt: self.txt.clone(), _ph: self._ph.clone() }
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

pub struct Acc {
    id: usize,
}
