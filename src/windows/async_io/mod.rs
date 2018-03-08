mod iocp;
mod buffer_pool;
mod async_producer;
mod async_consumer;

use windows::async_io::async_consumer::AsyncConsumer;
pub use windows::async_io::async_consumer::{
    Consumer,
    DummyConsumer,
};
use windows::async_io::async_producer::{
    AsyncFile,
    AsyncProducer,
};
use windows::async_io::buffer_pool::BufferPool;
use windows::async_io::iocp::IOCompletionPort;
use std::sync::{
    Arc,
    Mutex,
};
use std::thread;

pub fn process_file<T>(consumer: T)
    where T: Consumer + Send + 'static {
    let file = AsyncFile::open("\\\\.\\C:", 0).unwrap();
    let pool = BufferPool::new(1, 1024);
    let iocp = IOCompletionPort::new(1).unwrap();
    iocp.associate_file(&file).unwrap();

    let iocp = Arc::new(Mutex::new(iocp));
    let pool = Arc::new(Mutex::new(pool));

    let mut reader = AsyncProducer::new(pool.clone());
    let mut consumer = AsyncConsumer::new(pool.clone(), iocp.clone(), consumer);

    let read_thread = thread::spawn(move || {
        println!("produce thread");
        reader.read(&file, 4096 * 0xc0000).unwrap();
    });
    let consume_thread = thread::spawn(move || {
        println!("consume thread");
        consumer.consume();
    });
    consume_thread.join().unwrap();
    read_thread.join().unwrap();
    assert_eq!(pool.lock().unwrap().len(), 1);
}