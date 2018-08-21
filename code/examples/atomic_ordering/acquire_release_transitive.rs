use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering, ATOMIC_BOOL_INIT, ATOMIC_USIZE_INIT};
use std::thread;

static SYNC1: AtomicBool = ATOMIC_BOOL_INIT;
static SYNC2: AtomicBool = ATOMIC_BOOL_INIT;
static DATA: [AtomicUsize; 3] = [ATOMIC_USIZE_INIT, ATOMIC_USIZE_INIT, ATOMIC_USIZE_INIT];

fn thread_1() {
    DATA[0].store(100, Ordering::Relaxed);
    DATA[1].store(0, Ordering::Relaxed);
    DATA[2].store(200, Ordering::Relaxed);
    SYNC1.store(true, Ordering::Release);
}

fn thread_2() {
    while !SYNC1.load(Ordering::Acquire) {}
    SYNC2.store(true, Ordering::Release);
}

fn thread_3() {
    while !SYNC2.load(Ordering::Acquire) {}
    assert_eq!(DATA[2].load(Ordering::Relaxed), 200);
    assert_eq!(DATA[1].load(Ordering::Relaxed), 0);
    assert_eq!(DATA[0].load(Ordering::Relaxed), 100);
}

fn main() {
    SYNC1.store(false, Ordering::SeqCst);
    SYNC2.store(false, Ordering::SeqCst);

    let t1 = thread::spawn(move || {
        thread_1();
    });
    let t2 = thread::spawn(move || {
        thread_2();
    });
    let t3 = thread::spawn(move || {
        thread_3();
    });

    t1.join().unwrap();
    t2.join().unwrap();
    t3.join().unwrap();
}
