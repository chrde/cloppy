use parking_lot::{Mutex, Condvar};
use std::sync::Arc;

#[derive(Clone)]
pub struct BufferPool(Arc<(Mutex<Inner>, Condvar)>);

struct Inner {
    pool: Vec<Vec<u8>>,

}

impl BufferPool {
    pub fn new(capacity: usize, buffer_size: usize) -> Self {
        let pool = vec![0; buffer_size];
        let inner = Inner { pool: vec![pool; capacity] };
        BufferPool(Arc::new((Mutex::new(inner), Condvar::new())))
    }

    pub fn get(&mut self) -> Vec<u8> {
        let &(ref lock, ref cond) = &*self.0;
        let mut guard = lock.lock();
        if (*guard).pool.is_empty() {
            cond.wait(&mut guard);
        }
        (*guard).pool.pop().expect("with mutex, we can't have an empty vec")
    }

    pub fn put(&mut self, buf: Vec<u8>) {
        let &(ref lock, ref cond) = &*self.0;
        let mut guard = lock.lock();
        (*guard).pool.push(buf);
        cond.notify_one();
    }
}
