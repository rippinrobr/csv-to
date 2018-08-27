
use actix::prelude::*;
use barrel::*;
use barrel::backend::Pg;
use csv_converter::{
    db::SqliteDB,
    models::{ColumnDef}
};
use std::vec::Vec;

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
    type Result = Result<String, String>;
}

impl Handler<SqliteCreateTable> for SQLGen {
    type Result = Result<String, String>;

    fn handle(&mut self, msg: SqliteCreateTable, _: &mut Context<Self>) -> Self::Result {
        match SQLGen::generate_create_table(&msg.table_name, &msg.columns) {
            Ok(create_table_sql) => {
                match msg.db_conn.create_table(create_table_sql.clone()) {
                    Ok(_) => {
                        println!("Created table {}", &msg.table_name);
                        return Ok("".to_string())
                    },
                    Err(e) => {
                        return Err(format!("ERROR creating table: {}", e))
                    }
                }
            },
            Err(e) => {
                eprintln!("[generate_create_table] Error: {}", e);
                return Err(String::from(""))
            }
        };
        
        // println!("results: {:?}", results);
        // results
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

