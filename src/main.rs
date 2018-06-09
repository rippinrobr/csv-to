pub mod models;
pub mod commands;

extern crate actix;
extern crate codegen;
extern crate csv;

use commands::input::{Input, InputType};
use commands::parse_csv::{ParseFile};
use commands::code_gen::CodeGen;

fn main() {
    let input = Input{
        input_type: InputType::CSV,
        paths: vec!["../baseballdatabank/upstream/Teams.csv".to_owned()],
    };

    for path in input.paths {
        let parser = ParseFile{path:path};
        match parser.execute() {
            Ok(raw_content) => {
                let struct_string = CodeGen::generate_struct(&raw_content.file_name.trim_right_matches(".csv"), raw_content.columns);
                println!("{}", struct_string);
            },
            Err(e) => println!("error: {}", e)
        };
    }
}
