pub mod models;
pub mod workers;

extern crate barrel;
extern crate codegen;
extern crate csv;
extern crate regex;
extern crate sqlite;

use regex::Regex;
use workers::{
    input::{Input, InputType},
    output::{Output},
    parse_csv::{ParseFile},
    code_gen::CodeGen,
    sqlite::SqliteDB,
    sql_gen::SQLGen
};

// TODO: Add support for command line args and .env files
// TODO: Rename Output to be RustSrcDir
// TODO: Refactor main to create smaller, single purpose functions
fn main() {
    let col_name_validation_re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]+$").unwrap();
    let mut input = Input{
        input_type: InputType::CSV,
        files: vec![], //"../baseballdatabank/core/AwardsSharePlayers.csv".to_string()],
        directories: vec!["../baseballdatabank/core".to_owned()],
    };
    let output = Output::new("../tabletopbaseball_loader/src/models".to_string(), "../tabletopbaseball_loader/sql".to_string(), "../tabletopbaseball_loader/database".to_string());
    let sqlite_db = SqliteDB::new("../tabletopbaseball_loader/baseballdatabank_2017.db").unwrap();

    let mut created_file_names: Vec<String> = Vec::new();
    let mut create_table_statements: Vec<String> = Vec::new();

    if input.directories.len() > 0 {
        input.add_files_in_directories();
    }

    for file_path in input.files {
        let parser = ParseFile::new(file_path, col_name_validation_re.clone());
        match parser.execute() {
            Ok(parsed_content) => {
                let struct_name = parsed_content.file_name.trim_right_matches(".csv");
                let struct_string = CodeGen::generate_struct(&struct_name, &parsed_content.columns);
                
                match output.write_code_to_file(&struct_name, struct_string) {
                    Err(e) => eprintln!("ERROR: {}", e),
                    Ok(file_name) => {
                        println!("Created file {}.rs", file_name);
                        
                        created_file_names.push(file_name);
                        match SQLGen::generate_create_table(struct_name, &parsed_content.columns) {
                            Ok(stmt) => {
                                create_table_statements.push(stmt.to_owned());
                                match sqlite_db.create_table(stmt.clone()) {
                                    Ok(_) => {
                                        println!("the table {} was created", struct_name);
                                        let stmt = SQLGen::generate_insert_stmt(struct_name, &parsed_content.columns).unwrap();
                                        match sqlite_db.insert_rows(stmt, &parsed_content.columns, parsed_content.content_to_string_vec().unwrap()) {
                                            Ok(num_inserted) => println!("{} records insert into {}", num_inserted, struct_name),
                                            Err(e) => eprintln!("ERROR: {} inserting record into {}", e, struct_name)
                                        }
                                    },
                                    Err(e) => eprintln!("ERROR: there was a problem creating the table {}: {}", struct_name, e)
                                };
                            },
                            Err(e) => eprintln!("[Main.generate_create_table] Error: {}", e)
                        };
                    }
                };
            },
            Err(e) => println!("error: {}", e)
        };
    }

    if created_file_names.len() > 0 {
        let mod_file_contents = CodeGen::generate_mod_file_contents(created_file_names);
        match output.write_code_to_file("mod", mod_file_contents) {
            Ok(_) => println!("Created file mod.rs"),
            Err(e) => eprintln!("ERROR: {}", e)
        };
    }

    if create_table_statements.len() > 0 {
        match output.write_sql_to_file("schema", create_table_statements.join("\n")) {
            Ok(_) => println!("Created file schema.sql"),
            Err(e) => eprintln!("Error writing schema.sql file {}", e)
        };
    }
}
