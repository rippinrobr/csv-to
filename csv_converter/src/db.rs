
use super::models::{ColumnDef, DataTypes};
use sqlite;
use sqlite::{Connection, Error, Value};
use std::path::Path;
use std::vec::Vec;

pub struct SqliteDB {
    db_conn: Connection,
} 


impl SqliteDB {
    pub fn new(path: &str) -> Result<SqliteDB, Error> {
        let path_obj = Path::new(path);
        let db_conn = sqlite::open(path_obj)?;

        Ok(SqliteDB {
            db_conn: db_conn,
        })
    }

    pub fn create_table(&self, sql_stmt: String) -> Result<(), Error> {
        self.db_conn.execute(&sql_stmt)
    }

    pub fn insert_rows(&self, insert_stmt: String, columns: &Vec<ColumnDef>, content: Vec<Vec<String>>) -> Result<usize, Error> {
        let mut stmt = self.db_conn.prepare(insert_stmt).unwrap();
        let mut rows_inserted_count = 0;

        for vrec in content {
            stmt.reset().unwrap();
            rows_inserted_count += 1;
            let mut col_idx: usize = 1;
            for c in vrec.as_slice() {
                let value = &SqliteDB::get_value_type(&columns[col_idx-1], c.to_string());
                stmt.bind(col_idx, value).unwrap();
                col_idx += 1;
            }

            let _results = stmt.next().unwrap();
        } 
        Ok(rows_inserted_count)
    }

    // get_value_type converts the given col_val to approprate type for 
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


#[cfg(test)]
mod tests {
    use db::SqliteDB;

    #[test]
    fn new() {
        let my_path = "/tmp/test.db";

        match SqliteDB::new(my_path) {
            Ok(_) => {
                assert!(true);
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                assert!(false);
            }
        };
    }
}