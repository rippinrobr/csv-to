use barrel::backend::Sqlite;  // this works for SQLite also
use barrel::*;
use failure::Error;
use sqlite;
use sqlite::{Connection, Value};
use csv::StringRecord;
use csv_converter::models::{ColumnDef, DataTypes};
use super::StorageService;

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

        format!("{};", &m.make::<Sqlite>())
    }

    // get_value_type converts the given col_val to appropriate type for
    // the col provided.  For numeric columns if a non integer or float is
    // provided in col_value the value of 0 or 0.0 will be returned.
    fn get_value_type(col: &ColumnDef, col_value: String) -> sqlite::Value {
        match col.data_type {
            DataTypes::String => Value::String(col_value),
            DataTypes::I64 => {
                let value = match col_value.parse::<i64>() {
                    Ok(v) => v,
                    Err(_) => {
                        0
                    }
                };
                Value::Integer(value)
            },
            DataTypes::F64 => {
                let value = match col_value.parse::<f64>() {
                    Ok(v) => v,
                    Err(_) => {
                        0.0
                    }
                };
                Value::Float(value)
            },
            DataTypes::Empty => Value::Null
        }
    }
}

impl StorageService for SQLiteStore {
    fn create_insert_stmt(&self, store_name: String, column_defs: Vec<ColumnDef>) -> String {
        let mut placeholders: Vec<String> = Vec::new();
        for n in 0..column_defs.len() {
            placeholders.push(format!("?{}", n+1));
        }

        let col_names: Vec<String> = column_defs.into_iter().map(move |c| c.name.clone()).collect();
        format!("INSERT INTO {} ({}) VALUES ({})", store_name, col_names.join(", "), placeholders.join(", "))
    }

    /// Creates the table with the given name that will store the data from the related input file
    fn create_store(&self, name: String, column_defs: Vec<ColumnDef>, drop_tables: bool) -> Result<(), failure::Error> {
        if name == "" {
            return Err(failure::err_msg("name cannot be empty.".to_string()));
        }

        if column_defs.is_empty() {
            return Err(failure::err_msg("there must be at least 1 column.".to_string()));
        }

        let schema = SQLiteStore::generate_table_schema(name.clone(), column_defs.clone(), drop_tables);
        match self.create_table(schema) {
            Err(e) => Err(failure::err_msg(format!("table creation error: {:?}", e))),
            Ok(_) => Ok(())
        }
    }
    /// stores the data in the store that implements this trait, a table in relational databases but
    /// returns the number of records stored successfully or any error(s) the method encounters
    fn store_data(&self, column_defs: Vec<ColumnDef>, content: Vec<StringRecord>, insert_stmt: String) -> Result<usize, failure::Error> {
        let mut rows_inserted_count = 0;
        let mut stmt = self.conn.prepare(insert_stmt).unwrap();

        for vrec in content {
            stmt.reset().unwrap();
            rows_inserted_count += 1;

            let mut col_idx: usize = 1;
            for c in vrec.iter() {
                let value = &SQLiteStore::get_value_type(&column_defs[col_idx - 1], c.to_string());
                stmt.bind(col_idx, value).unwrap();
                col_idx += 1;
            }

            let _results = stmt.next().unwrap();
        }

        Ok(rows_inserted_count)
    }
}