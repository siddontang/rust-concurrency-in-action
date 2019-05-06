use futures::{future, lazy, Future};
use futures_cpupool::CpuPool;
use thread_pool::util::{Spawner, Task};

pub struct ThreadPool {
    pool: CpuPool,
}
impl ThreadPool {
    pub fn new(number: usize) -> ThreadPool {
        let pool = CpuPool::new(number);
        ThreadPool { pool: pool }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {}
}

impl Spawner for ThreadPool {
    fn spawn<T: FnOnce() + Send + 'static>(&self, t: T) {
        let f = self.pool.spawn(lazy(|| {
            t();
            future::ok::<(), ()>(())
        }));
        f.forget();
    }
}
