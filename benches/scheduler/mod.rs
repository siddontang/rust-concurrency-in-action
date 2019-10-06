use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use std::thread;

use crossbeam::channel::unbounded;
use num_cpus;
use test;

mod channel;
mod lock;
mod parking_lot;
mod spinlock;
mod util;

use self::util::Runner;

const SLOT_NUM: usize = 4096;

fn benchmark_scheduler<R: Runner>(b: &mut test::Bencher, r: R) {
    let (tx, rx) = unbounded();

    let rem = Arc::new(AtomicUsize::new(0));
    let num = num_cpus::get();

    b.iter(move || {
        rem.store(SLOT_NUM * 4, SeqCst);

        for i in 0..num {
            let rem = rem.clone();
            let tx = tx.clone();
            let r = r.clone();
            thread::spawn(move || {
                for j in 0..SLOT_NUM {
                    let tx = tx.clone();
                    let rem = rem.clone();

                    let task = Box::new(move || {
                        if 1 == rem.fetch_sub(1, SeqCst) {
                            tx.send(()).unwrap();
                        }
                    });
                    r.run(i * SLOT_NUM + j, task);
                }
            });
        }

        let _ = rx.recv().unwrap();
    })
}

#[bench]
pub fn benchmark_scheduler_channel(b: &mut test::Bencher) {
    let mut scheduler = channel::Scheduler::new(SLOT_NUM);
    let sender = scheduler.sender.clone();
    thread::spawn(move || {
        scheduler.run();
    });

    benchmark_scheduler(b, sender);
}

#[bench]
pub fn benchmark_scheduler_lock(b: &mut test::Bencher) {
    let scheduler = Arc::new(lock::Scheduler::new(SLOT_NUM));
    benchmark_scheduler(b, scheduler);
}

#[bench]
pub fn benchmark_scheduler_spinlock(b: &mut test::Bencher) {
    let scheduler = Arc::new(spinlock::Scheduler::new(SLOT_NUM));
    benchmark_scheduler(b, scheduler);
}

#[bench]
pub fn benchmark_scheduler_parking_log(b: &mut test::Bencher) {
    let scheduler = Arc::new(parking_lot::Scheduler::new(SLOT_NUM));
    benchmark_scheduler(b, scheduler);
}
