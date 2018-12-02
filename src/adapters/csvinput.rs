use failure::{Error, err_msg};
use std::io;
use regex::Regex;
use csv::{Reader, StringRecord};

use csv_converter::models::{ColumnDef, DataTypes, InputSource, ParsedContent};
use crate::ports::inputservice::InputService;


#[derive(Clone,Debug)]
pub struct CSVService {
    field_name_regex: Regex,
}

impl CSVService {
    /// Creates a new instance of the CSVService
    pub fn new() -> CSVService {
        CSVService {
            field_name_regex: Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]+$").unwrap(),
        }
    }

    pub fn create_column_defs(&self, headers: &StringRecord) -> Vec<ColumnDef> {
        let mut col_defs: Vec<ColumnDef> = Vec::new();
        let num_cols = headers.len();

        for n in 0..num_cols {
            // 0. get the name
            let cleaned_name = self.validate_field_name(&headers[n], &self.field_name_regex);
            let cd = ColumnDef {
                name: cleaned_name.clone(),
                data_type: DataTypes::Empty,
                has_data: false,
            };
            col_defs.push(cd);
        }

        col_defs
    }
}

impl InputService for CSVService {

    fn parse(&self, input: InputSource) -> Result<ParsedContent, Error> {
        let cols: Vec<ColumnDef> = Vec::new();
        let content: Vec<StringRecord> = Vec::new();
        let mut rdr = Reader::from_path(&input.location)?;
        let mut parsed_content = ParsedContent::default();
        parsed_content.file_name = input.location;

        // this is in its own scope because headers borrows from the reader
        {
            match rdr.headers() {
                Ok(headers) => parsed_content.columns = self.create_column_defs(headers),
                Err(e) => return Err(failure::err_msg(format!("{}", e)))
            }
        }
        println!("{:#?}", parsed_content.columns);
        // this loop is for the lines in a file
        for raw_record in rdr.records() {
            // this loop is for the columns
            for col_data in raw_record?.clone().iter() {
                //println!("{:?}", col_data);
            }
        }
        //println!("parsed_content: {:#?}", parsed_content);
        Ok(parsed_content)
    }
}