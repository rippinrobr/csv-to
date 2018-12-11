extern crate ansi_term;
extern crate csv_converter;

use std::fs;
use std::path::{PathBuf, Path};
use std::str::FromStr;
use ansi_term::Colour::{Green, Red};
use indicatif::{ProgressBar, ProgressStyle};
use glob::{glob_with, MatchOptions};

//use self::error::DbError;
use csv_converter::models::{ColumnDef, InputSource, ParsedContent};
use crate::ports::{
    inputservice::InputService,
    configservice::ConfigService,
    storageservice::StorageService,
};

/// DbApp is used to manage the creation of the database
/// This app is used when the db sub-command is provided
pub struct DbApp<C,I,S>
where
    C: ConfigService,
    I: InputService,
    S: StorageService,
{
    config_svc: C,
    input_svc: I,
    storage_svc: S,
}

impl<C,I,S> DbApp<C,I,S>
where
    C: ConfigService,
    I: InputService,
    S: StorageService,
{
    /// creates an instance of the DbApp struct
    pub fn new(config_svc: C, input_svc: I, storage_svc: S) -> DbApp<C,I,S> {
        DbApp{
            config_svc,
            input_svc,
            storage_svc,
        }
    }

    /// execute the application logic
    pub fn run(self) -> Result<(), std::io::Error> {
        let inputs = self.config_svc.get_input_sources();
        let mut errors: Vec<String> = Vec::new();

        let pbar = ProgressBar::new(inputs.len() as u64);
        pbar.set_style(ProgressStyle::default_bar()
            .template("{prefix:.cyan/blue} {msg} [{bar:40.cyan/blue}] {pos:>3/blue}/{len:3}files")
            .progress_chars("=> "));

        pbar.set_prefix("Processing");

        let mut num_files = 0;
        for input in inputs {
            pbar.set_message(&format!("{}", &input.location));
            match self.input_svc.parse(input) {
                Err(e) => eprintln!("ERROR: {}", e),
                Ok(pc) => {
                    self.store(self.get_table_name(pc.file_name.clone()), pc.records_parsed, pc.columns.clone(), pc.content.clone());
                    pbar.inc(1)
                }
            }

            num_files += 1;
        }
        pbar.finish_and_clear();

        // Pressing report
        self.display_report(errors, num_files);

        Ok(())
    }

    fn display_report(&self, errors: Vec<String>, num_files: u64) {
        let processed_msg = format!("Processed {} files", num_files);
        println!("\n{}", Green.bold().paint(processed_msg));
        if !errors.is_empty() {
            let err_msg =format!("There were {} errors", errors.len());
            println!("{}", Red.bold().paint(err_msg));
            for e in errors {
                eprintln!("{}", e);
            }
        }
    }

    fn store(&self, name: String, records_parsed: usize, columns: Vec<ColumnDef>, content: Vec<csv::StringRecord>) -> Result<(), failure::Error> {

        return match self.storage_svc.create_store(name.clone(), columns) {
            Ok(_) => {
                self.storage_svc.store_data(name.clone(), content);
                match self.storage_svc.validate(name, records_parsed) {
                    Err(e) => Err(failure::err_msg(format!("validation error: {}", e))),
                    Ok(_) => Ok(()),
                }
            },
            Err(err) => return Err(failure::err_msg(format!("unable to create storage {}", err))),
        };
    }

    fn get_table_name(&self, file_path: String) -> String {
        // TODO: Clean this up
        let name = String::from(Path::new(&file_path).file_name().unwrap().to_str().unwrap());
        let first_letter = name.trim_right_matches(".csv").chars().next().unwrap();
        name.trim_right_matches(".csv").to_string().replace(first_letter, &first_letter.to_string().to_uppercase())
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
    drop_tables: bool,
    no_headers: bool,
}

impl Config {
    /// Creates a struct of all the CmdLine Arguments
    pub fn new(files_path: Vec<PathBuf>, directories: Vec<PathBuf>, db_type: Types, connection_info: String, name: String, drop_tables: bool, no_headers: bool) -> Config {
        Config {
            files: Config::convert_to_vec_of_string(files_path),
            directories: Config::convert_to_vec_of_string(directories),
            db_type,
            connection_info,
            name,
            drop_tables,
            no_headers,
        }
    }

    fn create_input_source(has_headers: bool, file_path: String) -> InputSource {
        let meta = fs::metadata(file_path.clone()).unwrap();
        //TODO: default has_headers to true for now, will add a flag that says --no-headers
        InputSource {
            has_headers,
            location: file_path,
            size: meta.len(),
        }
    }

    fn convert_to_vec_of_string(paths: Vec<PathBuf>) -> Vec<String> {
        let mut string_paths: Vec<String> = Vec::new();

        for p in paths.into_iter() {
            string_paths.push(p.into_os_string().into_string().unwrap_or_default());
        }

        string_paths
    }
}

impl ConfigService for Config {
    /// get_locations returns the path's to the input files
    fn get_input_sources(&self) -> Vec<InputSource> {
        let mut sources: Vec<InputSource> = Vec::new();
        // used by the glob_with call to tell it how we want to look
        // for files in a directory
        let options = &MatchOptions {
            case_sensitive: false,
            require_literal_leading_dot: false,
            require_literal_separator: false,
        };

        // directories
        for d in &self.directories {
            for f in  glob_with(&format!("{}/*.{}", d, "csv"), options).unwrap() {
                match f {
                    Ok(file_path) => sources.push(Config::create_input_source(self.has_headers(),file_path.into_os_string().into_string().unwrap_or_default()) ),
                    Err(e) => eprintln!("ERROR: {}", e),
                }
            }
        }

        // files
        for file_path in &self.files {
           sources.push(Config::create_input_source(self.has_headers(),file_path.clone()) );
        }

        sources.to_owned()
    }

    fn has_headers(&self) -> bool {
        !self.no_headers
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