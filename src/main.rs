#![feature(plugin, custom_attribute, test)]
#![plugin(flamer)]
extern crate flame;
extern crate test;
extern crate parking_lot;
extern crate byteorder;
#[macro_use]
extern crate nom;
extern crate winapi;
extern crate ini;

mod windows;
mod ntfs;
mod user_settings;

use std::fs::File;
use windows::async_io::async_producer::AsyncReader;
use windows::async_io::buffer_pool::BufferPool;
use windows::async_io::Operation;
use std::sync::Arc;
use std::sync::Mutex;
use windows::async_io::Consumer;
use windows::async_io::OutputOperation;
use ntfs::VolumeData;


fn main() {
    println!("{:?}", windows::locate_user_data());
    let p = "\\\\.\\C:";
    {
        let operation = Operation::new(p);
        Operation::start(operation);
    }
    println!("kurwa");
//    let (mft, _) = ntfs::read_mft(p);
//    flame::start("main");
//    parser.parse(file_entry);
//    flame::end("main");
//    flame::dump_html(&mut File::create("flame-graph.html").unwrap()).unwrap();
//    let entry = parser.read_mft0();
//    println!("{:#?}", mft);
}

