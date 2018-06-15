pub mod models;
pub mod workers;

extern crate actix;
extern crate codegen;
extern crate csv;
extern crate futures;
use futures::{future, Future};
use actix::*;

//use actix::*;
use workers::input::{Input, InputType};
use workers::output::{Output};
use workers::parse_csv::{ParseFile};
use workers::code_gen::CodeGen;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut input = Input{
        input_type: InputType::CSV,
        files: vec![],
        directories: vec!["../baseballdatabank/core".to_owned()],
    };

    let output = Output::new("../tabletopbaseball_loader/src/models".to_string());
    let mut created_file_names: Vec<String> = Vec::new();

    if input.directories.len() > 0 {
        input.add_files_in_directories();
    }

    for file_path in input.files {
        let parser = ParseFile{path:file_path};
        match parser.execute() {
            Ok(results) => {
                let struct_name = results.file_name.trim_right_matches(".csv");
                let struct_string = CodeGen::generate_struct(&struct_name, results.columns);
                
                match output.write_to_file(&struct_name, struct_string) {
                    Err(e) => println!("ERROR: {}", e),
                    Ok(file_name) => {
                        println!("Created file {}.rs", file_name);
                        created_file_names.push(file_name);
                    }
                };
            },
            Err(e) => println!("error: {}", e)
        };
    }

    let mod_file_contents = CodeGen::generate_mod_file_contents(created_file_names);
    match output.write_to_file("mod", mod_file_contents) {
        Err(e) => println!("ERROR: {}", e),
        Ok(_) => println!("Created file mod.rs")
    };
}
