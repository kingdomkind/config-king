use std::{fs::{File, OpenOptions}, io::{Read, Write}};

use colour::*;

pub fn read_file_elements(save_location: String) -> Vec<String> {
    let file = OpenOptions::new()
    .read(true)
    .open(save_location);

    let mut content = Vec::new();
    file.unwrap().read_to_end(&mut content).unwrap();
    let content_str = String::from_utf8_lossy(&content);

    let elements: Vec<String> = content_str
    .split(';')
    .map(|s| s.trim_end_matches('\n').to_string())
    .filter(|s: &String| !s.is_empty()) // Filter out empty strings
    .collect();

    return elements;
}

pub fn create_file_location(save_location: String) {
    let res = File::create(save_location);

    match res {
        Err(err)=> {
            red!("ERROR: ");
            white_ln!("Failed to create file | {}", err);
        },
        _file => {
            green!("Created: ");
            white_ln!("config.king file");
        }
    }
}

pub fn overwrite_file(save_location: String, text: String) {
    let mut file = OpenOptions::new()
    .write(true)
    .truncate(true)
    .open(save_location).unwrap();

    let res = file.write_all(text.as_bytes());

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
}