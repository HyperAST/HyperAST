use crate::store::nodes::legion::{HashedNodeRef, NodeIdentifier};
use crate::store::SimpleStores;
use crate::types::{AnyType, HyperType, NodeStore, Shared, Typed};
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

pub struct Acc {
    id: usize,
    // lua: Lua,
}

static mut MAX_COUNT: usize = 0;

impl Drop for Acc {
    fn drop(&mut self) {
        unsafe { LUA_INSTANCES -= 1 };
        let count = unsafe { LUA_INSTANCES };
        assert_eq!(self.id, count); // TODO handle properly multiple stacks
        unsafe {
            MAX_COUNT = MAX_COUNT.max(count);
        }
        let lua = unsafe { &LUA_POOL[self.id] };
        log::warn!("{} drop {count} {:p}", lua.used_memory(), &self);
        if count < 2 {
            log::error!(
                "timings {} {} {} {} {}",
                unsafe { MAX_COUNT },
                unsafe { TIME_GEN },
                unsafe { TIME_INIT },
                unsafe { TIME_ACC },
                unsafe { TIME_FINISH }
            );
        }
    }
}

use super::Prepro;

static mut LUA_POOL: Vec<Lua> = vec![];

impl Prepro {
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

    fn new(chunk: impl AsRef<str>) -> Self {
        Self {
            txt: chunk.as_ref().into(),
        }
    }

    fn init<T: HyperType + 'static>(self, ty: T) -> Result<Acc> {
        let now = Instant::now();
        let id;
        let lua: &mut Lua = unsafe {
            if LUA_INSTANCES < LUA_POOL.len() {
                id = LUA_INSTANCES;
                &mut LUA_POOL[id]
            } else if LUA_INSTANCES == LUA_POOL.len() {
                id = LUA_INSTANCES;
                LUA_POOL.push(Self::gen_lua()?);
                &mut LUA_POOL[id]
            } else {
                panic!()
            }
        };
        unsafe { LUA_INSTANCES += 1 };
        let prepare_time = now.elapsed().as_secs_f64();
        let now = Instant::now();
        unsafe { TIME_GEN += prepare_time };
        log::warn!("gen {} {prepare_time}", &lua.used_memory());

        lua.scope(|scope| {
            let ty = scope.create_any_userdata(Ty(ty))?;
            lua.globals().set("TY", ty)?;
            // log::warn!("{} init {count}  {:p}", &lua.used_memory(), &self);
            lua.load(self.txt.as_ref()).exec()?;
            // log::warn!("{} inited", &lua.used_memory());
            Ok(())
        })?;
        let prepare_time = now.elapsed().as_secs_f64();
        unsafe { TIME_INIT += prepare_time };
        log::warn!("{} {prepare_time}", &lua.used_memory());
        Ok(Acc { id })
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
            let n = store.resolve(id);
            if s == "is_comment" {
                // let ty = n.try_get_type().unwrap();
                // TODO make a subtree handle on the consumer side to enable polyglote
                let ty = n.get_component::<T>().unwrap();
                let b = ty.as_shared() == Shared::Comment;
                return b.into_lua(lua);
            }
            let dd = n.get_component::<DerivedData>().unwrap();
            // log::error!("meta {:?}", dd.0.get("size"));
            let Some(d) = dd.0.get(s) else {
                return Err(mlua::Error::runtime(s));
            };
            d_to_lua(lua, d)
        });
    }
}

impl Acc {
    pub fn acc<'a, T: HyperType + 'static, T2: HyperType + Send + Sync + 'static, HAST: UserData + 'static>(
        &mut self,
        store: &'a HAST,
        ty: T,
        child: SubtreeHandle<T2>,
    ) -> Result<()> {
        let now = Instant::now();
        let lua = unsafe { &mut LUA_POOL[self.id] };
        // let lua = &mut self.lua;
        let acc = lua.globals().get::<_, mlua::Function>("acc")?;
        lua.scope(|scope| {
            let ty = scope.create_any_userdata(Ty(ty.as_static()))?;
            lua.globals().set("TY", ty)?;
            let child = scope.create_userdata(child)?;
            let store = scope.create_userdata_ref(store)?;
            lua.globals().set("STORE", store)?;
            log::warn!("{} acc", &lua.used_memory());
            let m: mlua::Value = acc.call((child,))?;
            debug_assert!(m.is_nil());
            Ok(())
        })?;
        let prepare_time = now.elapsed().as_secs_f64();
        unsafe { TIME_ACC += prepare_time };
        Ok(())
    }
    pub fn finish<T: HyperType>(mut self, subtree: &Subtr<T>) -> Result<DerivedData> {
        let now = Instant::now();
        let ptr = format!("{:p}", &self);
        let lua = unsafe { &mut LUA_POOL[self.id] };
        // let lua = &mut self.lua;
        let finish = lua.globals().get::<_, mlua::Function>("finish")?;
        let m = lua.scope(|scope| {
            let ty = subtree.ty();
            let ty = scope.create_any_userdata(Ty(ty.as_static()))?;
            lua.globals().set("TY", ty)?;
            log::warn!("{}", &lua.used_memory());
            let m: mlua::Value = finish.call(())?;
            log::warn!("{}", &lua.used_memory());
            // dbg!(&m);
            Ok(m)
        })?;
        log::warn!("{}", &lua.used_memory());
        // dbg!(&m);
        lua.gc_collect()?;
        // dbg!(&m);
        log::warn!("{} gced", &lua.used_memory());

        let map = DerivedData::try_from(m.as_table().unwrap())?;
        log::warn!("{}", &lua.used_memory());
        lua.sandbox(false)?;
        log::warn!("{} unbox {ptr}", &lua.used_memory());

        let prepare_time = now.elapsed().as_secs_f64();
        unsafe { TIME_FINISH += prepare_time };
        log::warn!("{} {prepare_time}", &lua.used_memory());
        // log::error!("{:?}", map.0.get("size"));
        Ok(map)
    }
    pub fn finish_with_label<T: HyperType>(mut self, subtree: &Subtr<T>, label: String) -> Result<DerivedData> {
        let now = Instant::now();
        let ptr = format!("{:p}", &self);
        let lua = unsafe { &mut LUA_POOL[self.id] };
        // let lua = &mut self.lua;
        let finish = lua.globals().get::<_, mlua::Function>("finish")?;
        let m = lua.scope(|scope| {
            let ty = subtree.ty();
            let ty = scope.create_any_userdata(Ty(ty.as_static()))?;
            lua.globals().set("TY", ty)?;
            // let subt = scope.create_any_userdata(label)?;
            lua.globals().set("L", label)?;
            log::warn!("{}", &lua.used_memory());
            let m: mlua::Value = finish.call(())?;
            log::warn!("{}", &lua.used_memory());
            // dbg!(&m);
            Ok(m)
        })?;
        log::warn!("{}", &lua.used_memory());
        // dbg!(&m);
        lua.gc_collect()?;
        // dbg!(&m);
        log::warn!("{} gced", &lua.used_memory());

        let map = DerivedData::try_from(m.as_table().unwrap())?;
        log::warn!("{}", &lua.used_memory());
        lua.sandbox(false)?;
        log::warn!("{} unbox {ptr}", &lua.used_memory());

        let prepare_time = now.elapsed().as_secs_f64();
        unsafe { TIME_FINISH += prepare_time };
        log::warn!("{} {prepare_time}", &lua.used_memory());
        // log::error!("{:?}", map.0.get("size"));
        Ok(map)
    }
}

static mut LUA_INSTANCES: usize = 0;
static mut TIME_GEN: f64 = 0.0;
static mut TIME_INIT: f64 = 0.0;
static mut TIME_ACC: f64 = 0.0;
static mut TIME_FINISH: f64 = 0.0;

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

impl<T: HyperType + 'static> crate::tree_gen::Prepro<T> for Prepro {
    const USING: bool = true;
    fn preprocessing(&self, ty: T) -> std::result::Result<Acc, String> {
        self.clone().init(ty).map_err(|x| x.to_string())
    }
}

impl<T: HyperType + 'static> crate::tree_gen::Prepro<T> for &Prepro {
    const USING: bool = true;
    fn preprocessing(&self, ty: T) -> std::result::Result<Acc, String> {
        (*self).preprocessing(ty)
    }
}

impl<HAST: crate::types::TypeStore, Acc> crate::tree_gen::More<HAST, Acc> for Prepro {
    const ENABLED: bool = false;
    fn match_precomp_queries(
        &self,
        _stores: &HAST,
        _acc: &Acc,
        _label: Option<&str>,
    ) -> crate::tree_gen::PrecompQueries {
        Default::default()
    }
}

impl<HAST: crate::types::TypeStore, Acc> crate::tree_gen::More<HAST, Acc> for &Prepro {
    const ENABLED: bool = false;
    fn match_precomp_queries(
        &self,
        _stores: &HAST,
        _acc: &Acc,
        _label: Option<&str>,
    ) -> crate::tree_gen::PrecompQueries {
        Default::default()
    }
}

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
