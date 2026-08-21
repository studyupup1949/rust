use crate::newrelic_fn::{
    nr_end_custom_segment, nr_end_transaction, nr_start_custom_segment, nr_start_web_transaction,
};
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;

fn run_parallel_thread(worker: usize, jobs: usize) {
    let pool = ThreadPool::new(worker);
    for _ in 0..jobs {
        //println!("{}", x);
        thread::sleep(Duration::from_millis(100));
        pool.execute(move || {
            nr_start_web_transaction("/api/test");
            let segment = nr_start_custom_segment("pool_execute");
            thread::sleep(Duration::from_secs(1));
            nr_end_custom_segment(segment);
            //println!("Thread Id {:?}", thread::current().id());
            nr_end_transaction();
        });
    }
}

// Load testing

#[test]
fn test_thread_pool() {
    let pool = ThreadPool::new(2);
    for _ in 0..100 {
        thread::sleep(Duration::from_millis(100));
        pool.execute(move || run_parallel_thread(2, 1000));
    }
    //    let pool = ThreadPool::new(worker);
    //    for _ in 0..jobs {
    //        pool.execute(move || {
    //            println!("{:?}", thread::current().id());
    //        });
    //    }
}
