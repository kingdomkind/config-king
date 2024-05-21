use mlua::prelude::*;
use std::{env, fs, io, process::{exit, Command}};
use colour::*;

/*
BIG TODOS:
    => Symlinks
*/

/*
COLOUR CODING:

Green - Action is successful
Grey - Action is already done
Default - Output from the bash commands
Yellow - Warning (ie. can be recovered from or only prevents one specific thing)
Red - Critical Error that prevents that overall stage from working properly, needs immediate attention
Bold White - Expected output that will always occur / Part of an action that is not yet finished
Cyan - New section
*/

// Config Variables
const SEE_STDOUT : bool = false;
const SEE_STDERR : bool = false;
const ASSUME_YES : bool = true;
const PACKAGE_REMOVE_WARN_LIMIT : u32 = 5;
const DEFAULT_YES : bool = true;

fn get_confirmation() -> bool {
    let mut accepted_response = false;
    let mut choice : bool = false;

    while !accepted_response {
        let mut response = String::new();
        accepted_response = true;

        io::stdin().read_line(&mut response).expect("failed to readline");

        match response.trim().to_lowercase().as_str() {
            "yes" | "y" | "ye" => choice = true,
            "no" | "n" | "nah" => choice = false,
            "" => { if DEFAULT_YES { choice = true; } else { choice = false; } },
            _ => accepted_response = false,
        }
    }

    return choice;
}

fn subtract_lua_vec(rust_table : Vec<String>, lua_table : mlua::Table) -> Vec<String> {

    let mut rust_table = rust_table;
    for pair in lua_table.pairs::<mlua::Value, mlua::Value>() {
        let Ok((_key, value)) = pair else { panic!() };
        match value {

            mlua::Value::String(string) => {
                let string = string.to_str().unwrap().to_string();
                if rust_table.contains(&string) {
                    let index = rust_table.iter().position(|r| *r == string);
                    rust_table.remove(index.unwrap());
                }
            },
            _ => (),

        }
    };

    return rust_table;
}

/*
fn subtract_rust_vec(rust_table : Vec<String>, subtract_table : Vec<String>) -> Vec<String> {
    let mut rust_table = rust_table;
    for value in subtract_table.iter() {
        if rust_table.contains(&value) {
            let index = rust_table.iter().position(|r| r == value);
            rust_table.remove(index.unwrap());
        }
    };

    return rust_table;
} */

// Runs Commands, and displays the output and returns if successful
fn send_output(mut output : Command) -> bool {

    if !SEE_STDOUT { output.stdout(std::process::Stdio::null()); }
    if !SEE_STDERR { output.stderr(std::process::Stdio::null()); }

    let mut spawned = output.spawn().expect("Unable to output command");
    let wait = spawned.wait().expect("Failed to wait for output to end");
    return wait.success();
}

// Builds AUR packages and installs them
fn build_aur(name : &str) {
    white_ln_bold!("Building (AUR) {}...", name);

    let mut output = Command::new("makepkg");
    output.arg("-si");
    if ASSUME_YES { output.arg("--noconfirm"); }

    let success = send_output(output);
    if success {
        green_ln!("Installed (AUR) {}...", name);
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

    // flatpak packages
    let flatpak_packages = Command::new("flatpak")
    .arg("list")
    .arg("--app")
    .arg("--columns=application")
    .output()
    .expect("Failed to execute command");
    
    let flatpak_packages: String = String::from_utf8(flatpak_packages.stdout).unwrap();
    let mut flatpak_packages : Vec<&str> = flatpak_packages.lines().collect();
    flatpak_packages.remove(0); // Remove the first value as it's the header "APPLICATION ID"


    // REMOVING PACKAGES //

    // Getting packages to remove
    let mut packages_to_remove = subtract_lua_vec(packages.iter().map(|x| x.to_string()).collect(), official_table.clone());
    packages_to_remove = subtract_lua_vec(packages_to_remove, aur_table.clone());
    let flapak_packages_to_remove: Vec<String> = subtract_lua_vec(flatpak_packages.iter().map(|x| x.to_string()).collect(), flatpak_table.clone());

    cyan_ln!("Beginning to remove packages...");

    // Checking if we should actually remove the packages, if above the regular warn limit
    let mut should_remove_package : bool = true;
    if (packages_to_remove.len() + flapak_packages_to_remove.len()) > PACKAGE_REMOVE_WARN_LIMIT.try_into().unwrap() {
        yellow_ln!("Packages to remove is above the warning limit of {} and are:", PACKAGE_REMOVE_WARN_LIMIT);

        for value in &packages_to_remove {
            yellow_ln!("{}", value);
        }

        yellow_ln!("Are you sure you want to remove the specified packages? [y/n]");
        should_remove_package = get_confirmation();
    }

    if should_remove_package {
        // Removing regular packages
        if packages_to_remove.len() > 0 {
            let mut output = Command::new("sudo");
            output.arg("pacman");
            output.arg("-Rns");
            if ASSUME_YES { output.arg("--noconfirm"); }
        
            let mut dep = Command::new("sudo");
            dep.arg("pacman");
            dep.arg("--asdep");
            dep.arg("-D");
        
            for value in &packages_to_remove {
                output.arg(value);
                dep.arg(value);
            }
        
            let success : bool = send_output(output);
            if success {
                green_ln!("Removed {:?}...", packages_to_remove);
            } else {
                let _success : bool = send_output(dep);
            }    
        }

        // Removing flatpack packages
        if flapak_packages_to_remove.len() > 0 {
            let mut output = Command::new("flatpak");
            output.arg("uninstall");
            if ASSUME_YES { output.arg("--assumeyes"); }

            for value in &flapak_packages_to_remove {
                output.arg(value);
            }

            let success = send_output(output);
            if success {
                green_ln!("Removed {:?}...", flapak_packages_to_remove);
            }
        
            let mut output = Command::new("flatpak");
            output.arg("uninstall");
            output.arg("--unused");
            if ASSUME_YES { output.arg("--assumeyes"); }

            let _success = send_output(output);
        }

        white_ln_bold!("Finished removing packages...");
    } else {
        grey_ln!("Skipping removing packages...");
    }

    // INSTALLING PACKAGES //

    cyan_ln!("Installing Packages...");
    white_ln_bold!("Upgrading System...");
    // Upgrade System
    let mut output = Command::new("sudo");
    output.arg("pacman");
    output.arg("-Syu");
    send_output(output);

    // Installing official packages
    for pair in official_table.pairs::<mlua::Value, mlua::Value>() {
        let (_key, value) = pair?;
        match value {

            mlua::Value::String(string) => {
                let string_str = string.to_str().unwrap();
                if packages.contains(&string_str) {
                    let index = packages.iter().position(|&r| r == string_str);
                    packages.remove(index.unwrap());
                } else {
                    white_ln_bold!("Attempting to install {}...", string_str);

                    // First we need to check if the package is in a group
                    // Ideally, we would allow group installations but it presents the issue of the config
                    // not lining up with installed packages, and without the ability to tell if a package
                    // was installed via a group, we cannot remedy this
                    let mut is_group : bool = false;
                    let output = Command::new("pacman")
                    .arg("-Sg")
                    .arg(string_str)
                    .output()
                    .expect("Failed to execute command");

                    let raw_out: String = String::from_utf8(output.stdout).unwrap();
                    let out : Vec<&str> = raw_out.lines().collect();

                    if out.len() != 0 {
                        is_group = true;
                    }

                    if is_group {
                        yellow_ln!("SKIPPING: The specified package of \"{}\" is a package group, which is not supported...", string_str);
                        yellow_ln!("Please instead install the packages specified by the group. See specified packages? [y/n]");
                        let see_packages = get_confirmation();
                        if see_packages {
                            for value in out {
                                yellow_ln!("{}", value);
                            }
                        }
                        break;
                    }

                    let mut output = Command::new("sudo");
                    output.arg("pacman");
                    output.arg("-S");
                    output.arg(string_str);
                    if ASSUME_YES { output.arg("--noconfirm"); }

                    let success = send_output(output);
                    if success {
                        green_ln!("Installed {}...", string_str);
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
                        white_ln_bold!("Pulled (AUR) {}...", string_str);
                    } else {
                        red_ln!("{:?}", String::from_utf8_lossy(&output.stderr));
                    }
                    
                    // Checking if already updated, if not, then build and continue
                    if String::from_utf8_lossy(&output.stdout) != "Already up to date.\n" {
                        build_aur(string_str);
                    } else {
                        grey_ln!("(AUR) {} is already up to date...", string_str);
                    }

                    env::set_current_dir(og_directory)?;
                    packages.remove(index.unwrap());
                    
                } else {
                    // Package isn't installed, need to set it up and install it
                    if global_install_location.is_empty() {
                        yellow_ln!("Unable to install (AUR) {} as the install location was not specified. (Try specifying GlobalInstallLocation?)", string_str);
                        break;
                    }

                    white_ln_bold!("Attempting to install (AUR) {}...", string_str);
                    let directory = global_install_location.clone() + "/" + string_str; // Can lead to double slash instances but doesn't seem to do anything
                    white_ln_bold!("Creating Directory at: {:?}", directory);
                    fs::create_dir_all::<&str>(directory.as_ref())?;

                    let output = Command::new("git")
                    .arg("clone")
                    .arg("https://aur.archlinux.org/".to_owned() + string_str + ".git")
                    .arg::<&str>(directory.as_ref())
                    .output()
                    .expect("Failed to execute command");
                
                    if output.status.success() {
                        white_ln_bold!("Cloned (AUR) {}...", string_str);
                    } else {
                        red_ln!("{:?}", String::from_utf8_lossy(&output.stderr));
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
    for pair in flatpak_table.pairs::<mlua::Value, mlua::Value>() {
        let (_key, value) = pair?;
        match value {

            mlua::Value::String(string) => {

                let string_str = string.to_str().unwrap();
                if flatpak_packages.contains(&string_str) {
                    let index = flatpak_packages.iter().position(|&r| r == string_str);
                    flatpak_packages.remove(index.unwrap());
                    grey_ln!("Already Installed {}...", string_str);
                } else {
                    white_ln_bold!("Attempting to install {}...", string_str);

                    let mut output = Command::new("flatpak");
                    output.arg("install");
                    output.arg(string_str);
                    if ASSUME_YES { output.arg("--assumeyes"); }

                    let success = send_output(output);
                    if success {
                        green_ln!("Installed {}...", string_str);
                    }
                }
            },

            _ => (),

        }
    }
    white_ln_bold!("Finished installing packages...");

    cyan_ln!("Finished (Completed all tasks)...");
    Ok(())
}
