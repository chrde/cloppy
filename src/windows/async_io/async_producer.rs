use windows::async_io::buffer_pool::BufferPool;
use windows::async_io::iocp::InputOperation;
use windows::{
    read_overlapped,
};
use std::fs::{
    File,
    OpenOptions,
};
use std::io;
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::{
    Arc,
};
use winapi::um::winbase::FILE_FLAG_OVERLAPPED;
use windows::async_io::iocp::IOCompletionPort;


pub struct AsyncFile {
    file: File,
    completion_key: usize,
}

impl AsyncFile {
    pub fn open<P: AsRef<Path>>(path: P, completion_key: usize) -> io::Result<Self> {
        OpenOptions::new().read(true).custom_flags(FILE_FLAG_OVERLAPPED).open(path)
            .map(|file| AsyncFile { file, completion_key })
    }

    pub fn file(&self) -> &File {
        &self.file
    }

    pub fn completion_key(&self) -> usize {
        self.completion_key
    }
}

pub struct AsyncReader {
    pool: BufferPool,
    iocp: Arc<IOCompletionPort>,
    file: AsyncFile,
}

impl AsyncReader {
    pub fn new(pool: BufferPool, iocp: Arc<IOCompletionPort>, file: AsyncFile) -> Self {
        AsyncReader { file, iocp, pool }
    }

    pub fn finish(&self) {
        let operation = Box::new(InputOperation::new(Vec::new(), 0));
        let lp_overlapped = Box::into_raw(operation);
        self.iocp.post(lp_overlapped as *mut _).unwrap();
    }


    pub fn read(&mut self, offset: u64) -> io::Result<()> {
        let buffer = self.pool.get().expect("TODO...");
        let length = buffer.len() as u32;
        let operation = Box::new(InputOperation::new(buffer, offset));
        let lp_buffer = operation.buffer;
        let lp_overlapped = Box::into_raw(operation);
        let result = read_overlapped(&self.file.file, lp_buffer, length, lp_overlapped as *mut _);
        result
    }
}
