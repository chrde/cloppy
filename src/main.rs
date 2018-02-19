#[macro_use]
extern crate nom;
extern crate byteorder;
extern crate winapi;
extern crate memmap;

mod windows;
mod ntfs;

use std::io::Cursor;
use std::io::SeekFrom;
use std::io::prelude::*;
use nom::IResult;
use nom::{le_u8, le_u16, le_u32, le_u64};
use byteorder::{LittleEndian, ReadBytesExt};
use std::ffi::OsString;
use ntfs::AttributeType;
use ntfs::Datarun;
use ntfs::Attribute;
use std::os::windows::prelude::*;
const DOS_NAMESPACE: u8 = 2;

#[derive(Default, Debug)]
pub struct FileEntry {
    id: u32,

    name: String,
    dos_flags: u32,
    parent_id: u64,
    real_size: u64,
    logical_size: u64,
    modified_date: u64,
    created_date: u64,
    dataruns: Vec<Datarun>,
}

impl FileEntry {
    fn new(attrs: Vec<Attribute>, id: u32) -> Self {
        let mut result = FileEntry::default();
        result.id = id;
        //TODO handle attribute flags (e.g: sparse or compressed)
        attrs.into_iter().fold(result, |mut acc, attr| {
            match attr.attr_type {
                AttributeType::Standard(val) => {
                    acc.dos_flags = val.dos_flags;
                    acc.modified_date = val.modified;
                    acc.created_date = val.created;
                    acc
                }
                AttributeType::Filename(val) => {
                    if val.namespace != DOS_NAMESPACE {
                        acc.name = val.name;
                        acc.parent_id = val.parent_id;
                        acc.real_size = val.real_size;
                        acc.logical_size = val.allocated_size;
                        acc.dos_flags = val.flags;
                    }
                    acc
                }
                AttributeType::Data(val) => {
                    acc.dataruns = val;
                    acc
                }
                _ => acc
            }
        })
    }
}

const SPEED_FACTOR: u64 = 4;

fn main() {
    let mut buffer: [u8; SPEED_FACTOR as usize * 1024] = [0; SPEED_FACTOR as usize * 1024];
    use std::time::Instant;
    let mut volume = windows::open_volume();
    println!("{}", windows::usn_journal_id(&volume.handle));
    let initial_offset = volume.initial_offset();
    volume.handle.seek(SeekFrom::Start(initial_offset)).unwrap();
    volume.handle.read_exact(&mut buffer).expect("fuck");

    let fr0 = ntfs::fixup_buffer(&mut buffer[..1024]);
    let mut absolute_lcn_offset = 0i64;
    let now = Instant::now();
    for (i, run) in fr0.dataruns.iter().enumerate() {
        absolute_lcn_offset += run.offset_lcn;
        let absolute_offset = absolute_lcn_offset as u64 * volume.bytes_per_cluster as u64;
        let file_record_count = run.length_lcn * volume.clusters_per_fr() as u64;
        println!("datarun {} started", file_record_count);
        for fr in 0..(file_record_count / SPEED_FACTOR) {
            volume.handle.seek(SeekFrom::Start(absolute_offset + SPEED_FACTOR * fr * volume.bytes_per_file_record as u64)).unwrap();
            volume.handle.read_exact(&mut buffer).expect(&fr.to_string());
            for buff in buffer.chunks_mut(1024) {
                ntfs::fixup_buffer(buff);
            }
        }
        println!("datarun {} finished", i);
        println!("total time {:?}", Instant::now().duration_since(now));
    }
}

