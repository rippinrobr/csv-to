extern crate csv_to;
extern crate exitcode;
extern crate structopt;

use csv_to::CsvTo;
use csv_to::db::{Config, DbApp};
use csv_to::adapters::{
    csvinput::CSVService,
    sqlitestore::SQLiteStore,
};
use structopt::StructOpt;

fn main() {
    let opt = CsvTo::from_args();
    
    let app = match opt {
        CsvTo::Db { files, directories, db_type, connection_info, name, no_headers } => {
            if files.is_empty() && directories.is_empty() {
                eprintln!("error: either -f, --files or -d, --directories must be provided");
                std::process::exit(exitcode::USAGE);
            }

            DbApp::new(
                Config::new(files, directories, db_type, connection_info.clone(), name, no_headers),
                CSVService::default(),
                SQLiteStore::new(connection_info.clone()).unwrap_or_else(|err| {
                    eprintln!("error while attempting to create a database connection: {}", err);
                    std::process::exit(exitcode::USAGE);
                }),
            )
        }
    };

    app.run().unwrap_or_else(|err| {
        eprintln!("ERROR: {}", err);
        std::process::exit(exitcode::IOERR);
    });
}

