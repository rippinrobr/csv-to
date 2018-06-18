use std::fs::File;
use std::io::Error;
use std::io::prelude::*;
use barrel::*;
use barrel::backend::Pg;
use models::ColumnDef;

pub struct SQLGen;

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

#[cfg(test)]
mod tests {
    use workers::sql_gen::SQLGen;
    use models::{ColumnDef, DataTypes};

    #[test] 
    fn generate_create_table() {
        let table_def = "CREATE TABLE \"people\" (\"name\" TEXT, \"age\" INTEGER, \"weight\" DOUBLE);".to_string();
        let cols: Vec<ColumnDef> = vec![ColumnDef::new("name".to_string(), DataTypes::String), ColumnDef::new("age".to_string(), DataTypes::I64), ColumnDef::new("weight".to_string(), DataTypes::F64)];
        ;
        match SQLGen::generate_create_table("people", &cols) {
            Ok(table) => assert_eq!(table_def, table),
            Err(_) => assert!(false)
        };
    }


    #[test] 
    fn generate_insert_stmt() {
        let insert_stmt = "INSERT INTO people (name, age, weight) VALUES (?1, ?2, ?3)".to_string();
        let cols: Vec<ColumnDef> = vec![ColumnDef::new("name".to_string(), DataTypes::String), ColumnDef::new("age".to_string(), DataTypes::I64), ColumnDef::new("weight".to_string(), DataTypes::F64)];
        match SQLGen::generate_insert_stmt("people", &cols) {
            Ok(stmt) => assert_eq!(insert_stmt, stmt),
            Err(_) => assert!(false)
        };
    }
}
