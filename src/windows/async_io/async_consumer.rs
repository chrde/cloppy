use windows::async_io::iocp::OutputOperation;
use windows::async_io::buffer_pool::BufferPool;
use windows::async_io::iocp::IOCompletionPort;
use std::sync::{
    Arc,
};

pub trait Consumer {
    fn consume(&mut self, operation: &mut OutputOperation);
}

pub struct AsyncConsumer<T: Consumer + Send + 'static> {
    pool: BufferPool,
    iocp: Arc<IOCompletionPort>,
    pub consumer: T,
}

impl<T: Consumer + Send + 'static> AsyncConsumer<T> {
    pub fn new(pool: BufferPool, iocp: Arc<IOCompletionPort>, consumer: T) -> Self {
        AsyncConsumer { pool, iocp, consumer }
    }

    pub fn consume(&mut self) {
        loop {
            let mut operation = self.iocp.get().unwrap();
            if operation.completion_key != 42 {
                println!("{}", operation.completion_key);
                break;
            }
            self.consumer.consume(&mut operation);
            self.pool.put(operation.buffer);
        }
    }
}
