pub mod code_gen;
pub mod input;
pub mod parse_csv;

use csv::{StringRecord};
use models::{ColumnDef};

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
}