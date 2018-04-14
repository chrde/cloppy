use std::path::Path;
use windows::async_io::{
    BufferPool,
    IOCompletionPort,
    OutputOperation,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use ntfs::FileEntry;
use ntfs::mft_reader::MftReader;
use ntfs::file_record::parse_file_record;
use ntfs::volume_data::VolumeData;

pub struct MftParser {
    volume_data: VolumeData,
    pub file_count: u32,
    counter: Arc<AtomicUsize>,
    pool: BufferPool,
    iocp: Arc<IOCompletionPort>,
    pub files: Vec<FileEntry>,
}

const FR_AT_ONCE: usize = 16;

impl MftParser {
    pub fn new(mft: &FileEntry, volume_data: VolumeData) -> Self {
        let counter = Arc::new(AtomicUsize::new(0));
        let pool = BufferPool::new(14, FR_AT_ONCE as usize * volume_data.bytes_per_cluster as usize);
        let iocp = Arc::new(IOCompletionPort::new(1).unwrap());

        let files = Vec::with_capacity(MftParser::estimate_capacity(&mft, &volume_data));
        MftParser { volume_data, file_count: 0, counter, pool: pool.clone(), iocp: iocp.clone(), files }
    }
    pub fn parse_iocp_buffer(&mut self) {
        let mut operations_count = 0;
        let mut finish = false;
        let mut end = false;
        while !end {
            operations_count += 1;
            let mut operation = self.iocp.get().unwrap();
            if operation.completion_key() != 42 {
                finish = true;
            }
            self.iocp_buffer_to_files(&mut operation);
            self.pool.put(operation.into_buffer());
            end = finish && operations_count == self.counter.load(Ordering::SeqCst);
        }
    }

    pub fn new_reader<P: AsRef<Path>>(&mut self, file: P, completion_key: usize) -> MftReader {
        MftReader::new(self.pool.clone(), self.iocp.clone(), file, completion_key, self.counter.clone())
    }
    fn estimate_capacity(mft: &FileEntry, volume: &VolumeData) -> usize {
        let clusters = mft.dataruns.iter().map(|d| d.length_lcn as u32).sum::<u32>();
        (clusters * volume.bytes_per_cluster / volume.bytes_per_file_record) as usize
    }

    fn iocp_buffer_to_files(&mut self, operation: &mut OutputOperation) {
        for buff in operation.buffer_mut().chunks_mut(self.volume_data.bytes_per_file_record as usize) {
            let entry = parse_file_record(buff, self.volume_data);
            if entry.id != 0 {
                self.files.push(entry);
                self.file_count += 1;
            }
        }
    }
}
