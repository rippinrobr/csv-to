use failure::Error;
use regex::Regex;
use std::io;
use std::str;
use csv::StringRecord;
use csv_converter::models::{ColumnDef, InputSource, ParsedContent};

pub trait InputService {
    fn parse(&self, input: InputSource) -> Result<ParsedContent, Error> ;

    fn validate_field_name(&self, name: &str, name_re: &Regex) -> String {
        if name == "" {
            return String::new();
        }

        let mut name_str = name.to_string();

        if name_re.is_match(name) {
            return name.to_string()
        }

        // 0. replace all + and - chars with plus and minus
        name_str = str::replace(&name_str, "+", "plus");
        name_str = str::replace(&name_str, "-", "minus");
        // 0.5 replace / with a _
        name_str = str::replace(&name_str, "/", "_");
        // 1. if name starts with a number then add _ at the beginning
        let first_char = name_str.chars().next().unwrap();
        if first_char.is_numeric() {
            return format!("_{}", name_str);
        }

        name_str
    }
 }