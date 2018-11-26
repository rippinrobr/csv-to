use std::path::PathBuf;
use std::str::FromStr;
use failure::Fail;
use db::error::DbError;

#[derive(Debug)]
pub enum Types {
    SQLite,
}

impl FromStr for Types {
    type Err = error::DbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower_s: &str = &s.to_lowercase();
        match lower_s {
            "sqlite" => Ok(Types::SQLite),
            _ => Err(error::DbError::new(format!("ERROR: '{}' is not a supported database type", lower_s), exitcode::USAGE))
        }
    }
}

pub fn run(files: &Vec<PathBuf>, db_type: Types, connection_info: &str, name: &str) -> Result<(), error::DbError> {
    // 0. Validate parameters, make sure name isn't empty, connection_info and at least one file
    if name == "" {
        return Err(DbError::new(format!("ERROR: the name argument cannot be empty"), exitcode::USAGE))
    }

    if connection_info == "" {
        return Err(DbError::new(format!("ERROR: the name connection_info cannot be empty"), exitcode::USAGE))
    }
    
    println!("{:?}, {:?}, {:?}, {:?}", files, db_type, connection_info, name);
    Ok(())
}

pub mod error {
    use failure::Fail;

    #[derive(Fail, Debug)]
    #[fail(display = "{}", msg)]
    pub struct DbError {
        msg: String,
        exit_code: exitcode::ExitCode,
    }

    impl DbError {
        pub fn get_exit_code(&self) -> exitcode::ExitCode {
            self.exit_code
        }

        pub fn get_msg(&self) -> String {
            self.msg.clone()
        }

        pub fn new(msg: String, exit_code: exitcode::ExitCode) -> DbError {
            DbError { msg, exit_code }
        }
    }
}