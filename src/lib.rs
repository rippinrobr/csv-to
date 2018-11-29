extern crate failure;
extern crate glob;
#[macro_use] extern crate structopt;
extern crate csv_converter;

pub mod adapters;
pub mod db;
pub mod ports;

use std::path::PathBuf;
use structopt::StructOpt;

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
    }
}