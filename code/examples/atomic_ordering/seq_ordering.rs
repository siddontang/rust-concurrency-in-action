use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering, ATOMIC_BOOL_INIT, ATOMIC_USIZE_INIT};
use std::thread;

static X: AtomicBool = ATOMIC_BOOL_INIT;
static Y: AtomicBool = ATOMIC_BOOL_INIT;
static Z: AtomicUsize = ATOMIC_USIZE_INIT;

fn write_x() {
    X.store(true, Ordering::SeqCst);
}

fn write_y() {
    Y.store(true, Ordering::SeqCst);
}

fn read_x_then_y() {
    while !X.load(Ordering::SeqCst) {}
    if Y.load(Ordering::SeqCst) {
        Z.fetch_add(1, Ordering::SeqCst);
    }
}

fn read_y_then_x() {
    while !Y.load(Ordering::SeqCst) {}
    if X.load(Ordering::SeqCst) {
        Z.fetch_add(1, Ordering::SeqCst);
    }
}

fn main() {
    X.store(false, Ordering::SeqCst);
    Y.store(false, Ordering::SeqCst);
    Z.store(0, Ordering::SeqCst);

    let t1 = thread::spawn(move || {
        write_x();
    });

    let t2 = thread::spawn(move || {
        write_y();
    });

    let t3 = thread::spawn(move || {
        read_x_then_y();
    });

    let t4 = thread::spawn(move || {
        read_y_then_x();
    });

    t1.join().unwrap();
    t2.join().unwrap();
    t3.join().unwrap();
    t4.join().unwrap();

    assert_ne!(Z.load(Ordering::SeqCst), 0);
}
