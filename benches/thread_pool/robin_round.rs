use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::thread;

use crossbeam::channel::{self, Sender};
use thread_pool::util::{Spawner, Task};

pub struct ThreadPool {
    count: AtomicUsize,
    txs: Option<Vec<Sender<Task>>>,
    handlers: Option<Vec<thread::JoinHandle<()>>>,
}
impl ThreadPool {
    pub fn new(number: usize) -> ThreadPool {
        let mut handlers = vec![];
        let mut txs = vec![];

        for _ in 0..number {
            let (tx, rx) = channel::unbounded::<Task>();
            let handle = thread::spawn(move || {
                while let Some(task) = rx.recv() {
                    task.call_box();
                }
            });

            txs.push(tx);
            handlers.push(handle);
        }

        ThreadPool {
            count: ATOMIC_USIZE_INIT,
            txs: Some(txs),
            handlers: Some(handlers),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        let txs = self.txs.take();
        for tx in txs {
            drop(tx);
        }

        let handlers = self.handlers.take().unwrap();
        for handle in handlers {
            handle.join().unwrap();
        }
    }
}

impl Spawner for ThreadPool {
    fn spawn<T: FnOnce() + Send + 'static>(&self, t: T) {
        let n = self.count.fetch_add(1, Ordering::Relaxed);
        let txs = self.txs.as_ref().unwrap();
        let tx = txs.get(n % txs.len());
        tx.as_ref().unwrap().send(Box::new(t));
    }
}
