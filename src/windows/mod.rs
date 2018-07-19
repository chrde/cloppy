use errors::MyErrorKind::*;
use failure::Error;
use failure::ResultExt;
use std::fs::File;
use std::io;
use std::os::windows::io::AsRawHandle;
use std::path::PathBuf;
use std::ptr;
use winapi::shared::winerror::{ERROR_IO_PENDING, SUCCEEDED};
use winapi::um::fileapi::ReadFile;
use winapi::um::knownfolders::FOLDERID_RoamingAppData;
use winapi::um::minwinbase::OVERLAPPED;
use winapi::um::shlobj::KF_FLAG_DEFAULT;
use winapi::um::shlobj::SHGetKnownFolderPath;
use windows::string::FromWide;

mod string;
pub mod async_io;
pub mod utils;

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
            false => Err(io::Error::last_os_error()).context(WindowsError("Failed to locate %APPDATA%"))?
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
