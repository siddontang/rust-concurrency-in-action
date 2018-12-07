use std::collections::VecDeque;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Mutex};
use std::usize;

pub use fxhash::FxHashMap as HashMap;

use thread_pool::crossbeam::ThreadPool;
use thread_pool::util::{Spawner, Task};
use crossbeam::MsQueue;

struct Latch {
    pub waiting: MsQueue<(usize, Task)>,
}

impl Latch {
    pub fn new() -> Latch {
        let (worker, _) = deque::fifo::<(usize, Task)>();
        Latch {
            waiting: worker,
        }
    }
}

struct Latches {
    slots: Vec<Latch>,
    size: usize,
}

impl Latches {
    pub fn new(size: usize) -> Latches {
        let power_of_two_size = usize::next_power_of_two(size);
        let mut slots = Vec::with_capacity(power_of_two_size);
        for _ in 0..power_of_two_size {
            slots.push(Arc::new(Latch::new()));
        }
        Latches {
            slots: slots,
            size: power_of_two_size,
        }
    }

    pub fn acquire(&self, key: usize, who: usize, task: Task) -> bool {
        let key = key & (self.size - 1);
        let mut latch = &self.slots[key];

        let empty = latch.waiting.is_empty();

        latch.waiting.push((who, task));

        empty
    }

    pub fn release(&self, key: usize, _: usize) {
        let key = key & (self.size - 1);

        let mut latch = &self.slots[key];
        while let  deque::Pop(front) = latch.waiting.pop() {

        front.1.call_box();
        }
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

    pub fn run(&self, key: usize, task: Task) {
        let id = self.gen_id();
        let latches = self.latches.clone();
        if latches.acquire(key, id, task) {
            self.pool.spawn(move || {
                latches.release(key, id);
            })
        }
    }
}
