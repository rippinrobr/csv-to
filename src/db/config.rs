use std::fs;
use std::path::PathBuf;
use glob::{glob_with, MatchOptions};
use csv_converter::models::{InputSource};
use crate::{
    ports::configservice::ConfigService,
    db::Types
};

/// Config contains all the parameters provided by the user
#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    directories: Vec<String>,
    db_type: Types,
    connection_info: String,
    name: String,
    drop_store: bool,
    no_headers: bool,
}

impl Config {
    /// Creates a struct of all the CmdLine Arguments
    pub fn new(files_path: Vec<PathBuf>, directories: Vec<PathBuf>, db_type: Types, connection_info: String, name: String, drop_tables: bool, no_headers: bool) -> Config {
        Config {
            files: Config::convert_to_vec_of_string(files_path),
            directories: Config::convert_to_vec_of_string(directories),
            db_type,
            connection_info,
            name,
            drop_store: drop_tables,
            no_headers,
        }
    }

    fn create_input_source(has_headers: bool, file_path: String) -> InputSource {
        let meta = fs::metadata(file_path.clone()).unwrap();
        //TODO: default has_headers to true for now, will add a flag that says --no-headers
        InputSource {
            has_headers,
            location: file_path,
            size: meta.len(),
        }
    }

    fn convert_to_vec_of_string(paths: Vec<PathBuf>) -> Vec<String> {
        let mut string_paths: Vec<String> = Vec::new();

        for p in paths.into_iter() {
            string_paths.push(p.into_os_string().into_string().unwrap_or_default());
        }

        string_paths
    }
}

impl ConfigService for Config {
    /// get_locations returns the path's to the input files
    fn get_input_sources(&self) -> Vec<InputSource> {
        let mut sources: Vec<InputSource> = Vec::new();
        // used by the glob_with call to tell it how we want to look
        // for files in a directory
        let options = &MatchOptions {
            case_sensitive: false,
            require_literal_leading_dot: false,
            require_literal_separator: false,
        };

        // Gets the files inside the given directories and adds them to the
        // input source
        for d in &self.directories {
            for f in  glob_with(&format!("{}/*.{}", d, "csv"), options).unwrap() {
                match f {
                    Ok(file_path) => sources.push(Config::create_input_source(self.has_headers(),file_path.into_os_string().into_string().unwrap_or_default()) ),
                    Err(e) => eprintln!("ERROR: {}", e),
                }
            }
        }

        // files
        for file_path in &self.files {
            sources.push(Config::create_input_source(self.has_headers(),file_path.clone()) );
        }

        sources.to_owned()
    }

    fn has_headers(&self) -> bool {
        !self.no_headers
    }

    fn should_drop_store(&self) -> bool { self.drop_store }
}
