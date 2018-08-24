use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{mpsc, Arc};

use num_cpus;
use test;

mod condvar;
mod crossbeam;
mod robin_round;
mod std_channel;

mod util;

use self::util::Spawner;

const NUM_SPAWN: usize = 10_000;

pub fn benchmark_thread_pool<T>(pool: T, b: &mut test::Bencher)
where
    T: Spawner,
{
    let (tx, rx) = mpsc::channel();
    let rem = Arc::new(AtomicUsize::new(0));
    b.iter(move || {
        rem.store(NUM_SPAWN, SeqCst);

        for _ in 0..NUM_SPAWN {
            let tx = tx.clone();
            let rem = rem.clone();

            pool.spawn(move || {
                if 1 == rem.fetch_sub(1, SeqCst) {
                    tx.send(()).unwrap();
                }
            })
        }

        let _ = rx.recv().unwrap();
    })
}

#[bench]
pub fn benchmark_std_channel_thread_pool(b: &mut test::Bencher) {
    let pool = std_channel::ThreadPool::new(num_cpus::get());
    benchmark_thread_pool(pool, b);
}

#[bench]
pub fn benchmark_crossbeam_channel_thread_pool(b: &mut test::Bencher) {
    let pool = crossbeam::ThreadPool::new(num_cpus::get());
    benchmark_thread_pool(pool, b);
}

#[bench]
pub fn benchmark_condvar_thread_pool(b: &mut test::Bencher) {
    let pool = condvar::ThreadPool::new(num_cpus::get());
    benchmark_thread_pool(pool, b);
}

#[bench]
pub fn benchmark_robin_round_thread_pool(b: &mut test::Bencher) {
    let pool = robin_round::ThreadPool::new(num_cpus::get());
    benchmark_thread_pool(pool, b);
}
