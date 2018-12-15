use barrel::backend::{Pg, SqlGenerator};
use barrel::types::Type;
use barrel::*;
use csv::StringRecord;
use failure::Error;
use postgres::{Connection, TlsMode};
//use postgres::types::Type;
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

    pub fn create_database(&self, name: String, drop_if_exists: bool) -> Result<(), Error> {
        match self.conn.execute(&format!("CREATE DATABASE {};", name), &[]) {
            Ok(_) => Ok(()),
            Err(e) => Err(failure::err_msg(format!("database creation error: {}", e)))
        }
    }

    fn exec(&self, sql_stmt: String) -> Result<(), Error> {
        match self.conn.execute(&sql_stmt, &[]) {
            Ok(_) => Ok(()),
            Err(e) => Err(failure::err_msg(format!("table creation error: {}", e)))
        }
    }


    fn drop_table_sql(table_name: &str) -> String {
        let mut d = Migration::new();

        d.drop_table_if_exists(table_name);

        format!("{};", &d.make::<Pg>())
    }

    fn generate_table_schema(name: String, cols: Vec<ColumnDef>, drop_table_if_exists: bool) -> String {
        let mut m = Migration::new();

        m.create_table(name, move |t| {
            for cd in &cols {
                let cname: &str = &cd.name;
                t.add_column(cname, Type{
                    nullable: true,
                    unique: false,
                    increments: false,
                    indexed: false,
                    default: None,
                    size: None,
                    inner: cd.data_type.to_database_type()
                });
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
        if name == "" {
            return Err(failure::err_msg("name cannot be empty.".to_string()));
        }

        if column_defs.is_empty() {
            return Err(failure::err_msg("there must be at least 1 column.".to_string()));
        }

        if drop_tables {
            match self.exec(PostgresStore::drop_table_sql(&name)) {
                Err(e) => eprintln!("{}", e),
                _ => ()
            }
        }

        let schema = PostgresStore::generate_table_schema(name.clone(), column_defs.clone(), drop_tables);
        println!("schema: {}", schema);
        match self.exec(schema) {
            Err(e) => Err(failure::err_msg(format!("table creation error: {:?}", e))),
            Ok(_) => Ok(())
        }

        //Err(failure::err_msg("[create store] not implemented"))
    }
    /// stores the data in the store that implements this trait, a table in relational databases but
    /// returns the number of records stored successfully or any error(s) the method encounters
    fn store_data(&self, column_defs: Vec<ColumnDef>, data: Vec<StringRecord>, insert_stmt: String) -> Result<usize, Error> {
        Err(failure::err_msg("[store data] not implemented"))
    }
}