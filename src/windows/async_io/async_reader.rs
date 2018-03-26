use windows::async_io::buffer_pool::BufferPool;
use windows::async_io::iocp::InputOperation;
use std::io;
use std::sync::{
    Arc,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use windows::async_io::iocp::IOCompletionPort;
use windows::async_io::iocp::AsyncFile;
use std::path::Path;

pub struct AsyncReader {
    pool: BufferPool,
    iocp: Arc<IOCompletionPort>,
    file: AsyncFile,
    counter: Arc<AtomicUsize>,
}

impl AsyncReader {
    pub fn new<P: AsRef<Path>>(pool: BufferPool, iocp: Arc<IOCompletionPort>, file_path: P, completion_key: usize, counter: Arc<AtomicUsize>) -> Self {
        let file = iocp.associate_file(file_path, 42).unwrap();
        AsyncReader { file, iocp, pool, counter }
    }

    pub fn finish(&self) {
        let operation = Box::new(InputOperation::new(Vec::new(), 0));
        self.iocp.post(operation, 99).unwrap();
    }


    pub fn read(&mut self, offset: u64) -> io::Result<()> {
        let buffer = self.pool.get();
        let operation = Box::new(InputOperation::new(buffer, offset));
        self.counter.fetch_add(1, Ordering::SeqCst);
        IOCompletionPort::submit(&self.file, operation)
    }
}
