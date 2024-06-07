use std::process::Command;
use colour::*;

use super::globals::*;
use super::utilities;

pub fn install_packages(package_names: Vec<String>) {
    let mut output = Command::new("sudo");
    output.arg("pacman");
    output.arg("-S");

    for package in &package_names {
        output.arg(package);
    }

    if ASSUME_YES { output.arg("--noconfirm"); }

    let success = utilities::send_output(output);
    if success {
        for package in package_names {
            green!("Installed: ");
            white_ln!("{}", package);
        }
    }
}

pub fn remove_system_packages(package_names: Vec<String>) {
    if package_names.len() > 0 {
        let mut output = Command::new("sudo");
        output.arg("pacman");
        output.arg("-Rns");
        if ASSUME_YES { output.arg("--noconfirm"); }

        for value in &package_names {
            output.arg(value);
        }
    
        let success : bool = utilities::send_output(output);
        if success {
            green!("Removed: ");
            white_ln!("{:?}", package_names);
        } else {
            yellow!("Warning: ");
            white_ln!("Failed to remove {:?}", package_names);
        }
    }
}

pub fn upgrade_all_packages() {
    let mut output = Command::new("sudo");
    output.arg("pacman");
    output.arg("-Syu");
    if ASSUME_YES { output.arg("--noconfirm"); }
    utilities::send_output(output);
}

pub fn is_package_group(package: String) -> bool {
    let output = Command::new("pacman")
    .arg("-Sg")
    .arg(package)
    .output()
    .expect("Failed to execute command");

    let raw_out: String = String::from_utf8(output.stdout).unwrap();
    let out : Vec<&str> = raw_out.lines().collect();

    if out.len() == 0 {
        return false;
    } else {
        return true;
    }
}

pub fn get_packages_in_group(package: String) -> Vec<String> {
    let output = Command::new("pacman")
    .arg("-Sg")
    .arg(package)
    .output()
    .expect("Failed to execute command");

    let raw_out: String = String::from_utf8(output.stdout).unwrap();
    let out : Vec<String> = utilities::vec_str_to_string(raw_out.lines().collect());
    
    return out;
}

pub fn subtract_vec(rust_table : Vec<String>, lua_table : mlua::Table) -> Vec<String> {

    let mut rust_table = rust_table;
    
    for pair in lua_table.pairs::<mlua::Value, mlua::Value>() {
        let Ok((_key, value)) = pair else { panic!() };

        let mut temp: Vec<String> = Vec::new();

        if value.is_string() {
            temp.push(value.to_string().unwrap());
        }

        if value.is_table() {
            let value = value.as_table().unwrap().clone().pairs::<mlua::Value, mlua::Value>();

            for secondary_pair in value {
                let (secondary_key, secondary_val) = secondary_pair.unwrap();
                let secondary_key = secondary_key.to_string().unwrap();

                if secondary_key.as_str() == "sub" {
                    let secondary_val = secondary_val.as_table().unwrap();

                    for tertiary_pair in secondary_val.clone().pairs::<mlua::Value, mlua::Value>() {
                        let (_tertiary_key, tertiary_val) = tertiary_pair.unwrap();
                        let tertiary_val = tertiary_val.to_string().unwrap();
                        temp.push(tertiary_val.clone());
                    }
                }
            }
        }

        for package in temp {
            if rust_table.contains(&package) {
                let index = rust_table.iter().position(|r| *r == package);
                rust_table.remove(index.unwrap());
            } else {
                println!("Didnt contian {}", package);
            }
        }
    };

    return rust_table;
}