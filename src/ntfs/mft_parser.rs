use ntfs::file_entry::FileEntry;
use ntfs::file_record::file_record;
use ntfs::FR_AT_ONCE;
use ntfs::mft_reader::MftReader;
use ntfs::volume_data::VolumeData;
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
    volume_data: VolumeData,
    counter: Arc<AtomicUsize>,
    pool: BufferPool,
    iocp: Arc<IOCompletionPort>,
    pub candidates: HashMap<i64, FileEntry>,
    pub faulty: Vec<FileEntry>,
    pub files: Vec<FileEntry>,
}

impl MftParser {
    pub fn new(mft: &FileEntry, volume_data: VolumeData) -> Self {
        let counter = Arc::new(AtomicUsize::new(0));
        let pool = BufferPool::new(16, FR_AT_ONCE as usize * volume_data.bytes_per_file_record as usize);
        let iocp = Arc::new(IOCompletionPort::new(1).unwrap());

        let candidates = HashMap::new();
        let faulty = Vec::new();
        let files = Vec::with_capacity(MftParser::estimate_capacity(&mft, &volume_data));
        MftParser { volume_data, counter, pool: pool.clone(), iocp: iocp.clone(), files, candidates, faulty }
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
            assert!(self.candidates.contains_key(&f.fr_number));
            let mut fix = self.candidates.remove(&f.fr_number).unwrap();
            f.names = fix.names;
            self.files.push(f);
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
        let fr_count = operation.content_len();
        for buff in operation.buffer_mut().chunks_mut(self.volume_data.bytes_per_file_record as usize).take(fr_count) {
            if let Some(f) = file_record(buff, self.volume_data) {
                if f.is_unused() {
                    continue;
                }
                if f.requires_name_fix() {
                    self.faulty.push(f);
                } else if f.is_candidate_for_fixes() {
                    self.candidates.insert(f.base_record, f);
                } else {
                    self.files.push(f);
                }
            }
        }
    }
}
