use failure::Error;
use regex::Regex;
use csv::StringRecord;
use csv_converter::models::{ColumnDef, ParsedContent};
use crate::ports::storageservice::StorageService;

pub struct SQLiteStore;

impl SQLiteStore {
    pub fn new() -> SQLiteStore {
        SQLiteStore{}
    }
}

impl StorageService for SQLiteStore {
    /// Creates the table that will store the data from the related input file
    fn create_store(&self, column_defs: Vec<ColumnDef>) -> Result<(), Error> {
        Err(failure::err_msg("not implemented"))
    }
    /// stores the data in the store that implements this trait, a table in relational databases but
    /// returns the number of records stored successfully or any error(s) the method encounters
    fn store_data(&self, column_defs: Vec<ColumnDef>, data: Vec<StringRecord> ) -> Result<usize, Error> {
        Err(failure::err_msg("not implemented"))
    }
    /// validates the number of records that existed in the CSV file were added to the store
    /// returns the true if the total_lines is equal to the number of records in the store
    fn validate(&self, total_lines: usize) -> Result<bool, Error> {
        Err(failure::err_msg("not implemented"))
    }
}