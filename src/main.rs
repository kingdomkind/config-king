use mlua::prelude::*;
use std::{env, fs, process::{exit, Command}};

/*
BIG TODOS:

    => The order of installing then removing packages needs to be reversed. You need to first remove the packages then install packages to prevent dependency issues.
    => Symlinks

*/

// Config Variables
const SEE_STDOUT : bool = false;
const SEE_STDERR : bool = true;
const ASSUME_YES : bool = true;

// Runs Commands, and displays the output and returns if successful
fn send_output(mut output : Command) -> bool{

    if !SEE_STDOUT { output.stdout(std::process::Stdio::null()); }
    if !SEE_STDERR { output.stderr(std::process::Stdio::null()); }

    let mut spawned = output.spawn().expect("Unable to output command");
    let wait = spawned.wait().expect("Failed to wait for output to end");
    return wait.success();
}

// Builds AUR packages and installs them
fn build_aur(name : &str) {
    println!("Building (AUR) {}...", name);

    let mut output = Command::new("makepkg");
    output.arg("-si");
    if ASSUME_YES { output.arg("--noconfirm"); }

    let success = send_output(output);
    if success {
        println!("Installed (AUR) {}...", name);
    }
}

// Gets the current directory the program is in
fn get_current_directory() -> String {
    let current_dir = Command::new("pwd").output().expect("Couldn't get current directory");
    let mut og_directory = String::from_utf8(current_dir.stdout).unwrap();
    og_directory.truncate(og_directory.len() - 1);
    return  og_directory;
}

// Main Function
fn main() -> Result<(), mlua::Error> {
    let lua = Lua::new();

    // Read the Lua file, cargo run should be run from /src
    let lua_script: String = fs::read_to_string(env::current_dir()
    .expect("Unable to get current directory").to_str()
    .expect("Unable to convert current directory to str").to_string() + "/config.lua")?;

    // Load the Lua script
    let globals = lua.globals();
    lua.load(&lua_script).exec()?;

    // Upgrade System
    let mut output = Command::new("sudo");
    output.arg("pacman");
    output.arg("-Syu");
    send_output(output);

    // Get currently installed packages -- this one needs to use .output to get the stdout.
    let output = Command::new("pacman")
    .arg("-Qeq")
    .output()
    .expect("Failed to execute command");

    if !output.status.success() {
        println!("Unable to get list of installed packages, exiting.");
        exit(1);
    }

    // Get readable system packages
    let raw_packages: String = String::from_utf8(output.stdout).unwrap();
    let mut packages : Vec<&str> = raw_packages.lines().collect();

    // Gets tables from the lua script
    let packages_table: mlua::Table = globals.get("Packages")?;
    let official_table: mlua::Table = packages_table.get("Official")?;
    let aur_table: mlua::Table = packages_table.get("Aur")?;
    let flatpak_table: mlua::Table = packages_table.get("Flatpak")?;

    // INSTALLING PACKAGES //

    // Installing official packages
    for pair in official_table.pairs::<mlua::Value, mlua::Value>() {
        let (_key, value) = pair?;
        match value {

            mlua::Value::String(string) => {
                let string_str = string.to_str().unwrap();
                if packages.contains(&string_str) {
                    let index = packages.iter().position(|&r| r == string_str);
                    packages.remove(index.unwrap());
                    // println!("Already Installed {}...", string_str);
                } else {
                    println!("Attempting to install {}...", string_str);

                    let mut output = Command::new("sudo");
                    output.arg("pacman");
                    output.arg("-S");
                    output.arg(string_str);
                    if ASSUME_YES { output.arg("--noconfirm"); }

                    let success = send_output(output);
                    if success {
                        println!("Installed {}...", string_str);
                    }
                }
            },

            _ => (),

        }
    }

    // Installing AUR packages
    let mut global_install_location : String = String::new();
    for pair in aur_table.pairs::<mlua::Value, mlua::Value>() {
        let (_key, val) = pair?;
        match val {

            mlua::Value::Table(table) => {
                for pair in table.pairs::<mlua::Value, mlua::Value>() {
                    let (index, value) = pair?; // index gives the var name, value gives the val
                    let index = index.as_string().unwrap();
                    let value = value.as_string().unwrap();
                    
                    if index == "GlobalInstallLocation" {
                        global_install_location = value.to_str()?.to_string();
                    }
                }
            },

            mlua::Value::String(string) => {

                let string_str = string.to_str().unwrap();

                if packages.contains(&string_str) {

                    // Package is already installed - check for updates
                    let index = packages.iter().position(|&r| r == string_str);
                    let directory = global_install_location.clone() + "/" + string_str; // Can lead to double slash instances but doesn't seem to do anything
                    
                    // Incase the install directory has changed or the folder was manually deleted
                    if !std::path::Path::new(&directory).exists() {
                        std::fs::create_dir(&directory)?;
                    }

                    let og_directory = get_current_directory();
                    env::set_current_dir(directory)?;

                    let output = Command::new("git")
                    .arg("pull")
                    .output()
                    .expect("Failed to execute command");

                    if output.status.success() {
                        println!("Pulled (AUR) {}...", string_str);
                    } else {
                        println!("{:?}", String::from_utf8_lossy(&output.stderr));
                    }
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                    
                    // Checking if already updated, if not, then build and continue
                    if String::from_utf8_lossy(&output.stdout) != "Already up to date.\n" {
                        build_aur(string_str);
                    }

                    env::set_current_dir(og_directory)?;
                    packages.remove(index.unwrap());
                    
                } else {
                    // Package isn't installed, need to set it up and install it
                    if global_install_location.is_empty() {
                        println!("Unable to install (AUR) {} as the install location was not specified. (Try specifying GlobalInstallLocation?)", string_str);
                        break;
                    }

                    println!("Attempting to install (AUR) {}...", string_str);
                    let directory = global_install_location.clone() + "/" + string_str; // Can lead to double slash instances but doesn't seem to do anything
                    println!("Creating Directory at: {:?}", directory);
                    fs::create_dir_all::<&str>(directory.as_ref())?;

                    let output = Command::new("git")
                    .arg("clone")
                    .arg("https://aur.archlinux.org/".to_owned() + string_str + ".git")
                    .arg::<&str>(directory.as_ref())
                    .output()
                    .expect("Failed to execute command");
                
                    if output.status.success() {
                        println!("Cloned (AUR) {}...", string_str);
                    } else {
                        println!("{:?}", String::from_utf8_lossy(&output.stderr));
                    }

                    let og_directory = get_current_directory();
                    env::set_current_dir(directory)?;
                    build_aur(string_str);
                    env::set_current_dir(og_directory)?;
                }
            },

            _ => (),
        }
    }

    // Installing flatpak packages
    let flatpak_packages = Command::new("flatpak")
    .arg("list")
    .arg("--app")
    .arg("--columns=application")
    .output()
    .expect("Failed to execute command");

    let flatpak_packages: String = String::from_utf8(flatpak_packages.stdout).unwrap();
    let mut flatpak_packages : Vec<&str> = flatpak_packages.lines().collect();
    flatpak_packages.remove(0); // Remove the first value as it's the header "APPLICATION ID"

    for pair in flatpak_table.pairs::<mlua::Value, mlua::Value>() {
        let (_key, value) = pair?;
        match value {

            mlua::Value::String(string) => {

                let string_str = string.to_str().unwrap();
                if flatpak_packages.contains(&string_str) {
                    let index = flatpak_packages.iter().position(|&r| r == string_str);
                    flatpak_packages.remove(index.unwrap());
                    println!("Already Installed {}...", string_str);
                } else {
                    println!("Attempting to install {}...", string_str);

                    let mut output = Command::new("flatpak");
                    output.arg("install");
                    output.arg(string_str);
                    if ASSUME_YES { output.arg("--assumeyes"); }

                    let success = send_output(output);
                    if success {
                        println!("Installed {}...", string_str);
                    }
                }
            },

            _ => (),

        }
    }

    // REMOVING PACKAGES //

    if packages.len() > 0 {
        let mut output = Command::new("sudo");
        output.arg("pacman");
        output.arg("-Rns");
        if ASSUME_YES { output.arg("--noconfirm"); }
    
        let mut dep = Command::new("sudo");
        dep.arg("pacman");
        dep.arg("--asdep");
        dep.arg("-D");
    
        for value in &packages {
            output.arg(value);
            dep.arg(value);
        }
    
        let success : bool = send_output(output);
        if success {
            println!("Removed {:?}...", packages);
        } else {
            let _success : bool = send_output(dep);
        }
    
        Command::new("pacman").arg("-Syu").output().expect("Failed to update entire system...");
    }

    if flatpak_packages.len() > 0 {
        let mut output = Command::new("flatpak");
        output.arg("uninstall");
        if ASSUME_YES { output.arg("--assumeyes"); }

        for value in &flatpak_packages {
            output.arg(value);
        }

        let success = send_output(output);
        if success {
            println!("Removed {:?}...", flatpak_packages);
        }
    
        let mut output = Command::new("flatpak");
        output.arg("uninstall");
        output.arg("--unused");
        if ASSUME_YES { output.arg("--assumeyes"); }

        let _success = send_output(output);
    }

    println!("Finished...");
    Ok(())
}
