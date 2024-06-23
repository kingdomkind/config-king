use std::{collections::HashMap, fs, path::Path, process::Command};
use colour::*;
use crate::{unstatic, utilities::{check_if_path_exists, remove_path}, AUTH_AGENT};

use super::utilities;

pub fn generate_symlinks(symlinks_table : mlua::Table) -> String {
    /*
    i just found something weird:
    If you create a symlink eg from /home/pika/Config/test2 to /home/pika/Config/scripts (so that when you enter test2, it mirrors scripts internals)
    if test2 already exists, when you cd into test2 and ls you will ssee scripts, then you can cd into scripts and see the subdirectories in there
    if test2 doesn't exist, when you cd into test2 and ls you will instantly see the subdirectories of scripts
    I'm not sure whether i should deny the former behaviour, i guess if someone thinks that is a bug i'll add a check to prevent that
     */

    let mut symlink_msg = String::from("symlinks=[");

    for pair in symlinks_table.pairs::<mlua::Value, mlua::Value>() {
        let (key, value) = pair.unwrap();
        match value {

            mlua::Value::String(string) => {

                let string_str = string.to_str().unwrap();
                let original_dir =  string_str.to_string();
                let link_dir = key.to_string().unwrap();
                let symlink_dir = link_dir.clone() + "=" + &original_dir;
                let mut already_exist = false;

                //println!("Debugging: {} {}", link_dir, original_dir);

                // Be aware this needs root permissions to check certain file systems
                let output = Command::new(unstatic!(AUTH_AGENT))
                .arg("test")
                .arg("-L")
                .arg(&link_dir)
                .output()
                .expect("Failed to execute command");

                if output.status.success() {
                    already_exist = true
                }

                //println!("After link dir check {}", already_exist);

                if !already_exist { // Only create the symlink if there's not already one there, we confirmed it was valid in the removal process

                    // Check that there's no random file there already
                    if check_if_path_exists(link_dir.clone()) {
                        remove_path(link_dir.clone());
                    }                    

                    let path_to_ensure_made = Path::new(&link_dir).parent().unwrap();
                    let output = Command::new(unstatic!(AUTH_AGENT))
                    .arg("test")
                    .arg("-d")
                    .arg(path_to_ensure_made)
                    .output()
                    .expect("Failed to execute command");

                    if !output.status.success() {
                        yellow!("WARNING: ");
                        white_ln!("The directory {:?} does not exist, would you like to create it? (y/n)", path_to_ensure_made);
                        let confirm = utilities::get_confirmation();
                        if confirm {
                            let mut output = Command::new(unstatic!(AUTH_AGENT));
                            output.arg("mkdir");
                            output.arg("-p");
                            output.arg(path_to_ensure_made);
                            let _res = utilities::send_output(output);
                        }
                    }

                    let mut output = Command::new(unstatic!(AUTH_AGENT));
                    output.arg("ln");
                    output.arg("-s");
                    output.arg(&original_dir);
                    output.arg(&link_dir);
                    let res = utilities::send_output(output);

                    match res {
                        false => {
                            red!("ERROR: ");
                            white_ln!("Failed to create symlink from {} to {}", link_dir, original_dir);
                            // continue; We do not need a continue. We assume the symlink already exists. If it does not, it will be cleaned up
                            // from the save.king file in the next run, as it will be discovered that it does not exist.
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

    return symlink_msg;
}

pub fn delete_old_symlinks(curent_symlinks: Vec<String>,  new_symlinks: HashMap<String, String>) {
    for value in curent_symlinks {
        let locations: Vec<String> = value
        .split('=')
        .map(|s| s.to_string())
        .collect();

        // 0 INDEX IS THE SYMLINK FILE / FOLDER LOCATION, 1 IS THE FILE / FOLDER THAT IS BEING SYMLINKED TO (ie. 0 shows as glowing text in a terminal, 1 doesn't)

        // Check if the symlink already exists, is valid, and if so skip this loop
        if Path::new(&locations[0]).exists() {
            if new_symlinks.contains_key(&locations[0]) {
                let metadata = fs::symlink_metadata(&locations[0]).unwrap();
                if metadata.file_type().is_symlink() {
                    if new_symlinks[&locations[0]] == locations[1] {
                        continue;
                    }
                }
            }
        }

        // Invalid symlink, banish it
        utilities::remove_path(locations[0].to_string());
    }
}

pub fn read_save(identifier_bound: usize, value: String) -> Vec<String> {
    let remainder = &value[identifier_bound+2..value.len()-1]; // +2 to slice of the =[ and -1 to slice the ]
    let mut current_symlinks: Vec<String> = Vec::new();

    let sub_elements: Vec<String> = remainder
    .split(',')
    .map(|s| s.to_string())
    .filter(|s: &String| !s.is_empty()) // Filter out empty strings
    .collect();

    for raw_path in sub_elements {
        //println!("Printing Raw Path: {}",raw_path);
        let path: &str =  &raw_path[1..raw_path.len()-1]; // remove speech marks
        current_symlinks.push(path.to_string());
    }

    return current_symlinks;
}