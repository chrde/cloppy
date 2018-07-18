use windows::async_io::{
    AsyncFile,
    BufferPool,
    IOCompletionPort,
    InputOperation,
};
use std::io;
use std::sync::{
    Arc,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::path::Path;
use ntfs::file_record::FileRecord;
use ntfs::volume_data::VolumeData;
use ntfs::FR_AT_ONCE;

pub struct MftReader {
    pool: BufferPool,
    iocp: Arc<IOCompletionPort>,
    file: AsyncFile,
    counter: Arc<AtomicUsize>,
}

impl MftReader {
    pub fn new<P: AsRef<Path>>(pool: BufferPool, iocp: Arc<IOCompletionPort>, file_path: P, completion_key: usize, counter: Arc<AtomicUsize>) -> Self {
        let file = iocp.associate_file(file_path, completion_key).unwrap();
        MftReader { file, iocp, pool, counter }
    }


    pub fn finish(&self) -> io::Result<()>  {
        let operation = Box::new(InputOperation::empty());
        self.counter.fetch_add(1, Ordering::SeqCst);
        self.iocp.post(operation, 99)
    }

    pub fn read(&mut self, offset: u64, content_len: usize) -> io::Result<()> {
        let buffer = self.pool.get();
        let operation = Box::new(InputOperation::new(buffer, offset, content_len));
        self.counter.fetch_add(1, Ordering::SeqCst);
        IOCompletionPort::submit(&self.file, operation)
    }

    pub fn read_all(&mut self, mft: &FileRecord, volume_data: VolumeData) {
        use std::time::Instant;
        let now = Instant::now();
        let mut absolute_lcn_offset = 0i64;
        for (i, run) in mft.data_attr.datarun.iter().enumerate() {
            absolute_lcn_offset += run.offset_lcn;
            let absolute_offset = absolute_lcn_offset as u64 * volume_data.bytes_per_cluster as u64;
            let mut file_record_count = run.length_lcn * volume_data.clusters_per_fr() as u64;
            println!("datarun {} started", file_record_count);

            let full_runs_count = file_record_count / FR_AT_ONCE;
            let partial_run_size = file_record_count % FR_AT_ONCE;
            for run in 0..full_runs_count {
                let offset = absolute_offset + run * FR_AT_ONCE * volume_data.bytes_per_file_record as u64;
                self.read(offset, FR_AT_ONCE as usize).unwrap();
            }
            if partial_run_size > 0 {
                let offset = absolute_offset + full_runs_count * FR_AT_ONCE * volume_data.bytes_per_file_record as u64;
                self.read(offset, partial_run_size as usize).unwrap();
            }
            println!("datarun {} finished. Partial time {:?}", i, Instant::now().duration_since(now));
        }
        println!("total time {:?}", Instant::now().duration_since(now));
        self.finish().unwrap();
    }
}

