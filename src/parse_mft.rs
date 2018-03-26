use std::path::Path;
use windows::async_io::{
    AsyncFile,
    BufferPool,
    AsyncReader,
    IOCompletionPort,
    OutputOperation,
};
use std::sync::Arc;
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

pub fn start<P: AsRef<Path>>(path: P) {
    let (mft, volume) = read_mft(path.as_ref());

    let mut parser = MftParser::new(&mft, volume);
    let mut reader = parser.new_reader(path, 42);

    let read_thread = thread::Builder::new().name("producer".to_string()).spawn(move || {
        read_all(&mft, volume, &mut reader);
    }).unwrap();
    let consume_thread = thread::Builder::new().name("consumer".to_string()).spawn(move || {
        parser.parse_record();
        println!("{}", parser.count);
    }).unwrap();
    read_thread.join().expect("reader panic");
    consume_thread.join().expect("consumer panic");
}

struct MftParser {
    volume_data: VolumeData,
    count: u64,
    pool: BufferPool,
    iocp: Arc<IOCompletionPort>,
    files: Vec<FileEntry>,
}

impl MftParser {
    pub fn new(mft: &FileEntry, volume_data: VolumeData) -> Self {
        let pool = BufferPool::new(14, SPEED_FACTOR * volume_data.bytes_per_cluster as usize);
        let iocp = Arc::new(IOCompletionPort::new(1).unwrap());

        let files = Vec::with_capacity(MftParser::estimate_capacity(&mft, &volume_data));
        MftParser { volume_data, count: 0, pool: pool.clone(), iocp: iocp.clone(), files }
    }
    pub fn parse_record(&mut self) {
//        let mut connection = sql::main();
//        let tx = connection.transaction().unwrap();
        loop {
            let mut operation = self.iocp.get().unwrap();
            if operation.completion_key() != 42 {
                break;
            }
            self.consume(&mut operation);
            self.pool.put(operation.into_buffer());
        }
//        tx.commit().unwrap();
    }

    pub fn new_reader<P: AsRef<Path>>(&mut self, file: P, completion_key: usize) -> AsyncReader {
        AsyncReader::new(self.pool.clone(), self.iocp.clone(), file, completion_key)
    }
    pub fn estimate_capacity(mft: &FileEntry, volume: &VolumeData) -> usize {
        let clusters = mft.dataruns.iter().map(|d| d.length_lcn as u32).sum::<u32>();
        (clusters * volume.bytes_per_cluster / volume.bytes_per_file_record) as usize
    }

    fn consume(&mut self, operation: &mut OutputOperation) {
        for buff in operation.buffer_mut().chunks_mut(self.volume_data.bytes_per_file_record as usize) {
            let entry = parse_file_record_basic(buff, self.volume_data);
            if entry.id != 0 {
                self.files.push(entry);
//                sql::insert_file(tx, &entry);
                self.count += 1;
            }
        }
    }
}
