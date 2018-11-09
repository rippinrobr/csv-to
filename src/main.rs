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
extern crate toml;

use csv_converter::config::{Config};
use csv_converter::config::OutputCfg;
use csv_converter::code_gen::CodeGen;
use csv_converter::models::{ColumnDef, ParsedContent};
use csv_converter::parse_csv::{ParseFile};
use csv_converter::sqlite_code_gen::SqliteCodeGen;
use csv_converter::db::{SqliteDB};
use actix::*;
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
    let app_name = "csv2api";
    let default_toml_file = "csv2api.toml";
    let create_directories_flag_name = "create-directories";
    let toml_path_flag = "toml";

    let matches = clap::App::new(app_name)
        .version(clap::crate_version!())
        .about("Parses and stores Wikipedia conspiracy theories data")
        .author("Rob Rowe.")
        .arg(clap::Arg::with_name(toml_path_flag)
            .short("t")
            .long("toml")
            .value_name("PATH TO TOML FILE")
            .help("A file containing settings used during the parsing and generation processes.")
            .default_value(default_toml_file)
            .takes_value(true)
            .required(false)
            .validator(validate_fs_path))
        .arg(clap::Arg::with_name(create_directories_flag_name)
            .short("c")
            .long("create-directories")
            .help("if this flag is provided and your project directories do not exist, they will be created automatically if this flag isn't provided the user will be asked if they want the directories to be created")
            .takes_value(false)
            .required(false))   
        .get_matches();
    
    // check to see if the user wants any missing directories to be created
    let create_directories = matches.occurrences_of(create_directories_flag_name) > 0;

    // If there isn't a -t or --toml switch then go with the default file
    let toml_file_path = matches.value_of(toml_path_flag).unwrap_or("csv2api.toml");
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
    
    // flags for what needs to be created
    let create_webserver = config.gen_webserver.unwrap_or(false);
    let create_models = config.gen_models.unwrap_or(false);
    let create_sql = config.gen_sql.unwrap_or(false);
            
    match does_path_exist(&config.clone().get_project_directory_path()) {
        Err(_) => {
            let my_cfg = config.clone();
    
            if !create_directories {
                eprintln!("\nERROR: The project directory '{}' does not exist\nERROR: use the -cd or --create-directory to have {} create the directory for you", &my_cfg.get_project_directory_path(), app_name);
                std::process::exit(1);
            }

            // I will prompt the user here if they didn't provide the -c | --create-directory flag
            create_dir(&config.clone().get_project_directory_path());
    
            if create_models {
                create_dir(&config.clone().get_models_directory_path());
            }

            if create_webserver {
                create_dir(&config.clone().get_actors_directory_path());
            }

            if create_sql {
                create_dir(&my_cfg.get_db_directory_path());
            }
        },
        Ok(_) => {
            let model_path_str = &config.clone().get_models_directory_path();
            if create_models && does_path_exist(model_path_str).is_err() {
                if !create_directories {
                    eprintln!("\nERROR: The directory '{}' does not exist\nERROR: use the -cd or --create-directory to have {} create the directory for you", model_path_str, app_name);
                    std::process::exit(1);
                }
                create_dir(model_path_str);
            }

            let actors_path_str = &config.clone().get_actors_directory_path();
            if create_webserver && does_path_exist(actors_path_str).is_err() {
                if !create_directories {
                    eprintln!("\nERROR: The directory '{}' does not exist\nERROR: use the -cd or --create-directory to have {} create the directory for you", actors_path_str, app_name);
                    std::process::exit(1);
                }
                create_dir(actors_path_str);
            }
            
            let db_code_path_str = &config.clone().get_db_directory_path();
            if create_sql && does_path_exist(db_code_path_str).is_err() {
                if !create_directories {
                    eprintln!("\nERROR: The directory '{}' does not exist\nERROR: use the -cd or --create-directory to have {} create the directory for you", db_code_path_str, app_name);
                    std::process::exit(1);
                }
                create_dir(db_code_path_str);
            }
        }
    }
    
    // get the files
    let csv_files = create_files_list(&config.directories, &config.files);

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

fn does_path_exist(path_str: &str) -> Result<bool, String>{
    let project_path = Path::new(path_str);
    
    if project_path.exists() {
        if project_path.is_dir() {
            return Ok(true);
        } 
        return Err(format!("The path '{}' provided is a file, not a directory.", path_str).clone());  
    } 
    
    Err(format!("The path '{} does not exist.", path_str))
}

fn show_dir_creation_error_and_exit(e: std::io::Error) {
    eprintln!("Unable to create the project directory, Error: {}", e);
    std::process::exit(1);
}

fn create_dir(path_str: &str) {
   let path = Path::new(path_str);

   match fs::create_dir_all(path) {
        Err(e) => show_dir_creation_error_and_exit(e),
        Ok(_) => println!("Created the directory '{}'", path_str),
    };
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