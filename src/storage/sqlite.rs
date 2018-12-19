use barrel::backend::Sqlite;
use barrel::types::Type;
use barrel::*;
use failure::Error;
use sqlite;
use sqlite::{Connection, Value};
use csv::StringRecord;
use crate::{ColumnDef, DataTypes};
use super::StorageService;

/// The adapter that handles the interactions with a SQLite store
pub struct SQLiteStore{
    conn: Connection,
}

impl SQLiteStore {
    // Creates a new instance of the SQLiteStore or an error if
    // a connection cannot be established
    pub fn new(conn: Connection) -> Self {
        Self{conn}
    }

    // Creates a new table in the database
    fn create_table(&self, sql_stmt: &str) -> Result<(), Error> {
        match self.conn.execute(sql_stmt) {
            Ok(_) => Ok(()),
            Err(e) => Err(failure::err_msg(format!("table creation error: {}", e)))
        }
    }

    // Creates the string that represents the schema for a table that maps to the columns
    // passed in
    fn generate_table_schema(name: String, cols: Vec<ColumnDef>, drop_table_if_exists: bool) -> Result<String, failure::Error> {
        if name == "" {
            return Err(failure::err_msg("Cannot create a table with an empty name"));
        }

        if cols.is_empty() {
            return Err(failure::err_msg("Cannot create a table with no columns"));
        }
        let mut m = Migration::new();

        if drop_table_if_exists {
            m.drop_table_if_exists(name.clone());
        }

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

        Ok(format!("{};", &m.make::<Sqlite>()))
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
    // Generates a string that contains the SQL for inserting a row into the given table
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

        match SQLiteStore::generate_table_schema(name.clone(), column_defs.clone(), drop_tables) {
            Ok(schema) => {
                match self.create_table(&schema) {
                    Err(e) => Err(failure::err_msg(format!("table creation error: {:?}", e))),
                    Ok(_) => Ok(())
                }
            },
            Err(e) => Err(e)
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

#[cfg(test)]
mod tests {
    use crate::storage::sqlite::SQLiteStore;
    use crate::{ColumnDef, DataTypes};
    use sqlite::Value;

    //==================================================
    // DataTypes tests
    #[test]
    fn get_value_type_with_string() {
        let test_val = String::from("hi");
        let cd = ColumnDef{
            potential_types: Vec::new(),
            name: String::from("mycol"),
            data_type: DataTypes::String,
        };
        let v: sqlite::Value = SQLiteStore::get_value_type(&cd, test_val.clone());
        assert_eq!(v, Value::String(test_val));
    }

    #[test]
    fn get_value_type_with_f64() {
        let test_val = 1.23;
        let cd = ColumnDef{
            potential_types: Vec::new(),
            name: String::from("mycol"),
            data_type: DataTypes::F64,
        };
        let v: sqlite::Value = SQLiteStore::get_value_type(&cd, test_val.to_string());
        assert_eq!(v, Value::Float(test_val));
    }

    #[test]
    fn get_value_type_with_i64() {
        let test_val = 123;
        let cd = ColumnDef{
            potential_types: Vec::new(),
            name: String::from("mycol"),
            data_type: DataTypes::I64,
        };
        let v: sqlite::Value = SQLiteStore::get_value_type(&cd, test_val.to_string());
        assert_eq!(v, Value::Integer(test_val));
    }

    #[test]
    fn get_value_type_with_empty() {
        let test_val = 123;
        let cd = ColumnDef{
            potential_types: Vec::new(),
            name: String::from("mycol"),
            data_type: DataTypes::Empty,
        };
        let v: sqlite::Value = SQLiteStore::get_value_type(&cd, test_val.to_string());
        assert_eq!(v, Value::Null);
    }

    #[test]
    fn generate_table_schema_with_empty_table_name_ret() {
        let name = String::new();
        let column_defs: Vec<ColumnDef> = Vec::new();

        match SQLiteStore::generate_table_schema(name.clone(), column_defs.clone(), false) {
            Ok(_) => {
                // I shouldn't have gotten here so I'm going to force a failure
                assert_eq!(true, false)
            },
            Err(e) => assert_eq!(format!("{}", e), "Cannot create a table with an empty name")
        }
    }

    #[test]
    fn generate_table_schema_with_no_columns_throws_error() {
        let name = String::from("mytable");
        let column_defs: Vec<ColumnDef> = Vec::new();

        match SQLiteStore::generate_table_schema(name.clone(), column_defs.clone(), false) {
            Ok(_) => {
                // I shouldn't have gotten here so I'm going to force a failure
                assert_eq!(true, false)
            },
            Err(e) => assert_eq!(format!("{}", e), "Cannot create a table with no columns")
        }
    }

    #[test]
    fn generate_table_schema_with_reqs_returns_proper_ddl() {
        let name = String::from("mytable");
        let c1 = ColumnDef{
            data_type: DataTypes::String,
            name: String::from("Col1"),
            potential_types: vec![DataTypes::String],
        };
        let c2 = ColumnDef{
            data_type: DataTypes::I64,
            name: String::from("Col2"),
            potential_types: vec![DataTypes::I64],
        };
        let column_defs = vec![c1, c2];

        match SQLiteStore::generate_table_schema(name.clone(), column_defs.clone(), false) {
            Ok(sql) => {
                // I shouldn't have gotten here so I'm going to force a failure
                assert_eq!(sql, String::from("CREATE TABLE \"mytable\" (\"Col1\" TEXT, \"Col2\" INTEGER);;"))
            },
            Err(e) => assert_eq!(format!("{}", e), "Cannot create a table with no columns")
        }
    }

    #[test]
    fn generate_table_schema_with_reqs_with_drop_stores_returns_proper_ddl() {
        let name = String::from("mytable");
        let c1 = ColumnDef{
            data_type: DataTypes::String,
            name: String::from("Col1"),
            potential_types: vec![DataTypes::String],
        };
        let c2 = ColumnDef{
            data_type: DataTypes::I64,
            name: String::from("Col2"),
            potential_types: vec![DataTypes::I64],
        };
        let column_defs = vec![c1, c2];

        match SQLiteStore::generate_table_schema(name.clone(), column_defs.clone(), true) {
            Ok(sql) => {
                // I shouldn't have gotten here so I'm going to force a failure
                assert_eq!(sql, String::from("DROP TABLE IF EXISTS \"mytable\";CREATE TABLE \"mytable\" (\"Col1\" TEXT, \"Col2\" INTEGER);;"))
            },
            Err(e) => assert_eq!(format!("{}", e), "Cannot create a table with no columns")
        }
    }
}