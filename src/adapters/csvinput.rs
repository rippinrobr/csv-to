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

    fn check_field_data_type(val: &str) -> DataTypes {
        match val.parse::<i64>() {
            Ok(_) => DataTypes::I64,
            _ => match val.parse::<f64>() {
                Ok(_) => DataTypes::F64,
                _ => DataTypes::String
            }
        }
    }
}

impl InputService for CSVService {

    fn parse(&self, input: InputSource) -> Result<ParsedContent, Error> {
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

        // this loop is for the lines in a file
        let mut col_index = 0;
        for raw_record in rdr.records() {
            let record = raw_record?.clone();
            // this loop is for the columns
            let mut col_index = 0;
            for col_data in record.clone().iter() {
                if col_data == "".to_string() {
                    continue;
                }

                // update columns data type if necessary
                if parsed_content.columns[col_index].data_type != DataTypes::String &&
                    parsed_content.columns[col_index].data_type != DataTypes::F64 {
                    let current_type = parsed_content.columns[col_index].data_type;
                    let possible_type: DataTypes = CSVService::check_field_data_type( col_data);

                    if possible_type != current_type &&  current_type == DataTypes::Empty {
                        parsed_content.columns[col_index].data_type = possible_type;
                    }
                }
                col_index += 1;
            }
            parsed_content.content.push(record);
            parsed_content.records_parsed += 1;
        }
        Ok(parsed_content)
    }
}