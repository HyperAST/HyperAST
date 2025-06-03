#![allow(unused)]
use crate::store::SimpleStores;
use crate::store::nodes::legion::{HashedNodeRef, NodeIdentifier};
use crate::tree_gen::WithChildren;
use crate::types::{AnyType, HyperAST, HyperType, Shared, StoreRefAssoc};
use rhai::Dynamic;
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;
use std::time::Instant;

pub static PREPRO_SIZE: &'static str = r#"
let size = 1; # init

fn acc(c) {
    size += c.size;
}
"#;

pub static PREPRO_SIZE_WITH_FINISH: &'static str = r#"
let size = 1; # init

fn acc(c) {
    size += c.size;
}

fn finish() {
    {size: size}
}
"#;

pub static PREPRO_MCC: &'static str = r#"
let mcc = if is_branch(TY) {1} else {0};

fn acc(c) {
    mcc += c.mcc;
}
"#;

pub static PREPRO_MCC_WITH_FINISH: &'static str = r#"
let mcc = 0

fn acc(c) {
    mcc += c.mcc;
}

fn finish() {
  if is_branch(TY){
    mcc += 1;
  }
  { mcc: mcc }
}
"#;

pub static PREPRO_LOC: &'static str = r#"
let LoC = 0;
let b = true;

fn acc(c) {
  if c.is_comment() {
    b = false;
  } else if b {
    LoC += c.LoC;
    b = true;
  } else {
    b = true;
  }
}

fn finish() {
  if is_comment() {
    LoC = 0;
  } else if is_nl() {
    LoC = 1
  }
  { LoC: LoC }
}
"#;

#[derive(Clone, ref_cast::RefCast)]
#[repr(transparent)]
struct Ty<T = &'static dyn HyperType>(T);

pub struct Prepro<HAST, Acc> {
    txt: std::sync::Arc<str>,
    scri_engine: (Scri, rhai::Engine),
    _ph: std::marker::PhantomData<(HAST, Acc)>,
}

impl<HAST, Acc> Eq for Prepro<HAST, Acc> {}

impl<HAST, Acc> PartialEq for Prepro<HAST, Acc> {
    fn eq(&self, other: &Self) -> bool {
        self.txt == other.txt
    }
}

pub struct Acc {
    value: rhai::Dynamic,
}

impl super::Scriptable for Acc {
    type Error = rhai::EvalAltResult;
    type Scripts = (Scri, rhai::Engine);
}

impl super::Finishable for Acc {
    fn finish<T: crate::types::HyperType>(
        self,
        scripts: &Self::Scripts,
        subtree: &crate::scripting::Subtr<T>,
    ) -> Result<crate::scripting::DerivedData, Self::Error> {
        todo!()
    }

    fn finish_with_label<T: crate::types::HyperType>(
        self,
        scripts: &Self::Scripts,
        subtree: &crate::scripting::Subtr<T>,
        label: &str,
    ) -> Result<crate::scripting::DerivedData, Self::Error> {
        todo!()
    }
}

struct Subtree<HAST, IdN> {
    hast: HAST,
    id: IdN,
}

impl<HAST, IdN> Clone for Subtree<HAST, IdN> {
    fn clone(&self) -> Self {
        panic!()
    }
}

unsafe impl Send for StorePtr {}
unsafe impl Sync for StorePtr {}
// engine.register_type_with_name::<Mean>("Mean")

#[derive(Clone)]
struct StorePtr(*const (), std::any::TypeId);
impl StorePtr {
    fn new<HAST: crate::types::StoreRefAssoc + 'static>(store: &HAST::S<'_>) -> Self {
        let store = store as *const HAST::S<'_>;
        let store = unsafe { std::mem::transmute(store) };
        let hid = std::any::TypeId::of::<HAST>();
        Self(store, hid)
    }

    fn as_ref<HAST: crate::types::StoreRefAssoc + 'static>(&self) -> &HAST::S<'_> {
        assert_eq!(std::any::TypeId::of::<HAST>(), self.1);
        let store: &HAST::S<'_> = unsafe { std::mem::transmute(&self.0) };
        store
    }
}

impl super::Accumulable for Acc {
    fn acc<
        'a,
        T: crate::types::HyperType + 'static,
        T2: crate::types::HyperType + Send + Sync + 'static,
        HAST: mlua::UserData + 'static,
    >(
        &mut self,
        _scripts: &Self::Scripts,
        _store: &'a HAST,
        _ty: T,
        _child: crate::scripting::SubtreeHandle<T2>,
    ) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn acc2<
        'a,
        T: crate::types::HyperType + 'static,
        T2: crate::types::HyperType + Send + Sync + 'static,
        HAST: crate::scripting::ScriptingHyperAST + 'static,
    >(
        &mut self,
        scripts: &Self::Scripts,
        store: &'a HAST::S<'_>,
        ty: T,
        child: crate::scripting::SubtreeHandle<T2>,
    ) -> Result<(), Self::Error> {
        use rhai::*;
        // let mut child = Map::new();
        // let child: Dynamic = child.into();
        // let child = child.into_read_only();
        let store = StorePtr::new::<HAST>(store);
        let mut scope = Scope::new();
        scope.push("TY", ty.as_static_str());
        scope.push("HAST", store);
        scope.push("a", Dynamic::take(&mut self.value));
        scope.push("child", child);
        let engine = &scripts.1;
        let ast = &scripts.0.acc;
        let value = engine.eval_ast_with_scope(&mut scope, ast).unwrap();
        // let value = engine.eval_statements_raw(&mut scope, ast).unwrap();
        self.value = value;
        Ok(())
    }
}

// impl<HAST, Acc> Clone for Prepro<HAST, &Acc> {
//     fn clone(&self) -> Self {
//         Self {
//             txt: self.txt.clone(),
//             scri: self.scri.clone(),
//             engine: self.engine.clone(),
//             _ph: self._ph.clone(),
//         }
//     }
// }

impl<HAST, Acc> TryFrom<&str> for Prepro<HAST, &Acc> {
    type Error = String;
    fn try_from(txt: &str) -> Result<Self, Self::Error> {
        let (engine, scri) = Self::_make(txt)?;
        Ok(Self {
            txt: txt.into(),
            scri_engine: (scri, engine),
            _ph: Default::default(),
        })
    }
}

impl<HAST, Acc> TryFrom<std::sync::Arc<str>> for Prepro<HAST, Acc> {
    type Error = String;

    fn try_from(txt: std::sync::Arc<str>) -> Result<Self, Self::Error> {
        let (engine, scri) = Self::_make(&txt)?;
        Ok(Self {
            txt,
            scri_engine: (scri, engine),
            _ph: Default::default(),
        })
    }
}

impl<HAST: HyperAST, Acc> crate::tree_gen::Prepro<HAST> for Prepro<HAST, &Acc>
where
    HAST::TS: crate::types::ETypeStore,
    <HAST::TS as crate::types::ETypeStore>::Ty2: HyperType + 'static,
{
    const USING: bool = true;
    type Scope = self::Acc;
    fn preprocessing(
        &self,
        ty: <HAST::TS as crate::types::ETypeStore>::Ty2,
    ) -> std::result::Result<Self::Scope, <Self::Scope as crate::scripting::Scriptable>::Error>
    {
        use rhai::*;
        let init = &self.scri_engine.0.init;
        let mut scope = Scope::new();
        scope.push("TY", ty.as_static_str()); // TODO replace by selecting inlined-TY script
        let engine = &self.scri_engine.1;
        let value = engine
            .eval_ast_with_scope(&mut scope, init)
            .map_err(|x| x.to_string())?;
        Ok(self::Acc { value })
    }

    fn scripts(&self) -> &<Self::Scope as crate::scripting::Scriptable>::Scripts {
        &self.scri_engine
    }
}

impl<HAST, Acc> Prepro<HAST, Acc> {
    fn _make(txt: &str) -> Result<(rhai::Engine, Scri), String> {
        use rhai::*;
        let mut engine = Engine::new();
        let ast = engine.compile(txt).map_err(|x| x.to_string())?;
        // TODO analyze the ast to deduce the banching on TY predicates
        // then each TY categories should make a different script
        // then we can easily optimize for equality with TY.
        // For predicates matching multiple TY, it is easier to not refer to TY
        // this way we can optimize the predicate while transfering to a more efficient predicate
        let mut scope = Scope::new();
        let ast = engine.optimize_ast(&mut scope, ast, OptimizationLevel::Full);
        // TODO if the resulting acc/finish is a no op put a Noop variant
        // TODO if the resulting acc is a accumulate put a Accu variant
        // NOTE it should help the compiler optimize the preprocessing
        // Anyway...Benchmark it !
        let scri = extractor(ast)?;
        Ok((engine, scri))
    }
}

fn extractor(ast: rhai::AST) -> Result<Scri, String> {
    use rhai::*;
    let Some(init) = ast.shared_lib().get_script_fn("init", 0) else {
        return Err("fn init() is missing or has a wrong signature".to_string());
    };
    // TODO find accessed globals, specialize for a single type of node
    let Some(acc) = ast.shared_lib().get_script_fn("acc", 2) else {
        return Err("fn acc(a, child) is missing or has a wrong signature".to_string());
    };
    if acc.params[0] != "a" {
        return Err("fn acc(a, child) is missing or has a wrong signature".to_string());
    }
    if acc.params[1] != "child" {
        return Err("fn acc(a, child) is missing or has a wrong signature".to_string());
    }
    // TODO find accessed fields on child
    let Some(finish) = ast.shared_lib().get_script_fn("finish", 1) else {
        return Err("fn finish(a) is missing or has a wrong signature".to_string());
    };
    if finish.params[0] != "a" {
        return Err("fn finish(a) is missing or has a wrong signature".to_string());
    }
    let init = AST::new(init.body.iter().cloned(), Module::new());
    let acc = AST::new(acc.body.iter().cloned(), Module::new());
    let finish = AST::new(finish.body.iter().cloned(), Module::new());
    Ok(Scri { init, acc, finish })
}

pub struct Scri {
    init: rhai::AST,
    acc: rhai::AST,
    finish: rhai::AST,
}

impl<HAST, Acc> crate::tree_gen::More<HAST> for Prepro<HAST, &Acc>
where
    HAST: StoreRefAssoc,
    Acc: WithChildren<HAST::IdN>,
{
    type Acc = Acc;
    const ENABLED: bool = false;
    fn match_precomp_queries(
        &self,
        _stores: <HAST as StoreRefAssoc>::S<'_>,
        _acc: &Acc,
        _label: Option<&str>,
    ) -> crate::tree_gen::PrecompQueries {
        Default::default()
    }
}

impl<HAST, Acc> crate::tree_gen::PreproTSG<HAST> for Prepro<HAST, &Acc>
where
    HAST: StoreRefAssoc,
    Acc: WithChildren<HAST::IdN>,
{
    const GRAPHING: bool = false;
    fn compute_tsg(
        &self,
        _stores: <HAST as StoreRefAssoc>::S<'_>,
        _acc: &Acc,
        _label: Option<&str>,
    ) -> std::result::Result<usize, std::string::String> {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils;

    use super::*;
    use crate::test_utils::tree::Type;
}
