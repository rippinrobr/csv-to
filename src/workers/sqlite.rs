use models::{ColumnDef, DataTypes};
use sqlite;
use sqlite::{Connection, Error, Value};
use std::path::Path;
use std::vec::Vec;

pub struct SqliteDB {
    path: String,
    db_conn: Connection,
} 

impl SqliteDB {
    pub fn new(path: &str) -> Result<SqliteDB, Error> {
        let path_obj = Path::new(path);
        let db_conn = sqlite::open(path_obj)?;

        Ok(SqliteDB {
            path: path.to_owned(),
            db_conn: db_conn,
        })
    }

    pub fn create_table(&self, sql_stmt: String) -> Result<(), Error> {
        self.db_conn.execute(&sql_stmt)
    }

    pub fn insert_rows(&self, insert_stmt: String, columns: &Vec<ColumnDef>, content: Vec<Vec<String>>) -> Result<i64, Error> {
        //println!("insert_stmt: {}", insert_stmt);
        let mut stmt = self.db_conn.prepare(insert_stmt).unwrap();
        for vrec in content {
            let mut i = 1;
            //let mut cloned_stmt = stmt.clone().unwrap();
            stmt.reset().unwrap();
            for c in vrec.as_slice() {
                let value = &SqliteDB::get_value_type(&columns[i-1], c.to_string());
                stmt.bind(i, value).unwrap();
                i += 1;
            }

            let _results = stmt.next().unwrap();
            //println!("results: {:?}", results);
        } 
    //   execute(&mut self, params: &[&ToSql]) -> Result<c_int>
        Ok(0)
    }

    fn get_value_type(col: &ColumnDef, col_value: String) -> sqlite::Value {
        println!("col.name: {}", col.name);
        match col.data_type {
            DataTypes::String => Value::String(col_value),
            DataTypes::I64 => {
                let value = match col_value.parse::<i64>() {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("i64 parse error: '{}' is not an int: {}", col_value, e);
                        0
                    }
                };
                Value::Integer(value)
            },
            DataTypes::F64 => Value::Float(col_value.parse::<f64>().unwrap()),
            DataTypes::Empty => Value::Null
        }
    }
}

#[cfg(test)]
mod tests {
    use workers::sqlite::SqliteDB;

    #[test]
    fn new() {
        let my_path = "/tmp/test.db";

        match SqliteDB::new(my_path) {
            Ok(db) => {
                assert_eq!(db.path, my_path);
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                assert!(false);
            }
        };
    }
}