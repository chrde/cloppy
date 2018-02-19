use std::fs::File;
use std::os::windows::io::AsRawHandle;
use std::ptr;
use winapi::um::ioapiset::DeviceIoControl;
use winapi::um::winioctl::{
    FSCTL_GET_NTFS_VOLUME_DATA,
    FSCTL_QUERY_USN_JOURNAL,
};
use byteorder;
use ntfs;

pub fn open_volume() -> VolumeData {
    let f = File::open("\\\\.\\C:").expect("Failed to open volume handle");
    let mut output = [0u8; 128];
    let mut count = 0;
    unsafe {
        DeviceIoControl(
            f.as_raw_handle(),
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
    let raw = ntfs::VolumeDataRaw::new(&output).expect("Failed to create VolumeData");
    VolumeData::new(raw, f)
}

pub fn usn_journal_id(v_handle: &File) -> u64{
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

pub struct VolumeData {
    mft_start_lcn: u64,
    pub bytes_per_cluster: u32,
    pub bytes_per_sector: u32,
    pub bytes_per_file_record: u32,
    pub handle: File,
}

impl VolumeData {
    fn new(data: ntfs::VolumeDataRaw, handle: File) -> VolumeData {
        VolumeData {
            mft_start_lcn: data.mft_start_lcn,
            bytes_per_cluster: data.bytes_per_cluster,
            bytes_per_sector: data.bytes_per_sector,
            bytes_per_file_record: data.bytes_per_file_record,
            handle,
        }
    }
    pub fn initial_offset(&self) -> u64 {
        self.bytes_per_cluster as u64 * self.mft_start_lcn
    }

    pub fn clusters_per_fr(&self) -> u32 {
        self.bytes_per_cluster / self.bytes_per_file_record
    }
    pub fn sectors_per_cluster(&self) -> u32 {
        self.bytes_per_cluster / self.bytes_per_sector
    }
}
