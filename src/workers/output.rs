use std::fs::File;
use std::io::Error;
use std::io::prelude::*;
use std::borrow::BorrowMut;

// TODO: 
#[derive(Debug)]
pub struct Output {
    pub directory: String,
}

impl Output {
    pub fn new(directory: String) -> Output {
        Output{
            directory: directory,
        }
    }

    pub fn write_to_file(&self, file_name: &str, code: String) -> Result<String, Error> {

        match File::create(format!("{}/{}.rs", &self.directory, &file_name).to_lowercase()) {
            Ok(mut file) => {
                match file.write_all(&code.into_bytes()) {
                    Ok(_) => Ok(file_name.to_string()),
                    Err(e) => Err(e)
                }
            },
            Err(e) => Err(e)
        }
    }
}