use mlua::prelude::*;
use std::{env, fs, process::{Command, Output}, ptr::null};

/* 
fn logger(to_print: &'static str, log_level: &'static str) {
    let array = ["juan"];
} */

fn main() -> Result<(), mlua::Error> {
    let lua = Lua::new();

    // Read the Lua file -- relative diretory should be ran from project base for testing (ie. in the main folder)
    let lua_script: String = fs::read_to_string(env::current_dir()
    .expect("Unable to get current directory").to_str()
    .expect("Unable to convert current directory to str").to_string() + "/config.lua")?;

    // Load the Lua script
    let globals = lua.globals();
    lua.load(&lua_script).exec()?;

    // Upgrade System
    Command::new("sudo")
    .arg("pacman")
    .arg("-Syu")
    .output()
    .expect("Failed to exec command");

    // Get currently installed packages
    let output = Command::new("pacman")
    .arg("-Qeq")
    .output()
    .expect("Failed to execute command");

    if !output.status.success() {
        println!("Command executed with failing error code");
    }

    let raw_packages: String = String::from_utf8(output.stdout).unwrap();
    let mut packages : Vec<&str> = raw_packages.lines().collect();

    let packages_table: mlua::Table = globals.get("Packages")?;
    let official_table: mlua::Table = packages_table.get("Official")?;
    let aur_table: mlua::Table = packages_table.get("Aur")?;

    // Iterate over the config table
    for pair in official_table.pairs::<mlua::Value, mlua::Value>() {
        let (_key, value) = pair?;
        match value {

            // STRING
            mlua::Value::String(string) => {

                let string_str = string.to_str().unwrap();

                if packages.contains(&string_str) {
                    let index = packages.iter().position(|&r| r == string_str);
                    packages.remove(index.unwrap());
                    // println!("Already Installed {}...", string_str);
                } else {
                    println!("Attempting to install {}...", string_str);

                    let output = Command::new("sudo")
                    .arg("pacman")
                    .arg("-S")
                    .arg(string_str)
                    .arg("--noconfirm")
                    .output()
                    .expect("Failed to execute command");
                
                    if output.status.success() {
                        println!("Installed {}...", string_str);
                    } else {
                        println!("{:?}", String::from_utf8_lossy(&output.stderr));
                    }
                }
            },

            // Catch all function
            _ => (),

        }
    }

    // Install Location
    let mut global_install_location : String = String::new();

    for pair in aur_table.pairs::<mlua::Value, mlua::Value>() {
        let (_key, val) = pair?;
        match val {
            // TABLE
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

            // STRING
            mlua::Value::String(string) => {

                let string_str = string.to_str().unwrap();

                if packages.contains(&string_str) {

                    // Package is already installed - check for updates
                    let index = packages.iter().position(|&r| r == string_str);
                    let directory = global_install_location.clone() + "/" + string_str; // Can lead to double slash instances but doesn't seem to do anything
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
                    if (String::from_utf8_lossy(&output.stdout) == "Already up to date.") {

                    }

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

                    let current_dir = Command::new("pwd").output().expect("e");
                    env::set_current_dir(directory)?;

                    
                    println!("Building (AUR) {}...", string_str);
                    let output = Command::new("makepkg")
                    .arg("-si")
                    .arg("--noconfirm")
                    .output()
                    .expect("Failed to execute command");
                
                    if output.status.success() {
                        println!("Installed (AUR) {}...", string_str);
                    } else {
                        println!("{:?}", String::from_utf8_lossy(&output.stderr));
                    }

                    let mut og_directory = String::from_utf8(current_dir.stdout).unwrap();
                    og_directory.truncate(og_directory.len() - 1);
                    println!("{:?}", og_directory);
                    env::set_current_dir(og_directory)?;
                }
            },

            // Catch all function
            _ => (),
        }
    }

    if packages.len() > 0 {
        let mut output = Command::new("pacman");
        output.arg("--noconfirm");
        output.arg("-Rns");
    
        let mut dep = Command::new("pacman");
        dep.arg("--asdep");
        dep.arg("-D");
    
        for value in &packages {
            output.arg(value);
            dep.arg(value);
        }
    
        let dep = dep.output().expect("Failed to set packages to be dependencies!");
        let output = output.output().expect("Failed to remove packages!");
    
        println!("Test: {:?}", String::from_utf8_lossy(&output.stdout));

        if output.status.success() {
            println!("Removed {:?}...", packages);
        } else {
            println!("pacman -Rns failed with: Stdout: {:?}, Stderr: {:?}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        }
    
        if !dep.status.success() {
            println!("pacman -D failed with: Stdout: {:?}, Stderr: {:?}", String::from_utf8_lossy(&dep.stdout), String::from_utf8_lossy(&dep.stderr));
        }
    
        Command::new("pacman").arg("-Syu").output().expect("Failed to update entire system...");
    }

    println!("Finished...");
    Ok(())
}
