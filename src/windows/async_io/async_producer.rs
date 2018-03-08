use windows::async_io::buffer_pool::BufferPool;
use windows::async_io::iocp::{
    InputOperation,
};
use windows::utils;
use std::fs::{
    File,
    OpenOptions,
};
use std::io;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::AsRawHandle;
use std::path::Path;
use std::ptr;
use std::sync::{
    Arc,
    Mutex,
};
use winapi::shared::winerror::ERROR_IO_PENDING;
use winapi::um::fileapi::ReadFile;
use winapi::um::winbase::FILE_FLAG_OVERLAPPED;


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

pub struct AsyncProducer {
    pool: Arc<Mutex<BufferPool>>,
}

impl AsyncProducer {
    pub fn new(pool: Arc<Mutex<BufferPool>>) -> Self {
        AsyncProducer { pool }
    }

    pub fn read(&mut self, file: &AsyncFile, offset: u64) -> io::Result<()> {
        let buffer = self.pool.lock().unwrap().get().expect("TODO...");
        let length = buffer.len() as u32;
        let operation = Box::new(InputOperation::new(buffer, offset));
        let lp_buffer = operation.buffer;
        unsafe {
            let lp_overlapped = Box::into_raw(operation);
            println!("Read scheduled1");
            match ReadFile(
                file.file.as_raw_handle(),
                lp_buffer as *mut _,
                length,
                ptr::null_mut(),
                lp_overlapped as *mut _,
            ) {
                v if v == 0 => {
                    match utils::last_error::<i32>() {
                        Err(ref e) if e.raw_os_error() == Some(ERROR_IO_PENDING as i32) => Ok(()),
                        Ok(_) => Ok(()),
                        Err(e) => {
                            println!("Read failed");
                            Err(e)
                        }
                    }
                }
                _ => Ok(()),
            }
        }
    }
}
