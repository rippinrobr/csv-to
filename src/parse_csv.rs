extern crate csv;

use super::models::{DataTypes, ParsedContent};
use super::models::ColumnDef;

use csv::{
    StringRecord,
    Reader
};
use std::{
    fmt,
    fs::File,
    io,
    io::Error
};

pub struct ParseFile {
    pub path: String,
}

impl ParseFile {
    
    pub fn parse_file(&self) -> Result<usize, Error> {
        let mut num_lines: usize = 0;
        let mut headers: Vec<String> = Vec::new();
        let mut data_types: Vec<DataTypes> = Vec::new();
        let mut columns: Vec<ColumnDef> = Vec::new();
        let mut col_count: usize = 0;
        
        // Build the CSV reader and iterate over each record.
        let file = File::open(&self.path)?;
        let reader = io::BufReader::new(file);
        // I'm doing this without headers so I can grab the headers AND
        // process the data.
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(reader);
        
        for result in rdr.records() {
            let record = result?.clone();
            if num_lines == 0 {
                let h = record;
                col_count = h.len();
                data_types = Vec::with_capacity(col_count);
                headers = Vec::with_capacity(col_count);
                for n in 0..col_count {
                   data_types.push(DataTypes::EMPTY);
                   headers.push(h[n].to_string());
                }
            } else {
                let mut col_index = 0;
                for col_data in record.iter() {
                    if data_types[col_index] != DataTypes::STRING {
                        let potential_type = check_col_data_type(col_data);
                        if potential_type != data_types[col_index] {
                           data_types[col_index] = potential_type;
                        }
                    }
                    col_index += 1;
                }
            }
            num_lines += 1;
        }
        if headers.len() > 0 {
            for n in 0..data_types.len() {
                columns.push(ColumnDef::new(headers[n].clone(), data_types[n].clone()));
            }
        }
        let m = ParsedContent::new(columns);
        println!("m: {:#?}", m);
        Ok(num_lines)
    }
}

fn check_col_data_type(val: &str) -> DataTypes {
    match val.parse::<i64>() {
        Ok(_) => DataTypes::I64,
        _ => match val.parse::<f64>() {
            Ok(_) => DataTypes::F64,
            _ => DataTypes::STRING
        }  
    }
}