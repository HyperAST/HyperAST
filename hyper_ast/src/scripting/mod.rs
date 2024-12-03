

mod preprocessing;
mod querying;

pub mod lua_scripting;
// mod rhai_impl {}
// mod native_impl {}

mod metrics; // TODO migrate to rhai_impl and preprocessing
mod exp_mlua; // TODO migrate to lua_impl and preprocessing




#[derive(Clone, PartialEq, Eq)]
pub struct Prepro {
    txt: std::sync::Arc<str>,
}

impl From<&str> for Prepro {
    fn from(txt: &str) -> Self {
        Self {
            txt: txt.into()
        }
    }
}