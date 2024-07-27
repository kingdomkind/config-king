use std::process::Command;
use colour::*;

use crate::unstatic;

use super::globals::*;
use super::utilities;

pub fn mark_package_as_explicit(package: String) {
    let mut output = Command::new(unstatic!(AUTH_AGENT));
    output.arg("pacman");
    output.arg("-D");
    output.arg("--asexplicit");
    output.arg(&package);

    let _success = utilities::send_output(output);
}

pub fn install_packages(package_names: Vec<String>) {
    let mut output = Command::new(unstatic!(AUTH_AGENT));
    output.arg("pacman");
    output.arg("-S");

    for package in &package_names {
        output.arg(package);
    }

    if unstatic!(ASSUME_YES) { output.arg("--noconfirm"); }

    let success = utilities::send_output(output);
    if success {
        for package in package_names {
            green!("Installed: ");
            white_ln!("{}", package);
        }
    }
}

pub fn remove_system_packages(package_names: Vec<String>) {
    // Clean unused deps
    println!("Removing Unused Dependencies");
    let command = if unstatic!(ASSUME_YES) {
        format!("pacman -Qtdq | {} pacman -Rns --noconfirm -", unstatic!(AUTH_AGENT))
    } else {
        format!("pacman -Qtdq | {} pacman -Rns -", unstatic!(AUTH_AGENT))
    };

    let _output = Command::new("sh")
    .arg("-c")
    .arg(command)
    .output();

    if package_names.len() > 0 {
        let mut output = Command::new(unstatic!(AUTH_AGENT));
        output.arg("pacman");
        output.arg("-Rns");
        if unstatic!(ASSUME_YES) { output.arg("--noconfirm"); }

        for value in &package_names {
            output.arg(value);
        }
    
        let success = output.output().unwrap();

        if success.status.success() {
            green!("Removed: ");
            white_ln!("{:?}", package_names);
        } else {
            // We need to check if it is a dependency of another package
            let mut packages_to_dep: Vec<String> = Vec::new();
            let mut package_names = package_names.clone();

            for package in &package_names {
                println!("{}", package);
            }

            for line in String::from_utf8(success.stdout).unwrap().split('\n') {
                if line.contains("breaks dependency") {
                    let words: Vec<&str> = line.split(' ').collect();
                    let target = words[2].to_string();
                    let index_option = package_names.iter().position(|x| *x == target);
                    // It can appear multiple times, and we can only remove it once
                    match index_option {
                        Some(_) => {
                            packages_to_dep.push(target.clone());
                            package_names.remove(index_option.unwrap());
                        },
                        None => (),
                    }
                }
            }

            for package in &packages_to_dep {
                let mut dep = Command::new(unstatic!(AUTH_AGENT));
                dep.arg("pacman");
                dep.arg("--asdep");
                dep.arg("-D");
                dep.arg(package);
                let _success = utilities::send_output(dep);
            }

            // Isn't a dependency, throw error
            if packages_to_dep.len() == 0 {
                yellow!("Warning: ");
                white_ln!("Failed to remove {:?}", package_names);
            } else {
                // Is a dependency, try again without that package (that package is now marked as dep)
                remove_system_packages(package_names);
            }
        }
    }
}

pub fn upgrade_all_packages() {
    let mut output = Command::new(unstatic!(AUTH_AGENT));
    output.arg("pacman");
    output.arg("-Syu");
    if unstatic!(ASSUME_YES) { output.arg("--noconfirm"); }
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
            }
        }
    };

    return rust_table;
}