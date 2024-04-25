use mlua::prelude::*;
use std::fs;

fn main() -> Result<(), mlua::Error> {
    let lua = Lua::new();

    // Read the Lua file
    let lua_script = fs::read_to_string("/home/pika/Software/Rust/firstrust/src/config.lua")?;

    // Load the Lua script
    let globals = lua.globals();
    lua.load(&lua_script).exec()?;

    // Get the 'config' table and iterate over it's values
    let config: mlua::Table = globals.get("Config")?;
    for pair in config.pairs::<mlua::Value, mlua::Value>() {
        let (key, value) = pair?;
        match value {
            
            // Seeing if the value is of type TABLE
            mlua::Value::Table(table) => {
                for pair in table.pairs::<mlua::Value, mlua::Value>() {
                    let (_index, value) = pair?;
                    println!("{:?} = {:?}", key, value);
                }
            },

            // Seeing if the value is of type STRING
            mlua::Value::String(string) => {
                println!("{:?}", string);
            },

            // Catch all function
            _ => (),

        }
    }

    Ok(())
}
