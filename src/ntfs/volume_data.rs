use byteorder::{LittleEndian, ByteOrder};

fn ntfs_volume_data(input: &[u8]) -> VolumeData {
    let bytes_per_sector = LittleEndian::read_u32(&input[0x28..]);
    let bytes_per_cluster = LittleEndian::read_u32(&input[0x2C..]);
    let bytes_per_file_record = LittleEndian::read_u32(&input[0x30..]);
    let mft_start_lcn = LittleEndian::read_u64(&input[0x40..]);
    VolumeData {
        mft_start_lcn,
        bytes_per_file_record,
        bytes_per_sector,
        bytes_per_cluster,
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VolumeData {
    pub mft_start_lcn: u64,
    pub bytes_per_cluster: u32,
    pub bytes_per_sector: u32,
    pub bytes_per_file_record: u32,
}

impl VolumeData {
    pub fn new(input: [u8; 128]) -> VolumeData {
        let bytes_per_sector = LittleEndian::read_u32(&input[0x28..]);
        let bytes_per_cluster = LittleEndian::read_u32(&input[0x2C..]);
        let bytes_per_file_record = LittleEndian::read_u32(&input[0x30..]);
        let mft_start_lcn = LittleEndian::read_u64(&input[0x40..]);
        VolumeData {
            mft_start_lcn,
            bytes_per_file_record,
            bytes_per_sector,
            bytes_per_cluster,
        }
    }
    pub fn initial_offset(&self) -> u64 {
        self.bytes_per_cluster as u64 * self.mft_start_lcn
    }

    pub fn clusters_per_fr(&self) -> u32 {
        self.bytes_per_cluster / self.bytes_per_file_record
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_attr() {
        let input = [206, 83, 254, 140, 132, 254, 140, 96, 255, 231, 245, 28, 0, 0, 0, 0, 255, 188, 158, 3, 0, 0, 0, 0, 1, 14, 94, 1, 0, 0, 0, 0, 95, 35, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 16, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 64, 133, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 192, 197, 247, 2, 0, 0, 0, 0, 224, 141, 248, 2, 0, 0, 0, 0, 32, 0, 0, 0, 3, 0, 1, 0, 0, 2, 0, 0, 2, 0, 0, 0, 0, 2, 0, 0, 255, 255, 255, 255, 62, 0, 0, 0, 0, 0, 0, 64];
        let output = VolumeData { mft_start_lcn: 786432, bytes_per_cluster: 4096, bytes_per_sector: 512, bytes_per_file_record: 1024 };
        assert_eq!(output, VolumeData::new(input));
    }
}
