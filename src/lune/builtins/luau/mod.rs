use mlua::prelude::*;

use crate::lune::util::TableBuilder;

mod options;
use options::{LuauCompileOptions, LuauLoadOptions};

const BYTECODE_ERROR_BYTE: u8 = 0;

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("compile", compile_source)?
        .with_function("load", load_source)?
        .build_readonly()
}

fn compile_source<'lua>(
    lua: &'lua Lua,
    (source, options): (LuaString<'lua>, LuauCompileOptions),
) -> LuaResult<LuaString<'lua>> {
    let bytecode = options.into_compiler().compile(source);

    match bytecode.first() {
        Some(&BYTECODE_ERROR_BYTE) => Err(LuaError::RuntimeError(
            String::from_utf8_lossy(&bytecode).into_owned(),
        )),
        Some(_) => lua.create_string(bytecode),
        None => panic!("Compiling resulted in empty bytecode"),
    }
}

fn load_source<'lua>(
    lua: &'lua Lua,
    (source, options): (LuaString<'lua>, LuauLoadOptions),
) -> LuaResult<LuaFunction<'lua>> {
    let mut chunk = lua.load(source.as_bytes()).set_name(options.debug_name);
    let env_changed = options.environment.is_some();

    if let Some(custom_environment) = options.environment {
        let environment = lua.create_table()?;

        // Inject all globals into the environment
        if options.inject_globals {
            for pair in lua.globals().pairs() {
                let (key, value): (LuaValue, LuaValue) = pair?;
                environment.set(key, value)?;
            }

            if let Some(global_metatable) = lua.globals().get_metatable() {
                environment.set_metatable(Some(global_metatable));
            }
        
        // Since we don't need to set the global metatable, we can just set a custom metatable if it exists
        } else if let Some(custom_metatable) = custom_environment.get_metatable() {
            environment.set_metatable(Some(custom_metatable));
        }

        // Inject the custom environment
        for pair in custom_environment.pairs() {
            let (key, value): (LuaValue, LuaValue) = pair?;
            environment.set(key, value)?;
        }

        chunk = chunk.set_environment(environment);
    }
    
    // Enable JIT if codegen is enabled and the environment hasn't changed, otherwise disable JIT since it'll fall back anyways
    lua.enable_jit(options.codegen_enabled && !env_changed);
    let function = chunk.into_function()?;
    lua.enable_jit(true);

    Ok(function)
}