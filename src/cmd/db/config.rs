use std::fs;
use std::path::PathBuf;
use glob::{glob_with, MatchOptions};
use crate::models::{InputSource};
use crate::{
    ConfigService,
    cmd::db::Types
};

/// Config contains all the parameters provided by the user
#[derive(Debug)]
pub struct Config {
    extension: String,
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
    pub fn new(extension: String, files_path: Vec<PathBuf>, directories: Vec<PathBuf>, db_type: Types, connection_info: String, name: String, drop_tables: bool, no_headers: bool) -> Config {
        Config {
            extension,
            files: Config::convert_to_vec_of_string(files_path),
            directories: Config::convert_to_vec_of_string(directories),
            db_type,
            connection_info,
            name,
            drop_store: drop_tables,
            no_headers,
        }
    }

    fn create_input_source(has_headers: bool, file_path: String) -> Result<InputSource, failure::Error> {
        match fs::metadata(file_path.clone()) {
            Ok(meta) => Ok(InputSource {
                            has_headers,
                            location: file_path,
                            size: meta.len(),
                        }),
            Err(e) => Err(failure::err_msg(format!("input source error: {}", e)))
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
            for f in  glob_with(&format!("{}/*.{}", d, &self.extension), options).unwrap() {
                match f {
                    Ok(file_path) => {
                        match Config::create_input_source(self.has_headers(),file_path.into_os_string().into_string().unwrap_or_default()) {
                            Ok(input_src) => {
                                if !input_src.location.ends_with(".sh") {
                                    sources.push(input_src)
                                }
                            },
                            Err(e) => eprintln!("{}", e),
                        }
                    },
                    Err(e) => eprintln!("ERROR: {}", e),
                }
            }
        }

        // files
        for file_path in &self.files {
            match Config::create_input_source(self.has_headers(),file_path.clone()) {
                Ok(input_src) => sources.push(input_src),
                Err(e) => eprintln!("{}", e),
            }
        }

        sources.to_owned()
    }

    fn has_headers(&self) -> bool {
        !self.no_headers
    }

    fn should_drop_store(&self) -> bool { self.drop_store }
}
