extern crate failure;
extern crate glob;
#[macro_use] extern crate failure_derive;
#[macro_use] extern crate structopt;
extern crate csv_converter;

pub mod adapters;
pub mod db;
pub mod ports;

use std::path::PathBuf;
use structopt::StructOpt;

use csv_converter::models::ParsedContent;

#[derive(Debug, StructOpt)]
#[structopt(name = "csv-to", about = "creates databases and code from CSV data")]
pub enum CsvTo {
    #[structopt(name = "db", about = "creates and loads a database from CSV file(s)")]
    Db {
        #[structopt(short = "f", parse(from_os_str), long = "files", help = "The CSV files to be processed, can be /path/to/files/ or a comma delimited string of paths")]
        files: Vec<PathBuf>,

        #[structopt(short = "d", parse(from_os_str), long = "directories", help = "The directories that contain CSV files to be processed, a comma delimited string of paths")]
        directories: Vec<PathBuf>,

        #[structopt(short = "t", long = "type", help = "The type of database to create, currently only SQLite is supported")]
        db_type: db::Types,

        #[structopt(short = "c", long = "connection-info", help = "Database connectivity information")]
        connection_info: String,
        
        #[structopt(short = "n", long = "name", help = "Name of the database to be created")]
        name: String,

        #[structopt(long = "no-headers", help = "The CSV file(s) have no column headers")]
        no_headers: bool
    }
}

pub trait App {
    fn run(&self) -> Result<ParsedContent, std::io::Error> ;
}

pub mod errors {
    //use failure::Error;

    // This is a new error type that you've created. It represents the ways a
    // toolchain could be invalid.
    //
    // The custom derive for Fail derives an impl of both Fail and Display.
    // We don't do any other magic like creating new types.
//    #[derive(Debug, Fail)]
//    enum ToolchainError {
//        #[fail(display = "invalid toolchain name: {}", name)]
//        InvalidToolchainName {
//            name: String,
//        },
//        #[fail(display = "unknown toolchain version: {}", version)]
//        UnknownToolchainVersion {
//            version: String,
//        }
//    }
}