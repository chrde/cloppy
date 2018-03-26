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

//TODO make this value 'smart' depending on the HD
const SPEED_FACTOR: usize = 16;

pub fn start<P: AsRef<Path>>(path: P) {
    let (mft, volume) = read_mft(path.as_ref());
    let pool = BufferPool::new(14, SPEED_FACTOR * volume.bytes_per_cluster as usize);
    let iocp = Arc::new(IOCompletionPort::new(1).unwrap());

    let mut reader = AsyncReader::new(pool.clone(), iocp.clone(), path, 42);
    let mut parser = MftParser { volume, count: 0, pool: pool.clone(), iocp: iocp.clone() };
//    let mut consumer = AsyncConsumer::new(pool.clone(), iocp.clone(), parser);

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
    volume: VolumeData,
    count: u64,
    pool: BufferPool,
    iocp: Arc<IOCompletionPort>,
}

impl MftParser {
    pub fn parse_record(&mut self) {
        let mut connection = sql::main();
        let tx = connection.transaction().unwrap();
        loop {
            let mut operation = self.iocp.get().unwrap();
            if operation.completion_key() != 42 {
                break;
            }
            self.consume(&mut operation, &tx);
            self.pool.put(operation.into_buffer());
        }
        tx.commit().unwrap();

    }

    fn consume(&mut self, operation: &mut OutputOperation, tx: &Transaction) {
        for buff in operation.buffer_mut().chunks_mut(self.volume.bytes_per_file_record as usize) {
            let entry = parse_file_record_basic(buff, self.volume);
            if entry.id != 0 {
                sql::insert_file(tx, &entry);
                self.count += 1;
            }
        }
    }
}
