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