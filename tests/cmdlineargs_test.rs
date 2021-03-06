extern crate assert_fs;

use std::process::{Command};

static CMD_PATH: &'static str = "./target/debug/csv-to";

#[test]
fn calling_csvto_with_no_args() {
    let usage = format!("csv-to {}
Rob Rowe <robrowe04@gmail.com>
creates databases and code from CSV data

USAGE:
    csv-to <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    db      creates and loads a database from CSV file(s)
    help    Prints this message or the help of the given subcommand(s)
", env!("CARGO_PKG_VERSION"));

    let output = Command::new(CMD_PATH)
        .output()
        .expect("failed to execute process");

    assert_eq!(usage, String::from_utf8_lossy(&output.stderr));
}

#[test]
fn calling_csvto_with_db_but_no_args() {
    let db_usage_msg = "error: The following required arguments were not provided:
    --connection-info <connection_info>
    --type <db_type>
    --name <name>

USAGE:
    csv-to db --connection-info <connection_info> --type <db_type> --extension <extension> --name <name>

For more information try --help
";
    let output = Command::new(CMD_PATH)
        .arg("db")
        .output()
        .expect("failed to execute process");

    assert_eq!(db_usage_msg, String::from_utf8_lossy(&output.stderr));
}

#[test]
fn calling_csvto_with_db_with_h() {
    let db_usage_msg = format!("csv-to-db {}
Rob Rowe <robrowe04@gmail.com>
creates and loads a database from CSV file(s)

USAGE:
    csv-to db [FLAGS] [OPTIONS] --connection-info <connection_info> --type <db_type> --name <name>

FLAGS:
        --delete-data    deletes the data from the tables, keeps the table's schema
        --drop-stores    Drops tables/collections if the already exist
    -h, --help           Prints help information
        --no-headers     The CSV file(s) have no column headers
    -s, --save-cache     Stores the meta data about each input's column data definitions
    -V, --version        Prints version information

OPTIONS:
    -c, --connection-info <connection_info>    Database connectivity information
    -t, --type <db_type>                       The type of database to create, valid types are sqlite, postgres, and
                                               mysql
    -d, --directories <directories>...         The directories that contain CSV files to be processed, a comma delimited
                                               string of paths
    -e, --extension <extension>                the file extension for the CSV files to be parsed [default: csv]
    -f, --files <files>...                     The CSV files to be processed, can be /path/to/files/ or a comma
                                               delimited string of paths
    -n, --name <name>                          Name of the database to be created
    -o, --one-table <one_table>                Store the parsed data in a single table with the name given here
", env!("CARGO_PKG_VERSION"));;

    let output = Command::new(CMD_PATH)
        .arg("db")
        .arg("-h")
        .output()
        .expect("failed to execute process");

    assert_eq!(db_usage_msg, String::from_utf8_lossy(&output.stdout));
}

#[test]
fn calling_csvto_with_db_with_help() {
    let db_usage_msg = format!("csv-to-db {}
Rob Rowe <robrowe04@gmail.com>
creates and loads a database from CSV file(s)

USAGE:
    csv-to db [FLAGS] [OPTIONS] --connection-info <connection_info> --type <db_type> --name <name>

FLAGS:
        --delete-data    deletes the data from the tables, keeps the table's schema
        --drop-stores    Drops tables/collections if the already exist
    -h, --help           Prints help information
        --no-headers     The CSV file(s) have no column headers
    -s, --save-cache     Stores the meta data about each input's column data definitions
    -V, --version        Prints version information

OPTIONS:
    -c, --connection-info <connection_info>    Database connectivity information
    -t, --type <db_type>                       The type of database to create, valid types are sqlite, postgres, and
                                               mysql
    -d, --directories <directories>...         The directories that contain CSV files to be processed, a comma delimited
                                               string of paths
    -e, --extension <extension>                the file extension for the CSV files to be parsed [default: csv]
    -f, --files <files>...                     The CSV files to be processed, can be /path/to/files/ or a comma
                                               delimited string of paths
    -n, --name <name>                          Name of the database to be created
    -o, --one-table <one_table>                Store the parsed data in a single table with the name given here
", env!("CARGO_PKG_VERSION"));;


    let output = Command::new(CMD_PATH)
        .arg("db")
        .arg("--help")
        .output()
        .expect("failed to execute process");

    assert_eq!(db_usage_msg, String::from_utf8_lossy(&output.stdout));
}

#[test]
fn calling_csvto_with_db_with_v() {
    let db_version_msg = format!("csv-to-db {}\n", env!("CARGO_PKG_VERSION"));
    
    let output = Command::new(CMD_PATH)
        .arg("db")
        .arg("-V")
        .output()
        .expect("failed to execute process");

    assert_eq!(db_version_msg, String::from_utf8_lossy(&output.stdout));
}

#[test]
fn calling_csvto_with_db_with_version() {
    let db_version_msg = format!("csv-to-db {}\n", env!("CARGO_PKG_VERSION"));
    
    let output = Command::new(CMD_PATH)
        .arg("db")
        .arg("--version")
        .output()
        .expect("failed to execute process");

    assert_eq!(db_version_msg, String::from_utf8_lossy(&output.stdout));
}

#[test]
fn calling_csvto_with_db_without_files_or_directories_and_no_piped_input() {
    let db_err_msg = "error: either -f, --files or -d, --directories must be provided\n";

    let output = Command::new(CMD_PATH)
        .arg("db")
        .arg("--connection-info")
        .arg("/tmp/my_test.db")
        .arg("--type")
        .arg("sqlite")
        .arg("--name")
        .arg("my_test_db")
        .output()
        .expect("failed to execute process");

    assert_eq!(db_err_msg, String::from_utf8_lossy(&output.stderr));
}

#[test]
fn calling_csvto_with_db_without_name() {
    let db_err_msg = "error: The following required arguments were not provided:
    --name <name>

USAGE:
    csv-to db --connection-info <connection_info> --type <db_type> --extension <extension> --files <files>... --name <name>

For more information try --help
";

    let output = Command::new(CMD_PATH)
        .arg("db")
        .arg("--connection-info")
        .arg("/tmp/my_test.db")
        .arg("--type")
        .arg("sqlite")
        .arg("--files")
        .arg("my_test_db")
        .output()
        .expect("failed to execute process");

    assert_eq!(db_err_msg, String::from_utf8_lossy(&output.stderr));
}


#[test]
fn calling_csvto_with_db_without_n_and_no_piped_input() {
    let db_err_msg = "error: The following required arguments were not provided:
    --name <name>

USAGE:
    csv-to db --connection-info <connection_info> --type <db_type> --extension <extension> --files <files>... --name <name>

For more information try --help
";

    let output = Command::new(CMD_PATH)
        .arg("db")
        .arg("-c")
        .arg("/tmp/my_test.db")
        .arg("-t")
        .arg("sqlite")
        .arg("-f")
        .arg("my_test_db")
        .output()
        .expect("failed to execute process");

    assert_eq!(db_err_msg, String::from_utf8_lossy(&output.stderr));
}

#[test]
fn calling_csvto_with_db_without_connection_info() {
    let db_err_msg = "error: The following required arguments were not provided:
    --connection-info <connection_info>

USAGE:
    csv-to db --connection-info <connection_info> --type <db_type> --extension <extension> --files <files>... --name <name>

For more information try --help
";

    let output = Command::new(CMD_PATH)
        .arg("db")
        .arg("--name")
        .arg("my_test_db")
        .arg("--type")
        .arg("sqlite")
        .arg("--files")
        .arg("my_test_db")
        .output()
        .expect("failed to execute process");

    assert_eq!(db_err_msg, String::from_utf8_lossy(&output.stderr));
}


#[test]
fn calling_csvto_with_db_without_c() {
    let db_err_msg = "error: The following required arguments were not provided:
    --connection-info <connection_info>

USAGE:
    csv-to db --connection-info <connection_info> --type <db_type> --extension <extension> --files <files>... --name <name>

For more information try --help
";

    let output = Command::new(CMD_PATH)
        .arg("db")
        .arg("-n")
        .arg("my_test_db")
        .arg("-t")
        .arg("sqlite")
        .arg("-f")
        .arg("my_test_db")
        .output()
        .expect("failed to execute process");

    assert_eq!(db_err_msg, String::from_utf8_lossy(&output.stderr));
}

#[test]
fn calling_csvto_with_db_without_type() {
    let db_err_msg = "error: The following required arguments were not provided:
    --type <db_type>

USAGE:
    csv-to db --connection-info <connection_info> --type <db_type> --extension <extension> --files <files>... --name <name>

For more information try --help
";

    let output = Command::new(CMD_PATH)
        .arg("db")
        .arg("--name")
        .arg("my_test_db")
        .arg("--connection-info")
        .arg("mytest.db")
        .arg("--files")
        .arg("my_test_csv_file")
        .output()
        .expect("failed to execute process");

    assert_eq!(db_err_msg, String::from_utf8_lossy(&output.stderr));
}


#[test]
fn calling_csvto_with_db_without_t() {
    let db_err_msg = "error: The following required arguments were not provided:
    --type <db_type>

USAGE:
    csv-to db --connection-info <connection_info> --type <db_type> --extension <extension> --files <files>... --name <name>

For more information try --help
";

    let output = Command::new(CMD_PATH)
        .arg("db")
        .arg("-n")
        .arg("my_test_db")
        .arg("-c")
        .arg("sqlite")
        .arg("-f")
        .arg("my_test_db")
        .output()
        .expect("failed to execute process");

    assert_eq!(db_err_msg, String::from_utf8_lossy(&output.stderr));
}

#[test]
fn calling_csvto_with_db_with_unsupported_db_type() {
    let db_err_msg = "error: Invalid value for '--type <db_type>': ERROR: 'mongo' is not a supported database type\n";

    let output = Command::new(CMD_PATH)
        .arg("db")
        .arg("-n")
        .arg("my_test_db")
        .arg("-c")
        .arg("/pathtosome.db")
        .arg("-t")
        .arg("mongo")
        .arg("-f")
        .arg("my_test_db")
        .output()
        .expect("failed to execute process");

    assert_eq!(db_err_msg, String::from_utf8_lossy(&output.stderr));
}


