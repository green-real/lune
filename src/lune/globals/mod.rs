use mlua::prelude::*;

use super::util::TableBuilder;

use mlua_luau_scheduler::Functions;

pub mod g_table;
mod print;
mod require;
mod version;
mod warn;

pub fn inject_all(lua: &Lua) -> LuaResult<()> {
    let all = TableBuilder::new(lua)?
        .with_value("_VERSION", version::create(lua)?)?
        .with_value("print", print::create(lua)?)?
        .with_value("require", require::create(lua)?)?
        .with_value("warn", warn::create(lua)?)?
        .build_readonly()?;

    let fns = Functions::new(lua)?;
    let co = lua.globals().get::<_, LuaTable>("coroutine")?;
    co.set("resume", fns.resume.clone())?;
    co.set("wrap", fns.wrap.clone())?;

    for res in all.pairs() {
        let (key, value): (LuaValue, LuaValue) = res.unwrap();
        lua.globals().set(key, value)?;
    }

    Ok(())
}
