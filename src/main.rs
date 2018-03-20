#![feature(plugin, custom_attribute, test)]
#![recursion_limit = "1024"]
extern crate byteorder;
extern crate core;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate getset;
extern crate ini;
#[macro_use]
extern crate nom;
extern crate parking_lot;
extern crate test;
extern crate winapi;

use failure::Error;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

mod windows;
mod ntfs;
mod parse_mft;
mod change_journal;
mod user_settings;
mod errors;

fn main() {
    if let Err(e) = run() {
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

fn run() -> Result<(()), Error> {
    let volume = "\\\\.\\C:";
//    let read_journal: JoinHandle<Result<(), Error>> = thread::Builder::new().name("read journal".to_string()).spawn(move || {
        let mut journal = change_journal::UsnJournal::new("\\\\.\\C:")?;
        journal.test();
//        loop {
//            let _x = journal.get_new_changes()?;
//            thread::sleep(Duration::from_secs(2));
//        }
//    })?;
//    read_journal.join().expect("reader journal  panic")?;
//    parse_mft::start(volume);
    Ok(())
}
