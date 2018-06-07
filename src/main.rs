pub mod code_gen;
pub mod input;
pub mod models;
pub mod parse_csv;

extern crate actix;
extern crate csv;
use input::{Input, InputType};
use parse_csv::{ParseFile};

fn main() {
    let input = Input{
        input_type: InputType::CSV,
        paths: vec!["../baseballdatabank/upstream/Teams.csv".to_owned()],
    };
    println!("Hello, world! {:#?}", input);

    for path in input.paths {
        println!("path: {}", path);
        let parser = ParseFile{path:path};
        match parser.parse_file() {
            Ok(r) => println!("results: {}", r),
            Err(e) => println!("error: {}", e)
        };
    }
}
