pub mod config;
pub mod input;

#[macro_use]
extern crate serde_derive;

pub fn sound_check() {
    println!("Is this thing on?");
}