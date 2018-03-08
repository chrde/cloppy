#![feature(plugin, custom_attribute, test)]
#![plugin(flamer)]
extern crate flame;
extern crate test;
extern crate byteorder;
extern crate memmap;
#[macro_use]
extern crate nom;
extern crate winapi;
extern crate ini;

mod windows;
mod ntfs;
mod user_settings;
use std::fs::File;


fn main() {
    println!("{:?}", windows::locate_user_data());
    let p = "\\\\.\\C:";
    let mut parser = ntfs::MftParser::new(p);
    let file_entry = parser.read_mft0();
//    flame::start("main");
    parser.parse(file_entry);
//    flame::end("main");
//    flame::dump_html(&mut File::create("flame-graph.html").unwrap()).unwrap();
//    let entry = parser.read_mft0();
//    println!("{:#?}", entry);
}
