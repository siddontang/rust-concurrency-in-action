#![feature(test)]

extern crate crossbeam;
extern crate futures;
extern crate futures_cpupool;
extern crate fxhash;
extern crate num_cpus;
extern crate parking_lot;
extern crate spin;
extern crate test;
extern crate tokio_sync;
extern crate tokio_threadpool;

mod misc;
mod scheduler;
mod thread_pool;
