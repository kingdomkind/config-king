use mlua::prelude::*;
use std::{collections::HashMap, env, fs::{self, File, OpenOptions}, io::{self, Read, Write},path::Path, process::{exit, Command}};
use colour::*;

/*
BIG TODOS:
    => Verify Symlinks are stable and reliable, or patch them if needed
    => Before trying to install a package, check if it is already installed in the system, not just through explicitly installed means. It could be dragged in as a
    dependency then someone could explicitly want to intsall it and it is still marked as a dep
    => Test if packages actually need to be set as a dep or not if removal fails
    => Check if install locations exist at the start of the script. If not, ask the user if they want them to be created
    => Fix AUR installs, use AUR api to grab correct git clone repo. Need to check pkgname for official and AUR packages to see what they install. They will install all the pkgnames.
    => Refactor code base into its own seperate sections
*/

/*
COLOUR CODING:

Green - Action is successful
Grey - Action is already done
Default - Output from the bash commands
Yellow - Warning (ie. can be recovered from or only prevents one specific thing)
Red - Critical Error that prevents that overall stage from working properly, needs immediate attention
White - Expected output that will always occur / Part of an action that is not yet finished
Cyan - New section
Magenta - Finished section
*/

// Config Variables
const SEE_STDOUT : bool = true;
const SEE_STDERR : bool = true;
const ASSUME_YES : bool = true;
const PACKAGE_REMOVE_WARN_LIMIT : u32 = 5;
const DEFAULT_YES : bool = true;

fn quick_install(package_name: String) {
    let mut output = Command::new("sudo");
    output.arg("pacman");
    output.arg("-S");
    output.arg(package_name);
    if ASSUME_YES { output.arg("--noconfirm"); }
    send_output(output);
}

fn is_package_installed(package_name: String) -> bool {
    let output = Command::new("which")
    .arg(package_name)
    .output()
    .expect("Failed to execute command");

    return output.status.success();
}

fn remove_path(path : String) {
    println!("Entered path remover");
    if Path::new(&path).exists() {
        let mut ret: Option<bool> = None;
        if Path::new(&path).is_dir() {
            yellow!("Warning: ");
            white_ln!("Are you sure you would like to remove the directory at {} [y/n]", path);
            let confirm = get_confirmation();
            if confirm {
                let mut output = Command::new("sudo");
                output.arg("rm");
                output.arg("-r");
                output.arg(&path);
                ret = Some(send_output(output));

            }
        } else {
            yellow!("Warning: ");
            white_ln!("Are you sure you would like to remove the file at {} [y/n]", path);
            let confirm = get_confirmation();
            if confirm {
                let mut output = Command::new("sudo");
                output.arg("rm");
                output.arg(&path);
                ret = Some(send_output(output));
            }
        };

        if !ret.is_none() {
            match ret.unwrap() {
                false => {
                    red!("ERROR: ");
                    white_ln!("Failed to delete path at {}", path);
                },
                true => {
                    green!("Removed: ");
                    white_ln!("Removed path at {}", path);
                }
            }
        } else {
            white_ln!("Skipped removing path");
        }
    }
}

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
    white_ln!("(AUR) Building {}", name);

    let mut output = Command::new("makepkg");
    output.arg("-si");
    if ASSUME_YES { output.arg("--noconfirm"); }

    let success = send_output(output);
    if success {
        green!("Installed: ");
        white_ln!("(AUR) {}", name);
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

    let mut arguments: Vec<String> = env::args().collect();
    arguments.remove(0);

    // Read the Lua file, cargo run should be run from the directory of the config file if no directory is specified
    let mut lua_script: String = String::new();

    for arg in arguments {
        let pos = arg.find('=');

        if !pos.is_none() {
            let key = &arg[..pos.unwrap()];
            let value = &arg[pos.unwrap() + 1..];

            match key {
                "directory" => {
                    lua_script = fs::read_to_string(value.to_string())?;
                },
                _ => { yellow!("Warning: "); white_ln!("{} is not a recognised key", key); }
            }
        } else {
            yellow!("Warning: ");
            white_ln!("No '=' found in the argument {}", arg)
        }
    }

    if lua_script.is_empty() {
        lua_script = fs::read_to_string(env::current_dir()
        .expect("Unable to get current directory").to_str()
        .expect("Unable to convert current directory to str").to_string() + "/config.lua")?;
    }

    // Ensure dependencies are installed!
    if !is_package_installed("flatpak".to_string()) {
        yellow!("Warning: ");
        white_ln!("Flatpak dependency not installed, installing now");
        quick_install("flatpak".to_string());
    }

    if !is_package_installed("git".to_string()) {
        yellow!("Warning: ");
        white_ln!("Git dependency not installed, installing now");
        quick_install("git".to_string());
    }

    let lua = Lua::new();

    // Load the Lua script
    let globals = lua.globals();
    lua.load(&lua_script).exec()?;

    // READING INSTALL LOCATIONS
    let mut install_locations : HashMap<String, String> = HashMap::new();
    let install_table: mlua::Table = globals.get("InstallLocations")?;

    for pair in install_table.pairs::<mlua::Value, mlua::Value>() {
        let (key, value) = pair?;
        match &value {

            mlua::Value::String(_string) => {
                install_locations.insert(key.to_string().unwrap(), value.to_string().unwrap());
            },

            _ => {
                red!("ERROR: ");
                white_ln!("Unerecognised value in install locations table, key {:?}, value {:?}", key, value);
            },

        }
    }

    // PACKAGES START
    cyan!("Starting: ");
    white_ln!("Removing packages");

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

    // SOMEHOW SET FLATPAK AS A REQUIRED DEPENDENCY, ALSO APPLICATION ID COLUMN DOESN'T SHOW FOR SOME REASON UNLESS YOU HAVE SOMETHING INSTALLED

    // flatpak packages
    let flatpak_packages = Command::new("flatpak")
    .arg("list")
    .arg("--app")
    .arg("--columns=application")
    .output()
    .expect("Failed to execute command");
    
    let flatpak_packages: String = String::from_utf8(flatpak_packages.stdout).unwrap();
    let mut flatpak_packages : Vec<&str> = flatpak_packages.lines().collect();

    if flatpak_packages.contains(&"Application ID") {
        flatpak_packages.remove(0); // Remove the first value as it's the header "Application ID"
    }

    // REMOVING PACKAGES //

    // Getting packages to remove
    let mut packages_to_remove = subtract_lua_vec(packages.iter().map(|x| x.to_string()).collect(), official_table.clone());
    packages_to_remove = subtract_lua_vec(packages_to_remove, aur_table.clone());
    let flapak_packages_to_remove: Vec<String> = subtract_lua_vec(flatpak_packages.iter().map(|x| x.to_string()).collect(), flatpak_table.clone());

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

    // TODO CHECK IF DIRECTORY EXISTS - ON FIRST RUN IT DOESNT!

    // Clean up AUR directory
    let entries = fs::read_dir(&install_locations["Aur"])?;
    let mut entry_names = Vec::new();
    for entry in entries {
        let file_name = entry?.file_name().into_string().unwrap();
        entry_names.push(file_name);
    }
    let aur_packages_to_remove = subtract_lua_vec(entry_names, aur_table.clone());

    for entry in aur_packages_to_remove {
        remove_path(install_locations["Aur"].clone() + &entry)
    }
    

    if should_remove_package {
        // Removing regular packages
        if packages_to_remove.len() > 0 {
            let mut output = Command::new("sudo");
            output.arg("pacman");
            output.arg("-Rns");
            if ASSUME_YES { output.arg("--noconfirm"); }
        
            /*
            let mut dep = Command::new("sudo");
            dep.arg("pacman");
            dep.arg("--asdep");
            dep.arg("-D");
            */

            for value in &packages_to_remove {
                output.arg(value);
                //dep.arg(value);
            }
        
            let success : bool = send_output(output);
            if success {
                green!("Removed: ");
                white_ln!("{:?}", packages_to_remove);
            } else {
                //let _success : bool = send_output(dep);
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
                white_ln!("{:?}", flapak_packages_to_remove);
            }
        
            let mut output = Command::new("flatpak");
            output.arg("uninstall");
            output.arg("--unused");
            if ASSUME_YES { output.arg("--assumeyes"); }

            let _success = send_output(output);
        }

        magenta!("Finished: ");
        white_ln!("Removed all intended packages");
    } else {
        grey_ln!("Skipping removing packages");
    }

    // INSTALLING PACKAGES //

    cyan!("Starting: ");
    white_ln!("Installing Packages");
    white_ln!("Upgrading System");
    
    // Upgrade System
    let mut output = Command::new("sudo");
    output.arg("pacman");
    output.arg("-Syu");
    if ASSUME_YES { output.arg("--noconfirm"); }
    send_output(output);

    green!("Installed: ");
    white_ln!("Upgraded System");

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
                    white_ln!("Attempting to install {}", string_str);

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
                        continue;
                    }

                    let mut output = Command::new("sudo");
                    output.arg("pacman");
                    output.arg("-S");
                    output.arg(string_str);
                    if ASSUME_YES { output.arg("--noconfirm"); }

                    let success = send_output(output);
                    if success {
                        green!("Installed: ");
                        white_ln!("{}", string_str);
                    }
                }
            },

            _ => (),

        }
    }

    // Installing AUR packages
    for pair in aur_table.pairs::<mlua::Value, mlua::Value>() {
        let (_key, val) = pair?;
        match val {

            mlua::Value::String(string) => {

                let string_str = string.to_str().unwrap();

                if packages.contains(&string_str) {

                    // Package is already installed - check for updates
                    let index = packages.iter().position(|&r| r == string_str);
                    let directory = install_locations["Aur"].clone() + "/" + string_str; // Can lead to double slash instances but doesn't seem to do anything
                    
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
                        // white_ln!("Pulled (AUR) {}", string_str); redundant
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
                    if !install_locations.contains_key("Aur") {
                        yellow_ln!("(AUR) Unable to install {} as the install location was not specified.", string_str);
                        continue;
                    }

                    white_ln!("(AUR) Attempting to install {}", string_str);
                    let directory = install_locations["Aur"].clone() + "/" + string_str; // Can lead to double slash instances but doesn't seem to do anything
                    white_ln!("Creating Directory at: {:?}", directory);
                    fs::create_dir_all::<&str>(directory.as_ref())?;

                    let output = Command::new("git")
                    .arg("clone")
                    .arg("https://aur.archlinux.org/".to_owned() + string_str + ".git")
                    .arg::<&str>(directory.as_ref())
                    .output()
                    .expect("Failed to execute command");
                
                    if output.status.success() {
                        white_ln!("(AUR) Cloned {}", string_str);
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
                    grey_ln!("(FLATPAK) {} is already up to date", string_str);
                } else {
                    white_ln!("(FLATPAK) Attempting to install {}", string_str);

                    let mut output = Command::new("flatpak");
                    output.arg("install");
                    output.arg(string_str);
                    if ASSUME_YES { output.arg("--assumeyes"); }

                    let success = send_output(output);
                    if success {
                        green!("Installed: ");
                        white_ln!("{}", string_str);
                    }
                }
            },

            _ => (),

        }
    }
    magenta!("Finished: ");
    white_ln!("Installed all intended packages");

    // Read cached save file
    cyan!("Starting: ");
    white_ln!("Reading previous save file");

    let save_exist = Path::new(&install_locations["Save"]).exists();

    // Extracted Content
    let mut symlink_vec: Vec<String> = Vec::new();

    if save_exist {
        let mut file = OpenOptions::new()
        .read(true)
        .open(&install_locations["Save"])?;

        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        let content_str = String::from_utf8_lossy(&content);

        let elements: Vec<String> = content_str
        .split(';')
        .map(|s| s.trim_end_matches('\n').to_string())
        .filter(|s: &String| !s.is_empty()) // Filter out empty strings
        .collect();

        for value in elements {
            let identifier_bound = value.find('=').unwrap();
            let identifier = &value[..identifier_bound];

            match identifier {
                "symlinks" => {
                    let remainder = &value[identifier_bound+2..value.len()-1]; // +2 to slice of the =[ and -1 to slice the ]
                    //println!("Remainder: {}", remainder);

                    let sub_elements: Vec<String> = remainder
                    .split(',')
                    .map(|s| s.to_string())
                    .filter(|s: &String| !s.is_empty()) // Filter out empty strings

                    .collect();

                    for raw_path in sub_elements {
                        //println!("Printing Raw Path: {}",raw_path);
                        let path: &str =  &raw_path[1..raw_path.len()-1]; // remove speech marks
                        symlink_vec.push(path.to_string());
                    }
                },
                _ => {
                    red!("ERROR: ");
                    white_ln!("Identifier Name: {} was not recognised in the config.king file!", identifier);
                },
            }
        }

    } else {
        yellow!("Warning: ");
        white_ln!("No previous run save file detected, expected behaviour for first run, generating new file");
        let res = File::create(&install_locations["Save"]);

        match res {
            Err(err)=> {
                red!("ERROR: ");
                white_ln!("Failed to create save file | {}", err);
            },
            _file => {
                green!("Created: ");
                white_ln!("config.king save file");
            }
        }
    }

    magenta!("Finished: ");
    white_ln!("Read save file");

    // Creating Symlinks
    cyan!("Starting: ");
    white_ln!("Regenerating Symlinks");

    let symlinks_table: mlua::Table = globals.get("Symlinks")?;
    let mut new_symlinks: HashMap<String, String> = HashMap::new();

    // Get Current symlinks as rust hash map
    for pair in symlinks_table.clone().pairs::<mlua::Value, mlua::Value>() {
        let (key, value) = pair?;
        match &value {

            mlua::Value::String(_string) => {
                new_symlinks.insert(key.to_string().unwrap(), value.to_string().unwrap());
            },

            _ => (),

        }
    }

    /* Previous dev code
    for (index, value) in &new_symlinks {
        println!("Index: {}, Value: {}", index, value);
    }

    for value in symlink_vec.clone() {
        println!("Iter: {}", value);
    } */

    // Deleting previous symlinks
    for value in symlink_vec {
        let locations: Vec<String> = value
        .split('=')
        .map(|s| s.to_string())
        .collect();

        //println!("Location0: {}, Location1: {}", &locations[0], &locations[1]);

        // Check if the symlink already exists, is valid, and if so skip this loop
        if Path::new(&locations[0]).exists() {
            //println!("Path Exists");
            if new_symlinks.contains_key(&locations[0]) {
                //println!("We still want it (Key still exists)");
                let metadata = fs::symlink_metadata(&locations[0])?;
                if metadata.file_type().is_symlink() {
                    //println!("It's a symlink");
                    if new_symlinks[&locations[0]] == locations[1] {
                        //println!("Bingo, we keep it");
                        continue;
                    } else {
                        //println!("Get rid of this fool");
                    }
                }
            }
        }

        // Invalid symlink, banish it
        remove_path(locations[0].to_string());
    }

    //println!("Next");
    // Creating new symlinks
    let mut symlink_msg = String::from("symlinks=[");

    for pair in symlinks_table.pairs::<mlua::Value, mlua::Value>() {
        let (key, value) = pair?;
        match value {

            mlua::Value::String(string) => {

                let string_str = string.to_str().unwrap();
                let original_dir =  string_str.to_string();
                let link_dir = key.to_string().unwrap();
                let symlink_dir = link_dir.clone() + "=" + &original_dir;
                let mut already_exist = false;

                if Path::new(&link_dir).exists() {
                    let metadata = fs::symlink_metadata(&link_dir)?;
                    already_exist = metadata.file_type().is_symlink();
                }

                if !already_exist { // Only create the symlink if there's not already one there, we confirmed it was valid in the removal process

                    let mut output = Command::new("sudo");
                    output.arg("ln");
                    output.arg("-s");
                    output.arg(&original_dir);
                    output.arg(&link_dir);
                    let res = send_output(output);

                    match res {
                        false => {
                            red!("ERROR: ");
                            white_ln!("Failed to create symlink from {} to {}", link_dir, original_dir);
                            continue;
                        },
                        true => {
                            green!("Created: ");
                            white_ln!("Symlink at {} which targets {}", link_dir, original_dir);
                        }
                    }
                }

                // Update Msg
                symlink_msg.push_str("\"");
                symlink_msg.push_str(&symlink_dir);
                symlink_msg.push_str("\","); 
            },

            _ => (),

        }
    }

    // Remove the trailing comma unless the list is empty, then skip
    if symlink_msg.chars().last() != Some('[') {
        symlink_msg.pop();
    }
    symlink_msg.push_str("];");

    magenta!("Finished: ");
    white_ln!("Managed all Symlinks");

    // Write updated information to the save file
    cyan!("Starting: ");
    white_ln!("Updating Save File");

    let mut file = OpenOptions::new()
    .write(true)
    .truncate(true)
    .open("/home/pika/.config-king/save.king")?;

    //println!("New save file: {}", symlink_msg);
    let res = file.write_all(symlink_msg.as_bytes());

    match res {
        Err(err)=> {
            red!("ERROR: ");
            white_ln!("Failed to update save.king file | {}", err);
        },
        Ok(()) => {
            green!("Updated: ");
            white_ln!("save.king file with new cached information");
        }
    }

    magenta!("Finished: ");
    white_ln!("Updated Save File");

    // Everything done, we can exit
    magenta!("Finished: ");
    white_ln!("Completed all tasks");
    Ok(())
}
