use std::io;
use std::mem;
use std::ptr;
use std::slice;
use std::os::windows::io::AsRawHandle;
use winapi::um::winbase::INFINITE;
use winapi::um::minwinbase::{
    OVERLAPPED,
    OVERLAPPED_ENTRY,
};
use windows::utils;
use winapi::um::winnt::HANDLE;
use winapi::um::ioapiset::{
    CreateIoCompletionPort,
    GetQueuedCompletionStatus,
    GetQueuedCompletionStatusEx,
    PostQueuedCompletionStatus,
};
use winapi::um::handleapi::{
    INVALID_HANDLE_VALUE,
    CloseHandle,
};
use errors::MyErrorKind::*;
use failure::{err_msg, Error, ResultExt};
use winapi::shared::basetsd::ULONG_PTR;
use windows::read_overlapped;
use std::os::windows::fs::OpenOptionsExt;
use winapi::um::winbase::FILE_FLAG_OVERLAPPED;
use std::path::Path;
use std::fs::{
    File,
    OpenOptions,
};
use errors::MyErrorKind::WindowsError;

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

pub struct OutputOperation(OVERLAPPED_ENTRY);

impl OutputOperation {
    pub fn zero() -> Self {
        OutputOperation(OVERLAPPED_ENTRY {
            dwNumberOfBytesTransferred: 0,
            lpCompletionKey: 0 as ULONG_PTR,
            lpOverlapped: ptr::null_mut(),
            Internal: 0,
        })
    }

    pub fn buffer_mut(&mut self) -> &mut [u8] {
        unsafe {
            let op = &mut *(self.0.lpOverlapped as *mut InputOperation);
            slice::from_raw_parts_mut(op.buffer, op.len)
        }
    }

    pub fn into_buffer(self) -> Vec<u8> {
        unsafe {
            let op = Box::from_raw(self.0.lpOverlapped as *mut InputOperation);
            Vec::from_raw_parts(op.buffer, op.len, op.capacity)
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
    pub fn new(mut buffer: Vec<u8>, offset: u64) -> Self {
        let mut overlapped;
        unsafe {
            overlapped = mem::zeroed::<OVERLAPPED>();
            let s = overlapped.u.s_mut();
            s.Offset = offset as u32;
            s.OffsetHigh = (offset >> 32) as u32;
        };
        let res = InputOperation {
            overlapped,
            buffer: buffer.as_mut_ptr(),
            len: buffer.len(),
            capacity: buffer.capacity(),
        };
        ::std::mem::forget(buffer);
        res
    }
}

impl IOCompletionPort {
    pub fn submit(file: &AsyncFile, operation: Box<InputOperation>) -> io::Result<()> {
        let length = operation.len as u32;
        let lp_buffer = operation.buffer;
        let lp_overlapped = Box::into_raw(operation);
        read_overlapped(&file.file, lp_buffer, length, lp_overlapped as *mut _)
    }

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

    pub fn associate_file<P: AsRef<Path>>(&self, file_path: P, completion_key: usize) -> io::Result<AsyncFile> {
        let file = OpenOptions::new().read(true).custom_flags(FILE_FLAG_OVERLAPPED).open(file_path)
            .map(|file| AsyncFile { file, completion_key }).unwrap();
        unsafe {
            match CreateIoCompletionPort(
                file.file.as_raw_handle(),
                self.0,
                file.completion_key,
                0,
            ) {
                v if v.is_null() => utils::last_error(),
                _ => Ok(file)
            }
        }
    }

    pub fn post(&self, operation: Box<InputOperation>, completion_key: usize) -> io::Result<()> {
        let lp_overlapped = Box::into_raw(operation) as *mut _;
        unsafe {
            match PostQueuedCompletionStatus(
                self.0,
                0,
                completion_key as ULONG_PTR,
                lp_overlapped,
            ) {
                v if v == 0 => utils::last_error(),
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

    pub fn get_many<'a>(&self, operations: &'a mut [OutputOperation1]) -> Result<&'a mut [OutputOperation1], Error> {
        let mut count = 0;
        let mut completion_key = 0;
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
            v if v == 0 => utils::last_error().context(WindowsError("IOCP - get_many"))?,
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
    use super::*;
    use std::io::Write;
    use std::fs::File;
    use std::env;
    use windows::read_overlapped;
    use std::path::PathBuf;

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
        let mut iocp = IOCompletionPort::new(1).unwrap();
        let file = iocp.associate_file(temp_file(), 42).unwrap();

        let operation = Box::new(InputOperation::new(vec![0u8; 20], 0));
        IOCompletionPort::submit(&file, operation).unwrap();
        let output_operation = iocp.get().unwrap();

        assert_eq!(output_operation.completion_key, 42);
        assert_eq!(output_operation.bytes_read, "hello world".as_bytes().len() as u32);
        assert_eq!(&output_operation.buffer[..output_operation.bytes_read as usize], "hello world".as_bytes());
    }

    #[test]
    fn test_iocp_post() {
        let operation = Box::new(InputOperation::new(vec![], 0));
        let mut iocp = IOCompletionPort::new(1).unwrap();

        iocp.post(operation, 42).unwrap();
        let output_operation = iocp.get().unwrap();

        assert_eq!(output_operation.completion_key, 42);
        assert_eq!(output_operation.bytes_read, 0);
    }
}
