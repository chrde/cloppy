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
use ntfs::file_entry::FileEntry;
use ntfs::volume_data::VolumeData;

//TODO make this value 'smart' depending on the HD
const SPEED_FACTOR: u64 = 4 * 16;


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

    pub fn read_all(&mut self, mft: &FileEntry, volume_data: VolumeData) {
        use std::time::Instant;
        let now = Instant::now();
        let mut absolute_lcn_offset = 0i64;
        for (i, run) in mft.dataruns.iter().enumerate() {
            absolute_lcn_offset += run.offset_lcn;
            let absolute_offset = absolute_lcn_offset as u64 * volume_data.bytes_per_cluster as u64;
            let mut file_record_count = run.length_lcn * volume_data.clusters_per_fr() as u64;
            println!("datarun {} started", file_record_count);

            let full_runs = file_record_count / SPEED_FACTOR;
            //TODO debug this
            let _partial_run_size = file_record_count % SPEED_FACTOR;
            for run in 0..full_runs {
                let offset = absolute_offset + SPEED_FACTOR * run * volume_data.bytes_per_file_record as u64;
                self.read(offset).unwrap();
            }
            let offset = absolute_offset + SPEED_FACTOR * (full_runs - 1) * volume_data.bytes_per_file_record as u64;
            self.read(offset).unwrap();
            println!("datarun {} finished. Partial time {:?}", i, Instant::now().duration_since(now));
        }
        println!("total time {:?}", Instant::now().duration_since(now));
        self.finish();
    }
}

