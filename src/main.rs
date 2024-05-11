use mlua::prelude::*;
use std::{env, fs, io::{BufRead, BufReader}, process::{Command, Output}, };

/* 
fn logger(to_print: &'static str, log_level: &'static str) {
    let array = ["juan"];
} */

fn send_output(mut output : Command) -> bool{
    let mut spawned = output.spawn().expect("Unable to output command");
    let reader = BufReader::new(spawned.stdout.take().expect("Failed to capture stdout"));

    for line in reader.lines() {
        println!("{}", line.expect("Failed to read line"));
    }

    let wait = spawned.wait().expect("Failed to wait for output to end");
    return  wait.success();

}

fn build_aur(name : &str) {
    println!("Building (AUR) {}...", name);
    let output = Command::new("makepkg")
    .arg("-si")
    .arg("--noconfirm")
    .output()
    .expect("Failed to execute command");

    if output.status.success() {
        println!("Installed (AUR) {}...", name);
    } else {
        println!("{:?}", String::from_utf8_lossy(&output.stderr));
    }
}

fn get_current_directory() -> String {
    let current_dir = Command::new("pwd").output().expect("Couldn't get current directory");
    let mut og_directory = String::from_utf8(current_dir.stdout).unwrap();
    og_directory.truncate(og_directory.len() - 1);
    //println!("{:?}", og_directory);
    return  og_directory;
}

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
    let flatpak_table: mlua::Table = packages_table.get("Flatpak")?;

    /*
    
    INSTALLING PACKAGES

     */

    // Iterate over the official table
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
                        println!("Entered");
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

            // Catch all function
            _ => (),
        }
    }

    let flatpak_packages = Command::new("flatpak")
    .arg("list")
    .arg("--app")
    .arg("--columns=application")
    .output()
    .expect("Failed to execute command");

    let flatpak_packages: String = String::from_utf8(flatpak_packages.stdout).unwrap();
    let mut flatpak_packages : Vec<&str> = flatpak_packages.lines().collect();
    flatpak_packages.remove(0);

    // Iterate over the config table
    for pair in flatpak_table.pairs::<mlua::Value, mlua::Value>() {
        let (_key, value) = pair?;
        match value {

            // STRING
            mlua::Value::String(string) => {

                let string_str = string.to_str().unwrap();

                if flatpak_packages.contains(&string_str) {
                    let index = flatpak_packages.iter().position(|&r| r == string_str);
                    flatpak_packages.remove(index.unwrap());
                    println!("Already Installed {}...", string_str);
                } else {
                    println!("Attempting to install {}...", string_str);

                    let output = Command::new("flatpak")
                    .arg("install")
                    .arg(string_str)
                    .arg("--assumeyes")
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

    /*
    
    REMOVING PACKAGES
    
     */

    if packages.len() > 0 {
        let mut output = Command::new("sudo");
        output.arg("pacman");
        output.arg("--noconfirm");
        output.arg("-Rns");
    
        let mut dep = Command::new("sudo");
        dep.arg("pacman");
        dep.arg("--asdep");
        dep.arg("-D");
    
        for value in &packages {
            output.arg(value);
            dep.arg(value);
        }
    
        let dep = dep.output().expect("Failed to set packages to be dependencies!");

        let ret : bool = send_output(output);
        println!("{}", ret);
        
        /* 
        let output: Output = output.output().expect("Failed to remove packages!");
    
        if output.status.success() {
            println!("Removed {:?}...", packages);
        } else {
            println!("pacman -Rns failed with: Stdout: {:?}, Stderr: {:?}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        } */
    
        if !dep.status.success() {
            println!("pacman -D failed with: Stdout: {:?}, Stderr: {:?}", String::from_utf8_lossy(&dep.stdout), String::from_utf8_lossy(&dep.stderr));
        }
    
        Command::new("pacman").arg("-Syu").output().expect("Failed to update entire system...");
    }

    if flatpak_packages.len() > 0 {
        let mut output = Command::new("flatpak");
        output.arg("uninstall");
        output.arg("--assumeyes");

        for value in &flatpak_packages {
            output.arg(value);
        }

        let output: Output = output.output().expect("Failed to remove packages!");

        if output.status.success() {
            println!("Removed {:?}...", flatpak_packages);
        } else {
            println!("flatpak uninstall failed: {:?}, Stderr: {:?}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        }
    
        let output = Command::new("flatpak")
        .arg("uninstall")
        .arg("--unused")
        .arg("--assumeyes")
        .output()
        .expect("Failed to execute command");
    
        if !output.status.success() {
            println!("Failed to uninstall unused runtimes: {:?}", String::from_utf8_lossy(&output.stderr));
        }
    }

    println!("Finished...");
    Ok(())
}
