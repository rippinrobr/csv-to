extern crate csv_to;
extern crate exitcode;
extern crate structopt;

use csv_to::CsvTo;
use csv_to::db::{Config, DbApp};
use csv_to::adapters::csvinput::CSVService;
use structopt::StructOpt;

fn main() {
    let opt = CsvTo::from_args();
    
    let app = match opt {
        CsvTo::Db { files, directories, db_type, connection_info, name, no_headers } => {
            if files.len() == 0 && directories.len() == 0 {
                eprintln!("error: either -f, --files or -d, --directories must be provided");
                std::process::exit(exitcode::USAGE);
            }

            DbApp::new(
                Config::new(files, directories, db_type, connection_info, name, no_headers),
                CSVService::new()
            )
        }
    };

    app.run().unwrap_or_else(|err| {
        eprintln!("ERROR: {}", err);
        std::process::exit(exitcode::IOERR);
    });
}

