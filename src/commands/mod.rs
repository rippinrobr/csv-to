pub mod input;
pub mod parse_csv;

use csv::{StringRecord};
use models::{ColumnDef};


#[derive(Debug)]
pub struct ParsedContent {
    columns: Vec<ColumnDef>,
    content: Vec<StringRecord>,
    file_name: String,
    records_parsed: usize,
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
}


trait Task {
    fn execute(&self) -> Self;
} 