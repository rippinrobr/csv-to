use barrel::backend::{Pg, SqlGenerator};
use barrel::*;
use csv::StringRecord;
use failure::Error;
use postgres::{Connection, TlsMode};
use postgres::types::Type;
use csv_converter::models::{ColumnDef, DataTypes};
use super::StorageService;

pub struct PostgresStore{
    conn: Connection,
}

impl PostgresStore{
    pub fn new(url: String) -> Result<PostgresStore, Error> {
        match Connection::connect(url, TlsMode::None) {
            Ok(conn) => Ok(PostgresStore{ conn }),
            Err(e) => Err(failure::err_msg(format!("{}", e)))
        }
    }

    fn create_table(&self, sql_stmt: String) -> Result<(), Error> {
        match self.conn.execute(&sql_stmt, &[]) {
            Ok(_) => Ok(()),
            Err(e) => Err(failure::err_msg(format!("table creation error: {}", e)))
        }
    }

    fn generate_table_schema(name: String, cols: Vec<ColumnDef>, drop_table_if_exists: bool) -> String {
        let mut m = Migration::new();

        if drop_table_if_exists {
            m.drop_table_if_exists(name.clone());
        }

        m.create_table(name, move |t| {
            for cd in &cols {
                let cname: &str = &cd.name;
                t.add_column(cname, cd.data_type.to_database_type());
            }
        }).without_id();

        format!("{};", &m.make::<Pg>())
    }
}

impl StorageService for PostgresStore {
    /// creates an insert or appropriate create statement for the backend store
    fn create_insert_stmt(&self, store_name: String, column_defs: Vec<ColumnDef>) -> String{
        String::new()
    }
    /// describes a method that will create a table for relational databases or the equivalent in a
    /// store that is supported
    fn create_store(&self, name: String, column_defs: Vec<ColumnDef>, drop_tables: bool) -> Result<(), Error> {
        Err(failure::err_msg("not implemented"))
    }
    /// stores the data in the store that implements this trait, a table in relational databases but
    /// returns the number of records stored successfully or any error(s) the method encounters
    fn store_data(&self, column_defs: Vec<ColumnDef>, data: Vec<StringRecord>, insert_stmt: String) -> Result<usize, Error> {
        Err(failure::err_msg("not implemented"))
    }
}