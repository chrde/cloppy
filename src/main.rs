#![allow(dead_code)]
extern crate conv;
#[macro_use]
extern crate lazy_static;
extern crate parking_lot;
#[macro_use]
extern crate typed_builder;
extern crate winapi;

use std::ffi::OsString;
use std::io;
use std::sync::mpsc;
use std::thread;

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

