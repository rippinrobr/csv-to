extern crate csv_to;
extern crate exitcode;
extern crate structopt;

use csv_to::{CsvTo, db};
use csv_to::db::Config;
use csv_to::adapters::csvinput::CSVService;
use structopt::StructOpt;

fn main() {
    let opt = CsvTo::from_args();
    
    match opt {
        CsvTo::Db { files, directories, db_type, connection_info, name } => {
            if files.len() == 0 && directories.len() == 0 {
                eprintln!("error: either -f, --files or -d, --directories must be provided");
                std::process::exit(exitcode::USAGE);
            }

            let app_config = Config::new(files, directories, db_type, connection_info, name);
            let csv_input = CSVService::new(&app_config.get_input_sources());
        }
    }
}

