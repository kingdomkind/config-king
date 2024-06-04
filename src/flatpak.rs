use std::process::Command;

use colour::*;

use super::globals::*;
use super::utilities;

pub fn remove_packages(package_names: Vec<String>) {
    if package_names.len() > 0 {
        let mut output = Command::new("flatpak");
        output.arg("uninstall");
        if ASSUME_YES { output.arg("--assumeyes"); }

        for value in &package_names {
            output.arg(value);
        }

        let success = utilities::send_output(output);
        if success {
            green!("Removed: ");
            white_ln!("{:?}", package_names);
        }
    
        let mut output = Command::new("flatpak");
        output.arg("uninstall");
        output.arg("--unused");
        if ASSUME_YES { output.arg("--assumeyes"); }

        let _success = utilities::send_output(output);
    }
}