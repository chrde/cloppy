use nom::{IResult, le_u32, le_u64};

fn ntfs_volume_data(input: &[u8]) -> IResult<&[u8], VolumeData> {
    do_parse!(input,
        take!(40) >>
        bytes_per_sector: le_u32 >>
        bytes_per_cluster: le_u32 >>
        bytes_per_file_record: le_u32 >>
        take!(12) >>
        mft_start_lcn: le_u64 >>
        (VolumeData{
            mft_start_lcn: mft_start_lcn,
            bytes_per_cluster: bytes_per_cluster,
            bytes_per_sector: bytes_per_sector,
            bytes_per_file_record: bytes_per_file_record,
        })
    )
}

#[derive(Copy, Clone)]
pub struct VolumeData {
    pub mft_start_lcn: u64,
    pub bytes_per_cluster: u32,
    pub bytes_per_sector: u32,
    pub bytes_per_file_record: u32,
}

impl VolumeData {
    pub fn new(input: [u8; 128]) -> VolumeData {
        ntfs_volume_data(&input).to_result().expect("Failed to create VolumeData")
    }
    pub fn initial_offset(&self) -> u64 {
        self.bytes_per_cluster as u64 * self.mft_start_lcn
    }

    pub fn clusters_per_fr(&self) -> u32 {
        self.bytes_per_cluster / self.bytes_per_file_record
    }
}
