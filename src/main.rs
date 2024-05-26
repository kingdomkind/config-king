use mlua::prelude::*;
use std::{env, fs::{self, remove_file, File, Metadata, OpenOptions}, io::{self, Read, Write}, ops::Index, os::unix::fs::symlink, panic::Location, path::{self, Path}, process::{exit, Command}};
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
Magenta - Finished section
*/

// Config Variables
const SEE_STDOUT : bool = false;
const SEE_STDERR : bool = true;
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
    white_ln_bold!("Building (AUR) {}", name);

    let mut output = Command::new("makepkg");
    output.arg("-si");
    if ASSUME_YES { output.arg("--noconfirm"); }

    let success = send_output(output);
    if success {
        green!("Installed: ");
        white_ln_bold!("(AUR) {}", name);
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

    cyan!("Starting: ");
    white_ln_bold!("Removing packages");

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
                green!("Removed: ");
                white_ln_bold!("{:?}", packages_to_remove);
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
                green!("Removed: ");
                white_ln_bold!("{:?}", flapak_packages_to_remove);
            }
        
            let mut output = Command::new("flatpak");
            output.arg("uninstall");
            output.arg("--unused");
            if ASSUME_YES { output.arg("--assumeyes"); }

            let _success = send_output(output);
        }

        magenta!("Finished: ");
        white_ln_bold!("Removed all intended packages");
    } else {
        grey_ln!("Skipping removing packages");
    }

    // INSTALLING PACKAGES //

    cyan!("Starting: ");
    white_ln_bold!("Installing Packages");
    white_ln_bold!("Upgrading System");
    
    // Upgrade System
    let mut output = Command::new("sudo");
    output.arg("pacman");
    output.arg("-Syu");
    if ASSUME_YES { output.arg("--noconfirm"); }
    send_output(output);

    green!("Installed: ");
    white_ln_bold!("Upgraded System");

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
                    white_ln_bold!("Attempting to install {}", string_str);

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
                        yellow_ln!("SKIPPING: The specified package of \"{}\" is a package group, which is not supported", string_str);
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
                        green!("Installed: ");
                        white_ln_bold!("{}", string_str);
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
                        white_ln_bold!("Pulled (AUR) {}", string_str);
                    } else {
                        red_ln!("{:?}", String::from_utf8_lossy(&output.stderr));
                    }
                    
                    // Checking if already updated, if not, then build and continue
                    if String::from_utf8_lossy(&output.stdout) != "Already up to date.\n" {
                        build_aur(string_str);
                    } else {
                        grey_ln!("(AUR) {} is already up to date", string_str);
                    }

                    env::set_current_dir(og_directory)?;
                    packages.remove(index.unwrap());
                    
                } else {
                    // Package isn't installed, need to set it up and install it
                    if global_install_location.is_empty() {
                        yellow_ln!("Unable to install (AUR) {} as the install location was not specified. (Try specifying GlobalInstallLocation?)", string_str);
                        break;
                    }

                    white_ln_bold!("Attempting to install (AUR) {}", string_str);
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
                        white_ln_bold!("Cloned (AUR) {}", string_str);
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
                    grey_ln!("Already Installed {}", string_str);
                } else {
                    white_ln_bold!("Attempting to install {}", string_str);

                    let mut output = Command::new("flatpak");
                    output.arg("install");
                    output.arg(string_str);
                    if ASSUME_YES { output.arg("--assumeyes"); }

                    let success = send_output(output);
                    if success {
                        green!("Installed: ");
                        white_ln_bold!("{}", string_str);
                    }
                }
            },

            _ => (),

        }
    }
    magenta!("Finished: ");
    white_ln_bold!("Installed all intended packages");

    // Read cached save file
    cyan!("Starting: ");
    white_ln_bold!("Reading previous save file");

    let save_exist = Path::new("/home/pika/.config-king/save.king").exists();
    let mut symlink_vec: Vec<String> = Vec::new();

    if save_exist {
        let mut file = OpenOptions::new()
        .read(true)
        //.write(true)
        .open("/home/pika/.config-king/save.king")?;

        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        let content_str = String::from_utf8_lossy(&content);

        let elements: Vec<String> = content_str
        .split(';')
        .map(|s| s.trim_end_matches('\n').to_string())
        .filter(|s| !s.is_empty()) // Filter out empty strings
        .collect();

        //println!("{}", elements.len());

        for value in elements {
            let identifier_bound = value.find('=').unwrap();
            let identifier = &value[..identifier_bound];
            //println!("Substring up to first '=': {}", identifier);

            match identifier {
                "symlinks" => {
                    let remainder = &value[identifier_bound+2..value.len()-1]; // +2 to slice of the =[ and -1 to slice the ]
                    println!("{}", remainder);

                    let sub_elements: Vec<String> = remainder
                    .split(',')
                    .map(|s| s.to_string())
                    .collect();

                    for raw_path in sub_elements {
                        let path =  &raw_path[1..raw_path.len()-1]; // remove speech marks
                        symlink_vec.push(path.to_string());
                    }
                },
                _ => {
                    red!("ERROR: ");
                    white_ln_bold!("Identifier Name: {} was not recognised in the config.king file!", identifier);
                },
            }
        }

    } else {
        yellow!("Warning: ");
        white_ln_bold!("No previous run save file detected, expected behaviour for first run, generating new file");
        let _ = File::create("/home/pika/.config-king/save.king");
    }

    magenta!("Finished: ");
    white_ln_bold!("Read save file");

    // Creating Symlinks
    cyan!("Starting: ");
    white_ln_bold!("Generating Symlinks");

    let symlinks_table: mlua::Table = globals.get("Symlinks")?;
    let mut symlink_msg = String::from("symlinks=[");

    for pair in symlinks_table.pairs::<mlua::Value, mlua::Value>() {
        let (key, value) = pair?;
        match value {

            mlua::Value::String(string) => {

                let string_str = string.to_str().unwrap();
                let source_dir = key.to_string().unwrap();
                let target_dir = string_str.to_string();
                let symlink_dir = source_dir.clone() + "=" + &target_dir;
                let metadata = fs::symlink_metadata(&source_dir);
                
                if metadata.unwrap().file_type().is_symlink() {
                    println!("already exits");
                } else {

                    let res = symlink(source_dir.clone(), target_dir.clone());

                    match res {
                        Err(err)=> {
                            red!("ERROR: ");
                            white_ln_bold!("Failed to create symlink from {} to {} | {}", source_dir, target_dir, err);
                        },
                        Ok(()) => {
                            green!("Created: ");
                            white_ln_bold!("Symlink at {} which targets {}", source_dir, target_dir);
                            symlink_msg.push_str("\"");
                            symlink_msg.push_str(&symlink_dir);
                            symlink_msg.push_str("\","); 
                        }
                    }
                }

                if symlink_vec.contains(&symlink_dir) {
                    let index = symlink_vec.iter().position(|r| *r == symlink_dir).unwrap();
                    symlink_vec.remove(index);
                    println!("Removed from list");
                }
            },

            _ => (),

        }
    }

    if symlink_msg.chars().last() != Some('[') {
        symlink_msg.pop(); // Remove trailing comma, if the list was not empty
    }
    symlink_msg.push_str("];");

    for value in symlink_vec {
        let locations: Vec<String> = value
        .split('=')
        .map(|s| s.to_string())
        .collect();

        if Path::new(&locations[0]).exists() {
            let res = if Path::new(&locations[0]).is_dir() {
                fs::remove_dir_all(&locations[0])
            } else {
                fs::remove_file(&locations[0])
            };

            match res {
                Err(err)=> {
                    red!("ERROR: ");
                    white_ln_bold!("Failed to delete symlink at {} | {}", locations[0], err);
                },
                Ok(()) => {
                    green!("Removed: ");
                    white_ln_bold!("Symlink at {} which targets {}", locations[0], locations[1]);
                }
            }
        }
    }

    magenta!("Finished: ");
    white_ln_bold!("Managed all Symlinks");

    cyan!("Starting: ");
    white_ln_bold!("Updating Save File");

    let mut file = OpenOptions::new()
    .write(true)
    .open("/home/pika/.config-king/save.king")?;

    println!("{}", symlink_msg);
    let res = file.write_all(symlink_msg.as_bytes());

    match res {
        Err(err)=> {
            red!("ERROR: ");
            white_ln_bold!("Failed to update save.king file | {}", err);
        },
        Ok(()) => {
            green!("Updated: ");
            white_ln_bold!("save.king file with new cached information");
        }
    }

    magenta!("Finished: ");
    white_ln_bold!("Updated Save File");

    magenta!("Finished: ");
    white_ln_bold!("Completed all tasks");
    Ok(())
}
