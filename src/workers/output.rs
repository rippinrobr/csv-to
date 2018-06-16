use std::fs::File;
use std::io::Error;
use std::io::prelude::*;

#[derive(Debug)]
pub struct Output {
    pub code_directory: String,
    sql_directory: String,
}

impl Output {
    pub fn new(code_directory: String, sql_directory: String) -> Output {
        Output{
            code_directory: code_directory,
            sql_directory: sql_directory
        }
    }

    pub fn write_code_to_file(&self, file_name: &str, code: String) -> Result<String, Error> {

        match File::create(format!("{}/{}.rs", &self.code_directory, &file_name).to_lowercase()) {
            Ok(mut file) => {
                match file.write_all(&code.into_bytes()) {
                    Ok(_) => Ok(file_name.to_string()),
                    Err(e) => Err(e)
                }
            },
            Err(e) => Err(e)
        }
    }

    pub fn write_sql_to_file(&self, file_name: &str, sql_str: String) -> Result<String, Error> {
        match File::create(format!("{}/{}.sql", &self.sql_directory, &file_name).to_lowercase()) {
            Ok(mut file) => {
                match file.write_all(&sql_str.into_bytes()) {
                    Ok(_) => Ok(file_name.to_string()),
                    Err(e) => Err(e)
                }
            },
            Err(e) => Err(e)
        }
    }
}