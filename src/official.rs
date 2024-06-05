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