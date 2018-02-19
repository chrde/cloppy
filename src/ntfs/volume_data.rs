use nom::{le_u32, le_u64, IResult};

#[derive(Debug)]
pub struct VolumeDataRaw {
    pub mft_start_lcn: u64,
    pub bytes_per_cluster: u32,
    pub bytes_per_sector: u32,
    pub bytes_per_file_record: u32,
}

impl VolumeDataRaw {
    pub fn new(input: &[u8]) -> Option<VolumeDataRaw> {
        ntfs_volume_data(input).to_result().ok()
    }
}

fn ntfs_volume_data(input: &[u8]) -> IResult<&[u8], VolumeDataRaw> {
    do_parse!(input,
        take!(40) >>
        bytes_per_sector: le_u32 >>
        bytes_per_cluster: le_u32 >>
        bytes_per_file_record: le_u32 >>
        take!(12) >>
        mft_start_lcn: le_u64 >>
        (VolumeDataRaw{
            mft_start_lcn: mft_start_lcn,
            bytes_per_cluster: bytes_per_cluster,
            bytes_per_sector: bytes_per_sector,
            bytes_per_file_record: bytes_per_file_record,
        })
    )
}
