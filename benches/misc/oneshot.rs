use futures::sync::oneshot as FutureOneshot;
use futures::{future, lazy, Future};
use std::thread;
use test;
use tokio_sync::oneshot as TokioOneshot;
use tokio_threadpool;

#[bench]
fn benchmark_future_oneshot(b: &mut test::Bencher) {
    b.iter(move || {
        let (tx, rx) = FutureOneshot::channel();

        tx.send(1).unwrap();
        rx.wait().unwrap();
    });
}

#[bench]
fn benchmark_tokio_oneshot(b: &mut test::Bencher) {
    b.iter(move || {
        let (tx, rx) = TokioOneshot::channel();

        tx.send(1).unwrap();
        rx.wait().unwrap();
    });
}

#[bench]
fn benchmark_future_oneshot_pool(b: &mut test::Bencher) {
    let pool = tokio_threadpool::Builder::new().pool_size(1).build();
    let sender = pool.sender();

    b.iter(move || {
        let (tx, rx) = FutureOneshot::channel();

        sender
            .spawn(lazy(|| {
                tx.send(1).unwrap();
                future::ok::<(), ()>(())
            }))
            .unwrap();

        rx.wait().unwrap();
    });

    pool.shutdown().wait().unwrap();
}

#[bench]
fn benchmark_tokio_oneshot_pool(b: &mut test::Bencher) {
    let pool = tokio_threadpool::Builder::new().pool_size(1).build();
    let sender = pool.sender();

    b.iter(move || {
        let (tx, rx) = TokioOneshot::channel();

        sender.spawn(lazy(|| {
            tx.send(1).unwrap();
            future::ok::<(), ()>(())
        })).unwrap();

        rx.wait().unwrap();
    });

    pool.shutdown().wait().unwrap();
}
