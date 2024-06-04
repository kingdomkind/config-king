use std::{io, path::Path, process::{exit, Command}};
use super::globals::*;
use colour::*;

pub fn is_system_package_installed(package_name: &str) -> bool {
    let output = Command::new("which")
    .arg(package_name)
    .output()
    .expect("Failed to execute command");

    return output.status.success();
}

pub fn vec_str_to_string(vector : Vec<&str>) -> Vec<String> {
    return vector.into_iter().map(|x| x.to_string()).collect();
}

pub fn get_installed_system_packages() -> Vec<String> {
    // Get currently installed packages -- this one needs to use .output to get the stdout.
    let output = Command::new("pacman")
    .arg("-Qeq")
    .output()
    .expect("Failed to execute command");

    if !output.status.success() {
        println!("Unable to get list of installed packages, exiting.");
        exit(1);
    }

    // Get readable system packages
    let raw_packages: String = String::from_utf8(output.stdout).unwrap();
    let packages: Vec<&str> = raw_packages.lines().collect();

    // Convert elements to string
    return vec_str_to_string(packages)
}


pub fn get_confirmation() -> bool {
    let mut accepted_response = false;
    let mut choice : bool = false;

    while !accepted_response {
        let mut response = String::new();
        accepted_response = true;

        io::stdin().read_line(&mut response).expect("failed to readline");

        match response.trim().to_lowercase().as_str() {
            "yes" | "y" | "ye" => choice = true,
            "no" | "n" | "nah" => choice = false,
            "" => { if DEFAULT_YES { choice = true; } else { choice = false; } },
            _ => accepted_response = false,
        }
    }

    return choice;
}

// Gets the current directory the program is in
pub fn get_current_directory() -> String {
    let current_dir = Command::new("pwd").output().expect("Couldn't get current directory");
    let mut og_directory = String::from_utf8(current_dir.stdout).unwrap();
    og_directory.truncate(og_directory.len() - 1);
    return  og_directory;
}

// Runs Commands, and displays the output and returns if successful
pub fn send_output(mut output : Command) -> bool {

    if !SEE_STDOUT { output.stdout(std::process::Stdio::null()); }
    if !SEE_STDERR { output.stderr(std::process::Stdio::null()); }

    let mut spawned = output.spawn().expect("Unable to output command");
    let wait = spawned.wait().expect("Failed to wait for output to end");
    return wait.success();
}

pub fn remove_path(path : String) {
    println!("Entered path remover");
    if Path::new(&path).exists() {
        let mut ret: Option<bool> = None;
        if Path::new(&path).is_dir() {
            yellow!("Warning: ");
            white_ln!("Are you sure you would like to remove the directory at {} [y/n]", path);
            let confirm = get_confirmation();
            if confirm {
                let mut output = Command::new("sudo");
                output.arg("rm");
                output.arg("-r");
                output.arg(&path);
                ret = Some(send_output(output));

            }
        } else {
            yellow!("Warning: ");
            white_ln!("Are you sure you would like to remove the file at {} [y/n]", path);
            let confirm = get_confirmation();
            if confirm {
                let mut output = Command::new("sudo");
                output.arg("rm");
                output.arg(&path);
                ret = Some(send_output(output));
            }
        };

        if !ret.is_none() {
            match ret.unwrap() {
                false => {
                    red!("ERROR: ");
                    white_ln!("Failed to delete path at {}", path);
                },
                true => {
                    green!("Removed: ");
                    white_ln!("Removed path at {}", path);
                }
            }
        } else {
            white_ln!("Skipped removing path");
        }
    }
}

pub fn subtract_lua_vec(rust_table : Vec<String>, lua_table : mlua::Table) -> Vec<String> {

    let mut rust_table = rust_table;
    for pair in lua_table.pairs::<mlua::Value, mlua::Value>() {
        let Ok((_key, value)) = pair else { panic!() };
        match value {

            mlua::Value::String(string) => {
                let string = string.to_str().unwrap().to_string();
                if rust_table.contains(&string) {
                    let index = rust_table.iter().position(|r| *r == string);
                    rust_table.remove(index.unwrap());
                }
            },
            _ => (),

        }
    };

    return rust_table;
}