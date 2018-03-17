use std::path::Path;
use windows::async_io::{
    AsyncFile,
    BufferPool,
    AsyncReader,
    IOCompletionPort,
    AsyncConsumer,
    OutputOperation,
    Consumer,
};
use std::sync::Arc;
use ntfs::{
    VolumeData,
    read_all,
    read_mft,
    parse_file_record_basic,
};
use std::thread;

//TODO make this value 'smart' depending on the HD
const SPEED_FACTOR: usize = 16;

pub fn start<P: AsRef<Path>>(path: P) {
    let (mft, volume) = read_mft(path.as_ref());
    let file = AsyncFile::open(path.as_ref(), 42).unwrap();
    let pool = BufferPool::new(14, SPEED_FACTOR * volume.bytes_per_cluster as usize);
    let iocp = Arc::new(IOCompletionPort::new(1).unwrap());
    iocp.associate_file(&file).unwrap();

    let mut reader = AsyncReader::new(pool.clone(), iocp.clone(), file);
    let mut consumer = AsyncConsumer::new(pool.clone(), iocp.clone(), MftParser { volume, count: 0 });

    let read_thread = thread::Builder::new().name("producer".to_string()).spawn(move || {
        read_all(&mft, volume, &mut reader);
    }).unwrap();
    let consume_thread = thread::Builder::new().name("consumer".to_string()).spawn(move || {
        consumer.consume();
        println!("{}", consumer.consumer.count);
    }).unwrap();
    read_thread.join().expect("reader panic");
    consume_thread.join().expect("consumer panic");
}

struct MftParser {
    volume: VolumeData,
    count: u64,
}

impl Consumer for MftParser {
    fn consume(&mut self, operation: &mut OutputOperation) {
        for buff in operation.buffer.chunks_mut(self.volume.bytes_per_file_record as usize) {
            let entry = parse_file_record_basic(buff, self.volume);
            if entry.id != 0 {
                self.count += 1;
            }
        }
    }
}
