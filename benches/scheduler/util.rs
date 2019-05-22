use thread_pool::util::Task;

pub trait Runner: Clone + Send + 'static{
    fn run(&self, key: usize, task: Task);
}
