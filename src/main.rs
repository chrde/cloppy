extern crate byteorder;
extern crate memmap;
#[macro_use]
extern crate nom;
extern crate winapi;

mod windows;
mod ntfs;


fn main() {
    let p = "\\\\.\\C:";
    let mut parser = ntfs::MftParser::new(p);
    parser.parse();
//    let entry = parser.read_mft0();
//    println!("{:#?}", entry);
}
