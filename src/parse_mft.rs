use std::path::Path;
use windows::async_io::{
    AsyncFile,
    BufferPool,
    AsyncReader,
    IOCompletionPort,
    OutputOperation,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use ntfs::{
    VolumeData,
    read_all,
    read_mft,
    parse_file_record_basic,
};
use std::thread;
use rusqlite::Transaction;
use sql;
use ntfs::FileEntry;

//TODO make this value 'smart' depending on the HD
const SPEED_FACTOR: usize = 16;

pub fn parse_volume<P: AsRef<Path>>(path: P) -> Vec<FileEntry> {
    let (mft, volume) = read_mft(path.as_ref());

    let mut parser = MftParser::new(&mft, volume);
    let mut reader = parser.new_reader(path, 42);

    let read_thread = thread::Builder::new().name("producer".to_string()).spawn(move || {
        read_all(&mft, volume, &mut reader);
    }).unwrap();
    parser.parse_iocp_buffer();
    assert_eq!(parser.file_count, parser.files.len() as u32);
    read_thread.join().expect("reader panic");
    parser.files
}

struct MftParser {
    volume_data: VolumeData,
    file_count: u32,
    counter: Arc<AtomicUsize>,
    pool: BufferPool,
    iocp: Arc<IOCompletionPort>,
    files: Vec<FileEntry>,
}

impl MftParser {
    pub fn new(mft: &FileEntry, volume_data: VolumeData) -> Self {
        let counter = Arc::new(AtomicUsize::new(0));
        let pool = BufferPool::new(14, SPEED_FACTOR * volume_data.bytes_per_cluster as usize);
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

    pub fn new_reader<P: AsRef<Path>>(&mut self, file: P, completion_key: usize) -> AsyncReader {
        AsyncReader::new(self.pool.clone(), self.iocp.clone(), file, completion_key, self.counter.clone())
    }
    fn estimate_capacity(mft: &FileEntry, volume: &VolumeData) -> usize {
        let clusters = mft.dataruns.iter().map(|d| d.length_lcn as u32).sum::<u32>();
        (clusters * volume.bytes_per_cluster / volume.bytes_per_file_record) as usize
    }

    fn iocp_buffer_to_files(&mut self, operation: &mut OutputOperation) {
        for buff in operation.buffer_mut().chunks_mut(self.volume_data.bytes_per_file_record as usize) {
            let entry = parse_file_record_basic(buff, self.volume_data);
            if entry.id != 0 {
                self.files.push(entry);
                self.file_count += 1;
            }
        }
    }
}
