extern crate csv_to;
extern crate exitcode;
extern crate structopt;

use csv_to::{CsvTo, db};
use csv_to::db::{Config, DbApp};
use csv_to::adapters::csvinput::CSVService;
use csv_to::ports::inputservice::InputService;
use structopt::StructOpt;

fn main() {
    let opt = CsvTo::from_args();
    
    match opt {
        CsvTo::Db { files, directories, db_type, connection_info, name } => {
            if files.len() == 0 && directories.len() == 0 {
                eprintln!("error: either -f, --files or -d, --directories must be provided");
                std::process::exit(exitcode::USAGE);
            }

            let db_app = DbApp::new(
                Config::new(files, directories, db_type, connection_info, name),
                CSVService::new()
            );

            db_app.run().unwrap_or_else(|err| {
                eprintln!("ERROR: {}", err);
                std::process::exit(exitcode::DATAERR);
            })
        }
    }
}

