extern crate csv_converter;

use std::path::PathBuf;
use std::str::FromStr;
//use failure::Fail;
use std::fs;
//use self::error::DbError;
use glob::{glob_with, MatchOptions};
use crate::ports::{
    inputservice::{InputService, InputSource},
    configservice::ConfigService,
};

/// DbApp is used to manage the creation of the database
/// This app is used when the db sub-command is provided
pub struct DbApp<C,I>
where
    C: ConfigService,
    I: InputService
{
    config_svc: C,
    input_svc: I,
}

impl<C,I> DbApp<C,I>
where
    C: ConfigService,
    I: InputService
{
    /// creates an instance of the DbApp struct
    pub fn new(config_svc: C, input_svc: I) -> DbApp<C,I> {
        DbApp{
            config_svc,
            input_svc,
        }
    }
}

/// Config contains all the parameters provided by the user
#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    directories: Vec<String>,
    db_type: Types,
    connection_info: String,
    name: String,
}

impl Config {
    /// Creates a struct of all the CmdLine Arguments
    pub fn new(files_path: Vec<PathBuf>, directories: Vec<PathBuf>, db_type: Types, connection_info: String, name: String) -> Config {
        Config {
            files: Config::convert_to_vec_of_string(files_path),
            directories: Config::convert_to_vec_of_string(directories),
            db_type,
            connection_info,
            name,
        }
    }

    /// get_locations returns the path's to the input files
    pub fn get_input_sources(self) -> Vec<InputSource> {
        let mut sources: Vec<InputSource> = Vec::new();
        let options = &MatchOptions {
            case_sensitive: false,
            require_literal_leading_dot: false,
            require_literal_separator: false,
        };

        // directories
        for d in self.directories {
            for f in  glob_with(&format!("{}/*.{}", d, "csv"), options).unwrap() {
                match f {
                    Ok(file_path) => sources.push(Config::create_input_source(file_path.into_os_string().into_string().unwrap_or(String::new())) ),
                    Err(e) => eprintln!("ERROR: {}", e),
                }
            }
        }

        // files
        for file_path in self.files {
           sources.push(Config::create_input_source(file_path) );
        }

        sources.to_owned()
    }

    fn create_input_source(file_path: String) -> InputSource {
        let meta = fs::metadata(file_path.clone()).unwrap();
        InputSource {
            location: file_path,
            size: meta.len(),
            columns: Vec::new(),
            content: Vec::new(),
        }
    }

    fn convert_to_vec_of_string(paths: Vec<PathBuf>) -> Vec<String> {
        let mut string_paths: Vec<String> = Vec::new();

        for p in paths.into_iter() {
            string_paths.push(p.into_os_string().into_string().unwrap_or(String::new()));
        }

        string_paths
    }
}

#[derive(Debug, Clone)]
pub enum Types {
    SQLite,
}

impl FromStr for Types {
    type Err = error::DbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower_s: &str = &s.to_lowercase();
        match lower_s {
            "sqlite" => Ok(Types::SQLite),
            _ => Err(error::DbError::new(format!("ERROR: '{}' is not a supported database type", lower_s), exitcode::USAGE))
        }
    }
}

pub mod error {
    use failure::Fail;

    #[derive(Fail, Debug)]
    #[fail(display = "{}", msg)]
    pub struct DbError {
        msg: String,
        exit_code: exitcode::ExitCode,
    }

    impl DbError {
        pub fn get_exit_code(&self) -> exitcode::ExitCode {
            self.exit_code
        }

        pub fn get_msg(&self) -> String {
            self.msg.clone()
        }

        pub fn new(msg: String, exit_code: exitcode::ExitCode) -> DbError {
            DbError { msg, exit_code }
        }
    }
}
//pub fn run(files: &Vec<PathBuf>, db_type: Types, connection_info: &str, name: &str) -> Result<(), error::DbError> {
//    // 0. Validate parameters, make sure name isn't empty, connection_info and at least one file
//    if name == "" {
//        return Err(DbError::new(format!("ERROR: the name argument cannot be empty"), exitcode::USAGE))
//    }
//
//    if connection_info == "" {
//        return Err(DbError::new(format!("ERROR: the name connection_info cannot be empty"), exitcode::USAGE))
//    }
//
//    let _ = get_paths(files);
//
//    // for fpath in files.into_iter() {
//    //   // 1. get file stats & open it
//    //   // TODO: Fix this so it handles the error and does NOT use unwrap()
//    //   let metadata = fs::metadata(fpath).unwrap();
//    //   //println!("{:?}", fpath);
//    //   if metadata.is_dir() {
//    //       println!("I'm a directory!");
//    //       let glob_path = format!("{:?}/*{}", fpath, ".csv");
//    //       match run(&create_pathbuf_vec(glob(&glob_path).unwrap()), db_type.clone(), connection_info, name) {
//    //         Ok(files) => println!("files: {:?}", files),
//    //         Err(e) => eprintln!("{}", e)
//    //       };
//    //       continue;
//    //   }
//    //   // 2. parse a file return the columns and their definitions along with other file related stats
//    //   // 3. Create Table Schema
//    // }
//    println!("all done!");
//    Ok(())
//}
//
//fn get_paths(files: &Vec<PathBuf>) -> Vec<csv_converter::input::InputFile> {
//    println!("get_paths!!!");
//    let inputs: Vec<csv_converter::input::InputFile> = Vec::new();
//    println!("files: {:?}", files);
//    for file in files.into_iter() {
//        let metadata = fs::metadata(file).unwrap();
//        if metadata.is_file() {
//            println!("pretend I created an InputFile struct for {:?}", file);
//        } else {
//            // TODO: Try the appeneding of the wildcard here and see if that works better
//            let dir_path = format!("{:?}/*{}", file, ".csv");
//            println!("dir_path: '{:?}'", dir_path);
//            let more_files = get_paths(files);
//            println!("more_files: {:?}", more_files);
//        }
//
//    }
//    inputs
//}
//
//fn create_pathbuf_vec(paths: glob::Paths) -> Vec<PathBuf> {
//    let mut files: Vec<PathBuf> = Vec::new();
//
//    for file in paths {
//        match file {
//            Ok(f) => {
//                println!("file: {:?}", f);
//                // let raw_path = file.unwrap();
//                // if raw_path.metadata().unwrap().is_file() {
//                files.push(f);
//            },
//            Err(e) => eprintln!("ERROR: {}", e)
//        }
//    }
//
//    files
//}
//
//fn convert_pathbuf_to_string(p: PathBuf) -> String {
//    p.into_os_string().into_string().unwrap()
//}
