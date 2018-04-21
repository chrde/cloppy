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
use file_listing::Operation;
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
mod file_listing;

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
    let (req_snd, req_rcv) = mpsc::channel();
    thread::spawn(move || {
        gui::init_wingui(req_snd).unwrap();
    });
    run_forever(req_rcv);
    Ok(0)
}

fn run_forever(receiver: mpsc::Receiver<Message>) {
    let con = sql::main();
    let mut operation = file_listing::FileListing {};
    let mut wnd = None;
    loop {
        let event = match receiver.recv() {
            Ok(e) => e,
            Err(_) => {
                println!("Channel closed. Probably UI thread exit.");
                return;
            }
        };
        match event {
            Message::START(main_wnd) => wnd = Some(main_wnd),
            Message::MSG(v) => operation.handle(v, &con, wnd.as_ref().expect("Didnt receive START with main_wnd")),
        }
    }
}

fn main1() {
    if let Err(e) = ntfs::start() {
        println!("{}", failure_to_string(e));
    }
}

pub enum Message {
    START(gui::Wnd),
    MSG(OsString),
}
