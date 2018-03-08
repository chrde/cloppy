use std::fs::File;
use std::os::windows::io::AsRawHandle;
use std::ptr;
use winapi::um::ioapiset::DeviceIoControl;
use winapi::um::winioctl::{
    FSCTL_GET_NTFS_VOLUME_DATA,
    FSCTL_QUERY_USN_JOURNAL,
};
use winapi::um::shlobj::SHGetKnownFolderPath;
use winapi::um::knownfolders::FOLDERID_RoamingAppData;
use winapi::um::shlobj::KF_FLAG_DEFAULT;
use winapi::um::fileapi::{
    ReadFile,
    CreateFileW,
    OPEN_EXISTING,
};
use  winapi::um::winnt::{
    FILE_SHARE_READ,
    FILE_SHARE_WRITE,
    FILE_SHARE_DELETE,
    GENERIC_READ,
    FILE_ATTRIBUTE_READONLY,
};
use  winapi::um::winbase::FILE_FLAG_NO_BUFFERING;
use std::path::PathBuf;
use windows::string::FromWide;
use winapi::shared::winerror::SUCCEEDED;
use std::io;
use windows::string::ToWide;
//use std::os::ext::io::FromRawHandle;
use std::os::windows::io::FromRawHandle;

mod string;
mod async_io;
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

pub fn open_file(name: &str) -> File {
    let mut output = [0u8; 128];
    let mut count = 0;
    unsafe {
        let result = CreateFileW(
            name.to_wide_null().as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            ptr::null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_READONLY |FILE_FLAG_NO_BUFFERING,
            ptr::null_mut(),
        );
        File::from_raw_handle(result)
    }
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

pub fn locate_user_data () -> io::Result<PathBuf> {
    unsafe {
        let mut string = ptr::null_mut();
        match SUCCEEDED(SHGetKnownFolderPath(&FOLDERID_RoamingAppData, KF_FLAG_DEFAULT, ptr::null_mut(),&mut string )){
            true => Ok(PathBuf::from_wide_ptr_null(string)),
            false => Err(io::Error::new(io::ErrorKind::Other, "Failed to locate %APPDATA%"))
        }
    }
}

pub fn read_file(file: &File, buffer: &mut [u8]) -> io::Result<()> {
    unsafe {
        let mut count = 0;
        match ReadFile(
            file.as_raw_handle(),
            buffer.as_mut_ptr() as *mut _,
            buffer.len() as u32,
            &mut count,
            ptr::null_mut(),
        ) {
            v if v == 0 =>Err(io::Error::last_os_error()),
            v => Ok(())
        }
    }

}