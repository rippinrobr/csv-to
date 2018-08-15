pub mod code_gen;
pub mod config;
pub mod input;
pub mod models;
pub mod output;

extern crate barrel;
extern crate codegen;
extern crate csv;
extern crate futures;
#[macro_use]
extern crate serde_derive;

pub fn sound_check() {
    println!("Is this thing on?");
}