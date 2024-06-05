use std::{collections::HashMap, fs, path::Path, process::Command};
use colour::*;
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

                if Path::new(&link_dir).exists() {
                    let metadata = fs::symlink_metadata(&link_dir).unwrap();
                    already_exist = metadata.file_type().is_symlink();
                    //println!("Link dir exists, is it a symlink? {}", already_exist);
                }

                //println!("After link dir check {}", already_exist);

                if !already_exist { // Only create the symlink if there's not already one there, we confirmed it was valid in the removal process

                    let mut output = Command::new("sudo");
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