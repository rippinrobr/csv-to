pub mod code_gen;
pub mod models;
pub mod commands;

extern crate actix;
extern crate csv;

use commands::input::{Input, InputType};
use commands::parse_csv::{ParseFile};


fn main() {
    let input = Input{
        input_type: InputType::CSV,
        paths: vec!["../baseballdatabank/upstream/Teams.csv".to_owned()],
    };
    println!("{:#?}", input);

    for path in input.paths {
        let parser = ParseFile{path:path};
        match parser.execute() {
            Ok(raw_content) => println!("results: {:#?}", raw_content),
            Err(e) => println!("error: {}", e)
        };
    }
}
