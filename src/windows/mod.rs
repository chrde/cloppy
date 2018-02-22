use std::fs::File;
use std::os::windows::io::AsRawHandle;
use std::ptr;
use winapi::um::ioapiset::DeviceIoControl;
use winapi::um::winioctl::{
    FSCTL_GET_NTFS_VOLUME_DATA,
    FSCTL_QUERY_USN_JOURNAL,
};

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

