extern crate csv;

use super::models::{DataTypes, ColumnDef, ParsedContent};
use csv::StringRecord;
use regex::Regex;
use std::collections::HashSet;

use std::{
    fs::File,
    io,
    io::Error,
    path::Path,
};

pub struct ParseFile {
    path: String,
    col_name_re: Regex,
    rust_keywords: HashSet<String>,
}

// TODO: write errors out to stderr  
// FIXME: Find a better way of doing keyword checks
impl ParseFile {
    pub fn new(path:String) -> ParseFile {
        let mut rust_keywords: HashSet<String> = HashSet::new();

        for kw in vec!["as".to_string(), "break".to_string(), "const".to_string(), "continue".to_string(), "crate".to_string(), 
            "else".to_string(), "enum".to_string(),"extern".to_string(), "false".to_string(), "fn".to_string(), "for".to_string(),
            "for".to_string(), "if".to_string(), " impl".to_string(), "in".to_string(), "let".to_string(), "loop".to_string(),
            "match".to_string(), "mod".to_string(), "move".to_string(), "mut".to_string(), "pub".to_string(), "ref".to_string(),
            "return".to_string(), "Self".to_string(), "self".to_string(), "static".to_string(), "struct".to_string(), 
            "super".to_string(), "trait".to_string(), "true".to_string(), "type".to_string(), "unsafe".to_string(), "use".to_string(),
            "where".to_string(), "while".to_string(), "abstract".to_string(), "alignof".to_string(), "become".to_string(), 
            "box".to_string(), "do".to_string(), "final".to_string(), "macro".to_string(), "offsetof".to_string(), "override".to_string(),
            "priv".to_string(), "proc".to_string(), "sizeof".to_string(), "typeof".to_string(), "unsized".to_string(), 
            "virtual".to_string(), "yield".to_string()] {
                rust_keywords.insert(kw);
            }
        
        ParseFile {
            path: path,
            col_name_re:  Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]+$").unwrap(),
            rust_keywords: rust_keywords,
        }
    }

    pub fn execute(&self) -> Result<ParsedContent, Error> {
        let mut num_lines: usize = 0;
        let mut headers: Vec<String> = Vec::new();
        let mut data_types: Vec<DataTypes> = Vec::new();
        let mut columns: Vec<ColumnDef> = Vec::new();
        let mut data: Vec<StringRecord> = Vec::new();
        
        // Build the CSV reader and iterate over each record.
        let file = File::open(&self.path)?;
        let reader = io::BufReader::new(file);
        // I'm doing this without headers so I can grab the headers AND
        // process the data.  If true then the header row is skipped apparently
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(reader);
        
        for result in rdr.records() {
            let record = result?.clone();
            if num_lines == 0 {
                let h = record;
                let col_count = h.len();
                data_types = Vec::with_capacity(col_count);
                headers = Vec::with_capacity(col_count);
                for n in 0..col_count {
                    data_types.push(DataTypes::Empty);
                    let col_name = self.check_col_name(&h[n]);
                    headers.push(col_name.to_owned());
                }
            } else {
                let mut col_index = 0;
                for col_data in record.iter() {
                    if col_data == "".to_string() {
                        continue;
                    }

                    if data_types[col_index] != DataTypes::String && data_types[col_index] != DataTypes::F64 {
                        let potential_type = check_col_data_type(col_data);
                        if potential_type != data_types[col_index] {
                            if data_types[col_index] == DataTypes::Empty {
                                // I'm not convinced I won't need this for more debugging so I'm going to leave it here for a bit.
                                // println!("[{}] => data_types[col_index]: '{:#?}'\tpotential_type: {:#?}", headers[col_index], data_types[col_index], potential_type);
                                data_types[col_index] = potential_type;
                            }
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
    // FIXME: CLean this mess up
    fn check_col_name(&self, name: &str) -> String {
        if self.rust_keywords.contains(&name.to_lowercase()) {
            return format!("_{}", name);
        }

        if self.col_name_re.is_match(name) {
            return name.to_string()
        }

        let name_str = name.to_string();
        let mut name_chars = name_str.chars();
        let first_char: char = name_chars.next().unwrap();
        
        if name.len() == 1 {
            if first_char.is_alphabetic() {
                return name.to_string()
            } 

            if first_char == '+' {
                return self.check_col_name(&name_str.replace(first_char, "plus"));
            }

            if first_char == '-' {
                return self.check_col_name(&name_str.replace(first_char, "minus"));
            }

            return "INVALID_COLUMN_NAME".to_string();
        }

        eprintln!("invalid struct field name: {}, fixing", name);
        if first_char == '+' {
            return self.check_col_name(&name_str.replace(first_char, "plus"));
        }

        if first_char == '-' {
            return self.check_col_name(&name_str.replace(first_char, "minus"));
        }

        if first_char.is_numeric() {
            return self.check_col_name(&format!("_{}", name));
        }

        while let Some(name_char) = name_chars.next() {
            if !name_char.is_alphanumeric() && name_char != '_' {
                if name_char == '+' {
                    return self.check_col_name(&name_str.replace(name_char, "plus"));
                }

                if name_char == '-' {
                    return self.check_col_name(&name_str.replace(name_char, "minus"));
                }

                if name_char == '/' {
                    return self.check_col_name(&name_str.replace(name_char, ""));
                }
                
                return self.check_col_name(&name_str.replace(name_char, "_"));
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
            _ => DataTypes::String
        }  
    }
}