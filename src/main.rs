use mlua::prelude::*;
use std::{env, fs, process::Command, string};

fn main() -> Result<(), mlua::Error> {
    let lua = Lua::new();

    // Read the Lua file -- relative diretory should be ran from project base for testing (ie. in the main folder)
    let lua_script: String = fs::read_to_string(env::current_dir()
    .expect("Unable to get current directory").to_str()
    .expect("Unable to convert current directory to str").to_string() + "/src/config.lua")?;

    // Load the Lua script
    let globals = lua.globals();
    lua.load(&lua_script).exec()?;

    // Get currently installed packages
    let output = Command::new("pacman")
    .arg("-Qeq")
    .output()
    .expect("Failed to execute command");

    if !output.status.success() {
        println!("Command executed with failing error code");
    }

    let raw_packages = String::from_utf8(output.stdout).unwrap();
    let packages : Vec<&str> = raw_packages.lines().collect();
    /*
    for value in &packages {
        println!("{}", value);
    }*/

    // Get the 'config' table and iterate over it's values
    let config: mlua::Table = globals.get("Config")?;
    for pair in config.pairs::<mlua::Value, mlua::Value>() {
        let (_key, value) = pair?;
        match value {

            /* CURRENTLY THE EXTRA FUNCTIONALITY PROVIDED BY TABLES IS NOT YET IMPLEMENTED, AS I YET TO HAVE A USE FOR IT. WHEN I DO, IT WILL BE IMPLEMENTED
            // Seeing if the value is of type TABLE
            mlua::Value::Table(table) => {
                for pair in table.pairs::<mlua::Value, mlua::Value>() {
                    let (_index, value) = pair?;
                    println!("{:?} = {:?}", key, value);
                }
            },

            */
            // Seeing if the value is of type STRING
            mlua::Value::String(string) => {

                let string_str = string.to_str().unwrap();

                if packages.contains(&string_str) {
                    print!("Found");
                }

                println!("{:?}", string);
            },

            // Catch all function
            _ => (),

        }
    }

    Ok(())
}
