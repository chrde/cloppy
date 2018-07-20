use errors::MyErrorKind::WindowsError;
use failure::{Error, ResultExt};
use std::fs::{
    File,
    OpenOptions,
};
use std::io;
use std::mem;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::AsRawHandle;
use std::path::Path;
use std::ptr;
use std::slice;
use winapi::shared::basetsd::ULONG_PTR;
use winapi::um::handleapi::{
    CloseHandle,
    INVALID_HANDLE_VALUE,
};
use winapi::um::ioapiset::{
    CreateIoCompletionPort,
    GetQueuedCompletionStatus,
    GetQueuedCompletionStatusEx,
    PostQueuedCompletionStatus,
};
use winapi::um::minwinbase::{
    OVERLAPPED,
    OVERLAPPED_ENTRY,
};
use winapi::um::winbase::FILE_FLAG_OVERLAPPED;
use winapi::um::winbase::INFINITE;
use winapi::um::winnt::HANDLE;
use windows::read_overlapped;

unsafe impl Send for IOCompletionPort {}

unsafe impl Sync for IOCompletionPort {}

pub struct IOCompletionPort(HANDLE);

#[repr(C)]
pub struct InputOperation {
    overlapped: OVERLAPPED,
    content_len: usize,
    pub buffer: *mut u8,
    buffer_len: usize,
    buffer_capacity: usize,
}

pub struct OutputOperation(OVERLAPPED_ENTRY);

impl OutputOperation {
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        unsafe {
            let op = &mut *(self.0.lpOverlapped as *mut InputOperation);
            slice::from_raw_parts_mut(op.buffer, op.buffer_len)
        }
    }

    pub fn into_buffer(self) -> Vec<u8> {
        unsafe {
            let op = Box::from_raw(self.0.lpOverlapped as *mut InputOperation);
            Vec::from_raw_parts(op.buffer, op.buffer_len, op.buffer_capacity)
        }
    }

    pub fn content_len(&self) -> usize {
        unsafe {
            let op = &mut *(self.0.lpOverlapped as *mut InputOperation);
            op.content_len
        }
    }

    pub fn completion_key(&self) -> usize {
        self.0.lpCompletionKey as usize
    }
}

pub struct AsyncFile {
    pub file: File,
    pub completion_key: usize,
}

impl InputOperation {
    pub fn empty() -> Self {
        InputOperation::new(Vec::new(), 0, 0)
    }
    pub fn new(mut buffer: Vec<u8>, offset: u64, content_len: usize) -> Self {
        let mut overlapped;
        unsafe {
            overlapped = mem::zeroed::<OVERLAPPED>();
            let s = overlapped.u.s_mut();
            s.Offset = offset as u32;
            s.OffsetHigh = (offset >> 32) as u32;
        };
        let res = InputOperation {
            overlapped,
            content_len,
            buffer: buffer.as_mut_ptr(),
            buffer_len: buffer.len(),
            buffer_capacity: buffer.capacity(),
        };
        ::std::mem::forget(buffer);
        res
    }
}

impl IOCompletionPort {
    pub fn submit(file: &AsyncFile, operation: Box<InputOperation>) -> Result<(), Error> {
        let length = operation.buffer_len as u32;
        let lp_buffer = operation.buffer;
        let lp_overlapped = Box::into_raw(operation);
        read_overlapped(&file.file, lp_buffer, length, lp_overlapped as *mut _)
    }

    pub fn new(threads: u32) -> Result<Self, Error> {
        unsafe {
            match CreateIoCompletionPort(
                INVALID_HANDLE_VALUE,
                ptr::null_mut(),
                0,
                threads,
            ) {
                v if v.is_null() => Err(io::Error::last_os_error()).context(WindowsError("CreateIoCompletionPort -creation failed"))?,
                v => Ok(IOCompletionPort(v))
            }
        }
    }

    pub fn associate_file<P: AsRef<Path>>(&self, file_path: P, completion_key: usize) -> Result<AsyncFile, Error> {
        let file = OpenOptions::new().read(true).custom_flags(FILE_FLAG_OVERLAPPED).open(file_path)
            .map(|file| AsyncFile { file, completion_key }).unwrap();
        unsafe {
            match CreateIoCompletionPort(
                file.file.as_raw_handle(),
                self.0,
                file.completion_key,
                0,
            ) {
                v if v.is_null() => Err(io::Error::last_os_error()).context(WindowsError("CreateIoCompletionPort - associating a file failed"))?,
                _ => Ok(file)
            }
        }
    }

    pub fn post(&self, operation: Box<InputOperation>, completion_key: usize) -> Result<(), Error> {
        let lp_overlapped = Box::into_raw(operation) as *mut _;
        unsafe {
            match PostQueuedCompletionStatus(
                self.0,
                0,
                completion_key as ULONG_PTR,
                lp_overlapped,
            ) {
                v if v == 0 => Err(io::Error::last_os_error()).context(WindowsError("PostQueuedCompletionStatus failed"))?,
                _ => Ok(())
            }
        }
    }

    pub fn get(&self) -> Result<OutputOperation, Error> {
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
                v if v == 0 => Err(io::Error::last_os_error()).context(WindowsError("GetQueuedCompletionStatus failed"))?,
                _ => {
                    Ok(OutputOperation(OVERLAPPED_ENTRY {
                        dwNumberOfBytesTransferred: bytes_read,
                        lpCompletionKey: completion_key,
                        lpOverlapped: overlapped,
                        Internal: 0,
                    }))
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_many<'a>(&self, operations: &'a mut [OutputOperation]) -> Result<&'a mut [OutputOperation], Error> {
        let mut count = 0;
        let len = operations.len() as u32;
        match unsafe {
            GetQueuedCompletionStatusEx(
                self.0,
                operations.as_mut_ptr() as *mut _,
                len,
                &mut count,
                INFINITE,
                0,
            )
        } {
            v if v == 0 => Err(io::Error::last_os_error()).context(WindowsError("GetQueuedCompletionStatusEx failed"))?,
            _ => {
                Ok(&mut operations[..count as usize])
            }
        }
    }
}

impl Drop for IOCompletionPort {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use super::*;

    fn temp_file() -> PathBuf {
        let mut dir = env::temp_dir();
        dir.push("iocp_test");
        {
            let mut tmp_file = File::create(&dir).unwrap();
            write!(tmp_file, "hello world").unwrap();
        }
        dir
    }

    #[test]
    fn test_iocp_read() {
        let iocp = IOCompletionPort::new(1).unwrap();
        let file = iocp.associate_file(temp_file(), 42).unwrap();

        let operation = Box::new(InputOperation::new(vec![0u8; 20], 0, 0));
        IOCompletionPort::submit(&file, operation).unwrap();

        let output_operation = iocp.get().unwrap();
        let bytes_read = output_operation.0.dwNumberOfBytesTransferred as usize;

        assert_eq!(bytes_read, "hello world".as_bytes().len());
        assert_eq!(&output_operation.into_buffer()[..bytes_read], "hello world".as_bytes());
    }

    #[test]
    fn test_iocp_post() {
        let operation = Box::new(InputOperation::new(vec![], 0, 0));
        let iocp = IOCompletionPort::new(1).unwrap();

        iocp.post(operation, 42).unwrap();
        let output_operation = iocp.get().unwrap();

        assert_eq!(output_operation.completion_key(), 42);
        assert_eq!(output_operation.0.dwNumberOfBytesTransferred, 0);
    }
}
