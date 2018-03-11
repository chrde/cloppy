pub struct BufferPool {
    buffer_size: usize,
//    pool: Vec<Vec<u8>>,
}

impl BufferPool {
    pub fn new(capacity: usize, buffer_size: usize) -> Self {
//        let buffer = vec![0; buffer_size];
        BufferPool {
//            pool: vec![buffer; capacity],
            buffer_size,
        }
    }

//    pub fn len(&self) -> usize {
//        self.pool.len()
//    }

    pub fn get(&mut self) -> Option<Vec<u8>> {
//        if self.pool.is_empty() {
        Some(vec![0; self.buffer_size])
//        } else {
//            self.pool.pop()
//        }
    }

    pub fn put(&mut self, buf: Vec<u8>) {
//        self.pool.push(buf)
    }
}

impl Drop for BufferPool {
    fn drop(&mut self) {
        println!("dropping pool");
    }
}