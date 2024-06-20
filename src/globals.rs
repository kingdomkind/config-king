use std::{collections::HashMap, env, fs, sync::Mutex};

use colour::*;
use once_cell::sync::Lazy;

// Config Variables
pub static DIRECTORY: Lazy<Mutex<String>> = Lazy::new(|| {
    let ret = match_arguments("DIRECTORY".to_string());
    if ret == "" { 
        let file = fs::read_to_string(env::current_dir()
        .expect("Unable to get current directory").to_str()
        .expect("Unable to convert current directory to str").to_string() + "/config.lua").unwrap();
        return Mutex::new(file);
    } else { 
        let file = fs::read_to_string(ret).unwrap();
        return Mutex::new(file);
    };
});

pub static PACKAGE_REMOVE_WARN_LIMIT: Lazy<Mutex<u32>> = Lazy::new(|| {
    let ret = match_arguments("PACKAGE_REMOVE_WARN_LIMIT".to_string());
    if ret == "" { return Mutex::new(5); } else  { return Mutex::new(ret.parse().unwrap()); };
});

pub static SEE_STDERR: Lazy<Mutex<bool>> = Lazy::new(|| {
    let ret = match_arguments("SEE_STDERR".to_string());
    if ret == "false" { return Mutex::new(false); } else  { return Mutex::new(true); };
});

pub static SEE_STDOUT: Lazy<Mutex<bool>> = Lazy::new(|| {
    let ret = match_arguments("SEE_STDOUT".to_string());
    if ret == "false" { return Mutex::new(false); } else  { return Mutex::new(true); };
});

pub static ASSUME_YES: Lazy<Mutex<bool>> = Lazy::new(|| {
    let ret = match_arguments("ASSUME_YES".to_string());
    if ret == "false" { return Mutex::new(false); } else  { return Mutex::new(true); };
});

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