pub mod code_gen;
pub mod config;
pub mod input;
pub mod models;
pub mod output;
pub mod db;
pub mod sqlite_code_gen;
pub mod sql_gen;

extern crate barrel;
extern crate codegen;
extern crate csv;
extern crate futures;
extern crate regex;
extern crate sqlite;

#[macro_use]
extern crate serde_derive;

use std::fs::File;
use std::io::Error;
use std::io::prelude::*;

// write_code_to_file writes the given String to a file at the path created by combining dir_path 
// and file_name.
pub fn write_code_to_file(dir_path: &str, file_name: &str, code: String) -> Result<String, Error> {

    match File::create(format!("{}/{}", dir_path, &file_name).to_lowercase()) {
        Ok(mut file) => {
            match file.write_all(&code.into_bytes()) {
                Ok(_) => Ok(file_name.to_string()),
                Err(e) => Err(e)
            }
        },
        Err(e) => Err(e)
    }
}

