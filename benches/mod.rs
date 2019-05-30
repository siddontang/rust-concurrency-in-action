#![feature(test)]

extern crate crossbeam;
extern crate futures;
extern crate futures_cpupool;
extern crate fxhash;
extern crate num_cpus;
extern crate spin;
extern crate test;
extern crate tokio_threadpool;
extern crate parking_lot;

mod scheduler;
mod thread_pool;
