use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use thread_pool::util::{Spawner, Task};

struct State {
    queue: VecDeque<Task>,
    stopped: bool,
}

pub struct ThreadPool {
    handlers: Option<Vec<thread::JoinHandle<()>>>,
    notifer: Arc<(Mutex<State>, Condvar)>,
}

fn next_task(notifer: &Arc<(Mutex<State>, Condvar)>, timeout: Option<Duration>) -> Option<Task> {
    let &(ref lock, ref cvar) = &**notifer;
    let mut state = lock.lock().unwrap();
    loop {
        if state.stopped {
            return None;
        }
        match state.queue.pop_front() {
            Some(t) => {
                return Some(t);
            }
            None => {
                state = match timeout {
                    Some(t) => cvar.wait_timeout(state, t).unwrap().0,
                    None => cvar.wait(state).unwrap(),
                };
            }
        }
    }
}

impl ThreadPool {
    pub fn new(number: usize, timeout: Option<Duration>) -> ThreadPool {
        let mut handlers = vec![];
        let s = State {
            queue: VecDeque::with_capacity(1024),
            stopped: false,
        };
        let notifer = Arc::new((Mutex::new(s), Condvar::new()));
        for _ in 0..number {
            let notifer = notifer.clone();
            let handle = thread::spawn(move || {
                while let Some(task) = next_task(&notifer, timeout) {
                    task.call_box();
                }
            });

            handlers.push(handle);
        }

        ThreadPool {
            notifer: notifer,
            handlers: Some(handlers),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        let &(ref lock, ref cvar) = &*self.notifer;
        {
            let mut state = lock.lock().unwrap();
            state.stopped = true;
            cvar.notify_all();
        }

        let handlers = self.handlers.take().unwrap();
        for handle in handlers {
            handle.join().unwrap();
        }
    }
}

impl Spawner for ThreadPool {
    fn spawn<T: FnOnce() + Send + 'static>(&self, t: T) {
        let task = Box::new(t);
        let &(ref lock, ref cvar) = &*self.notifer;
        {
            let mut state = lock.lock().unwrap();
            state.queue.push_back(task);
            cvar.notify_one();
        }
    }
}
