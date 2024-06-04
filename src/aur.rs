use std::fs;
use std::path::Path;
use std::process::Command;
use super::utilities;
use super::globals::*;
use colour::*;

// Builds AUR packages and installs them
pub fn build_aur(name : &str) {
    white_ln!("(AUR) Building {}", name);

    let mut output = Command::new("makepkg");
    output.arg("-si");
    if ASSUME_YES { output.arg("--noconfirm"); }

    let success = utilities::send_output(output);
    if success {
        green!("Installed: ");
        white_ln!("(AUR) {}", name);
    }
}

pub fn remove_uninstalled_aur_directories(aur_table : mlua::Table, aur_location : String) {
    // TODO CHECK IF DIRECTORY EXISTS - ON FIRST RUN IT DOESNT!

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