use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering, ATOMIC_BOOL_INIT, ATOMIC_USIZE_INIT};
use std::thread;

static X: AtomicBool = ATOMIC_BOOL_INIT;
static Y: AtomicBool = ATOMIC_BOOL_INIT;
static Z: AtomicUsize = ATOMIC_USIZE_INIT;

fn write_x_then_y() {
    X.store(true, Ordering::Relaxed);
    Y.store(true, Ordering::Relaxed);
}

fn read_y_then_x() {
    while !Y.load(Ordering::Relaxed) {}
    if X.load(Ordering::Relaxed) {
        Z.fetch_add(1, Ordering::SeqCst);
    }
}

fn main() {
    X.store(false, Ordering::SeqCst);
    Y.store(false, Ordering::SeqCst);
    Z.store(0, Ordering::SeqCst);

    let t1 = thread::spawn(move || {
        write_x_then_y();
    });

    let t2 = thread::spawn(move || {
        read_y_then_x();
    });

    t1.join().unwrap();
    t2.join().unwrap();

    assert_ne!(Z.load(Ordering::SeqCst), 0);
}
