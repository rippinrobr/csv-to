extern crate barrel;

use barrel::backend::Pg;
use barrel::types;
use barrel::*;

use csv::StringRecord;
use failure::Error;
use postgres::{Connection, TlsMode};
use postgres::types::{ToSql};
use crate::models::{ColumnDef, DataTypes};
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
            Err(e) => Err(failure::err_msg(format!("exec: {}", e)))
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

    // get_to_sql converts the given col_val to appropriate type for
    // the col provided.  For numeric columns if a non integer or float is
    // provided in col_value the value of 0 or 0.0 will be returned.
//    fn get_pg_value(col: &ColumnDef, col_value: String) -> PgValue {
//        match col.data_type {
//            DataTypes::String => PgValue::new(col_value, DataTypes::String),
//            DataTypes::I64 => PgValue::new(col_value, DataTypes::I64),
//            DataTypes::F64 => PgValue::new(col_value, DataTypes::F64),
//            DataTypes::Empty => PgValue::new("".to_string(), DataTypes::Empty),
//        }
//    }
}

impl StorageService for PostgresStore {
    /// creates an insert or appropriate create statement for the backend store
    fn create_insert_stmt(&self, store_name: String, column_defs: Vec<ColumnDef>) -> String{
        let mut placeholders: Vec<String> = Vec::new();
        for n in 0..column_defs.len() {
            if column_defs[n].data_type == DataTypes::String {
                placeholders.push(format!("'${}'", n + 1));
            } else {
                placeholders.push(format!("${}", n + 1));
            }
        }

        let dbl_quote = String::from("\"");
        let col_names: Vec<String> = column_defs.into_iter().map(move |c| format!("\"{}\"", c.name.clone())).collect();
        format!("INSERT INTO \"{}\" ({}) VALUES ({})", store_name, col_names.join(", "), placeholders.join(", "))
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
//        let mut rows_inserted_count = 0;
//        //let mut stmt = self.conn.prepare(&insert_stmt).unwrap();
//
//        for vrec in data {
//            let mut insert_sql = insert_stmt.clone();
////            let mut row: Vec<&ToSql> = Vec::new();
////            // TODO: Replicate the getting of values from sqlite. I think thats what will get me
////            // around the issues I'm having now
//            let mut col_idx: usize = 1;
//            for c in vrec.iter() {
//                let placeholder = format!("${}", col_idx);
//                let cleaned_val: &str = &c.replace("'", "''");
//                let val = match cleaned_val {
//                    "" => "''",
//                    _ => c
//                };
//                insert_sql = insert_sql.replace(&placeholder, &val.replace("'", "''"));
//                col_idx += 1;
//            }
//
//            println!("{}", insert_sql);
//            match self.exec( insert_sql) {
//                Ok(_) => rows_inserted_count += 1,
//                Err(e) => eprintln!("insert error: {}", e)
//            }
//        }
//
//        Ok(rows_inserted_count)

        Err(failure::err_msg("[store data] not implemented"))
    }
}

//#[derive(Default)]
//struct PgValue {
//    pub val: String,
//    chosen_type: DataTypes,
//}
//
//impl PgValue {
//    fn new(val: String, chosen_type: DataTypes) -> PgValue {
//        PgValue{
//            val,
//            chosen_type,
//        }
//    }
//
//    fn string(self) -> String {
//        self.val
//    }
//
//    fn int(self) -> i64 {
//        match self.val.parse::<i64>() {
//            Ok(v) => v,
//            Err(_) => {
//                0
//            }
//        }
//    }
//
//    fn for_float(self) -> f64 {
//        match self.val.parse::<f64>() {
//            Ok(v) => v,
//            Err(_) => {
//                0.0
//            }
//        }
//    }
//
//    fn for_null() -> String {
//        String::from("null")
//    }
//}