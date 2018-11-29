use std::io;
use csv::StringRecord;
use csv_converter::models::{ColumnDef, InputSource, ParsedContent};
use crate::ports::inputservice::InputService;

#[derive(Clone,Debug)]
pub struct CSVService {
    //inputs: &'a [InputSource]
}

impl CSVService {
    /// Creates a new instance of the CSVService
    pub fn new() -> CSVService {
        CSVService {}
    }
}

impl InputService for CSVService {

    fn parse(&self, input: InputSource) -> Result<ParsedContent, io::Error> {
        let cols: Vec<ColumnDef> = Vec::new();
        let content: Vec<StringRecord> = Vec::new();

        Ok(ParsedContent::new(cols, content, input.location,0))
    }
}