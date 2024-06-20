use std::{collections::HashMap, env, fs, sync::Mutex};

use colour::*;
use once_cell::sync::Lazy;

// Config Variables
pub const SEE_STDOUT : bool = true;
pub const SEE_STDERR : bool = true;
pub const ASSUME_YES : bool = true;
pub const PACKAGE_REMOVE_WARN_LIMIT : u32 = 5;
//pub const DEFAULT_YES : bool = true;



pub static DEFAULT_YES: Lazy<Mutex<bool>> = Lazy::new(|| {
    let ret = match_arguments("DEFAULT_YES".to_string());
    if ret == "false" { return Mutex::new(false); } else  { return Mutex::new(true); };
});

fn match_arguments(target: String) -> String {
    let mut configured_arguments: HashMap<String, String> = HashMap::new();
    let mut arguments: Vec<String> = env::args().collect();
    arguments.remove(0);

    for arg in arguments {
        let pos = arg.find('=');

        if !pos.is_none() {
            let key = &arg[..pos.unwrap()];
            let value = &arg[pos.unwrap() + 1..];
            configured_arguments.insert(key.to_string(), value.to_string());
        } else {
            yellow!("Warning: ");
            white_ln!("No '=' found in the argument {}", arg)
        }
    }

    if configured_arguments.contains_key(&target) {
        return configured_arguments[&target].clone();
    } else {
        return String::new();
    }
} 