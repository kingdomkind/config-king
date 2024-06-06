use aur::{make_and_install_package, pull_package};
use mlua::prelude::*;
use save::overwrite_file;
use std::{collections::HashMap, env, fs, path::Path, process::Command, time::Instant};
use colour::*;

mod globals;
mod utilities;
mod official;
mod aur;
mod flatpak;
mod symlinks;
mod save;

use globals::*;

/*
BIG TODOS:
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

// Main Function
fn main() -> Result<(), mlua::Error> {

    // Begin time tracking
    let time = Instant::now();

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
    if !utilities::is_system_package_installed("flatpak") {
        yellow!("Warning: ");
        white_ln!("Flatpak dependency not installed, installing now");
        official::install_packages(vec![String::from("flatpak")]);
    }

    if !utilities::is_system_package_installed("git") {
        yellow!("Warning: ");
        white_ln!("Git dependency not installed, installing now");
        official::install_packages(vec![String::from("flatpak")]);
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

    // FORMING PACKAGE VARIABLES

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
    let mut flatpak_packages = utilities::vec_str_to_string(flatpak_packages.lines().collect());

    if flatpak_packages.contains(&"Application ID".to_string()) {
        flatpak_packages.remove(0); // Remove the first value as it's the header "Application ID"
    }

    // REMOVING PACKAGES //
    let mut system_packages: Vec<String> = utilities::get_installed_system_packages();

    cyan!("Starting: ");
    white_ln!("Removing packages");

    // Getting packages to remove
    let mut packages_to_remove = utilities::subtract_lua_vec(system_packages.clone(), official_table.clone());
    packages_to_remove = utilities::subtract_lua_vec(packages_to_remove, aur_table.clone());
    let flatpak_packages_to_remove: Vec<String> = utilities::subtract_lua_vec(flatpak_packages.clone(), flatpak_table.clone());

    // Checking if we should actually remove the packages, if above the regular warn limit
    let mut should_remove_package : bool = true;
    if (packages_to_remove.len() + flatpak_packages_to_remove.len()) > PACKAGE_REMOVE_WARN_LIMIT.try_into().unwrap() {
        yellow_ln!("Packages to remove is above the warning limit of {} and are:", PACKAGE_REMOVE_WARN_LIMIT);

        for value in &packages_to_remove {
            yellow_ln!("{}", value);
        }

        yellow_ln!("Are you sure you want to remove the specified packages? [y/n]");
        should_remove_package = utilities::get_confirmation();
    }

    if should_remove_package {

        // Cleaning up old AUR directories
        aur::remove_uninstalled_aur_directories(aur_table.clone(), install_locations["Aur"].clone());
        // Removing system (official + AUR) packages
        official::remove_system_packages(packages_to_remove);
        // Removing flatpack packages
        flatpak::remove_packages(flatpak_packages_to_remove);

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
    official::upgrade_all_packages();

    green!("Installed: ");
    white_ln!("Upgraded System");

    // Installing official packages
    for pair in official_table.pairs::<mlua::Value, mlua::Value>() {
        let (_key, value) = pair?;
        match value {

            mlua::Value::String(string) => {
                let string_str = string.to_str().unwrap();
                if system_packages.contains(&string_str.to_string()) {
                    let index = system_packages.iter().position(|r| r == string_str);
                    system_packages.remove(index.unwrap());
                } else {
                    white_ln!("Attempting to install {}", string_str);

                    // Deny if package is in a group, as we cannot track packages installed from groups!
                    let is_group = official::is_package_group(string_str.to_string());

                    if is_group {
                        yellow_ln!("SKIPPING: The specified package of \"{}\" is a package group, which is not supported", string_str);
                        yellow_ln!("Please instead install the packages specified by the group. See specified packages? [y/n]");
                        let see_packages = utilities::get_confirmation();
                        if see_packages {
                            let packages_in_group = official::get_packages_in_group(string_str.to_string());
                            for value in packages_in_group {
                                yellow_ln!("{}", value);
                            }
                        }
                        continue;
                    }

                    official::install_packages(vec![string_str.to_string()]);
                }
            },

            _ => (),

        }
    }

    // Installing AUR packages
    for pair in aur_table.pairs::<mlua::Value, mlua::Value>() {
        let (_key, val) = pair?;

        let mut packages: Vec<String> = Vec::new();

        if val.is_string() {
            packages.push(val.to_string().unwrap());
        }

        if val.is_table() {
            let val = val.as_table().unwrap().clone().pairs::<mlua::Value, mlua::Value>();

            for secondary_pair in val {
                let (_secondary_key, secondary_val) = secondary_pair?;
                packages.push(secondary_val.to_string().unwrap());
            }
        }

        let mut install_required = true;
        let mut all_pkgs_installed = true;

        for package in &packages {
            if !system_packages.contains(&package) {
                all_pkgs_installed = false;
            }
        }

        if all_pkgs_installed {

            // Package is already installed - check for updates
            let index = system_packages.iter().position(|r| *r == packages[0]);
            let directory = install_locations["Aur"].clone() + "/" + &packages[0]; // Can lead to double slash instances but doesn't seem to do anything
                    
            // Incase the install directory has changed or the folder was manually deleted
            if !std::path::Path::new(&directory).exists() {
                std::fs::create_dir(&directory)?;
            } else {
                install_required = false;
                let needs_update = pull_package(install_locations["Aur"].clone(), packages[0].clone());
                if needs_update { make_and_install_package(install_locations["Aur"].clone(), packages.clone()) }
                system_packages.remove(index.unwrap());
            }
        }

        if install_required == true {
            // Package isn't installed, need to set it up and install it
            if !install_locations.contains_key("Aur") {
                yellow_ln!("(AUR) Unable to install {} as the install location was not specified.", packages[0]);
                continue;
            }

            white_ln!("(AUR) Attempting to install {}", packages[0]);
            aur::clone_package(install_locations["Aur"].clone(), packages[0].clone());
            aur::make_and_install_package(install_locations["Aur"].clone(), packages.clone());
        }
    }

    // Installing flatpak packages
    for pair in flatpak_table.pairs::<mlua::Value, mlua::Value>() {
        let (_key, value) = pair?;
        match value {

            mlua::Value::String(string) => {

                let string_str = string.to_str().unwrap();
                if flatpak_packages.contains(&string_str.to_string()) {
                    let index = flatpak_packages.iter().position(|r| r == string_str);
                    flatpak_packages.remove(index.unwrap());
                } else {
                    flatpak::install_packages(vec![string_str.to_string()]);
                }
            },

            _ => (),

        }
    }

    // UPDATE ALL FLATPAKS NOW!

    magenta!("Finished: ");
    white_ln!("Installed all intended packages");

    // Read cached save file
    cyan!("Starting: ");
    white_ln!("Reading previous save file");

    let save_exist = Path::new(&install_locations["Save"]).exists();

    // Extracted Content
    let mut current_symlinks: Vec<String> = Vec::new();

    if save_exist {
        let elements = save::read_file_elements(install_locations["Save"].clone());

        for value in elements {
            let identifier_bound = value.find('=').unwrap();
            let identifier = &value[..identifier_bound];

            // Match Names to their respective implementations
            match identifier {

                // Note that symlinks identifier can only appear once as it overwrites the previous vector
                "symlinks" => {
                    current_symlinks = symlinks::read_save(identifier_bound, value);
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

        save::create_file_location(install_locations["Save"].clone());
    }

    magenta!("Finished: ");
    white_ln!("Read save file");

    // Creating Symlinks
    cyan!("Starting: ");
    white_ln!("Regenerating Symlinks");

    let symlinks_table: mlua::Table = globals.get("Symlinks")?;
    // Get Current symlinks as rust hash map
    let new_symlinks: HashMap<String, String> = utilities::convert_lua_dictionary_to_hashmap_string(symlinks_table.clone());

    // Deleting previous symlinks
    symlinks::delete_old_symlinks(current_symlinks, new_symlinks);

    // Creating new symlinks
    let symlink_msg = symlinks::generate_symlinks(symlinks_table);

    magenta!("Finished: ");
    white_ln!("Managed all Symlinks");

    // Write updated information to the save file
    cyan!("Starting: ");
    white_ln!("Updating Save File");

    overwrite_file(install_locations["Save"].clone(), symlink_msg);

    magenta!("Finished: ");
    white_ln!("Updated Save File");

    // Everything done, we can exit
    let elapsed_time: f32 = (time.elapsed().as_millis() as f32) / 1000.0;
    magenta!("Finished: ");
    white_ln!("Completed all tasks in {}s", format!("{:.2}",elapsed_time));
    Ok(())
}
