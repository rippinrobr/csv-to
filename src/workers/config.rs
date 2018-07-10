extern crate toml;

use workers::input::*;
use std::fs::{self, DirEntry};
use std::path::Path;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub input_type: InputType,
    pub files: Vec<String>,
    pub directories: Vec<String>
}

impl Config {
    pub fn load(config_str: &str) -> Config {
        match toml::from_str(config_str) {
            Ok(config) => {
                let mut cfg: Config = config;
                cfg.add_files_in_directories();
                cfg
            },
            Err(e) => panic!(format!("Config ERROR: {}", e))
        }
    }

    pub fn add_files_in_directories(&mut self) -> usize {
        let mut num_files: usize = 0;
        
        for dir in &self.directories {
            let dir_path = Path::new(dir);
            if dir_path.is_dir() {
                let paths = fs::read_dir(dir_path).unwrap();
                for dir_entry in paths {
                    let path = dir_entry.unwrap().path();
                    if path.is_file() { 
                        let path_str: String = path.display().to_string();
                        if !path_str.ends_with("csv") && !path_str.ends_with("CSV") {
                            continue;
                        }
        
                        &self.files.push(path_str);
                        num_files += 1;
                    }
                }
            }
        }
        num_files
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_input() {
        let test_yaml = r#"
        input_type = 'CSV'
        files = ['a.csv', 'b.csv' ]
        directories = ['~/src/baseballdatabank/core', '~/src/hockeydatabank' ]

        "#;
        
        let actual = Config::load(test_yaml);

        assert_eq!("CSV", actual.input_type.to_string());
        assert_eq!(2, actual.files.len());
        assert_eq!("a.csv", actual.files[0]);
        assert_eq!("b.csv", actual.files[1]);
        assert_eq!(2, actual.directories.len());
        assert_eq!("~/src/baseballdatabank/core", actual.directories[0]);
        assert_eq!("~/src/hockeydatabank", actual.directories[1]);
    }
}