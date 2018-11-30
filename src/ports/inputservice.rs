use failure::Error;
use regex::Regex;
use std::io;
use csv::StringRecord;
use csv_converter::models::{ColumnDef, InputSource, ParsedContent};

pub trait InputService {
    fn parse(&self, input: InputSource) -> Result<ParsedContent, Error> ;

    fn validate_field_name(&self, name: &str, name_re: &Regex) -> String {
        if name == "" {
            return String::new();
        }

        let name_str = name.to_string();

        if name_re.is_match(name) {
            return name.to_string()
        }

//        while let Some(name_char) = name_str.chars().next() {
//            if !name_char.is_alphanumeric() && name_char != '_' {
//                if name_char == '+' {
//                    return self.validate_field_name(&name_str.replace(name_char, "plus"), name_re);
//                }
//
//                if name_char == '-' {
//                    return self.validate_field_name(&name_str.replace(name_char, "minus"), name_re);
//                }
//
//                if name_char == '/' {
//                    return self.validate_field_name(&name_str.replace(name_char, ""), name_re);
//                }
//
//                return self.validate_field_name(&name_str.replace(name_char, "_"), name_re);
//            }
//        }

        name_str
    }
 }