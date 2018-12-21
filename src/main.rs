extern crate csv_to;
extern crate exitcode;
extern crate structopt;

use postgres::{Connection, TlsMode};
use csv_to::cache::json::JsonCache;
use csv_to::cmd::db;
use csv_to::cmd::db::{
    DbApp,
    Types,
    config::Config,
};
use csv_to::parsers::csv::CSVService;
use csv_to::storage::{
    postgres::PostgresStore,
    sqlite::SQLiteStore
};
use structopt::StructOpt;
use std::path::PathBuf;

fn main() {
    let opt = CsvTo::from_args();
    let cache_svc = JsonCache::new(String::from("./cache"));
    let csv_svc = CSVService::default();
    // As I build out the sub-commands this match will have multiple options, all of which will
    // implement the App trait
    //let app = match opt {
    match opt {
        CsvTo::Db { extension, files, directories, db_type, connection_info, name, drop_stores,
                    no_headers, save_cache } => {

            if files.is_empty() && directories.is_empty() {
                eprintln!("error: either -f, --files or -d, --directories must be provided");
                std::process::exit(exitcode::USAGE);
            }

            let config_svc = Config::new(extension, files, directories, db_type.clone(),
                                         connection_info.clone(), name, drop_stores,
                                         no_headers, save_cache);

            match db_type {
                Types::Postgres => {
                    let conn = Connection::connect(connection_info.clone(), TlsMode::None).unwrap_or_else(|err| {
                        eprintln!("ERROR: {}", err);
                        std::process::exit(exitcode::IOERR);
                    });

                    DbApp::new(
                        config_svc,
                        csv_svc,
                        cache_svc,
                        PostgresStore::new(conn)
                    ).run().unwrap_or_else(|err| {
                        eprintln!("ERROR: {}", err);
                        std::process::exit(exitcode::IOERR);
                    });
                },
                Types::SQLite => {
                    let conn = sqlite::open(connection_info.clone()).unwrap_or_else(|err| {
                       eprintln!("ERROR: {}", err);
                        std::process::exit(exitcode::IOERR);
                    });

                    DbApp::new(
                        config_svc,
                        csv_svc,
                        cache_svc,
                        SQLiteStore::new(conn),
                     ).run().unwrap_or_else(|err| {
                        eprintln!("ERROR: {}", err);
                        std::process::exit(exitcode::IOERR);
                    });
                }
            }
        }
    };
}

/// All command line options/flags broken into their sub-commands
#[derive(Debug, StructOpt)]
#[structopt(name = "csv-to", about = "creates databases and code from CSV data")]
pub enum CsvTo {
    #[structopt(name = "db", about = "creates and loads a database from CSV file(s)")]
    Db {
        #[structopt(short = "c", long = "connection-info", help = "Database connectivity information")]
        connection_info: String,

        #[structopt(short = "t", long = "type", help = "The type of database to create, valid types are sqlite and postgres")]
        db_type: db::Types,

        #[structopt(short = "d", parse(from_os_str), long = "directories", help = "The directories that contain CSV files to be processed, a comma delimited string of paths")]
        directories: Vec<PathBuf>,

        #[structopt(long = "drop-stores", help = "Drops tables/collections if the already exist")]
        drop_stores: bool,

        #[structopt(short = "e", long = "extension", help = "the file extension for the CSV files to be parsed", default_value = "csv")]
        extension: String,

        #[structopt(short = "f", parse(from_os_str), long = "files", help = "The CSV files to be processed, can be /path/to/files/ or a comma delimited string of paths")]
        files: Vec<PathBuf>,

        #[structopt(short = "n", long = "name", help = "Name of the database to be created")]
        name: String,

        #[structopt(long = "no-headers", help = "The CSV file(s) have no column headers")]
        no_headers: bool,

        #[structopt(long = "save-cache", help = "Stores the meta data about each input's column data definitions")]
        save_cache: bool,
    }
}

// This trait is what all of the sub-commands will implement so they can have a common
// interface that the main can call into to start the csv_to logic started
//trait App {
//    fn run(&self) -> Result<ParsedContent, std::io::Error> ;
//}
