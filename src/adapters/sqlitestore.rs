use barrel::*;
use barrel::backend::Pg;  // this works for SQLite also
use failure::Error;
use regex::Regex;
use sqlite;
use sqlite::{Connection, Value};
use csv::StringRecord;
use csv_converter::models::{ColumnDef, ParsedContent};
use crate::ports::storageservice::StorageService;

pub struct SQLiteStore{
    conn: Connection,
}

impl SQLiteStore {
    pub fn new(db_path: String) -> Result<SQLiteStore, sqlite::Error> {
        Ok(SQLiteStore{
            conn: sqlite::open(db_path)?
        })
    }

    fn create_table(&self, sql_stmt: String) -> Result<(), Error> {
        match self.conn.execute(&sql_stmt) {
            Ok(_) => Ok(()),
            Err(e) => Err(failure::err_msg(format!("table creation error: {}", e)))
        }
    }

    fn generate_table_schema(name: String, cols: Vec<ColumnDef>) -> String {
        let mut m = Migration::new();

        m.create_table(name, move |t| {
            for cd in &cols {
                let cname: &str = &cd.name;
                t.add_column(cname, cd.data_type.to_database_type());
            }
        }).without_id();

        format!("{};", &m.make::<Pg>())
    }
}

impl StorageService for SQLiteStore {
    /// Creates the table with the given name that will store the data from the related input file
    fn create_store(&self, name: String, column_defs: Vec<ColumnDef>) -> Result<(), failure::Error> {
        if name == "" {
            return Err(failure::err_msg("name cannot be empty.".to_string()));
        }

        if column_defs.is_empty() {
            return Err(failure::err_msg("there must be at least 1 column.".to_string()));
        }

        let schema = SQLiteStore::generate_table_schema(name.clone(), column_defs.clone());
        self.create_table(schema)
    }
    /// stores the data in the store that implements this trait, a table in relational databases but
    /// returns the number of records stored successfully or any error(s) the method encounters
    fn store_data(&self, name: String, data: Vec<StringRecord> ) -> Result<usize, failure::Error> {
        Err(failure::err_msg("not implemented"))
    }
    /// validates the number of records that existed in the CSV file were added to the store
    /// returns the true if the total_lines is equal to the number of records in the store
    fn validate(&self, name: String, total_lines: usize) -> Result<bool, failure::Error> {
        Err(failure::err_msg("not implemented"))
    }
}