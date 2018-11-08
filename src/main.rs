// pub mod models;
pub mod actors;

extern crate csv_converter;
extern crate actix;
extern crate barrel;
extern crate clap;
extern crate codegen;
extern crate csv;
extern crate futures;
extern crate regex;
extern crate sqlite;
// #[macro_use]
//extern crate serde_derive;
extern crate toml;

use csv_converter::config::{Config, OutputCfg};
use csv_converter::code_gen::CodeGen;
use csv_converter::models::{ColumnDef, ParsedContent};
use csv_converter::parse_csv::{ParseFile};
use csv_converter::sqlite_code_gen::SqliteCodeGen;
use csv_converter::db::{SqliteDB};
use actix::*;
//use actix::prelude::*;
use futures::{future, Future};
use std::fs::{self};
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use actors::{
    code_gen::{ Generator, CodeGenStruct, CodeGenHandler, CodeGenDbActor},
    sqlite::{SQLGen, SqliteCreateTable}
};

fn main() {
    let default_toml_file = "csv2api.toml";

    let matches = clap::App::new("csv2api")
        .version(clap::crate_version!())
        .about("Parses and stores Wikipedia conspiracy theories data")
        .author("Rob Rowe.")
        .arg(clap::Arg::with_name("toml")
            .short("t")
            .long("toml")
            .value_name("PATH TO TOML FILE")
            .help("A file containing settings used during the parsing and generation processes.")
            .default_value(default_toml_file)
            .takes_value(true)
            .required(false)
            .validator(validate_fs_path))
        .get_matches();
    
    // If there isn't a -t or --toml switch then go with the default file
    let toml_file_path = matches.value_of("toml").unwrap_or("csv2api.toml");
    if toml_file_path == default_toml_file {
        match validate_fs_path(toml_file_path.to_string()) {
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            },
            _ => ()
        }
    }
    // processing the toml file to get the configuration values
    let mut toml_file_handle = File::open(toml_file_path).expect("csv2api.toml not found");
    let mut config_content = String::new();
    
    toml_file_handle.read_to_string(&mut config_content)
        .expect("something went wrong reading the config file");
    
    let config = &Config::load(&config_content);
    // get the files
    let csv_files = create_files_list(&config.directories, &config.files);

    // flags for what needs to be created
    let create_webserver = config.gen_webserver.unwrap_or(false);
    let create_models = config.gen_models.unwrap_or(false);
    let create_sql = config.gen_sql.unwrap_or(false);

    let system = System::new("csv2api");
    let mut structs: Vec<String> = vec![];
    let mut column_meta: Vec<(String, Vec<ColumnDef>)> = Vec::new();

    // process the files
    for file in csv_files.iter() {
        let parser = ParseFile::new(file.clone());
        
        match parser.execute() {
            Ok(parsed_content) => {
                if create_models {
                    call_code_gen_struct_actor(config.output.clone(), parsed_content.clone());
                }

                if create_webserver {
                    call_code_gen_handler_actor(config.output.clone(), parsed_content.clone());
                }

                if create_sql {
                    sqlite_load_table(config.clone(), &parsed_content.file_name.replace(".csv", ""), parsed_content.columns.clone(), parsed_content.content_to_string_vec().unwrap());
                }

                structs.push(parsed_content.get_struct_name());
                column_meta.push((parsed_content.get_struct_name(), parsed_content.columns));
            },
            Err(e) => {
                println!("ERROR: Parsing {} threw {}", file, e);
            }
        }
    }
    
    if create_webserver {
        let output = config.output.clone();
        let output_db_uri = config.output_db.db_uri.clone();
        let base_dir = format!("{}/{}/src", &output.output_dir, &output.project_name.unwrap());
        let actors_dir = format!("{}/actors", base_dir);
        let db_dir = format!("{}/db", base_dir);
        let mod_src = CodeGen::generate_mod_file(&structs);
        let web_svc_code = CodeGen::generate_webservice(output_db_uri.unwrap(), &structs);
        let db_layer_code = SqliteCodeGen::generate_db_layer(&column_meta);

        call_code_gen_db_actor(actors_dir.clone());
        match csv_converter::write_code_to_file(&db_dir, "mod.rs", db_layer_code) {
            Ok(msg) => println!("{}", msg),
            Err(e) => {
                eprintln!("Error while creating {}/mod.rs file. {}",db_dir, e);
                std::process::exit(1);
            },
        };

        match csv_converter::write_code_to_file(&base_dir, "main.rs", web_svc_code) {
            Ok(msg) => println!("{}", msg),
            Err(e) => {
                eprintln!("Error while creating {}/main.rs file. {}",base_dir, e);
                std::process::exit(1);
            }
        };

        match csv_converter::write_code_to_file(&actors_dir, "mod.rs", format!("{}pub mod db;", mod_src)) {
            Ok(msg) => println!("{}", msg),
            Err(e) => {
                eprintln!("Error while creating {}/main.rs file. {}", actors_dir, e);
                std::process::exit(1);
            }
        }

        match csv_converter::write_code_to_file(&format!("{}/models", base_dir), "mod.rs", mod_src.clone()) {
            Ok(msg) => println!("{}", msg),
            Err(e) => {
                eprintln!("Error while creating {}/models/mod.rs file. {}", mod_src.clone(), e);
                std::process::exit(1);
            }
        }
    }

    system.run();
}

fn validate_fs_path(path: String) -> Result<(), String> {
    let given_path = Path::new(&path);
    if !given_path.exists() {
      return Err(format!("The path '{}' does not exist.", path))
    }
    
    if given_path.is_dir() {
        return Err(format!("The path '{}' is a directory, not a *.toml file.", path))
    } 

    Ok(())
}

fn call_code_gen_struct_actor(output_cfg: OutputCfg, parsed_content: ParsedContent) {
    let addr = Generator.start();
    let output_dir = &output_cfg.output_dir;
    let project_name = output_cfg.project_name.unwrap_or("".to_string());
    
    let res = addr.send(CodeGenStruct{
        struct_name: parsed_content.get_struct_name(),
        models_dir: format!("{}/{}/src/models", output_dir, project_name),
        parsed_content: parsed_content});

    Arbiter::spawn(res.then(|res| {
        match res {
            Ok(struct_src) => println!("{}", struct_src),
            _ => println!("Something wrong"),
        }
        
        System::current().stop();
        future::result(Ok(()))
    }));
}

fn call_code_gen_handler_actor(output_cfg: OutputCfg, parsed_content: ParsedContent) {
    let addr = Generator.start();
    let output_dir = &output_cfg.output_dir;
    let project_name = output_cfg.project_name.unwrap_or("".to_string());
    
    let res = addr.send(CodeGenHandler{
        struct_name: parsed_content.get_struct_name(),
        actors_dir: format!("{}/{}/src/actors", output_dir, project_name),
        parsed_content: parsed_content});

    Arbiter::spawn(res.then(|res| {
        match res {
            Ok(struct_src) => println!("{}", struct_src),
            _ => println!("Something wrong"),
        }
        
        System::current().stop();
        future::result(Ok(()))
    }));
}

fn call_code_gen_db_actor(db_dir: String) {
    let addr = Generator.start();
    let file_name = "db.rs".to_string();
    
    let res = addr.send(CodeGenDbActor{
        db_src_dir: db_dir.clone(),
        file_name: file_name,
    });
    
    Arbiter::spawn(res.then(|res| {
        match res {
            Ok(_) => println!("Created the db actor"),
            _ => println!("Something wrong"),
        }
        
        System::current().stop();
        future::result(Ok(()))
    }));
}

fn sqlite_load_table(cfg: Config, table_name: &str, columns: Vec<ColumnDef>, contents: Vec<Vec<String>>) {
    let addr = SQLGen.start();
    let tname = table_name.to_string().clone();
    let db_uri = cfg.output_db.db_uri.unwrap();
    let db_conn = SqliteDB::new(&db_uri).unwrap();
    
    let res = addr.send(SqliteCreateTable{
        columns: columns.clone(),
        db_conn: db_conn,
        table_name: table_name.to_string(),
    });

    
    Arbiter::spawn(res.then( move |res| {
        match res {
            // now that the table is created I can insert the data
            Ok(msg) => {
                match msg {
                    Ok(_) => {
                        let writer_dbconn = SqliteDB::new(&db_uri).unwrap();
                        let stmt = SQLGen::generate_insert_stmt(&tname, &columns.clone()).unwrap();
                        
                        match writer_dbconn.insert_rows(stmt, &columns, contents) {
                            Ok(num_inserted) => println!("{} records insert into {}", num_inserted, tname),
                            Err(e) => eprintln!("ERROR: {} inserting record into {}", e, tname)
                        }
                    },
                    Err(e) => {
                        eprintln!("ERROR: {}", e)
                    }
                }
            }
            ,
            _ => unreachable!(),
        }
        
        System::current().stop();
        future::result(Ok(()))
    }));
}

fn create_files_list(dirs: &Vec<String>, cfg_files: &Vec<String>) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();

    for d in dirs {
        let path = Path::new(&d);
        if path.is_dir() {
            let paths = fs::read_dir(path).unwrap();
            for entity in paths {
                let entity_path = entity.unwrap().path();
                if entity_path.is_file() {
                    let ep_str = entity_path.display().to_string();
                    if !ep_str.ends_with("csv") && !ep_str.ends_with("CSV") {
                        continue;
                    }

                    files.push(ep_str)
                }
            }
        }
    }

    for f in cfg_files {
        if !files.contains(f) {
            files.push(f.to_string());
        }
        
    }

    files.to_owned()
}