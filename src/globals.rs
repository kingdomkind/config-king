use std::{env, fs};

use colour::*;

// Config Variables
pub const SEE_STDOUT : bool = true;
pub const SEE_STDERR : bool = true;
pub const ASSUME_YES : bool = true;
pub const PACKAGE_REMOVE_WARN_LIMIT : u32 = 5;
pub const DEFAULT_YES : bool = true;

/*
fn match_arguments(target: String) {
    let mut arguments: Vec<String> = env::args().collect();
    arguments.remove(0);

    // Read the Lua file, cargo run should be run from the directory of the config file if no directory is specified
    let mut lua_script: String = String::new();

    for arg in arguments {
        let pos = arg.find('=');

        if !pos.is_none() {
            let key = &arg[..pos.unwrap()];
            let value = &arg[pos.unwrap() + 1..];

            match key {
                "directory" => {
                    lua_script = fs::read_to_string(value.to_string())?;
                },
                _ => { yellow!("Warning: "); white_ln!("{} is not a recognised key", key); }
            }
        } else {
            yellow!("Warning: ");
            white_ln!("No '=' found in the argument {}", arg)
        }
    }
} */