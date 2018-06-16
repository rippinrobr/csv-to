pub mod models;
pub mod workers;

extern crate actix;
extern crate barrel;
extern crate codegen;
extern crate csv;
extern crate futures;
extern crate regex;

use regex::Regex;
use workers::{
    input::{Input, InputType},
    output::{Output},
    parse_csv::{ParseFile},
    code_gen::CodeGen,
    sql_gen::SQLGen
};

// TODO: Add support for command line args and .env files
// TODO: Rename Output to be RustSrcDir
fn main() {
    let col_name_validation_re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]+$").unwrap();
    let mut input = Input{
        input_type: InputType::CSV,
        files: vec![],
        directories: vec!["../baseballdatabank/core".to_owned()],
    };

    let output = Output::new("../tabletopbaseball_loader/src/models".to_string(), "../tabletopbaseball_loader/sql".to_string());
    let mut created_file_names: Vec<String> = Vec::new();
    let mut create_table_statements: Vec<String> = Vec::new();

    if input.directories.len() > 0 {
        input.add_files_in_directories();
    }

    for file_path in input.files {
        let parser = ParseFile::new(file_path, col_name_validation_re.clone());
        match parser.execute() {
            Ok(results) => {
                let struct_name = results.file_name.trim_right_matches(".csv");
                let struct_string = CodeGen::generate_struct(&struct_name, &results.columns);
                
                match output.write_code_to_file(&struct_name, struct_string) {
                    Err(e) => println!("ERROR: {}", e),
                    Ok(file_name) => {
                        println!("Created file {}.rs", file_name);
                        created_file_names.push(file_name);
                        match SQLGen::generate_create_table(struct_name, &results.columns) {
                            Ok(stmt) => create_table_statements.push(stmt),
                            Err(e) => eprintln!("[Main.generate_create_table] Error: {}", e)
                        };
                    }
                };


            },
            Err(e) => println!("error: {}", e)
        };
    }

    let mod_file_contents = CodeGen::generate_mod_file_contents(created_file_names);
    match output.write_code_to_file("mod", mod_file_contents) {
        Err(e) => println!("ERROR: {}", e),
        Ok(_) => println!("Created file mod.rs")
    };

    println!("number of create table statements: {}", create_table_statements.len());
    match output.write_sql_to_file("schema", create_table_statements.join("\n")) {
        Ok(_) => println!("Created file schema.sql"),
        Err(e) => println!("Error writing schema.sql file {}", e)
    };
}
