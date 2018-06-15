extern crate csv;

use models::{DataTypes, ColumnDef};
use workers::{ParsedContent};
use csv::StringRecord;
use regex::Regex;

use std::{
    fs::File,
    io,
    io::Error,
    path::Path,
};

pub struct ParseFile {
    path: String,
    col_name_re: Regex,
}

impl ParseFile {

   pub fn new(path:String, col_name_re: Regex) -> ParseFile {
       ParseFile {
           path: path,
           col_name_re:  col_name_re,
       }
   }

   pub fn execute(&self) -> Result<ParsedContent, Error> {
        let mut num_lines: usize = 0;
        let mut headers: Vec<String> = Vec::new();
        let mut data_types: Vec<DataTypes> = Vec::new();
        let mut columns: Vec<ColumnDef> = Vec::new();
        let mut data: Vec<StringRecord> = Vec::new();
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
                   headers.push( self.check_col_name(&h[n]));
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
                data.push(record);
            }
            num_lines += 1;
        }
        if headers.len() > 0 {
            for n in 0..data_types.len() {
                columns.push(ColumnDef::new(headers[n].clone(), data_types[n].clone()));
            }
        }

        let file_path_str = self.path.clone();
        let raw_file_name = Path::new(&file_path_str).file_name().unwrap();
        let file_name = raw_file_name.to_str().unwrap();
        let content = ParsedContent::new(columns, data, String::from(file_name), num_lines);
        Ok(content)
    }

    // TODO: Need to do something if the col name is a single char long and 
    // is not a letter.
    fn check_col_name(&self, name: &str) -> String {
        if self.col_name_re.is_match(name) {
            return name.to_string()
        }

        let name_str = name.to_string();
        let mut name_chars = name_str.chars();
        let first_char: char = name_chars.next().unwrap();
        // if the 
        
        if name.len() == 1 {
            if first_char.is_alphabetic() {
                return name.to_string()
            } 

            return "INVALID_COLUMN_NAME".to_string();
        }

        println!("invalid struct field name: {}", name);
        if first_char.is_numeric() {
            return self.check_col_name(&format!("_{}", name));
        }
        
        while let Some(name_char) = name_chars.next() {
            if !name_char.is_alphanumeric() && name_char != '_' {
                return name_str.replacen(name_char, "_", 1);
            }
        }

        name_str.to_owned()
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