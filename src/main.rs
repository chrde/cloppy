extern crate byteorder;
extern crate memmap;
#[macro_use]
extern crate nom;
extern crate winapi;
extern crate ini;

mod windows;
mod ntfs;
mod user_settings;


fn main() {
    println!("{:?}", windows::locate_user_data());
//    let p = "\\\\.\\C:";
//    let mut parser = ntfs::MftParser::new(p);
//    parser.parse();
//    let entry = parser.read_mft0();
//    println!("{:#?}", entry);
}
