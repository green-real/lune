use mlua::prelude::*;

mod fs;
mod net;
mod process;
mod require;
mod serde;
mod stdio;
mod task;
mod top_level;

const BUILTINS_AS_GLOBALS: &[&str] = &["fs", "net", "process", "stdio", "task"];

pub fn create(lua: &'static Lua, args: Vec<String>) -> LuaResult<()> {
    // Create all builtins
    let builtins = vec![
        ("fs", fs::create(lua)?),
        ("net", net::create(lua)?),
        ("process", process::create(lua, args)?),
        ("serde", self::serde::create(lua)?),
        ("stdio", stdio::create(lua)?),
        ("task", task::create(lua)?),
    ];

    // TODO: Remove this when we have proper LSP support for custom
    // require types and no longer need to have builtins as globals
    let lua_globals = lua.globals();
    for name in BUILTINS_AS_GLOBALS {
        let builtin = builtins.iter().find(|(gname, _)| gname == name).unwrap();
        lua_globals.set(*name, builtin.1.clone())?;
    }

    // Create our importer (require) with builtins
    let require_fn = require::create(lua, builtins)?;

    // Create all top-level globals
    let globals = vec![
        ("require", require_fn),
        ("print", lua.create_function(top_level::top_level_print)?),
        ("warn", lua.create_function(top_level::top_level_warn)?),
        ("error", lua.create_function(top_level::top_level_error)?),
        (
            "printinfo",
            lua.create_function(top_level::top_level_printinfo)?,
        ),
    ];

    // Set top-level globals and seal them
    for (name, global) in globals {
        lua_globals.set(name, global)?;
    }
    lua_globals.set_readonly(true);

    Ok(())
}
