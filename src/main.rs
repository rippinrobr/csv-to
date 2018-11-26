#[macro_use] extern crate structopt;
extern crate csv_to;
extern crate exitcode;

use std::path::PathBuf;
use structopt::StructOpt;
use csv_to::db;

fn main() {
    let opt = CsvTo::from_args();
    
    match opt {
        CsvTo::Db { files, db_type, connection_info, name } => {
            db::run(&files, db_type, &connection_info, &name).unwrap_or_else(|err| {
                eprintln!("ERROR: An error occured while attempting to create a database. Error: {:?}", err.get_msg());
                std::process::exit(err.get_exit_code());
            });
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "csv-to", about = "creates databases and code from CSV data")]
enum CsvTo {
    #[structopt(name = "db", about = "creates and loads a database from CSV file(s)")]
    Db {
        #[structopt(short = "f", required = true, parse(from_os_str), long = "files", help = "The CSV files to be processed, can be /path/to/files/* or a comma delimited string of paths")]
        files: Vec<PathBuf>,

        #[structopt(short = "t", long = "type", help = "The type of database to create, currently only SQLite is supported")]
        db_type: db::Types,

        #[structopt(short = "c", long = "connection-info", help = "Database connectivity information")]
        connection_info: String,
        
        #[structopt(short = "n", long = "name", help = "Name of the database to be created")]
        name: String,
    }
}