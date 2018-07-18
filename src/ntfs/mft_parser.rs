use ntfs::file_record::FileRecord;
use ntfs::FR_AT_ONCE;
use ntfs::mft_reader::MftReader;
use ntfs::volume_data::VolumeData;
use slog::Logger;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use windows::async_io::{
    BufferPool,
    IOCompletionPort,
    OutputOperation,
};

pub struct MftParser {
    logger: Logger,
    volume_data: VolumeData,
    counter: Arc<AtomicUsize>,
    pool: BufferPool,
    iocp: Arc<IOCompletionPort>,
    pub candidates: HashMap<i64, FileRecord>,
    pub faulty: Vec<FileRecord>,
    pub files: Vec<FileRecord>,
}

impl MftParser {
    pub fn new(logger: Logger, mft: &FileRecord, volume_data: VolumeData) -> Self {
        let counter = Arc::new(AtomicUsize::new(0));
        let pool = BufferPool::new(16, FR_AT_ONCE as usize * volume_data.bytes_per_file_record as usize);
        let iocp = Arc::new(IOCompletionPort::new(1).unwrap());

        let candidates = HashMap::new();
        let faulty = Vec::new();
        let capacity = MftParser::estimate_capacity(&mft, &volume_data);
        info!(logger, "{:?}", volume_data; "estimated size" => capacity);
        let files = Vec::with_capacity(capacity);
        MftParser { volume_data, counter, pool: pool.clone(), iocp: iocp.clone(), files, candidates, faulty, logger }
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
        self.fix_dir_hardlinks();
    }

    pub fn fix_dir_hardlinks(&mut self) {
        assert_eq!(self.faulty.len(), self.candidates.len());
        for mut f in self.faulty.drain(..) {
            assert!(self.candidates.contains_key(&f.fr_number()));
            let mut fix = self.candidates.remove(&f.fr_number()).unwrap();
            f.name_attrs = fix.name_attrs;
            self.files.push(f);
        }
    }

    pub fn new_reader<P: AsRef<Path>>(&mut self, file: P, completion_key: usize) -> MftReader {
        MftReader::new(self.pool.clone(), self.iocp.clone(), file, completion_key, self.counter.clone(), self.logger.clone())
    }
    fn estimate_capacity(mft: &FileRecord, volume: &VolumeData) -> usize {
        let clusters = mft.data_attr.datarun.iter().map(|d| d.length_lcn as u32).sum::<u32>();
        (clusters * volume.bytes_per_cluster / volume.bytes_per_file_record) as usize
    }

    fn iocp_buffer_to_files(&mut self, operation: &mut OutputOperation) {
        let fr_count = operation.content_len();
        for buff in operation.buffer_mut().chunks_mut(self.volume_data.bytes_per_file_record as usize).take(fr_count) {
            if let Some(f) = FileRecord::parse_mft_entry(buff, self.volume_data) {
                if f.is_unused() {
                    continue;
                }
                if f.requires_name_fix() {
                    self.faulty.push(f);
                } else if f.is_candidate_for_fixes() {
                    self.candidates.insert(f.header.base_record as i64, f);
                } else {
                    self.files.push(f);
                }
            }
        }
    }
}
