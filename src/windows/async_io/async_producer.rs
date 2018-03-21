use windows::async_io::buffer_pool::BufferPool;
use windows::async_io::iocp::InputOperation;
use std::io;
use std::sync::{
    Arc,
};
use windows::async_io::iocp::IOCompletionPort;
use windows::async_io::iocp::AsyncFile;
use std::path::Path;

pub struct AsyncReader {
    pool: BufferPool,
    iocp: Arc<IOCompletionPort>,
    file: AsyncFile,
}

impl AsyncReader {
    pub fn new<P: AsRef<Path>>(pool: BufferPool, iocp: Arc<IOCompletionPort>, file_path: P, completion_key: usize) -> Self {
        let file = iocp.associate_file(file_path, 42).unwrap();
        AsyncReader { file, iocp, pool }
    }

    pub fn finish(&self) {
        let operation = Box::new(InputOperation::new(Vec::new(), 0));
        let lp_overlapped = Box::into_raw(operation);
        self.iocp.post(lp_overlapped as *mut _).unwrap();
    }


    pub fn read(&mut self, offset: u64) -> io::Result<()> {
        let buffer = self.pool.get().expect("TODO...");
        let operation = Box::new(InputOperation::new(buffer, offset));
        IOCompletionPort::submit(&self.file, operation)
    }
}
