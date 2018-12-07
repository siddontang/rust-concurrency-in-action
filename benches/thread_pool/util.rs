pub trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

pub type Task = Box<FnBox + Send>;

pub trait Spawner {
    fn spawn<T: FnOnce() + Send + 'static>(&self, t: T);
}
