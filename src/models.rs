use std::default::Default;
use std::fmt;
use std::fs::File;
use std::io::{self, BufReader};

use barrel::types::BaseType;
use csv::StringRecord;

/// Potential data types for parsed columns and will be used when creating database tables
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
    /// Converts a DataTypes value to a string
    pub fn string(&self) -> &str {
        match *self {
            DataTypes::Empty => "",
            DataTypes::F64 => "f64",
            DataTypes::I64 => "i64",
            DataTypes::String => "String"
        }
    }

    /// Converts a DataTypes value to a Barrel::BaseType
    pub fn to_database_type(self) -> BaseType {
        match self {
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

/// Keeps meta data about the data in each column
#[derive(Clone, Default)]
pub struct ColumnDef{
    pub name: String, 
    pub data_type: DataTypes,
    pub potential_types: Vec<DataTypes>,
}

impl ColumnDef {
    /// creates a new ColumnDef with the name and data type provided
    pub fn new(name: String, data_type: DataTypes) -> ColumnDef {
        ColumnDef{
            name: name, 
            data_type: data_type,
            potential_types: Vec::new(),
        }
    }

    // determines if the column's potential data type can be changed or not
    pub fn is_data_type_changeable(&self) -> bool {
       self.data_type == DataTypes::Empty || self.data_type != DataTypes::String || self.data_type != DataTypes::F64
    }
}

impl fmt::Debug for ColumnDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "---\nname: {:?}\ndata_type: {:?}\n", self.name, self.data_type)
    }
}

pub trait Input {
    fn get_reader(&self) -> Result<BufReader<File>, io::Error>;
}

/// contains information about what the file contains and where it lives.
/// has_headers: indicates that the file has a header row or not
/// location: the path/uri for the input source
/// size: the size in bytes of the file's content
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
/// contains information about file during and after parsing
/// columns: A Vector of th ColumnDef objects that describe the column, name, data type, etc
/// content: Each line of the file is stored in a Vector of StringRecords (product of the CSV parsing
/// errors: contains all parsing errors that occurred while parsing the file
/// the name of the file parsed
/// the number of records parsed, used to validate that all records were stored in the database
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

    // goes through the column's proposed data types,
    // DataTypes::String trumps all other data types
    // DataTypes::F64 is second in line
    // DataTypes::I64 is next
    // if a column ends with DataTypes::Empty will be
    // changed to DataTypes::String
    pub fn set_column_data_types(&mut self) {
        for idx in 0..self.columns.len() {
            if self.columns[idx].potential_types.contains(&DataTypes::String) {
                self.columns[idx].data_type = DataTypes::String;
            } else if self.columns[idx].potential_types.contains(&DataTypes::F64) {
                self.columns[idx].data_type = DataTypes::F64;
            } else if self.columns[idx].potential_types.contains(&DataTypes::I64) {
                self.columns[idx].data_type = DataTypes::I64;
            }
            // I'm here and the data type is still empty then there's no other option but to default
            // it to string
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
}