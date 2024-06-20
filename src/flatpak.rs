use std::process::Command;

use colour::*;

use crate::unstatic;

use super::globals::*;
use super::utilities;

pub fn remove_packages(package_names: Vec<String>) {
    if package_names.len() > 0 {
        let mut output = Command::new("flatpak");
        output.arg("uninstall");
        if unstatic!(ASSUME_YES) { output.arg("--assumeyes"); }

        for package in &package_names {
            output.arg(package);
        }

        let success = utilities::send_output(output);
        if success {
            green!("Removed: ");
            white_ln!("{:?}", package_names);
        }
    
        let mut output = Command::new("flatpak");
        output.arg("uninstall");
        output.arg("--unused");
        if unstatic!(ASSUME_YES) { output.arg("--assumeyes"); }

        let _success = utilities::send_output(output);
    }
}

pub fn install_packages(package_names: Vec<String>) {

    for package in &package_names {
        white_ln!("(FLATPAK) Attempting to install {}", package);
    }

    let mut output = Command::new("flatpak");
    output.arg("install");

    for package in &package_names {
        output.arg(package);
    }

    if unstatic!(ASSUME_YES) { output.arg("--assumeyes"); }

    let success = utilities::send_output(output);
    if success {
        for package in &package_names {
            green!("Installed: ");
            white_ln!("{}", package);
        }
    }
}

pub fn upgrade_all_packages() {
    let mut output = Command::new("flatpak");
    output.arg("update");
    if unstatic!(ASSUME_YES) { output.arg("--assumeyes"); }
    utilities::send_output(output);
}

pub fn subtract_vec(rust_table : Vec<String>, lua_table : mlua::Table) -> Vec<String> {

    let mut rust_table = rust_table;

    for pair in lua_table.pairs::<mlua::Value, mlua::Value>() {
        let Ok((_key, value)) = pair else { panic!() };

        let package = value.to_string().unwrap();
        if rust_table.contains(&package) {
            let index = rust_table.iter().position(|r| *r == package);
            rust_table.remove(index.unwrap());
        }
    };

    return rust_table;
}