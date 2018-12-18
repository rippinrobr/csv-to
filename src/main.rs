extern crate csv_to;
extern crate exitcode;
extern crate structopt;

use csv_to::CsvTo;
use csv_to::cmd::db::{DbApp, Types};
use csv_to::cmd::db::config::Config;
use csv_to::parsers::csv::CSVService;
use csv_to::storage::{
    postgres::PostgresStore,
    sqlite::SQLiteStore
};
use structopt::StructOpt;

fn main() {
    let opt = CsvTo::from_args();

    // As I build out the sub-commands this match will have multiple options, all of which will
    // implement the App trait
    //let app = match opt {
    match opt {
        CsvTo::Db { extension, files, directories, db_type, connection_info, name, drop_stores, no_headers } => {
            if files.is_empty() && directories.is_empty() {
                eprintln!("error: either -f, --files or -d, --directories must be provided");
                std::process::exit(exitcode::USAGE);
            }

            // FIXME: get this so that the StorageService is the only thing being determined here
            // so that I can have one DbApp::new() and run() call
            match db_type {
                Types::Postgres => {
                    DbApp::new(
                        Config::new(extension, files, directories, db_type, connection_info.clone(), name, drop_stores, no_headers),
                        CSVService::default(),
                        PostgresStore::new(connection_info.clone()).unwrap_or_else(|err| {
                            eprintln!("error while attempting to create a database connection: {}", err);
                            std::process::exit(exitcode::USAGE);
                        }),
                    ).run().unwrap_or_else(|err| {
                        eprintln!("ERROR: {}", err);
                        std::process::exit(exitcode::IOERR);
                    });
                },
                Types::SQLite => {
                    DbApp::new(
                        Config::new(extension, files, directories, db_type, connection_info.clone(), name, drop_stores, no_headers),
                        CSVService::default(),
                        SQLiteStore::new(connection_info.clone()).unwrap_or_else(|err| {
                            eprintln!("error while attempting to create a database connection: {}", err);
                            std::process::exit(exitcode::USAGE);
                        }),
                    ).run().unwrap_or_else(|err| {
                        eprintln!("ERROR: {}", err);
                        std::process::exit(exitcode::IOERR);
                    });
                }
            }
        }
    };
}
