pub mod code_gen;
pub mod config;
pub mod input;
pub mod models;
pub mod output;
pub mod parse_csv;
pub mod db;
pub mod sqlite_code_gen;
pub mod sql_gen;

extern crate barrel;
extern crate codegen;
extern crate csv;
extern crate futures;
extern crate regex;
extern crate sqlite;

#[macro_use]
extern crate serde_derive;

pub fn sound_check() {
    println!("Is this thing on?");
}