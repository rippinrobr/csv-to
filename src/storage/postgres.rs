use barrel::backend::Pg;
use barrel::*;

use csv::StringRecord;
use failure::Error;
use failure::err_msg;
use postgres::Connection;
use crate::{ColumnDef, DataTypes};
use super::StorageService;

/// Manages interactions with a Postgres database
pub struct PostgresStore{
    conn: Connection,
}

impl PostgresStore{
    /// returns an instance of the PostgresStore which is is used to interact with a Postgres
    /// database server
    pub fn new(conn: Connection) -> Self {
        Self{ conn }
    }

    /// creates a database with the given name
    pub fn create_database(&self, name: &str, _drop_if_exists: bool) -> Result<(), Error> {
        match self.conn.execute(&format!("CREATE DATABASE {};", name), &[]) {
            Ok(_) => Ok(()),
            Err(e) => Err(failure::err_msg(format!("database creation error: {}", e)))
        }
    }

    fn exec(&self, sql_stmt: &str) -> Result<(), Error> {
        match self.conn.execute(&sql_stmt, &[]) {
            Ok(_) => Ok(()),
            Err(e) => Err(failure::err_msg(format!("exec: {}\n{}", e, sql_stmt)))
        }
    }

    fn drop_table_sql(table_name: &str) -> Result<String, Error> {
        if table_name == "" {
            return Err(failure::err_msg("cannot drop a table schema without a name"))
        }
        let mut d = Migration::new();

        d.drop_table_if_exists(table_name);

        Ok(format!("{};", &d.make::<Pg>()))
    }

    fn generate_table_schema(name: String, cols: Vec<ColumnDef>) -> Result<String, Error> {
        if name == "" {
            return Err(failure::err_msg("cannot create a table schema without a name"));
        }

        if cols.is_empty() {
            return Err(failure::err_msg("cannot create a table schema without at least one column"));
        }

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

        Ok(format!("{};", &m.make::<Pg>()))
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
            match &PostgresStore::drop_table_sql(&name.to_lowercase()) {
                Ok(stmt) => {
                    if let Err(e) = self.exec(stmt) {
                        eprintln!("ERROR: {}", e);
                    }
                },
                Err(e) => eprintln!("ERROR: {}", e),
            }
        }

        match PostgresStore::generate_table_schema(name.clone().to_lowercase(), column_defs.clone()) {
            Ok(stmt) => {
                match self.exec(&stmt) {
                    Err(e) => Err(failure::err_msg(format!("table creation error: {:?}", e))),
                    Ok(_) => Ok(())
                }
            },
            Err(e) => Err(e)
        }
    }

    fn delete_data_in_table(&self, name: String) -> Result<(), Error> {
        if name == "" {
            return Err(err_msg("cannot delete data from a table with an empty name"))
        }

        match self.exec(&format!("delete from {};", name)) {
            Err(e) => Err(failure::err_msg(format!("data deletion error: {:?}", e))),
            Ok(_) => Ok(())
        }
    }

    /// stores the data in the store that implements this trait, a table in relational databases but
    /// returns the number of records stored successfully or any error(s) the method encounters
    fn store_data(&self, column_defs: Vec<ColumnDef>, data: Vec<StringRecord>, insert_stmt: String) -> Result<usize, Error> {
        let mut rows_inserted_count = 0;
        for line in data {
            let mut vals: Vec<String> = Vec::new();
            for (col_idx, rec) in line.iter().enumerate()  {
                if column_defs[col_idx].data_type == DataTypes::String {
                    vals.push(format!("'{}'", rec.replace("'", "''")));
                } else if rec != "" {
                        vals.push(rec.to_string())
                } else {
                    vals.push("0".to_string());
                }
            }

            match self.exec(&format!("{} ({})", insert_stmt, vals.join(", "))) {
                Err(e) => eprintln!("{}", e),
                _ => {
                    rows_inserted_count += 1;
                }
            }
        }

        Ok(rows_inserted_count)
    }
}

#[cfg(test)]
mod tests {
    use crate::{ColumnDef, DataTypes};
    use crate::storage::postgres::PostgresStore;

    #[test]
    fn generate_table_schema_with_empty_name_returns_error() {
        let name = "";
        let cols: Vec<ColumnDef>  = Vec::new();

        match PostgresStore::generate_table_schema(name.to_string(), cols) {
            Ok(_) => {
                // this should not be reached
                assert_eq!(0,1);
            },
            Err(e) => {
                assert_eq!(format!("{}",e), "cannot create a table schema without a name");
            }
        }
    }

    #[test]
    fn generate_table_schema_with_empty_columns_returns_error() {
        let name = "mine";
        let cols: Vec<ColumnDef>  = Vec::new();

        match PostgresStore::generate_table_schema(name.to_string(), cols) {
            Ok(_) => {
                // this should not be reached
                assert_eq!(0,1);
            },
            Err(e) => {
                assert_eq!(format!("{}",e), "cannot create a table schema without at least one column");
            }
        }
    }

    #[test]
    fn generate_table_schema_with_valid_inputs() {
        let name = "mine";
        let cols: Vec<ColumnDef>  = vec![ColumnDef{
            name: String::from("mycol"),
            data_type: DataTypes::String,
            potential_types: Vec::new(),
        }];

        match PostgresStore::generate_table_schema(name.to_string(), cols) {
            Ok(schema) => {
                assert_eq!(schema, String::from("CREATE TABLE \"mine\" (\"mycol\" TEXT);;"));
            },
            Err(_) => {
                // shouldn't reach this spot
                assert_eq!(0,1);
            }
        }
    }

    #[test]
    fn drop_table_sql_with_empty_name_returns_error() {
        match PostgresStore::drop_table_sql("") {
            Err(e) => assert_eq!(format!("{}",e), "cannot drop a table schema without a name"),
            Ok(_) => {
                // should not reach this
                assert_eq!(0,1)
            }
        }
    }

    #[test]
    fn drop_table_sql_with_valid_input() {
        match PostgresStore::drop_table_sql("mytable") {
            // should not reach this
            Err(e) => assert_eq!(0, 1),
            Ok(stmt) => {
                assert_eq!(stmt, String::from("DROP TABLE IF EXISTS \"mytable\";;"))
            }
        }
    }
}