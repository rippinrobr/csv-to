// TODO: This will become the struct that lays out where the results of the
// data is stored, typically it will be a database.  Maybe this should have
// a vec of output types as a field so I can havae SQLite, Postgress, MySQL, 
// etc...
// use std::fs::File;
// use std::io::Error;
// use std::io::prelude::*;

#[derive(Debug)]
pub struct Output {
    pub src_directory: String,
    sql_directory: String,
}

impl Output {
    pub fn new(src_directory: String, sql_directory: String) -> Output {
        Output{
            src_directory: src_directory,
            sql_directory: sql_directory,
        }
    }
}