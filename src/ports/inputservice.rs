use std::io;
use csv::StringRecord;
use csv_converter::models::{ColumnDef, InputSource, ParsedContent};

pub trait InputService {
    fn parse(&self, input: InputSource) -> Result<ParsedContent, io::Error> ;
}