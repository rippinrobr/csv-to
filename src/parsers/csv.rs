use failure::{Error};
use regex::Regex;
use csv::{Reader, StringRecord};

use crate::{ColumnDef, DataTypes, Input, InputSource, ParsedContent};
use super::InputService;


#[derive(Clone,Debug)]
pub struct CSVService {
    field_name_regex: Regex,
}

impl Default for CSVService {
    fn default() -> CSVService {
        CSVService {
            field_name_regex: Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]+$").unwrap(),
        }
    }
}

impl CSVService {

    fn create_column_defs(&self, headers: &StringRecord) -> Vec<ColumnDef> {
        let mut col_defs: Vec<ColumnDef> = Vec::new();

        for (n, header )in  headers.iter().enumerate() {
            let cleaned_name = self.validate_field_name(n, header, &self.field_name_regex);
            let cd = ColumnDef {
                name: cleaned_name.clone(),
                data_type: DataTypes::Empty,
                potential_types: Vec::new(),
            };
            col_defs.push(cd);
        }

        col_defs
    }

    fn check_field_data_type(val: &str) -> DataTypes {
        if val == "" {
            return DataTypes::Empty;
        }

        match val.parse::<i64>() {
            Ok(_) => DataTypes::I64,
            Err(e) => {
                match val.parse::<f64>() {
                    Ok(_) => DataTypes::F64,
                    Err(e) => {
                        return DataTypes::String
                    }
                }
            }
        }
    }
}

impl InputService for CSVService {

    fn parse(&self, input: InputSource) -> Result<ParsedContent, Error> {
        let file = input.get_reader()?;
        let mut rdr = Reader::from_reader(file);
        let mut parsed_content = ParsedContent::default();
        parsed_content.file_name = input.location;
        
        if input.has_headers {
            match rdr.headers() {
                Ok(headers) => parsed_content.columns = self.create_column_defs(headers),
                Err(e) => return Err(failure::err_msg(format!("{}", e)))
            }
        } else {
            let pos = rdr.position().clone();

            match rdr.headers() {
                Ok(headers) => {
                    let num_cols = headers.len();
                    let mut cols: Vec<String> = Vec::new();

                    for idx in 0..num_cols {
                        cols.push(format!("col_{}", idx));
                    }

                    parsed_content.columns = self.create_column_defs(&StringRecord::from(cols))
                },
                Err(e) => return Err(failure::err_msg(format!("{}", e)))
            }
            rdr.seek(pos)?;
        }

        // this loop is for the lines in a file
        for raw_record in rdr.records() {
            parsed_content.records_parsed += 1;
            let record = match raw_record {
                Ok(rec) => rec,
                Err(e) => {
                    parsed_content.errors.push(format!("{} -> parse error -> {}", &parsed_content.file_name, e));
                    continue
                }
            };
            // this loop is for the columns
            let mut col_index = 0;
            for col_data in record.clone().iter() {
                if parsed_content.columns[col_index].is_data_type_changeable() {
                    let possible_type: DataTypes = CSVService::check_field_data_type(col_data);
                    parsed_content.columns[col_index].potential_types.push(possible_type);
                }
                col_index += 1;
            }
            parsed_content.content.push(record);
        }

        &parsed_content.set_column_data_types();
        Ok(parsed_content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use crate::InputSource;

    #[test]
    fn create_column_defs_with_valid_string_record() {
        let record = StringRecord::from(vec!["alpha", "bravo", "charlie"]);
        let svc = CSVService::default();

        let col_defs = svc.create_column_defs(&record);
        assert_eq!(3, col_defs.len());
        assert_eq!(String::from("alpha"), col_defs[0].name);
        assert_eq!(DataTypes::Empty, col_defs[0].data_type);

        assert_eq!(String::from("bravo"), col_defs[1].name);
        assert_eq!(DataTypes::Empty, col_defs[1].data_type);

        assert_eq!(String::from("charlie"), col_defs[2].name);
        assert_eq!(DataTypes::Empty, col_defs[2].data_type);
    }

    #[test]
    fn create_column_defs_for_file_with_no_headers() {
        let record = StringRecord::from(vec!["", "", ""]);
        let svc = CSVService::default();

        let col_defs = svc.create_column_defs(&record);
        assert_eq!(3, col_defs.len());
        assert_eq!(String::from("col_0"), col_defs[0].name);
        assert_eq!(DataTypes::Empty, col_defs[0].data_type);

        assert_eq!(String::from("col_1"), col_defs[1].name);
        assert_eq!(DataTypes::Empty, col_defs[1].data_type);

        assert_eq!(String::from("col_2"), col_defs[2].name);
        assert_eq!(DataTypes::Empty, col_defs[2].data_type);
    }

    #[test]
    fn check_field_data_type_with_int() {
        assert_eq!(CSVService::check_field_data_type("111"), DataTypes::I64);
    }

    #[test]
    fn check_field_data_type_with_float() {
        assert_eq!(CSVService::check_field_data_type("11.1"), DataTypes::F64);
    }

    #[test]
    fn check_field_data_type_with_string() {
        assert_eq!(CSVService::check_field_data_type("rob"), DataTypes::String);
    }

    #[test]
    fn check_field_data_type_with_empty_string() {
        assert_eq!(CSVService::check_field_data_type(""), DataTypes::Empty);
    }

    #[test]
    fn parse_with_headers() {
        use std::io::Write;
        use assert_fs::prelude::*;

        let file_name = "testing_with_headers.csv";
        // Create a directory inside of `std::env::temp_dir()`.
        let tmp_dir = assert_fs::TempDir::new().unwrap();
        let file_path = tmp_dir.path().join(file_name);
        let mut tmp_file = File::create(file_path.clone()).unwrap();

        writeln!(tmp_file, "first,second,third").unwrap();
        writeln!(tmp_file, "abc,def,ghi").unwrap();
        let tmp_path = tmp_dir.into_path();

        let input_source = InputSource{
            has_headers: true,
            location: file_path.clone().into_os_string().into_string().unwrap(),
            size: 0,
        };

        let svc = CSVService::default();
        match svc.parse(input_source) {
            Ok(pc) => {
                assert_eq!(pc.columns.len(),3);
                assert_eq!(pc.content.len(), 1);
            },
            Err(e) => {
                std::fs::remove_dir_all(file_path.clone());
                panic!(format!("No Parsed content!! {}", e))
            }
        }
    }

    #[test]
    fn parse_without_headers() {
        use std::io::Write;
        use assert_fs::prelude::*;

        let file_name = "testing_with_headers.csv";
        // Create a directory inside of `std::env::temp_dir()`.
        let tmp_dir = assert_fs::TempDir::new().unwrap();
        let file_path = tmp_dir.path().join(file_name);
        let mut tmp_file = File::create(file_path.clone()).unwrap();

        writeln!(tmp_file, "abc,def,ghi").unwrap();
        let tmp_path = tmp_dir.into_path();

        let input_source = InputSource{
            has_headers: false,
            location: file_path.clone().into_os_string().into_string().unwrap(),
            size: 0,
        };

        let svc = CSVService::default();
        match svc.parse(input_source) {
            Ok(pc) => {
                assert_eq!(pc.columns.len(),3);
                if pc.columns.len() == 3 {
                    assert_eq!(pc.columns[0].name, "col_0");
                    assert_eq!(pc.columns[1].name, "col_1");
                    assert_eq!(pc.columns[2].name, "col_2");
                }
                assert_eq!(pc.content.len(), 1);
            },
            Err(e) => {
                std::fs::remove_dir_all(file_path.clone());
                panic!(format!("No Parsed content!! {}", e))
            }
        }
    }
}