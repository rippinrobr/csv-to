pub mod postgres;
pub mod sqlite;

use barrel::backend::{Pg, SqlGenerator};
use barrel::*;
use failure::Error;
use csv::StringRecord;
use csv_converter::models::DataTypes;
use csv_converter::models::ColumnDef;

pub trait StorageService {
    /// creates an insert or appropriate create statement for the backend store
    fn create_insert_stmt(&self, store_name: String, column_defs: Vec<ColumnDef>) -> String;
    /// describes a method that will create a table for relational databases or the equivalent in a
    /// store that is supported
    fn create_store(&self, name: String, column_defs: Vec<ColumnDef>, drop_tables: bool) -> Result<(), Error>;
    /// stores the data in the store that implements this trait, a table in relational databases but
    /// returns the number of records stored successfully or any error(s) the method encounters
    fn store_data(&self, column_defs: Vec<ColumnDef>, data: Vec<StringRecord>, insert_stmt: String) -> Result<usize, Error>;
}