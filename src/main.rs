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
extern crate lazy_static;
#[macro_use]
extern crate typed_builder;
extern crate winapi;

use errors::failure_to_string;
use std::ffi::OsString;
use std::io;
use std::sync::mpsc;
use std::thread;

mod windows;
mod ntfs;
mod sql;
//mod user_settings;
mod errors;
mod gui;
mod resources;

fn main() {
    match try_main() {
        Ok(code) => ::std::process::exit(code),
        Err(err) => {
            let msg = format!("Error: {}", err);
            panic!(msg);
        }
    }
}

fn try_main() -> io::Result<i32> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        gui::init_wingui(sender).unwrap();
    });
    run_forever(receiver);
    Ok(0)
}

fn run_forever(receiver: mpsc::Receiver<OsString>) {
    loop {
        let event = match receiver.recv() {
            Ok(e) => e,
            Err(_) => {
                println!("Channel closed. Probably UI thread exit.");
                return;
            }
        };
        println!("{:?}", event);
    }
}

fn ntfs_main() {
    if let Err(e) = ntfs::start() {
        println!("{}", failure_to_string(e));
    }
}
