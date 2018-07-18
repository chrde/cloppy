use byteorder::{ByteOrder, LittleEndian};
use errors::MyErrorKind::*;
use failure::{Error, ResultExt};
use ntfs::windows_api::structs::*;
use std::fs::File;
use std::io;
use std::mem;
use std::os::windows::io::AsRawHandle;
use std::ptr;
use winapi::ctypes::c_void;
use winapi::shared::minwindef::BYTE;
use winapi::um::ioapiset::DeviceIoControl;
use winapi::um::winioctl::{
    FSCTL_GET_NTFS_FILE_RECORD,
    FSCTL_GET_NTFS_VOLUME_DATA,
    FSCTL_QUERY_USN_JOURNAL,
    FSCTL_READ_USN_JOURNAL,
    NTFS_FILE_RECORD_INPUT_BUFFER,
    NTFS_FILE_RECORD_OUTPUT_BUFFER,
};
use winapi::um::winnt::LARGE_INTEGER;

pub fn get_volume_data(file: &File) -> Result<[u8; 128], Error> {
    let mut output = [0u8; 128];
    let mut count = 0;
    match unsafe {
        DeviceIoControl(
            file.as_raw_handle(),
            FSCTL_GET_NTFS_VOLUME_DATA,
            ptr::null_mut(),
            0,
            output.as_mut_ptr() as *mut _,
            output.len() as u32,
            &mut count,
            ptr::null_mut(),
        )
    } {
        v if v == 0 || count != 128 => {
            Err(io::Error::last_os_error()).context(WindowsError("Failed to read volume data"))?
        }
        _ => Ok(output),
    }
}

pub fn get_file_record<'a>(v_handle: &File, fr_number: i64, buffer: &'a mut [u8]) -> Result<(&'a mut [u8]), Error> {
    let mut bytes_read = 0;
    let buffer_size = buffer.len();
    match unsafe {
        let mut fr = mem::zeroed::<LARGE_INTEGER>();
        *fr.QuadPart_mut() = fr_number as i64;
        let mut input = NTFS_FILE_RECORD_INPUT_BUFFER { FileReferenceNumber: fr };
        DeviceIoControl(
            v_handle.as_raw_handle(),
            FSCTL_GET_NTFS_FILE_RECORD,
            &mut input as *mut _ as *mut c_void,
            mem::size_of::<NTFS_FILE_RECORD_INPUT_BUFFER>() as u32,
            buffer.as_mut_ptr() as *mut c_void,
            buffer_size as u32,
            &mut bytes_read,
            ptr::null_mut(),
        )
    } {
        v if v == 0 => Err(io::Error::last_os_error()).context(WindowsError("Failed to get file record"))?,
        _ => {
            let size = mem::size_of::<NTFS_FILE_RECORD_OUTPUT_BUFFER>() - mem::size_of::<BYTE>() - 3;
            Ok(&mut buffer[size..])
        }
    }
}


pub fn read_usn_journal<'a>(v_handle: &File, start_at: i64, journal_id: u64, buf: &'a mut [u8]) -> Result<&'a [u8], Error> {
    let mut bytes_read = 0;
    let mut x = ReadUsnJournalDataV0::new(start_at, journal_id);
    match unsafe {
        DeviceIoControl(
            v_handle.as_raw_handle(),
            FSCTL_READ_USN_JOURNAL,
            &mut x as *mut _ as *mut c_void,
            mem::size_of::<ReadUsnJournalDataV0>() as u32,
            buf.as_mut_ptr() as *mut _,
            buf.len() as u32,
            &mut bytes_read,
            ptr::null_mut(),
        )
    } {
        v if v == 0 => Err(io::Error::last_os_error()).context(WindowsError("Failed to read usn_journal"))?,
        _ => Ok(&buf[..bytes_read as usize]),
    }
}


pub fn get_usn_journal(v_handle: &File) -> Result<UsnJournal, Error> {
    let mut output = [0u8; 80];
    let mut bytes_read = 0;
    unsafe {
        DeviceIoControl(
            v_handle.as_raw_handle(),
            FSCTL_QUERY_USN_JOURNAL,
            ptr::null_mut(),
            0,
            output.as_mut_ptr() as *mut _,
            output.len() as u32,
            &mut bytes_read,
            ptr::null_mut(),
        );
    }
    if bytes_read == 80 {
        let usn_journal_id = LittleEndian::read_u64(&output);
        let next_usn = LittleEndian::read_i64(&output[16..]);
        Ok(UsnJournal {
            usn_journal_id,
            next_usn,
        })
    } else {
        Err(WindowsError("Failed to query usn_journal"))?
    }
}