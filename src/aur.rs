use std::env;
use std::fs;
use std::process::Command;
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
    let aur_packages_to_remove = utilities::subtract_lua_vec(entry_names, aur_table.clone());

    for entry in aur_packages_to_remove {
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

pub fn clone_package(aur_location: String, package: String) {
    let og_directory = utilities::get_current_directory();
    let _ = env::set_current_dir(aur_location);

    let output = Command::new("git")
    .arg("clone")
    .arg("https://aur.archlinux.org/".to_owned() + &package + ".git")
    .output()
    .expect("Failed to execute command");

    if output.status.success() {
        white_ln!("(AUR) Cloned {}", package);
    } else {
        red_ln!("{:?}", String::from_utf8_lossy(&output.stderr));
    }

    let _ = env::set_current_dir(og_directory);
}

pub fn make_and_install_package(aur_location: String, packages: Vec<String>) {

    // We assume the first value in packages is the base package
    let og_directory = utilities::get_current_directory();
    let _ = env::set_current_dir(aur_location + "/" + &packages[0]);

    white_ln!("(AUR) Building {}", packages[0]);

    let mut output = Command::new("makepkg");
    output.arg("-s");
    if ASSUME_YES { output.arg("--noconfirm"); }

    let success = utilities::send_output(output);
    if success {
        green!("Built: ");
        white_ln!("(AUR) {}", packages[0]);
    }

    let output = Command::new("ls")
    .output()
    .expect("Failed to execute command");

    let possible_pkgs = String::from_utf8(output.stdout).unwrap();
    let possible_pkgs: Vec<&str> = possible_pkgs.split("\n").filter(|x| x.contains(".pkg.tar.zst")).collect();

    let mut output = Command::new("sudo"); // Issue - base package may NOT be the one you want to install, string pattern matching of .contains isn't good enough
    output.arg("pacman");
    output.arg("-U");

    for package in &packages {
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

    if ASSUME_YES { output.arg("--noconfirm"); }

    let success = utilities::send_output(output);
    if success {
        green!("Installed: ");
        white_ln!("(AUR) {}", packages[0]);
    }

    let _ = env::set_current_dir(og_directory);
}