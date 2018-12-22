//! The parsers module
//!
//!
pub mod csv;

use failure::Error;
use regex::Regex;
use std::str;
use crate::{InputSource, ParsedContent};

pub trait InputService {
    // responsible for parsing the file, creating a description of each column in the file
    fn parse(&self, input: InputSource) -> Result<ParsedContent, Error> ;
    // ensures that the name of the field is valid for creating columns in a database table and
    // struct fields
    fn validate_field_name(&self, idx: usize,  name: &str, name_re: &Regex) -> String {
        if name == "" {
            return format!("col_{}", idx);
        }

        let mut name_str = name.to_string();
        if name_re.is_match(name) {
            return name.to_string()
        }

        // 0. replace all + and - chars with plus and minus
        name_str = str::replace(&name_str, "+", "plus");
        name_str = str::replace(&name_str, "-", "minus");
        name_str = str::replace( &name_str, ".", "_");

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