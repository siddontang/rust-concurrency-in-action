use crossbeam::channel::{self, Sender};
use std::thread;

use thread_pool::util::{Spawner, Task};

pub struct ThreadPool {
    tx: Option<Sender<Task>>,
    handlers: Option<Vec<thread::JoinHandle<()>>>,
}
impl ThreadPool {
    pub fn new(number: usize) -> ThreadPool {
        let (tx, rx) = channel::unbounded::<Task>();
        let mut handlers = vec![];

        for _ in 0..number {
            let rx = rx.clone();
            let handle = thread::spawn(move || {
                while let Ok(task) = rx.recv() {
                    task.call_box();
                }
            });

            handlers.push(handle);
        }

        ThreadPool {
            tx: Some(tx),
            handlers: Some(handlers),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        let tx = self.tx.take();
        drop(tx);

        let handlers = self.handlers.take().unwrap();
        for handle in handlers {
            handle.join().unwrap();
        }
    }
}

impl Spawner for ThreadPool {
    fn spawn<T: FnOnce() + Send + 'static>(&self, t: T) {
        self.tx.as_ref().unwrap().send(Box::new(t));
    }
}
