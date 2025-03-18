use crate::store::nodes::legion::{HashedNodeRef, NodeIdentifier};
use crate::store::SimpleStores;
use crate::tree_gen::WithChildren;
use crate::types::{AnyType, HyperAST, HyperType, Shared, StoreRefAssoc};
use mlua::prelude::*;
use mlua::{Lua, MetaMethod, Result, UserData, Value};
use rhai::Dynamic;
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;
use std::time::Instant;

pub static PREPRO_SIZE: &'static str = r#"
size = 1 -- init

function acc(c)
    size += c.size
end
"#;

pub static PREPRO_SIZE_WITH_FINISH: &'static str = r#"
local size = 1 -- init

function acc(c)
    size += c.size
end

function finish()
    return {size = size}
end
"#;

pub static PREPRO_MCC: &'static str = r#"
local mcc = if is_branch() then 1 else 0

function acc(c)
  mcc += c.mcc
end
"#;

pub static PREPRO_MCC_WITH_FINISH: &'static str = r#"
local mcc = 0

function acc(c)
  mcc += c.mcc
end

function finish()
  if is_branch() then
    mcc += 1
  end
  return { mcc = mcc }
end
"#;

pub static PREPRO_LOC: &'static str = r#"
local LoC = 0
local b = true

function acc(c)
  if c.is_comment then
    b = false
  elseif b then
    LoC += c.LoC
    b = true
  else 
    b = true
  end
end

function finish()
  if is_comment() then
    LoC = 0
  elseif is_nl() then
    LoC = 1
  end
  return {
    LoC = LoC
  }
end
"#;

impl Drop for Acc {
    fn drop(&mut self) {
        let count = LUA_INSTANCES.get() - 1;
        LUA_INSTANCES.set(count);
        assert_eq!(self.id, count); // TODO handle properly multiple stacks
        MAX_COUNT.set(MAX_COUNT.get().max(count));

        LUA_POOL.with_borrow_mut(|pool| {
            let lua = &pool[self.id as usize];
            log::info!("{} drop {count} {:p}", lua.used_memory(), &self);
            if count < 2 {
                // log::info!(
                //     "timings {} {} {} {} {}",
                //     MAX_COUNT.get(),
                //     // unsafe { TIME_GEN },
                //     // unsafe { TIME_INIT },
                //     // unsafe { TIME_ACC },
                //     // unsafe { TIME_FINISH }
                // );
            }
        });
    }
}

use super::{Acc, Prepro};

use std::cell::Cell;
use std::cell::RefCell;

thread_local! {
    static LUA_POOL: RefCell<Vec<Lua>>  = RefCell::new(vec![]);
    static LUA_INSTANCES: Cell<u16> = Cell::new(0);
    static MAX_COUNT: Cell<u16> = Cell::new(0);
    // pub static FOO: Cell<u32> = Cell::new(1);

    // static BAR: RefCell<Vec<f32>> = RefCell::new(vec![1.0, 2.0]);
}

impl<HAST, Acc> Prepro<HAST, &Acc> {
    fn gen_lua() -> Result<Lua> {
        let lua = Lua::new();
        // lua.set_memory_limit(260000)?; // fatal runtime error: Rust cannot catch foreign exceptions
        let pred_meth = lua.create_function(|lua, _: ()| {
            let ty: Value = lua.globals().get("TY")?;
            let ty = ty.as_userdata().unwrap();
            let ty = ty.borrow::<Ty<&'static dyn HyperType>>().unwrap();
            use crate::types::HyperType;
            Ok(ty.deref().0.as_shared() == crate::types::Shared::Branch)
        })?;
        lua.globals().set("is_branch", pred_meth)?;
        let pred_meth = lua.create_function(|lua, _: ()| {
            let ty: Value = lua.globals().get("TY")?;
            let ty = ty.as_userdata().unwrap();
            let ty = ty.borrow::<Ty<&'static dyn HyperType>>().unwrap();
            use crate::types::HyperType;
            Ok(ty.deref().0.as_shared() == crate::types::Shared::Comment)
        })?;
        lua.globals().set("is_comment", pred_meth)?;
        let pred_meth = lua.create_function(|lua, _: ()| {
            let ty: Value = lua.globals().get("TY")?;
            let ty = ty.as_userdata().unwrap();
            let ty = ty.borrow::<Ty<&'static dyn HyperType>>().unwrap();
            use crate::types::HyperType;
            Ok(ty.deref().0.is_spaces())
        })?;
        lua.globals().set("is_spaces", pred_meth)?;
        let pred_meth = lua.create_function(|lua, _: ()| {
            let ty: Value = lua.globals().get("TY")?;
            let ty = ty.as_userdata().unwrap();
            let ty = ty.borrow::<Ty<&'static dyn HyperType>>().unwrap();
            use crate::types::HyperType;
            if ty.deref().0.is_spaces() {
                let l: Value = lua.globals().get("L")?;
                let s = l.as_str().unwrap();
                Ok(s.contains("\n"))
            } else {
                Ok(false)
            }
        })?;
        lua.globals().set("is_nl", pred_meth)?;
        let mt = if let Some(mt) = lua.globals().get_metatable() {
            mt
        } else {
            lua.create_table()?
        };
        lua.globals().set_metatable(Some(mt));
        lua.sandbox(true)?;
        Ok(lua)
    }

    pub fn new(chunk: impl AsRef<str>) -> Self {
        Self {
            txt: chunk.as_ref().into(),
            _ph: Default::default(),
        }
    }
    pub fn from_arc(chunk: std::sync::Arc<str>) -> Self {
        Self {
            txt: chunk,
            _ph: Default::default(),
        }
    }

    fn init<T: HyperType + 'static>(self, ty: T) -> Result<self::Acc> {
        let now = Instant::now();
        let mut count = LUA_INSTANCES.get();
        LUA_POOL.with_borrow_mut(|pool| {
            let id;
            let lua = if (count as usize) < pool.len() {
                id = count;
                &mut pool[id as usize]
            } else if count as usize == pool.len() {
                id = count;
                pool.push(Self::gen_lua().expect("a lua interpretor"));
                &mut pool[id as usize]
            } else {
                panic!()
            };
            count += 1;
            LUA_INSTANCES.set(count);

            let prepare_time = now.elapsed().as_secs_f64();
            let now = Instant::now();
            // unsafe { TIME_GEN += prepare_time };
            log::debug!("gen {} {prepare_time}", &lua.used_memory());

            lua.scope(|scope| {
                let ty = scope.create_any_userdata(Ty(ty))?;
                lua.globals().set("TY", ty)?;
                // log::warn!("{} init {count}  {:p}", &lua.used_memory(), &self);
                lua.load(self.txt.as_ref()).exec()?;
                // log::warn!("{} inited", &lua.used_memory());
                Ok(())
            })?;
            let prepare_time = now.elapsed().as_secs_f64();
            // unsafe { TIME_INIT += prepare_time };
            log::debug!("{} {prepare_time}", &lua.used_memory());
            Ok(self::Acc { id })
        })
    }
}
#[derive(Clone, ref_cast::RefCast)]
#[repr(transparent)]
struct Ty<T = &'static dyn HyperType>(T);
pub trait Subtree: UserData {
    fn ty(&self) -> &'static dyn HyperType;
}

impl UserData for Ty {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::Eq, |_, a, b: Value| {
            if let Some(s) = b.as_str() {
                Ok(a.0.to_string() == s.to_string())
            } else {
                Err(mlua::Error::BadArgument {
                    to: None,
                    pos: 42,
                    name: None,
                    cause: mlua::Error::runtime("").into(),
                })
            }
        });
    }
}

pub struct Subtr<'a, T>(
    pub T,
    pub &'a crate::store::nodes::legion::dyn_builder::EntityBuilder,
);
impl<'a, T: HyperType> crate::scripting::lua_scripting::Subtree for Subtr<'a, T> {
    fn ty(&self) -> &'static dyn HyperType {
        self.0.as_static()
    }
}
impl<'a, T: HyperType> UserData for Subtr<'a, T> {}

pub struct SubtrLegion<'a, T>(
    pub crate::store::nodes::legion::HashedNodeRef<'a>,
    pub PhantomData<&'a T>,
);
impl<'a, T: HyperType + Send + Sync + 'static> crate::scripting::lua_scripting::Subtree
    for SubtrLegion<'a, T>
{
    fn ty(&self) -> &'static dyn HyperType {
        let t = self.0.get_component::<T>().unwrap();
        t.as_static()
    }
}
impl<'a, T: HyperType> UserData for SubtrLegion<'a, T> {}

impl mlua::UserData for AnyType {}
impl mlua::UserData for HashedNodeRef<'_> {}

pub struct SubtreeHandle<T>(NodeIdentifier, PhantomData<T>);
impl<T> From<NodeIdentifier> for SubtreeHandle<T> {
    fn from(value: NodeIdentifier) -> Self {
        Self(value, PhantomData)
    }
}
impl<T: HyperType + Send + Sync + 'static> mlua::UserData for SubtreeHandle<T> {
    // fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
    //     fields.add_meta_field(name, value);
    // }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::Index, |lua, this, v: Value| {
            let s = v.as_str().unwrap();
            let id = &this.0;
            let store: Value = lua.globals().get("STORE")?;
            let store = store
                .as_userdata()
                .unwrap()
                .borrow::<SimpleStores<()>>()
                .unwrap();
            let n = crate::types::NodeStore::resolve(&store.node_store, id);
            if s == "is_comment" {
                // let ty = n.try_get_type().unwrap();
                // TODO make a subtree handle on the consumer side to enable polyglote
                let ty = n.get_component::<T>().unwrap();
                let b = ty.as_shared() == Shared::Comment;
                return b.into_lua(lua);
            }
            let dd = n.get_component::<DerivedData>().unwrap();
            let Some(d) = dd.0.get(s) else {
                return Err(mlua::Error::runtime(s));
            };
            d_to_lua(lua, d)
        });
    }
}

impl Acc {
    pub fn acc<
        'a,
        T: HyperType + 'static,
        T2: HyperType + Send + Sync + 'static,
        HAST: UserData + 'static,
    >(
        &mut self,
        store: &'a HAST,
        ty: T,
        child: SubtreeHandle<T2>,
    ) -> Result<()> {
        // let now = Instant::now();
        LUA_POOL.with_borrow_mut(|pool| {
            let lua = &mut pool[self.id as usize];
            // let lua = &mut self.lua;
            let acc = lua.globals().get::<_, mlua::Function>("acc")?;
            lua.scope(|scope| {
                let ty = scope.create_any_userdata(Ty(ty.as_static()))?;
                lua.globals().set("TY", ty)?;
                let child = scope.create_userdata(child)?;
                let store = scope.create_userdata_ref(store)?;
                lua.globals().set("STORE", store)?;
                log::debug!("{} acc", &lua.used_memory());
                let m: mlua::Value = acc.call((child,))?;
                debug_assert!(m.is_nil());
                Ok(())
            })?;
            // let prepare_time = now.elapsed().as_secs_f64();
            // unsafe { TIME_ACC += prepare_time };
            Ok(())
        })
    }
    pub fn finish<T: HyperType>(self, subtree: &Subtr<T>) -> Result<DerivedData> {
        let now = Instant::now();
        let ptr = format!("{:p}", &self);
        LUA_POOL.with_borrow_mut(|pool| {
            let lua = &mut pool[self.id as usize];
            // let lua = &mut self.lua;
            let finish = lua.globals().get::<_, mlua::Function>("finish")?;
            let m = lua.scope(|scope| {
                let ty = subtree.ty();
                let ty = scope.create_any_userdata(Ty(ty.as_static()))?;
                lua.globals().set("TY", ty)?;
                log::debug!("{}", &lua.used_memory());
                let m: mlua::Value = finish.call(())?;
                log::debug!("{}", &lua.used_memory());
                // dbg!(&m);
                Ok(m)
            })?;
            log::debug!("{}", &lua.used_memory());
            // dbg!(&m);
            lua.gc_collect()?;
            // dbg!(&m);
            log::debug!("{} gced", &lua.used_memory());

            let map = DerivedData::try_from(m.as_table().unwrap())?;
            log::debug!("{}", &lua.used_memory());
            lua.sandbox(false)?;
            log::debug!("{} unbox {ptr}", &lua.used_memory());

            let prepare_time = now.elapsed().as_secs_f64();
            // unsafe { TIME_FINISH += prepare_time };
            log::debug!("{} {prepare_time}", &lua.used_memory());
            Ok(map)
        })
    }
    pub fn finish_with_label<T: HyperType>(
        self,
        subtree: &Subtr<T>,
        label: String,
    ) -> Result<DerivedData> {
        LUA_POOL.with_borrow_mut(|pool| {
            let lua = &mut pool[self.id as usize];
            let now = Instant::now();
            let ptr = format!("{:p}", &self);
            // let lua = &mut self.lua;
            let finish = lua.globals().get::<_, mlua::Function>("finish")?;
            let m = lua.scope(|scope| {
                let ty = subtree.ty();
                let ty = scope.create_any_userdata(Ty(ty.as_static()))?;
                lua.globals().set("TY", ty)?;
                // let subt = scope.create_any_userdata(label)?;
                lua.globals().set("L", label)?;
                log::debug!("{}", &lua.used_memory());
                let m: mlua::Value = finish.call(())?;
                log::debug!("{}", &lua.used_memory());
                // dbg!(&m);
                Ok(m)
            })?;
            log::debug!("{}", &lua.used_memory());
            // dbg!(&m);
            lua.gc_collect()?;
            // dbg!(&m);
            log::debug!("{} gced", &lua.used_memory());

            let map = DerivedData::try_from(m.as_table().unwrap())?;
            log::debug!("{}", &lua.used_memory());
            lua.sandbox(false)?;
            log::debug!("{} unbox {ptr}", &lua.used_memory());

            let prepare_time = now.elapsed().as_secs_f64();
            // unsafe { TIME_FINISH += prepare_time };
            log::debug!("{} {prepare_time}", &lua.used_memory());
            Ok(map)
        })
    }
}

// WARN if used in parallele the result will tend to bias toward a lower runtime
// static mut TIME_GEN: f64 = 0.0;
// static mut TIME_INIT: f64 = 0.0;
// static mut TIME_ACC: f64 = 0.0;
// static mut TIME_FINISH: f64 = 0.0;

#[derive(Default)]
pub struct DerivedData(
    // good way to improve compatibility and reusability
    pub rhai::Map,
);

impl TryFrom<&LuaTable<'_>> for DerivedData {
    type Error = mlua::Error;

    fn try_from(value: &LuaTable<'_>) -> std::result::Result<Self, Self::Error> {
        let mut map = rhai::Map::new();
        value.for_each(|k: Value, v: Value| {
            let key = k
                .as_str()
                .ok_or_else(|| mlua::Error::FromLuaConversionError {
                    from: k.type_name(),
                    to: "str",
                    message: None,
                })?;
            let value = match &v {
                LuaValue::Nil => rhai::Dynamic::UNIT,
                LuaValue::Boolean(b) => rhai::Dynamic::from_bool(*b),
                LuaValue::Integer(i) => rhai::Dynamic::from_int(*i as i64),
                LuaValue::Number(f) => rhai::Dynamic::from_float(*f),
                LuaValue::String(s) => rhai::Dynamic::from_str(s.to_str()?).map_err(|_| {
                    mlua::Error::FromLuaConversionError {
                        from: "LuaString",
                        to: "rhai::Dynamic::Str",
                        message: Some(format!("WIP in {}:{}", file!(), line!())),
                    }
                })?,
                // LuaValue::LightUserData(light_user_data) => todo!(),
                // LuaValue::Vector(vector) => todo!(),
                // LuaValue::Table(table) => todo!(),
                // LuaValue::Function(function) => todo!(),
                // LuaValue::Thread(thread) => todo!(),
                // LuaValue::UserData(any_user_data) => todo!(),
                // LuaValue::Error(error) => todo!(),
                _ => {
                    return Err(mlua::Error::FromLuaConversionError {
                        from: v.type_name(),
                        to: "rhai::Dynamic",
                        message: Some(format!("WIP in {}:{}", file!(), line!())),
                    })
                }
            };
            map.insert(key.into(), value);
            Ok(())
        })?;
        Ok(Self(map))
    }
}

// struct S {
//     ty: AnyType,
// }
// impl UserData for S {
//     fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
//         methods.add_meta_method(MetaMethod::Eq, |_, a, b: Value| {
//             if let Some(s) = b.as_str() {
//                 Ok(a.ty.to_string() == b.to_string()?)
//             } else {
//                 Err(mlua::Error::BadArgument {
//                     to: None,
//                     pos: 42,
//                     name: None,
//                     cause: mlua::Error::runtime("aaa").into(),
//                 })
//             }
//         });
//     }
// }
// impl Subtree for S {
//     fn ty(&self) -> &'static dyn HyperType {
//         todo!()
//         // Ty::ref_cast(&self.ty)
//     }
// }

// impl S {
//     fn finish(self, acc: Acc) -> Result<SS> {
//         todo!()
//         // let ty = self.ty;
//         // let dd = acc.finish(&self)?;
//         // Ok(SS { ty, dd })
//     }
// }
// struct SS {
//     ty: AnyType,
//     dd: DerivedData,
// }
// impl Subtree for SS {
//     fn ty(&self) -> AnyType {
//         todo!()
//         // Ty::ref_cast(&self.ty)
//     }
// }
// impl UserData for SS {
//     fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
//         // for (k,v) in
//         fields.add_field_method_get("ty", |lua, this| this.ty().clone().into_lua(lua));
//         // fields.add_field_method_get("ty", |lua, this| {
//         //     this.dd
//         //         .0
//         //         .get("ty")
//         //         .as_ref()
//         //         .unwrap()
//         //         .as_immutable_string_ref()
//         //         .unwrap()
//         //         .as_str()
//         //         .into_lua(lua)
//         // });
//     }
//     fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
//         methods.add_method("is_comment", |lua, this, v: ()| {
//             Ok(this.ty().0.as_shared() == crate::types::Shared::Comment)
//         });
//         methods.add_method("is_branch", |lua, this, v: ()| {
//             Ok(this.ty().0.as_shared() == crate::types::Shared::Branch)
//         });
//         methods.add_meta_method(MetaMethod::Eq, |_, a, b: Value| {
//             if let Some(s) = b.as_str() {
//                 Ok(a.ty.to_string() == s.to_string())
//             } else {
//                 Err(mlua::Error::BadArgument {
//                     to: None,
//                     pos: 42,
//                     name: None,
//                     cause: mlua::Error::runtime("aaa").into(),
//                 })
//             }
//         });
//         methods.add_meta_method(MetaMethod::Index, |lua, this, v: Value| {
//             dbg!(&v);
//             d_to_lua(lua, this.dd.0.get(v.as_str().unwrap()).as_ref().unwrap())
//         });
//     }
// }

fn d_to_lua<'a>(lua: &'a Lua, d: &Dynamic) -> Result<Value<'a>> {
    if let Ok(v) = d.as_int() {
        (v as i32).into_lua(lua)
    } else if let Ok(v) = d.as_float() {
        v.into_lua(lua)
    } else if let Ok(v) = d.as_bool() {
        v.into_lua(lua)
    } else if let Ok(v) = d.as_immutable_string_ref() {
        v.as_str().into_lua(lua)
    } else {
        dbg!(d);
        todo!()
    }
}

impl<HAST: HyperAST, Acc> crate::tree_gen::Prepro<HAST> for Prepro<HAST, &Acc>
where
    HAST::TS: crate::types::ETypeStore,
    <HAST::TS as crate::types::ETypeStore>::Ty2: HyperType + 'static,
{
    const USING: bool = true;
    fn preprocessing(
        &self,
        ty: <HAST::TS as crate::types::ETypeStore>::Ty2,
    ) -> std::result::Result<self::Acc, String> {
        self.clone().init(ty).map_err(|x| x.to_string())
    }
}

impl<HAST: HyperAST, Acc> crate::tree_gen::Prepro<HAST> for &Prepro<HAST, &Acc>
where
    HAST::TS: crate::types::ETypeStore,
    <HAST::TS as crate::types::ETypeStore>::Ty2: HyperType + 'static,
{
    const USING: bool = true;
    fn preprocessing(
        &self,
        ty: <HAST::TS as crate::types::ETypeStore>::Ty2,
    ) -> std::result::Result<self::Acc, String> {
        (*self).preprocessing(ty)
    }
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
    ) -> std::result::Result<usize, std::string::String>
where
        // <HAST as crate::types::HyperASTShared>::IdN: Copy,
        // HAST: 'static,
    {
        Ok(0)
    }
}

// impl<S: crate::tree_gen::DerefStore, Acc> crate::tree_gen::More for &Prepro<&S, &Acc> {
//     type T = ();
//     type Acc = Acc;
//     const ENABLED: bool = false;
//     fn match_precomp_queries(
//         &self,
//         _stores: <Self::S as crate::tree_gen::DerefStore>::Raw,
//         _acc: &Acc,
//         _label: Option<&str>,
//     ) -> crate::tree_gen::PrecompQueries {
//         Default::default()
//     }
// }

#[cfg(test)]
mod tests {
    use crate::test_utils;

    use super::*;
    use crate::test_utils::tree::Type;
    // use crate_gen_ts_java::types::Type;

    // impl From<Type> for S {
    //     fn from(ty: Type) -> Self {
    //         let ty = crate_gen_ts_java::types::as_any(&ty);
    //         Self { ty }
    //     }
    // }

    // fn prepro_with_finish(chunk: &str) -> std::result::Result<(), LuaError> {
    //     let s_class = S::from(Type::ClassDeclaration);
    //     let s_meth = S::from(Type::MethodDeclaration);
    //     let s_if = S::from(Type::IfStatement);
    //     let prepro = Prepro::new(chunk);
    //     let mut acc_class = prepro.clone().init(&s_class)?;
    //     let mut acc_meth = prepro.clone().init(&s_meth)?;
    //     let mut acc_if = prepro.clone().init(&s_if)?;
    //     let s_if = s_if.finish(acc_if)?;
    //     dbg!(&s_if.dd.0);
    //     acc_meth.acc(&s_if)?;
    //     let s_meth = s_meth.finish(acc_meth)?;
    //     acc_class.acc(&s_meth)?;
    //     let s_class = s_class.finish(acc_class)?;
    //     dbg!(s_class.dd.0);
    //     Ok(())
    // }

    // #[test]
    // fn test_with_finish_size() -> Result<()> {
    //     let chunk = PREPRO_SIZE_WITH_FINISH;
    //     prepro_with_finish(chunk)
    // }

    // #[test]
    // fn test_with_finish_mcc() -> Result<()> {
    //     let chunk = PREPRO_MCC_WITH_FINISH;
    //     prepro_with_finish(chunk)
    // }

    // #[test]
    // fn test_with_finish_loc() -> Result<()> {
    //     let chunk = PREPRO_LOC;
    //     prepro_with_finish(chunk)
    // }
}
