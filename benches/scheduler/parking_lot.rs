use std::collections::VecDeque;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc};
use std::usize;

pub use fxhash::FxHashMap as HashMap;
use parking_lot::Mutex;

use scheduler::util::Runner;
use thread_pool::crossbeam::ThreadPool;
use thread_pool::util::{Spawner, Task};

struct Latch {
    pub waiting: VecDeque<(usize, Task)>,
}

impl Latch {
    pub fn new() -> Latch {
        Latch {
            waiting: VecDeque::new(),
        }
    }
}

struct Latches {
    slots: Vec<Arc<Mutex<Latch>>>,
    size: usize,
}

impl Latches {
    pub fn new(size: usize) -> Latches {
        let power_of_two_size = usize::next_power_of_two(size);
        let mut slots = Vec::with_capacity(power_of_two_size);
        for _ in 0..power_of_two_size {
            slots.push(Arc::new(Mutex::new(Latch::new())));
        }
        Latches {
            slots: slots,
            size: power_of_two_size,
        }
    }

    pub fn acquire(&self, key: usize, who: usize, task: Task) -> bool {
        let key = key & (self.size - 1);
        let mut latch = (&self.slots[key]).lock();

        let empty = latch.waiting.len() == 0;

        latch.waiting.push_back((who, task));

        empty
    }

    pub fn release(&self, key: usize, who: usize) -> Option<usize> {
        let key = key & (self.size - 1);

        let mut latch = (&self.slots[key]).lock();
        let front = latch.waiting.pop_front().unwrap();
        assert_eq!(front.0, who);

        front.1.call_box();

        latch.waiting.front().as_ref().map(|(key, _)| *key)
    }
}

pub struct Scheduler {
    latches: Arc<Latches>,

    id_alloc: AtomicUsize,

    pool: Arc<ThreadPool>,
}

impl Scheduler {
    pub fn new(count: usize) -> Self {
        Scheduler {
            latches: Arc::new(Latches::new(count)),
            id_alloc: AtomicUsize::new(0),
            pool: Arc::new(ThreadPool::new(4)),
        }
    }

    #[inline]
    fn gen_id(&self) -> usize {
        self.id_alloc.fetch_add(1, Relaxed)
    }
}

impl Runner for Arc<Scheduler> {
    fn run(&self, key: usize, task: Task) {
        let id = self.gen_id();
        let latches = self.latches.clone();
        if latches.acquire(key, id, task) {
            self.pool.spawn(move || {
                let mut who = id;
                while let Some(pending) = latches.release(key, who) {
                    who = pending;
                }
            })
        }
    }
}
