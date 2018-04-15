#![feature(plugin, custom_attribute, test)]
#![recursion_limit = "1024"]
extern crate byteorder;
extern crate core;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate ini;
#[macro_use]
extern crate nom;
extern crate parking_lot;
extern crate test;
extern crate winapi;
#[macro_use]
extern crate bitflags;
extern crate rusqlite;
extern crate time;

use errors::failure_to_string;

mod windows;
mod ntfs;
mod sql;
//mod user_settings;
mod errors;

fn main() {
    if let Err(e) = ntfs::start() {
        println!("{}", failure_to_string(e));
    }
}

