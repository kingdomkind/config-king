use std::process::Command;

use super::globals::*;
use super::utilities;

pub fn install_packages(package_names: Vec<&str>) {
    let mut output = Command::new("sudo");
    output.arg("pacman");
    output.arg("-S");

    for package in package_names {
        output.arg(package);
    }
    
    if ASSUME_YES { output.arg("--noconfirm"); }
    utilities::send_output(output);
}