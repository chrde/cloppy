use std::fs::File;
use byteorder::{ByteOrder, LittleEndian};
use std::os::windows::io::AsRawHandle;
use std::ptr;
use std::mem;
use winapi::um::ioapiset::DeviceIoControl;
use winapi::um::winioctl::{
    FSCTL_GET_NTFS_VOLUME_DATA,
    FSCTL_QUERY_USN_JOURNAL,
    FSCTL_GET_NTFS_FILE_RECORD,
    FSCTL_READ_USN_JOURNAL,
    NTFS_FILE_RECORD_INPUT_BUFFER,
    NTFS_FILE_RECORD_OUTPUT_BUFFER,
};
use winapi::um::winnt::LARGE_INTEGER;
use winapi::um::shlobj::SHGetKnownFolderPath;
use winapi::um::knownfolders::FOLDERID_RoamingAppData;
use winapi::um::shlobj::KF_FLAG_DEFAULT;
use winapi::um::winnt::USN;
use winapi::um::minwinbase::OVERLAPPED;
use winapi::um::fileapi::ReadFile;
use std::path::PathBuf;
use windows::string::FromWide;
use winapi::shared::winerror::{ERROR_IO_PENDING, SUCCEEDED};
use std::io;
use errors::MyErrorKind::*;
use failure::{err_msg, Error, ResultExt};
use winapi::ctypes::c_void;
use std::path::Path;
use std::slice;
use winapi::shared::minwindef::BYTE;
use std::fmt;

mod string;
pub mod async_io;
pub mod utils;

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
            utils::last_error().context(WindowsError("Failed to read volume data"))?
        }
        _ => Ok(output),
    }
}

bitflags! {
    pub struct WinUsnChanges: u32 {
        const FILE_CREATE= 0x00000100;
        const FILE_DELETE= 0x00000200;
        const RENAME_NEW_NAME= 0x00002000;
        const BASIC_INFO_CHANGE= 0x00008000;
        const CLOSE= 0x80000000;
    }
}

impl fmt::Display for WinUsnChanges {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "hi!")
    }
}

#[repr(C)]
struct ReadUsnJournalDataV0 {
    start: i64,
    reason_mask: u32,
    return_only_on_close: u32,
    timeout: u64,
    bytes_to_wait_for: u64,
    usn_journal_id: u64,
}

impl ReadUsnJournalDataV0 {
    fn new(start: i64, usn_journal_id: u64) -> Self {
        ReadUsnJournalDataV0 {
            start,
            reason_mask: WinUsnChanges::all().bits(),
            return_only_on_close: 1,
            timeout: 1,
            bytes_to_wait_for: 1,
            usn_journal_id,
        }
    }
}

pub fn get_file_record<'a>(v_handle: &File, fr_number: i64, buffer: &'a mut [u8]) -> Result<(&'a mut [u8], i64), Error> {
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
        v if v == 0 => utils::last_error().context(WindowsError("Failed to get file record"))?,
        _ => {
            let fr_number = LittleEndian::read_i64(buffer);
            let size = mem::size_of::<NTFS_FILE_RECORD_OUTPUT_BUFFER>() - mem::size_of::<BYTE>() - 3;
            Ok((&mut buffer[size..], fr_number))
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
        v if v == 0 => utils::last_error().context(WindowsError("Failed to read usn_journal"))?,
        _ => Ok(&buf[..bytes_read as usize]),
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UsnJournal {
    pub usn_journal_id: u64,
    pub next_usn: i64,
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

pub fn locate_user_data() -> Result<PathBuf, Error> {
    unsafe {
        let mut string = ptr::null_mut();
        match SUCCEEDED(SHGetKnownFolderPath(
            &FOLDERID_RoamingAppData,
            KF_FLAG_DEFAULT,
            ptr::null_mut(),
            &mut string,
        )) {
            true => Ok(PathBuf::from_wide_ptr_null(string)),
            false => Err(WindowsError("Failed to locate %APPDATA%"))?,
        }
    }
}

pub fn read_overlapped(
    file: &File,
    lp_buffer: *mut u8,
    length: u32,
    lp_overlapped: *mut OVERLAPPED,
) -> io::Result<()> {
    unsafe {
        match ReadFile(
            file.as_raw_handle(),
            lp_buffer as *mut _,
            length,
            ptr::null_mut(),
            lp_overlapped as *mut _,
        ) {
            v if v == 0 => match utils::last_error::<i32>() {
                Err(ref e) if e.raw_os_error() == Some(ERROR_IO_PENDING as i32) => Ok(()),
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            _ => Ok(()),
        }
    }
}
