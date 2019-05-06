use std::collections::VecDeque;
use std::usize;

pub use fxhash::FxHashMap as HashMap;
pub use std::collections::hash_map::Entry as HashMapEntry;

use crossbeam::channel::{self, Receiver, Sender};

use thread_pool::crossbeam::ThreadPool;
use thread_pool::util::{Spawner, Task};

#[derive(Clone)]
struct Latch {
    pub waiting: VecDeque<usize>,
}

impl Latch {
    pub fn new() -> Latch {
        Latch {
            waiting: VecDeque::new(),
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
        Latches {
            slots: vec![Latch::new(); power_of_two_size],
            size: power_of_two_size,
        }
    }

    pub fn acquire(&mut self, key: usize, who: usize) -> bool {
        let mut acquired: bool = false;
        let key = key & (self.size - 1);
        let latch = &mut self.slots[key];

        let front = latch.waiting.front().cloned();
        match front {
            Some(cid) => {
                if cid == who {
                    acquired = true;
                } else {
                    latch.waiting.push_back(who);
                }
            }
            None => {
                latch.waiting.push_back(who);
                acquired = true;
            }
        }

        acquired
    }

    pub fn release(&mut self, key: usize, who: usize) -> Option<usize> {
        let key = key & (self.size - 1);

        let latch = &mut self.slots[key];
        let front = latch.waiting.pop_front().unwrap();
        assert_eq!(front, who);

        latch.waiting.front().cloned()
    }
}

pub enum Cmd {
    Request { key: usize, task: Task },
    Finished { key: usize, who: usize },
}

struct TaskContext {
    key: usize,
    task: Task,
}

impl TaskContext {
    pub fn new(key: usize, task: Task) -> Self {
        TaskContext {
            key: key,
            task: task,
        }
    }
}

pub struct Scheduler {
    tasks: HashMap<usize, TaskContext>,

    pub sender: Sender<Cmd>,
    receiver: Receiver<Cmd>,

    latches: Latches,

    id_alloc: usize,

    pool: ThreadPool,
}

impl Scheduler {
    pub fn new(count: usize) -> Self {
        let (sender, receiver) = channel::unbounded();
        Scheduler {
            tasks: Default::default(),
            sender: sender,
            receiver: receiver,
            latches: Latches::new(count),
            id_alloc: 0,
            pool: ThreadPool::new(4),
        }
    }

    #[inline]
    fn gen_id(&mut self) -> usize {
        self.id_alloc += 1;
        self.id_alloc
    }

    pub fn run(&mut self) {
        while let Ok(res) = self.receiver.recv() {
            match res {
                Cmd::Request { key, task } => {
                    let id = self.gen_id();
                    let sender = self.sender.clone();
                    self.tasks.insert(id, TaskContext::new(key, task));
                    if self.latches.acquire(key, id) {
                        self.pool.spawn(move || {
                            sender.send(Cmd::Finished { key: key, who: id });
                        });
                    }
                }
                Cmd::Finished { key, who } => {
                    let ctx = self.tasks.remove(&who).unwrap();
                    ctx.task.call_box();

                    if let Some(pending) = self.latches.release(key, who) {
                        let pending_key = self.tasks.get(&pending).unwrap().key;
                        let sender = self.sender.clone();
                        self.pool.spawn(move || {
                            sender.send(Cmd::Finished {
                                key: pending_key,
                                who: pending,
                            });
                        });
                    }
                }
            }
        }
    }
}
