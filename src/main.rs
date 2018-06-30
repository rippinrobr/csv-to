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
    sqlite_code_gen::SqliteCodeGen,
    sql_gen::SQLGen
};

// TODO: Add support for command line args and .env files
// TODO: Rename Output to be OuputProjectDir
// TODO: Refactor main to create smaller, single purpose functions
fn main() {
    let col_name_validation_re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]+$").unwrap();
    
    let mut input = Input{
        input_type: InputType::CSV,
        files: vec![], 
        directories: vec!["../baseballdatabank/core".to_owned()],
    };

    // this needs to go away until I have a better approach to loading the db as the output 
    let output = Output::new("../tabletopbaseball_loader/src".to_string(),
                            "../tabletopbaseball_loader/sql".to_string());
    
    let sql_generator = SQLGen::new("../tabletopbaseball_loader/sql".to_string());
    let sqlite_db = SqliteDB::new("../tabletopbaseball_loader/database/baseball_databank_2017.db").unwrap();

    let models_dir: &str = &(output.src_directory.clone() + "/models");
    let mut created_file_names: Vec<String> = Vec::new();
    let mut create_table_statements: Vec<String> = Vec::new();
    let mut struct_meta: Vec<(String, Vec<models::ColumnDef>)> = Vec::new();

    if input.directories.len() > 0 {
        input.add_files_in_directories();
    }

    for file_path in input.files {
        let parser = ParseFile::new(file_path, col_name_validation_re.clone());
        match parser.execute() {
            Ok(parsed_content) => {
                let tmp_struct_name = parsed_content.get_struct_name().clone();
                let struct_name = tmp_struct_name;
                struct_meta.push((struct_name.clone(), parsed_content.columns.clone()));
                let struct_string = CodeGen::generate_struct(&struct_name, &parsed_content.columns);
                
                match CodeGen::write_code_to_file(models_dir, &format!("{}.rs",struct_name), struct_string) {
                    Err(e) => eprintln!("ERROR: {}", e),
                    Ok(file_name) => {
                        println!("Created file {}.rs", file_name);
                        
                        created_file_names.push(file_name.replace(".rs", ""));
                        match SQLGen::generate_create_table(&struct_name, &parsed_content.columns) {
                            Ok(stmt) => {
                                create_table_statements.push(stmt.to_owned());
                                match sqlite_db.create_table(stmt.clone()) {
                                    Ok(_) => {
                                        println!("the table {} was created", struct_name);
                                        let stmt = SQLGen::generate_insert_stmt(&struct_name, &parsed_content.columns).unwrap();
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
        let mod_file_contents = CodeGen::generate_mod_file_contents(&created_file_names);
        match CodeGen::write_code_to_file(models_dir, "mod.rs", mod_file_contents) {
            Ok(_) => println!("Created file mod.rs"),
            Err(e) => eprintln!("ERROR: {}", e)
        };

        let db_actor_file_contents = CodeGen::generate_db_actor();
        let actors_dir: &str = &format!("{}/actors", output.src_directory);
        match CodeGen::write_code_to_file(actors_dir, "db_actor.rs", db_actor_file_contents) {
            Ok(_) => println!("Created file actors/db_actor.rs"),
            Err(e) => eprintln!("ERROR: {}", e)
        };
        
        match CodeGen::write_code_to_file(actors_dir, "mod.rs", "pub mod db_actor;".to_string()) {
            Ok(_) => println!("Created file actors/mod.rs"),
            Err(e) => eprintln!("ERROR: {}", e)
        };
        
        let main_fn_src = CodeGen::generate_webservice("./database/baseball_databank_2017.db".to_string(), &created_file_names);
        match CodeGen::write_code_to_file(&output.src_directory, "main.rs", main_fn_src) {
            Ok(_) => println!("Created file main.rs"),
            Err(e) => eprintln!("ERROR: {}", e)
        }

        let db_layer_src = SqliteCodeGen::generate_db_layer(&struct_meta);
        match CodeGen::write_code_to_file(&format!("{}/db", output.src_directory), "mod.rs", db_layer_src) {
            Ok(_) => println!("Created file db/mod.rs"),
            Err(e) => eprintln!("ERROR: {}", e)
        }

        for meta in &struct_meta {
            let actor_src = CodeGen::create_handler_actor(meta);
            let file_name = &format!("{}.rs", &meta.0.to_lowercase());
            match CodeGen::write_code_to_file(&format!("{}/actors", output.src_directory), file_name, actor_src) {
                Ok(_) => {
                    println!("Created file actors/{}", file_name);
                },
                Err(e) => eprintln!("ERROR: {}", e)
            }

        }

        match CodeGen::create_curl_script("../tabletopbaseball_loader", &created_file_names) {
            Ok(_) => println!("Created file curl_test.sh"),
            Err(e) => eprintln!("ERROR: {}", e)
        }
    }

    if create_table_statements.len() > 0 {
        match sql_generator.write_sql_to_file("schema", create_table_statements.join("\n")) {
            Ok(_) => println!("Created file schema.sql"),
            Err(e) => eprintln!("Error writing schema.sql file {}", e)
        };
    }
}
