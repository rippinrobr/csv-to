use failure::{Error, err_msg};
use std::io;
use regex::Regex;
use csv::{Reader, StringRecord};

use csv_converter::models::{ColumnDef, InputSource, ParsedContent};
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

    pub fn create_column_defs(&self, headers: StringRecord) -> Vec<ColumnDef> {
        let col_defs: Vec<ColumnDef> = Vec::new();
        let num_cols = headers.len();

        for n in 0..num_cols {
            // 0. get the name
            let cleaned_name = self.validate_field_name(&headers[n], &self.field_name_regex);
            println!("name: {:?} => cleaned_name: {:?}", &headers[n], &cleaned_name);
            //ColumnDef::new()
            //parsed_content.push(DataTypes::Empty);
            //= self.check_col_name(&h[n]);
//                      headers.push(col_name.to_owned());
        }

        col_defs
    }
}

impl InputService for CSVService {

    fn parse(&self, input: InputSource) -> Result<ParsedContent, Error> {
        let cols: Vec<ColumnDef> = Vec::new();
        let content: Vec<StringRecord> = Vec::new();
        let mut rdr = Reader::from_path(&input.location)?;
        let mut csv_iter = rdr.records();
        let mut parsed_content = ParsedContent::default();

        println!("input.location: {:?}", input.location);
        if input.has_headers && parsed_content.records_parsed == 0 {
            match csv_iter.next() {
                Some(rec) => {
                    let headers = rec.unwrap();
                    parsed_content.columns = self.create_column_defs(headers);

                    parsed_content.records_parsed += 1;
//                    match rec {
//                       Ok(headers) => {
//                           parsed_content.columns = self.create_column_defs(headers);
//                           parsed_content.file_name = input.location;
//                       },
//                        None => return format::err_msg("No headers found when the the headers should exist"),
//                    }
                },
                None => {
                    eprintln!("ERROR: NO RECORD RETURNED");
                }
            }
        }
        // take care of headers here if there are any

        // process the rows here

        Ok(ParsedContent::new(cols, content, input.location.clone(),0))
    }
}