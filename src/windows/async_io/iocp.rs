use std::io;
use std::mem;
use std::ptr;
use std::os::windows::io::AsRawHandle;
use winapi::um::winbase::INFINITE;
use winapi::um::minwinbase::OVERLAPPED;
use windows::utils;
use windows::async_io::async_producer::AsyncFile;
use winapi::um::winnt::HANDLE;
use winapi::um::ioapiset::{
    CreateIoCompletionPort,
    GetQueuedCompletionStatus,
};
use winapi::um::handleapi::{
    INVALID_HANDLE_VALUE,
    CloseHandle,
};

unsafe impl Send for IOCompletionPort {}
unsafe impl Sync for IOCompletionPort {}

pub struct IOCompletionPort(HANDLE);

#[repr(C)]
pub struct InputOperation {
    overlapped: OVERLAPPED,
    pub buffer: *mut u8,
    len: usize,
    capacity: usize,
}

pub struct OutputOperation {
    overlapped: OVERLAPPED,
    completion_key: usize,
    bytes_read: u32,
    pub buffer: Vec<u8>,
}

impl InputOperation {
    pub fn new(mut buffer: Vec<u8>, offset: u64) -> Self {
        let mut overlapped;
        unsafe {
            overlapped = mem::zeroed::<OVERLAPPED>();
            let s = overlapped.u.s_mut();
            s.Offset = offset as u32;
            s.OffsetHigh = (offset >> 32) as u32;
        };
        InputOperation {
            overlapped,
            buffer: buffer.as_mut_ptr(),
            len: buffer.len(),
            capacity: buffer.capacity(),
        }
    }
}
impl Drop for InputOperation {
    fn drop(&mut self) {
        println!("closing input operation");
//        unsafe { CloseHandle(self.0) };
    }
}

impl IOCompletionPort {
    pub fn new(threads: u32) -> io::Result<Self> {
        unsafe {
            match CreateIoCompletionPort(
                INVALID_HANDLE_VALUE,
                ptr::null_mut(),
                0,
                threads,
            ) {
                v if v.is_null() => utils::last_error(),
                v => Ok(IOCompletionPort(v))
            }
        }
    }
    pub fn associate_file(&self, file: &AsyncFile) -> io::Result<()> {
        unsafe {
            match CreateIoCompletionPort(
                file.file().as_raw_handle(),
                self.0,
                file.completion_key(),
                0,
            ) {
                v if v.is_null() => utils::last_error(),
                _ => Ok(())
            }
        }
    }
    pub fn get(&self) -> io::Result<OutputOperation> {
        let mut bytes_read = 0;
        let mut completion_key = 0;
        unsafe {
            let mut overlapped = ptr::null_mut();
            match GetQueuedCompletionStatus(
                self.0,
                &mut bytes_read,
                &mut completion_key,
                &mut overlapped,
                INFINITE,
            ) {
                v if v == 0 => utils::last_error(),
                _ => {
                    let x = Box::from_raw(overlapped as *mut InputOperation);
                    println!("{} {}", x.len, x.capacity);
                    let buffer = Vec::from_raw_parts(x.buffer, x.len, x.capacity);
                    Ok(OutputOperation {
                        overlapped: x.overlapped,
                        completion_key,
                        bytes_read,
                        buffer,
                    })
                }
            }
        }
    }
}

impl Drop for IOCompletionPort {
    fn drop(&mut self) {
        println!("closing iocp");
        unsafe { CloseHandle(self.0) };
    }
}
