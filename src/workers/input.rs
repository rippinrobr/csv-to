use std::fmt;
use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;

pub enum InputType {
    CSV,
}

impl fmt::Debug for InputType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            InputType::CSV => "CSV",
        };
        write!(f, "{:#?}", printable)
    }
}

// TODO: 
#[derive(Debug)]
pub struct Input {
    pub input_type: InputType,
    pub files: Vec<String>,
    pub directories: Vec<String>
}

impl Input {
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