use ntfs::VolumeData;
use std::fs::File;
use std::sync::{
    Arc,
    Mutex,
};
use std::thread;
use std::time;
use windows;
use ntfs;
pub use windows::async_io::async_consumer::{
    Consumer,
    DummyConsumer,
};
pub use windows::async_io::iocp::{
    OutputOperation
};
use windows::async_io::async_consumer::AsyncConsumer;
use windows::async_io::async_producer::{
    AsyncFile,
    AsyncReader,
};
use windows::async_io::buffer_pool::BufferPool;
use windows::async_io::iocp::IOCompletionPort;
use ntfs::FileEntry;
use std::path::Path;

mod iocp;
pub mod buffer_pool;
pub mod async_producer;
mod async_consumer;
use ntfs::FILENAME;

//TODO make this value 'smart' depending on the HD
const SPEED_FACTOR: usize = 1;

pub struct Operation {
    reader: AsyncReader,
//    consumer: AsyncConsumer<DummyConsumer>,
    iocp: IOCompletionPort,
    mft: FileEntry,
    volume: VolumeData,
}
impl Drop for Operation {
    fn drop(&mut self) {
        println!("dropping operation");
    }
}

impl Operation {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let (mft, volume) = ntfs::fake(path.as_ref());
//        let (mft, volume) = ntfs::read_mft(path.as_ref());
        let file = AsyncFile::open(path.as_ref(), 42).unwrap();
//        let pool = Arc::new(Mutex::new(BufferPool::new(1, 1024)));//SPEED_FACTOR * volume.bytes_per_cluster as usize)));
//        let iocp = Arc::new(IOCompletionPort::new(1).unwrap());
        let iocp = IOCompletionPort::new(1).unwrap();
        iocp.associate_file(&file).unwrap();

        let reader = AsyncReader::new(file);
//        let reader = AsyncReader::new(pool.clone(), iocp.clone(), file);
//        let consumer = AsyncConsumer::new(pool.clone(), iocp.clone(), MftParser{volume});
//        let consumer = AsyncConsumer::new(pool.clone(), iocp.clone(), DummyConsumer{});

        Operation { reader, iocp, mft, volume }
    }

    pub fn start(mut self) {
//        let mut reader = self.reader;
//        let mut consumer = self.consumer;
        let read_thread = thread::Builder::new().name("producer".to_string()).spawn(move || {
//            ntfs::read_all(&mft, volume, &mut reader);
            self.reader.read(4096 * 0xc0000).unwrap();
//            ::std::mem::forget(reader);
//            thread::sleep(time::Duration::from_millis(5000));
            println!("produce thread over");
            self
//            reader
        }).unwrap();
        println!("continue");
//        let consume_thread = thread::Builder::new().name("consumer".to_string()).spawn(move || {
//            thread::sleep(time::Duration::from_millis(5000));
//            consumer.consume();
//            ::std::mem::forget(consumer);
//            println!("consume thread over");
//            consumer
//        }).unwrap();
        let reader = read_thread.join().expect("reader panic");
//        let consumer = consume_thread.join();//.expect("consumer panic");
        println!("main exit");
    }
}

struct MftParser {
    volume: VolumeData,
}

impl Consumer for MftParser {
    fn consume(&mut self, operation: &mut OutputOperation) {
        for buff in operation.buffer.chunks_mut(self.volume.bytes_per_file_record as usize) {
            let entry = ntfs::parse_file_record(buff, self.volume, FILENAME);
            println!("{:?}", entry);
        }
    }
}
