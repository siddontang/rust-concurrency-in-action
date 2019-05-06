use futures::{future, lazy, Future};
use thread_pool::util::{Spawner, Task};
use tokio_threadpool;

pub struct ThreadPool {
    pool: Option<tokio_threadpool::ThreadPool>,
}
impl ThreadPool {
    pub fn new(number: usize) -> ThreadPool {
        let pool = tokio_threadpool::Builder::new().pool_size(number).build();
        ThreadPool { pool: Some(pool) }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        let pool = self.pool.take().unwrap();
        pool.shutdown().wait().unwrap();
    }
}

impl Spawner for ThreadPool {
    fn spawn<T: FnOnce() + Send + 'static>(&self, t: T) {
        self.pool.as_ref().unwrap().spawn(lazy(|| {
            t();
            future::ok::<(), ()>(())
        }));
    }
}
