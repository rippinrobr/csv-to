use std::fmt;
use std::string::ToString;
use std::path::PathBuf;
use super::models::{DataTypes, ColumnDef, ParsedContent};
use csv::StringRecord;

#[derive(Clone, Debug)]
pub struct InputFile {
    file_path: PathBuf,
    size: u64,
    columns: Vec<ColumnDef>,
    content: Vec<StringRecord>,
}

// POC below, this may live but it may not
#[derive(Copy, Clone, PartialEq, Deserialize)]
pub enum InputType {
    CSV,
    NotSupported
}

impl InputType {
    
    pub fn get_input_type(str_type: &str) -> InputType {
        
        if str_type.to_uppercase() == "CSV" {
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

#[cfg(test)]
mod test {
    use input::InputType;

    #[test]
    fn check_csv_str_inputs() {
        let upper = "CSV".to_string();
        let lower = "csv".to_string(); 
        let mixed = "cSv".to_string();

        assert_eq!(InputType::get_input_type(&upper), InputType::CSV);
        assert_eq!(InputType::get_input_type(&lower), InputType::CSV);
        assert_eq!(InputType::get_input_type(&mixed), InputType::CSV);
    }

    #[test]
    fn check_unsupported_input_type() {
        let unsupported = "binary".to_string();

        assert_eq!(InputType::get_input_type(&unsupported), InputType::NotSupported);
    }
}