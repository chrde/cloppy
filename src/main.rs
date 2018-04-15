#![feature(plugin, custom_attribute, test)]
#![allow(dead_code)]
#![recursion_limit = "1024"]
#[macro_use]
extern crate bitflags;
extern crate byteorder;
extern crate conv;
extern crate core;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate ini;
#[macro_use]
extern crate nom;
extern crate parking_lot;
extern crate rusqlite;
extern crate test;
extern crate time;
#[macro_use]
extern crate typed_builder;
extern crate winapi;

use errors::failure_to_string;

mod windows;
mod ntfs;
mod sql;
//mod user_settings;
mod errors;

fn ntfs_main() {
    if let Err(e) = ntfs::start() {
        println!("{}", failure_to_string(e));
    }
}

