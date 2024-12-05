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

#[derive(Clone, PartialEq, Eq)]
pub struct Prepro {
    txt: std::sync::Arc<str>,
}

impl From<&str> for Prepro {
    fn from(txt: &str) -> Self {
        Self { txt: txt.into() }
    }
}

pub struct Acc {
    id: usize,
}
