use mlua::prelude::*;
use mlua::{Lua, MetaMethod, UserData, UserDataFields, UserDataMethods};

#[test]
fn test_mlua() -> LuaResult<()> {
    let lua = Lua::new();

    let map_table = lua.create_table()?;
    map_table.set(1, "one")?;
    map_table.set("two", 2)?;

    lua.globals().set("map_table", map_table)?;

    lua.load("for k,v in pairs(map_table) do print(k,v,k,v) end")
        .exec()?;

    Ok(())
}

#[test]
fn test_mlua_scope() -> LuaResult<()> {
    let lua = Lua::new();
    let mut rust_val = 0;

    lua.scope(|scope| {
        // We create a 'sketchy' Lua callback that holds a mutable reference to the variable
        // `rust_val`. Outside of a `Lua::scope` call, this would not be allowed
        // because it could be unsafe.

        lua.globals().set(
            "sketchy",
            scope.create_function_mut(|_, ()| {
                rust_val = 42;
                Ok(())
            })?,
        )?;

        lua.load("sketchy()").exec()
    })?;

    assert_eq!(rust_val, 42);
    Ok(())
}

#[derive(Default, Debug)]
struct Rectangle {
    length: u32,
    width: u32,
}

impl UserData for Rectangle {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("length", |_, this| Ok(this.length));
        fields.add_field_method_set("length", |_, this, val| {
            this.length = val;
            Ok(())
        });
        fields.add_field_method_get("width", |_, this| Ok(this.width));
        fields.add_field_method_set("width", |_, this, val| {
            this.width = val;
            Ok(())
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("area", |_, this, ()| Ok(this.length * this.width));
        methods.add_method("diagonal", |_, this, ()| {
            Ok((this.length.pow(2) as f64 + this.width.pow(2) as f64).sqrt())
        });

        // Constructor
        methods.add_meta_function(MetaMethod::Call, |_, ()| Ok(Rectangle::default()));
    }
}

impl<'lua> FromLua<'lua> for Rectangle {
    fn from_lua(value: LuaValue<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        let v = value.as_userdata().unwrap();
        Ok(Self {
            length: v.get("length")?,
            width: v.get("width")?,
        })
    }
}

#[test]
fn test_userdata() -> mlua::Result<()> {
    let lua = Lua::new();
    let rect = Rectangle::default();
    lua.globals().set("rect", rect)?;
    lua.load(
        "
    rect.width = 10
    rect.length = 5
    assert(rect:area() == 50)
    assert(rect:diagonal() - 11.1803 < 0.0001)
",
    )
    .exec()?;
    let rect = lua.globals().get::<_, Rectangle>("rect")?;
    dbg!(rect);
    Ok(())
}

#[test]
fn test_userdata_scope() -> mlua::Result<()> {
    let lua = Lua::new();
    let mut rect = Rectangle::default();
    lua.scope(|scope| {
        let rect = scope.create_userdata_ref_mut(&mut rect)?;
        lua.globals().set("rect", rect)?;
        lua.load(
            "
        rect.width = 10
        rect.length = 5
        assert(rect:area() == 50)
        assert(rect:diagonal() - 11.1803 < 0.0001)
    ",
        )
        .exec()
    })?;
    dbg!(rect);
    Ok(())
}

#[test]
fn test_function_call() -> mlua::Result<()> {
    let lua = Lua::new();
    let sum: mlua::Function = lua
        .load(
            r#"
        function(a, b)
            return a + b
        end
"#,
        )
        .eval()?;

    assert_eq!(sum.call::<_, u32>((3, 4))?, 3 + 4);
    Ok(())
}

#[test]
fn test_scope_function_call() -> mlua::Result<()> {
    let lua = Lua::new();
    let mut rect = Rectangle::default();
    lua.scope(|scope| {
        let rect = scope.create_userdata_ref_mut(&mut rect)?;
        let func: mlua::Function = lua
            .load(
                r#"
        function(rect)
            rect.width = 10
            rect.length = 5
            assert(rect:area() == 50)
            assert(rect:diagonal() - 11.1803 < 0.0001)
        end
"#,
            )
            .eval()?;

        func.call::<_, ()>(rect)?;
        Ok(())
    })?;
    dbg!(rect);
    Ok(())
}

#[test]
fn test_scope_function_call2() -> mlua::Result<()> {
    let lua = Lua::new();
    let mut rect = Rectangle::default();
    let func: mlua::Function = lua
        .load(
            r#"
    function(rect)
        rect.width = 10
        rect.length = 5
        assert(rect:area() == 50)
        assert(rect:diagonal() - 11.1803 < 0.0001)
    end
"#,
        )
        .eval()?;
    lua.scope(|scope| {
        let rect = scope.create_userdata_ref_mut(&mut rect)?;
        func.call::<_, ()>(rect)?;
        Ok(())
    })?;
    dbg!(rect);
    Ok(())
}

#[test]
fn test_scope_function_call_named() -> mlua::Result<()> {
    let lua = Lua::new();
    lua.sandbox(true)?;
    // lua.set_memory_limit(260000)?; // fatal runtime error: Rust cannot catch foreign exceptions
    dbg!(&lua.used_memory());
    lua.load(
        r#"
    local a = 42
    function acc (c)
        a += c
    end
    function finish ()
        return a
    end
    function compress (m)
        -- equivalent to m & Ob11
        return bit32.band(m,0b11)
    end
"#,
    )
    .exec()?;
    // let init = lua.globals().get::<_, mlua::Function>("init")?;
    dbg!(&lua.used_memory());
    let acc = lua.globals().get::<_, mlua::Function>("acc")?;
    let finish = lua.globals().get::<_, mlua::Function>("finish")?;
    let compress = lua.globals().get::<_, mlua::Function>("compress")?;
    let mut m = mlua::Value::Nil;
    lua.scope(|_scope| {
        // a = init.call(())?;
        acc.call::<_, ()>((1,))?;
        // let a = scope.create_userdata_ref_mut(&mut a)?;
        // func.call::<_, ()>((a, 1))?;
        dbg!(&lua.used_memory());
        m = finish.call(())?;
        let comp: mlua::Value = compress.call((m.clone(),))?;
        dbg!(comp);
        Ok(())
    })?;
    dbg!(m);
    Ok(())
}

#[test]
fn test_pre_post_mcc_per_file() -> mlua::Result<()> {
    let lua = Lua::new();
    lua.sandbox(true)?;
    // lua.set_memory_limit(260000)?; // fatal runtime error: Rust cannot catch foreign exceptions
    dbg!(&lua.used_memory());
    lua.load(
        r#"
    local mcc_per_file = {}
    local path = []
    -- return false to stop descending
    function pre(s)
        if s.is_directory()
        and c.mcc > 10 then
            path.push(s.name)
            return true
        else if s.is_file()
        and c.mcc > 10 then
            path.push(s.name)
            mcc_per_file.insert(
                vector.join("/", path),
                c.mcc
            )
            return false
        else
            return false
        end
    end
    function post(c)
        path.pop()
    end
"#,
    )
    .exec()
}

#[test]
fn test_pre_post_max_mcc_file() -> mlua::Result<()> {
    let lua = Lua::new();
    lua.sandbox(true)?;
    // lua.set_memory_limit(260000)?; // fatal runtime error: Rust cannot catch foreign exceptions
    dbg!(&lua.used_memory());
    lua.load(
        r#"
        local max_mcc = 0
        local file = ""
        local path = []
        -- return false to stop descending
        function pre(s)
            if s.is_directory()
            and c.mcc > max_mcc then
                path.push(s.name)
                return true
            else if s.is_file()
            and c.mcc > max_mcc then
                file = vector.join("/", path, s.name)
                max_mcc = c.mcc
                return false
            else
                return false
            end
        end
        function post(c)
            path.pop()
        end
    "#,
    )
    .exec()
}

#[test]
fn test_pre_post_max_mcc_ratio() -> mlua::Result<()> {
    let lua = Lua::new();
    lua.sandbox(true)?;
    // lua.set_memory_limit(260000)?; // fatal runtime error: Rust cannot catch foreign exceptions
    dbg!(&lua.used_memory());
    lua.load(
        r#"
        local max_ratio = 0
        local file = ""
        local path = []
        -- return false to stop descending
        function pre(s)
            if s.is_directory() or s.is_file()
            and c.mcc / c.lines > max_ratio then
                path.push(s.name)
                return true
            else if s.is_method()
            and c.mcc / c.lines > max_mcc then
                file = vector.join("/", path, s.name)
                max_ratio = c.mcc / c.lines
                return false
            else
                return false
            end
        end
        function post(c)
            path.pop()
        end
    "#,
    )
    .exec()
}
