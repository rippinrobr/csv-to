extern crate barrel;
extern crate failure;
extern crate glob;
extern crate failure_derive;
extern crate postgres;

pub mod cmd;
pub mod cache;
pub mod parsers;
pub mod storage;

use std::default::Default;
use std::fmt;
use std::fs::File;
use std::io::{self, BufReader};

use barrel::types::BaseType;
use csv::StringRecord;
use serde;
use serde_derive::{Deserialize, Serialize};
use serde_json;

/// ConfigService is used to encapsulate the input from the user and allows each 'app' or sub-command
/// in csv-to to have access to the input without having to worry about parsing and gathering
pub trait ConfigService {
    /// Returns a Vec<InputSource> that represents all input files/sources
    fn get_input_sources(&self) -> Vec<InputSource>;
    /// Returns the name of the run
    fn get_name(&self) -> String;
    /// Returns true if the input files have column headers, currently
    /// all files have them or none of them do
    fn has_headers(&self) -> bool;
    /// Returns the name of the single table to store the data in or None if not used
    fn has_single_table(&self) -> Option<String>;
    /// Returns true if the user provides --delete-data as a command line flag
    fn should_delete_data(&self) -> bool;
    /// Returns true if tables/collections should be removed before
    /// loading the data
    fn should_drop_store(&self) -> bool;
    /// Indicates that the user asked to save to cache or not
    fn should_save_cache(&self) -> bool;
}

/// Potential data types for parsed columns and will be used when creating database tables
#[derive(PartialEq,Clone, Copy, Serialize, Deserialize)]
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
    pub fn to_str(&self) -> &str {
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
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ColumnDef{
    pub name: String,
    pub data_type: DataTypes,
    #[serde(skip)]
    pub potential_types: Vec<DataTypes>,
}

impl ColumnDef {
    // determines if the column's potential data type can be changed or not
    pub fn is_data_type_changeable(&self) -> bool {
        self.data_type == DataTypes::Empty || (self.data_type != DataTypes::F64 && self.data_type != DataTypes::String)
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

    pub fn new(cols: Vec<ColumnDef>, content: Vec<StringRecord>, errors: Vec<String>, file_name: String, num_lines: usize) -> Self {
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
    use barrel::types::BaseType;
    use csv::{StringRecord};
    use crate::{ColumnDef, DataTypes, ParsedContent};

    //==================================================
    // DataTypes tests
    #[test]
    fn data_types_to_database_type_empty() {
        assert_eq!(DataTypes::Empty.to_database_type(), BaseType::Text);
    }

    #[test]
    fn data_types_to_database_type_f64() {
        assert_eq!(DataTypes::F64.to_database_type(), BaseType::Double);
    }

    #[test]
    fn data_types_to_database_type_i64() {
        assert_eq!(DataTypes::I64.to_database_type(), BaseType::Integer);
    }

    #[test]
    fn data_types_to_database_type_string() {
        assert_eq!(DataTypes::String.to_database_type(), BaseType::Text);
    }

    #[test]
    fn data_type_to_str_empty() {
        assert_eq!(DataTypes::Empty.to_str(), "");
    }

    #[test]
    fn data_type_to_str_f64() {
        assert_eq!(DataTypes::F64.to_str(), "f64");
    }

    #[test]
    fn data_type_to_str_i64() {
        assert_eq!(DataTypes::I64.to_str(), "i64");
    }

    #[test]
    fn data_type_to_str_string() {
        assert_eq!(DataTypes::String.to_str(), "String");
    }

    //==================================================
    // ColumnDef tests
    #[test]
    fn is_data_type_change_with_empty_dt() {
        let cd = ColumnDef{
            data_type: DataTypes::Empty,
            name: String::from("mycol"),
            potential_types: Vec::new(),
        };

        assert_eq!(cd.is_data_type_changeable(), true);
    }

    #[test]
    fn is_data_type_change_with_i64_dt() {
        let cd = ColumnDef{
            data_type: DataTypes::I64,
            name: String::from("mycol"),
            potential_types: Vec::new(),
        };

        assert_eq!(cd.is_data_type_changeable(), true);
    }


    #[test]
    fn is_data_type_change_with_f64_dt() {
        let cd = ColumnDef{
            data_type: DataTypes::F64,
            name: String::from("mycol"),
            potential_types: Vec::new(),
        };

        assert_eq!(cd.is_data_type_changeable(), false);
    }

    #[test]
    fn is_data_type_change_with_string_dt() {
        let cd = ColumnDef{
            data_type: DataTypes::String,
            name: String::from("mycol"),
            potential_types: Vec::new(),
        };

        assert_eq!(cd.is_data_type_changeable(), false);
    }

    #[test]
    fn set_column_data_types_with_empty_values_should_give_string_data_type() {
        let mut pc = ParsedContent::default();
        let mut col_def = ColumnDef::default();

        col_def.potential_types = vec![DataTypes::Empty, DataTypes::Empty, DataTypes::Empty];
        pc.columns.push(col_def);
        pc.set_column_data_types();

        assert_eq!(pc.columns[0].data_type, DataTypes::String);
    }

    #[test]
    fn set_column_data_types_should_be_string_when_at_least_1_potential_type_is_string() {
        let mut pc = ParsedContent::default();
        let mut col_def = ColumnDef::default();

        col_def.potential_types = vec![DataTypes::F64, DataTypes::String, DataTypes::I64];
        pc.columns.push(col_def);
        pc.set_column_data_types();

        assert_eq!(pc.columns[0].data_type, DataTypes::String);
    }


    #[test]
    fn set_column_data_types_should_be_i64_when_at_least_1_potential_type_is_i64_and_others_are_empty() {
        let mut pc = ParsedContent::default();
        let mut col_def = ColumnDef::default();

        col_def.potential_types = vec![DataTypes::I64, DataTypes::Empty, DataTypes::Empty];
        pc.columns.push(col_def);
        pc.set_column_data_types();

        assert_eq!(pc.columns[0].data_type, DataTypes::I64);
    }

    #[test]
    fn set_column_data_types_should_be_f64_when_at_least_1_potential_type_is_f64() {
        let mut pc = ParsedContent::default();
        let mut col_def = ColumnDef::default();

        col_def.potential_types = vec![DataTypes::I64, DataTypes::Empty, DataTypes::F64];
        pc.columns.push(col_def);
        pc.set_column_data_types();

        assert_eq!(pc.columns[0].data_type, DataTypes::F64);
    }


    #[test]
    fn new() {
        let cols: Vec<ColumnDef> = vec![ColumnDef{name: String::from("test"), data_type: DataTypes::String, potential_types: Vec::new()}];
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