use mlua::prelude::*;
use std::{env, fs, process::Command};

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

    Command::new("pacman")
    .arg("-Syu")
    .output()
    .expect("Failed to exec command");

    if !output.status.success() {
        println!("Command executed with failing error code");
    }

    let raw_packages = String::from_utf8(output.stdout).unwrap();
    let mut packages : Vec<&str> = raw_packages.lines().collect();

    // Get the 'config' table and iterate over it's values
    let config: mlua::Table = globals.get("Config")?;
    for pair in config.pairs::<mlua::Value, mlua::Value>() {
        let (_key, value) = pair?;
        match value {

            // Seeing if the value is of type STRING
            mlua::Value::String(string) => {

                let string_str = string.to_str().unwrap();

                if packages.contains(&string_str) {
                    let index = packages.iter().position(|&r| r == string_str);
                    packages.remove(index.unwrap());
                } else {
                    println!("Doesnt contain..");

                    let output = Command::new("pacman")
                    .arg("-S signal-desktop")
                    .arg("--noconfirm")
                    .output()
                    .expect("Failed to execute command");
                
                    println!("{}", string_str);
                    println!("{:?}", output);
                }
            },

            // Catch all function
            _ => (),

        }
    }

    for value in &packages {
        println!("{}", value);
    }

    Ok(())
}
