use std::default::Default;
use std::fmt;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

use barrel::types;
use barrel::*;
use barrel::types::{Type, BaseType};

use csv::{Error, StringRecord};

#[derive(PartialEq,Clone, Copy)]
pub enum DataTypes {
    Empty,
    F64,
    I64, 
    String,
}

impl Default for DataTypes {
    fn default() -> DataTypes { DataTypes::Empty }
}

impl DataTypes {
    pub fn string(&self) -> &str {
        match *self {
            DataTypes::Empty => "",
            DataTypes::F64 => "f64",
            DataTypes::I64 => "i64",
            DataTypes::String => "String"
        }
    }

    pub fn to_database_type(&self) -> BaseType {
        match *self {
            DataTypes::Empty => BaseType::Text,
            DataTypes::F64 => BaseType::Double,
            DataTypes::I64 => BaseType::Integer,
            DataTypes::String => BaseType::Text
        }
    }
}

impl fmt::Debug for DataTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            DataTypes::Empty => "",
            DataTypes::F64 => "f64",
            DataTypes::I64 => "i64",
            DataTypes::String => "string"
        };
        write!(f, "{:#?}", printable)
    }
}


#[derive(Clone, Default)]
pub struct ColumnDef{
    pub name: String, 
    pub data_type: DataTypes,
    pub has_data: bool,
    pub potential_types: Vec<DataTypes>,
}

impl ColumnDef {
    pub fn new(name: String, data_type: DataTypes) -> ColumnDef {
        ColumnDef{
            name: name, 
            data_type: data_type,
            has_data: false,
            potential_types: Vec::new(),
        }
    }

    pub fn col_has_data(&mut self) {
        self.has_data = true;
    }

//    pub fn get_types_empty_state(data_type: DataTypes) -> String {
//        if data_type == DataTypes::String {
//            return String::from("''");
//        }
//
//        if data_type == DataTypes::F64 {
//            return String::from("0.0");
//        }
//
//        return String::from("0");
//    }

    pub fn is_data_type_changeable(&self) -> bool {
       self.data_type == DataTypes::Empty || self.data_type != DataTypes::String || self.data_type != DataTypes::F64
    }

    pub fn is_data_type_empty(&self) -> bool {
        self.data_type == DataTypes::Empty
    }

    pub fn set_data_type(&mut self, data_type: DataTypes) {
        self.data_type = data_type;
    }
}

impl fmt::Debug for ColumnDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "---\nname: {:?}\ndata_type: {:?}\nhas_data: {:?}", self.name, self.data_type, self.has_data)
    }
}

pub trait Input {
    fn get_reader(&self) -> Result<BufReader<File>, io::Error>;
}

#[derive(Clone,Debug)]
pub struct InputSource {
    pub has_headers: bool,
    pub location: String,
    pub size: u64,
}

impl Input for InputSource {
    fn get_reader(&self) -> Result<BufReader<File>, io::Error> {
        match File::open(&self.location) {
            Ok(f) => Ok(BufReader::new(f)),
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug)]
pub struct ParsedContent {
    pub columns: Vec<ColumnDef>,
    pub content: Vec<StringRecord>,
    pub errors: Vec<String>,
    pub file_name: String,
    pub records_parsed: usize,
}

impl Clone for ParsedContent {
    fn clone(&self) -> ParsedContent { 
        ParsedContent {
            columns: (*self).columns.clone(),
            content: (*self).content.clone(),
            errors: (*self).errors.clone(),
            file_name: (*self).file_name.clone(),
            records_parsed: (*self).records_parsed,
        }
    }
}

impl Default for ParsedContent {
    fn default() -> ParsedContent {
        ParsedContent {
            columns: Vec::new(),
            content: Vec::new(),
            errors: Vec::new(),
            file_name: String::new(),
            records_parsed: 0,
        }
    }
}

impl ParsedContent {

    pub fn new(cols: Vec<ColumnDef>, content: Vec<StringRecord>, errors: Vec<String>, file_name: String, num_lines: usize) -> ParsedContent {
        ParsedContent {
            columns: cols,
            content,
            errors,
            file_name,
            records_parsed: num_lines,
        }
    }

    pub fn content_to_string_vec(&self) -> Result<Vec<Vec<String>>, Error> {
        let mut content_strings: Vec<Vec<String>> = Vec::new();

        for line in &self.content {
            let s: Vec<String>= line.deserialize(None)?;
            content_strings.push(s);
        }

        Ok(content_strings)
    }

    pub fn get_struct_name(self) -> String {
        let name = format!("{}", Path::new(&self.file_name).display());
        let first_letter = name.trim_right_matches(".csv").chars().next().unwrap();
        name.trim_right_matches(".csv").to_string().replace(first_letter, &first_letter.to_string().to_uppercase())
    }

    pub fn set_column_data_types(&mut self) {
        for idx in 0..self.columns.len() {
            if self.columns[idx].potential_types.contains(&DataTypes::String) {
                self.columns[idx].data_type = DataTypes::String;
            } else {
                if self.columns[idx].potential_types.contains(&DataTypes::F64) {
                    self.columns[idx].data_type = DataTypes::F64;
                } else if self.columns[idx].potential_types.contains(&DataTypes::I64) {
                    self.columns[idx].data_type = DataTypes::I64;
                }
            }
            if self.columns[idx].data_type == DataTypes::Empty {
                self.columns[idx].data_type = DataTypes::String;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use csv::{StringRecord};
    use csv::Error;
    use crate::models::{ColumnDef, DataTypes, ParsedContent};

    #[test]
    fn new() {
        let cols: Vec<ColumnDef> = vec![ColumnDef::new("test".to_string(), DataTypes::String)];
        let cols_len = cols.len();
        let content: Vec<StringRecord> = vec![StringRecord::new()];
        let file_name = "my-file".to_string();
        let num_lines = 22;

        let pc = ParsedContent::new(cols, content.clone(), Vec::new(),  file_name.clone(), num_lines);
        assert_eq!(cols_len, pc.columns.len());
        assert_eq!(content.len(), pc.content.len());
        assert_eq!(file_name, pc.file_name);
        assert_eq!(num_lines, pc.records_parsed);
    }

    #[test]
    fn content_to_string_vec() {
        let cols: Vec<ColumnDef> = vec![ColumnDef::new("test".to_string(), DataTypes::String)];
        let cols_len = cols.len();
        let content: Vec<StringRecord> = vec![StringRecord::from(vec!["a", "b", "c"])];
        let file_name = "my-file".to_string();
        let errors: Vec<String> = Vec::new();
        let num_lines = 22;

        let pc = ParsedContent::new(cols, content.clone(), errors.clone(), file_name.clone(), num_lines);
        
        let mut string_vec: Vec<Vec<String>> = pc.content_to_string_vec().unwrap();
        assert_eq!(1, string_vec.len());

        let str_record = &string_vec.pop().unwrap();
        assert_eq!("a,b,c".to_string(), str_record.join(","));
    }
}