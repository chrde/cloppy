use windows::async_io::iocp::OutputOperation;
use windows::async_io::buffer_pool::BufferPool;
use windows::async_io::iocp::IOCompletionPort;
use std::sync::{
    Arc,
    Mutex,
};

pub trait Consumer {
    fn consume(&mut self, operation: &mut OutputOperation);
}

pub struct DummyConsumer {}

impl Consumer for DummyConsumer {
    fn consume(&mut self, operation: &mut OutputOperation) {
        println!("consumed {}", operation.buffer.len());
    }
}

pub struct AsyncConsumer<T: Consumer + Send + 'static> {
    pool: Arc<Mutex<BufferPool>>,
    iocp: Arc<IOCompletionPort>,
    consumer: T,
}

impl<T: Consumer + Send + 'static> AsyncConsumer<T> {
    pub fn new(pool: Arc<Mutex<BufferPool>>, iocp: Arc<IOCompletionPort>, consumer: T) -> Self {
        AsyncConsumer { pool, iocp, consumer }
    }

    pub fn consume(&mut self) {
//        loop {
        let mut operation = self.iocp.get().unwrap();
        println!("one");
        self.consumer.consume(&mut operation);
        println!("two");
        self.pool.lock().unwrap().put(operation.buffer);
        println!("three");
//        }
    }
}

impl <T: Consumer + Send + 'static>Drop for AsyncConsumer<T> {
    fn drop(&mut self) {
        println!("drop async consumer");
    }
}