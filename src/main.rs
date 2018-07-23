pub mod models;
pub mod workers;

extern crate actix;
extern crate barrel;
extern crate clap;
extern crate codegen;
extern crate csv;
extern crate futures;
extern crate regex;
extern crate sqlite;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use actix::*;
use actix::prelude::*;
use futures::{future, Future};
use std::fs::{self, DirEntry};
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use workers::{
    ParsedContent,
    config::Config,
    config::OutputCfg,
    output::{Output},
    parse_csv::{ParseFile},
    code_gen::{CodeGen, CodeGenStruct, CodeGenHandler, CodeGenDbActor},
    sqlite::SqliteDB,
    sqlite_code_gen::SqliteCodeGen,
    sql_gen::SQLGen
};

// WITHOUT RAYON cargo run  2.47s user 0.14s system 98% cpu 2.645 total
// WITH RAYON 

// TODO: Add support for command line args and .env files
// TODO: Rename Output to be OuputProjectDir
// TODO: Refactor main to create smaller, single purpose functions
fn main() {
    let matches = clap::App::new("csv2api")
        .version("0.0.1")
        .about("Parses and stores Wikipedia conspiracy theories data")
        .author("Rob Rowe.")
        .arg(clap::Arg::with_name("toml")
            .short("t")
            .long("toml")
            .value_name("PATH TO TOML FILE")
            .help("an alternative TOML file")
            .takes_value(true)
            .required(false))
        .get_matches();
    
    // If there isn't a -t or --toml switch then go with the default file
    let toml_file_path = matches.value_of("toml").unwrap_or("csv2api.toml");

    // processing the toml file to get the configuration values
    let mut toml_file_handle = File::open(toml_file_path).expect("csv2api.toml not found");
    let mut config_content = String::new();
    toml_file_handle.read_to_string(&mut config_content)
        .expect("something went wrong reading the config file");
    
    let config = &Config::load(&config_content);
    // println!("config: {:?}", config);
    
    // get the files
    let csv_files = create_files_list(&config.directories, &config.files);

    // flags for what needs to be created
    let create_webserver = config.gen_webserver.unwrap_or(false);
    let create_models = config.gen_models.unwrap_or(false);
    let create_sql = config.gen_sql.unwrap_or(false);

    let system = System::new("csv2api");
    let mut structs: Vec<String> = vec![];
    let mut column_meta: Vec<(String, Vec<models::ColumnDef>)> = Vec::new();

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
                    //println!("I should generate the sql");
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
        let base_dir = format!("{}/{}/src", &output.output_dir, &output.project_name.unwrap());
        let actors_dir = format!("{}/actors", base_dir);
        let db_dir = format!("{}/db", base_dir);
        let mod_src = CodeGen::generate_mod_file(&structs);
        let web_svc_code = CodeGen::generate_webservice("db placeholder".to_string(), &structs);
        let db_layer_code = SqliteCodeGen::generate_db_layer(&column_meta);

        call_code_gen_db_actor(actors_dir.clone());
        CodeGen::write_code_to_file(&db_dir, "mod.rs", db_layer_code);
        CodeGen::write_code_to_file(&base_dir, "main.rs", web_svc_code);
        CodeGen::write_code_to_file(&actors_dir, "mod.rs", format!("{}pub mod db;", mod_src));
        CodeGen::write_code_to_file(&format!("{}/models", base_dir), "mod.rs", mod_src);
    }

    system.run();
}

fn call_code_gen_struct_actor(output_cfg: OutputCfg, parsed_content: ParsedContent) {
    let addr = CodeGen.start();
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
    let addr = CodeGen.start();
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
    let addr = CodeGen.start();
    let file_name = "db.rs".to_string();
    let dir_path = db_dir.clone();

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

//     // TODO: Add to Output.Sqlite config
//     //let sqlite_db_path = "../hockey-db/database/baseball_databank_2017.db";
//     // TODO: Remove this and use the value in config if it exists
//     let sqlite_db_path = "../hockey-db/database/hockey_databank_2017.db";
    
//     let output = Output::new("../hockey-db/src".to_string(),
//                             "../hockey-db/sql".to_string());    
//     // TODO: Move this into the section where I know that SQL is to be generated
//     let sql_generator = SQLGen::new("../hockey-db/sql".to_string());
//     // let sqlite_db = SqliteDB::new(sqlite_db_path).unwrap();


//     let models_dir: &str = &(output.src_directory.clone() + "/models");
//     // let mut created_file_names: Vec<String> = Vec::new();
//     // let mut create_table_statements: Vec<String> = Vec::new();
//     let mut column_meta: Vec<(String, Vec<models::ColumnDef>)> = Vec::new();


// //    let files = config.get_files();
//     for file_path in files {
//         let parser = ParseFile::new(file_path.to_string());
//         match parser.execute() {
//             Ok(parsed_content) => {
                
//                 if config.output.gen_models.unwrap_or(false) {
//                     let cfg = config.clone();
//                     //println!("{:#?}", cfg.output);
//                     let models_dir = &cfg.output.code_gen.unwrap().models_dir();
//                     //create_models(&parsed_content, &models_dir); 
//                     let struct_name = parsed_content.get_struct_name().clone();                    
//                     let struct_string = CodeGen::generate_struct(&struct_name, &parsed_content.columns);
//                     match CodeGen::write_code_to_file(models_dir, &format!("{}.rs",struct_name), struct_string) {
//                         Err(e) => eprintln!("ERROR: {} [models_dir: {} file: {}]", e, models_dir, file_path),
//                         Ok(file_name) => println!("Created file {}", file_name)
//                     }
//                 }
//             },
//             Err(e) => {
//                 println!("ERROR: {} for file {} ", e, file_path);
//             }
//         }
//     }
//     //             let tmp_struct_name = parsed_content.get_struct_name().clone();
//     //             let struct_name = tmp_struct_name;
//     //             column_meta.push((struct_name.clone(), parsed_content.columns.clone()));

//                 //let _cfg = config.to_owned();
//                 // if cfg.should_gen_models() {
//                 //     let struct_string = CodeGen::generate_struct(&struct_name, &parsed_content.columns);
//                 //     match CodeGen::write_code_to_file(models_dir, &format!("{}.rs",struct_name), struct_string) {
//                 //         Err(e) => eprintln!("ERROR: {}", e),
//                 //         Ok(file_name) => println!("Created file {}", file_name)
//                 //     }
//                 // }

//                 // COMMENTED OUT WHILE I WORK ON MOVING THE 
//                 // let struct_string = CodeGen::generate_struct(&struct_name, &parsed_content.columns);
//                 // match CodeGen::write_code_to_file(models_dir, &format!("{}.rs",struct_name), struct_string) {
//                 //     Err(e) => eprintln!("ERROR: {}", e),
//                 //     Ok(file_name) => {
//                 //         println!("Created file {}", file_name);
                        
//                 //         // TODO: instead of writing out the files at the end I want to write them out as the code is generated
//                 //         // at lest for the models
//                 //         created_file_names.push(file_name.replace(".rs", ""));

//                 //         // TODO: Move the sqlite generation out to its own match.  Eventually this will be its own
//                 //         // actor
//                 //         match SQLGen::generate_create_table(&struct_name, &parsed_content.columns) {
//                 //             Ok(stmt) => {
//                 //                 create_table_statements.push(stmt.to_owned());
//                 //                 match sqlite_db.create_table(stmt.clone()) {
//                 //                     Ok(_) => {
//                 //                         println!("the table {} was created", struct_name);
//                 //                         let stmt = SQLGen::generate_insert_stmt(&struct_name, &parsed_content.columns).unwrap();
//                 //                         match sqlite_db.insert_rows(stmt, &parsed_content.columns, parsed_content.content_to_string_vec().unwrap()) {
//                 //                             Ok(num_inserted) => println!("{} records insert into {}", num_inserted, struct_name),
//                 //                             Err(e) => eprintln!("ERROR: {} inserting record into {}", e, struct_name)
//                 //                         }
//                 //                     },
//                 //                     Err(e) => eprintln!("ERROR: there was a problem creating the table {}: {}", struct_name, e)
//                 //                 };
//                 //             },
//                 //             Err(e) => eprintln!("[Main.generate_create_table] Error: {}", e)
//                 //         };
//                 //     }
//                 // };
//            // },
//            //  Err(e) => println!("error: {}", e);
//     //     };
//     // }

//     // TODO: Change this if so that it keys off of a config value. This section may not be needed 
//     // or if it is there will be a smaller amount of code here
//     // if created_file_names.len() > 0 {
//     //     let mod_file_contents = CodeGen::generate_mod_file_contents(&created_file_names);
//     //     match CodeGen::write_code_to_file(models_dir, "mod.rs", mod_file_contents) {
//     //         Ok(_) => println!("Created file mod.rs"),
//     //         Err(e) => eprintln!("ERROR: {}", e)
//     //     };

//     //     let db_actor_file_contents = CodeGen::generate_db_actor();
//     //     let actors_dir: &str = &format!("{}/actors", output.src_directory);
//     //     match CodeGen::write_code_to_file(actors_dir, "db_actor.rs", db_actor_file_contents) {
//     //         Ok(_) => println!("Created file actors/db_actor.rs"),
//     //         Err(e) => eprintln!("ERROR: {}", e)
//     //     };

//     //     println!("INFO: actors_dir: {}", actors_dir);
//     //     let actor_mod_file_str = CodeGen::generate_mod_file(actors_dir);
//     //     match CodeGen::write_code_to_file(actors_dir, "mod.rs", actor_mod_file_str) {
//     //         Ok(_) => println!("Created file mod.rs"),
//     //         Err(e) => eprintln!("ERROR: creating actors/mod.rs {}", e)
//     //     };

//     //     let main_fn_src = CodeGen::generate_webservice(sqlite_db_path.to_string(), &created_file_names);
//     //     match CodeGen::write_code_to_file(&output.src_directory, "main.rs", main_fn_src) {
//     //         Ok(_) => println!("Created file main.rs"),
//     //         Err(e) => eprintln!("ERROR: {}", e)
//     //     }

//     //     let db_layer_src = SqliteCodeGen::generate_db_layer(&column_meta);
//     //     match CodeGen::write_code_to_file(&format!("{}/db", output.src_directory), "mod.rs", db_layer_src) {
//     //         Ok(_) => println!("Created file db/mod.rs"),
//     //         Err(e) => eprintln!("ERROR: {}", e)
//     //     }

//     //     for meta in &column_meta {
//     //         let actor_src = CodeGen::create_handler_actor(meta);
//     //         let file_name = &format!("{}.rs", &meta.0.to_lowercase());
//     //         match CodeGen::write_code_to_file(&format!("{}/actors", output.src_directory), file_name, actor_src) {
//     //             Ok(_) => {
//     //                 println!("Created file actors/{}", file_name);
//     //             },
//     //             Err(e) => eprintln!("ERROR: {}", e)
//     //         }

//     //     }

//     //     match CodeGen::create_curl_script("../tabletopbaseball_loader", &created_file_names) {
//         //     Ok(_) => println!("Created file curl_test.sh"),
//         //     Err(e) => eprintln!("ERROR: {}", e)
//         // }
//     //}

//     // if create_table_statements.len() > 0 {
//     //     match sql_generator.write_sql_to_file("schema", create_table_statements.join("\n")) {
//     //         Ok(_) => println!("Created file schema.sql"),
//     //         Err(e) => eprintln!("Error writing schema.sql file {}", e)
//     //     };
//     // }
// }

// fn create_models(parsed_content: &ParsedContent, model_dir_path: &str ) {
//     let struct_name = parsed_content.get_struct_name().clone();
//     let struct_string = CodeGen::generate_struct(&struct_name, &parsed_content.columns);
    
//     match CodeGen::write_code_to_file(model_dir_path, &format!("{}.rs",struct_name), struct_string) {
//         Err(e) => eprintln!("ERROR: {}", e),
//         Ok(file_name) => println!("Created file {}", file_name)
//     }
// }