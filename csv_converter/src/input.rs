use std::fmt;
use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;
use std::string::ToString;

#[derive(Clone, PartialEq, Deserialize)]
pub enum InputType {
    CSV,
    NotSupported
}

impl InputType {
    
    pub fn get_input_type(str_type: &str) -> InputType {
        
        if str_type == "CSV" || str_type == "csv" {
            InputType::CSV
        } else {
            InputType::NotSupported
        }
    }
}

impl fmt::Debug for InputType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            InputType::CSV => "CSV",
            InputType::NotSupported => "NOT_SUPPORTED"
        };
        write!(f, "{:#?}", printable)
    }
}

impl ToString for InputType {
    fn to_string(&self) -> String {
        (match *self {
            InputType::CSV => "CSV",
            InputType::NotSupported => "NOT_SUPPORTED"
        }).to_string()
    }
}
// TODO: Add support for a TOML file to take in input parameters 
//       and any other params that will be or are required
#[derive(Debug, Deserialize)]
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

#[cfg(test)]
mod test {
    use workers::input::InputType;

    #[test]
    fn check_csv_str_inputs() {
        let upper = "CSV".to_string();
        let lower = "csv".to_string(); 
        let mixed = "cSv".to_string();

        assert_eq!(InputType::get_input_type(&upper), InputType::CSV);
        assert_eq!(InputType::get_input_type(&lower), InputType::CSV);
    }

    #[test]
    fn check_unsupported_input_type() {
        let unsupported = "binary".to_string();

        assert_eq!(InputType::get_input_type(&unsupported), InputType::NotSupported);
    }
}