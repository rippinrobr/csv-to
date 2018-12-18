extern crate barrel;

use barrel::backend::Pg;
use barrel::*;

use csv::StringRecord;
use failure::Error;
use postgres::{Connection, TlsMode};
use crate::{ColumnDef, DataTypes};
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

    pub fn create_database(&self, name: String, _drop_if_exists: bool) -> Result<(), Error> {
        match self.conn.execute(&format!("CREATE DATABASE {};", name), &[]) {
            Ok(_) => Ok(()),
            Err(e) => Err(failure::err_msg(format!("database creation error: {}", e)))
        }
    }

    fn exec(&self, sql_stmt: String) -> Result<(), Error> {
        match self.conn.execute(&sql_stmt, &[]) {
            Ok(_) => Ok(()),
            Err(e) => Err(failure::err_msg(format!("exec: {}\n{}", e, sql_stmt)))
        }
    }

    fn drop_table_sql(table_name: &str) -> String {
        let mut d = Migration::new();

        d.drop_table_if_exists(table_name);

        format!("{};", &d.make::<Pg>())
    }

    fn generate_table_schema(name: String, cols: Vec<ColumnDef>, _drop_table_if_exists: bool) -> String {
        let mut m = Migration::new();

        m.create_table(name, move |t| {
            for cd in &cols {
                let cname: &str = &cd.name.to_lowercase();
                t.add_column(cname,  barrel::types::Type{
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
        let col_names: Vec<String> = column_defs.into_iter().map(move |c| format!("\"{}\"", c.name.clone().to_lowercase())).collect();
        format!("INSERT INTO {} ({}) VALUES ", store_name.to_lowercase(), col_names.join(", "))
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
            match self.exec(PostgresStore::drop_table_sql(&name.to_lowercase())) {
                Err(e) => eprintln!("{}", e),
                _ => ()
            }
        }

        let schema = PostgresStore::generate_table_schema(name.clone().to_lowercase(), column_defs.clone(), drop_tables);
        match self.exec(schema) {
            Err(e) => Err(failure::err_msg(format!("table creation error: {:?}", e))),
            Ok(_) => Ok(())
        }
    }
    /// stores the data in the store that implements this trait, a table in relational databases but
    /// returns the number of records stored successfully or any error(s) the method encounters
    fn store_data(&self, column_defs: Vec<ColumnDef>, data: Vec<StringRecord>, insert_stmt: String) -> Result<usize, Error> {
        let mut rows_inserted_count = 0;
        for line in data {
            let mut col_idx: usize = 0;

            let mut vals: Vec<String> = Vec::new();
            for rec in line.iter() {
                if column_defs[col_idx].data_type == DataTypes::String {
                    vals.push(format!("'{}'", rec.replace("'", "''")));
                } else {
                    if rec != "" {
                        vals.push(rec.to_string())
                    } else {
                        vals.push("0".to_string());
                    }
                }
                col_idx += 1;
            }

            match self.exec(format!("{} ({})", insert_stmt, vals.join(", "))) {
                Err(e) => eprintln!("{}", e),
                _ => {
                    rows_inserted_count += 1;
                    ()
                }
            }
        }

        Ok(rows_inserted_count)
    }
}