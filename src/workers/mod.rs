pub mod code_gen;
pub mod input;
pub mod output;
pub mod parse_csv;
pub mod sqlite;
pub mod sqlite_code_gen;
pub mod sql_gen;

use csv::{StringRecord};
use csv::Error;
use models::{ColumnDef};
//use std::error::Error;

pub trait WorkOrder {
    fn execute() -> Result<i32, String>;
}

#[derive(Debug)]
pub struct ParsedContent {
    pub columns: Vec<ColumnDef>,
    pub content: Vec<StringRecord>,
    pub file_name: String,
    pub records_parsed: usize,
}

impl ParsedContent {
    pub fn new(cols: Vec<ColumnDef>, content: Vec<StringRecord>, file_name: String, num_lines: usize) -> ParsedContent {
        ParsedContent {
            columns: cols,
            content: content,
            file_name: file_name,
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

    pub fn get_struct_name(&self) -> &str {
        self.file_name.trim_right_matches(".csv")
    }
}

#[cfg(test)]
mod tests {
    use csv::{StringRecord};
    use csv::Error;
    use workers::ParsedContent;
    use models::{ColumnDef, DataTypes};

    #[test]
    fn new() {
        let cols: Vec<ColumnDef> = vec![ColumnDef::new("test".to_string(), DataTypes::String)];
        let cols_len = cols.len();
        let content: Vec<StringRecord> = vec![StringRecord::new()];
        let file_name = "my-file".to_string();
        let num_lines = 22;

        let pc = ParsedContent::new(cols, content.clone(), file_name.clone(), num_lines);
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
        let num_lines = 22;

        let pc = ParsedContent::new(cols, content.clone(), file_name.clone(), num_lines);
        
        let mut string_vec: Vec<Vec<String>> = pc.content_to_string_vec().unwrap();
        assert_eq!(1, string_vec.len());

        let str_record = &string_vec.pop().unwrap();
        assert_eq!("a,b,c".to_string(), str_record.join(","));
    }
}