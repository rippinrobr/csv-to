extern crate ansi_term;

pub mod config;

use std::path::Path;
use std::str::FromStr;
use ansi_term::Colour::{Green, Red};
use indicatif::{ProgressBar, ProgressStyle};

//use self::error::DbError;
use crate::models::{ColumnDef};
use crate::parsers::InputService;
use crate::ConfigService;
use crate::storage::StorageService;

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
        let mut results: Vec<DBResults> = Vec::new();

        let pbar = ProgressBar::new(inputs.len() as u64);
        pbar.set_style(ProgressStyle::default_bar()
            .template("{prefix:.cyan/blue} {msg} [{bar:40.cyan/blue}] {pos:>3/blue}/{len:3}files")
            .progress_chars("=> "));
        pbar.set_prefix("Processing");

        let mut num_files = 0;
        for input in inputs {
            pbar.set_message(&format!("{}", &input.location));
            match self.input_svc.parse(input) {
                Err(e) => errors.push(format!("parse error: {:?}", e)),
                Ok(mut pc) => {
                    if !&pc.errors.is_empty() {
                        errors.append(&mut pc.errors.clone());
                    }
                    pc.set_column_data_types();
                    pbar.set_prefix("Loading Data...");
                    match self.store(self.get_table_name(pc.file_name.clone()),
                               pc.records_parsed,
                               pc.columns.clone(),
                               pc.content.clone()) {
                        Ok(result) => results.push(result),
                        Err(e) => errors.push(format!("{}", e)),
                    }
                    pbar.inc(1)
                }
            }
            num_files += 1;
        }
        pbar.finish_and_clear();

        // Pressing report
        self.display_report(results, errors, num_files);
        Ok(())
    }

    fn display_report(&self, store_results: Vec<DBResults>, errors: Vec<String>, num_files: u64) {
        let processed_msg = format!("{} files processed", num_files);
        let num_errors = errors.len();

        let err_stmt = match num_errors == 0 {
            true =>  format!("{}", Green.bold().paint("0 errors")),
            false => format!("{}", Red.bold().paint(format!("{} Errors", num_errors)))
        };

        println!("\ncsv-to results");
        println!("-------------------");
        println!("{} / {}", Green.bold().paint(processed_msg), err_stmt);
        for r in store_results {
            match r.get_results() {
                Ok(msg) => println!("{}", msg),
                Err(msg) => println!("{}", Red.bold().paint(format!("{}", msg)))
            }
        }

        if num_errors > 0 {
            let err_msg =format!("\nError Details\n-------------");
            println!("{}", Red.bold().paint(err_msg));
            for e in errors {
                eprintln!("{}", e);
            }
        }
    }

    fn store(&self, name: String, records_parsed: usize, columns: Vec<ColumnDef>, content: Vec<csv::StringRecord>) -> Result<DBResults, failure::Error> {

        return match self.storage_svc.create_store(name.clone(), columns.clone(), self.config_svc.should_drop_store()) {
            Ok(_) => {
                let insert_stmt = self.storage_svc.create_insert_stmt(name.clone(), columns.clone());
                match self.storage_svc.store_data( columns.clone(), content, insert_stmt) {
                    Ok(records_inserted) => Ok(DBResults::new(name.clone(), records_parsed, records_inserted)),
                     Err(e) => Err(e)
                }
            },
            Err(err) => return Err(failure::err_msg(format!("unable to create storage {}", err))),
        }
    }

    fn get_table_name(&self, file_path: String) -> String {
        // TODO: Clean this up
        let name = String::from(Path::new(&file_path).file_name().unwrap().to_str().unwrap());
        let first_letter = name.trim_right_matches(".csv").chars().next().unwrap();
        name.trim_right_matches(".csv").to_string().replace(first_letter, &first_letter.to_string().to_uppercase())
    }
}

struct DBResults {
    name: String,
    num_parsed: usize,
    num_stored: usize,
}

impl DBResults {
    pub fn new(name: String, num_parsed: usize, num_stored: usize) -> DBResults {
        DBResults{
            name,
            num_parsed,
            num_stored,
        }
    }

    pub fn get_results(&self) -> Result<String, failure::Error> {
        if &self.num_stored != &self.num_parsed {
           return  Err(failure::err_msg(format!("❌ {}: had {} errors", &self.name, self.num_parsed - self.num_stored)));
        }

        Ok(format!("✅ {}: {} records loaded", &self.name, &self.num_stored))
    }
}

#[derive(Debug, Clone)]
pub enum Types {
    Postgres,
    SQLite,
}

impl FromStr for Types {
    type Err = error::DbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower_s: &str = &s.to_lowercase();
        match lower_s {
            "sqlite" => Ok(Types::SQLite),
            "postgres" => Ok(Types::Postgres),
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

