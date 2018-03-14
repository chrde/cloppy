use std::fs::File;
use std::os::windows::io::AsRawHandle;
use std::ptr;
use winapi::um::ioapiset::{
    DeviceIoControl,
};
use winapi::um::winioctl::{
    FSCTL_GET_NTFS_VOLUME_DATA,
    FSCTL_QUERY_USN_JOURNAL,
};
use winapi::um::shlobj::SHGetKnownFolderPath;
use winapi::um::knownfolders::FOLDERID_RoamingAppData;
use winapi::um::shlobj::KF_FLAG_DEFAULT;
use winapi::um::minwinbase::OVERLAPPED;
use winapi::um::fileapi::{
    ReadFile,
};
use std::path::PathBuf;
use windows::string::FromWide;
use winapi::shared::winerror::{
    SUCCEEDED,
    ERROR_IO_PENDING,
};
use std::io;

mod string;
pub mod async_io;
mod utils;

pub fn open_volume(file: &File) -> [u8; 128] {
    let mut output = [0u8; 128];
    let mut count = 0;
    unsafe {
        DeviceIoControl(
            file.as_raw_handle(),
            FSCTL_GET_NTFS_VOLUME_DATA,
            ptr::null_mut(),
            0,
            output.as_mut_ptr() as *mut _,
            output.len() as u32,
            &mut count,
            ptr::null_mut(),
        );
    }
    assert_eq!(count, 128);
    output
}

pub fn usn_journal_id(v_handle: &File) -> u64 {
    let mut output = [0u8; 80];
    let mut count = 0;
    unsafe {
        DeviceIoControl(
            v_handle.as_raw_handle(),
            FSCTL_QUERY_USN_JOURNAL,
            ptr::null_mut(),
            0,
            output.as_mut_ptr() as *mut _,
            output.len() as u32,
            &mut count,
            ptr::null_mut(),
        );
    }
    assert_eq!(count, 80);
    use std::io::Cursor;
    use byteorder::{LittleEndian, ReadBytesExt};
    Cursor::new(&output[..8]).read_u64::<LittleEndian>().expect("Failed to query usn_journal_id")
}

pub fn locate_user_data() -> io::Result<PathBuf> {
    unsafe {
        let mut string = ptr::null_mut();
        match SUCCEEDED(SHGetKnownFolderPath(&FOLDERID_RoamingAppData, KF_FLAG_DEFAULT, ptr::null_mut(), &mut string)) {
            true => Ok(PathBuf::from_wide_ptr_null(string)),
            false => Err(io::Error::new(io::ErrorKind::Other, "Failed to locate %APPDATA%"))
        }
    }
}

pub fn read_overlapped(file: &File, lp_buffer: *mut u8, length: u32, lp_overlapped: *mut OVERLAPPED) -> io::Result<()> {
    unsafe {
        match ReadFile(
            file.as_raw_handle(),
            lp_buffer as *mut _,
            length,
            ptr::null_mut(),
            lp_overlapped as *mut _,
        ) {
            v if v == 0 => {
                match utils::last_error::<i32>() {
                    Err(ref e) if e.raw_os_error() == Some(ERROR_IO_PENDING as i32) => Ok(()),
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            }
            _ => Ok(()),
        }
    }
}