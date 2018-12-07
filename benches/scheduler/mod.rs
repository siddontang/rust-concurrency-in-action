use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use std::thread;

use crossbeam::channel::unbounded;
use test;
mod channel;
mod lock;
// mod lockfree;

const SLOT_NUM: usize = 4096;

#[bench]
pub fn benchmark_scheduler_channel(b: &mut test::Bencher) {
    let mut scheduler = channel::Scheduler::new(SLOT_NUM);
    let sender = scheduler.sender.clone();
    thread::spawn(move || {
        scheduler.run();
    });

    let (tx, rx) = unbounded();

    let rem = Arc::new(AtomicUsize::new(0));

    b.iter(move || {
        rem.store(SLOT_NUM * 4, SeqCst);

        for i in 0..4 {
            let rem = rem.clone();
            let tx = tx.clone();
            let sender = sender.clone();
            thread::spawn(move || {
                for j in 0..SLOT_NUM {
                    let tx = tx.clone();
                    let rem = rem.clone();

                    let task = Box::new(move || {
                        if 1 == rem.fetch_sub(1, SeqCst) {
                            tx.send(());
                        }
                    });
                    sender.send(channel::Cmd::Request {
                        key: i * SLOT_NUM + j,
                        task: task,
                    });
                }
            });
        }

        let _ = rx.recv().unwrap();
    })
}

#[bench]
pub fn benchmark_scheduler_lock(b: &mut test::Bencher) {
    let scheduler = Arc::new(lock::Scheduler::new(SLOT_NUM));

    let (tx, rx) = unbounded();

    let rem = Arc::new(AtomicUsize::new(0));

    b.iter(move || {
        rem.store(SLOT_NUM * 4, SeqCst);

        for i in 0..4 {
            let rem = rem.clone();
            let tx = tx.clone();
            let scheduler = scheduler.clone();
            thread::spawn(move || {
                for j in 0..SLOT_NUM {
                    let tx = tx.clone();
                    let rem = rem.clone();

                    let task = Box::new(move || {
                        if 1 == rem.fetch_sub(1, SeqCst) {
                            tx.send(());
                        }
                    });
                    scheduler.run(i * SLOT_NUM + j, task);
                }
            });
        }

        let _ = rx.recv().unwrap();
    })
}
