//! StorageService Ports and Adapters
//!
//! This module contains the StorageService trait and adapters for the supported data stores. Currently
//! only Postgres and SQLite are supported.
//!
//!
pub mod mysql;
pub mod postgres;
pub mod sqlite;

use failure::Error;
use csv::StringRecord;
use crate::ColumnDef;

pub trait StorageService {
    /// creates an insert or appropriate create statement for the backend store
    fn create_insert_stmt(&self, store_name: String, column_defs: Vec<ColumnDef>) -> String;
    /// describes a method that will create a table for relational databases or the equivalent in a
    /// store that is supported
    fn create_store(&self, name: String, column_defs: Vec<ColumnDef>, drop_tables: bool) -> Result<(), Error>;
    /// deletes all data in the given table
    fn delete_data_in_table(&self, name: String) -> Result<(), Error>;
    /// stores the data in the store that implements this trait, a table in relational databases but
    /// returns the number of records stored successfully or any error(s) the method encounters
    fn store_data(&self, column_defs: Vec<ColumnDef>, data: Vec<StringRecord>, insert_stmt: String) -> Result<usize, Error>;
}