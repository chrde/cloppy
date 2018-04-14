#![feature(plugin, custom_attribute, test)]
#![recursion_limit = "1024"]
extern crate byteorder;
extern crate core;
#[macro_use]
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
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use sql::{
    insert_files,
    delete_file,
    insert_file,
    update_file,
};
use change_journal::UsnChange;

mod windows;
mod ntfs;
mod sql;
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

fn run() -> Result<(), Error> {
    let volume_path = "\\\\.\\C:";
    let mut sql_con = sql::main();
    {
        let files = parse_mft::parse_volume(volume_path);
        insert_files(&mut sql_con, &files);
    }
    let mut journal = change_journal::UsnJournal::new(volume_path)?;
    println!("usn journal");
    let read_journal: JoinHandle<Result<(), Error>> = thread::Builder::new().name("read journal".to_string()).spawn(move || {
        loop {
            let tx = sql_con.transaction().unwrap();
            let changes = journal.get_new_changes()?;
            for change in changes {
                if change != UsnChange::IGNORE {
                    println!("{:?}", change);
                }
                match change {
                    UsnChange::DELETE(id) => { delete_file(&tx, id) }
                    UsnChange::UPDATE(entry) => { update_file(&tx, &entry) }
                    UsnChange::NEW(entry) => { insert_file(&tx, &entry) }
                    UsnChange::IGNORE => {}
                }
            }
            tx.commit().unwrap();
        }
    })?;
    read_journal.join().unwrap();
    Ok(())
}