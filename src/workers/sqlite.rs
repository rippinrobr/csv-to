use actix::prelude::*;
use barrel::*;
use barrel::backend::Pg;
use csv_converter::models::{ColumnDef, DataTypes};
use sqlite;
use sqlite::{Connection, Error, Value};
use std::path::Path;
use std::vec::Vec;
use csv::StringRecord;
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

    fn get_value_type(col: &ColumnDef, col_value: String) -> sqlite::Value {
        match col.data_type {
            DataTypes::String => Value::String(col_value),
            DataTypes::I64 => {
                let value = match col_value.parse::<i64>() {
                    Ok(v) => v,
                    Err(e) => {
                        //eprintln!("WARNING: i64 parse error: {} => '{}' is not an int: {}", col.name,  col_value, e);
                        0
                    }
                };
                Value::Integer(value)
            },
            DataTypes::F64 => {
                let value = match col_value.parse::<f64>() {
                    Ok(v) => v,
                    Err(e) => {
                        //eprintln!("WARNING: f64 parse error: {} => '{}' f64 parse error: : {}", col.name, col_value, e);
                        0.0
                    }
                };
                Value::Float(value)
            },
            DataTypes::Empty => Value::Null
        }
    }

}

pub struct SQLGen;

impl Actor for SQLGen {
    type Context = Context<Self>;
}

impl SQLGen {
    pub fn generate_create_table(name: &str, columns: &Vec<ColumnDef>) -> Result<String, String> {
        if name == "" {
            return Err("[generate_create_table] name cannot be empty.".to_string());
        }
        
        if columns.len() == 0 {
            return Err("[generate_create_table] there must be at least 1 column.".to_string());
        }
        
        let mut m = Migration::new();
        let cols = columns.clone();
        m.create_table(name, move |t| {
            for cd in &cols {
                let cname: &str = &cd.name;
                t.add_column(cname, cd.data_type.to_database_type());
            }
        }).without_id();
        Ok(format!("{};", &m.make::<Pg>()))
    }

    pub fn generate_insert_stmt(name: &str, columns: &Vec<ColumnDef>) -> Result<String, String> { 
        if name == "" {
            return Err("[generate_insert_stmt] name cannot be empty.".to_string());
        }
        
        if columns.len() == 0 {
            return Err("[generate_insert_stmt] there must be at least 1 column.".to_string());
        }
        
        let mut placeholders: Vec<String> = Vec::new();
        for n in 0..columns.len() {
            placeholders.push(format!("?{}", n+1));
        }
        let col_names: Vec<String> = columns.into_iter().map(move |c| c.name.clone()).collect();
        let sql_stmt = format!("INSERT INTO {} ({}) VALUES ({})", name, col_names.join(", "), placeholders.join(", "));
        Ok(sql_stmt)
    }
}
pub struct SqliteCreateTable {
    pub columns: Vec<ColumnDef>,
    pub db_conn: SqliteDB,
    pub table_name: String,
}

impl Message for SqliteCreateTable {
    type Result = String;
}

impl Handler<SqliteCreateTable> for SQLGen {
    type Result = String;

    fn handle(&mut self, msg: SqliteCreateTable, _: &mut Context<Self>) -> Self::Result {
        let table_sql = match SQLGen::generate_create_table(&msg.table_name, &msg.columns) {
            Ok(create_table_sql) => {
                match msg.db_conn.create_table(create_table_sql.clone()) {
                    Ok(_) => println!("Created table {}", &msg.table_name),
                    Err(e) => eprintln!("Error creating table: {}", e)
                }
                create_table_sql
            },
            Err(e) => {
                eprintln!("[generate_create_table] Error: {}", e);
                String::from("")
            }
        };
        
        table_sql
    }
}

pub struct SqliteLoadTable {
    pub columns: Vec<ColumnDef>,
    pub content: Vec<Vec<String>>,
    pub db_conn: SqliteDB,
    pub table_name: String,
}

impl Message for SqliteLoadTable {
    type Result = String;
}

impl Handler<SqliteLoadTable> for SQLGen {
    type Result = String;

    fn handle(&mut self, msg: SqliteLoadTable, _: &mut Context<Self>) -> Self::Result {
        match SQLGen::generate_insert_stmt(&msg.table_name, &msg.columns) {
            Ok(insert_sql) => {
                match msg.db_conn.insert_rows(insert_sql, &msg.columns, msg.content) {
                    Ok(_) => println!("Loaded data into {}", &msg.table_name),
                    Err(e) => eprintln!("Error loading table: {}", e)
                }
                "".to_string()
            },
            Err(e) => {
                eprintln!("[SqliteLoadTable] Error: {}", e);
                String::from("")
            }
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
                assert!(true);
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                assert!(false);
            }
        };
    }
}