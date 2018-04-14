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

use failure::Error;

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

pub fn failure_to_string(e: failure::Error) -> String {
    use std::fmt::Write;

    let mut result = String::new();
    for (i, cause) in e.causes().into_iter().enumerate() {
        if i > 0 {
            let _ = writeln!(&mut result, "\tCaused by: {}", cause);
        } else {
            let _ = writeln!(&mut result, "{}", cause);
        }
    }
    if let Some(bt) = e.cause().backtrace() {
        let _ = writeln!(&mut result, "{}", bt);
    }
    result
}
