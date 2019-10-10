use crossbeam::channel::{self, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

use thread_pool::util::{Spawner, Task};

pub struct ThreadPool {
    tx: Option<Sender<Task>>,
    handlers: Option<Vec<thread::JoinHandle<()>>>,
}
impl ThreadPool {
    pub fn new(number: usize) -> ThreadPool {
        ThreadPool::new_with_timeout(number, None)
    }

    pub fn new_with_timeout(number: usize, timeout: Option<Duration>) -> ThreadPool {
        let (tx, rx) = channel::unbounded::<Task>();
        let mut handlers = vec![];

        for _ in 0..number {
            let rx = rx.clone();
            let handle = thread::spawn(move || loop {
                match timeout {
                    Some(t) => match rx.recv_timeout(t) {
                        Ok(task) => task.call_box(),
                        Err(RecvTimeoutError::Timeout) => {}
                        Err(_) => break,
                    },
                    None => match rx.recv() {
                        Ok(task) => task.call_box(),
                        Err(_) => break,
                    },
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
        self.tx.as_ref().unwrap().send(Box::new(t)).unwrap();
    }
}
