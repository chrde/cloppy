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
        println!("{:?}", operation.buffer)
    }
}

pub struct AsyncConsumer<T: Consumer> {
    pool: Arc<Mutex<BufferPool>>,
    iocp: Arc<Mutex<IOCompletionPort>>,
    consumer: T,
}

impl<T: Consumer> AsyncConsumer<T> {
    pub fn new(pool: Arc<Mutex<BufferPool>>, iocp: Arc<Mutex<IOCompletionPort>>, consumer: T) -> Self {
        AsyncConsumer { pool, iocp, consumer }
    }

    pub fn consume(&mut self) {
        let mut operation = self.iocp.lock().unwrap().get().unwrap();
        self.consumer.consume(&mut operation);
        self.pool.lock().unwrap().put(operation.buffer);
    }
}