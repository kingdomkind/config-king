use std::env;
use std::fs;
use std::process::Command;

use crate::unstatic;

use super::utilities;
use super::globals::*;
use colour::*;

pub fn remove_uninstalled_aur_directories(aur_table : mlua::Table, aur_location : String) {
    // Clean up AUR directory
    let entries = fs::read_dir(&aur_location);
    let mut entry_names = Vec::new();
    for entry in entries.unwrap() {
        let file_name = entry.unwrap().file_name().into_string().unwrap();
        entry_names.push(file_name);
    }
    
    for pair in aur_table.pairs::<mlua::Value, mlua::Value>() {
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

                if secondary_key.as_str() == "base" {
                    let secondary_val = secondary_val.to_string().unwrap();
                    temp.push(secondary_val);
                }
            }
        }

        for package in temp {
            if entry_names.contains(&package) {
                let index = entry_names.iter().position(|r| *r == package);
                entry_names.remove(index.unwrap());

            }
        }
    }

    for entry in entry_names {
        utilities::remove_path(aur_location.to_owned() + &entry);
    }   
}

pub fn pull_package(aur_location: String, package: String) -> bool {
    let og_directory = utilities::get_current_directory();
    let _ = env::set_current_dir(aur_location + "/" + &package);

    let output = Command::new("git")
    .arg("pull")
    .output()
    .expect("Failed to execute command");

    if output.status.success() {
        // white_ln!("Pulled (AUR) {}", string_str); redundant
    } else {
        red_ln!("{:?}", String::from_utf8_lossy(&output.stderr));
    }
    
    let _ = env::set_current_dir(og_directory);

    // Checking if already updated, if not, then build and continue
    if String::from_utf8_lossy(&output.stdout) != "Already up to date.\n" {
        return true;
    } else {
        grey_ln!("(AUR) {} is already up to date", package);
        return false;
    }
}

pub fn clone_package(aur_location: String, package: String, url: String) {
    let og_directory = utilities::get_current_directory();
    let _ = env::set_current_dir(aur_location);

    let mut output = Command::new("git");
    output.arg("clone");
    if String::is_empty(&url) {
        output.arg("https://aur.archlinux.org/".to_owned() + &package + ".git");
    } else {
        output.arg(url);
    }
    let success = utilities::send_output(output);

    if success {
        white_ln!("(AUR) Cloned {}", package);
    }

    let _ = env::set_current_dir(og_directory);
}

pub fn make_and_install_package(aur_location: String, base_package: String, sub_packages: Vec<String>) {

    // We assume the first value in packages is the base package
    let og_directory = utilities::get_current_directory();
    let _ = env::set_current_dir(aur_location + "/" + &base_package);

    white_ln!("(AUR) Building {}", base_package);

    let mut output = Command::new("makepkg");
    output.arg("-s");
    if unstatic!(ASSUME_YES) { output.arg("--noconfirm"); }

    let success = utilities::send_output(output);
    if success {
        green!("Built: ");
        white_ln!("(AUR) {}", base_package);
    }

    let output = Command::new("ls")
    .output()
    .expect("Failed to execute command");

    let possible_pkgs = String::from_utf8(output.stdout).unwrap();
    let possible_pkgs: Vec<&str> = possible_pkgs.split("\n").filter(|x| x.contains(".pkg.tar.zst")).collect();

    let mut output = Command::new("pacman"); // Issue - base package may NOT be the one you want to install, string pattern matching of .contains isn't good enough
    output.arg("-U");

    for package in &sub_packages {
        println!("User wants to install {}", package);
        let mut best_package: &str = "";
        let mut best_length = 9999;

        for option in &possible_pkgs {
            if option.contains(package) {
                if option.len() < best_length {
                    best_package = option;
                    best_length = option.len();
                }
            }
        }

        println!("Aur install of {}", best_package);
        output.arg(best_package);
    }

    if unstatic!(ASSUME_YES) { output.arg("--noconfirm"); }

    let success = utilities::send_output(output);
    if success {
        green!("Installed: ");
        white_ln!("(AUR) {}", base_package);
    }

    let _ = env::set_current_dir(og_directory);
}