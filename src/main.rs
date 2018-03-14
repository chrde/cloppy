#![feature(plugin, custom_attribute, test)]
extern crate test;
extern crate parking_lot;
extern crate byteorder;
#[macro_use]
extern crate nom;
extern crate winapi;
extern crate ini;

mod windows;
mod ntfs;
mod parse_mft;
mod user_settings;

fn main() {
    println!("{:?}", windows::locate_user_data());
    let p = "\\\\.\\C:";
    parse_mft::start(p);
    println!("kurwa");
}

