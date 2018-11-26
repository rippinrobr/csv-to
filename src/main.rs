extern crate csv_to;
extern crate exitcode;
extern crate structopt;

use csv_to::{CsvTo, db};
use structopt::StructOpt;

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

